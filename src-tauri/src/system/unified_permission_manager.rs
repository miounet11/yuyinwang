// UnifiedPermissionManager - 统一权限管理系统
// 整合现有三套快捷键管理器的权限检查，提供统一的权限管理接口

use crate::errors::{AppError, AppResult};
use crate::system::permission_manager::{PermissionGuide, PermissionManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tokio::sync::mpsc;

/// 权限类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionType {
    Microphone,
    Accessibility,
    InputMonitoring,
    ScreenRecording,
    FullDiskAccess,
}

/// 权限状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
}

/// 权限检查报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionReport {
    pub permissions: HashMap<PermissionType, PermissionStatus>,
    pub all_critical_granted: bool,
    pub missing_critical: Vec<PermissionType>,
    pub missing_optional: Vec<PermissionType>,
    pub check_timestamp: u64,
    pub next_check_recommended: u64,
}

/// 权限引导信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidanceInfo {
    pub permission_type: PermissionType,
    pub current_status: PermissionStatus,
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
    pub image_url: Option<String>,
    pub is_automated: bool,
}

/// 权限状态监听器trait
pub trait PermissionStateListener: Send + Sync {
    fn on_permission_changed(&self, permission: PermissionType, new_status: PermissionStatus);
    fn on_critical_permission_lost(&self, permission: PermissionType);
    fn on_all_permissions_granted(&self);
}

/// 权限检查器trait
pub trait PermissionChecker: Send + Sync {
    fn check_permission(&self) -> AppResult<PermissionStatus>;
    fn get_permission_type(&self) -> PermissionType;
    fn is_critical(&self) -> bool;
}

