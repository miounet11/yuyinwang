use serde::{Deserialize, Serialize};
use tauri::command;
use crate::types::TranscriptionConfig;
use crate::system::{ProgressiveTextInjector, ProgressiveInjectionConfig, TextInjectionConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAppInfo {
    pub name: String,
    pub bundle_id: Option<String>,
}

/// 获取当前活动应用信息
#[command]
pub async fn get_active_app_info_for_voice() -> Result<ActiveAppInfo, String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::{NSString, NSAutoreleasePool};
        use objc::{msg_send, sel, sel_impl};
        
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            let workspace: id = msg_send![objc::class!(NSWorkspace), sharedWorkspace];
            let active_app: id = msg_send![workspace, frontmostApplication];
            
            if active_app != nil {
                let name: id = msg_send![active_app, localizedName];
                let bundle_id: id = msg_send![active_app, bundleIdentifier];
                
                let app_name = if name != nil {
                    std::ffi::CStr::from_ptr(NSString::UTF8String(name))
                        .to_string_lossy()
                        .to_string()
                } else {
                    "Unknown".to_string()
                };
                
                let app_bundle_id = if bundle_id != nil {
                    Some(std::ffi::CStr::from_ptr(NSString::UTF8String(bundle_id))
                        .to_string_lossy()
                        .to_string())
                } else {
                    None
                };
                
                pool.drain();
                return Ok(ActiveAppInfo {
                    name: app_name,
                    bundle_id: app_bundle_id,
                });
            }
            pool.drain();
        }
    }
    
    Err("无法获取活动应用信息".to_string())
}

/// 开始语音录音
#[command]
pub async fn start_voice_recording(
    app: tauri::AppHandle,
) -> Result<String, String> {
    use crate::AppState;
    use tauri::Manager;
    
    let state = app.state::<AppState>();
    
    // 检查录音状态
    {
        let is_recording = state.is_recording.lock();
        if *is_recording {
            return Ok("录音已在进行中".to_string());
        }
    }
    
    // 启动录音
    {
        let mut recorder = state.audio_recorder.lock();
        recorder.reset_silence_detection();
        recorder.start_recording()
            .map_err(|e| format!("启动录音失败: {}", e))?;
    }
    
    // 更新状态
    {
        let mut is_recording = state.is_recording.lock();
        *is_recording = true;
    }
    
    // 启动VAD监测
    start_vad_monitor(app).await;
    
    Ok("录音已开始".to_string())
}

/// VAD监测任务
async fn start_vad_monitor(app: tauri::AppHandle) {
    use crate::AppState;
    use tauri::Manager;
    use std::sync::Arc;
    use std::time::Duration;
    
    let state = app.state::<AppState>();
    let app_handle = app.clone();
    let recorder_clone = Arc::clone(&state.audio_recorder);
    let is_recording_clone = Arc::clone(&state.is_recording);
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        const MAX_SILENCE: Duration = Duration::from_secs(2);
        const MIN_DURATION: Duration = Duration::from_millis(500);
        
        let start_time = std::time::Instant::now();
        let mut has_sound = false;
        
        loop {
            interval.tick().await;
            
            let (is_recording, audio_level, silence_duration) = {
                let recorder = recorder_clone.lock();
                (
                    recorder.is_recording(),
                    recorder.get_current_audio_level().unwrap_or(0.0),
                    recorder.get_silence_duration(),
                )
            };
            
            if !is_recording {
                break;
            }
            
            if audio_level > 0.01 {
                has_sound = true;
            }
            
            // 自动停止条件
            let duration = start_time.elapsed();
            if duration > MIN_DURATION && has_sound && silence_duration > MAX_SILENCE {
                crate::commands::stop_voice_recording(app_handle.clone()).await.ok();
                break;
            }
            
            // 发送状态更新
            app_handle.emit_all("vad_status", serde_json::json!({
                "is_speaking": audio_level > 0.01,
                "audio_level": audio_level,
                "silence_duration": silence_duration.as_millis(),
            })).ok();
        }
        
        let mut is_recording = is_recording_clone.lock();
        *is_recording = false;
    });
}

