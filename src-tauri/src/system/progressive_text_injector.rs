// 渐进式文本注入器 - 如输入法般的实时文本注入体验
// Week 2 核心组件：随着语音转录实时注入文本

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};
use serde::{Deserialize, Serialize};

use crate::audio::streaming_transcriptor::TranscriptionEvent;
use crate::system::{TextInjector, TextInjectionConfig, ApplicationInfo};
use crate::errors::{AppError, AppResult};

/// 渐进式注入配置
#[derive(Debug, Clone)]
pub struct ProgressiveInjectionConfig {
    /// 是否启用渐进式注入
    pub enabled: bool,
    /// 最小注入字符数（低于此数量不注入）
    pub min_inject_length: usize,
    /// 注入间隔（毫秒）
    pub inject_interval_ms: u64,
    /// 最大待注入队列长度
    pub max_queue_length: usize,
    /// 是否启用退格删除旧文本
    pub enable_backspace_correction: bool,
    /// 置信度阈值（低于此值不注入）
    pub min_confidence_threshold: f64,
    /// 是否只在最终结果时注入
    pub final_only: bool,
    /// 智能合并相同前缀
    pub smart_prefix_merging: bool,
}

impl Default for ProgressiveInjectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_inject_length: 2,
            inject_interval_ms: 200, // 200ms间隔，平衡响应性和性能
            max_queue_length: 50,
            enable_backspace_correction: true,
            min_confidence_threshold: 0.65,
            final_only: false, // 启用实时注入体验
            smart_prefix_merging: true,
        }
    }
}

/// 注入队列条目
#[derive(Debug, Clone)]
struct InjectionItem {
    pub text: String,
    pub is_partial: bool,
    pub confidence: f64,
    pub timestamp: Instant,
    pub should_replace_previous: bool,
}

/// 渐进式文本注入器
pub struct ProgressiveTextInjector {
    config: ProgressiveInjectionConfig,
    text_injector: TextInjector,
    injection_queue: Arc<Mutex<VecDeque<InjectionItem>>>,
    last_injected_text: Arc<Mutex<String>>,
    target_app: Arc<Mutex<Option<ApplicationInfo>>>,
    is_active: Arc<Mutex<bool>>,
    transcription_receiver: Option<broadcast::Receiver<TranscriptionEvent>>,
}

impl ProgressiveTextInjector {
    /// 创建新的渐进式文本注入器
    pub fn new(
        config: ProgressiveInjectionConfig,
        injection_config: TextInjectionConfig,
    ) -> Self {
        Self {
            config,
            text_injector: TextInjector::new(injection_config),
            injection_queue: Arc::new(Mutex::new(VecDeque::new())),
            last_injected_text: Arc::new(Mutex::new(String::new())),
            target_app: Arc::new(Mutex::new(None)),
            is_active: Arc::new(Mutex::new(false)),
            transcription_receiver: None,
        }
    }

    /// 启动渐进式注入监听
    pub async fn start_listening(
        &mut self,
        mut transcription_receiver: broadcast::Receiver<TranscriptionEvent>,
        target_app: Option<ApplicationInfo>,
    ) -> AppResult<()> {
        // 设置目标应用
        {
            let mut app_guard = self.target_app.lock().unwrap();
            *app_guard = target_app.clone();
        }

        // 设置为活动状态
        {
            let mut active = self.is_active.lock().unwrap();
            if *active {
                return Err(AppError::StreamingError("渐进式注入已在运行中".to_string()));
            }
            *active = true;
        }

        println!("🚀 启动渐进式文本注入监听，目标应用: {:?}", target_app.map(|a| a.name));

        // 启动事件处理循环
        let injection_queue = Arc::clone(&self.injection_queue);
        let last_injected = Arc::clone(&self.last_injected_text);
        let is_active = Arc::clone(&self.is_active);
        let config = self.config.clone();
        let injector = self.text_injector.clone();

        tokio::spawn(async move {
            Self::event_processing_loop(
                transcription_receiver,
                injection_queue,
                last_injected,
                is_active,
                config,
                injector,
            ).await;
        });

        Ok(())
    }

