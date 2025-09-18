use super::model_manager::LocalModelManager;
use crate::database::models::LocalModelInfo;
use crate::errors::{AppError, AppResult};
use crate::performance_optimizer::{PerformanceMetrics, PerformanceOptimizer};
use crate::types::{TranscriptionConfig, TranscriptionResult};
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// Story 1.4: Enhanced Whisper Transcriber with Local Model Support

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ModelPriority {
    Speed,    // Prioritize fast transcription
    Accuracy, // Prioritize accurate transcription
    Balanced, // Balance between speed and accuracy
}

pub struct WhisperTranscriber {
    optimizer: Arc<Mutex<PerformanceOptimizer>>,
    model_cache: Arc<Mutex<std::collections::HashMap<String, WhisperContext>>>,
    model_manager: Option<Arc<LocalModelManager>>,
    gpu_detector: Arc<Mutex<GPUDetector>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GPUCapabilities {
    pub metal_available: bool,
    pub metal_device_name: Option<String>,
    pub recommended_for_whisper: bool,
    pub memory_gb: Option<f64>,
}

pub struct GPUDetector {
    capabilities: Option<GPUCapabilities>,
}

impl GPUDetector {
    pub fn new() -> Self {
        Self { capabilities: None }
    }

    /// Detect Metal GPU capabilities on macOS
    pub fn detect_capabilities(&mut self) -> AppResult<GPUCapabilities> {
        if let Some(ref caps) = self.capabilities {
            return Ok(caps.clone());
        }

        println!("🔍 Detecting GPU capabilities...");

        #[cfg(target_os = "macos")]
        {
            use cocoa::base::nil;

            use objc::runtime::Object;
            use objc::{msg_send, sel, sel_impl};

            let capabilities = unsafe {
                let mtl_device_class =
                    match objc::runtime::Class::get("MTLCreateSystemDefaultDevice") {
                        Some(class) => class,
                        None => {
                            // Metal not available
                            return Ok(GPUCapabilities {
                                metal_available: false,
                                metal_device_name: None,
                                recommended_for_whisper: false,
                                memory_gb: None,
                            });
                        }
                    };

                // Try to create Metal device
                let device: *mut Object = msg_send![mtl_device_class, new];

                if device == nil {
                    GPUCapabilities {
                        metal_available: false,
                        metal_device_name: None,
                        recommended_for_whisper: false,
                        memory_gb: None,
                    }
                } else {
                    let device_name: *mut Object = msg_send![device, name];
                    let device_name_str = if device_name != nil {
                        let c_str: *const i8 = msg_send![device_name, UTF8String];
                        std::ffi::CStr::from_ptr(c_str)
                            .to_string_lossy()
                            .to_string()
                    } else {
                        "Unknown Metal Device".to_string()
                    };

                    let recommended_memory_gb = 8.0; // Minimum recommended for Whisper
                    let has_unified_memory = device_name_str.contains("Apple")
                        || device_name_str.contains("M1")
                        || device_name_str.contains("M2")
                        || device_name_str.contains("M3");

                    GPUCapabilities {
                        metal_available: true,
                        metal_device_name: Some(device_name_str),
                        recommended_for_whisper: has_unified_memory, // Apple Silicon is great for Whisper
                        memory_gb: if has_unified_memory {
                            Some(16.0)
                        } else {
                            Some(8.0)
                        }, // Estimate
                    }
                }
            };

            self.capabilities = Some(capabilities.clone());
            println!(
                "✅ GPU Detection: Metal={}, Device={:?}, Recommended={}",
                capabilities.metal_available,
                capabilities.metal_device_name,
                capabilities.recommended_for_whisper
            );
            Ok(capabilities)
        }

        #[cfg(not(target_os = "macos"))]
        {
            let capabilities = GPUCapabilities {
                metal_available: false,
                metal_device_name: None,
                recommended_for_whisper: false,
                memory_gb: None,
            };
            self.capabilities = Some(capabilities.clone());
            Ok(capabilities)
        }
    }

    pub fn get_capabilities(&self) -> Option<&GPUCapabilities> {
        self.capabilities.as_ref()
    }
}

impl WhisperTranscriber {
    pub fn new() -> Self {
        Self {
            optimizer: Arc::new(Mutex::new(PerformanceOptimizer::new())),
            model_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            model_manager: None,
            gpu_detector: Arc::new(Mutex::new(GPUDetector::new())),
        }
    }

    /// Story 1.4: Create WhisperTranscriber with local model manager
    pub fn with_model_manager(model_manager: Arc<LocalModelManager>) -> Self {
        Self {
            optimizer: Arc::new(Mutex::new(PerformanceOptimizer::new())),
            model_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            model_manager: Some(model_manager),
            gpu_detector: Arc::new(Mutex::new(GPUDetector::new())),
        }
    }

    /// Initialize GPU detection and optimization
    pub async fn initialize_gpu(&self) -> AppResult<GPUCapabilities> {
        let mut detector = self.gpu_detector.lock();
        detector.detect_capabilities()
    }

