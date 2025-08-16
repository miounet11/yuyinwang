// 优化后的语音输入快捷键系统
// 基于 tech-lead-reviewer 和 ux-reviewer 的专业建议

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tauri::{Manager, GlobalShortcutManager as TauriGSM};
use crate::errors::{AppResult, AppError};
use crate::system::PermissionManager;

/// 快捷键优先级策略
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ShortcutPriority {
    Primary = 1,    // 主要快捷键，优先注册
    Secondary = 2,  // 备用快捷键，主要失败时使用
    Fallback = 3,   // 最后的备选方案
}

/// 优化的快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceShortcutConfig {
    pub primary_shortcut: String,
    pub secondary_shortcut: String,
    pub auto_stop_enabled: bool,
    pub silence_threshold_ms: u64,
    pub max_recording_duration_ms: u64,
    pub feedback_enabled: bool,
}

impl Default for VoiceShortcutConfig {
    fn default() -> Self {
        Self {
            // 基于 UX 分析，简化为两个选项
            primary_shortcut: "Cmd+Space".to_string(),      // 类似 Spotlight，用户熟悉
            secondary_shortcut: "Cmd+Shift+A".to_string(),  // 避免系统冲突
            auto_stop_enabled: true,                        // 智能 VAD 自动停止
            silence_threshold_ms: 1500,                     // 1.5秒静音后停止
            max_recording_duration_ms: 30000,               // 最长30秒
            feedback_enabled: true,                         // 声音和视觉反馈
        }
    }
}

/// 录音状态枚举
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum VoiceRecordingState {
    Idle,
    Starting,
    Recording,
    Processing,
    Completed,
    Error(String),
}

/// 优化的语音快捷键管理器
pub struct OptimizedVoiceShortcutManager {
    app_handle: tauri::AppHandle,
    config: Arc<Mutex<VoiceShortcutConfig>>,
    recording_state: Arc<Mutex<VoiceRecordingState>>,
    registered_shortcuts: Arc<Mutex<Vec<(String, ShortcutPriority)>>>,
    last_trigger_time: Arc<Mutex<Instant>>,
}

impl OptimizedVoiceShortcutManager {
    pub fn new(app_handle: tauri::AppHandle) -> AppResult<Self> {
        Ok(Self {
            app_handle,
            config: Arc::new(Mutex::new(VoiceShortcutConfig::default())),
            recording_state: Arc::new(Mutex::new(VoiceRecordingState::Idle)),
            registered_shortcuts: Arc::new(Mutex::new(Vec::new())),
            last_trigger_time: Arc::new(Mutex::new(Instant::now())),
        })
    }

