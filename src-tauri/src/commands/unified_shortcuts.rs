use crate::shortcuts::{PerformanceReport, ShortcutMetrics};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, State};

/// 应用状态类型
type AppState = crate::AppState;

/// 快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfigRequest {
    pub shortcut_id: String,
    pub key_combination: String,
    pub description: Option<String>,
}

/// 注册快捷键
#[tauri::command]
pub async fn register_unified_shortcut(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    config: ShortcutConfigRequest,
) -> Result<String, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    let shortcut_config = crate::shortcuts::unified_shortcut_manager::ShortcutConfig {
        shortcut_id: config.shortcut_id.clone(),
        key_combination: config.key_combination.clone(),
        description: config
            .description
            .unwrap_or_else(|| "用户自定义快捷键".to_string()),
        shortcut_type: crate::shortcuts::unified_shortcut_manager::ShortcutType::Custom(
            "user_defined".to_string(),
        ),
        trigger_mode: crate::shortcuts::unified_shortcut_manager::TriggerMode::Press,
        enabled: true,
        priority: 50,
    };

    manager
        .register_shortcut(shortcut_config)
        .map_err(|e| e.to_string())?;

    Ok(format!("快捷键 '{}' 注册成功", config.shortcut_id))
}

/// 注销快捷键
#[tauri::command]
pub async fn unregister_unified_shortcut(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    shortcut_id: String,
) -> Result<String, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    manager
        .unregister_shortcut(&shortcut_id)
        .map_err(|e| e.to_string())?;

    Ok(format!("快捷键 '{}' 注销成功", shortcut_id))
}

/// 获取所有已注册的统一快捷键
#[tauri::command]
pub async fn get_unified_registered_shortcuts(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    Ok(manager.get_registered_shortcuts())
}

/// 检查快捷键冲突
#[tauri::command]
pub async fn check_shortcut_conflict(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    key_combination: String,
) -> Result<Option<String>, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    Ok(manager.check_conflict(&key_combination))
}

/// 获取可用的预设方案
#[tauri::command]
pub async fn get_shortcut_presets(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    Ok(manager.get_available_presets())
}

/// 应用预设方案
#[tauri::command]
pub async fn apply_shortcut_preset(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<String, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    manager
        .apply_preset(&preset_id)
        .map_err(|e| e.to_string())?;

    Ok(format!("预设方案 '{}' 应用成功", preset_id))
}

/// 获取快捷键性能指标
#[tauri::command]
pub async fn get_shortcut_metrics(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<ShortcutMetrics>, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    Ok(manager.get_performance_metrics())
}

/// 获取性能报告
#[tauri::command]
pub async fn get_performance_report(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<PerformanceReport, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    Ok(manager.get_performance_report())
}

/// 运行快捷键基准测试
#[tauri::command]
pub async fn run_shortcut_benchmark(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    iterations: Option<u32>,
) -> Result<String, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    let iterations = iterations.unwrap_or(100);
    let report = manager
        .run_benchmark(iterations)
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "基准测试完成 - 平均响应时间: {:.2}ms, 最快: {}ms, 最慢: {}ms",
        report.average_response_time_ms, report.fastest_response_ms, report.slowest_response_ms
    ))
}

/// 导出快捷键配置
#[tauri::command]
pub async fn export_shortcut_config(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    manager.export_config().map_err(|e| e.to_string())
}

/// 导入快捷键配置
#[tauri::command]
pub async fn import_shortcut_config(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    config_json: String,
) -> Result<String, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    manager
        .import_config(&config_json)
        .map_err(|e| e.to_string())?;

    Ok("快捷键配置导入成功".to_string())
}

/// 重置所有快捷键
#[tauri::command]
pub async fn reset_all_shortcuts(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.unified_shortcut_manager.clone();
    let manager = manager.lock();

    manager.reset_all().map_err(|e| e.to_string())?;

    Ok("所有快捷键已重置".to_string())
}