    /// 停止渐进式注入
    pub async fn stop_listening(&mut self) -> AppResult<()> {
        let mut is_active = self.is_active.lock().unwrap();
        *is_active = false;
        println!("🛑 渐进式文本注入已停止");
        Ok(())
    }

    /// 事件处理主循环
    async fn event_processing_loop(
        mut receiver: broadcast::Receiver<TranscriptionEvent>,
        injection_queue: Arc<Mutex<VecDeque<InjectionItem>>>,
        last_injected_text: Arc<Mutex<String>>,
        is_active: Arc<Mutex<bool>>,
        config: ProgressiveInjectionConfig,
        text_injector: TextInjector,
    ) {
        let mut last_injection_time = Instant::now();

        while let Ok(event) = receiver.recv().await {
            // 检查是否应该继续处理
            {
                let active = is_active.lock().unwrap();
                if !*active {
                    break;
                }
            }

            match event {
                TranscriptionEvent::StreamingTranscription { text, is_partial, confidence, .. } => {
                    // 处理流式转录事件
                    if config.enabled && !config.final_only {
                        Self::process_streaming_text(
                            &text,
                            is_partial,
                            confidence,
                            &injection_queue,
                            &config,
                        ).await;
                    }
                }
                TranscriptionEvent::FinalText { text, .. } => {
                    // 处理最终转录事件
                    if config.enabled {
                        Self::process_final_text(
                            &text,
                            &injection_queue,
                            &config,
                        ).await;
                    }
                }
                _ => {
                    // 忽略其他事件类型
                }
            }

            // 检查是否应该执行注入
            if last_injection_time.elapsed() >= Duration::from_millis(config.inject_interval_ms) {
                Self::execute_pending_injections(
                    &injection_queue,
                    &last_injected_text,
                    &text_injector,
                    &config,
                ).await;
                last_injection_time = Instant::now();
            }
        }

        // 处理剩余的注入队列
        Self::execute_pending_injections(
            &injection_queue,
            &last_injected_text,
            &text_injector,
            &config,
        ).await;

        println!("🔚 渐进式注入事件处理循环结束");
    }

    /// 处理流式转录文本
    async fn process_streaming_text(
        text: &str,
        is_partial: bool,
        confidence: f64,
        injection_queue: &Arc<Mutex<VecDeque<InjectionItem>>>,
        config: &ProgressiveInjectionConfig,
    ) {
        // 过滤低质量或过短的文本
        if text.trim().len() < config.min_inject_length || confidence < config.min_confidence_threshold {
            return;
        }

        let item = InjectionItem {
            text: text.trim().to_string(),
            is_partial,
            confidence,
            timestamp: Instant::now(),
            should_replace_previous: is_partial && config.enable_backspace_correction,
        };

        let mut queue = injection_queue.lock().unwrap();
        
        // 智能合并：如果新文本是前一个文本的扩展，替换而不是追加
        if config.smart_prefix_merging {
            if let Some(last_item) = queue.back_mut() {
                if text.starts_with(&last_item.text) && text.len() > last_item.text.len() {
                    // 替换为更长的版本
                    last_item.text = text.trim().to_string();
                    last_item.confidence = confidence;
                    last_item.timestamp = Instant::now();
                    return;
                }
            }
        }

        // 添加到队列
        queue.push_back(item);
        
        // 限制队列长度
        while queue.len() > config.max_queue_length {
            queue.pop_front();
        }

        println!("📝 添加流式文本到注入队列: '{}' (部分={}, 置信度={:.2})", text, is_partial, confidence);
    }