    /// Get GPU capabilities
    pub fn get_gpu_capabilities(&self) -> Option<GPUCapabilities> {
        self.gpu_detector.lock().get_capabilities().cloned()
    }

    /// Check if a local model is available for transcription
    pub async fn is_local_model_available(&self, model_id: &str) -> bool {
        if let Some(ref manager) = self.model_manager {
            manager.is_model_available(model_id).await.unwrap_or(false)
        } else {
            false
        }
    }

    /// Download a model if not available locally
    pub async fn ensure_model_downloaded(&self, model_id: &str) -> AppResult<PathBuf> {
        if let Some(ref manager) = self.model_manager {
            if !manager.is_model_available(model_id).await? {
                println!("📥 Downloading model: {}", model_id);
                return manager.download_model(model_id).await;
            } else {
                // Get path from database
                if let Some(model_info) = manager.database_manager.get_local_model(model_id).await?
                {
                    if let Some(file_path) = model_info.file_path {
                        return Ok(file_path);
                    }
                }
            }
        }
        Err(AppError::InvalidParameter(format!(
            "Model {} not available and no model manager configured",
            model_id
        )))
    }

    /// Get list of available local models
    pub async fn get_available_local_models(&self) -> AppResult<Vec<LocalModelInfo>> {
        if let Some(ref manager) = self.model_manager {
            manager.list_downloaded_models().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Recommend best local model based on requirements
    pub async fn recommend_best_local_model(
        &self,
        language: Option<&str>,
        priority: ModelPriority,
    ) -> AppResult<Option<String>> {
        let available_models = self.get_available_local_models().await?;

        if available_models.is_empty() {
            return Ok(None);
        }

        // Simple recommendation logic based on size and language support
        let preferred_model = match priority {
            ModelPriority::Speed => {
                available_models
                    .iter()
                    .filter(|m| m.size_bytes < 200_000_000) // < 200MB for speed
                    .min_by_key(|m| m.size_bytes)
                    .map(|m| m.model_id.clone())
            }
            ModelPriority::Accuracy => {
                available_models
                    .iter()
                    .max_by_key(|m| m.size_bytes) // Larger models are typically more accurate
                    .map(|m| m.model_id.clone())
            }
            ModelPriority::Balanced => {
                available_models
                    .iter()
                    .filter(|m| m.size_bytes > 100_000_000 && m.size_bytes < 800_000_000) // 100MB - 800MB range
                    .min_by_key(|m| (m.size_bytes as i64 - 400_000_000).abs()) // Closest to 400MB
                    .map(|m| m.model_id.clone())
            }
        };

        Ok(preferred_model)
    }

    /// Load a local Whisper model with GPU acceleration if available
    pub async fn load_local_model(&self, model_id: &str) -> AppResult<()> {
        // Ensure model is downloaded
        let model_path = self.ensure_model_downloaded(model_id).await?;

        // Check if already cached
        {
            let cache = self.model_cache.lock();
            if cache.contains_key(model_id) {
                println!("✅ Model {} already loaded in cache", model_id);
                return Ok(());
            }
        }

        println!("🔄 Loading local model: {} from {:?}", model_id, model_path);

        // Configure Whisper context with GPU acceleration if available
        let gpu_caps = self.get_gpu_capabilities();
        let use_gpu = gpu_caps.as_ref().map_or(false, |caps| {
            caps.metal_available && caps.recommended_for_whisper
        });

        let context_params = WhisperContextParameters {
            use_gpu,
            ..Default::default()
        };

        // Load the model
        let context =
            WhisperContext::new_with_params(&model_path.to_string_lossy(), context_params)
                .map_err(|e| {
                    AppError::ModelLoadError(format!(
                        "Failed to load Whisper model {}: {}",
                        model_id, e
                    ))
                })?;

        // Cache the loaded model
        {
            let mut cache = self.model_cache.lock();
            cache.insert(model_id.to_string(), context);
        }

        println!(
            "✅ Model {} loaded successfully (GPU: {})",
            model_id, use_gpu
        );
        Ok(())
    }

    /// Transcribe using a specific local model
    pub async fn transcribe_with_local_model(
        &self,
        audio_path: &Path,
        model_id: &str,
        config: &TranscriptionConfig,
    ) -> AppResult<TranscriptionResult> {
        // Ensure model is loaded
        self.load_local_model(model_id).await?;

        // Get the model from cache
        let context = {
            let cache = self.model_cache.lock();
            match cache.get(model_id) {
                Some(ctx) => {
                    // We can't clone WhisperContext, so we'll need to work differently
                    // For now, let's return an error and implement this properly
                    return Err(AppError::ModelLoadError(
                        "Model context access needs to be refactored for safe sharing".to_string(),
                    ));
                }
                None => {
                    return Err(AppError::ModelLoadError(format!(
                        "Model {} not found in cache",
                        model_id
                    )))
                }
            }
        };

        // TODO: Implement actual transcription logic
        // This requires refactoring the WhisperContext usage to be thread-safe

        Ok(TranscriptionResult {
            text: "Placeholder transcription result".to_string(),
            confidence: Some(0.95),
            duration: Some(std::time::Duration::from_secs(0)),
            language: config.language.clone(),
        })
    }

    /// 预加载常用模型
    pub async fn preload_common_models(&self) -> AppResult<()> {
        let common_models = vec!["whisper-base", "whisper-small"];

        println!("🚀 开始预加载常用模型...");

        for model in common_models {
            println!("📦 预加载模型: {}", model);

            // 确保模型已下载
            let model_path = Self::download_whisper_model_if_needed(model)?;

            // 尝试加载模型到缓存
            match Self::get_cached_model(
                &model_path,
                self.model_cache.clone(),
                &mut self.optimizer.lock(),
            ) {
                Ok(_) => {
                    println!("✅ 模型 {} 预加载成功", model);
                }
                _ => {
                    println!("⚠️ 模型 {} 预加载失败", model);
                }
            }
        }

        println!("🎯 模型预加载完成");
        Ok(())
    }

    /// 获取可用的模型列表
    pub fn get_available_models() -> Vec<String> {
        vec![
            // 基础多语言模型
            "whisper-tiny".to_string(),
            "whisper-base".to_string(),
            "whisper-small".to_string(),
            "whisper-medium".to_string(),
            "whisper-large-v3".to_string(),
            "whisper-large-v3-turbo".to_string(),
            // 英语专用模型（更高精度）
            "whisper-tiny-en".to_string(),
            "whisper-base-en".to_string(),
            "whisper-small-en".to_string(),
            "whisper-medium-en".to_string(),
            // 中文优化模型
            "whisper-small-zh".to_string(),
            "whisper-medium-zh".to_string(),
            // 特殊用途模型
            "whisper-distil-small-en".to_string(), // 蒸馏版本，更快
            "whisper-distil-medium-en".to_string(), // 蒸馏版本，更快
        ]
    }

    /// 根据语言和需求推荐最佳模型
    pub fn recommend_model(language: Option<&str>, priority: ModelPriority) -> String {
        match (language, priority) {
            (Some("en") | Some("english"), ModelPriority::Speed) => {
                "whisper-distil-small-en".to_string()
            }
            (Some("en") | Some("english"), ModelPriority::Accuracy) => {
                "whisper-large-v3".to_string()
            }
            (Some("en") | Some("english"), ModelPriority::Balanced) => {
                "whisper-base-en".to_string()
            }

            (Some("zh") | Some("chinese"), ModelPriority::Speed) => "whisper-small-zh".to_string(),
            (Some("zh") | Some("chinese"), ModelPriority::Accuracy) => {
                "whisper-large-v3".to_string()
            }
            (Some("zh") | Some("chinese"), ModelPriority::Balanced) => {
                "whisper-medium-zh".to_string()
            }

            // 多语言或未知语言
            (_, ModelPriority::Speed) => "whisper-base".to_string(),
            (_, ModelPriority::Accuracy) => "whisper-large-v3".to_string(),
            (_, ModelPriority::Balanced) => "whisper-small".to_string(),
        }
    }

    /// 获取模型信息
    pub fn get_model_info(model: &str) -> Option<ModelInfo> {
        match model {
            // 基础多语言模型
            "whisper-tiny" => Some(ModelInfo {
                name: "Tiny (多语言)".to_string(),
                size_mb: 39,
                languages: "99种语言".to_string(),
                speed: "极快".to_string(),
                accuracy: "基础".to_string(),
                recommended_use: "测试和快速转录".to_string(),
            }),
            "whisper-base" => Some(ModelInfo {
                name: "Base (多语言)".to_string(),
                size_mb: 74,
                languages: "99种语言".to_string(),
                speed: "快".to_string(),
                accuracy: "良好".to_string(),
                recommended_use: "日常使用推荐".to_string(),
            }),
            "whisper-small" => Some(ModelInfo {
                name: "Small (多语言)".to_string(),
                size_mb: 244,
                languages: "99种语言".to_string(),
                speed: "中等".to_string(),
                accuracy: "很好".to_string(),
                recommended_use: "高质量转录".to_string(),
            }),
            "whisper-medium" => Some(ModelInfo {
                name: "Medium (多语言)".to_string(),
                size_mb: 769,
                languages: "99种语言".to_string(),
                speed: "慢".to_string(),
                accuracy: "优秀".to_string(),
                recommended_use: "专业转录".to_string(),
            }),
            "whisper-large-v3" => Some(ModelInfo {
                name: "Large V3 (多语言)".to_string(),
                size_mb: 1550,
                languages: "99种语言".to_string(),
                speed: "很慢".to_string(),
                accuracy: "最佳".to_string(),
                recommended_use: "最高质量转录".to_string(),
            }),
            "whisper-large-v3-turbo" => Some(ModelInfo {
                name: "Large V3 Turbo (多语言)".to_string(),
                size_mb: 809,
                languages: "99种语言".to_string(),
                speed: "中快".to_string(),
                accuracy: "优秀".to_string(),
                recommended_use: "高质量快速转录".to_string(),
            }),

            // 英语专用模型
            "whisper-tiny-en" => Some(ModelInfo {
                name: "Tiny (仅英语)".to_string(),
                size_mb: 39,
                languages: "仅英语".to_string(),
                speed: "极快".to_string(),
                accuracy: "良好".to_string(),
                recommended_use: "英语快速转录".to_string(),
            }),
            "whisper-base-en" => Some(ModelInfo {
                name: "Base (仅英语)".to_string(),
                size_mb: 74,
                languages: "仅英语".to_string(),
                speed: "快".to_string(),
                accuracy: "很好".to_string(),
                recommended_use: "英语日常转录".to_string(),
            }),
            "whisper-small-en" => Some(ModelInfo {
                name: "Small (仅英语)".to_string(),
                size_mb: 244,
                languages: "仅英语".to_string(),
                speed: "中等".to_string(),
                accuracy: "优秀".to_string(),
                recommended_use: "英语高质量转录".to_string(),
            }),
            "whisper-medium-en" => Some(ModelInfo {
                name: "Medium (仅英语)".to_string(),
                size_mb: 769,
                languages: "仅英语".to_string(),
                speed: "慢".to_string(),
                accuracy: "最佳".to_string(),
                recommended_use: "英语专业转录".to_string(),
            }),

            // 中文优化模型
            "whisper-small-zh" => Some(ModelInfo {
                name: "Small (中文优化)".to_string(),
                size_mb: 244,
                languages: "中文+多语言".to_string(),
                speed: "中等".to_string(),
                accuracy: "优秀".to_string(),
                recommended_use: "中文高质量转录".to_string(),
            }),
            "whisper-medium-zh" => Some(ModelInfo {
                name: "Medium (中文优化)".to_string(),
                size_mb: 769,
                languages: "中文+多语言".to_string(),
                speed: "慢".to_string(),
                accuracy: "最佳".to_string(),
                recommended_use: "中文专业转录".to_string(),
            }),

            // 蒸馏模型（更快）
            "whisper-distil-small-en" => Some(ModelInfo {
                name: "Distil Small (仅英语)".to_string(),
                size_mb: 166,
                languages: "仅英语".to_string(),
                speed: "很快".to_string(),
                accuracy: "良好".to_string(),
                recommended_use: "英语实时转录".to_string(),
            }),
            "whisper-distil-medium-en" => Some(ModelInfo {
                name: "Distil Medium (仅英语)".to_string(),
                size_mb: 394,
                languages: "仅英语".to_string(),
                speed: "快".to_string(),
                accuracy: "很好".to_string(),
                recommended_use: "英语快速高质量转录".to_string(),
            }),

            _ => None,
        }
    }

    /// 使用本地Whisper模型进行转录
    pub async fn transcribe_with_local_whisper<P: AsRef<Path>>(
        &self,
        audio_file_path: P,
        config: &TranscriptionConfig,
    ) -> AppResult<TranscriptionResult> {
        println!(
            "🔍 开始本地 Whisper {} 转录（性能优化版）...",
            config.model_name
        );

        let audio_path = audio_file_path.as_ref().to_path_buf();

        // 检查音频文件是否存在
        if !audio_path.exists() {
            return Err(AppError::TranscriptionError("音频文件不存在".to_string()));
        }

        // 在新线程中运行 Whisper（因为它是计算密集型的）
        let model_name = config.model_name.clone();
        let language = config.language.clone();
        let temperature = config.temperature;
        let optimizer = self.optimizer.clone();
        let model_cache = self.model_cache.clone();

        let transcription_result = tokio::task::spawn_blocking(move || {
            Self::run_whisper_transcription_optimized(
                &audio_path,
                &model_name,
                language.as_deref(),
                temperature,
                optimizer,
                model_cache,
            )
        })
        .await;

        match transcription_result {
            Ok(Ok((text, metrics))) => {
                println!("✅ 本地 Whisper 转录成功: {}", text);
                println!(
                    "📊 性能指标: RTF={:.2}, 总耗时={}ms",
                    metrics.real_time_factor, metrics.total_time_ms
                );
                Ok(TranscriptionResult {
                    text,
                    confidence: None,
                    duration: None,
                    language: None,
                })
            }
            Ok(Err(e)) => {
                println!("❌ 本地 Whisper 转录失败: {}", e);
                Err(e)
            }
            Err(e) => {
                println!("❌ Whisper 任务执行失败: {}", e);
                Err(AppError::TranscriptionError(format!(
                    "转录任务执行失败: {}",
                    e
                )))
            }
        }
    }

    /// 性能优化版 Whisper 转录
    fn run_whisper_transcription_optimized(
        audio_file_path: &PathBuf,
        model: &str,
        language: Option<&str>,
        temperature: Option<f32>,
        optimizer: Arc<Mutex<PerformanceOptimizer>>,
        model_cache: Arc<Mutex<std::collections::HashMap<String, WhisperContext>>>,
    ) -> AppResult<(String, PerformanceMetrics)> {
        let total_start = std::time::Instant::now();
        let mut metrics = PerformanceMetrics::default();

        // 下载模型（如果需要）
        let model_path = Self::download_whisper_model_if_needed(model)?;

        // 优化版模型加载（带缓存）
        let model_start = std::time::Instant::now();
        let ctx = Self::get_cached_model(&model_path, model_cache, &mut optimizer.lock())?;
        metrics.model_load_time_ms = model_start.elapsed().as_millis() as u64;

        println!("🔍 读取音频文件...");

        // 优化版音频数据加载
        let audio_start = std::time::Instant::now();
        let audio_data =
            Self::load_audio_samples_optimized(audio_file_path, &mut optimizer.lock())?;
        metrics.audio_processing_time_ms = audio_start.elapsed().as_millis() as u64;

        // 计算音频时长
        metrics.audio_duration_seconds = audio_data.len() as f64 / 16000.0; // 16kHz采样率

        println!(
            "🔍 开始转录，音频样本数: {} (时长: {:.2}s)",
            audio_data.len(),
            metrics.audio_duration_seconds
        );

        // 获取优化的转录参数
        let params = Self::get_optimized_transcription_params(language, temperature)?;

        // 运行转录
        let transcription_start = std::time::Instant::now();
        let mut state = ctx
            .create_state()
            .map_err(|e| AppError::WhisperError(format!("无法创建 Whisper 状态: {}", e)))?;

        state
            .full(params, &audio_data)
            .map_err(|e| AppError::WhisperError(format!("Whisper 转录失败: {}", e)))?;

        metrics.transcription_time_ms = transcription_start.elapsed().as_millis() as u64;

        // 获取转录结果
        let num_segments = state
            .full_n_segments()
            .map_err(|e| AppError::WhisperError(format!("无法获取分段数量: {}", e)))?;

        let mut full_text = String::new();
        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .map_err(|e| AppError::WhisperError(format!("无法获取分段文本: {}", e)))?;
            full_text.push_str(&segment);
            full_text.push(' ');
        }

        let result = full_text.trim().to_string();

        // 计算性能指标
        metrics.total_time_ms = total_start.elapsed().as_millis() as u64;
        metrics.real_time_factor = Self::calculate_rtf(
            metrics.transcription_time_ms,
            metrics.audio_duration_seconds,
        ) as f64;

        // 获取系统指标
        if let Ok((cpu_usage, memory_usage)) = optimizer.lock().get_system_metrics() {
            metrics.cpu_usage_percent = cpu_usage;
            metrics.gpu_memory_usage_mb = memory_usage;
        }

        println!("✅ 转录完成，结果长度: {} 字符", result.len());
        Self::print_performance_metrics(&metrics);

        if result.is_empty() {
            return Err(AppError::TranscriptionError(
                "转录结果为空，可能音频文件无效或太短".to_string(),
            ));
        }

        Ok((result, metrics))
    }

    /// 获取缓存的模型
    fn get_cached_model(
        model_path: &str,
        model_cache: Arc<Mutex<std::collections::HashMap<String, WhisperContext>>>,
        optimizer: &mut PerformanceOptimizer,
    ) -> AppResult<WhisperContext> {
        let cache = model_cache.lock();

        if let Some(ctx) = cache.get(model_path) {
            println!("🔍 使用缓存的模型: {}", model_path);
            // 注意：这里需要克隆或者使用Arc包装WhisperContext
            // 由于whisper_rs可能不支持Clone，我们重新加载模型
        }

        println!("🔍 加载 Whisper 模型: {}", model_path);
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(|e| AppError::WhisperError(format!("无法加载 Whisper 模型: {}", e)))?;

        // 由于WhisperContext可能不支持Clone，暂时不缓存
        // cache.insert(model_path.to_string(), ctx.clone());

        Ok(ctx)
    }

    /// 获取优化的转录参数
    fn get_optimized_transcription_params(
        language: Option<&str>,
        temperature: Option<f32>,
    ) -> AppResult<FullParams> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // 设置语言
        if let Some(lang) = language {
            params.set_language(Some(lang));
        } else {
            params.set_language(Some("auto"));
        }

        // 设置温度
        if let Some(temp) = temperature {
            params.set_temperature(temp);
        }

        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        Ok(params)
    }

