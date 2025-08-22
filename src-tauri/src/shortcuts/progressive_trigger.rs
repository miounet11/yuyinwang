// 渐进式语音输入触发系统 - Week 3 核心组件
// 实现长按快捷键直接启动渐进式文本注入

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tauri::{Manager, GlobalShortcutManager};

/// 长按触发配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressiveTriggerConfig {
    /// 快捷键组合 (默认: "Option+Space")
    pub shortcut: String,
    /// 长按阈值时间 (毫秒，默认: 800)
    pub long_press_threshold_ms: u64,
    /// 是否启用长按触发
    pub enabled: bool,
    /// 是否启用实时注入
    pub enable_real_time_injection: bool,
    /// 触发反馈音效
    pub trigger_sound_enabled: bool,
    /// 目标应用自动检测
    pub auto_detect_target_app: bool,
}

impl Default for ProgressiveTriggerConfig {
    fn default() -> Self {
        Self {
            shortcut: "Option+Space".to_string(),
            long_press_threshold_ms: 800,
            enabled: true,
            enable_real_time_injection: true,
            trigger_sound_enabled: true,
            auto_detect_target_app: true,
        }
    }
}

/// 触发状态
#[derive(Debug, Clone)]
enum TriggerState {
    Idle,
    KeyDown(Instant),
    LongPressTriggered,
    VoiceInputActive,
}

/// 渐进式触发管理器
pub struct ProgressiveTriggerManager {
    config: Arc<Mutex<ProgressiveTriggerConfig>>,
    state: Arc<Mutex<TriggerState>>,
    app_handle: Option<tauri::AppHandle>,
    is_monitoring: Arc<Mutex<bool>>,
}

