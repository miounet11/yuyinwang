// Story 1.4: Network Monitoring Service for Auto Mode Switching

use crate::errors::{AppError, AppResult};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    Online,
    Offline,
    Limited, // 网络连接存在但质量差
    Unknown,
}

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub is_connected: bool,
    pub latency_ms: Option<u64>,
    pub connection_quality: f64, // 0.0-1.0
    pub last_checked: Instant,
    pub consecutive_failures: u32,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            is_connected: false,
            latency_ms: None,
            connection_quality: 0.0,
            last_checked: Instant::now(),
            consecutive_failures: 0,
        }
    }
}

pub struct NetworkMonitor {
    status: Arc<Mutex<NetworkStatus>>,
    metrics: Arc<Mutex<NetworkMetrics>>,
    status_sender: broadcast::Sender<NetworkStatus>,
    monitoring: Arc<Mutex<bool>>,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        let (status_sender, _) = broadcast::channel(16);

        Self {
            status: Arc::new(Mutex::new(NetworkStatus::Unknown)),
            metrics: Arc::new(Mutex::new(NetworkMetrics::default())),
            status_sender,
            monitoring: Arc::new(Mutex::new(false)),
        }
    }

    /// 开始网络监控
    pub async fn start_monitoring(&self, check_interval: Duration) -> AppResult<()> {
        {
            let mut monitoring = self.monitoring.lock();
            if *monitoring {
                return Err(AppError::InvalidOperation(
                    "Network monitoring already started".to_string(),
                ));
            }
            *monitoring = true;
        }

        let status_arc = self.status.clone();
        let metrics_arc = self.metrics.clone();
        let sender = self.status_sender.clone();
        let monitoring_arc = self.monitoring.clone();

        tokio::spawn(async move {
            println!("🌐 开始网络监控，检查间隔: {:?}", check_interval);

            loop {
                // 检查是否继续监控
                let should_continue = *monitoring_arc.lock();
                if !should_continue {
                    break;
                }

                let new_status = Self::check_network_status().await;

                // 在分离的作用域中处理状态更新，确保在await之前释放所有锁
                let previous_status = {
                    let status_guard = status_arc.lock();
                    *status_guard
                };

                // 更新状态
                {
                    let mut status_guard = status_arc.lock();
                    *status_guard = new_status;
                }

                // 更新指标
                {
                    let mut metrics = metrics_arc.lock();
                    metrics.last_checked = Instant::now();

                    match new_status {
                        NetworkStatus::Online => {
                            metrics.is_connected = true;
                            metrics.consecutive_failures = 0;
                            metrics.connection_quality = 1.0;
                        }
                        NetworkStatus::Limited => {
                            metrics.is_connected = true;
                            metrics.consecutive_failures += 1;
                            metrics.connection_quality = 0.5;
                        }
                        NetworkStatus::Offline => {
                            metrics.is_connected = false;
                            metrics.consecutive_failures += 1;
                            metrics.connection_quality = 0.0;
                        }
                        NetworkStatus::Unknown => {
                            metrics.consecutive_failures += 1;
                        }
                    }
                }

                // 只有状态变化时才发送通知
                if previous_status != new_status {
                    println!("🌐 网络状态变化: {:?} -> {:?}", previous_status, new_status);
                    let _ = sender.send(new_status);
                }

                tokio::time::sleep(check_interval).await;
            }

            println!("🌐 网络监控已停止");
        });

        Ok(())
    }

    /// 停止网络监控
    pub fn stop_monitoring(&self) {
        *self.monitoring.lock() = false;
        println!("🌐 停止网络监控");
    }

    /// 获取当前网络状态
    pub fn get_current_status(&self) -> NetworkStatus {
        *self.status.lock()
    }

    /// 获取网络指标
    pub fn get_metrics(&self) -> NetworkMetrics {
        self.metrics.lock().clone()
    }

    /// 订阅网络状态变化
    pub fn subscribe_status_changes(&self) -> broadcast::Receiver<NetworkStatus> {
        self.status_sender.subscribe()
    }

    /// 立即检查网络状态
    pub async fn check_now(&self) -> NetworkStatus {
        let status = Self::check_network_status().await;
        *self.status.lock() = status.clone();

        let mut metrics = self.metrics.lock();
        metrics.last_checked = Instant::now();
        metrics.is_connected = matches!(status, NetworkStatus::Online | NetworkStatus::Limited);

        status
    }

    /// 检查网络连接状态（核心逻辑）
    async fn check_network_status() -> NetworkStatus {
        // 方法1: 尝试连接多个可靠的DNS服务器
        let dns_servers = vec![
            "8.8.8.8:53",         // Google DNS
            "1.1.1.1:53",         // Cloudflare DNS
            "114.114.114.114:53", // 114 DNS (中国)
        ];

        let mut successful_connections = 0;
        let start_time = Instant::now();

        for dns_server in &dns_servers {
            match tokio::time::timeout(Duration::from_secs(3), Self::test_connection(dns_server))
                .await
            {
                Ok(Ok(_)) => {
                    successful_connections += 1;
                }
                Ok(Err(_)) | Err(_) => {
                    // 连接失败或超时
                }
            }
        }

        let elapsed = start_time.elapsed();

        // 根据成功连接数判断状态
        match successful_connections {
            3 => NetworkStatus::Online,
            1..=2 => {
                if elapsed > Duration::from_secs(2) {
                    NetworkStatus::Limited // 连接慢
                } else {
                    NetworkStatus::Online // 部分连接但速度正常
                }
            }
            0 => {
                // 尝试方法2: 检查系统网络接口
                if Self::check_system_network_interfaces().await {
                    NetworkStatus::Limited // 有网络接口但无法连接外部
                } else {
                    NetworkStatus::Offline // 完全离线
                }
            }
            _ => NetworkStatus::Unknown,
        }
    }

    /// 测试到特定地址的连接
    async fn test_connection(address: &str) -> AppResult<()> {
        use tokio::net::TcpStream;

        TcpStream::connect(address).await.map_err(|e| {
            AppError::NetworkError(format!("Connection failed to {}: {}", address, e))
        })?;

        Ok(())
    }

    /// 检查系统网络接口状态
    async fn check_system_network_interfaces() -> bool {
        #[cfg(target_os = "macos")]
        {
            // 在 macOS 上检查网络接口
            match tokio::process::Command::new("ifconfig").output().await {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // 查找活动的网络接口（有IP地址的）
                    output_str.contains("inet ")
                        && (output_str.contains("en0")
                            || output_str.contains("en1")
                            || output_str.contains("wlan"))
                }
                Err(_) => false,
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // 其他平台的简单检查
            true // 假设有网络接口
        }
    }

    /// 测试到特定API端点的连接质量
    pub async fn test_api_endpoint(&self, url: &str) -> AppResult<Duration> {
        let start = Instant::now();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| AppError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        let response =
            client.get(url).send().await.map_err(|e| {
                AppError::NetworkError(format!("Failed to connect to {}: {}", url, e))
            })?;

        if response.status().is_success() {
            Ok(start.elapsed())
        } else {
            Err(AppError::NetworkError(format!(
                "API endpoint returned status: {}",
                response.status()
            )))
        }
    }

    /// 获取连接质量评分 (0.0-1.0)
    pub fn get_connection_quality_score(&self) -> f64 {
        let metrics = self.metrics.lock();

        // 基于延迟、失败次数等计算质量评分
        let base_score = match metrics.is_connected {
            true => 1.0,
            false => 0.0,
        };

        // 根据连续失败次数降低评分
        let failure_penalty = (metrics.consecutive_failures as f64 * 0.1).min(0.5);

        (base_score - failure_penalty).max(0.0)
    }
}