    /// 处理最终转录文本
    async fn process_final_text(
        text: &str,
        injection_queue: &Arc<Mutex<VecDeque<InjectionItem>>>,
        config: &ProgressiveInjectionConfig,
    ) {
        if text.trim().len() < config.min_inject_length {
            return;
        }

        let item = InjectionItem {
            text: text.trim().to_string(),
            is_partial: false,
            confidence: 0.95, // 最终文本假设高置信度
            timestamp: Instant::now(),
            should_replace_previous: false,
        };

        let mut queue = injection_queue.lock().unwrap();
        queue.push_back(item);

        println!("✅ 添加最终文本到注入队列: '{}'", text);
    }

    /// 执行待处理的文本注入
    async fn execute_pending_injections(
        injection_queue: &Arc<Mutex<VecDeque<InjectionItem>>>,
        last_injected_text: &Arc<Mutex<String>>,
        text_injector: &TextInjector,
        config: &ProgressiveInjectionConfig,
    ) {
        let item_to_inject = {
            let mut queue = injection_queue.lock().unwrap();
            queue.pop_front()
        };

        if let Some(item) = item_to_inject {
            // 检查是否需要执行退格删除
            let text_to_inject = if item.should_replace_previous && config.enable_backspace_correction {
                let last_text = {
                    let last_injected = last_injected_text.lock().unwrap();
                    last_injected.clone()
                };

                if !last_text.is_empty() {
                    // 生成退格字符串 + 新文本
                    let backspaces = "\u{8}".repeat(last_text.chars().count()); // Unicode退格字符
                    format!("{}{}", backspaces, item.text)
                } else {
                    item.text.clone()
                }
            } else {
                item.text.clone()
            };

            // 执行文本注入
            match text_injector.inject_text(&text_to_inject).await {
                Ok(_) => {
                    // 更新最后注入的文本
                    {
                        let mut last_injected = last_injected_text.lock().unwrap();
                        *last_injected = item.text.clone();
                    }

                    println!("✅ 渐进式注入成功: '{}' (置信度: {:.2})", item.text, item.confidence);
                }
                Err(e) => {
                    eprintln!("❌ 渐进式注入失败: {} - {}", item.text, e);
                }
            }

            // 添加注入间隔
            tokio::time::sleep(Duration::from_millis(config.inject_interval_ms)).await;
        }
    }

    /// 检查是否正在运行
    pub fn is_active(&self) -> bool {
        *self.is_active.lock().unwrap()
    }

    /// 获取当前注入队列长度
    pub fn queue_length(&self) -> usize {
        self.injection_queue.lock().unwrap().len()
    }

    /// 清空注入队列
    pub fn clear_queue(&self) {
        let mut queue = self.injection_queue.lock().unwrap();
        queue.clear();
        println!("🧹 渐进式注入队列已清空");
    }

    /// 获取最后注入的文本
    pub fn get_last_injected_text(&self) -> String {
        self.last_injected_text.lock().unwrap().clone()
    }

    /// 设置目标应用
    pub fn set_target_app(&self, app: Option<ApplicationInfo>) {
        let mut target_app = self.target_app.lock().unwrap();
        *target_app = app;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::TextInjectionConfig;

    #[tokio::test]
    async fn test_progressive_injector_creation() {
        let prog_config = ProgressiveInjectionConfig::default();
        let inject_config = TextInjectionConfig::default();
        
        let injector = ProgressiveTextInjector::new(prog_config, inject_config);
        
        assert!(!injector.is_active());
        assert_eq!(injector.queue_length(), 0);
        assert_eq!(injector.get_last_injected_text(), "");
    }

    #[tokio::test]
    async fn test_queue_management() {
        let prog_config = ProgressiveInjectionConfig::default();
        let inject_config = TextInjectionConfig::default();
        
        let injector = ProgressiveTextInjector::new(prog_config, inject_config);
        
        // 测试队列操作
        assert_eq!(injector.queue_length(), 0);
        
        injector.clear_queue();
        assert_eq!(injector.queue_length(), 0);
    }
}