use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::errors::{AppError, AppResult};
use super::types::{AIAgentType, AIAgentRequest, AIAgentResponse, AgentConfig};

#[derive(Debug)]
pub struct AIProcessor {
    client: Client,
    api_key: String,
    config: AgentConfig,
}

impl AIProcessor {
    pub fn new(client: Client, api_key: String, config: AgentConfig) -> Self {
        Self {
            client,
            api_key,
            config,
        }
    }

    /// 处理单个AI代理请求
    pub async fn process(&self, request: AIAgentRequest) -> AppResult<AIAgentResponse> {
        let start_time = std::time::Instant::now();
        
        let system_prompt = self.get_system_prompt(&request.agent_type, &request.options)?;
        
        match self.call_openai_api(&request.text, &system_prompt, &request.context).await {
            Ok(processed_text) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                
                Ok(AIAgentResponse {
                    success: true,
                    original_text: request.text,
                    processed_text,
                    agent_type: request.agent_type,
                    processing_time_ms: processing_time,
                    metadata: request.options,
                    error: None,
                })
            },
            Err(error) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                
                Ok(AIAgentResponse {
                    success: false,
                    original_text: request.text,
                    processed_text: String::new(),
                    agent_type: request.agent_type,
                    processing_time_ms: processing_time,
                    metadata: request.options,
                    error: Some(error.to_string()),
                })
            }
        }
    }

    /// 获取系统提示词
    fn get_system_prompt(&self, agent_type: &AIAgentType, options: &HashMap<String, String>) -> AppResult<String> {
        let prompt = match agent_type {
            AIAgentType::TextEnhancement => {
                "你是一位专业的文本编辑器。请增强以下文本，使其更清晰、专业和结构良好，同时保持原意不变。纠正任何语法或拼写错误。"
            },
            AIAgentType::Translation => {
                let default_lang = "英语".to_string();
                let target_lang = options.get("target_language")
                    .unwrap_or(&default_lang);
                return Ok(format!("你是一位专业翻译。请将以下文本翻译成{}。保持原文的语调和上下文。", target_lang));
            },
            AIAgentType::Summarization => {
                "你是一位摘要专家。请为以下文本创建简洁的摘要，捕捉关键点和主要思想。保持简洁但全面。"
            },
            AIAgentType::GrammarCorrection => {
                "你是一位语法专家。请纠正以下文本中的语法错误，同时保持原意和语调不变。"
            },
            AIAgentType::ToneAdjustment => {
                let default_tone = "专业".to_string();
                let target_tone = options.get("target_tone")
                    .unwrap_or(&default_tone);
                return Ok(format!("你是一位沟通专家。请调整以下文本的语调，使其更加{}。保持核心信息的同时改变语调。", target_tone));
            },
            AIAgentType::KeywordExtraction => {
                "你是关键词提取专家。请从以下文本中提取最重要的关键词和短语。以逗号分隔的列表形式返回。"
            },
            AIAgentType::CodeExplanation => {
                "你是编程专家。请用简单的术语解释以下代码或技术文本，让非技术人员也能理解。"
            },
            AIAgentType::SpeechToText => {
                "你是语音识别专家。请将以下语音内容转换为准确的文本，保持原意不变。"
            },
            AIAgentType::AutoInput => {
                "你是智能输入助手。请基于上下文生成合适的文本输入内容。"
            },
            AIAgentType::Formatter => {
                "你是文本格式化专家。请将以下内容格式化，使结构清晰，易于阅读。"
            },
            AIAgentType::Custom => {
                return Ok(options.get("system_prompt")
                    .unwrap_or(&"请处理以下内容：".to_string())
                    .clone());
            }
        };

        Ok(prompt.to_string())
    }

    /// 调用OpenAI API
    async fn call_openai_api(
        &self, 
        text: &str, 
        system_prompt: &str,
        context: &Option<String>
    ) -> AppResult<String> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<Message>,
            temperature: f32,
            max_tokens: u32,
        }

        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<Choice>,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: MessageResponse,
        }

        #[derive(Deserialize)]
        struct MessageResponse {
            content: String,
        }

        let mut messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            }
        ];

        // 添加上下文消息（如果有）
        if let Some(ctx) = context {
            messages.push(Message {
                role: "assistant".to_string(),
                content: format!("上下文信息: {}", ctx),
            });
        }

        messages.push(Message {
            role: "user".to_string(),
            content: text.to_string(),
        });

        let request_body = OpenAIRequest {
            model: self.config.model.clone(),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };

        let response = self.client
            .post(&self.config.api_endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .send()
            .await
            .map_err(|e| AppError::HttpRequestError(format!("发送请求失败: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
            return Err(AppError::OpenAiApiError(format!("API错误({}): {}", status, error_text)));
        }

        let response_data: OpenAIResponse = response.json().await
            .map_err(|e| AppError::DataSerializationError(format!("解析响应失败: {}", e)))?;

        response_data.choices.first()
            .map(|choice| choice.message.content.trim().to_string())
            .filter(|content| !content.is_empty())
            .ok_or_else(|| AppError::OpenAiApiError("AI没有返回响应内容".to_string()))
    }

    /// 批量处理请求
    pub async fn batch_process(&self, requests: Vec<AIAgentRequest>) -> AppResult<Vec<AIAgentResponse>> {
        let mut results = Vec::new();
        
        for request in requests {
            let result = self.process(request).await?;
            results.push(result);
        }
        
        Ok(results)
    }

    /// 测试API连接
    pub async fn test_connection(&self) -> AppResult<bool> {
        let test_request = AIAgentRequest {
            text: "测试连接".to_string(),
            agent_type: AIAgentType::TextEnhancement,
            options: HashMap::new(),
            context: None,
        };

        match self.process(test_request).await {
            Ok(response) => Ok(response.success),
            Err(_) => Ok(false),
        }
    }

    /// 更新配置
    pub fn update_config(&mut self, config: AgentConfig) {
        self.config = config;
    }

    /// 更新API密钥
    pub fn update_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }
}