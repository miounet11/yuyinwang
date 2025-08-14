use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AIAgentType {
    TextEnhancement,
    Translation,
    Summarization,
    GrammarCorrection,
    ToneAdjustment,
    KeywordExtraction,
    CodeExplanation,
    SpeechToText,
    AutoInput,
    Formatter,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAgentRequest {
    pub text: String,
    pub agent_type: AIAgentType,
    pub options: HashMap<String, String>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAgentResponse {
    pub success: bool,
    pub original_text: String,
    pub processed_text: String,
    pub agent_type: AIAgentType,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainProcessingRequest {
    pub text: String,
    pub chain: Vec<ChainStep>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    pub agent_type: AIAgentType,
    pub options: HashMap<String, String>,
    pub condition: Option<String>, // 条件执行
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainProcessingResponse {
    pub success: bool,
    pub original_text: String,
    pub final_text: String,
    pub steps: Vec<AIAgentResponse>,
    pub total_processing_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
    pub api_endpoint: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
            timeout_seconds: 30,
            api_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
        }
    }
}

impl AIAgentType {
    /// 获取代理类型的显示名称
    pub fn display_name(&self) -> &str {
        match self {
            AIAgentType::TextEnhancement => "文本增强",
            AIAgentType::Translation => "翻译",
            AIAgentType::Summarization => "摘要",
            AIAgentType::GrammarCorrection => "语法修正",
            AIAgentType::ToneAdjustment => "语调调整",
            AIAgentType::KeywordExtraction => "关键词提取",
            AIAgentType::CodeExplanation => "代码解释",
            AIAgentType::SpeechToText => "语音转文字",
            AIAgentType::AutoInput => "自动输入",
            AIAgentType::Formatter => "格式化",
            AIAgentType::Custom => "自定义",
        }
    }

    /// 获取代理类型的描述
    pub fn description(&self) -> &str {
        match self {
            AIAgentType::TextEnhancement => "优化文本内容，使其更清晰、专业",
            AIAgentType::Translation => "将文本翻译为指定语言",
            AIAgentType::Summarization => "生成文本的简洁摘要",
            AIAgentType::GrammarCorrection => "修正语法和拼写错误",
            AIAgentType::ToneAdjustment => "调整文本的语调和风格",
            AIAgentType::KeywordExtraction => "提取文本中的关键词",
            AIAgentType::CodeExplanation => "解释代码和技术内容",
            AIAgentType::SpeechToText => "将语音转换为文字",
            AIAgentType::AutoInput => "基于上下文自动生成输入内容",
            AIAgentType::Formatter => "格式化文本结构",
            AIAgentType::Custom => "使用自定义提示词处理文本",
        }
    }

    /// 获取所有可用的代理类型
    pub fn all_types() -> Vec<AIAgentType> {
        vec![
            AIAgentType::TextEnhancement,
            AIAgentType::Translation,
            AIAgentType::Summarization,
            AIAgentType::GrammarCorrection,
            AIAgentType::ToneAdjustment,
            AIAgentType::KeywordExtraction,
            AIAgentType::CodeExplanation,
            AIAgentType::SpeechToText,
            AIAgentType::AutoInput,
            AIAgentType::Formatter,
            AIAgentType::Custom,
        ]
    }
}