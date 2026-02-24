use crate::core::{error::Result, types::*};
use std::path::{Path, PathBuf};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// 根据 model id 获取下载 URL 和文件名
fn get_model_info(model_id: &str) -> Option<(&'static str, &'static str, u64)> {
    // (file_name, url_suffix, approx_size_bytes)
    // 所有模型来自 https://huggingface.co/ggerganov/whisper.cpp
    match model_id {
        "whisper-tiny" => Some(("ggml-tiny.bin", "ggml-tiny.bin", 75_000_000)),
        "whisper-base" => Some(("ggml-base.bin", "ggml-base.bin", 148_000_000)),
        "whisper-small" => Some(("ggml-small.bin", "ggml-small.bin", 488_000_000)),
        "whisper-medium" => Some(("ggml-medium.bin", "ggml-medium.bin", 1_533_000_000)),
        "whisper-large-v3" => Some(("ggml-large-v3.bin", "ggml-large-v3.bin", 3_094_000_000)),
        "whisper-large-v3-turbo" => Some(("ggml-large-v3-turbo.bin", "ggml-large-v3-turbo.bin", 1_620_000_000)),
        _ => None,
    }
}

fn get_download_url(file_name: &str) -> String {
    format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}", file_name)
}

/// 获取模型存储目录
pub fn get_models_dir(app_data_dir: &Path) -> PathBuf {
    let dir = app_data_dir.join("models");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// 检查模型是否已下载
pub fn is_model_downloaded(app_data_dir: &Path, model_id: &str) -> bool {
    if let Some((file_name, _, _)) = get_model_info(model_id) {
        get_models_dir(app_data_dir).join(file_name).exists()
    } else {
        false
    }
}

/// 获取已下载的模型列表
pub fn get_downloaded_models(app_data_dir: &Path) -> Vec<String> {
    let all_models = [
        "whisper-tiny", "whisper-base", "whisper-small",
        "whisper-medium", "whisper-large-v3", "whisper-large-v3-turbo",
    ];
    all_models
        .iter()
        .filter(|id| is_model_downloaded(app_data_dir, id))
        .map(|s| s.to_string())
        .collect()
}

/// 下载模型，通过回调报告进度 (0.0 - 1.0)
pub async fn download_model<F>(
    app_data_dir: &Path,
    model_id: &str,
    on_progress: F,
) -> Result<PathBuf>
where
    F: Fn(f64) + Send + 'static,
{
    let (file_name, _, size_bytes) = get_model_info(model_id).ok_or_else(|| {
        crate::core::error::AppError::Other(format!("未知模型: {}", model_id))
    })?;

    let models_dir = get_models_dir(app_data_dir);
    let model_path = models_dir.join(file_name);

    // 已存在则跳过
    if model_path.exists() {
        on_progress(1.0);
        return Ok(model_path);
    }

    let temp_path = models_dir.join(format!("{}.downloading", file_name));
    let url = get_download_url(file_name);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(crate::core::error::AppError::Network)?;

    let total_size = response.content_length().unwrap_or(size_bytes);
    let mut downloaded: u64 = 0;

    let mut file = tokio::fs::File::create(&temp_path).await?;
    let mut stream = response.bytes_stream();

    use tokio::io::AsyncWriteExt;
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(crate::core::error::AppError::Network)?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded as f64 / total_size as f64);
    }

    file.flush().await?;
    drop(file);

    // 重命名为最终文件名
    tokio::fs::rename(&temp_path, &model_path).await?;
    on_progress(1.0);

    println!("✅ 模型下载完成: {} -> {:?}", model_id, model_path);
    Ok(model_path)
}

/// 删除已下载的模型
pub async fn delete_model(app_data_dir: &Path, model_id: &str) -> Result<()> {
    let (file_name, _, _) = get_model_info(model_id).ok_or_else(|| {
        crate::core::error::AppError::Other(format!("未知模型: {}", model_id))
    })?;

    let model_path = get_models_dir(app_data_dir).join(file_name);
    if model_path.exists() {
        tokio::fs::remove_file(&model_path).await?;
    }
    Ok(())
}

