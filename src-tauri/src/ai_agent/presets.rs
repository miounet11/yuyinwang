use std::collections::HashMap;
use super::types::{AIAgentType, ChainStep};

/// 预设的AI代理工作流
pub struct AgentPresets;

impl AgentPresets {
    /// 获取所有预设工作流
    pub fn get_all_presets() -> HashMap<String, Vec<ChainStep>> {
        let mut presets = HashMap::new();
        
        // 专业邮件工作流
        presets.insert(
            "professional_email".to_string(),
            vec![
                ChainStep {
                    agent_type: AIAgentType::GrammarCorrection,
                    options: HashMap::new(),
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::TextEnhancement,
                    options: HashMap::new(),
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::ToneAdjustment,
                    options: {
                        let mut opts = HashMap::new();
                        opts.insert("target_tone".to_string(), "professional".to_string());
                        opts
                    },
                    condition: None,
                },
            ]
        );

        // 翻译和本地化工作流
        presets.insert(
            "translation_workflow".to_string(),
            vec![
                ChainStep {
                    agent_type: AIAgentType::GrammarCorrection,
                    options: HashMap::new(),
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::Translation,
                    options: {
                        let mut opts = HashMap::new();
                        opts.insert("target_language".to_string(), "English".to_string());
                        opts
                    },
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::TextEnhancement,
                    options: HashMap::new(),
                    condition: None,
                },
            ]
        );

        // 内容创作工作流
        presets.insert(
            "content_creation".to_string(),
            vec![
                ChainStep {
                    agent_type: AIAgentType::TextEnhancement,
                    options: HashMap::new(),
                    condition: Some("not_empty".to_string()),
                },
                ChainStep {
                    agent_type: AIAgentType::KeywordExtraction,
                    options: HashMap::new(),
                    condition: Some("is_long".to_string()),
                },
                ChainStep {
                    agent_type: AIAgentType::Formatter,
                    options: HashMap::new(),
                    condition: None,
                },
            ]
        );

        // 技术文档工作流
        presets.insert(
            "technical_documentation".to_string(),
            vec![
                ChainStep {
                    agent_type: AIAgentType::CodeExplanation,
                    options: HashMap::new(),
                    condition: Some("has_code".to_string()),
                },
                ChainStep {
                    agent_type: AIAgentType::TextEnhancement,
                    options: HashMap::new(),
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::Formatter,
                    options: HashMap::new(),
                    condition: None,
                },
            ]
        );

        // 学术写作工作流
        presets.insert(
            "academic_writing".to_string(),
            vec![
                ChainStep {
                    agent_type: AIAgentType::GrammarCorrection,
                    options: HashMap::new(),
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::TextEnhancement,
                    options: HashMap::new(),
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::ToneAdjustment,
                    options: {
                        let mut opts = HashMap::new();
                        opts.insert("target_tone".to_string(), "academic".to_string());
                        opts
                    },
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::Formatter,
                    options: HashMap::new(),
                    condition: None,
                },
            ]
        );

        // 社交媒体工作流
        presets.insert(
            "social_media".to_string(),
            vec![
                ChainStep {
                    agent_type: AIAgentType::TextEnhancement,
                    options: HashMap::new(),
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::ToneAdjustment,
                    options: {
                        let mut opts = HashMap::new();
                        opts.insert("target_tone".to_string(), "casual".to_string());
                        opts
                    },
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::Summarization,
                    options: HashMap::new(),
                    condition: Some("is_long".to_string()),
                },
            ]
        );

        // 客户服务工作流
        presets.insert(
            "customer_service".to_string(),
            vec![
                ChainStep {
                    agent_type: AIAgentType::ToneAdjustment,
                    options: {
                        let mut opts = HashMap::new();
                        opts.insert("target_tone".to_string(), "friendly".to_string());
                        opts
                    },
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::TextEnhancement,
                    options: HashMap::new(),
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::GrammarCorrection,
                    options: HashMap::new(),
                    condition: None,
                },
            ]
        );

        // 快速修正工作流
        presets.insert(
            "quick_fix".to_string(),
            vec![
                ChainStep {
                    agent_type: AIAgentType::GrammarCorrection,
                    options: HashMap::new(),
                    condition: None,
                },
                ChainStep {
                    agent_type: AIAgentType::TextEnhancement,
                    options: HashMap::new(),
                    condition: Some("not_empty".to_string()),
                },
            ]
        );

        presets
    }

    /// 根据用途获取推荐的工作流
    pub fn get_recommended_preset(purpose: &str) -> Option<Vec<ChainStep>> {
        let presets = Self::get_all_presets();
        
        let preset_key = match purpose.to_lowercase().as_str() {
            "email" | "邮件" => "professional_email",
            "translate" | "翻译" => "translation_workflow", 
            "content" | "内容" => "content_creation",
            "technical" | "技术" => "technical_documentation",
            "academic" | "学术" => "academic_writing",
            "social" | "社交" => "social_media",
            "service" | "客服" => "customer_service",
            "fix" | "修正" => "quick_fix",
            _ => return None,
        };
        
        presets.get(preset_key).cloned()
    }

    /// 获取预设的显示名称
    pub fn get_preset_display_names() -> HashMap<String, String> {
        let mut names = HashMap::new();
        
        names.insert("professional_email".to_string(), "专业邮件".to_string());
        names.insert("translation_workflow".to_string(), "翻译工作流".to_string());
        names.insert("content_creation".to_string(), "内容创作".to_string());
        names.insert("technical_documentation".to_string(), "技术文档".to_string());
        names.insert("academic_writing".to_string(), "学术写作".to_string());
        names.insert("social_media".to_string(), "社交媒体".to_string());
        names.insert("customer_service".to_string(), "客户服务".to_string());
        names.insert("quick_fix".to_string(), "快速修正".to_string());
        
        names
    }

    /// 获取预设的描述
    pub fn get_preset_descriptions() -> HashMap<String, String> {
        let mut descriptions = HashMap::new();
        
        descriptions.insert(
            "professional_email".to_string(), 
            "适用于商务邮件，包含语法修正、文本增强和专业语调调整".to_string()
        );
        descriptions.insert(
            "translation_workflow".to_string(),
            "完整的翻译工作流，包含语法修正、翻译和文本增强".to_string()
        );
        descriptions.insert(
            "content_creation".to_string(),
            "内容创作工作流，包含文本增强、关键词提取和格式化".to_string()
        );
        descriptions.insert(
            "technical_documentation".to_string(),
            "技术文档处理，包含代码解释、文本增强和格式化".to_string()
        );
        descriptions.insert(
            "academic_writing".to_string(),
            "学术写作工作流，包含语法修正、文本增强、学术语调和格式化".to_string()
        );
        descriptions.insert(
            "social_media".to_string(),
            "社交媒体内容处理，包含文本增强、轻松语调和摘要".to_string()
        );
        descriptions.insert(
            "customer_service".to_string(),
            "客服响应优化，包含友好语调、文本增强和语法修正".to_string()
        );
        descriptions.insert(
            "quick_fix".to_string(),
            "快速修正工作流，包含语法修正和基本文本增强".to_string()
        );
        
        descriptions
    }

    /// 验证预设是否存在
    pub fn preset_exists(preset_id: &str) -> bool {
        Self::get_all_presets().contains_key(preset_id)
    }

    /// 获取预设的步骤数量
    pub fn get_preset_step_count(preset_id: &str) -> usize {
        Self::get_all_presets()
            .get(preset_id)
            .map_or(0, |steps| steps.len())
    }
}