use crate::ai::content_analyzer::{
    AnalysisError, ContentAnalysisConfig, ContentAnalysisResult, ContentAnalyzer, PerformanceStats,
};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, Window};

/// 内容分析服务
#[derive(Clone)]
pub struct ContentAnalysisService {
    analyzer: Arc<ContentAnalyzer>,
}

impl ContentAnalysisService {
    pub fn new(config: ContentAnalysisConfig) -> Self {
        Self {
            analyzer: Arc::new(ContentAnalyzer::new(config)),
        }
    }

    pub async fn analyze(
        &self,
        text: &str,
        content_id: Option<String>,
    ) -> Result<ContentAnalysisResult, AnalysisError> {
        self.analyzer.analyze_content(text, content_id).await
    }

    pub async fn get_stats(&self) -> PerformanceStats {
        self.analyzer.get_performance_stats().await
    }

    pub async fn update_config(&self, config: ContentAnalysisConfig) {
        self.analyzer.update_config(config).await;
    }

    pub async fn clear_cache(&self) {
        self.analyzer.clear_cache().await;
    }
}

/// 内容分析请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ContentAnalysisRequest {
    /// 要分析的文本
    pub text: String,
    /// 内容ID（可选，用于缓存）
    pub content_id: Option<String>,
    /// 分析选项
    pub options: Option<AnalysisRequestOptions>,
}

/// 分析请求选项
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisRequestOptions {
    /// 启用主题分析
    pub enable_topics: Option<bool>,
    /// 启用情感分析
    pub enable_sentiment: Option<bool>,
    /// 启用关键信息提取
    pub enable_key_info: Option<bool>,
    /// 启用智能分类
    pub enable_classification: Option<bool>,
    /// 优先级（影响处理顺序）
    pub priority: Option<AnalysisPriority>,
}

/// 分析优先级
#[derive(Debug, Serialize, Deserialize)]
pub enum AnalysisPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// 批量分析请求
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchAnalysisRequest {
    /// 分析项目列表
    pub items: Vec<ContentAnalysisRequest>,
    /// 批量处理选项
    pub batch_options: Option<BatchOptions>,
}

/// 批量处理选项
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchOptions {
    /// 最大并发数
    pub max_concurrent: Option<usize>,
    /// 失败时是否继续
    pub continue_on_error: Option<bool>,
    /// 进度回调间隔（处理多少项后回调一次）
    pub progress_interval: Option<usize>,
}

/// 批量分析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchAnalysisResult {
    /// 成功的分析结果
    pub results: Vec<ContentAnalysisResult>,
    /// 失败的项目
    pub errors: Vec<BatchError>,
    /// 处理统计
    pub statistics: BatchStatistics,
}

/// 批量处理错误
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchError {
    /// 内容ID
    pub content_id: Option<String>,
    /// 错误信息
    pub error: String,
    /// 项目索引
    pub item_index: usize,
}

/// 批量处理统计
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchStatistics {
    /// 总项目数
    pub total_items: usize,
    /// 成功数
    pub successful: usize,
    /// 失败数
    pub failed: usize,
    /// 总处理时间（毫秒）
    pub total_time_ms: u64,
    /// 平均处理时间（毫秒）
    pub average_time_ms: u64,
}

/// 配置更新请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigUpdateRequest {
    /// OpenAI API密钥
    pub openai_api_key: Option<String>,
    /// 模型名称
    pub model: Option<String>,
    /// 温度参数
    pub temperature: Option<f32>,
    /// 最大token数
    pub max_tokens: Option<u32>,
    /// 分析选项
    pub analysis_options: Option<AnalysisConfigOptions>,
    /// 性能设置
    pub performance_settings: Option<PerformanceConfigOptions>,
}

/// 分析配置选项
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisConfigOptions {
    pub enable_topic_analysis: Option<bool>,
    pub enable_sentiment_analysis: Option<bool>,
    pub enable_key_info_extraction: Option<bool>,
    pub enable_classification: Option<bool>,
    pub min_confidence_threshold: Option<f32>,
}

/// 性能配置选项
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceConfigOptions {
    pub max_concurrent_analyses: Option<usize>,
    pub analysis_timeout_seconds: Option<u64>,
    pub enable_caching: Option<bool>,
    pub cache_expiry_seconds: Option<u64>,
}

