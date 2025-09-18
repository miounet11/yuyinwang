// Story 1.4: Transcription Mode Management with Auto-Switching

use crate::errors::AppResult;
use crate::network::{NetworkMonitor, NetworkStatus};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranscriptionMode {
    Local,  // 仅使用本地模型
    Cloud,  // 仅使用云端API
    Auto,   // 自动选择（基于网络状态和性能）
    Hybrid, // 混合模式（同时使用，选择最优结果）
}

impl Default for TranscriptionMode {
    fn default() -> Self {
        TranscriptionMode::Auto
    }
}

#[derive(Debug, Clone)]
pub struct ModeChangeEvent {
    pub from_mode: TranscriptionMode,
    pub to_mode: TranscriptionMode,
    pub reason: String,
    pub automatic: bool,
}

#[derive(Debug, Clone)]
pub struct ModeManagerConfig {
    pub auto_switch_enabled: bool,
    pub cloud_api_timeout_ms: u64,
    pub local_model_priority: bool,
    pub network_quality_threshold: f64, // 0.0-1.0，低于此值切换到本地
    pub switch_debounce_ms: u64,        // 防抖时间，避免频繁切换
}

impl Default for ModeManagerConfig {
    fn default() -> Self {
        Self {
            auto_switch_enabled: true,
            cloud_api_timeout_ms: 10000,    // 10秒
            local_model_priority: false,    // 优先云端（准确性更高）
            network_quality_threshold: 0.6, // 60%以下网络质量切换本地
            switch_debounce_ms: 5000,       // 5秒防抖
        }
    }
}

pub struct TranscriptionModeManager {
    current_mode: Arc<Mutex<TranscriptionMode>>,
    user_preferred_mode: Arc<Mutex<TranscriptionMode>>,
    active_mode: Arc<Mutex<TranscriptionMode>>, // 当前实际使用的模式
    config: Arc<Mutex<ModeManagerConfig>>,
    network_monitor: Arc<NetworkMonitor>,
    mode_change_sender: broadcast::Sender<ModeChangeEvent>,
    last_switch_time: Arc<Mutex<std::time::Instant>>,
}

impl TranscriptionModeManager {
    pub fn new(network_monitor: Arc<NetworkMonitor>) -> Self {
        let (mode_change_sender, _) = broadcast::channel(16);

        Self {
            current_mode: Arc::new(Mutex::new(TranscriptionMode::Auto)),
            user_preferred_mode: Arc::new(Mutex::new(TranscriptionMode::Auto)),
            active_mode: Arc::new(Mutex::new(TranscriptionMode::Local)), // 默认本地保险
            config: Arc::new(Mutex::new(ModeManagerConfig::default())),
            network_monitor,
            mode_change_sender,
            last_switch_time: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    /// 设置用户首选模式
    pub async fn set_user_mode(&self, mode: TranscriptionMode) -> AppResult<()> {
        println!("🎯 用户设置转录模式: {:?}", mode);

        let previous_mode = *self.current_mode.lock();
        *self.user_preferred_mode.lock() = mode.clone();
        *self.current_mode.lock() = mode.clone();

        // 根据新模式更新活动模式
        let active_mode = self.determine_active_mode().await;
        self.set_active_mode(active_mode, format!("用户手动设置模式为 {:?}", mode), false)
            .await?;

        Ok(())
    }

    /// 获取当前模式
    pub fn get_current_mode(&self) -> TranscriptionMode {
        *self.current_mode.lock()
    }

    /// 获取当前活动模式（实际使用的模式）
    pub fn get_active_mode(&self) -> TranscriptionMode {
        *self.active_mode.lock()
    }

    /// 获取用户首选模式
    pub fn get_user_preferred_mode(&self) -> TranscriptionMode {
        *self.user_preferred_mode.lock()
    }

    /// 更新配置
    pub fn update_config(&self, config: ModeManagerConfig) {
        *self.config.lock() = config;
        println!("🔧 模式管理器配置已更新");
    }

    /// 获取当前配置
    pub fn get_config(&self) -> ModeManagerConfig {
        self.config.lock().clone()
    }

    /// 开始自动模式管理
    pub async fn start_auto_management(&self) -> AppResult<()> {
        let config = self.config.lock().clone();

        if !config.auto_switch_enabled {
            println!("🤖 自动模式切换已禁用");
            return Ok(());
        }

        println!("🤖 开始自动转录模式管理");

        // 订阅网络状态变化
        let mut network_status_rx = self.network_monitor.subscribe_status_changes();
        let mode_manager = Arc::new(self.clone());

        tokio::spawn(async move {
            while let Ok(network_status) = network_status_rx.recv().await {
                if let Err(e) = mode_manager
                    .handle_network_status_change(network_status)
                    .await
                {
                    eprintln!("❌ 处理网络状态变化失败: {}", e);
                }
            }
        });

        // 定期评估和调整模式
        let mode_manager_clone = Arc::new(self.clone());
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                if let Err(e) = mode_manager_clone.periodic_mode_evaluation().await {
                    eprintln!("❌ 定期模式评估失败: {}", e);
                }
            }
        });

