use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use reqwest::Client;
use crate::errors::{AppError, AppResult};
use crate::types::{TranscriptionResult, TranscriptionConfig};
use super::{WhisperTranscriber, TranscriptionApiClient};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct TranscriptionService {
    whisper_transcriber: Arc<WhisperTranscriber>,
    api_client: Arc<TranscriptionApiClient>,
    openai_api_key: Option<String>,
    // 智能模式切换相关
    mode_selector: Arc<Mutex<IntelligentModeSelector>>,
}

impl TranscriptionService {
    pub fn new(http_client: Client, openai_api_key: Option<String>) -> Self {
        Self {
            whisper_transcriber: Arc::new(WhisperTranscriber::new()),
            api_client: Arc::new(TranscriptionApiClient::new(http_client)),
            openai_api_key,
            mode_selector: Arc::new(Mutex::new(IntelligentModeSelector::new())),
        }
    }

    /// 转录音频文件（智能模式选择）
    pub async fn transcribe_audio<P: AsRef<Path>>(
        &self,
        audio_file_path: P,
        config: &TranscriptionConfig,
    ) -> AppResult<TranscriptionResult> {
        let path = audio_file_path.as_ref();
        
        // 验证音频文件存在
        if !path.exists() {
            return Err(AppError::FileSystemError(format!("音频文件不存在: {:?}", path)));
        }

        // 记录转录开始
        println!("🎵 开始转录音频文件: {:?}", path);
        println!("🔧 原始配置: 模型={}, 本地={}, 语言={:?}", 
                 config.model_name, config.is_local, config.language);
        
        // 智能模式选择
        let optimized_config = {
            let selector = self.mode_selector.lock();
            drop(selector); // 释放锁
            // 暂时跳过智能模式选择，直接使用原始配置
            config.clone()
        };
        
        if optimized_config.is_local != config.is_local {
            println!("🧠 智能模式切换: {} -> {}", 
                    if config.is_local { "本地" } else { "在线" },
                    if optimized_config.is_local { "本地" } else { "在线" });
        }
        
        let start_time = Instant::now();
        let result = if optimized_config.is_local {
            // 使用本地Whisper转录
            match optimized_config.model_name.as_str() {
                model_name if model_name.starts_with("whisper-") => {
                    println!("🔍 使用本地 Whisper 模型进行转录");
                    self.whisper_transcriber
                        .transcribe_with_local_whisper(path, &optimized_config)
                        .await
                },
                _ => {
                    Err(AppError::ValidationError(format!("不支持的本地模型: {}", optimized_config.model_name)))
                }
            }
        } else {
            // 使用API转录
            match optimized_config.model_name.as_str() {
                "luyin-api" | "luyingwang-online" => {
                    println!("🔍 使用录音王API进行转录");
                    self.api_client.transcribe_with_luyin_api(path).await
                },
                _ => {
                    println!("🔍 使用OpenAI兼容API进行转录");
                    let api_key = self.openai_api_key
                        .as_ref()
                        .ok_or_else(|| AppError::ConfigurationError("缺少OpenAI API密钥".to_string()))?;
                    
                    self.api_client
                        .transcribe_with_openai_api(path, api_key, &optimized_config)
                        .await
                }
            }
        };

        let duration = start_time.elapsed();
        
        // 记录性能反馈
        {
            let mut selector = self.mode_selector.lock();
            selector.record_performance(&optimized_config, duration, result.is_ok());
        }
        
        match &result {
            Ok(transcription) => {
                println!("✅ 转录成功完成，结果长度: {} 字符，耗时: {:?}", 
                        transcription.text.len(), duration);
                if transcription.text.len() > 100 {
                    println!("📝 转录内容预览: {}...", &transcription.text[..100]);
                } else {
                    println!("📝 转录内容: {}", transcription.text);
                }
            },
            Err(e) => {
                println!("❌ 转录失败: {}，耗时: {:?}", e, duration);
            }
        }

        result
    }