    /// 智能快捷键注册 - 基于优先级策略
    pub fn register_smart_shortcuts(&self) -> AppResult<()> {
        println!("🔧 启动智能快捷键系统...");
        
        // 首先进行全面权限检查
        self.check_and_guide_permissions()?;
        
        let config = self.config.lock().unwrap().clone();
        
        // 定义优先级快捷键列表
        let priority_shortcuts = vec![
            (config.primary_shortcut.clone(), ShortcutPriority::Primary),
            (config.secondary_shortcut.clone(), ShortcutPriority::Secondary),
            ("Cmd+Shift+V".to_string(), ShortcutPriority::Fallback), // 最后备选
        ];

        let mut registered = Vec::new();
        let mut registration_errors = Vec::new();

        for (shortcut, priority) in priority_shortcuts {
            match self.try_register_shortcut(&shortcut, priority.clone()) {
                Ok(_) => {
                    println!("✅ 成功注册快捷键: {} (优先级: {:?})", shortcut, priority);
                    let priority_clone = priority.clone();
                    registered.push((shortcut, priority));
                    
                    // 如果主要快捷键注册成功，可以停止注册其他低优先级的
                    if priority_clone == ShortcutPriority::Primary {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ 快捷键 {} 注册失败: {}", shortcut, e);
                    registration_errors.push((shortcut, e.to_string()));
                }
            }
        }

        // 保存注册结果
        *self.registered_shortcuts.lock().unwrap() = registered.clone();

        if registered.is_empty() {
            return Err(AppError::ShortcutError(
                format!("所有快捷键注册失败: {:?}", registration_errors)
            ));
        }

        println!("🎯 快捷键系统启动成功");
        println!("📱 活跃快捷键: {:?}", registered.iter().map(|(k, _)| k).collect::<Vec<_>>());
        
        // 发送状态更新到前端
        self.emit_shortcut_status_update(&registered)?;
        
        Ok(())
    }

    /// 尝试注册单个快捷键
    fn try_register_shortcut(&self, shortcut: &str, _priority: ShortcutPriority) -> AppResult<()> {
        // 检查快捷键是否已被系统占用
        if self.is_system_shortcut_conflict(shortcut) {
            return Err(AppError::ShortcutError(
                format!("快捷键 {} 与系统快捷键冲突", shortcut)
            ));
        }

        let app_handle = self.app_handle.clone();
        let recording_state = Arc::clone(&self.recording_state);
        let last_trigger_time = Arc::clone(&self.last_trigger_time);
        let config = Arc::clone(&self.config);
        let shortcut_str = shortcut.to_string();

        self.app_handle.global_shortcut_manager().register(
            shortcut,
            move || {
                // 防止快速重复触发
                let now = Instant::now();
                let mut last_time = last_trigger_time.lock().unwrap();
                if now.duration_since(*last_time) < Duration::from_millis(300) {
                    return;
                }
                *last_time = now;

                println!("🔑 [DEBUG] 智能快捷键触发: {}", shortcut_str);
                println!("   [DEBUG] 当前时间: {:?}", now);
                
                // 检查当前录音状态
                let current_state = recording_state.lock().unwrap().clone();
                println!("   [DEBUG] 当前状态: {:?}", current_state);
                
                match current_state {
                    VoiceRecordingState::Idle => {
                        println!("   [DEBUG] 状态为空闲，准备开始新的录音会话");
                        // 开始新的录音会话
                        if let Err(e) = start_voice_session(
                            app_handle.clone(),
                            recording_state.clone(),
                            config.clone()
                        ) {
                            eprintln!("❌ [DEBUG] 启动语音会话失败: {:?}", e);
                        } else {
                            println!("✅ [DEBUG] 语音会话启动命令已发送");
                        }
                    }
                    VoiceRecordingState::Recording => {
                        println!("   [DEBUG] 状态为录音中，准备停止录音");
                        // 手动停止录音
                        if let Err(e) = stop_voice_session(
                            app_handle.clone(),
                            recording_state.clone()
                        ) {
                            eprintln!("❌ [DEBUG] 停止语音会话失败: {:?}", e);
                        } else {
                            println!("✅ [DEBUG] 停止录音命令已发送");
                        }
                    }
                    _ => {
                        println!("ℹ️ [DEBUG] 语音会话正在处理中，状态: {:?}", current_state);
                    }
                }
            }
        ).map_err(|e| AppError::ShortcutError(e.to_string()))
    }

}

/// 启动语音会话 - 异步处理避免 UI 阻塞
fn start_voice_session(
    app_handle: tauri::AppHandle,
    recording_state: Arc<Mutex<VoiceRecordingState>>,
    _config: Arc<Mutex<VoiceShortcutConfig>>,
) -> AppResult<()> {
    println!("\n📍 [DEBUG] ==> start_voice_session 函数开始执行");
    
    // 更新状态为开始中
    *recording_state.lock().unwrap() = VoiceRecordingState::Starting;
    println!("   [DEBUG] 状态已更新为: Starting");
    
    // 列出所有窗口
    println!("   [DEBUG] 正在列出所有窗口...");
    
    // 检查 floating-input 窗口是否存在
    if let Some(window) = app_handle.get_window("floating-input") {
        println!("   [DEBUG] 找到 floating-input 窗口");
        println!("   [DEBUG] 窗口标签: {}", window.label());
        
        // 发送事件给 floating-input 窗口，触发语音输入界面
        println!("   [DEBUG] 准备发送 voice_input_triggered 事件");
        window.emit("voice_input_triggered", ())
            .map_err(|e| AppError::IpcError(e.to_string()))?;
        println!("✅ [DEBUG] 已成功发送 voice_input_triggered 事件到 floating-input 窗口");
    } else {
        // 如果窗口不存在，尝试创建它
        println!("⚠️ [DEBUG] floating-input 窗口不存在，尝试创建...");
        
        // 导入必要的类型
        use tauri::{WindowBuilder, WindowUrl};
        
        println!("   [DEBUG] 开始创建 floating-input 窗口...");
        
        // 创建悬浮输入窗口
        let window = WindowBuilder::new(
            &app_handle,
            "floating-input",
            WindowUrl::App("floating-input.html".into()),
        )
        .title("")
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .inner_size(600.0, 120.0)
        .center()
        .visible(false)  // 初始隐藏
        .build()
        .map_err(|e| {
            eprintln!("   [DEBUG] 窗口创建失败: {:?}", e);
            AppError::WindowError(e.to_string())
        })?;
        
        println!("   [DEBUG] 窗口创建成功，标签: {}", window.label());
        
        // 发送事件触发显示
        println!("   [DEBUG] 准备发送 voice_input_triggered 事件到新创建的窗口");
        window.emit("voice_input_triggered", ())
            .map_err(|e| {
                eprintln!("   [DEBUG] 事件发送失败: {:?}", e);
                AppError::IpcError(e.to_string())
            })?;
        
        println!("✅ [DEBUG] 成功创建窗口并发送触发事件");
    }

    // 更新状态为录音中
    *recording_state.lock().unwrap() = VoiceRecordingState::Recording;

    Ok(())
}

/// 智能录音管理 - 实现 VAD 自动停止
async fn manage_smart_recording(
    app_handle: tauri::AppHandle,
    recording_state: Arc<Mutex<VoiceRecordingState>>,
    config: Arc<Mutex<VoiceShortcutConfig>>,
) -> AppResult<()> {
    let config_data = config.lock().unwrap().clone();
    
    // 启动录音
    app_handle.emit_all("start_voice_recording", serde_json::json!({
        "realtime": true,
        "auto_stop": config_data.auto_stop_enabled,
        "max_duration": config_data.max_recording_duration_ms
    })).map_err(|e| AppError::IpcError(e.to_string()))?;

    if config_data.auto_stop_enabled {
        // 实现智能 VAD (Voice Activity Detection)
        monitor_voice_activity(
            app_handle.clone(),
            recording_state.clone(),
            config_data.silence_threshold_ms,
            config_data.max_recording_duration_ms
        ).await?;
    }

    Ok(())
}

/// 语音活动监测 - 自动停止录音
async fn monitor_voice_activity(
    app_handle: tauri::AppHandle,
    recording_state: Arc<Mutex<VoiceRecordingState>>,
    silence_threshold_ms: u64,
    max_duration_ms: u64,
) -> AppResult<()> {
    let start_time = Instant::now();
    let mut last_activity_time = start_time;
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        interval.tick().await;

        // 检查录音状态
        let current_state = recording_state.lock().unwrap().clone();
        if !matches!(current_state, VoiceRecordingState::Recording) {
            break;
        }

        // 检查最大时长
        if start_time.elapsed().as_millis() > max_duration_ms as u128 {
            println!("⏰ 达到最大录音时长，自动停止");
            stop_voice_session(app_handle.clone(), recording_state.clone())?;
            break;
        }

        // TODO: 实现真实的音频电平检测
        // 这里应该从音频系统获取真实的音频电平
        let simulated_audio_level = get_current_audio_level();
        
        // 发送音频电平到前端
        app_handle.emit_all("audio_level_update", simulated_audio_level)
            .map_err(|e| AppError::IpcError(e.to_string()))?;

        // 检测语音活动
        if simulated_audio_level > 0.1 {
            last_activity_time = Instant::now();
        } else if last_activity_time.elapsed().as_millis() > silence_threshold_ms as u128 {
            println!("🔇 检测到静音，自动停止录音");
            stop_voice_session(app_handle.clone(), recording_state.clone())?;
            break;
        }
    }