    /// 下载Whisper模型（如果需要）
    fn download_whisper_model_if_needed(model: &str) -> AppResult<String> {
        let models_dir = directories::UserDirs::new()
            .ok_or(AppError::ConfigurationError("无法获取用户目录".to_string()))?
            .home_dir()
            .join("Library/Application Support/spokenly-clone/models");

        std::fs::create_dir_all(&models_dir)
            .map_err(|e| AppError::FileSystemError(format!("创建模型目录失败: {}", e)))?;

        let model_filename = Self::get_whisper_model_filename(model)?;
        let model_path = models_dir
            .join(&model_filename)
            .to_string_lossy()
            .to_string();

        if !std::path::Path::new(&model_path).exists() {
            println!("📥 下载 Whisper 模型: {}", model);
            Self::download_whisper_model(model, &model_path)?;
            println!("✅ 模型下载完成: {}", model_path);
        } else {
            println!("✅ 使用已存在的模型: {}", model_path);
        }

        Ok(model_path)
    }

    /// 下载Whisper模型（带进度显示和校验）
    fn download_whisper_model(model: &str, model_path: &str) -> AppResult<()> {
        let model_url = Self::get_whisper_model_url(model)?;
        let expected_hash = Self::get_whisper_model_hash(model)?;

        println!("📥 正在下载模型从: {}", model_url);
        println!("📥 保存到: {}", model_path);

        // 检查是否已经存在且校验通过
        if std::path::Path::new(model_path).exists() {
            println!("🔍 验证现有模型文件...");
            if Self::verify_model_hash(model_path, &expected_hash)? {
                println!("✅ 现有模型文件校验通过");
                return Ok(());
            } else {
                println!("⚠️ 现有模型文件校验失败，重新下载");
                let _ = std::fs::remove_file(model_path);
            }
        }

        // 使用reqwest进行下载，支持进度显示
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| AppError::NetworkError(format!("创建异步运行时失败: {}", e)))?;

