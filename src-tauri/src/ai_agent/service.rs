use std::sync::Arc;
use std::collections::HashMap;
use reqwest::Client;
use crate::errors::{AppError, AppResult};
use crate::types::AIPrompt;
use super::types::{
    AIAgentType, AIAgentRequest, AIAgentResponse, 
    ChainProcessingRequest, ChainProcessingResponse, 
    AgentConfig
};
use super::processors::AIProcessor;

#[derive(Debug)]
pub struct AIAgentService {
    processor: Arc<AIProcessor>,
    prompts: Arc<tokio::sync::RwLock<Vec<AIPrompt>>>,
}

impl AIAgentService {
    pub fn new(client: Client, api_key: String, config: AgentConfig) -> Self {
        Self {
            processor: Arc::new(AIProcessor::new(client, api_key, config)),
            prompts: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// 处理单个AI代理请求
    pub async fn process_agent_request(&self, request: AIAgentRequest) -> AppResult<AIAgentResponse> {
        println!("🤖 开始AI代理处理: {:?}", request.agent_type);
        
        // 验证请求
        self.validate_request(&request)?;
        
        // 处理请求
        let result = self.processor.process(request).await?;
        
        if result.success {
            println!("✅ AI代理处理成功，耗时: {}ms", result.processing_time_ms);
        } else {
            println!("❌ AI代理处理失败: {:?}", result.error);
        }
        
        Ok(result)
    }

    /// 链式处理请求
    pub async fn process_chain(&self, request: ChainProcessingRequest) -> AppResult<ChainProcessingResponse> {
        let total_start = std::time::Instant::now();
        println!("🔗 开始链式AI处理，步骤数: {}", request.chain.len());
        
        let mut results = Vec::new();
        let mut current_text = request.text.clone();
        
        for (index, step) in request.chain.iter().enumerate() {
            println!("🔗 执行步骤 {}/{}: {:?}", index + 1, request.chain.len(), step.agent_type);
            
            // 检查条件执行
            if let Some(condition) = &step.condition {
                if !self.evaluate_condition(condition, &current_text) {
                    println!("⏭️ 跳过步骤 {} (条件不满足): {}", index + 1, condition);
                    continue;
                }
            }
            
            let agent_request = AIAgentRequest {
                text: current_text.clone(),
                agent_type: step.agent_type.clone(),
                options: step.options.clone(),
                context: request.context.clone(),
            };
            
            match self.processor.process(agent_request).await {
                Ok(response) => {
                    if response.success {
                        current_text = response.processed_text.clone();
                        println!("✅ 步骤 {} 完成，耗时: {}ms", index + 1, response.processing_time_ms);
                    } else {
                        println!("❌ 步骤 {} 失败: {:?}", index + 1, response.error);
                        return Ok(ChainProcessingResponse {
                            success: false,
                            original_text: request.text,
                            final_text: current_text,
                            steps: results,
                            total_processing_time_ms: total_start.elapsed().as_millis() as u64,
                            error: response.error,
                        });
                    }
                    results.push(response);
                },
                Err(e) => {
                    println!("❌ 步骤 {} 执行错误: {}", index + 1, e);
                    return Ok(ChainProcessingResponse {
                        success: false,
                        original_text: request.text,
                        final_text: current_text,
                        steps: results,
                        total_processing_time_ms: total_start.elapsed().as_millis() as u64,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        let total_time = total_start.elapsed().as_millis() as u64;
        println!("✅ 链式处理完成，总耗时: {}ms", total_time);
        
        Ok(ChainProcessingResponse {
            success: true,
            original_text: request.text,
            final_text: current_text,
            steps: results,
            total_processing_time_ms: total_time,
            error: None,
        })
    }

    /// 使用提示词处理文本
    pub async fn process_with_prompt(
        &self, 
        text: String, 
        prompt_id: String,
    ) -> AppResult<AIAgentResponse> {
        let prompts = self.prompts.read().await;
        let prompt = prompts.iter()
            .find(|p| p.id == prompt_id)
            .ok_or_else(|| AppError::ValidationError(format!("提示词不存在: {}", prompt_id)))?;
        
        let mut options = HashMap::new();
        options.insert("system_prompt".to_string(), prompt.prompt_text.clone());
        
        let request = AIAgentRequest {
            text,
            agent_type: self.parse_agent_type(&prompt.agent_type)?,
            options,
            context: None,
        };
        
        self.process_agent_request(request).await
    }

    /// 添加提示词
    pub async fn add_prompt(&self, prompt: AIPrompt) -> AppResult<()> {
        let mut prompts = self.prompts.write().await;
        
        // 检查是否已存在相同ID的提示词
        if prompts.iter().any(|p| p.id == prompt.id) {
            return Err(AppError::ValidationError(format!("提示词ID已存在: {}", prompt.id)));
        }
        
        prompts.push(prompt);
        Ok(())
    }

    /// 更新提示词
    pub async fn update_prompt(&self, prompt: AIPrompt) -> AppResult<()> {
        let mut prompts = self.prompts.write().await;
        
        if let Some(existing) = prompts.iter_mut().find(|p| p.id == prompt.id) {
            *existing = prompt;
            Ok(())
        } else {
            Err(AppError::ValidationError(format!("提示词不存在: {}", prompt.id)))
        }
    }

    /// 删除提示词
    pub async fn remove_prompt(&self, prompt_id: &str) -> AppResult<()> {
        let mut prompts = self.prompts.write().await;
        let initial_len = prompts.len();
        prompts.retain(|p| p.id != prompt_id);
        
        if prompts.len() == initial_len {
            Err(AppError::ValidationError(format!("提示词不存在: {}", prompt_id)))
        } else {
            Ok(())
        }
    }

    /// 获取所有提示词
    pub async fn get_prompts(&self) -> Vec<AIPrompt> {
        self.prompts.read().await.clone()
    }

    /// 获取指定类型的提示词
    pub async fn get_prompts_by_type(&self, agent_type: &str) -> Vec<AIPrompt> {
        let prompts = self.prompts.read().await;
        prompts.iter()
            .filter(|p| p.agent_type == agent_type)
            .cloned()
            .collect()
    }

    /// 初始化默认提示词
    pub async fn initialize_default_prompts(&self) -> AppResult<()> {
        let default_prompts = self.create_default_prompts();
        let mut prompts = self.prompts.write().await;
        prompts.extend(default_prompts);
        Ok(())
    }

    /// 测试AI服务连接
    pub async fn test_connection(&self) -> AppResult<bool> {
        self.processor.test_connection().await
    }

    /// 验证请求
    fn validate_request(&self, request: &AIAgentRequest) -> AppResult<()> {
        if request.text.trim().is_empty() {
            return Err(AppError::ValidationError("输入文本不能为空".to_string()));
        }
        
        if request.text.len() > 10000 {
            return Err(AppError::ValidationError("输入文本过长，最大支持10000字符".to_string()));
        }
        
        Ok(())
    }

    /// 评估条件
    fn evaluate_condition(&self, condition: &str, text: &str) -> bool {
        match condition {
            "not_empty" => !text.trim().is_empty(),
            "has_code" => text.contains("```") || text.contains("function") || text.contains("class"),
            "is_long" => text.len() > 500,
            "is_short" => text.len() <= 500,
            _ => true, // 默认条件为真
        }
    }

    /// 解析代理类型
    fn parse_agent_type(&self, agent_type_str: &str) -> AppResult<AIAgentType> {
        match agent_type_str {
            "text-enhancer" => Ok(AIAgentType::TextEnhancement),
            "translator" => Ok(AIAgentType::Translation),
            "summarizer" => Ok(AIAgentType::Summarization),
            "grammar-check" => Ok(AIAgentType::GrammarCorrection),
            "tone-adjuster" => Ok(AIAgentType::ToneAdjustment),
            "keyword-extractor" => Ok(AIAgentType::KeywordExtraction),
            "code-explainer" => Ok(AIAgentType::CodeExplanation),
            "speech-to-text" => Ok(AIAgentType::SpeechToText),
            "auto-input" => Ok(AIAgentType::AutoInput),
            "formatter" => Ok(AIAgentType::Formatter),
            "custom" => Ok(AIAgentType::Custom),
            _ => Err(AppError::ValidationError(format!("不支持的代理类型: {}", agent_type_str))),
        }
    }

    /// 创建默认提示词
    fn create_default_prompts(&self) -> Vec<AIPrompt> {
        use uuid::Uuid;
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        vec![
            AIPrompt {
                id: Uuid::new_v4().to_string(),
                name: "文本增强".to_string(),
                description: "优化和增强文本内容，使其更清晰准确".to_string(),
                agent_type: "text-enhancer".to_string(),
                prompt_text: "请优化以下文本，使其更清晰、准确和专业：".to_string(),
                is_active: true,
                created_at: timestamp,
                updated_at: timestamp,
            },
            AIPrompt {
                id: Uuid::new_v4().to_string(),
                name: "翻译".to_string(),
                description: "将文本翻译为目标语言".to_string(),
                agent_type: "translator".to_string(),
                prompt_text: "请将以下文本翻译为指定的目标语言，保持原意和语调：".to_string(),
                is_active: true,
                created_at: timestamp,
                updated_at: timestamp,
            },
            AIPrompt {
                id: Uuid::new_v4().to_string(),
                name: "摘要".to_string(),
                description: "生成内容的简洁摘要".to_string(),
                agent_type: "summarizer".to_string(),
                prompt_text: "请为以下内容生成简洁明了的摘要，突出要点：".to_string(),
                is_active: true,
                created_at: timestamp,
                updated_at: timestamp,
            },
        ]
    }
}