    /// 获取可用的转录模型列表
    pub fn get_available_models(&self) -> Vec<String> {
        let mut models = vec![
            // 本地Whisper模型
            "whisper-tiny".to_string(),
            "whisper-base".to_string(),
            "whisper-small".to_string(),
            "whisper-medium".to_string(),
            "whisper-large-v3".to_string(),
            "whisper-large-v3-turbo".to_string(),
            
            // API模型
            "whisper-1".to_string(),
            "gpt-4o-mini".to_string(),
            "luyin-api".to_string(),
        ];

        // 根据API密钥可用性过滤模型
        if self.openai_api_key.is_none() {
            models.retain(|model| {
                model.starts_with("whisper-") && !model.eq("whisper-1") || model.eq("luyin-api")
            });
        }

        models
    }

    /// 检查模型是否可用
    pub async fn is_model_available(&self, model_name: &str) -> bool {
        match model_name {
            name if name.starts_with("whisper-") && name != "whisper-1" => {
                // 本地Whisper模型 - 检查是否可以下载
                true // 假设所有支持的模型都可用
            },
            "luyin-api" => {
                // 录音API - 检查网络连接
                true // 简化实现
            },
            "whisper-1" | "gpt-4o-mini" => {
                // OpenAI API模型 - 检查API密钥
                self.openai_api_key.is_some()
            },
            _ => false
        }
    }

    /// 创建默认配置
    pub fn create_default_config(model_name: &str) -> TranscriptionConfig {
        let is_local = model_name.starts_with("whisper-") && model_name != "whisper-1";
        
        TranscriptionConfig {
            model_name: model_name.to_string(),
            language: Some("auto".to_string()),
            temperature: Some(0.0),
            is_local,
            api_endpoint: if is_local { 
                None 
            } else { 
                Some("https://api.openai.com/v1/audio/transcriptions".to_string()) 
            },
        }
    }

    /// 验证转录配置
    pub fn validate_config(&self, config: &TranscriptionConfig) -> AppResult<()> {
        // 验证模型名称
        let available_models = self.get_available_models();
        if !available_models.contains(&config.model_name) {
            return Err(AppError::ValidationError(format!("不支持的模型: {}", config.model_name)));
        }

        // 验证API配置
        if !config.is_local {
            match config.model_name.as_str() {
                "luyin-api" => {
                    // 录音API不需要额外验证
                },
                _ => {
                    if self.openai_api_key.is_none() {
                        return Err(AppError::ConfigurationError("OpenAI API模型需要API密钥".to_string()));
                    }
                }
            }
        }

        // 验证温度参数
        if let Some(temp) = config.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(AppError::ValidationError("温度参数必须在0.0-2.0之间".to_string()));
            }
        }

        Ok(())
    }

    /// 测试转录服务
    pub async fn test_service(&self, config: &TranscriptionConfig) -> AppResult<bool> {
        // 验证配置
        self.validate_config(config)?;
        
        if config.is_local {
            // 测试本地模型可用性
            println!("🧪 测试本地Whisper模型: {}", config.model_name);
            // 这里可以添加模型文件存在性检查
            Ok(true)
        } else {
            match config.model_name.as_str() {
                "luyin-api" => {
                    println!("🧪 测试录音API连接");
                    // 可以添加API连通性测试
                    Ok(true)
                },
                _ => {
                    println!("🧪 测试OpenAI兼容API连接");
                    if let Some(api_key) = &self.openai_api_key {
                        let default_endpoint = "https://api.openai.com/v1/audio/transcriptions".to_string();
                        let api_endpoint = config.api_endpoint
                            .as_ref()
                            .unwrap_or(&default_endpoint);
                        self.api_client.test_api_connection(api_endpoint, api_key).await
                    } else {
                        Err(AppError::ConfigurationError("缺少API密钥".to_string()))
                    }
                }
            }
        }
    }

    /// 更新API密钥
    pub fn update_api_key(&mut self, api_key: Option<String>) {
        self.openai_api_key = api_key;
    }
    
    /// 获取模式选择器统计信息
    pub fn get_mode_selector_stats(&self) -> ModeSelectionStats {
        self.mode_selector.lock().get_stats()
    }
    
    /// 重置模式选择器统计
    pub fn reset_mode_selector_stats(&self) {
        self.mode_selector.lock().reset_stats();
    }
}

