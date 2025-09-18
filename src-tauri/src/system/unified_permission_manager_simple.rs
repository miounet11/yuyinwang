// 简化版统一权限管理器 - 专注于解决Story 1.1的核心需求
// 整合现有三套快捷键管理器的权限检查，提供统一的权限管理接口

use crate::errors::AppResult;
use crate::system::permission_manager::PermissionManager;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

/// 权限类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionType {
    Microphone,
    Accessibility,
    InputMonitoring,
}

/// 统一权限状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnifiedPermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
}

/// 权限检查报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPermissionReport {
    pub permissions: HashMap<PermissionType, UnifiedPermissionStatus>,
    pub all_critical_granted: bool,
    pub missing_critical: Vec<PermissionType>,
    pub missing_optional: Vec<PermissionType>,
    pub check_timestamp: u64,
    pub next_check_recommended: u64,
}

/// 权限引导信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedGuidanceInfo {
    pub permission_type: PermissionType,
    pub current_status: UnifiedPermissionStatus,
    pub is_critical: bool,
    pub title: String,
    pub description: String,
    pub steps: Vec<GuidanceStep>,
    pub troubleshooting: Vec<String>,
}

/// 引导步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidanceStep {
    pub step_number: u8,
    pub title: String,
    pub description: String,
    pub action_type: String,
    pub action_data: Option<String>,
    pub is_automated: bool,
}

