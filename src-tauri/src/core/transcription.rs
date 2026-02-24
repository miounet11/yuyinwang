use crate::core::{error::Result, types::*};
use reqwest::multipart;
use rubato::{SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction, Resampler};
use std::path::Path;

const LUYIN_BASE_URL: &str = "https://ly.gl173.com";
const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const POLL_INTERVAL_MS: u64 = 3000;
const MAX_POLL_COUNT: u32 = 60;

pub struct TranscriptionService {
    client: reqwest::Client,
    settings: AppSettings,
    app_data_dir: Option<std::path::PathBuf>,
}

impl TranscriptionService {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            client: reqwest::Client::new(),
            settings,
            app_data_dir: None,
        }
    }

    pub fn with_app_data_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.app_data_dir = Some(dir);
        self
    }

    /// 根据 selected_model 路由到对应后端
    pub async fn transcribe_audio(&self, audio_path: &Path) -> Result<TranscriptionResult> {
        let provider = ModelProvider::from_model_id(&self.settings.selected_model);

        match provider {
            ModelProvider::LuYinWang => self.transcribe_luyin(audio_path).await,
            ModelProvider::OpenAI => self.transcribe_openai(audio_path).await,
            ModelProvider::Deepgram => self.transcribe_openai_compat(audio_path, "deepgram").await,
            ModelProvider::Mistral => self.transcribe_openai_compat(audio_path, "mistral").await,
            ModelProvider::ElevenLabs => self.transcribe_openai_compat(audio_path, "elevenlabs").await,
            ModelProvider::LocalWhisper => self.transcribe_local_whisper(audio_path).await,
        }
    }

    pub async fn transcribe_samples(
        &self,
        samples: &[f32],
        sample_rate: u32,
    ) -> Result<TranscriptionResult> {
        let provider = ModelProvider::from_model_id(&self.settings.selected_model);

        // 本地模型：直接传 f32 samples，跳过 WAV 中间步骤
        if provider == ModelProvider::LocalWhisper {
            let app_data_dir = self.app_data_dir.as_ref().ok_or_else(|| {
                crate::core::error::AppError::Transcription(
                    "应用数据目录未设置，无法使用本地模型".into(),
                )
            })?;

            let model_id = &self.settings.selected_model;
            if !crate::core::local_whisper::is_model_downloaded(app_data_dir, model_id) {
                return Err(crate::core::error::AppError::Transcription(format!(
                    "模型 {} 尚未下载，请先在「听写模型」页面下载",
                    model_id
                )));
            }

            // 重采样到 16kHz（如果需要）
            let resampled = Self::ensure_16khz(samples, sample_rate)?;

            let models_dir = crate::core::local_whisper::get_models_dir(app_data_dir);
            let file_name = Self::get_model_filename(model_id);
            let model_path = models_dir.join(file_name);

            return tokio::task::spawn_blocking(move || {
                crate::core::local_whisper::transcribe_local(&model_path, &resampled, None)
            })
            .await
            .map_err(|e| {
                crate::core::error::AppError::Transcription(format!("推理线程异常: {}", e))
            })?;
        }

        // 在线模型：需要写 WAV 文件上传
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("recording_{}.wav", chrono::Utc::now().timestamp()));

        crate::core::audio::save_audio_to_wav(samples, sample_rate, temp_path.to_str().unwrap())?;
        let result = self.transcribe_audio(&temp_path).await;
        let _ = tokio::fs::remove_file(&temp_path).await;

        result
    }

    // ================================================================
    // LuYinWang API (3-step: upload → create task → poll)
    // ================================================================

    fn get_luyin_token(&self) -> Result<&str> {
        self.settings
            .luyin_token
            .as_deref()
            .filter(|t| !t.is_empty())
            .ok_or_else(|| {
                crate::core::error::AppError::Transcription(
                    "录音王 Token 未配置，请在「听写模型」页面配置".into(),
                )
            })
    }

    async fn transcribe_luyin(&self, audio_path: &Path) -> Result<TranscriptionResult> {
        let token = self.get_luyin_token()?;

        let file_id = self.luyin_upload(audio_path, token).await?;
        let task_id = self.luyin_create_task(file_id, token).await?;
        let text = self.luyin_poll(task_id, token).await?;

        Ok(TranscriptionResult {
            text,
            language: Some("zh".to_string()),
            duration: None,
        })
    }

    async fn luyin_upload(&self, audio_path: &Path, token: &str) -> Result<i64> {
        let file = tokio::fs::read(audio_path).await?;
        let file_name = audio_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav")
            .to_string();

        let part = multipart::Part::bytes(file)
            .file_name(file_name)
            .mime_str("audio/wav")?;

        let form = multipart::Form::new().part("file[]", part);

        let response = self
            .client
            .post(format!("{}/api/v1/upload-file", LUYIN_BASE_URL))
            .bearer_auth(token)
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        if status.as_u16() == 401 {
            return Err(crate::core::error::AppError::Transcription(
                "Token 已过期或无效，请更新录音王 Token".into(),
            ));
        }
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(crate::core::error::AppError::Transcription(format!(
                "上传失败 ({}): {}",
                status, error_text
            )));
        }

        let body: serde_json::Value = response.json().await?;
        if body["code"].as_i64() != Some(200) {
            return Err(crate::core::error::AppError::Transcription(format!(
                "上传错误: {}",
                body["message"].as_str().unwrap_or("unknown")
            )));
        }

        body["data"][0]["file_id"]
            .as_i64()
            .ok_or_else(|| crate::core::error::AppError::Transcription("响应中无 file_id".into()))
    }

    async fn luyin_create_task(&self, file_id: i64, token: &str) -> Result<String> {
        let response = self
            .client
            .post(format!("{}/api/v1/task-add", LUYIN_BASE_URL))
            .bearer_auth(token)
            .form(&[("file_id", file_id.to_string())])
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;
        if body["code"].as_i64() != Some(200) {
            return Err(crate::core::error::AppError::Transcription(format!(
                "创建任务失败: {}",
                body["message"].as_str().unwrap_or("unknown")
            )));
        }

        body["data"]["task_id"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| crate::core::error::AppError::Transcription("响应中无 task_id".into()))
    }

    async fn luyin_poll(&self, task_id: String, token: &str) -> Result<String> {
        for _ in 0..MAX_POLL_COUNT {
            let response = self
                .client
                .post(format!("{}/api/v1/task-progress", LUYIN_BASE_URL))
                .bearer_auth(token)
                .form(&[("task_id", &task_id)])
                .send()
                .await?;

            let body: serde_json::Value = response.json().await?;
            if body["code"].as_i64() != Some(200) {
                return Err(crate::core::error::AppError::Transcription(format!(
                    "查询进度失败: {}",
                    body["message"].as_str().unwrap_or("unknown")
                )));
            }

            let progress = body["data"]["progress"].as_i64().unwrap_or(0);
            if progress == 1 {
                return body["data"]["result"]
                    .as_str()
                    .map(String::from)
                    .ok_or_else(|| {
                        crate::core::error::AppError::Transcription("响应中无转录结果".into())
                    });
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
        }

        Err(crate::core::error::AppError::Transcription(
            "转录超时（3分钟）".into(),
        ))
    }

    // ================================================================
    // OpenAI Whisper API
    // ================================================================

    fn get_openai_key(&self) -> Result<&str> {
        self.settings
            .openai_api_key
            .as_deref()
            .filter(|k| !k.is_empty())
            .ok_or_else(|| {
                crate::core::error::AppError::Transcription(
                    "OpenAI API Key 未配置，请在「听写模型」页面配置".into(),
                )
            })
    }

    async fn transcribe_openai(&self, audio_path: &Path) -> Result<TranscriptionResult> {
        let api_key = self.get_openai_key()?;

        let file = tokio::fs::read(audio_path).await?;
        let file_name = audio_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav")
            .to_string();

        let part = multipart::Part::bytes(file)
            .file_name(file_name)
            .mime_str("audio/wav")?;

        let model = &self.settings.selected_model;
        let form = multipart::Form::new()
            .part("file", part)
            .text("model", model.clone());

        let response = self
            .client
            .post(format!("{}/audio/transcriptions", OPENAI_BASE_URL))
            .bearer_auth(api_key)
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        if status.as_u16() == 401 {
            return Err(crate::core::error::AppError::Transcription(
                "OpenAI API Key 无效".into(),
            ));
        }
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(crate::core::error::AppError::Transcription(format!(
                "OpenAI 转录失败 ({}): {}",
                status, error_text
            )));
        }

        let body: serde_json::Value = response.json().await?;
        let text = body["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(TranscriptionResult {
            text,
            language: body["language"].as_str().map(String::from),
            duration: body["duration"].as_f64(),
        })
    }

    // ================================================================
    // 其他 OpenAI 兼容 API（Deepgram/Mistral/ElevenLabs 占位）
    // ================================================================

    async fn transcribe_openai_compat(
        &self,
        _audio_path: &Path,
        provider: &str,
    ) -> Result<TranscriptionResult> {
        Err(crate::core::error::AppError::Transcription(format!(
            "{} 转录接口尚未集成，请使用 LuYinWang 或 OpenAI",
            provider
        )))
    }

    // ================================================================
    // Local Whisper (whisper.cpp via whisper-rs)
    // ================================================================

    async fn transcribe_local_whisper(&self, audio_path: &Path) -> Result<TranscriptionResult> {
        let app_data_dir = self.app_data_dir.as_ref().ok_or_else(|| {
            crate::core::error::AppError::Transcription(
                "应用数据目录未设置，无法使用本地模型".into(),
            )
        })?;

        let model_id = &self.settings.selected_model;
        if !crate::core::local_whisper::is_model_downloaded(app_data_dir, model_id) {
            return Err(crate::core::error::AppError::Transcription(format!(
                "模型 {} 尚未下载，请先在「听写模型」页面下载",
                model_id
            )));
        }

        // 读取音频文件并获取采样率
        let file_bytes = tokio::fs::read(audio_path).await?;
        let (samples, source_sample_rate) = Self::decode_audio_to_f32_with_rate(&file_bytes)?;

        // 重采样到 16kHz（如果需要）
        let samples = Self::ensure_16khz(&samples, source_sample_rate)?;

        // 本地推理（在阻塞线程中运行，避免阻塞 tokio）
        let models_dir = crate::core::local_whisper::get_models_dir(app_data_dir);
        let file_name = Self::get_model_filename(model_id);
        let model_path = models_dir.join(file_name);

        tokio::task::spawn_blocking(move || {
            crate::core::local_whisper::transcribe_local(&model_path, &samples, None)
        })
        .await
        .map_err(|e| {
            crate::core::error::AppError::Transcription(format!("推理线程异常: {}", e))
        })?
    }

    fn get_model_filename(model_id: &str) -> &'static str {
        match model_id {
            "whisper-tiny" => "ggml-tiny.bin",
            "whisper-base" => "ggml-base.bin",
            "whisper-small" => "ggml-small.bin",
            "whisper-medium" => "ggml-medium.bin",
            "whisper-large-v3" => "ggml-large-v3.bin",
            "whisper-large-v3-turbo" => "ggml-large-v3-turbo.bin",
            _ => "ggml-small.bin",
        }
    }

    /// 将 WAV 文件字节解码为 f32 samples (mono)，同时返回原始采样率
    fn decode_audio_to_f32_with_rate(wav_bytes: &[u8]) -> Result<(Vec<f32>, u32)> {
        let cursor = std::io::Cursor::new(wav_bytes);
        let reader = hound::WavReader::new(cursor).map_err(|e| {
            crate::core::error::AppError::Transcription(format!("WAV 解码失败: {}", e))
        })?;

        let spec = reader.spec();
        let source_rate = spec.sample_rate;

        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => {
                let max_val = (1 << (spec.bits_per_sample - 1)) as f32;
                reader
                    .into_samples::<i32>()
                    .filter_map(|s| s.ok())
                    .map(|s| s as f32 / max_val)
                    .collect()
            }
            hound::SampleFormat::Float => {
                reader
                    .into_samples::<f32>()
                    .filter_map(|s| s.ok())
                    .collect()
            }
        };

        // 如果是多声道，取第一个声道
        let channels = spec.channels as usize;
        let mono = if channels > 1 {
            samples.iter().step_by(channels).copied().collect()
        } else {
            samples
        };

        Ok((mono, source_rate))
    }

    /// 确保 samples 是 16kHz，如果不是则重采样
    fn ensure_16khz(samples: &[f32], source_rate: u32) -> Result<Vec<f32>> {
        const TARGET_RATE: u32 = 16000;
        if source_rate == TARGET_RATE {
            return Ok(samples.to_vec());
        }

        Self::resample(samples, source_rate, TARGET_RATE)
    }

    /// 使用 rubato 进行高质量重采样
    fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }

        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let ratio = to_rate as f64 / from_rate as f64;
        let chunk_size = 1024;

        let mut resampler = SincFixedIn::<f64>::new(
            ratio,
            2.0,
            params,
            chunk_size,
            1, // mono
        )
        .map_err(|e| {
            crate::core::error::AppError::Transcription(format!("创建重采样器失败: {}", e))
        })?;

        let mut output: Vec<f32> = Vec::with_capacity((samples.len() as f64 * ratio) as usize + 1024);

        // 分块处理
        let mut pos = 0;
        while pos < samples.len() {
            let end = (pos + chunk_size).min(samples.len());
            let mut chunk: Vec<f64> = samples[pos..end].iter().map(|&s| s as f64).collect();

            // 最后一块需要填充到 chunk_size
            if chunk.len() < chunk_size {
                chunk.resize(chunk_size, 0.0);
            }

            let input = vec![chunk];
            let result = resampler.process(&input, None).map_err(|e| {
                crate::core::error::AppError::Transcription(format!("重采样失败: {}", e))
            })?;

            if !result.is_empty() {
                output.extend(result[0].iter().map(|&s| s as f32));
            }

            pos += chunk_size;
        }

        // 裁剪到预期长度
        let expected_len = (samples.len() as f64 * ratio).round() as usize;
        output.truncate(expected_len);

        println!("🔄 重采样: {}Hz → {}Hz ({} → {} samples)", from_rate, to_rate, samples.len(), output.len());
        Ok(output)
    }
}