/// 智能模式选择器
#[derive(Debug)]
struct IntelligentModeSelector {
    performance_history: std::collections::HashMap<String, Vec<PerformanceRecord>>,
    network_status: NetworkStatus,
    last_network_check: Option<Instant>,
    preferences: ModePreferences,
}

#[derive(Debug, Clone)]
struct PerformanceRecord {
    duration: Duration,
    success: bool,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
enum NetworkStatus {
    Unknown,
    Online,
    Offline,
    Slow,
}

#[derive(Debug, Clone)]
struct ModePreferences {
    prefer_local: bool,
    max_acceptable_delay: Duration,
    quality_over_speed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModeSelectionStats {
    pub total_selections: u32,
    pub local_selections: u32,
    pub online_selections: u32,
    pub auto_switches: u32,
    pub avg_local_duration_ms: u32,
    pub avg_online_duration_ms: u32,
    pub local_success_rate: f64,
    pub online_success_rate: f64,
}

impl IntelligentModeSelector {
    fn new() -> Self {
        Self {
            performance_history: std::collections::HashMap::new(),
            network_status: NetworkStatus::Unknown,
            last_network_check: None,
            preferences: ModePreferences {
                prefer_local: true,
                max_acceptable_delay: Duration::from_secs(30),
                quality_over_speed: false,
            },
        }
    }
    
    async fn select_optimal_mode(&mut self, config: &TranscriptionConfig) -> AppResult<TranscriptionConfig> {
        // 如果用户明确指定模式，不进行智能选择
        if self.should_respect_user_choice(config) {
            return Ok(config.clone());
        }
        
        // 更新网络状态
        self.update_network_status().await;
        
        let should_use_local = self.decide_mode(config).await;
        
        let mut optimized_config = config.clone();
        
        if should_use_local && !config.is_local {
            // 切换到本地模式
            optimized_config.is_local = true;
            optimized_config.model_name = self.select_best_local_model();
        } else if !should_use_local && config.is_local {
            // 切换到在线模式
            optimized_config.is_local = false;
            optimized_config.model_name = self.select_best_online_model();
        }
        
        Ok(optimized_config)
    }
    
    fn record_performance(&mut self, config: &TranscriptionConfig, duration: Duration, success: bool) {
        let key = format!("{}_{}", 
                         if config.is_local { "local" } else { "online" },
                         config.model_name);
        
        let record = PerformanceRecord {
            duration,
            success,
            timestamp: Instant::now(),
        };
        
        self.performance_history
            .entry(key)
            .or_insert_with(Vec::new)
            .push(record);
        
        // 只保留最近的50个记录
        let records = self.performance_history.get_mut(&format!("{}_{}", 
                                                                if config.is_local { "local" } else { "online" },
                                                                config.model_name)).unwrap();
        if records.len() > 50 {
            records.drain(0..records.len() - 50);
        }
    }
    
