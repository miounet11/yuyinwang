use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use serde_json::Value;

pub struct SecureApiClient {
    client: Client,
}

impl SecureApiClient {
    /// 创建安全的HTTP客户端
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = ClientBuilder::new()
            // 启用HTTPS证书验证
            .danger_accept_invalid_certs(false)
            // .danger_accept_invalid_hostnames(false) // 此方法在新版本中已移除
            // 设置合理的超时
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            // 限制重定向次数
            .redirect(reqwest::redirect::Policy::limited(3))
            // 设置用户代理
            .user_agent("LuYinWang/3.0.1 (Secure Client)")
            .build()?;
            
        Ok(Self { client })
    }
    
    /// 安全的第三方API调用 - ly.gl173.com
    pub async fn safe_transcribe_via_luyin_api(
        &self, 
        audio_file_path: &std::path::PathBuf
    ) -> Result<TranscriptionResult, String> {
        use std::fs::File;
        use std::io::Read;
        
        // 1. 验证文件路径和大小
        const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
        
        let metadata = std::fs::metadata(audio_file_path)
            .map_err(|e| format!("无法读取文件信息: {}", e))?;
            
        if metadata.len() > MAX_FILE_SIZE {
            return Err("文件大小超过100MB限制".to_string());
        }
        
        // 2. 验证文件类型
        let allowed_extensions = ["wav", "mp3", "m4a", "flac", "ogg"];
        if let Some(extension) = audio_file_path.extension() {
            let ext_str = extension.to_str().unwrap_or("").to_lowercase();
            if !allowed_extensions.contains(&ext_str.as_str()) {
                return Err(format!("不支持的音频格式: {}", ext_str));
            }
        } else {
            return Err("文件必须有有效的音频扩展名".to_string());
        }
        
        // 3. 读取音频文件
        let mut file = File::open(audio_file_path)
            .map_err(|e| format!("无法打开音频文件: {}", e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("无法读取音频文件: {}", e))?;
            
        // 4. 验证URL安全性
        self.validate_api_endpoint("https://ly.gl173.com")?;
        
        // 5. 上传文件（带重试机制）
        let file_id = self.upload_with_retry(&buffer, audio_file_path).await?;
        
        // 6. 创建转换任务
        let task_id = self.create_transcription_task(&file_id).await?;
        
        // 7. 轮询结果（带超时）
        self.poll_transcription_result(&task_id).await
    }
    
    /// 验证API端点安全性
    fn validate_api_endpoint(&self, url: &str) -> Result<(), String> {
        // 确保是HTTPS
        if !url.starts_with("https://") {
            return Err("API端点必须使用HTTPS".to_string());
        }
        
        // 验证域名白名单
        let allowed_hosts = ["ly.gl173.com", "api.openai.com", "api.deepgram.com"];
        let url_parsed = url::Url::parse(url)
            .map_err(|_| "无效的URL格式".to_string())?;
            
        if let Some(host) = url_parsed.host_str() {
            if !allowed_hosts.contains(&host) {
                return Err(format!("不允许的API主机: {}", host));
            }
        } else {
            return Err("无法解析主机名".to_string());
        }
        
        Ok(())
    }
    
    async fn upload_with_retry(&self, buffer: &[u8], file_path: &std::path::PathBuf) -> Result<String, String> {
        use reqwest::multipart::{Form, Part};
        
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav");
            
        for attempt in 1..=3 {
            let part = Part::bytes(buffer.to_vec()).file_name(file_name.to_string());
            let form = Form::new().part("file[]", part);
            
            match self.client
                .post("https://ly.gl173.com/api/v1/upload-file")
                .multipart(form)
                .send()
                .await {
                Ok(response) => {
                    if response.status().is_success() {
                        let json: Value = response.json().await
                            .map_err(|e| format!("解析响应失败: {}", e))?;
                        
                        if let Some(file_id) = json.get("file_id").and_then(|v| v.as_str()) {
                            return Ok(file_id.to_string());
                        } else {
                            return Err("响应中缺少file_id".to_string());
                        }
                    } else {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        return Err(format!("上传失败: HTTP {} - {}", status, text));
                    }
                }
                Err(e) if attempt < 3 => {
                    println!("上传重试 {}/3: {}", attempt, e);
                    tokio::time::sleep(Duration::from_secs(2 * attempt)).await;
                }
                Err(e) => return Err(format!("上传失败: {}", e)),
            }
        }
        
        Err("上传重试失败".to_string())
    }
    
    async fn create_transcription_task(&self, file_id: &str) -> Result<String, String> {
        let response = self.client
            .post("https://ly.gl173.com/api/v1/task-add")
            .form(&[("file_id", file_id)])
            .send()
            .await
            .map_err(|e| format!("创建任务失败: {}", e))?;
            
        if !response.status().is_success() {
            return Err(format!("创建任务失败: HTTP {}", response.status()));
        }
        
        let json: Value = response.json().await
            .map_err(|e| format!("解析任务响应失败: {}", e))?;
            
        if let Some(task_id) = json.get("task_id").and_then(|v| v.as_str()) {
            Ok(task_id.to_string())
        } else {
            Err("响应中缺少task_id".to_string())
        }
    }
    
    async fn poll_transcription_result(&self, task_id: &str) -> Result<TranscriptionResult, String> {
        const MAX_ATTEMPTS: u32 = 60; // 最大轮询次数 (10分钟)
        const POLL_INTERVAL: u64 = 10; // 轮询间隔 (秒)
        
        for attempt in 1..=MAX_ATTEMPTS {
            let response = self.client
                .post("https://ly.gl173.com/api/v1/task-progress")
                .form(&[("task_id", task_id)])
                .send()
                .await
                .map_err(|e| format!("查询进度失败: {}", e))?;
                
            if !response.status().is_success() {
                return Err(format!("查询进度失败: HTTP {}", response.status()));
            }
            
            let json: Value = response.json().await
                .map_err(|e| format!("解析进度响应失败: {}", e))?;
                
            if let Some(status) = json.get("status").and_then(|v| v.as_str()) {
                match status {
                    "completed" => {
                        if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                            return Ok(TranscriptionResult {
                                text: text.to_string(),
                                confidence: 0.9, // 默认置信度
                                duration: 0.0,   // 第三方API可能不提供
                            });
                        } else {
                            return Err("转录完成但缺少文本结果".to_string());
                        }
                    }
                    "failed" => {
                        let error = json.get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("未知错误");
                        return Err(format!("转录失败: {}", error));
                    }
                    "processing" | "pending" => {
                        println!("转录进行中... 尝试 {}/{}", attempt, MAX_ATTEMPTS);
                        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL)).await;
                    }
                    _ => {
                        return Err(format!("未知任务状态: {}", status));
                    }
                }
            } else {
                return Err("响应中缺少状态信息".to_string());
            }
        }
        
        Err("转录超时 (10分钟)".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub duration: f32,
}