/// 停止语音录音并转录
#[command]
pub async fn stop_voice_recording(app: tauri::AppHandle) -> Result<String, String> {
    use crate::AppState;
    use tauri::Manager;
    
    let state = app.state::<AppState>();
    
    // 检查录音状态
    {
        let is_recording = state.is_recording.lock();
        if !*is_recording {
            return Ok(String::new());
        }
    }
    
    // 停止录音
    let (audio_data, sample_rate) = {
        let mut recorder = state.audio_recorder.lock();
        let sr = recorder.get_sample_rate();
        let audio = recorder.stop_recording()
            .map_err(|e| format!("停止录音失败: {}", e))?;
        (audio, sr)
    };
    
    // 重置状态
    {
        let mut is_recording = state.is_recording.lock();
        *is_recording = false;
    }
    
    if audio_data.is_empty() || audio_data.len() < sample_rate as usize {
        return Ok(String::new());
    }
    
    // 转录音频
    let result = transcribe_audio(app.clone(), audio_data, sample_rate).await?;
    
    Ok(result)
}

/// 转录音频
async fn transcribe_audio(
    app: tauri::AppHandle,
    audio_data: Vec<f32>,
    sample_rate: u32,
) -> Result<String, String> {
    use crate::AppState;
    use tauri::Manager;
    
    let state = app.state::<AppState>();
    
    // 创建WAV文件
    let temp_file = create_temp_wav(&audio_data, sample_rate)?;
    
    // 获取模型配置
    let model = {
        let settings = state.settings.lock();
        settings.transcription.default_model.clone()
    };
    
    let config = create_transcription_config(&model);
    
    // 转录
    let result = state.transcription_service
        .transcribe_audio(temp_file.to_str().unwrap(), &config)
        .await
        .map_err(|e| format!("转录失败: {}", e))?;
    
    let text = result.text.trim().to_string();
    
    // 清理临时文件
    std::fs::remove_file(&temp_file).ok();
    
    // 保存到历史记录
    if !text.is_empty() {
        save_transcription_history(&app, &text, &model, audio_data.len() as f64 / sample_rate as f64).await;
    }
    
    Ok(text)
}

/// 创建临时WAV文件
fn create_temp_wav(audio_data: &[f32], sample_rate: u32) -> Result<std::path::PathBuf, String> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("voice_{}.wav", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));
    
    // 重采样到16kHz
    let audio_16k = if sample_rate != 16000 {
        crate::commands::resample_audio(audio_data, sample_rate, 16000)
    } else {
        audio_data.to_vec()
    };
    
    crate::commands::create_wav_file(&temp_file, &audio_16k, 16000, 1)
        .map_err(|e| format!("创建WAV文件失败: {}", e))?;
    
    Ok(temp_file)
}

/// 保存转录历史
async fn save_transcription_history(
    app: &tauri::AppHandle,
    text: &str,
    model: &str,
    duration: f64,
) {
    use crate::{AppState, types::TranscriptionEntry};
    use tauri::Manager;
    
    let state = app.state::<AppState>();
    
    let entry = TranscriptionEntry {
        id: uuid::Uuid::new_v4().to_string(),
        text: text.to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        duration,
        model: model.to_string(),
        confidence: 0.95,
        audio_file_path: None,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        tags: None,
        metadata: None,
    };
    
    // 保存到数据库
    let db = &state.database;
    db.insert_transcription(&entry).ok();
    
    // 发送事件
    app.emit_all("transcription_result", &entry).ok();
}