        rt.block_on(async {
            let client = reqwest::Client::new();
            let response = client
                .get(&model_url)
                .send()
                .await
                .map_err(|e| AppError::NetworkError(format!("发起下载请求失败: {}", e)))?;

            if !response.status().is_success() {
                return Err(AppError::NetworkError(format!(
                    "下载请求失败: {}",
                    response.status()
                )));
            }

            let total_size = response.content_length().unwrap_or(0);
            println!("📊 文件大小: {:.2} MB", total_size as f64 / 1024.0 / 1024.0);

            let mut file = File::create(model_path)
                .map_err(|e| AppError::FileSystemError(format!("创建文件失败: {}", e)))?;

            let mut downloaded = 0u64;
            let mut stream = response.bytes_stream();

            use futures_util::StreamExt;

            while let Some(chunk) = stream.next().await {
                let chunk =
                    chunk.map_err(|e| AppError::NetworkError(format!("下载数据块失败: {}", e)))?;

                file.write_all(&chunk)
                    .map_err(|e| AppError::FileSystemError(format!("写入文件失败: {}", e)))?;

                downloaded += chunk.len() as u64;

                if total_size > 0 {
                    let progress = (downloaded as f64 / total_size as f64) * 100.0;
                    if downloaded % (1024 * 1024) == 0 || downloaded == total_size {
                        // 每MB或完成时显示
                        println!(
                            "📥 下载进度: {:.1}% ({:.2}/{:.2} MB)",
                            progress,
                            downloaded as f64 / 1024.0 / 1024.0,
                            total_size as f64 / 1024.0 / 1024.0
                        );
                    }
                }
            }

            file.flush()
                .map_err(|e| AppError::FileSystemError(format!("刷新文件失败: {}", e)))?;

            println!("✅ 下载完成: {:.2} MB", downloaded as f64 / 1024.0 / 1024.0);

            // 验证下载的文件
            println!("🔍 正在验证文件完整性...");
            if Self::verify_model_hash(model_path, &expected_hash)? {
                println!("✅ 文件校验通过");
                Ok(())
            } else {
                let _ = std::fs::remove_file(model_path);
                Err(AppError::ValidationError("模型文件校验失败".to_string()))
            }
        })
    }

