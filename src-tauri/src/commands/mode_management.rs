// Story 1.4: Tauri Commands for Transcription Mode Management

use crate::network::{NetworkMonitor, TranscriptionMode, TranscriptionModeManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, Window};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeStatus {
    pub current_mode: String,
    pub active_mode: String,
    pub user_preferred_mode: String,
    pub network_status: String,
    pub network_quality: f64,
    pub auto_switch_enabled: bool,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub auto_switch_enabled: bool,
    pub cloud_api_timeout_ms: u64,
    pub local_model_priority: bool,
    pub network_quality_threshold: f64,
    pub switch_debounce_ms: u64,
}

/// 获取当前转录模式状态
#[tauri::command]
pub async fn get_transcription_mode_status(
    mode_manager: State<'_, Arc<TranscriptionModeManager>>,
    network_monitor: State<'_, Arc<NetworkMonitor>>,
) -> Result<ModeStatus, String> {
    println!("🔍 获取转录模式状态...");

    let current_mode = mode_manager.get_current_mode();
    let active_mode = mode_manager.get_active_mode();
    let user_preferred_mode = mode_manager.get_user_preferred_mode();
    let network_status = network_monitor.get_current_status();
    let network_quality = network_monitor.get_connection_quality_score();
    let config = mode_manager.get_config();

    let (_, recommendation) = mode_manager
        .get_mode_recommendation()
        .await
        .map_err(|e| format!("Failed to get recommendation: {}", e))?;

    Ok(ModeStatus {
        current_mode: format!("{:?}", current_mode),
        active_mode: format!("{:?}", active_mode),
        user_preferred_mode: format!("{:?}", user_preferred_mode),
        network_status: format!("{:?}", network_status),
        network_quality,
        auto_switch_enabled: config.auto_switch_enabled,
        recommendation: Some(recommendation),
    })
}

/// 设置转录模式
#[tauri::command]
pub async fn set_transcription_mode(
    mode: String,
    mode_manager: State<'_, Arc<TranscriptionModeManager>>,
    window: Window,
) -> Result<(), String> {
    println!("🎯 设置转录模式: {}", mode);

    let transcription_mode = match mode.to_lowercase().as_str() {
        "local" => TranscriptionMode::Local,
        "cloud" => TranscriptionMode::Cloud,
        "auto" => TranscriptionMode::Auto,
        "hybrid" => TranscriptionMode::Hybrid,
        _ => return Err(format!("不支持的转录模式: {}", mode)),
    };

    mode_manager
        .set_user_mode(transcription_mode)
        .await
        .map_err(|e| format!("Failed to set mode: {}", e))?;

    // 通知前端模式变化
    let _ = window.emit("mode_changed", &mode);

    println!("✅ 转录模式已设置为: {}", mode);
    Ok(())
}

/// 更新模式管理器配置
#[tauri::command]
pub async fn update_mode_config(
    config: ModeConfig,
    mode_manager: State<'_, Arc<TranscriptionModeManager>>,
) -> Result<(), String> {
    println!("🔧 更新模式管理器配置...");

    let mode_config = crate::network::transcription_mode_manager::ModeManagerConfig {
        auto_switch_enabled: config.auto_switch_enabled,
        cloud_api_timeout_ms: config.cloud_api_timeout_ms,
        local_model_priority: config.local_model_priority,
        network_quality_threshold: config.network_quality_threshold,
        switch_debounce_ms: config.switch_debounce_ms,
    };

    mode_manager.update_config(mode_config);

    println!("✅ 模式管理器配置已更新");
    Ok(())
}

/// 强制重新评估转录模式
#[tauri::command]
pub async fn force_reevaluate_mode(
    mode_manager: State<'_, Arc<TranscriptionModeManager>>,
    window: Window,
) -> Result<String, String> {
    println!("🔄 强制重新评估转录模式...");

    let new_mode = mode_manager
        .force_reevaluate()
        .await
        .map_err(|e| format!("Failed to reevaluate mode: {}", e))?;

    let mode_str = format!("{:?}", new_mode);

    // 通知前端模式变化
    let _ = window.emit("mode_reevaluated", &mode_str);

    println!("✅ 模式重新评估完成: {}", mode_str);
    Ok(mode_str)
}

