use tauri::Manager;
use serde_json::json;

#[tauri::command]
pub async fn run_full_diagnostics(app: tauri::AppHandle) -> Result<String, String> {
    // 权限检查
    let mic = crate::commands::check_permission("microphone".to_string()).await.unwrap_or(false);
    let acc = crate::commands::check_permission("accessibility".to_string()).await.unwrap_or(false);
    let input_mon = crate::commands::check_permission("input-monitoring".to_string()).await.unwrap_or(false);

    // 音频录制快速测试
    let (audio_ok, sample_count, audio_max, audio_rms) = {
        let state = app.state::<crate::AppState>();
        let mut recorder = state.audio_recorder.lock();
        let _ = recorder.start_recording();
        drop(recorder);
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        let mut recorder = state.audio_recorder.lock();
        match recorder.stop_recording() {
            Ok(data) => {
                let max = data.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
                let rms = if data.is_empty() { 0.0 } else { (data.iter().map(|x| x*x).sum::<f32>() / data.len() as f32).sqrt() };
                (data.len() > 0, data.len(), max, rms)
            }
            Err(_) => (false, 0, 0.0, 0.0)
        }
    };

    // 流式/渐进式快速测试
    let streaming_ok = {
        let start = crate::commands::start_progressive_voice_input(None, app.clone(), Some(false)).await.is_ok();
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        let stop = crate::commands::stop_voice_recording(app.clone()).await.is_ok();
        start && stop
    };

    let result = json!({
        "permissions": {
            "microphone": mic,
            "accessibility": acc,
            "input_monitoring": input_mon
        },
        "audio": {
            "ok": audio_ok,
            "sample_count": sample_count,
            "max_level": audio_max,
            "rms": audio_rms
        },
        "streaming": {
            "ok": streaming_ok
        }
    });

    Ok(result.to_string())
}

#[tauri::command]
pub async fn run_self_repair(app: tauri::AppHandle) -> Result<String, String> {
    let mut actions: Vec<String> = Vec::new();

    // 权限引导
    let mic = crate::commands::check_permission("microphone".to_string()).await.unwrap_or(false);
    if !mic {
        let _ = crate::commands::open_system_preferences("microphone".to_string()).await;
        actions.push("opened_system_preferences:microphone".into());
    }
    let acc = crate::commands::check_permission("accessibility".to_string()).await.unwrap_or(false);
    if !acc {
        let _ = crate::commands::open_system_preferences("accessibility".to_string()).await;
        actions.push("opened_system_preferences:accessibility".into());
    }
    let input_mon = crate::commands::check_permission("input-monitoring".to_string()).await.unwrap_or(false);
    if !input_mon {
        let _ = crate::commands::open_system_preferences("input-monitoring".to_string()).await;
        actions.push("opened_system_preferences:input-monitoring".into());
    }

    // 重启 Fn/Alt+Space 监听
    let sm = crate::shortcuts::ShortcutManager::new(app.clone());
    if sm.start_fn_key_listener().is_ok() {
        actions.push("restart_fn_listener".into());
    }

    // 重新注册全局快捷键
    if let Ok(manager) = crate::shortcuts::EnhancedShortcutManager::new(app.clone()) {
        if manager.register_shortcuts().is_ok() {
            actions.push("register_shortcuts".into());
        }
    }

    // 重置录音器
    {
        let state = app.state::<crate::AppState>();
        let mut recorder = state.audio_recorder.lock();
        recorder.force_reset();
        actions.push("recorder_reset".into());
    }

    let result = json!({
        "repaired": actions
    });
    Ok(result.to_string())
}