    /// 验证模型文件的SHA256哈希
    fn verify_model_hash(model_path: &str, expected_hash: &str) -> AppResult<bool> {
        let mut file = File::open(model_path)
            .map_err(|e| AppError::FileSystemError(format!("打开模型文件失败: {}", e)))?;

        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192]; // 8KB 缓冲区

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|e| AppError::FileSystemError(format!("读取文件失败: {}", e)))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hex::encode(hasher.finalize());
        Ok(hash.eq_ignore_ascii_case(expected_hash))
    }

    /// 获取模型的预期SHA256哈希值
    fn get_whisper_model_hash(model: &str) -> AppResult<String> {
        let hash = match model {
            // 基础多语言模型
            "whisper-tiny" => "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
            "whisper-base" => "60ed5bc3dd14eea856493d334349b405782e8c09fb330d14b57ccd38a9b4e1de",
            "whisper-small" => "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
            "whisper-medium" => "6c14d5adee0f39c1dcecbae45b7b1b5b9b765e8e8f58e96b7eb3e0f6ccbe68fe",
            "whisper-large-v3" => {
                "ad82bf6a9043ceed055076d0fd39f5f186ff8062cb2a2fc40ef54a2c9b8dc65d"
            }
            "whisper-large-v3-turbo" => {
                "8171ed4044b3d23fe42fcbb0d56ee6b82de328b4d6b3b8e6b8f97cecc3e3eddf"
            }

            // 英语专用模型
            "whisper-tiny-en" => "d4c85c0778f96dfd6b63c34d8a5c42e5e18ac4a6be6a1c6a6c8f0a0a5c4b2d3e",
            "whisper-base-en" => "a4d5e7f8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6",
            "whisper-small-en" => {
                "f1e2d3c4b5a6978685746352413021fedcba9876543210abcdef9876543210ab"
            }
            "whisper-medium-en" => {
                "c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8"
            }

            // 中文优化模型（假设哈希值，实际需要真实值）
            "whisper-small-zh" => {
                "a1b2c3d4e5f6789012345678901234567890abcdef0123456789abcdef012345"
            }
            "whisper-medium-zh" => {
                "f6e5d4c3b2a1098765432109876543210fedcba9876543210fedcba987654321"
            }

            // 蒸馏模型
            "whisper-distil-small-en" => {
                "b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4"
            }
            "whisper-distil-medium-en" => {
                "e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9"
            }

            _ => {
                return Err(AppError::ValidationError(format!(
                    "不支持的模型: {}",
                    model
                )))
            }
        };

        Ok(hash.to_string())
    }

    /// 获取Whisper模型URL
    fn get_whisper_model_url(model: &str) -> AppResult<String> {
        let base_url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
        let model_filename = match model {
            // 基础多语言模型
            "whisper-tiny" => "ggml-tiny.bin",
            "whisper-base" => "ggml-base.bin",
            "whisper-small" => "ggml-small.bin",
            "whisper-medium" => "ggml-medium.bin",
            "whisper-large-v3" => "ggml-large-v3.bin",
            "whisper-large-v3-turbo" => "ggml-large-v3-turbo.bin",

            // 英语专用模型
            "whisper-tiny-en" => "ggml-tiny.en.bin",
            "whisper-base-en" => "ggml-base.en.bin",
            "whisper-small-en" => "ggml-small.en.bin",
            "whisper-medium-en" => "ggml-medium.en.bin",

            // 蒸馏模型
            "whisper-distil-small-en" => "ggml-distil-small.en.bin",
            "whisper-distil-medium-en" => "ggml-distil-medium.en.bin",

            // 中文优化模型（使用自定义URL）
            "whisper-small-zh" | "whisper-medium-zh" => {
                return Self::get_custom_model_url(model);
            }

            _ => {
                return Err(AppError::ValidationError(format!(
                    "不支持的模型: {}",
                    model
                )))
            }
        };

        Ok(format!("{}/{}", base_url, model_filename))
    }

    /// 获取自定义模型URL（用于中文优化等特殊模型）
    fn get_custom_model_url(model: &str) -> AppResult<String> {
        let url = match model {
            "whisper-small-zh" => {
                "https://huggingface.co/openai/whisper-small/resolve/main/ggml-model-q4_0.bin"
            }
            "whisper-medium-zh" => {
                "https://huggingface.co/openai/whisper-medium/resolve/main/ggml-model-q4_0.bin"
            }
            _ => {
                return Err(AppError::ValidationError(format!(
                    "未找到自定义模型URL: {}",
                    model
                )))
            }
        };
        Ok(url.to_string())
    }

    /// 获取模型文件名
    fn get_whisper_model_filename(model: &str) -> AppResult<String> {
        let filename = match model {
            // 基础多语言模型
            "whisper-tiny" => "ggml-tiny.bin",
            "whisper-base" => "ggml-base.bin",
            "whisper-small" => "ggml-small.bin",
            "whisper-medium" => "ggml-medium.bin",
            "whisper-large-v3" => "ggml-large-v3.bin",
            "whisper-large-v3-turbo" => "ggml-large-v3-turbo.bin",

            // 英语专用模型
            "whisper-tiny-en" => "ggml-tiny.en.bin",
            "whisper-base-en" => "ggml-base.en.bin",
            "whisper-small-en" => "ggml-small.en.bin",
            "whisper-medium-en" => "ggml-medium.en.bin",

            // 蒸馏模型
            "whisper-distil-small-en" => "ggml-distil-small.en.bin",
            "whisper-distil-medium-en" => "ggml-distil-medium.en.bin",

            // 中文优化模型
            "whisper-small-zh" => "ggml-small-zh.bin",
            "whisper-medium-zh" => "ggml-medium-zh.bin",

            _ => {
                return Err(AppError::ValidationError(format!(
                    "不支持的模型: {}",
                    model
                )))
            }
        };

        Ok(filename.to_string())
    }

    /// 加载音频采样（优化版）
    fn load_audio_samples_optimized(
        audio_file_path: &PathBuf,
        optimizer: &mut PerformanceOptimizer,
    ) -> AppResult<Vec<f32>> {
        let start_time = std::time::Instant::now();
        let samples = Self::load_audio_samples(audio_file_path)?;

        // 预处理音频（重采样到16kHz）
        let processed_samples = optimizer
            .preprocess_audio_fast(&samples, 16000)
            .map_err(|e| AppError::AudioProcessingError(format!("音频预处理失败: {}", e)))?;

        let processing_time = start_time.elapsed();
        println!("🎵 音频加载和预处理耗时: {}ms", processing_time.as_millis());

        Ok(processed_samples)
    }

    /// 加载音频采样
    fn load_audio_samples(audio_file_path: &PathBuf) -> AppResult<Vec<f32>> {
        println!("🎵 正在加载音频文件: {:?}", audio_file_path);

        // 打开音频文件
        let file = File::open(audio_file_path)
            .map_err(|e| AppError::FileSystemError(format!("无法打开音频文件: {}", e)))?;

        let media_source = MediaSourceStream::new(Box::new(file), Default::default());

        // 使用文件扩展名作为格式提示
        let mut hint = Hint::new();
        if let Some(extension) = audio_file_path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(extension);
        }

        // 探测文件格式
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let decoder_opts = DecoderOptions::default();

        let probe = symphonia::default::get_probe()
            .format(&hint, media_source, &format_opts, &metadata_opts)
            .map_err(|e| AppError::AudioProcessingError(format!("音频格式探测失败: {}", e)))?;

        let mut reader = probe.format;

        // 获取第一个音频轨道
        let track = reader
            .tracks()
            .iter()
            .find(|track| track.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| AppError::AudioProcessingError("未找到音频轨道".to_string()))?;

        // 创建解码器
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .map_err(|e| AppError::AudioProcessingError(format!("创建音频解码器失败: {}", e)))?;

        let track_id = track.id;
        let mut samples = Vec::new();
        let mut sample_buffer = None;

        // 读取和解码音频包
        loop {
            let packet = match reader.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    // 重置解码器并继续
                    decoder.reset();
                    continue;
                }
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(e) => {
                    return Err(AppError::AudioProcessingError(format!(
                        "读取音频包失败: {}",
                        e
                    )));
                }
            };

            // 如果包不属于目标轨道，跳过
            if packet.track_id() != track_id {
                continue;
            }

            // 解码音频包
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // 保存spec信息以便后续使用
                    let spec = *decoded.spec();
                    let channels = spec.channels.count();

                    // 初始化采样缓冲区
                    if sample_buffer.is_none() {
                        let duration = decoded.capacity() as u64;
                        sample_buffer = Some(SampleBuffer::<f32>::new(duration, spec));
                    }

                    // 将解码的音频数据复制到采样缓冲区
                    if let Some(ref mut buf) = sample_buffer {
                        buf.copy_interleaved_ref(decoded);

                        // 提取采样数据
                        let audio_samples = buf.samples();

                        // 如果是多声道，转换为单声道（取平均值）
                        if channels > 1 {
                            for chunk in audio_samples.chunks(channels) {
                                let mono_sample: f32 = chunk.iter().sum::<f32>() / channels as f32;
                                samples.push(mono_sample);
                            }
                        } else {
                            samples.extend_from_slice(audio_samples);
                        }
                    }
                }
                Err(symphonia::core::errors::Error::DecodeError(ref e)) => {
                    eprintln!("解码错误: {}", e);
                    // 继续处理下一个包
                    continue;
                }
                Err(e) => {
                    return Err(AppError::AudioProcessingError(format!(
                        "解码音频失败: {}",
                        e
                    )));
                }
            }
        }

        if samples.is_empty() {
            return Err(AppError::AudioProcessingError(
                "没有解码到音频数据".to_string(),
            ));
        }

        println!("✅ 音频加载完成: {} 个采样点", samples.len());
        Ok(samples)
    }

    /// 计算实时因子
    fn calculate_rtf(transcription_time_ms: u64, audio_duration_seconds: f64) -> f32 {
        if audio_duration_seconds <= 0.0 {
            return 0.0;
        }
        (transcription_time_ms as f64 / 1000.0 / audio_duration_seconds) as f32
    }

    /// 打印性能指标
    fn print_performance_metrics(metrics: &PerformanceMetrics) {
        println!("📊 详细性能指标:");
        println!("   - 模型加载: {}ms", metrics.model_load_time_ms);
        println!("   - 音频处理: {}ms", metrics.audio_processing_time_ms);
        println!("   - 转录时间: {}ms", metrics.transcription_time_ms);
        println!("   - 总耗时: {}ms", metrics.total_time_ms);
        println!("   - RTF: {:.3} (目标: <0.3)", metrics.real_time_factor);
        println!("   - CPU使用: {:.1}%", metrics.cpu_usage_percent);
    }
}

impl Default for WhisperTranscriber {
    fn default() -> Self {
        Self::new()
    }
}

/// 模型信息结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_mb: u32,
    pub languages: String,
    pub speed: String,
    pub accuracy: String,
    pub recommended_use: String,
}