/// 简化版统一权限管理器
pub struct UnifiedPermissionManagerSimple {
    app_handle: AppHandle,
    last_check_time: Arc<RwLock<Option<Instant>>>,
    wizard_completed: Arc<RwLock<bool>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl UnifiedPermissionManagerSimple {
    /// 创建新的统一权限管理器
    pub fn new(app_handle: AppHandle) -> AppResult<Self> {
        Ok(Self {
            app_handle,
            last_check_time: Arc::new(RwLock::new(None)),
            wizard_completed: Arc::new(RwLock::new(false)),
            monitoring_active: Arc::new(RwLock::new(false)),
        })
    }

    /// 检查所有权限状态
    pub fn check_all_permissions(&self) -> AppResult<UnifiedPermissionReport> {
        println!("🔍 开始检查所有权限状态...");
        let start_time = Instant::now();

        let mut permissions = HashMap::new();
        let mut missing_critical = Vec::new();
        let mut missing_optional = Vec::new();

        // 检查麦克风权限
        let microphone_status = match PermissionManager::check_microphone_permission() {
            Ok(true) => UnifiedPermissionStatus::Granted,
            Ok(false) => UnifiedPermissionStatus::Denied,
            Err(_) => UnifiedPermissionStatus::NotDetermined,
        };
        permissions.insert(PermissionType::Microphone, microphone_status.clone());
        if microphone_status != UnifiedPermissionStatus::Granted {
            missing_critical.push(PermissionType::Microphone);
        }

        // 检查辅助功能权限
        let accessibility_status = match PermissionManager::check_accessibility_permission() {
            Ok(true) => UnifiedPermissionStatus::Granted,
            Ok(false) => UnifiedPermissionStatus::Denied,
            Err(_) => UnifiedPermissionStatus::NotDetermined,
        };
        permissions.insert(PermissionType::Accessibility, accessibility_status.clone());
        if accessibility_status != UnifiedPermissionStatus::Granted {
            missing_optional.push(PermissionType::Accessibility);
        }

        // 检查输入监控权限
        let input_monitoring_status = match PermissionManager::check_input_monitoring_permission() {
            Ok(true) => UnifiedPermissionStatus::Granted,
            Ok(false) => UnifiedPermissionStatus::Denied,
            Err(_) => UnifiedPermissionStatus::NotDetermined,
        };
        permissions.insert(
            PermissionType::InputMonitoring,
            input_monitoring_status.clone(),
        );
        if input_monitoring_status != UnifiedPermissionStatus::Granted {
            missing_critical.push(PermissionType::InputMonitoring);
        }

        // 更新最后检查时间
        {
            let mut last_check = self.last_check_time.write();
            *last_check = Some(Instant::now());
        }

        let all_critical_granted = missing_critical.is_empty();
        let check_duration = start_time.elapsed();

        println!("✅ 权限检查完成，耗时: {:?}", check_duration);
        println!("📊 权限状态: {} 个权限已检查", permissions.len());
        println!("🔴 缺失关键权限: {}", missing_critical.len());
        println!("🟡 缺失可选权限: {}", missing_optional.len());

        Ok(UnifiedPermissionReport {
            permissions,
            all_critical_granted,
            missing_critical,
            missing_optional,
            check_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            next_check_recommended: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 300, // 5分钟后建议重新检查
        })
    }

    /// 请求特定权限
    pub fn request_permission(&self, permission: PermissionType) -> AppResult<bool> {
        println!("📝 请求权限: {:?}", permission);

        // 检查当前状态
        let current_status = self.check_single_permission(&permission)?;

        if current_status == UnifiedPermissionStatus::Granted {
            println!("✅ 权限已授予: {:?}", permission);
            return Ok(true);
        }

        // 打开系统设置来请求权限
        let panel = match permission {
            PermissionType::Microphone => "microphone",
            PermissionType::Accessibility => "accessibility",
            PermissionType::InputMonitoring => "input_monitoring",
        };

        PermissionManager::open_system_preferences(panel)?;

        // 启动轮询检查权限状态变化
        self.start_permission_polling(permission)?;

        Ok(false) // 权限请求已发起，但尚未授予
    }

    /// 检查单个权限状态
    fn check_single_permission(
        &self,
        permission: &PermissionType,
    ) -> AppResult<UnifiedPermissionStatus> {
        match permission {
            PermissionType::Microphone => match PermissionManager::check_microphone_permission() {
                Ok(true) => Ok(UnifiedPermissionStatus::Granted),
                Ok(false) => Ok(UnifiedPermissionStatus::Denied),
                Err(_) => Ok(UnifiedPermissionStatus::NotDetermined),
            },
            PermissionType::Accessibility => {
                match PermissionManager::check_accessibility_permission() {
                    Ok(true) => Ok(UnifiedPermissionStatus::Granted),
                    Ok(false) => Ok(UnifiedPermissionStatus::Denied),
                    Err(_) => Ok(UnifiedPermissionStatus::NotDetermined),
                }
            }
            PermissionType::InputMonitoring => {
                match PermissionManager::check_input_monitoring_permission() {
                    Ok(true) => Ok(UnifiedPermissionStatus::Granted),
                    Ok(false) => Ok(UnifiedPermissionStatus::Denied),
                    Err(_) => Ok(UnifiedPermissionStatus::NotDetermined),
                }
            }
        }
    }

    /// 启动权限轮询检查
    fn start_permission_polling(&self, permission: PermissionType) -> AppResult<()> {
        let app_handle = self.app_handle.clone();
        let permission_clone = permission.clone();

        tokio::spawn(async move {
            let mut attempts = 0;
            const MAX_ATTEMPTS: u32 = 12; // 1分钟，每5秒检查一次

            while attempts < MAX_ATTEMPTS {
                tokio::time::sleep(Duration::from_secs(5)).await;
                attempts += 1;

                let status = match permission_clone {
                    PermissionType::Microphone => {
                        PermissionManager::check_microphone_permission().unwrap_or(false)
                    }
                    PermissionType::Accessibility => {
                        PermissionManager::check_accessibility_permission().unwrap_or(false)
                    }
                    PermissionType::InputMonitoring => {
                        PermissionManager::check_input_monitoring_permission().unwrap_or(false)
                    }
                };

                if status {
                    println!("🎉 权限已授予: {:?}", permission_clone);

                    // 发送事件到前端
                    let _ = app_handle.emit_all(
                        "permission_granted",
                        serde_json::json!({
                            "permission": permission_clone,
                            "status": "granted"
                        }),
                    );
                    break;
                }
            }

            if attempts >= MAX_ATTEMPTS {
                println!("⏰ 权限检查超时: {:?}", permission_clone);
                let _ = app_handle.emit_all(
                    "permission_timeout",
                    serde_json::json!({
                        "permission": permission_clone
                    }),
                );
            }
        });

        Ok(())
    }

    /// 获取权限引导信息
    pub fn get_permission_guidance(&self, permission: PermissionType) -> UnifiedGuidanceInfo {
        let current_status = self
            .check_single_permission(&permission)
            .unwrap_or(UnifiedPermissionStatus::NotDetermined);

        match permission {
            PermissionType::InputMonitoring => UnifiedGuidanceInfo {
                permission_type: permission,
                current_status,
                is_critical: true,
                title: "输入监控权限".to_string(),
                description: "此权限对于快捷键功能至关重要，没有此权限快捷键将无法工作。"
                    .to_string(),
                steps: vec![
                    GuidanceStep {
                        step_number: 1,
                        title: "打开系统偏好设置".to_string(),
                        description: "点击苹果菜单 > 系统偏好设置".to_string(),
                        action_type: "open_system_preferences".to_string(),
                        action_data: Some("security".to_string()),
                        is_automated: true,
                    },
                    GuidanceStep {
                        step_number: 2,
                        title: "进入安全性与隐私".to_string(),
                        description: "点击'安全性与隐私'图标".to_string(),
                        action_type: "navigate".to_string(),
                        action_data: None,
                        is_automated: false,
                    },
                    GuidanceStep {
                        step_number: 3,
                        title: "启用输入监控".to_string(),
                        description: "在左侧选择'输入监控'，然后勾选 Recording King".to_string(),
                        action_type: "enable_permission".to_string(),
                        action_data: Some("input_monitoring".to_string()),
                        is_automated: false,
                    },
                ],
                troubleshooting: vec![
                    "如果看不到 Recording King，请先启动应用".to_string(),
                    "如果勾选后仍无效，请重启应用".to_string(),
                    "某些情况下需要重启系统才能生效".to_string(),
                ],
            },
            PermissionType::Microphone => UnifiedGuidanceInfo {
                permission_type: permission,
                current_status,
                is_critical: true,
                title: "麦克风权限".to_string(),
                description: "此权限是录音功能的基础，没有此权限无法进行语音录制。".to_string(),
                steps: vec![
                    GuidanceStep {
                        step_number: 1,
                        title: "打开系统偏好设置".to_string(),
                        description: "点击苹果菜单 > 系统偏好设置".to_string(),
                        action_type: "open_system_preferences".to_string(),
                        action_data: Some("microphone".to_string()),
                        is_automated: true,
                    },
                    GuidanceStep {
                        step_number: 2,
                        title: "启用麦克风权限".to_string(),
                        description: "在麦克风权限列表中找到并勾选 Recording King".to_string(),
                        action_type: "enable_permission".to_string(),
                        action_data: Some("microphone".to_string()),
                        is_automated: false,
                    },
                ],
                troubleshooting: vec![
                    "确保麦克风设备已正确连接".to_string(),
                    "检查系统音量设置".to_string(),
                    "重启应用以使权限生效".to_string(),
                ],
            },
            PermissionType::Accessibility => UnifiedGuidanceInfo {
                permission_type: permission,
                current_status,
                is_critical: false,
                title: "辅助功能权限".to_string(),
                description: "此权限用于文本注入功能，可以自动将转录结果插入到其他应用中。"
                    .to_string(),
                steps: vec![
                    GuidanceStep {
                        step_number: 1,
                        title: "打开系统偏好设置".to_string(),
                        description: "点击苹果菜单 > 系统偏好设置".to_string(),
                        action_type: "open_system_preferences".to_string(),
                        action_data: Some("accessibility".to_string()),
                        is_automated: true,
                    },
                    GuidanceStep {
                        step_number: 2,
                        title: "启用辅助功能权限".to_string(),
                        description: "在辅助功能权限列表中找到并勾选 Recording King".to_string(),
                        action_type: "enable_permission".to_string(),
                        action_data: Some("accessibility".to_string()),
                        is_automated: false,
                    },
                ],
                troubleshooting: vec![
                    "没有此权限时可以手动复制粘贴转录结果".to_string(),
                    "某些应用可能需要额外的权限配置".to_string(),
                ],
            },
        }
    }

    /// 检查权限向导是否已完成
    pub fn is_wizard_completed(&self) -> bool {
        *self.wizard_completed.read()
    }

    /// 标记权限向导为已完成
    pub fn mark_wizard_completed(&self) {
        let mut completed = self.wizard_completed.write();
        *completed = true;
        println!("✅ 权限向导已标记为完成");
    }

    /// 开始权限状态监控
    pub fn start_monitoring(&self) -> AppResult<()> {
        {
            let mut monitoring = self.monitoring_active.write();
            if *monitoring {
                return Ok(());
            }
            *monitoring = true;
        }

        let app_handle = self.app_handle.clone();
        let monitoring_active = self.monitoring_active.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // 每30秒检查一次

            while *monitoring_active.read() {
                interval.tick().await;

                let _ = app_handle.emit_all(
                    "permission_status_update",
                    serde_json::json!({
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    }),
                );
            }
        });

        println!("🔄 权限状态监控已启动");
        Ok(())
    }

    /// 停止权限状态监控
    pub fn stop_monitoring(&self) {
        let mut monitoring = self.monitoring_active.write();
        *monitoring = false;
        println!("⏹️ 权限状态监控已停止");
    }

    /// 重置权限状态（用于测试和故障排除）
    pub fn reset_permission_state(&self) {
        // 重置向导状态
        *self.wizard_completed.write() = false;

        // 停止监控
        self.stop_monitoring();

        println!("🔄 权限状态已重置");
    }
}