/// 统一权限管理器
pub struct UnifiedPermissionManager {
    app_handle: AppHandle,
    permission_state: Arc<RwLock<HashMap<PermissionType, PermissionStatus>>>,
    permission_checkers: HashMap<PermissionType, Box<dyn PermissionChecker>>,
    state_listeners: Arc<RwLock<Vec<Box<dyn PermissionStateListener>>>>,
    last_check_time: Arc<RwLock<Instant>>,
    wizard_completed: Arc<RwLock<bool>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl UnifiedPermissionManager {
    /// 创建新的统一权限管理器
    pub fn new(app_handle: AppHandle) -> AppResult<Self> {
        let mut manager = Self {
            app_handle,
            permission_state: Arc::new(RwLock::new(HashMap::new())),
            permission_checkers: HashMap::new(),
            state_listeners: Arc::new(RwLock::new(Vec::new())),
            last_check_time: Arc::new(RwLock::new(Instant::now())),
            wizard_completed: Arc::new(RwLock::new(false)),
            monitoring_active: Arc::new(RwLock::new(false)),
        };

        // 注册默认的权限检查器
        manager.register_default_checkers()?;

        Ok(manager)
    }

    /// 注册默认的权限检查器
    fn register_default_checkers(&mut self) -> AppResult<()> {
        // 注册麦克风权限检查器
        self.permission_checkers.insert(
            PermissionType::Microphone,
            Box::new(MicrophonePermissionChecker::new()),
        );

        // 注册辅助功能权限检查器
        self.permission_checkers.insert(
            PermissionType::Accessibility,
            Box::new(AccessibilityPermissionChecker::new()),
        );

        // 注册输入监控权限检查器
        self.permission_checkers.insert(
            PermissionType::InputMonitoring,
            Box::new(InputMonitoringPermissionChecker::new()),
        );

        println!("✅ 已注册 {} 个权限检查器", self.permission_checkers.len());
        Ok(())
    }

    /// 检查所有权限状态
    pub async fn check_all_permissions(&self) -> AppResult<PermissionReport> {
        println!("🔍 开始检查所有权限状态...");
        let start_time = Instant::now();

        let mut permissions = HashMap::new();
        let mut missing_critical = Vec::new();
        let mut missing_optional = Vec::new();

        // 并行检查所有权限
        let mut check_tasks = Vec::new();

        for (permission_type, checker) in &self.permission_checkers {
            let permission_type = permission_type.clone();
            let checker_clone = checker.as_ref() as *const dyn PermissionChecker;

            // 安全地克隆检查器指针（这是一个简化实现，实际项目中应该使用Arc）
            let task = tokio::spawn(async move {
                unsafe {
                    let checker = &*checker_clone;
                    let status = checker
                        .check_permission()
                        .await
                        .unwrap_or(PermissionStatus::NotDetermined);
                    (permission_type, status, checker.is_critical())
                }
            });
            check_tasks.push(task);
        }

        // 等待所有检查完成
        for task in check_tasks {
            match task.await {
                Ok((permission_type, status, is_critical)) => {
                    permissions.insert(permission_type.clone(), status.clone());

                    if status != PermissionStatus::Granted {
                        if is_critical {
                            missing_critical.push(permission_type);
                        } else {
                            missing_optional.push(permission_type);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ 权限检查任务失败: {}", e);
                }
            }
        }

        // 更新内部状态
        {
            let mut state = self.permission_state.write().unwrap();
            *state = permissions.clone();
        }

        // 更新最后检查时间
        {
            let mut last_check = self.last_check_time.write().unwrap();
            *last_check = Instant::now();
        }

        let all_critical_granted = missing_critical.is_empty();
        let check_duration = start_time.elapsed();

        println!("✅ 权限检查完成，耗时: {:?}", check_duration);
        println!("📊 权限状态: {} 个权限已检查", permissions.len());
        println!("🔴 缺失关键权限: {}", missing_critical.len());
        println!("🟡 缺失可选权限: {}", missing_optional.len());

        // 通知监听器
        self.notify_listeners_about_changes(&permissions).await;

        Ok(PermissionReport {
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
    pub async fn request_permission(&self, permission: PermissionType) -> AppResult<bool> {
        println!("📝 请求权限: {:?}", permission);

        // 检查当前状态
        if let Some(checker) = self.permission_checkers.get(&permission) {
            let current_status = checker.check_permission().await?;

            if current_status == PermissionStatus::Granted {
                println!("✅ 权限已授予: {:?}", permission);
                return Ok(true);
            }

            // 打开系统设置来请求权限
            let panel = match permission {
                PermissionType::Microphone => "microphone",
                PermissionType::Accessibility => "accessibility",
                PermissionType::InputMonitoring => "input_monitoring",
                PermissionType::ScreenRecording => "screen_recording",
                PermissionType::FullDiskAccess => "full_disk_access",
            };

            PermissionManager::open_system_preferences(panel)?;

            // 启动轮询检查权限状态变化
            self.start_permission_polling(permission.clone()).await?;

            Ok(false) // 权限请求已发起，但尚未授予
        } else {
            Err(AppError::PermissionError(format!(
                "未找到权限检查器: {:?}",
                permission
            )))
        }
    }

    /// 启动权限轮询检查
    async fn start_permission_polling(&self, permission: PermissionType) -> AppResult<()> {
        let checkers = &self.permission_checkers;
        if let Some(checker) = checkers.get(&permission) {
            let checker_ptr = checker.as_ref() as *const dyn PermissionChecker;
            let app_handle = self.app_handle.clone();
            let permission_clone = permission.clone();

            tokio::spawn(async move {
                let mut attempts = 0;
                const MAX_ATTEMPTS: u32 = 12; // 1分钟，每5秒检查一次

                while attempts < MAX_ATTEMPTS {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    attempts += 1;

                    unsafe {
                        let checker = &*checker_ptr;
                        if let Ok(status) = checker.check_permission().await {
                            if status == PermissionStatus::Granted {
                                println!("🎉 权限已授予: {:?}", permission_clone);

                                // 发送事件到前端
                                let _ = app_handle.emit_all(
                                    "permission_granted",
                                    serde_json::json!({
                                        "permission": permission_clone,
                                        "status": status
                                    }),
                                );
                                break;
                            }
                        }
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
        }

        Ok(())
    }

    /// 注册权限状态监听器
    pub fn register_state_listener(&self, listener: Box<dyn PermissionStateListener>) {
        let mut listeners = self.state_listeners.write().unwrap();
        listeners.push(listener);
        println!(
            "📡 已注册权限状态监听器，当前监听器数量: {}",
            listeners.len()
        );
    }

    /// 获取权限引导信息
    pub fn get_permission_guidance(&self, permission: PermissionType) -> GuidanceInfo {
        let current_status = {
            let state = self.permission_state.read().unwrap();
            state
                .get(&permission)
                .cloned()
                .unwrap_or(PermissionStatus::NotDetermined)
        };

        match permission {
            PermissionType::InputMonitoring => GuidanceInfo {
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
                        image_url: None,
                        is_automated: true,
                    },
                    GuidanceStep {
                        step_number: 2,
                        title: "进入安全性与隐私".to_string(),
                        description: "点击'安全性与隐私'图标".to_string(),
                        action_type: "navigate".to_string(),
                        action_data: None,
                        image_url: None,
                        is_automated: false,
                    },
                    GuidanceStep {
                        step_number: 3,
                        title: "选择隐私标签".to_string(),
                        description: "点击窗口顶部的'隐私'标签".to_string(),
                        action_type: "navigate".to_string(),
                        action_data: None,
                        image_url: None,
                        is_automated: false,
                    },
                    GuidanceStep {
                        step_number: 4,
                        title: "启用输入监控".to_string(),
                        description: "在左侧列表中选择'输入监控'，然后勾选 Recording King"
                            .to_string(),
                        action_type: "enable_permission".to_string(),
                        action_data: Some("input_monitoring".to_string()),
                        image_url: None,
                        is_automated: false,
                    },
                ],
                troubleshooting: vec![
                    "如果看不到 Recording King，请先启动应用".to_string(),
                    "如果勾选后仍无效，请重启应用".to_string(),
                    "某些情况下需要重启系统才能生效".to_string(),
                ],
            },
            PermissionType::Microphone => GuidanceInfo {
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
                        image_url: None,
                        is_automated: true,
                    },
                    GuidanceStep {
                        step_number: 2,
                        title: "启用麦克风权限".to_string(),
                        description: "在麦克风权限列表中找到并勾选 Recording King".to_string(),
                        action_type: "enable_permission".to_string(),
                        action_data: Some("microphone".to_string()),
                        image_url: None,
                        is_automated: false,
                    },
                ],
                troubleshooting: vec![
                    "确保麦克风设备已正确连接".to_string(),
                    "检查系统音量设置".to_string(),
                    "重启应用以使权限生效".to_string(),
                ],
            },
            PermissionType::Accessibility => GuidanceInfo {
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
                        image_url: None,
                        is_automated: true,
                    },
                    GuidanceStep {
                        step_number: 2,
                        title: "启用辅助功能权限".to_string(),
                        description: "在辅助功能权限列表中找到并勾选 Recording King".to_string(),
                        action_type: "enable_permission".to_string(),
                        action_data: Some("accessibility".to_string()),
                        image_url: None,
                        is_automated: false,
                    },
                ],
                troubleshooting: vec![
                    "没有此权限时可以手动复制粘贴转录结果".to_string(),
                    "某些应用可能需要额外的权限配置".to_string(),
                ],
            },
            _ => GuidanceInfo {
                permission_type: permission,
                current_status,
                is_critical: false,
                title: "权限配置".to_string(),
                description: "请根据系统提示配置此权限。".to_string(),
                steps: vec![],
                troubleshooting: vec![],
            },
        }
    }

    /// 检查权限向导是否已完成
    pub fn is_wizard_completed(&self) -> bool {
        *self.wizard_completed.read().unwrap()
    }

    /// 标记权限向导为已完成
    pub fn mark_wizard_completed(&self) {
        let mut completed = self.wizard_completed.write().unwrap();
        *completed = true;
        println!("✅ 权限向导已标记为完成");
    }

    /// 开始权限状态监控
    pub async fn start_monitoring(&self) -> AppResult<()> {
        {
            let mut monitoring = self.monitoring_active.write().unwrap();
            if *monitoring {
                return Ok(());
            }
            *monitoring = true;
        }

        let permission_state = self.permission_state.clone();
        let app_handle = self.app_handle.clone();
        let monitoring_active = self.monitoring_active.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // 每30秒检查一次

            while *monitoring_active.read().unwrap() {
                interval.tick().await;

                // 这里会调用一个简化的权限检查
                // 在实际实现中，你需要重新检查权限状态

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
        let mut monitoring = self.monitoring_active.write().unwrap();
        *monitoring = false;
        println!("⏹️ 权限状态监控已停止");
    }

    /// 通知监听器权限状态变化
    async fn notify_listeners_about_changes(
        &self,
        new_permissions: &HashMap<PermissionType, PermissionStatus>,
    ) {
        let listeners = self.state_listeners.read().unwrap();

        for (permission_type, status) in new_permissions {
            for listener in listeners.iter() {
                listener.on_permission_changed(permission_type.clone(), status.clone());
            }
        }

        // 检查是否所有关键权限都已授予
        let all_critical_granted = new_permissions
            .iter()
            .filter(|(ptype, _)| {
                matches!(
                    ptype,
                    PermissionType::Microphone | PermissionType::InputMonitoring
                )
            })
            .all(|(_, status)| *status == PermissionStatus::Granted);

        if all_critical_granted {
            for listener in listeners.iter() {
                listener.on_all_permissions_granted();
            }
        }
    }
}

// 具体的权限检查器实现

/// 麦克风权限检查器
struct MicrophonePermissionChecker;

impl MicrophonePermissionChecker {
    fn new() -> Self {
        Self
    }
}

impl PermissionChecker for MicrophonePermissionChecker {
    async fn check_permission(&self) -> AppResult<PermissionStatus> {
        match PermissionManager::check_microphone_permission() {
            Ok(true) => Ok(PermissionStatus::Granted),
            Ok(false) => Ok(PermissionStatus::Denied),
            Err(_) => Ok(PermissionStatus::NotDetermined),
        }
    }

    fn get_permission_type(&self) -> PermissionType {
        PermissionType::Microphone
    }

    fn is_critical(&self) -> bool {
        true
    }
}

/// 辅助功能权限检查器
struct AccessibilityPermissionChecker;

impl AccessibilityPermissionChecker {
    fn new() -> Self {
        Self
    }
}

impl PermissionChecker for AccessibilityPermissionChecker {
    async fn check_permission(&self) -> AppResult<PermissionStatus> {
        match PermissionManager::check_accessibility_permission() {
            Ok(true) => Ok(PermissionStatus::Granted),
            Ok(false) => Ok(PermissionStatus::Denied),
            Err(_) => Ok(PermissionStatus::NotDetermined),
        }
    }

    fn get_permission_type(&self) -> PermissionType {
        PermissionType::Accessibility
    }

    fn is_critical(&self) -> bool {
        false // 辅助功能权限是可选的
    }
}

/// 输入监控权限检查器
struct InputMonitoringPermissionChecker;

impl InputMonitoringPermissionChecker {
    fn new() -> Self {
        Self
    }
}

impl PermissionChecker for InputMonitoringPermissionChecker {
    async fn check_permission(&self) -> AppResult<PermissionStatus> {
        match PermissionManager::check_input_monitoring_permission() {
            Ok(true) => Ok(PermissionStatus::Granted),
            Ok(false) => Ok(PermissionStatus::Denied),
            Err(_) => Ok(PermissionStatus::NotDetermined),
        }
    }

    fn get_permission_type(&self) -> PermissionType {
        PermissionType::InputMonitoring
    }

    fn is_critical(&self) -> bool {
        true // 输入监控是关键权限
    }
}
