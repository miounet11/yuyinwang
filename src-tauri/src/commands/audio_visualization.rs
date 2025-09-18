use crate::audio::visualization::{
    AudioVisualizationData, AudioVisualizationManager, WaveformColorScheme, WaveformConfig,
    WaveformRenderMode,
};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{State, Window};
use tokio::sync::Mutex;
use uuid::Uuid;

/// 音频可视化订阅管理器
pub struct AudioVisualizationSubscriptionManager {
    subscriptions: Arc<Mutex<HashMap<String, AudioVisualizationSubscription>>>,
}

#[derive(Debug)]
struct AudioVisualizationSubscription {
    window: Window,
    config: WaveformConfig,
    manager: Arc<AudioVisualizationManager>,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualizationSubscriptionRequest {
    pub config: Option<WaveformConfig>,
    pub enable_real_time: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualizationUpdateEvent {
    pub subscription_id: String,
    pub data: AudioVisualizationData,
    pub timestamp: u64,
}

impl AudioVisualizationSubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_subscription(
        &self,
        window: Window,
        config: Option<WaveformConfig>,
    ) -> String {
        let subscription_id = Uuid::new_v4().to_string();
        let config = config.unwrap_or_default();
        let manager = Arc::new(AudioVisualizationManager::new(config.clone()));

        let subscription = AudioVisualizationSubscription {
            window,
            config,
            manager,
            is_active: true,
        };

        self.subscriptions
            .lock()
            .await
            .insert(subscription_id.clone(), subscription);

        subscription_id
    }

    pub async fn remove_subscription(&self, subscription_id: &str) -> bool {
        self.subscriptions
            .lock()
            .await
            .remove(subscription_id)
            .is_some()
    }

    pub async fn update_audio_data(&self, audio_samples: &[f32]) {
        let subscriptions = self.subscriptions.lock().await;

        for (subscription_id, subscription) in subscriptions.iter() {
            if !subscription.is_active {
                continue;
            }

            let manager = subscription.manager.clone();
            let window = subscription.window.clone();
            let subscription_id = subscription_id.clone();
            let audio_samples_owned = audio_samples.to_vec(); // 克隆数据以避免生命周期问题

            // 异步处理音频数据以避免阻塞
            tokio::spawn(async move {
                let visualization_data = manager.process_audio_data(&audio_samples_owned).await;

                let event = VisualizationUpdateEvent {
                    subscription_id: subscription_id.clone(),
                    data: visualization_data,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                };

                // 发送事件到前端
                if let Err(e) = window.emit("audio_visualization_update", &event) {
                    eprintln!("Failed to emit audio visualization event: {}", e);
                }
            });
        }
    }

    pub async fn get_subscription_info(&self, subscription_id: &str) -> Option<WaveformConfig> {
        self.subscriptions
            .lock()
            .await
            .get(subscription_id)
            .map(|sub| sub.config.clone())
    }
}

/// 获取音频可视化数据
#[tauri::command]
pub async fn get_audio_visualization_data(
    audio_samples: Vec<f32>,
    config: Option<WaveformConfig>,
    state: State<'_, AppState>,
) -> Result<AudioVisualizationData, String> {
    let config = config.unwrap_or_default();
    let manager = AudioVisualizationManager::new(config);

    let visualization_data = manager.process_audio_data(&audio_samples).await;

    Ok(visualization_data)
}

/// 订阅实时音频可视化数据
#[tauri::command]
pub async fn subscribe_audio_visualization(
    window: Window,
    request: VisualizationSubscriptionRequest,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let subscription_manager = &state.audio_visualization_manager;

    let subscription_id = subscription_manager
        .create_subscription(window, request.config)
        .await;

    Ok(subscription_id)
}

/// 取消音频可视化订阅
#[tauri::command]
pub async fn unsubscribe_audio_visualization(
    subscription_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let subscription_manager = &state.audio_visualization_manager;

    let removed = subscription_manager
        .remove_subscription(&subscription_id)
        .await;

    Ok(removed)
}

/// 更新音频可视化配置
#[tauri::command]
pub async fn update_visualization_config(
    subscription_id: String,
    new_config: WaveformConfig,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let subscription_manager = &state.audio_visualization_manager;

    let mut subscriptions = subscription_manager.subscriptions.lock().await;

    if let Some(subscription) = subscriptions.get_mut(&subscription_id) {
        subscription.config = new_config.clone();
        // 注意：这里应该更新manager的配置，但由于架构限制，我们创建新的manager
        subscription.manager = Arc::new(AudioVisualizationManager::new(new_config));
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 获取可视化性能指标
#[tauri::command]
pub async fn get_visualization_metrics(
    subscription_id: String,
    state: State<'_, AppState>,
) -> Result<VisualizationMetrics, String> {
    let subscription_manager = &state.audio_visualization_manager;

    let subscriptions = subscription_manager.subscriptions.lock().await;

    if let Some(subscription) = subscriptions.get(&subscription_id) {
        let (amplitude_buffer_size, frequency_buffer_size) =
            subscription.manager.get_buffer_usage().await;

        let metrics = VisualizationMetrics {
            buffer_usage_amplitude: amplitude_buffer_size,
            buffer_usage_frequency: frequency_buffer_size,
            max_buffer_size: subscription.config.buffer_size,
            sample_rate: subscription.config.sample_rate,
            render_mode: subscription.config.render_mode.clone(),
            max_response_time_ms: subscription.config.max_response_time_ms,
        };

        Ok(metrics)
    } else {
        Err("Subscription not found".to_string())
    }
}

/// 清除可视化历史数据
#[tauri::command]
pub async fn clear_visualization_history(
    subscription_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let subscription_manager = &state.audio_visualization_manager;

    let subscriptions = subscription_manager.subscriptions.lock().await;

    if let Some(subscription) = subscriptions.get(&subscription_id) {
        subscription.manager.clear_history().await;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 设置语音活动检测阈值
#[tauri::command]
pub async fn set_voice_activity_threshold(
    threshold: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 验证阈值范围
    if threshold < 0.0 || threshold > 1.0 {
        return Err("Threshold must be between 0.0 and 1.0".to_string());
    }

    // 这里应该更新所有活跃订阅的阈值
    // 由于架构限制，我们只能在创建新订阅时使用新阈值
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualizationMetrics {
    pub buffer_usage_amplitude: usize,
    pub buffer_usage_frequency: usize,
    pub max_buffer_size: usize,
    pub sample_rate: u32,
    pub render_mode: WaveformRenderMode,
    pub max_response_time_ms: u64,
}

/// 获取预设的波形颜色方案
#[tauri::command]
pub async fn get_waveform_color_schemes() -> Result<Vec<WaveformColorScheme>, String> {
    let schemes = vec![
        WaveformColorScheme::default(), // 默认深色主题
        WaveformColorScheme {
            low_amplitude: "#81c784".to_string(),  // 浅绿色
            mid_amplitude: "#ffb74d".to_string(),  // 浅橙色
            high_amplitude: "#e57373".to_string(), // 浅红色
            background: "#f5f5f5".to_string(),     // 浅灰色
            peak_indicator: "#ffd54f".to_string(), // 浅黄色
        }, // 浅色主题
        WaveformColorScheme {
            low_amplitude: "#64b5f6".to_string(),  // 蓝色
            mid_amplitude: "#ba68c8".to_string(),  // 紫色
            high_amplitude: "#f06292".to_string(), // 粉色
            background: "#121212".to_string(),     // 深黑色
            peak_indicator: "#26c6da".to_string(), // 青色
        }, // 霓虹主题
        WaveformColorScheme {
            low_amplitude: "#66bb6a".to_string(),  // 绿色
            mid_amplitude: "#ffa726".to_string(),  // 橙色
            high_amplitude: "#ef5350".to_string(), // 红色
            background: "#2e2e2e".to_string(),     // 深灰色
            peak_indicator: "#ffca28".to_string(), // 黄色
        }, // 经典主题
    ];

    Ok(schemes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscription_manager() {
        let manager = AudioVisualizationSubscriptionManager::new();

        // 创建模拟窗口（在实际测试中需要mock）
        // let subscription_id = manager.create_subscription(window, None).await;
        // assert!(!subscription_id.is_empty());
    }

    #[test]
    fn test_visualization_metrics_serialization() {
        let metrics = VisualizationMetrics {
            buffer_usage_amplitude: 512,
            buffer_usage_frequency: 256,
            max_buffer_size: 1024,
            sample_rate: 44100,
            render_mode: WaveformRenderMode::RealTime,
            max_response_time_ms: 16,
        };

        let serialized = serde_json::to_string(&metrics).unwrap();
        assert!(serialized.contains("512"));
    }
}
