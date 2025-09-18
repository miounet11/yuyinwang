// 简化版统一权限管理 Tauri 命令接口

use crate::system::unified_permission_manager_simple::{
    PermissionType, UnifiedGuidanceInfo, UnifiedPermissionManagerSimple, UnifiedPermissionReport,
};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

/// 统一权限管理器状态
pub type UnifiedPermissionManagerState = Arc<Mutex<UnifiedPermissionManagerSimple>>;

/// 检查所有权限状态
#[tauri::command]
pub async fn unified_check_all_permissions(
    manager: State<'_, UnifiedPermissionManagerState>,
) -> Result<UnifiedPermissionReport, String> {
    let manager = manager.lock();
    manager.check_all_permissions().map_err(|e| e.to_string())
}

/// 请求特定权限
#[tauri::command]
pub async fn unified_request_permission(
    manager: State<'_, UnifiedPermissionManagerState>,
    permission: String,
) -> Result<bool, String> {
    let permission_type = match permission.as_str() {
        "microphone" => PermissionType::Microphone,
        "accessibility" => PermissionType::Accessibility,
        "input_monitoring" => PermissionType::InputMonitoring,
        _ => return Err("未知的权限类型".to_string()),
    };

    let manager = manager.lock();
    manager
        .request_permission(permission_type)
        .map_err(|e| e.to_string())
}

/// 获取权限引导信息
#[tauri::command]
pub async fn unified_get_permission_guidance(
    manager: State<'_, UnifiedPermissionManagerState>,
    permission: String,
) -> Result<UnifiedGuidanceInfo, String> {
    let permission_type = match permission.as_str() {
        "microphone" => PermissionType::Microphone,
        "accessibility" => PermissionType::Accessibility,
        "input_monitoring" => PermissionType::InputMonitoring,
        _ => return Err("未知的权限类型".to_string()),
    };

    let manager = manager.lock();
    Ok(manager.get_permission_guidance(permission_type))
}

/// 检查权限向导是否已完成
#[tauri::command]
pub async fn unified_is_wizard_completed(
    manager: State<'_, UnifiedPermissionManagerState>,
) -> Result<bool, String> {
    let manager = manager.lock();
    Ok(manager.is_wizard_completed())
}

/// 标记权限向导为已完成
#[tauri::command]
pub async fn unified_mark_wizard_completed(
    manager: State<'_, UnifiedPermissionManagerState>,
) -> Result<(), String> {
    let manager = manager.lock();
    manager.mark_wizard_completed();
    Ok(())
}

/// 开始权限状态监控
#[tauri::command]
pub async fn unified_start_permission_monitoring(
    manager: State<'_, UnifiedPermissionManagerState>,
) -> Result<(), String> {
    let manager = manager.lock();
    manager.start_monitoring().map_err(|e| e.to_string())
}

/// 停止权限状态监控
#[tauri::command]
pub async fn unified_stop_permission_monitoring(
    manager: State<'_, UnifiedPermissionManagerState>,
) -> Result<(), String> {
    let manager = manager.lock();
    manager.stop_monitoring();
    Ok(())
}

/// 获取快速权限状态检查
#[tauri::command]
pub async fn unified_quick_permission_check(
    manager: State<'_, UnifiedPermissionManagerState>,
) -> Result<serde_json::Value, String> {
    let manager = manager.lock();

    // 快速检查关键权限
    let report = manager.check_all_permissions().map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "all_critical_granted": report.all_critical_granted,
        "missing_critical_count": report.missing_critical.len(),
        "missing_critical": report.missing_critical,
        "can_use_shortcuts": report.permissions.get(&PermissionType::InputMonitoring)
            .map(|s| matches!(s, crate::system::unified_permission_manager_simple::UnifiedPermissionStatus::Granted))
            .unwrap_or(false),
        "can_record_audio": report.permissions.get(&PermissionType::Microphone)
            .map(|s| matches!(s, crate::system::unified_permission_manager_simple::UnifiedPermissionStatus::Granted))
            .unwrap_or(false),
        "wizard_needed": !manager.is_wizard_completed() && !report.all_critical_granted,
        "check_timestamp": report.check_timestamp,
    }))
}