        Ok(())
    }

    /// 处理网络状态变化
    async fn handle_network_status_change(&self, network_status: NetworkStatus) -> AppResult<()> {
        let current_mode = *self.current_mode.lock();

        // 只有在Auto模式下才进行自动切换
        if current_mode != TranscriptionMode::Auto {
            return Ok(());
        }

        let config = self.config.lock().clone();

        // 防抖检查
        {
            let last_switch = *self.last_switch_time.lock();
            let debounce_duration = std::time::Duration::from_millis(config.switch_debounce_ms);

            if last_switch.elapsed() < debounce_duration {
                println!("🕐 模式切换防抖中，跳过此次切换");
                return Ok(());
            }
        }

        let suggested_mode = match network_status {
            NetworkStatus::Online => {
                if config.local_model_priority {
                    TranscriptionMode::Local
                } else {
                    TranscriptionMode::Cloud
                }
            }
            NetworkStatus::Limited => {
                // 网络质量差，根据阈值决定
                let quality = self.network_monitor.get_connection_quality_score();
                if quality < config.network_quality_threshold {
                    TranscriptionMode::Local
                } else {
                    TranscriptionMode::Cloud
                }
            }
            NetworkStatus::Offline => TranscriptionMode::Local,
            NetworkStatus::Unknown => TranscriptionMode::Local, // 保险起见使用本地
        };

        let current_active = *self.active_mode.lock();

        if current_active != suggested_mode {
            let reason = format!(
                "网络状态变化: {:?}, 质量: {:.2}",
                network_status,
                self.network_monitor.get_connection_quality_score()
            );

            self.set_active_mode(suggested_mode, reason, true).await?;
        }

        Ok(())
    }

    /// 定期模式评估
    async fn periodic_mode_evaluation(&self) -> AppResult<()> {
        let current_mode = *self.current_mode.lock();

        // 只在Auto模式下进行评估
        if current_mode != TranscriptionMode::Auto {
            return Ok(());
        }

        let optimal_mode = self.determine_active_mode().await;
        let current_active = *self.active_mode.lock();

        if optimal_mode != current_active {
            let reason = "定期性能评估建议切换模式".to_string();
            self.set_active_mode(optimal_mode, reason, true).await?;
        }

        Ok(())
    }

    /// 确定最优的活动模式
    async fn determine_active_mode(&self) -> TranscriptionMode {
        let user_mode = *self.user_preferred_mode.lock();
        let config = self.config.lock().clone();

        match user_mode {
            TranscriptionMode::Local => TranscriptionMode::Local,
            TranscriptionMode::Cloud => TranscriptionMode::Cloud,
            TranscriptionMode::Hybrid => TranscriptionMode::Hybrid,
            TranscriptionMode::Auto => {
                // 自动决策逻辑
                let network_status = self.network_monitor.get_current_status();
                let network_quality = self.network_monitor.get_connection_quality_score();

                match network_status {
                    NetworkStatus::Online => {
                        if network_quality >= config.network_quality_threshold {
                            if config.local_model_priority {
                                TranscriptionMode::Local
                            } else {
                                TranscriptionMode::Cloud
                            }
                        } else {
                            TranscriptionMode::Local
                        }
                    }
                    NetworkStatus::Limited => {
                        if network_quality >= config.network_quality_threshold {
                            TranscriptionMode::Cloud
                        } else {
                            TranscriptionMode::Local
                        }
                    }
                    NetworkStatus::Offline | NetworkStatus::Unknown => TranscriptionMode::Local,
                }
            }
        }
    }

    /// 设置活动模式并发送变化事件
    async fn set_active_mode(
        &self,
        mode: TranscriptionMode,
        reason: String,
        automatic: bool,
    ) -> AppResult<()> {
        let previous_mode = *self.active_mode.lock();

        if previous_mode == mode {
            return Ok(());
        }

        *self.active_mode.lock() = mode.clone();
        *self.last_switch_time.lock() = std::time::Instant::now();

        let event = ModeChangeEvent {
            from_mode: previous_mode,
            to_mode: mode.clone(),
            reason: reason.clone(),
            automatic,
        };

        println!(
            "🔄 转录模式切换: {:?} -> {:?} ({})",
            previous_mode,
            mode,
            if automatic { "自动" } else { "手动" }
        );
        println!("   原因: {}", reason);

        // 发送模式变化事件
        let _ = self.mode_change_sender.send(event);

        Ok(())
    }

    /// 订阅模式变化事件
    pub fn subscribe_mode_changes(&self) -> broadcast::Receiver<ModeChangeEvent> {
        self.mode_change_sender.subscribe()
    }

    /// 强制重新评估模式
    pub async fn force_reevaluate(&self) -> AppResult<TranscriptionMode> {
        println!("🔄 强制重新评估转录模式");

        let optimal_mode = self.determine_active_mode().await;
        self.set_active_mode(
            optimal_mode.clone(),
            "用户请求强制重新评估".to_string(),
            false,
        )
        .await?;

        Ok(optimal_mode)
    }

    /// 获取模式切换建议
    pub async fn get_mode_recommendation(&self) -> AppResult<(TranscriptionMode, String)> {
        let current_mode = *self.active_mode.lock();
        let optimal_mode = self.determine_active_mode().await;
        let network_status = self.network_monitor.get_current_status();
        let network_quality = self.network_monitor.get_connection_quality_score();

        let recommendation = if current_mode == optimal_mode {
            format!("当前模式 {:?} 已是最优选择", current_mode)
        } else {
            format!(
                "建议切换到 {:?} 模式 (网络: {:?}, 质量: {:.0}%)",
                optimal_mode,
                network_status,
                network_quality * 100.0
            )
        };

        Ok((optimal_mode, recommendation))
    }
}

// 为了支持克隆，需要实现Clone trait（简化版本）
impl Clone for TranscriptionModeManager {
    fn clone(&self) -> Self {
        Self {
            current_mode: self.current_mode.clone(),
            user_preferred_mode: self.user_preferred_mode.clone(),
            active_mode: self.active_mode.clone(),
            config: self.config.clone(),
            network_monitor: self.network_monitor.clone(),
            mode_change_sender: self.mode_change_sender.clone(),
            last_switch_time: self.last_switch_time.clone(),
        }
    }
}