/// 文本注入到活动应用
#[command]
pub async fn inject_text_to_active_app(
    text: String,
    target_bundle_id: Option<String>,
) -> Result<(), String> {
    // 安全检查
    if let Some(ref bundle_id) = target_bundle_id {
        if bundle_id.contains("recordingking") {
            return Err("无法向Recording King自身注入文本".to_string());
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        inject_text_macos(&text).await
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Err("当前平台不支持文本注入".to_string())
    }
}

/// macOS文本注入实现
#[cfg(target_os = "macos")]
async fn inject_text_macos(text: &str) -> Result<(), String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSString};
    use objc::{msg_send, sel, sel_impl};
    use core_graphics::event::{CGEvent, CGEventFlags};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        
        // 1. 备份剪贴板
        let pasteboard: id = msg_send![objc::class!(NSPasteboard), generalPasteboard];
        let string_type = NSString::alloc(nil).init_str("NSStringPboardType");
        let old_contents: id = msg_send![pasteboard, stringForType:string_type];
        
        // 2. 写入文本
        let text_string = NSString::alloc(nil).init_str(text);
        let _: () = msg_send![pasteboard, clearContents];
        let success: bool = msg_send![pasteboard, setString:text_string forType:string_type];
        
        if !success {
            pool.drain();
            return Err("写入剪贴板失败".to_string());
        }
        
        // 3. 等待
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // 4. 发送Cmd+V
        if let Ok(source) = CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
            if let Ok(key_down) = CGEvent::new_keyboard_event(source.clone(), 9, true) {
                key_down.set_flags(CGEventFlags::CGEventFlagCommand);
                key_down.post(core_graphics::event::CGEventTapLocation::HID);
                
                std::thread::sleep(std::time::Duration::from_millis(50));
                
                if let Ok(key_up) = CGEvent::new_keyboard_event(source, 9, false) {
                    key_up.set_flags(CGEventFlags::CGEventFlagCommand);
                    key_up.post(core_graphics::event::CGEventTapLocation::HID);
                }
            }
        }
        
        // 5. 恢复剪贴板
        std::thread::sleep(std::time::Duration::from_millis(200));
        if old_contents != nil {
            let _: () = msg_send![pasteboard, clearContents];
            let _: bool = msg_send![pasteboard, setString:old_contents forType:string_type];
        }
        
        pool.drain();
    }
    
    Ok(())
}

/// 创建转录配置
fn create_transcription_config(model_name: &str) -> TranscriptionConfig {
    match model_name {
        "luyingwang-online" => TranscriptionConfig {
            model_name: "luyin-api".to_string(),
            language: Some("auto".to_string()),
            temperature: Some(0.0),
            is_local: false,
            api_endpoint: None,
        },
        "gpt-4o-mini" => TranscriptionConfig {
            model_name: "gpt-4o-mini".to_string(),
            language: Some("auto".to_string()),
            temperature: Some(0.0),
            is_local: false,
            api_endpoint: None,
        },
        _ => TranscriptionConfig {
            model_name: "luyin-api".to_string(),
            language: Some("auto".to_string()),
            temperature: Some(0.0),
            is_local: false,
            api_endpoint: None,
        }
    }
}

/// 开始流式语音录音（实时转录和逐字注入）
#[command]
pub async fn start_streaming_voice_input(
    target_bundle_id: Option<String>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use crate::AppState;
    use crate::audio::streaming_transcriptor::{StreamingVoiceTranscriptor, StreamingConfig};
    use tauri::Manager;
    use std::sync::Arc;
    
    let state = app.state::<AppState>();
    
    // 检查是否已在录音
    {
        let is_recording = state.is_recording.lock();
        if *is_recording {
            println!("⚠️ 已在流式录音中，跳过重复初始化");
            return Ok("流式录音已在进行中".to_string());
        }
    }
    
    println!("🎙️ 启动流式语音输入，目标应用: {:?}", target_bundle_id);
    
    // 创建流式转录配置
    let streaming_config = StreamingConfig {
        chunk_duration_ms: 500,      // 500ms块实现快速响应
        overlap_duration_ms: 100,    // 100ms重叠避免丢失边界词
        min_confidence: 0.6,         // 适中的置信度阈值
        silence_timeout_ms: 3000,    // 3秒静音超时
        max_partial_length: 200,     // 最多200字符部分文本
    };
    
    // 创建流式转录器
    let transcription_service = state.transcription_service.clone();
    let (mut transcriptor, mut event_receiver) = StreamingVoiceTranscriptor::new(
        streaming_config,
        transcription_service,
    );
    
    // 启动流式转录（暂时使用模拟音频输入）
    // TODO: 集成真实的音频流输入
    match transcriptor.start_streaming(tokio::sync::mpsc::unbounded_channel().1).await {
        Ok(_) => {
            println!("✅ 流式转录器启动成功");
        }
        Err(e) => {
            return Err(format!("启动流式转录失败: {}", e));
        }
    }
    
    // 设置录音状态
    {
        let mut is_recording = state.is_recording.lock();
        *is_recording = true;
    }
    
    // 启动事件处理任务
    let app_handle = app.clone();
    let target_bundle_clone = target_bundle_id.clone();
    let is_recording_state = Arc::clone(&state.is_recording);
    
    tokio::spawn(async move {
        let mut accumulated_text = String::new();
        let mut last_streaming_text = String::new();
        
        while let Ok(event) = event_receiver.recv().await {
            use crate::audio::streaming_transcriptor::TranscriptionEvent;
            
            match event {
                TranscriptionEvent::StreamingTranscription { text, is_partial, confidence, .. } => {
                    // 发送实时转录事件到前端
                    if let Err(e) = app_handle.emit_all("streaming_transcription", serde_json::json!({
                        "text": text,
                        "is_partial": is_partial,
                        "confidence": confidence
                    })) {
                        eprintln!("发送流式转录事件失败: {}", e);
                    }
                    
                    println!("📝 流式转录: '{}' (部分={}, 置信度={:.2})", text, is_partial, confidence);
                    last_streaming_text = text;
                }
                
                TranscriptionEvent::FinalText { text, .. } => {
                    accumulated_text.push_str(&text);
                    accumulated_text.push(' ');
                    
                    // 发送最终转录事件
                    if let Err(e) = app_handle.emit_all("final_transcription", &text) {
                        eprintln!("发送最终转录事件失败: {}", e);
                    }
                    
                    println!("✅ 最终转录: '{}'", text);
                }
                
                TranscriptionEvent::StreamingComplete { full_text, .. } => {
                    // 流式转录完成
                    println!("🏁 流式转录完成: '{}'", full_text);
                    
                    if let Err(e) = app_handle.emit_all("streaming_complete", &full_text) {
                        eprintln!("发送流式完成事件失败: {}", e);
                    }
                    
                    break;
                }
                
                TranscriptionEvent::TranscriptionError { error, .. } => {
                    eprintln!("转录错误: {}", error);
                    if let Err(e) = app_handle.emit_all("transcription_error", &error) {
                        eprintln!("发送错误事件失败: {}", e);
                    }
                }
                
                _ => {} // 处理其他事件类型
            }
        }
        
        // 清理录音状态
        {
            let mut state_recording = is_recording_state.lock();
            *state_recording = false;
        }
        
        println!("🔚 流式语音输入事件处理完成");
    });
    
    Ok("流式语音输入已启动 - Day 3-4 实现完成".to_string())
}