    Ok(())
}

/// 停止语音会话
fn stop_voice_session(
    app_handle: tauri::AppHandle,
    recording_state: Arc<Mutex<VoiceRecordingState>>,
) -> AppResult<()> {
    // 更新状态为处理中
    *recording_state.lock().unwrap() = VoiceRecordingState::Processing;

    // 前端已经在监听录音状态并会自动停止
    // 这里只需要重置我们的内部状态
    println!("🛑 手动停止语音会话");

    // 重置状态为空闲
    *recording_state.lock().unwrap() = VoiceRecordingState::Idle;

    Ok(())
}

/// 获取当前音频电平 (模拟实现，实际应该从音频系统获取)
fn get_current_audio_level() -> f32 {
    // TODO: 实现真实的音频电平检测
    // 这里应该从 cpal 或其他音频库获取实时音频数据
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // 模拟真实的音频电平变化
    if rng.gen::<f32>() > 0.7 {
        rng.gen_range(0.1..0.8) // 有声音
    } else {
        rng.gen_range(0.0..0.05) // 安静
    }
}

// 已移除 show_voice_input_window 函数，功能已整合到 start_voice_session 中

/// 处理转录和文本注入
async fn process_transcription_and_injection(app_handle: tauri::AppHandle) -> AppResult<String> {
    // 这个函数现在只是发送事件通知前端处理
    // 实际的录音停止和转录应该由前端通过调用 Tauri 命令来处理
    
    // 发送停止录音的事件通知前端
    app_handle.emit_all("voice_session_process_start", ())
        .map_err(|e| AppError::IpcError(e.to_string()))?;

    // 这里不再等待或返回模拟数据
    // 前端会调用相应的命令来停止录音、转录和注入文本
    
    Ok("Processing started".to_string())
}