impl ProgressiveTriggerManager {
    /// 创建新的触发管理器
    pub fn new(config: ProgressiveTriggerConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            state: Arc::new(Mutex::new(TriggerState::Idle)),
            app_handle: None,
            is_monitoring: Arc::new(Mutex::new(false)),
        }
    }

    /// 初始化管理器
    pub fn initialize(&mut self, app_handle: tauri::AppHandle) -> Result<(), String> {
        self.app_handle = Some(app_handle);
        Ok(())
    }

    /// 启动长按监听
    pub async fn start_monitoring(&self) -> Result<String, String> {
        let app_handle = self.app_handle.as_ref()
            .ok_or("应用句柄未初始化")?;

        // 检查是否已在监听
        {
            let mut monitoring = self.is_monitoring.lock().unwrap();
            if *monitoring {
                return Ok("长按触发监听已在运行中".to_string());
            }
            *monitoring = true;
        }

        let shortcut = {
            let config = self.config.lock().unwrap();
            config.shortcut.clone()
        };

        println!("🚀 启动渐进式语音输入长按监听: {}", shortcut);

        // 获取当前活动应用（在注册快捷键之前）
        let target_bundle_id = if self.config.lock().unwrap().auto_detect_target_app {
            match crate::commands::get_active_app_info_for_voice().await {
                Ok(app_info) => {
                    if let Some(bundle_id) = &app_info.bundle_id {
                        if !bundle_id.contains("recordingking") {
                            println!("🎯 检测到目标应用: {} ({})", app_info.name, bundle_id);
                            Some(bundle_id.clone())
                        } else {
                            println!("⚠️ 跳过Recording King自身");
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        };

        // 注册快捷键
        let app_clone = app_handle.clone();
        let config_clone = Arc::clone(&self.config);
        let state_clone = Arc::clone(&self.state);
        let is_monitoring_clone = Arc::clone(&self.is_monitoring);

        match app_handle.global_shortcut_manager().register(&shortcut, move || {
            // 执行长按触发逻辑
            tokio::spawn(Self::handle_shortcut_trigger(
                app_clone.clone(),
                config_clone.clone(),
                state_clone.clone(),
                target_bundle_id.clone(),
            ));
        }) {
            Ok(_) => {
                println!("✅ 长按触发快捷键已注册: {}", shortcut);
                Ok(format!("长按触发监听已启动 ({})", shortcut))
            }
            Err(e) => {
                // 重置监听状态
                {
                    let mut monitoring = is_monitoring_clone.lock().unwrap();
                    *monitoring = false;
                }
                println!("❌ 注册快捷键失败: {}", e);
                Err(format!("注册快捷键失败: {}", e))
            }
        }
    }

    /// 处理快捷键触发
    async fn handle_shortcut_trigger(
        app_handle: tauri::AppHandle,
        config: Arc<Mutex<ProgressiveTriggerConfig>>,
        state: Arc<Mutex<TriggerState>>,
        target_bundle_id: Option<String>,
    ) {
        let trigger_config = {
            let cfg = config.lock().unwrap();
            cfg.clone()
        };

        if !trigger_config.enabled {
            println!("⚠️ 长按触发已禁用");
            return;
        }

        println!("🎙️ 长按快捷键触发，启动渐进式语音输入");

        // 发送触发事件
        if let Err(e) = app_handle.emit_all("progressive_trigger_activated", serde_json::json!({
            "trigger": "long_press",
            "shortcut": trigger_config.shortcut,
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "target_app": target_bundle_id,
        })) {
            eprintln!("发送触发事件失败: {}", e);
        }

        // 更新状态
        {
            let mut current_state = state.lock().unwrap();
            *current_state = TriggerState::VoiceInputActive;
        }

        // 启动渐进式语音输入
        match crate::commands::start_progressive_voice_input(
            target_bundle_id,
            app_handle.clone(),
            Some(trigger_config.enable_real_time_injection),
        ).await {
            Ok(message) => {
                println!("✅ 渐进式语音输入启动成功: {}", message);
                
                // 发送成功事件
                if let Err(e) = app_handle.emit_all("progressive_voice_input_started", serde_json::json!({
                    "success": true,
                    "message": message,
                    "real_time_injection": trigger_config.enable_real_time_injection,
                })) {
                    eprintln!("发送启动成功事件失败: {}", e);
                }
            }
            Err(e) => {
                eprintln!("❌ 启动渐进式语音输入失败: {}", e);
                
                // 发送错误事件
                if let Err(emit_error) = app_handle.emit_all("progressive_voice_input_error", serde_json::json!({
                    "success": false,
                    "error": e,
                })) {
                    eprintln!("发送错误事件失败: {}", emit_error);
                }
                
                // 重置状态
                let mut current_state = state.lock().unwrap();
                *current_state = TriggerState::Idle;
            }
        }
    }

    /// 停止监听
    pub fn stop_monitoring(&self) -> Result<String, String> {
        if let Some(app_handle) = &self.app_handle {
            let shortcut = {
                let config = self.config.lock().unwrap();
                config.shortcut.clone()
            };

            match app_handle.global_shortcut_manager().unregister(&shortcut) {
                Ok(_) => {
                    let mut monitoring = self.is_monitoring.lock().unwrap();
                    *monitoring = false;
                    
                    let mut state = self.state.lock().unwrap();
                    *state = TriggerState::Idle;
                    
                    println!("🛑 长按触发监听已停止");
                    Ok("长按触发监听已停止".to_string())
                }
                Err(e) => {
                    println!("❌ 取消注册快捷键失败: {}", e);
                    Err(format!("取消注册失败: {}", e))
                }
            }
        } else {
            Err("应用句柄未初始化".to_string())
        }
    }

    /// 检查是否正在监听
    pub fn is_monitoring(&self) -> bool {
        *self.is_monitoring.lock().unwrap()
    }

    /// 获取当前状态
    pub fn get_status(&self) -> serde_json::Value {
        let config = self.config.lock().unwrap();
        let is_monitoring = *self.is_monitoring.lock().unwrap();
        let state = match *self.state.lock().unwrap() {
            TriggerState::Idle => "idle",
            TriggerState::KeyDown(_) => "key_down",
            TriggerState::LongPressTriggered => "triggered",
            TriggerState::VoiceInputActive => "active",
        };

        serde_json::json!({
            "monitoring": is_monitoring,
            "state": state,
            "config": {
                "shortcut": config.shortcut,
                "threshold_ms": config.long_press_threshold_ms,
                "enabled": config.enabled,
                "real_time_injection": config.enable_real_time_injection,
                "sound_enabled": config.trigger_sound_enabled,
                "auto_detect_app": config.auto_detect_target_app,
            }
        })
    }

    /// 更新配置
    pub fn update_config(&self, new_config: ProgressiveTriggerConfig) -> Result<(), String> {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
        println!("🔧 长按触发配置已更新");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progressive_trigger_config_default() {
        let config = ProgressiveTriggerConfig::default();
        assert_eq!(config.shortcut, "Option+Space");
        assert_eq!(config.long_press_threshold_ms, 800);
        assert!(config.enabled);
        assert!(config.enable_real_time_injection);
    }

    #[test]
    fn test_trigger_manager_creation() {
        let config = ProgressiveTriggerConfig::default();
        let manager = ProgressiveTriggerManager::new(config);
        assert!(!manager.is_monitoring());
    }
}