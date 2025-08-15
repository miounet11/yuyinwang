use reqwest::{Client, multipart::{Form, Part}};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::path::Path;
use crate::errors::{AppError, AppResult};
use crate::types::{TranscriptionResult, TranscriptionConfig};

#[derive(Debug)]
pub struct TranscriptionApiClient {
    client: Client,
}

impl TranscriptionApiClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// 使用OpenAI兼容API进行转录
    pub async fn transcribe_with_openai_api<P: AsRef<Path>>(
        &self,
        audio_file_path: P,
        api_key: &str,
        config: &TranscriptionConfig,
    ) -> AppResult<TranscriptionResult> {
        println!("🔍 使用OpenAI API进行转录...");
        
        // 读取音频文件
        let mut file = File::open(&audio_file_path).await
            .map_err(|e| AppError::FileSystemError(format!("无法打开音频文件: {}", e)))?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await
            .map_err(|e| AppError::FileSystemError(format!("无法读取音频文件: {}", e)))?;
        
        // 创建 multipart form 数据
        let filename = audio_file_path.as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav");
            
        let part = Part::bytes(buffer)
            .file_name(filename.to_string())
            .mime_str("audio/wav")
            .map_err(|e| AppError::HttpRequestError(format!("创建请求部件失败: {}", e)))?;
        
        println!("🔍 调试信息: 发送到API的模型参数 = '{}'", config.model_name);
        
        let mut form = Form::new()
            .part("file", part)
            .text("model", config.model_name.clone())
            .text("response_format", "verbose_json");
        
        // 设置语言参数
        if let Some(language) = &config.language {
            form = form.text("language", language.clone());
        }
        
        // 设置温度参数
        if let Some(temperature) = config.temperature {
            form = form.text("temperature", temperature.to_string());
        }
        
        // 获取API端点
        let default_endpoint = "https://api.openai.com/v1/audio/transcriptions".to_string();
        let api_endpoint = config.api_endpoint
            .as_ref()
            .unwrap_or(&default_endpoint);
        
        // 发送请求
        let response = self.client
            .post(api_endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::HttpRequestError(format!("发送请求失败: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
            return Err(AppError::ApiTranscriptionError(format!("API请求失败: {}", error_text)));
        }
        
        // 解析响应
        let response_text = response.text().await
            .map_err(|e| AppError::HttpRequestError(format!("读取响应失败: {}", e)))?;
        
        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AppError::DataSerializationError(format!("解析JSON响应失败: {}", e)))?;
        
        let text = json["text"].as_str()
            .ok_or_else(|| AppError::ApiTranscriptionError("响应中没有text字段".to_string()))?
            .to_string();
        
        println!("✅ OpenAI API转录成功: {}", text);
        Ok(TranscriptionResult { 
            text, 
            confidence: None, 
            duration: None, 
            language: None 
        })
    }

    /// 使用自定义录音API进行转录
    pub async fn transcribe_with_luyin_api<P: AsRef<Path>>(
        &self,
        audio_file_path: P,
    ) -> AppResult<TranscriptionResult> {
        // 获取 Bearer Token
        let bearer_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJodHRwczovL3JlY29yZC10by10ZXh0LmNvbS9hcGkvdjEvbG9nb3V0IiwiaWF0IjoxNzUzODU4NzIxLCJleHAiOjE3NjI0OTg3MjEsIm5iZiI6MTc1Mzg1ODcyMSwianRpIjoiNTlZQjBUMExqWGV4NGZqdiIsInN1YiI6IjEiLCJwcnYiOiIyM2JkNWM4OTQ5ZjYwMGFkYjM5ZTcwMWM0MDA4NzJkYjdhNTk3NmY3IiwiZGV2aWNlX2lkIjoiYmYyZTdkODU4NWU0YmM3YTFjY2VmNWE0YzI2OTkxZDQiLCJpc19sb2dpbiI6MH0.NxgG2hysvK7we4QuyNwpNoX5etfvHTW4ZqL8s1T-5oc";
        println!("🔍 使用录音API进行转录...");
        
        // 读取音频文件
        let mut file = File::open(&audio_file_path).await
            .map_err(|e| AppError::FileSystemError(format!("无法打开音频文件: {}", e)))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await
            .map_err(|e| AppError::FileSystemError(format!("无法读取音频文件: {}", e)))?;

        // 1) 上传文件，获取 file_id
        let file_name = audio_file_path.as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("recording.wav");
            
        let part = Part::bytes(buffer)
            .file_name(file_name.to_string())
            .mime_str("audio/wav")
            .map_err(|e| AppError::HttpRequestError(format!("创建上传部件失败: {}", e)))?;

        let form = Form::new().part("file[]", part);
        let upload_resp = self.client
            .post("https://ly.gl173.com/api/v1/upload-file")
            .header("Authorization", format!("Bearer {}", bearer_token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::HttpRequestError(format!("上传文件失败: {}", e)))?;

        let status = upload_resp.status();
        let upload_text = upload_resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(AppError::ApiTranscriptionError(format!("上传接口错误({}): {}", status, upload_text)));
        }
        
        let upload_json: serde_json::Value = serde_json::from_str(&upload_text)
            .map_err(|e| AppError::DataSerializationError(format!("解析上传响应失败: {} - {}", e, upload_text)))?;
        
        if upload_json["code"].as_i64().unwrap_or(0) != 200 {
            return Err(AppError::ApiTranscriptionError(format!("上传返回非200: {}", upload_text)));
        }
        
        let file_id_val = upload_json["data"][0]["file_id"].clone();
        let file_id = if let Some(id) = file_id_val.as_i64() { 
            id.to_string() 
        } else { 
            file_id_val.to_string() 
        };
        
        if file_id.is_empty() || file_id == "null" {
            return Err(AppError::ApiTranscriptionError(format!("无法获取file_id: {}", upload_text)));
        }

        // 2) 创建转换任务，得到 task_id
        let task_resp = self.client
            .post("https://ly.gl173.com/api/v1/task-add")
            .header("Authorization", format!("Bearer {}", bearer_token))
            .form(&[("file_id", file_id.clone())])
            .send()
            .await
            .map_err(|e| AppError::HttpRequestError(format!("创建任务失败: {}", e)))?;

        let task_text = task_resp.text().await.unwrap_or_default();
        let task_json: serde_json::Value = serde_json::from_str(&task_text)
            .map_err(|e| AppError::DataSerializationError(format!("解析任务响应失败: {}", e)))?;
        
        if task_json["code"].as_i64().unwrap_or(0) != 200 {
            return Err(AppError::ApiTranscriptionError(format!("创建任务返回非200: {}", task_text)));
        }
        
        let task_id = task_json["data"]["task_id"].as_str()
            .ok_or_else(|| AppError::ApiTranscriptionError("无法获取task_id".to_string()))?;

        // 3) 轮询任务状态直到完成
        println!("⏳ 等待转录完成，任务ID: {}", task_id);
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 60; // 最大等待5分钟
        
        loop {
            if attempts >= MAX_ATTEMPTS {
                return Err(AppError::ApiTranscriptionError("转录超时".to_string()));
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            attempts += 1;
            
            let status_resp = self.client
                .post("https://ly.gl173.com/api/v1/task-progress")
                .header("Authorization", format!("Bearer {}", bearer_token))
                .form(&[("task_id", task_id.to_string())])
                .send()
                .await
                .map_err(|e| AppError::HttpRequestError(format!("查询任务状态失败: {}", e)))?;

            let status_text = status_resp.text().await.unwrap_or_default();
            let status_json: serde_json::Value = serde_json::from_str(&status_text)
                .map_err(|e| AppError::DataSerializationError(format!("解析状态响应失败: {}", e)))?;
            
            if status_json["code"].as_i64().unwrap_or(0) != 200 {
                return Err(AppError::ApiTranscriptionError(format!("查询任务状态返回非200: {}", status_text)));
            }
            
            let progress = status_json["data"]["progress"].as_i64().unwrap_or(0);
            println!("📊 任务进度: {} (尝试 {}/{})", progress, attempts, MAX_ATTEMPTS);
            
            if progress == 1 {
                // 转录完成，尝试多种可能的结果字段
                println!("🔍 完整状态响应: {}", status_text);
                
                let result_text = status_json["data"]["result"].as_str()
                    .or_else(|| status_json["data"]["content"].as_str())
                    .or_else(|| status_json["data"]["text"].as_str())
                    .or_else(|| status_json["result"].as_str())
                    .or_else(|| status_json["content"].as_str())
                    .or_else(|| status_json["text"].as_str())
                    .unwrap_or("")
                    .to_string();
                
                if result_text.is_empty() {
                    // 如果所有字段都为空，打印完整响应用于调试
                    println!("❌ 转录结果为空，完整响应数据: {:#}", status_json);
                    return Err(AppError::ApiTranscriptionError(format!("转录结果为空，响应: {}", status_text)));
                }
                
                println!("✅ 录音王API转录成功: {}", result_text);
                return Ok(TranscriptionResult { 
                    text: result_text, 
                    confidence: None, 
                    duration: None, 
                    language: None 
                });
            } else if progress == 0 {
                // 仍在转换中，继续等待
                continue;
            } else {
                return Err(AppError::ApiTranscriptionError(format!("未知进度值: {}", progress)));
            }
        }
    }

    /// 通用转录方法
    pub async fn transcribe<P: AsRef<Path>>(
        &self,
        audio_file_path: P,
        config: &TranscriptionConfig,
        api_key: Option<&str>,
    ) -> AppResult<TranscriptionResult> {
        if config.is_local {
            return Err(AppError::ValidationError("API客户端不支持本地转录".to_string()));
        }
        
        match config.model_name.as_str() {
            "luyin-api" | "luyingwang-online" => {
                self.transcribe_with_luyin_api(audio_file_path).await
            },
            _ => {
                let api_key = api_key.ok_or_else(|| {
                    AppError::ConfigurationError("OpenAI API转录需要API密钥".to_string())
                })?;
                self.transcribe_with_openai_api(audio_file_path, api_key, config).await
            }
        }
    }

    /// 测试API连接
    pub async fn test_api_connection(&self, api_endpoint: &str, api_key: &str) -> AppResult<bool> {
        let response = self.client
            .get(&format!("{}/models", api_endpoint.trim_end_matches("/audio/transcriptions")))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| AppError::HttpRequestError(format!("测试API连接失败: {}", e)))?;
        
        Ok(response.status().is_success())
    }
}