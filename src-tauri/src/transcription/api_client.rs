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

    /// 使用自定义录音API进行转录（带重试机制）
    pub async fn transcribe_with_luyin_api<P: AsRef<Path>>(
        &self,
        audio_file_path: P,
    ) -> AppResult<TranscriptionResult> {
        const MAX_RETRIES: u32 = 3;
        let mut retry_count = 0;
        
        loop {
            match self.transcribe_with_luyin_api_internal(&audio_file_path).await {
                Ok(result) => {
                    // 如果结果不为空或者已经重试多次，返回结果
                    if !result.text.is_empty() || retry_count >= MAX_RETRIES {
                        if result.text.is_empty() && retry_count >= MAX_RETRIES {
                            println!("⚠️ 转录多次重试后仍然失败，返回空结果");
                        }
                        return Ok(result);
                    }
                    
                    // 结果为空但还有重试机会
                    retry_count += 1;
                    println!("⚠️ 转录结果为空，第 {}/{} 次重试...", retry_count, MAX_RETRIES);
                    
                    // 等待一段时间再重试
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
                Err(e) => {
                    // 如果是最后一次重试，返回空结果而不是错误
                    if retry_count >= MAX_RETRIES - 1 {
                        println!("❌ 转录失败 {} 次后放弃: {}", MAX_RETRIES, e);
                        return Ok(TranscriptionResult {
                            text: "".to_string(),
                            confidence: None,
                            duration: None,
                            language: None,
                        });
                    }
                    
                    retry_count += 1;
                    println!("❌ 转录失败: {}，第 {}/{} 次重试...", e, retry_count, MAX_RETRIES);
                    
                    // 等待更长时间再重试
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
            }
        }
    }
    
    /// 内部转录方法（实际执行）
    async fn transcribe_with_luyin_api_internal<P: AsRef<Path>>(
        &self,
        audio_file_path: P,
    ) -> AppResult<TranscriptionResult> {
        // 获取 Bearer Token（优先从环境变量，否则使用默认值）
        let bearer_token = std::env::var("LUYIN_API_TOKEN").unwrap_or_else(|_| {
            // 临时使用硬编码token，建议配置环境变量
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJodHRwczovL3JlY29yZC10by10ZXh0LmNvbS9hcGkvdjEvbG9nb3V0IiwiaWF0IjoxNzUzODU4NzIxLCJleHAiOjE3NjI0OTg3MjEsIm5iZiI6MTc1Mzg1ODcyMSwianRpIjoiNTlZQjBUMExqWGV4NGZqdiIsInN1YiI6IjEiLCJwcnYiOiIyM2JkNWM4OTQ5ZjYwMGFkYjM5ZTcwMWM0MDA4NzJkYjdhNTk3NmY3IiwiZGV2aWNlX2lkIjoiYmYyZTdkODU4NWU0YmM3YTFjY2VmNWE0YzI2OTkxZDQiLCJpc19sb2dpbiI6MH0.NxgG2hysvK7we4QuyNwpNoX5etfvHTW4ZqL8s1T-5oc".to_string()
        });
        println!("🔍 使用录音API进行转录...");
        
        // 读取音频文件
        let mut file = File::open(&audio_file_path).await
            .map_err(|e| AppError::FileSystemError(format!("无法打开音频文件: {}", e)))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await
            .map_err(|e| AppError::FileSystemError(format!("无法读取音频文件: {}", e)))?;
        
        // 输出音频文件详细信息
        let file_size = buffer.len();
        let file_size_mb = file_size as f64 / 1024.0 / 1024.0;
        println!("📊 音频文件信息:");
        println!("   - 文件路径: {:?}", audio_file_path.as_ref());
        println!("   - 文件大小: {} bytes ({:.2} MB)", file_size, file_size_mb);
        
        if file_size == 0 {
            println!("❌ 错误：音频文件为空");
            return Ok(TranscriptionResult {
                text: "".to_string(),
                confidence: None,
                duration: None,
                language: None,
            });
        }
        
        if file_size > 25 * 1024 * 1024 {
            println!("⚠️ 警告：音频文件过大 ({:.2} MB)，可能导致上传失败", file_size_mb);
        }

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
        println!("📤 开始上传文件到API...");
        let upload_resp = self.client
            .post("https://ly.gl173.com/api/v1/upload-file")
            .header("Authorization", format!("Bearer {}", bearer_token))
            .multipart(form)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| {
                println!("❌ 上传文件失败: {}", e);
                AppError::HttpRequestError(format!("上传文件失败: {}", e))
            })?;
        println!("✅ 文件上传完成");

        let status = upload_resp.status();
        let upload_text = upload_resp.text().await.unwrap_or_default();
        
        println!("📥 上传响应状态: {}", status);
        println!("📥 上传响应内容: {}", upload_text);
        
        if !status.is_success() {
            return Err(AppError::ApiTranscriptionError(format!("上传接口错误({}): {}", status, upload_text)));
        }
        
        let upload_json: serde_json::Value = serde_json::from_str(&upload_text)
            .map_err(|e| AppError::DataSerializationError(format!("解析上传响应失败: {} - {}", e, upload_text)))?;
        
        let code = upload_json["code"].as_i64().unwrap_or(0);
        
        // 处理常见错误代码
        if code == 26004 || code == 401 {
            println!("⚠️ 上传文件失败(code: {})，返回空结果", code);
            println!("🔍 完整响应: {}", upload_text);
            println!("💡 可能原因：token过期或API服务异常");
            // 统一返回空结果而不是错误，让重试机制正常工作
            return Ok(TranscriptionResult {
                text: "".to_string(),
                confidence: None,
                duration: None,
                language: None,
            });
        }
        
        if code != 200 {
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
        println!("🔄 创建转录任务，file_id: {}", file_id);
        let task_resp = self.client
            .post("https://ly.gl173.com/api/v1/task-add")
            .header("Authorization", format!("Bearer {}", bearer_token))
            .form(&[("file_id", file_id.clone())])
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| {
                println!("❌ 创建任务失败: {}", e);
                AppError::HttpRequestError(format!("创建任务失败: {}", e))
            })?;
        println!("✅ 任务创建完成");

        let task_text = task_resp.text().await.unwrap_or_default();
        println!("📝 任务创建响应: {}", task_text);
        let task_json: serde_json::Value = serde_json::from_str(&task_text)
            .map_err(|e| AppError::DataSerializationError(format!("解析任务响应失败: {}", e)))?;
        
        let code = task_json["code"].as_i64().unwrap_or(0);
        
        // 处理常见错误代码
        if code == 26004 || code == 401 {
            println!("⚠️ 创建任务失败(code: {})，返回空结果", code);
            println!("🔍 完整响应: {}", task_text);
            println!("💡 可能原因：token过期或API服务异常");
            // 统一返回空结果而不是错误，让重试机制正常工作
            return Ok(TranscriptionResult {
                text: "".to_string(),
                confidence: None,
                duration: None,
                language: None,
            });
        }
        
        if code != 200 {
            return Err(AppError::ApiTranscriptionError(format!("创建任务返回非200: {}", task_text)));
        }
        
        let task_id = task_json["data"]["task_id"].as_str()
            .ok_or_else(|| AppError::ApiTranscriptionError("无法获取task_id".to_string()))?;

        // 3) 轮询任务状态直到完成
        println!("⏳ 等待转录完成，任务ID: {}", task_id);
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 60; // 最大尝试次数（时间取决于轮询间隔）
        
        // 首次请求前等待一下，让服务器有时间处理
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        loop {
            if attempts >= MAX_ATTEMPTS {
                println!("⚠️ 转录超时，返回空结果");
                return Ok(TranscriptionResult { 
                    text: "".to_string(), 
                    confidence: None, 
                    duration: None, 
                    language: None 
                });
            }
            
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
            
            let code = status_json["code"].as_i64().unwrap_or(0);
            
            // 处理特定错误代码
            if code == 26004 {
                // 任务不存在或已过期，尝试返回空结果而不是错误
                println!("⚠️ 任务ID无效或已过期(26004)，返回空结果");
                return Ok(TranscriptionResult {
                    text: "".to_string(),
                    confidence: None,
                    duration: None,
                    language: None,
                });
            }
            
            if code != 200 {
                return Err(AppError::ApiTranscriptionError(format!("查询任务状态返回非200: {}", status_text)));
            }
            
            let progress = status_json["data"]["progress"].as_i64().unwrap_or(0);
            println!("📊 任务进度: {}% (尝试 {}/{}) - task_id: {}", 
                     progress * 100, attempts, MAX_ATTEMPTS, task_id);
            
            if progress == 1 {
                // 转录完成，尝试多种可能的结果字段
                println!("🔍 完整状态响应: {}", status_text);
                println!("📊 尝试获取结果字段...");
                
                // 尝试各种字段并记录
                let result_text = if let Some(text) = status_json["data"]["result"].as_str() {
                    println!("✅ 从 data.result 获取到结果: {}", text);
                    text.to_string()
                } else if let Some(text) = status_json["data"]["content"].as_str() {
                    println!("✅ 从 data.content 获取到结果: {}", text);
                    text.to_string()
                } else if let Some(text) = status_json["data"]["text"].as_str() {
                    println!("✅ 从 data.text 获取到结果: {}", text);
                    text.to_string()
                } else if let Some(text) = status_json["result"].as_str() {
                    println!("✅ 从 result 获取到结果: {}", text);
                    text.to_string()
                } else if let Some(text) = status_json["content"].as_str() {
                    println!("✅ 从 content 获取到结果: {}", text);
                    text.to_string()
                } else if let Some(text) = status_json["text"].as_str() {
                    println!("✅ 从 text 获取到结果: {}", text);
                    text.to_string()
                } else {
                    println!("❌ 未找到任何结果字段");
                    "".to_string()
                };
                
                if result_text.is_empty() {
                    // 如果所有字段都为空，打印完整响应用于调试
                    println!("⚠️ API返回结果为空，完整响应数据: {:#}", status_json);
                    // 返回空结果而不是错误，让前端可以重试
                    return Ok(TranscriptionResult { 
                        text: "".to_string(), 
                        confidence: None, 
                        duration: None, 
                        language: None 
                    });
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
                // 根据尝试次数调整等待时间：前5次快速轮询，之后逐渐增加间隔
                let wait_time = if attempts <= 5 {
                    1000  // 前5次：1秒
                } else if attempts <= 10 {
                    2000  // 6-10次：2秒
                } else {
                    3000  // 10次以后：3秒
                };
                tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
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