/// 测试权限功能
#[tauri::command]
pub async fn unified_test_permissions(
    manager: State<'_, UnifiedPermissionManagerState>,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let manager = manager.lock();

    println!("🧪 开始权限功能测试...");

    let mut test_results = serde_json::Map::new();

    // 测试权限检查速度
    let start_time = std::time::Instant::now();
    let report = manager.check_all_permissions().map_err(|e| e.to_string())?;
    let check_duration = start_time.elapsed();

    test_results.insert(
        "permission_check_duration_ms".to_string(),
        serde_json::Value::Number(serde_json::Number::from(check_duration.as_millis() as u64)),
    );

    // 测试各项权限状态
    for (permission, status) in &report.permissions {
        test_results.insert(
            format!("{:?}_status", permission).to_lowercase(),
            serde_json::Value::String(format!("{:?}", status)),
        );
    }

    // 测试关键功能可用性
    test_results.insert(
        "shortcuts_available".to_string(),
        serde_json::Value::Bool(report.all_critical_granted),
    );

    test_results.insert("recording_available".to_string(),
        serde_json::Value::Bool(
            report.permissions.get(&PermissionType::Microphone)
                .map(|s| matches!(s, crate::system::unified_permission_manager_simple::UnifiedPermissionStatus::Granted))
                .unwrap_or(false)
        ));

    // 发送测试完成事件
    let _ = app_handle.emit_all("permission_test_completed", &test_results);

    println!("✅ 权限功能测试完成");
    Ok(serde_json::Value::Object(test_results))
}

/// 获取权限状态摘要（用于UI显示）
#[tauri::command]
pub async fn unified_get_permission_summary(
    manager: State<'_, UnifiedPermissionManagerState>,
) -> Result<serde_json::Value, String> {
    let manager = manager.lock();
    let report = manager.check_all_permissions().map_err(|e| e.to_string())?;

    let mut summary = serde_json::Map::new();

    // 总体状态
    summary.insert(
        "overall_status".to_string(),
        serde_json::Value::String(if report.all_critical_granted {
            "ready".to_string()
        } else {
            "needs_setup".to_string()
        }),
    );

    summary.insert(
        "total_permissions".to_string(),
        serde_json::Value::Number(serde_json::Number::from(report.permissions.len())),
    );

    summary.insert("granted_count".to_string(),
        serde_json::Value::Number(serde_json::Number::from(
            report.permissions.values()
                .filter(|s| matches!(s, crate::system::unified_permission_manager_simple::UnifiedPermissionStatus::Granted))
                .count()
        )));

    // 具体权限状态
    let mut permissions_detail = serde_json::Map::new();

    for (permission_type, status) in &report.permissions {
        let mut permission_info = serde_json::Map::new();
        permission_info.insert(
            "status".to_string(),
            serde_json::Value::String(format!("{:?}", status)),
        );
        permission_info.insert(
            "is_critical".to_string(),
            serde_json::Value::Bool(matches!(
                permission_type,
                PermissionType::Microphone | PermissionType::InputMonitoring
            )),
        );
        permission_info.insert(
            "friendly_name".to_string(),
            serde_json::Value::String(match permission_type {
                PermissionType::Microphone => "麦克风权限".to_string(),
                PermissionType::Accessibility => "辅助功能权限".to_string(),
                PermissionType::InputMonitoring => "输入监控权限".to_string(),
            }),
        );

        permissions_detail.insert(
            format!("{:?}", permission_type).to_lowercase(),
            serde_json::Value::Object(permission_info),
        );
    }

    summary.insert(
        "permissions".to_string(),
        serde_json::Value::Object(permissions_detail),
    );

    // 推荐操作
    summary.insert(
        "recommended_action".to_string(),
        serde_json::Value::String(if report.all_critical_granted {
            if manager.is_wizard_completed() {
                "all_ready".to_string()
            } else {
                "mark_wizard_completed".to_string()
            }
        } else {
            "run_permission_wizard".to_string()
        }),
    );

    summary.insert(
        "wizard_completed".to_string(),
        serde_json::Value::Bool(manager.is_wizard_completed()),
    );

    Ok(serde_json::Value::Object(summary))
}

/// 重置权限状态（用于测试和故障排除）
#[tauri::command]
pub async fn unified_reset_permission_state(
    manager: State<'_, UnifiedPermissionManagerState>,
) -> Result<(), String> {
    let manager = manager.lock();
    manager.reset_permission_state();
    println!("🔄 权限状态已重置");
    Ok(())
}