/// 获取网络状态
#[tauri::command]
pub async fn get_network_status(
    network_monitor: State<'_, Arc<NetworkMonitor>>,
) -> Result<serde_json::Value, String> {
    println!("🌐 获取网络状态...");

    let status = network_monitor.get_current_status();
    let metrics = network_monitor.get_metrics();

    let result = serde_json::json!({
        "status": format!("{:?}", status),
        "is_connected": metrics.is_connected,
        "quality_score": network_monitor.get_connection_quality_score(),
        "consecutive_failures": metrics.consecutive_failures,
        "last_checked": metrics.last_checked.elapsed().as_secs()
    });

    Ok(result)
}

/// 立即检查网络状态
#[tauri::command]
pub async fn check_network_now(
    network_monitor: State<'_, Arc<NetworkMonitor>>,
    window: Window,
) -> Result<String, String> {
    println!("🌐 立即检查网络状态...");

    let status = network_monitor.check_now().await;
    let status_str = format!("{:?}", status);

    // 通知前端网络状态更新
    let _ = window.emit("network_status_updated", &status_str);

    println!("✅ 网络状态检查完成: {}", status_str);
    Ok(status_str)
}

/// 测试API端点连接
#[tauri::command]
pub async fn test_api_endpoint(
    url: String,
    network_monitor: State<'_, Arc<NetworkMonitor>>,
) -> Result<serde_json::Value, String> {
    println!("🔗 测试API端点连接: {}", url);

    match network_monitor.test_api_endpoint(&url).await {
        Ok(duration) => {
            println!("✅ API端点连接成功，延迟: {:?}", duration);
            Ok(serde_json::json!({
                "success": true,
                "latency_ms": duration.as_millis(),
                "message": "连接成功"
            }))
        }
        Err(e) => {
            println!("❌ API端点连接失败: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "latency_ms": null,
                "message": format!("连接失败: {}", e)
            }))
        }
    }
}

/// 订阅模式变化事件（WebSocket风格）
#[tauri::command]
pub async fn subscribe_mode_changes(
    mode_manager: State<'_, Arc<TranscriptionModeManager>>,
    window: Window,
) -> Result<(), String> {
    println!("📡 订阅模式变化事件...");

    let mut mode_change_rx = mode_manager.subscribe_mode_changes();

    tokio::spawn(async move {
        while let Ok(event) = mode_change_rx.recv().await {
            let event_data = serde_json::json!({
                "from_mode": format!("{:?}", event.from_mode),
                "to_mode": format!("{:?}", event.to_mode),
                "reason": event.reason,
                "automatic": event.automatic,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            if let Err(e) = window.emit("mode_change_event", &event_data) {
                eprintln!("❌ 发送模式变化事件失败: {}", e);
            } else {
                println!(
                    "📡 模式变化事件已发送: {:?} -> {:?}",
                    event.from_mode, event.to_mode
                );
            }
        }
    });

    Ok(())
}

/// 订阅网络状态变化事件
#[tauri::command]
pub async fn subscribe_network_changes(
    network_monitor: State<'_, Arc<NetworkMonitor>>,
    window: Window,
) -> Result<(), String> {
    println!("📡 订阅网络状态变化事件...");

    let mut network_status_rx = network_monitor.subscribe_status_changes();
    let network_monitor_clone = network_monitor.inner().clone();

    tokio::spawn(async move {
        while let Ok(status) = network_status_rx.recv().await {
            let event_data = serde_json::json!({
                "status": format!("{:?}", status),
                "quality_score": network_monitor_clone.get_connection_quality_score(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            if let Err(e) = window.emit("network_status_event", &event_data) {
                eprintln!("❌ 发送网络状态事件失败: {}", e);
            } else {
                println!("📡 网络状态事件已发送: {:?}", status);
            }
        }
    });

    Ok(())
}