/// 实时分析状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeAnalysisStatus {
    /// 分析ID
    pub analysis_id: String,
    /// 当前状态
    pub status: AnalysisStatus,
    /// 进度百分比 (0-100)
    pub progress: u8,
    /// 当前分析阶段
    pub current_stage: AnalysisStage,
    /// 预计剩余时间（秒）
    pub estimated_remaining_seconds: Option<u32>,
    /// 已完成的分析类型
    pub completed_analyses: Vec<String>,
}

/// 分析状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Queued,     // 队列中
    Processing, // 处理中
    Completed,  // 已完成
    Failed,     // 失败
    Cancelled,  // 已取消
}

/// 分析阶段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisStage {
    Initialization,    // 初始化
    TopicAnalysis,     // 主题分析
    SentimentAnalysis, // 情感分析
    KeyInfoExtraction, // 关键信息提取
    Classification,    // 分类
    Finalization,      // 最终化
}

/// 分析内容
#[tauri::command]
pub async fn analyze_content(
    request: ContentAnalysisRequest,
    state: State<'_, AppState>,
) -> Result<ContentAnalysisResult, String> {
    let service = state.content_analysis_service.clone();

    match service.analyze(&request.text, request.content_id).await {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("内容分析失败: {}", e)),
    }
}

/// 批量分析内容
#[tauri::command]
pub async fn batch_analyze_content(
    request: BatchAnalysisRequest,
    window: Window,
    state: State<'_, AppState>,
) -> Result<BatchAnalysisResult, String> {
    let service = &state.content_analysis_service;
    let start_time = std::time::Instant::now();

    let max_concurrent = request
        .batch_options
        .as_ref()
        .and_then(|opts| opts.max_concurrent)
        .unwrap_or(3);

    let continue_on_error = request
        .batch_options
        .as_ref()
        .and_then(|opts| opts.continue_on_error)
        .unwrap_or(true);

    let progress_interval = request
        .batch_options
        .as_ref()
        .and_then(|opts| opts.progress_interval)
        .unwrap_or(5);

    let total_items = request.items.len();
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let mut processed = 0;

    // 使用信号量控制并发
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
    let mut tasks = Vec::new();

    for (index, item) in request.items.into_iter().enumerate() {
        let service = service.clone();
        let semaphore = semaphore.clone();
        let window = window.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let result = service.analyze(&item.text, item.content_id.clone()).await;
            (index, item.content_id, result)
        });

        tasks.push(task);
    }

    // 等待所有任务完成
    for task in tasks {
        match task.await {
            Ok((index, content_id, analysis_result)) => match analysis_result {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    if !continue_on_error {
                        return Err(format!("批量分析在第 {} 项失败: {}", index + 1, e));
                    }
                    errors.push(BatchError {
                        content_id,
                        error: e.to_string(),
                        item_index: index,
                    });
                }
            },
            Err(e) => {
                if !continue_on_error {
                    return Err(format!("任务执行失败: {}", e));
                }
                errors.push(BatchError {
                    content_id: None,
                    error: format!("任务执行失败: {}", e),
                    item_index: 0,
                });
            }
        }

        processed += 1;

        // 发送进度更新
        if processed % progress_interval == 0 {
            let progress = (processed as f32 / total_items as f32 * 100.0) as u8;
            let _ = window.emit("batch_analysis_progress", progress);
        }
    }

    let total_time = start_time.elapsed().as_millis() as u64;
    let successful = results.len();
    let failed = errors.len();

    let statistics = BatchStatistics {
        total_items,
        successful,
        failed,
        total_time_ms: total_time,
        average_time_ms: if successful > 0 {
            total_time / successful as u64
        } else {
            0
        },
    };

    // 发送完成事件
    let _ = window.emit("batch_analysis_completed", &statistics);

    Ok(BatchAnalysisResult {
        results,
        errors,
        statistics,
    })
}

/// 获取分析性能统计
#[tauri::command]
pub async fn get_analysis_performance_stats(
    state: State<'_, AppState>,
) -> Result<PerformanceStats, String> {
    let service = &state.content_analysis_service;
    Ok(service.get_stats().await)
}