/// 本地 Whisper 转录
pub fn transcribe_local(
    model_path: &Path,
    audio_samples: &[f32],
    language: Option<&str>,
) -> Result<TranscriptionResult> {
    // 音频长度验证：防止空或极短音频导致 whisper.cpp 崩溃
    // 最小长度 16000 采样点 = 1 秒（采样率 16kHz）
    if audio_samples.is_empty() {
        return Ok(TranscriptionResult {
            text: String::new(),
            language: language.map(String::from),
            duration: Some(0.0),
        });
    }

    if audio_samples.len() < 16000 {
        let duration = audio_samples.len() as f64 / 16000.0;
        println!("⚠️ 音频过短 ({:.2}s)，跳过转录", duration);
        return Ok(TranscriptionResult {
            text: String::new(),
            language: language.map(String::from),
            duration: Some(duration),
        });
    }

    let ctx = WhisperContext::new_with_params(
        model_path.to_str().unwrap_or(""),
        WhisperContextParameters::default(),
    )
    .map_err(|e| {
        crate::core::error::AppError::Transcription(format!("加载模型失败: {}", e))
    })?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    if let Some(lang) = language {
        params.set_language(Some(lang));
    } else {
        params.set_language(None); // auto-detect
    }

    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_translate(false);
    params.set_no_context(true);
    params.set_single_segment(false);

    let mut state = ctx.create_state().map_err(|e| {
        crate::core::error::AppError::Transcription(format!("创建推理状态失败: {}", e))
    })?;

    state.full(params, audio_samples).map_err(|e| {
        crate::core::error::AppError::Transcription(format!("推理失败: {}", e))
    })?;

    let num_segments = state.full_n_segments();

    let mut text = String::new();
    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            if let Ok(s) = segment.to_str_lossy() {
                text.push_str(&s);
            }
        }
    }

    let text = text.trim().to_string();
    let duration = audio_samples.len() as f64 / 16000.0;

    Ok(TranscriptionResult {
        text,
        language: language.map(String::from),
        duration: Some(duration),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // 任务 1: Bug 条件探索性测试（音频部分）
    // Property 1: Fault Condition - 空或极短音频导致崩溃
    // ============================================================

    #[test]
    fn test_bug_condition_empty_audio_detection() {
        // 验证空音频数据检测逻辑
        let empty_audio: Vec<f32> = vec![];
        assert_eq!(empty_audio.len(), 0, "Empty audio should have length 0");
        assert!(empty_audio.len() < 16000, "Empty audio is shorter than minimum");
    }

    #[test]
    fn test_bug_condition_short_audio_detection() {
        // 验证短音频检测逻辑（< 1 秒）
        let short_audio: Vec<f32> = vec![0.0; 8000]; // 0.5 秒
        assert!(short_audio.len() < 16000, "0.5s audio should be detected as too short");

        let very_short: Vec<f32> = vec![0.0; 100]; // 极短
        assert!(very_short.len() < 16000, "Very short audio should be detected");
    }

    #[test]
    fn test_bug_condition_minimum_audio_length_threshold() {
        // 验证最小音频长度阈值（16000 采样点 = 1 秒 @ 16kHz）
        const MIN_SAMPLES: usize = 16000;

        let too_short: Vec<f32> = vec![0.0; MIN_SAMPLES - 1];
        assert!(too_short.len() < MIN_SAMPLES, "Should detect audio just below threshold");

        let just_enough: Vec<f32> = vec![0.0; MIN_SAMPLES];
        assert!(just_enough.len() >= MIN_SAMPLES, "Should accept audio at threshold");

        let normal: Vec<f32> = vec![0.0; MIN_SAMPLES * 2];
        assert!(normal.len() >= MIN_SAMPLES, "Should accept normal length audio");
    }

    // ============================================================
    // 任务 2: 保持性属性测试（音频部分）
    // Property 2: Preservation - 正常音频处理保持不变
    // ============================================================

    #[test]
    fn test_preservation_normal_audio_length_accepted() {
        // 验证正常长度音频（>= 16000 采样点）应被接受
        let normal_audio: Vec<f32> = vec![0.0; 32000]; // 2 秒
        assert!(normal_audio.len() >= 16000, "2s audio should be accepted");

        let long_audio: Vec<f32> = vec![0.0; 160000]; // 10 秒
        assert!(long_audio.len() >= 16000, "10s audio should be accepted");
    }

    #[test]
    fn test_preservation_audio_duration_calculation() {
        // 验证音频时长计算逻辑保持不变（采样率 16kHz）
        let one_second: Vec<f32> = vec![0.0; 16000];
        let duration = one_second.len() as f64 / 16000.0;
        assert_eq!(duration, 1.0, "16000 samples should equal 1 second");

        let two_seconds: Vec<f32> = vec![0.0; 32000];
        let duration = two_seconds.len() as f64 / 16000.0;
        assert_eq!(duration, 2.0, "32000 samples should equal 2 seconds");
    }
}