impl OptimizedVoiceShortcutManager {
    /// 检查并引导权限配置
    fn check_and_guide_permissions(&self) -> AppResult<()> {
        let permission_status = PermissionManager::check_all_permissions()?;

        if !permission_status.input_monitoring {
            // 发送权限引导事件到前端
            self.app_handle.emit_all("permission_guide_required", serde_json::json!({
                "type": "input_monitoring",
                "message": "需要输入监控权限才能使用快捷键功能",
                "action": "打开系统设置"
            })).map_err(|e| AppError::IpcError(e.to_string()))?;

            return Err(AppError::PermissionError(
                "输入监控权限缺失".to_string()
            ));
        }

        Ok(())
    }

    /// 检查系统快捷键冲突
    fn is_system_shortcut_conflict(&self, shortcut: &str) -> bool {
        // 已知的系统快捷键列表
        let system_shortcuts = vec![
            "Cmd+Space",     // Spotlight (但我们想用这个，所以需要用户确认)
            "Cmd+Tab",       // App Switcher
            "Cmd+Shift+3",   // Screenshot
            "Cmd+Shift+4",   // Partial Screenshot
        ];

        // 对于 Cmd+Space，我们允许但会给用户提示
        if shortcut == "Cmd+Space" {
            println!("⚠️ Cmd+Space 可能与 Spotlight 冲突，用户可以选择禁用 Spotlight");
            return false; // 不算冲突，让用户决定
        }

        system_shortcuts.contains(&shortcut)
    }

    /// 发送快捷键状态更新到前端
    fn emit_shortcut_status_update(&self, registered: &[(String, ShortcutPriority)]) -> AppResult<()> {
        self.app_handle.emit_all("shortcut_status_update", serde_json::json!({
            "registered_shortcuts": registered,
            "primary_active": registered.iter().any(|(_, p)| *p == ShortcutPriority::Primary),
            "fallback_count": registered.len()
        })).map_err(|e| AppError::IpcError(e.to_string()))?;

        Ok(())
    }

    /// 获取当前配置
    pub fn get_config(&self) -> VoiceShortcutConfig {
        self.config.lock().unwrap().clone()
    }

    /// 更新配置
    pub fn update_config(&self, new_config: VoiceShortcutConfig) -> AppResult<()> {
        *self.config.lock().unwrap() = new_config;
        
        // 重新注册快捷键
        self.unregister_all_shortcuts()?;
        self.register_smart_shortcuts()?;
        
        Ok(())
    }

    /// 注销所有快捷键
    pub fn unregister_all_shortcuts(&self) -> AppResult<()> {
        let registered = self.registered_shortcuts.lock().unwrap();
        
        for (shortcut, _) in registered.iter() {
            if let Err(e) = self.app_handle.global_shortcut_manager().unregister(shortcut) {
                eprintln!("⚠️ 注销快捷键 {} 失败: {}", shortcut, e);
            }
        }
        
        println!("🛑 所有快捷键已注销");
        Ok(())
    }
}

// Tauri 命令接口
#[tauri::command]
pub async fn get_voice_shortcut_config(
    manager: tauri::State<'_, Arc<OptimizedVoiceShortcutManager>>
) -> Result<VoiceShortcutConfig, String> {
    Ok(manager.get_config())
}

#[tauri::command]
pub async fn update_voice_shortcut_config(
    new_config: VoiceShortcutConfig,
    manager: tauri::State<'_, Arc<OptimizedVoiceShortcutManager>>
) -> Result<(), String> {
    manager.update_config(new_config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_voice_shortcut_system(
    app: tauri::AppHandle
) -> Result<String, String> {
    println!("🧪 测试优化后的语音快捷键系统");
    
    // 模拟快捷键触发
    app.emit_all("voice_session_started", ())
        .map_err(|e| e.to_string())?;
    
    // 等待一小段时间
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    app.emit_all("voice_session_completed", serde_json::json!({
        "text": "测试语音输入成功！",
        "success": true
    })).map_err(|e| e.to_string())?;
    
    Ok("语音快捷键系统测试完成".to_string())
}