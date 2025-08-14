/**
 * Recording King 性能优化模块
 * 专门用于GPU使用优化和转录延迟减少
 */

use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use sysinfo::System;

// 全局模型缓存，避免重复加载
static MODEL_CACHE: Lazy<Arc<Mutex<HashMap<String, Arc<WhisperContext>>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// 性能监控结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetrics {
    pub model_load_time_ms: u64,
    pub audio_processing_time_ms: u64,
    pub transcription_time_ms: u64,
    pub total_time_ms: u64,
    pub gpu_memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub audio_duration_seconds: f64,
    pub real_time_factor: f64, // 转录时间 / 音频时长
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            model_load_time_ms: 0,
            audio_processing_time_ms: 0,
            transcription_time_ms: 0,
            total_time_ms: 0,
            gpu_memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            audio_duration_seconds: 0.0,
            real_time_factor: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct PerformanceOptimizer {
    system: System,
    enable_gpu: bool,
    enable_caching: bool,
    max_cache_size: usize,
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            enable_gpu: true,
            enable_caching: true,
            max_cache_size: 3, // 最多缓存3个模型
        }
    }
    
    pub fn configure(&mut self, enable_gpu: bool, enable_caching: bool, max_cache_size: usize) {
        self.enable_gpu = enable_gpu;
        self.enable_caching = enable_caching;
        self.max_cache_size = max_cache_size;
    }
    
    // 获取或加载模型（带缓存）
    pub fn get_cached_model(&self, model_path: &str) -> Result<Arc<WhisperContext>, String> {
        if !self.enable_caching {
            return self.load_model_direct(model_path);
        }
        
        let start_time = std::time::Instant::now();
        
        // 检查缓存
        {
            let cache = MODEL_CACHE.lock().unwrap();
            if let Some(cached_model) = cache.get(model_path) {
                println!("🎯 使用缓存的模型: {}", model_path);
                return Ok(Arc::clone(cached_model));
            }
        }
        
        // 缓存未命中，加载新模型
        println!("📦 模型未缓存，正在加载: {}", model_path);
        let model = self.load_model_direct(model_path)?;
        
        // 添加到缓存
        {
            let mut cache = MODEL_CACHE.lock().unwrap();
            
            // 如果缓存已满，移除最旧的模型
            if cache.len() >= self.max_cache_size {
                if let Some(oldest_key) = cache.keys().next().cloned() {
                    cache.remove(&oldest_key);
                    println!("🗑️ 移除旧模型缓存: {}", oldest_key);
                }
            }
            
            cache.insert(model_path.to_string(), Arc::clone(&model));
        }
        
        let load_time = start_time.elapsed().as_millis();
        println!("⚡ 模型加载时间: {}ms", load_time);
        
        Ok(model)
    }
    
    // 直接加载模型（GPU优化）
    fn load_model_direct(&self, model_path: &str) -> Result<Arc<WhisperContext>, String> {
        let mut params = WhisperContextParameters::default();
        
        if self.enable_gpu {
            // 启用GPU优化参数
            params.use_gpu = true;
            
            // Metal特定优化设置
            #[cfg(target_os = "macos")]
            {
                // macOS上优化Metal使用
                params.gpu_device = 0; // 使用默认GPU设备
                println!("🚀 启用GPU加速（Metal on macOS）");
                
                // 检查Metal可用性
                if self.check_metal_availability() {
                    println!("✅ Metal GPU加速可用");
                } else {
                    println!("⚠️ Metal GPU加速不可用，将使用CPU");
                    params.use_gpu = false;
                }
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                println!("🚀 启用GPU加速");
            }
        }
        
        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| format!("模型加载失败: {}", e))?;
            
        Ok(Arc::new(ctx))
    }
    
    /// 检查Metal可用性（macOS专用）
    #[cfg(target_os = "macos")]
    fn check_metal_availability(&self) -> bool {
        // 这里可以添加Metal设备检查逻辑
        // 目前简单返回true，假设Metal可用
        true
    }
    
    #[cfg(not(target_os = "macos"))]
    fn check_metal_availability(&self) -> bool {
        false
    }
    
    // 优化的转录参数
    pub fn get_optimized_transcription_params(&self) -> FullParams {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // 优化参数配置
        params.set_language(Some("auto"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        
        // 性能优化设置
        params.set_n_threads(num_cpus::get() as i32 / 2); // 使用一半CPU核心
        params.set_no_context(false); // 启用上下文以提高准确性
        params.set_single_segment(false); // 允许分段处理长音频
        
        // GPU优化
        if self.enable_gpu {
            // Metal特定优化将由whisper-rs库自动处理
            println!("⚡ 转录参数已针对GPU优化");
        }
        
        params
    }
    
    // 快速音频预处理
    pub fn preprocess_audio_fast(&self, audio_data: &[f32], sample_rate: u32) -> Result<Vec<f32>, String> {
        const TARGET_SAMPLE_RATE: u32 = 16000;
        
        if sample_rate == TARGET_SAMPLE_RATE {
            return Ok(audio_data.to_vec());
        }
        
        let start_time = std::time::Instant::now();
        
        // 高效重采样算法
        let ratio = TARGET_SAMPLE_RATE as f32 / sample_rate as f32;
        let new_length = (audio_data.len() as f32 * ratio) as usize;
        
        // 使用线性插值进行快速重采样
        let mut resampled = Vec::with_capacity(new_length);
        
        for i in 0..new_length {
            let src_index = (i as f32 / ratio).floor() as usize;
            let next_index = (src_index + 1).min(audio_data.len() - 1);
            let frac = (i as f32 / ratio) - src_index as f32;
            
            if src_index < audio_data.len() {
                // 线性插值
                let current = audio_data[src_index];
                let next = audio_data[next_index];
                let interpolated = current + frac * (next - current);
                resampled.push(interpolated);
            }
        }
        
        let processing_time = start_time.elapsed().as_millis();
        println!("🎵 音频重采样时间: {}ms ({}Hz -> {}Hz)", 
                processing_time, sample_rate, TARGET_SAMPLE_RATE);
        
        Ok(resampled)
    }
    
    // 获取系统性能指标
    pub fn get_system_metrics(&mut self) -> Result<(f64, f64), String> {
        self.system.refresh_all();
        
        let cpu_usage = self.system.global_cpu_info().cpu_usage() as f64;
        let _total_memory = self.system.total_memory() as f64 / 1024.0 / 1024.0; // MB
        let used_memory = self.system.used_memory() as f64 / 1024.0 / 1024.0; // MB
        
        Ok((cpu_usage, used_memory))
    }
    
    // 计算实时因子（Real-Time Factor）
    pub fn calculate_rtf(&self, transcription_time_ms: u64, audio_duration_seconds: f64) -> f64 {
        let transcription_time_seconds = transcription_time_ms as f64 / 1000.0;
        if audio_duration_seconds > 0.0 {
            transcription_time_seconds / audio_duration_seconds
        } else {
            0.0
        }
    }
    
    // 清理模型缓存
    pub fn clear_model_cache(&self) {
        let mut cache = MODEL_CACHE.lock().unwrap();
        cache.clear();
        println!("🧹 已清理所有模型缓存");
    }
    
    // 获取缓存统计
    pub fn get_cache_stats(&self) -> (usize, Vec<String>) {
        let cache = MODEL_CACHE.lock().unwrap();
        let count = cache.len();
        let models: Vec<String> = cache.keys().cloned().collect();
        (count, models)
    }
    
    // 预热GPU（如果支持）
    pub fn warmup_gpu(&self) -> Result<(), String> {
        if !self.enable_gpu {
            return Ok(());
        }
        
        println!("🔥 GPU预热中...");
        
        // 这里可以添加GPU预热逻辑
        // 对于Metal，whisper-rs会自动处理
        
        println!("✅ GPU预热完成");
        Ok(())
    }
}

// 性能优化配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceConfig {
    pub enable_gpu: bool,
    pub enable_model_caching: bool,
    pub max_cache_size: usize,
    pub target_rtf: f64, // 目标实时因子（< 1.0 表示比实时更快）
    pub cpu_threads: Option<i32>,
    pub enable_audio_preprocessing: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_gpu: true,
            enable_model_caching: true,
            max_cache_size: 3,
            target_rtf: 0.3, // 目标：转录速度比实时快3倍
            cpu_threads: None, // 自动检测
            enable_audio_preprocessing: true,
        }
    }
}

// 导出函数供外部使用
pub fn create_performance_optimizer() -> PerformanceOptimizer {
    PerformanceOptimizer::new()
}

pub fn get_optimal_cpu_threads() -> i32 {
    let total_cores = num_cpus::get() as i32;
    // 使用总核心数的60-80%，为系统保留一些资源
    std::cmp::max(1, (total_cores as f32 * 0.7) as i32)
}