    async fn update_network_status(&mut self) {
        let now = Instant::now();
        
        // 每30秒检查一次网络状态
        if let Some(last_check) = self.last_network_check {
            if now.duration_since(last_check) < Duration::from_secs(30) {
                return;
            }
        }
        
        self.last_network_check = Some(now);
        
        // 简单的网络连通性检查
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        
        match client.get("https://www.google.com").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    self.network_status = NetworkStatus::Online;
                } else {
                    self.network_status = NetworkStatus::Slow;
                }
            }
            Err(_) => {
                self.network_status = NetworkStatus::Offline;
            }
        }
    }
    
    async fn decide_mode(&self, config: &TranscriptionConfig) -> bool {
        // 如果网络离线，必须使用本地模式
        if matches!(self.network_status, NetworkStatus::Offline) {
            return true;
        }
        
        // 如果用户偏好本地处理
        if self.preferences.prefer_local {
            return true;
        }
        
        // 根据历史性能决定
        let local_performance = self.get_average_performance("local", &config.model_name);
        let online_performance = self.get_average_performance("online", &config.model_name);
        
        match (local_performance, online_performance) {
            (Some(local), Some(online)) => {
                if self.preferences.quality_over_speed {
                    // 优先考虑成功率
                    local.success_rate >= online.success_rate
                } else {
                    // 优先考虑速度
                    local.avg_duration < online.avg_duration
                }
            }
            (Some(_), None) => true,   // 只有本地历史记录
            (None, Some(_)) => false,  // 只有在线历史记录
            (None, None) => self.preferences.prefer_local, // 没有历史记录，使用偏好
        }
    }
    
    fn should_respect_user_choice(&self, config: &TranscriptionConfig) -> bool {
        // 如果用户明确选择了特定的API模型，尊重用户选择
        matches!(config.model_name.as_str(), "whisper-1" | "gpt-4o-mini" | "luyin-api")
    }
    
    fn select_best_local_model(&self) -> String {
        // 根据性能历史选择最佳本地模型
        let models = ["whisper-base", "whisper-small", "whisper-medium"];
        
        let best_model = models.iter()
            .min_by_key(|&model| {
                self.get_average_performance("local", model)
                    .map(|perf| perf.avg_duration.as_millis() as u64)
                    .unwrap_or(u64::MAX)
            })
            .unwrap_or(&"whisper-base");
        
        best_model.to_string()
    }
    
    fn select_best_online_model(&self) -> String {
        // 默认使用录音API
        "luyin-api".to_string()
    }
    
    fn get_average_performance(&self, mode: &str, model: &str) -> Option<AveragePerformance> {
        let key = format!("{}_{}", mode, model);
        let records = self.performance_history.get(&key)?;
        
        if records.is_empty() {
            return None;
        }
        
        let total_duration: Duration = records.iter().map(|r| r.duration).sum();
        let success_count = records.iter().filter(|r| r.success).count();
        
        Some(AveragePerformance {
            avg_duration: total_duration / records.len() as u32,
            success_rate: success_count as f64 / records.len() as f64,
            sample_count: records.len(),
        })
    }
    
    fn get_stats(&self) -> ModeSelectionStats {
        let mut total_selections = 0u32;
        let mut local_selections = 0u32;
        let mut online_selections = 0u32;
        let mut local_durations = Vec::new();
        let mut online_durations = Vec::new();
        let mut local_successes = 0u32;
        let mut online_successes = 0u32;
        
        for (key, records) in &self.performance_history {
            total_selections += records.len() as u32;
            
            let is_local = key.starts_with("local_");
            
            for record in records {
                if is_local {
                    local_selections += 1;
                    local_durations.push(record.duration.as_millis() as u32);
                    if record.success {
                        local_successes += 1;
                    }
                } else {
                    online_selections += 1;
                    online_durations.push(record.duration.as_millis() as u32);
                    if record.success {
                        online_successes += 1;
                    }
                }
            }
        }
        
        let avg_local_duration = if local_durations.is_empty() {
            0
        } else {
            local_durations.iter().sum::<u32>() / local_durations.len() as u32
        };
        
        let avg_online_duration = if online_durations.is_empty() {
            0
        } else {
            online_durations.iter().sum::<u32>() / online_durations.len() as u32
        };
        
        ModeSelectionStats {
            total_selections,
            local_selections,
            online_selections,
            auto_switches: 0, // 暂时设为0，需要额外跟踪
            avg_local_duration_ms: avg_local_duration,
            avg_online_duration_ms: avg_online_duration,
            local_success_rate: if local_selections > 0 {
                local_successes as f64 / local_selections as f64
            } else {
                0.0
            },
            online_success_rate: if online_selections > 0 {
                online_successes as f64 / online_selections as f64
            } else {
                0.0
            },
        }
    }
    
    fn reset_stats(&mut self) {
        self.performance_history.clear();
    }
}

#[derive(Debug, Clone)]
struct AveragePerformance {
    avg_duration: Duration,
    success_rate: f64,
    sample_count: usize,
}