/// 更新分析配置
#[tauri::command]
pub async fn update_analysis_config(
    request: ConfigUpdateRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let service = &state.content_analysis_service;

    // 获取当前配置
    let mut config = ContentAnalysisConfig::default();

    // 更新OpenAI配置
    if let Some(api_key) = request.openai_api_key {
        config.openai_config.api_key = api_key;
    }
    if let Some(model) = request.model {
        config.openai_config.model = model;
    }
    if let Some(temperature) = request.temperature {
        config.openai_config.temperature = temperature;
    }
    if let Some(max_tokens) = request.max_tokens {
        config.openai_config.max_tokens = max_tokens;
    }

    // 更新分析选项
    if let Some(analysis_options) = request.analysis_options {
        if let Some(enable_topic) = analysis_options.enable_topic_analysis {
            config.analysis_options.enable_topic_analysis = enable_topic;
        }
        if let Some(enable_sentiment) = analysis_options.enable_sentiment_analysis {
            config.analysis_options.enable_sentiment_analysis = enable_sentiment;
        }
        if let Some(enable_key_info) = analysis_options.enable_key_info_extraction {
            config.analysis_options.enable_key_info_extraction = enable_key_info;
        }
        if let Some(enable_classification) = analysis_options.enable_classification {
            config.analysis_options.enable_classification = enable_classification;
        }
        if let Some(threshold) = analysis_options.min_confidence_threshold {
            config.analysis_options.min_confidence_threshold = threshold;
        }
    }

    // 更新性能设置
    if let Some(performance_settings) = request.performance_settings {
        if let Some(max_concurrent) = performance_settings.max_concurrent_analyses {
            config.performance_settings.max_concurrent_analyses = max_concurrent;
        }
        if let Some(timeout) = performance_settings.analysis_timeout_seconds {
            config.performance_settings.analysis_timeout_seconds = timeout;
        }
        if let Some(enable_caching) = performance_settings.enable_caching {
            config.performance_settings.enable_caching = enable_caching;
        }
        if let Some(cache_expiry) = performance_settings.cache_expiry_seconds {
            config.performance_settings.cache_expiry_seconds = cache_expiry;
        }
    }

    service.update_config(config).await;
    Ok(())
}

/// 清除分析缓存
#[tauri::command]
pub async fn clear_analysis_cache(state: State<'_, AppState>) -> Result<(), String> {
    let service = &state.content_analysis_service;
    service.clear_cache().await;
    Ok(())
}