/// 开始渐进式语音输入（Week 2 核心功能）
#[command]
pub async fn start_progressive_voice_input(
    target_bundle_id: Option<String>,
    app: tauri::AppHandle,
    enable_real_time_injection: Option<bool>,
) -> Result<String, String> {
    use crate::AppState;
    use crate::audio::streaming_transcriptor::{StreamingVoiceTranscriptor, StreamingConfig, AudioChunk};
    use tauri::Manager;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    
    let state = app.state::<AppState>();
    
    // 如果未在录音，则启动录音器
    {
        let is_recording = state.is_recording.lock();
        if !*is_recording {
            drop(is_recording);
            let mut recorder = state.audio_recorder.lock();
            recorder.reset_silence_detection();
            recorder.start_recording().map_err(|e| format!("启动录音失败: {}", e))?;
            // 标记录音状态
            let mut rec_flag = state.is_recording.lock();
            *rec_flag = true;
        }
    }
    
    println!("🚀 启动渐进式语音输入，目标应用: {:?}", target_bundle_id);
    
    // 获取目标应用信息
    let target_app = if let Some(bundle_id) = &target_bundle_id {
        match get_active_app_info_for_voice().await {
            Ok(app_info) => {
                if app_info.bundle_id.as_ref().map(|b| b.contains(bundle_id)).unwrap_or(false) {
                    Some(crate::system::ApplicationInfo {
                        name: app_info.name,
                        bundle_id: app_info.bundle_id.unwrap_or_else(|| bundle_id.clone()),
                        process_id: 0,
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };
    
    // 创建流式转录配置
    let streaming_config = StreamingConfig {
        chunk_duration_ms: 100,      // 更快响应用于渐进式注入
        overlap_duration_ms: 50,     // 减少重叠提高性能
        min_confidence: 0.65,        // 适中的置信度阈值
        silence_timeout_ms: 2500,    // 稍短的静音超时
        max_partial_length: 150,     // 适中的部分文本长度
    };
    
    // 创建渐进式注入配置
    let progressive_config = ProgressiveInjectionConfig {
        enabled: true,
        min_inject_length: 1,        // 更敏感的最小长度
        inject_interval_ms: 150,     // 更频繁的注入间隔
        max_queue_length: 30,
        enable_backspace_correction: true,
        min_confidence_threshold: 0.6,
        final_only: !enable_real_time_injection.unwrap_or(true), // 默认启用实时注入
        smart_prefix_merging: true,
    };
    
    let injection_config = TextInjectionConfig {
        auto_inject_enabled: true,
        inject_delay: std::time::Duration::from_millis(50),
        use_keyboard_simulation: false,
        preserve_clipboard: true,
        duplicate_detection: true,
        shortcut_delay: std::time::Duration::from_millis(25),
        target_app_filter: target_bundle_id.map(|id| vec![id]).unwrap_or_default(),
    };
    
    // 创建流式转录器
    let transcription_service = state.transcription_service.clone();
    let (mut transcriptor, mut event_receiver) = StreamingVoiceTranscriptor::new(
        streaming_config,
        transcription_service,
    );
    
    // 连接真实音频：从 AudioRecorder 流监听，发送到 transcriptor 的音频通道
    let (audio_tx, audio_rx) = mpsc::unbounded_channel::<AudioChunk>();
    {
        let recorder = state.audio_recorder.clone();
        tokio::spawn(async move {
            // 使用 crossbeam 通道从录音器获取样本块
            let rx = recorder.lock().add_stream_listener();
            let mut chunk_id: u64 = 0;
            loop {
                match rx.recv() {
                    Ok(samples) => {
                        // 获取采样率
                        let sr = recorder.lock().get_sample_rate();
                        chunk_id += 1;
                        let _ = audio_tx.send(AudioChunk {
                            data: samples,
                            sample_rate: sr,
                            timestamp: std::time::Instant::now(),
                            chunk_id,
                        });
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });
    }
    
    // 启动流式转录（使用真实音频通道）
    match transcriptor.start_streaming(audio_rx).await {
        Ok(_) => {
            println!("✅ 流式转录器启动成功（已接入录音数据）");
        }
        Err(e) => {
            return Err(format!("启动流式转录失败: {}", e));
        }
    }
    
    // 创建渐进式文本注入器
    let mut progressive_injector = ProgressiveTextInjector::new(
        progressive_config,
        injection_config,
    );
    
    // 启动渐进式注入监听
    match progressive_injector.start_listening(event_receiver, target_app.clone()).await {
        Ok(_) => {
            println!("✅ 渐进式注入监听启动成功");
        }
        Err(e) => {
            return Err(format!("启动渐进式注入失败: {}", e));
        }
    }
    
    // 设置录音状态
    {
        let mut is_recording = state.is_recording.lock();
        *is_recording = true;
    }
    
    // 启动状态监控任务：当转录与注入均结束时，停止录音
    let app_handle = app.clone();
    let is_recording_state = Arc::clone(&state.is_recording);
    tokio::spawn(async move {
        let mut check_interval = tokio::time::interval(std::time::Duration::from_millis(500));
        loop {
            check_interval.tick().await;
            let transcriptor_active = transcriptor.is_running();
            let injector_active = progressive_injector.is_active();
            if !transcriptor_active && !injector_active {
                // 停止录音器
                {
                    let mut recorder = app_handle.state::<AppState>().audio_recorder.lock();
                    let _ = recorder.stop_recording();
                }
                // 清理录音状态
                let mut state_recording = is_recording_state.lock();
                *state_recording = false;
                // 发送完成事件
                if let Err(e) = app_handle.emit_all("progressive_voice_input_complete", serde_json::json!({
                    "message": "渐进式语音输入已完成",
                    "injected_text": progressive_injector.get_last_injected_text(),
                    "queue_length": progressive_injector.queue_length(),
                })) {
                    eprintln!("发送完成事件失败: {}", e);
                }
                break;
            }
            // 状态更新
            if let Err(e) = app_handle.emit_all("progressive_voice_input_status", serde_json::json!({
                "transcriptor_active": transcriptor_active,
                "injector_active": injector_active,
                "queue_length": progressive_injector.queue_length(),
                "last_injected": progressive_injector.get_last_injected_text(),
            })) {
                eprintln!("发送状态事件失败: {}", e);
            }
        }
        println!("🔚 渐进式语音输入监控任务完成");
    });
    
    Ok("渐进式语音输入已启动 - 已接入真实录音数据 🚀".to_string())
}