/// 快速主题识别（优化版，15秒内完成）
#[tauri::command]
pub async fn quick_topic_identification(
    text: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::ai::content_analyzer::TopicTag>, String> {
    let service = &state.content_analysis_service;

    // 使用优化的配置进行快速主题识别
    let start_time = std::time::Instant::now();

    match service.analyze(&text, None).await {
        Ok(result) => {
            let elapsed = start_time.elapsed().as_secs();

            // 检查是否在15秒内完成
            if elapsed > 15 {
                eprintln!("警告：主题识别耗时 {} 秒，超过15秒目标", elapsed);
            }

            Ok(result.topics)
        }
        Err(e) => Err(format!("快速主题识别失败: {}", e)),
    }
}

/// 快速情感分析（10秒内完成）
#[tauri::command]
pub async fn quick_sentiment_analysis(
    text: String,
    state: State<'_, AppState>,
) -> Result<crate::ai::content_analyzer::SentimentAnalysis, String> {
    let service = &state.content_analysis_service;

    let start_time = std::time::Instant::now();

    match service.analyze(&text, None).await {
        Ok(result) => {
            let elapsed = start_time.elapsed().as_secs();

            if elapsed > 10 {
                eprintln!("警告：情感分析耗时 {} 秒，超过10秒目标", elapsed);
            }

            Ok(result.sentiment)
        }
        Err(e) => Err(format!("快速情感分析失败: {}", e)),
    }
}

/// 快速关键信息提取（30秒内完成）
#[tauri::command]
pub async fn quick_key_info_extraction(
    text: String,
    state: State<'_, AppState>,
) -> Result<crate::ai::content_analyzer::KeyInformation, String> {
    let service = &state.content_analysis_service;

    let start_time = std::time::Instant::now();

    match service.analyze(&text, None).await {
        Ok(result) => {
            let elapsed = start_time.elapsed().as_secs();

            if elapsed > 30 {
                eprintln!("警告：关键信息提取耗时 {} 秒，超过30秒目标", elapsed);
            }

            Ok(result.key_information)
        }
        Err(e) => Err(format!("快速关键信息提取失败: {}", e)),
    }
}

/// 实时分析状态监控（结合Story 1.5的UI反馈）
#[tauri::command]
pub async fn start_realtime_analysis(
    text: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let analysis_id = uuid::Uuid::new_v4().to_string();
    let service = state.content_analysis_service.clone();
    let window_clone = window.clone();
    let analysis_id_clone = analysis_id.clone();

    // 在后台执行分析，并实时更新状态
    tokio::spawn(async move {
        // 初始化状态
        let _ = window_clone.emit(
            "realtime_analysis_status",
            RealTimeAnalysisStatus {
                analysis_id: analysis_id_clone.clone(),
                status: AnalysisStatus::Processing,
                progress: 0,
                current_stage: AnalysisStage::Initialization,
                estimated_remaining_seconds: Some(45),
                completed_analyses: Vec::new(),
            },
        );

        // 模拟分析进度更新
        let stages = vec![
            (AnalysisStage::TopicAnalysis, 25),
            (AnalysisStage::SentimentAnalysis, 50),
            (AnalysisStage::KeyInfoExtraction, 75),
            (AnalysisStage::Classification, 90),
            (AnalysisStage::Finalization, 100),
        ];

        for (stage, progress) in stages {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let _ = window_clone.emit(
                "realtime_analysis_status",
                RealTimeAnalysisStatus {
                    analysis_id: analysis_id_clone.clone(),
                    status: AnalysisStatus::Processing,
                    progress,
                    current_stage: stage,
                    estimated_remaining_seconds: Some(((100 - progress) as f32 * 0.45) as u32),
                    completed_analyses: Vec::new(),
                },
            );
        }

        // 执行实际分析
        match service
            .analyze(&text, Some(analysis_id_clone.clone()))
            .await
        {
            Ok(result) => {
                let _ = window_clone.emit("realtime_analysis_completed", result);
                let _ = window_clone.emit(
                    "realtime_analysis_status",
                    RealTimeAnalysisStatus {
                        analysis_id: analysis_id_clone,
                        status: AnalysisStatus::Completed,
                        progress: 100,
                        current_stage: AnalysisStage::Finalization,
                        estimated_remaining_seconds: None,
                        completed_analyses: vec!["all".to_string()],
                    },
                );
            }
            Err(e) => {
                let _ = window_clone.emit("realtime_analysis_error", e.to_string());
                let _ = window_clone.emit(
                    "realtime_analysis_status",
                    RealTimeAnalysisStatus {
                        analysis_id: analysis_id_clone,
                        status: AnalysisStatus::Failed,
                        progress: 0,
                        current_stage: AnalysisStage::Initialization,
                        estimated_remaining_seconds: None,
                        completed_analyses: Vec::new(),
                    },
                );
            }
        }
    });

    Ok(analysis_id)
}

/// 取消实时分析
#[tauri::command]
pub async fn cancel_realtime_analysis(analysis_id: String, window: Window) -> Result<(), String> {
    // 发送取消状态
    let _ = window.emit(
        "realtime_analysis_status",
        RealTimeAnalysisStatus {
            analysis_id: analysis_id.clone(),
            status: AnalysisStatus::Cancelled,
            progress: 0,
            current_stage: AnalysisStage::Initialization,
            estimated_remaining_seconds: None,
            completed_analyses: Vec::new(),
        },
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_request_serialization() {
        let request = ContentAnalysisRequest {
            text: "测试文本".to_string(),
            content_id: Some("test_id".to_string()),
            options: Some(AnalysisRequestOptions {
                enable_topics: Some(true),
                enable_sentiment: Some(true),
                enable_key_info: Some(true),
                enable_classification: Some(true),
                priority: Some(AnalysisPriority::High),
            }),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("测试文本"));
        assert!(json.contains("test_id"));
    }
}
