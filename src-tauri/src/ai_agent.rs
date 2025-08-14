use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAgentRequest {
    pub text: String,
    pub agent_type: AIAgentType,
    pub options: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAgentResponse {
    pub original_text: String,
    pub processed_text: String,
    pub agent_type: AIAgentType,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AIAgentType {
    TextEnhancement,
    Translation,
    Summarization,
    GrammarCorrection,
    ToneAdjustment,
    KeywordExtraction,
    CodeExplanation,
    Custom,
}

pub struct AIAgent {
    api_key: String,
    client: reqwest::Client,
}

impl AIAgent {
    pub fn new(api_key: String, client: reqwest::Client) -> Self {
        Self { api_key, client }
    }

    pub async fn process(&self, request: AIAgentRequest) -> Result<AIAgentResponse, String> {
        let system_prompt = match &request.agent_type {
            AIAgentType::TextEnhancement => {
                "You are a professional text editor. Enhance the following text to be more clear, \
                 professional, and well-structured while maintaining the original meaning. \
                 Correct any grammar or spelling errors."
            }
            AIAgentType::Translation => {
                let target_lang = request.options.get("target_language")
                    .map(|s| s.as_str())
                    .unwrap_or("English");
                &format!("You are a professional translator. Translate the following text to {}. \
                         Maintain the tone and context of the original text.", target_lang)
            }
            AIAgentType::Summarization => {
                "You are an expert at summarizing text. Create a concise summary of the following text, \
                 capturing the key points and main ideas. Keep it brief but comprehensive."
            }
            AIAgentType::GrammarCorrection => {
                "You are a grammar expert. Correct any grammatical errors in the following text \
                 while keeping the original meaning and tone intact."
            }
            AIAgentType::ToneAdjustment => {
                let target_tone = request.options.get("target_tone")
                    .map(|s| s.as_str())
                    .unwrap_or("professional");
                &format!("You are a communication expert. Adjust the tone of the following text to be more {}. \
                         Maintain the core message while changing the tone.", target_tone)
            }
            AIAgentType::KeywordExtraction => {
                "You are an expert at keyword extraction. Extract the most important keywords and phrases \
                 from the following text. Return them as a comma-separated list."
            }
            AIAgentType::CodeExplanation => {
                "You are a programming expert. Explain the following code or technical text in simple terms \
                 that a non-technical person can understand."
            }
            AIAgentType::Custom => {
                request.options.get("system_prompt")
                    .map(|s| s.as_str())
                    .unwrap_or("Process the following text:")
            }
        };

        // Call OpenAI API
        let processed_text = self.call_openai_api(&request.text, system_prompt).await?;

        Ok(AIAgentResponse {
            original_text: request.text.clone(),
            processed_text,
            agent_type: request.agent_type,
            metadata: request.options,
        })
    }

    async fn call_openai_api(&self, text: &str, system_prompt: &str) -> Result<String, String> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<Message>,
            temperature: f32,
            max_tokens: i32,
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

        let request_body = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: text.to_string(),
                },
            ],
            temperature: 0.7,
            max_tokens: 1000,
        };

        let response = self.client
            .post("https://ttkk.inping.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI API error: {}", error_text));
        }

        let response_data: OpenAIResponse = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        response_data.choices.first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| "No response from AI".to_string())
    }

    pub async fn process_chain(&self, text: String, agents: Vec<AIAgentType>) -> Result<Vec<AIAgentResponse>, String> {
        let mut results = Vec::new();
        let mut current_text = text;

        for agent_type in agents {
            let request = AIAgentRequest {
                text: current_text.clone(),
                agent_type: agent_type.clone(),
                options: HashMap::new(),
            };

            let response = self.process(request).await?;
            current_text = response.processed_text.clone();
            results.push(response);
        }

        Ok(results)
    }
}

// Preset agent configurations
pub fn get_preset_agents() -> HashMap<String, Vec<AIAgentType>> {
    let mut presets = HashMap::new();
    
    // Professional email workflow
    presets.insert(
        "professional_email".to_string(),
        vec![
            AIAgentType::GrammarCorrection,
            AIAgentType::TextEnhancement,
            AIAgentType::ToneAdjustment,
        ]
    );

    // Translation and localization workflow
    presets.insert(
        "translate_and_adapt".to_string(),
        vec![
            AIAgentType::Translation,
            AIAgentType::TextEnhancement,
        ]
    );

    // Meeting notes workflow
    presets.insert(
        "meeting_notes".to_string(),
        vec![
            AIAgentType::Summarization,
            AIAgentType::KeywordExtraction,
        ]
    );

    // Code documentation workflow
    presets.insert(
        "code_documentation".to_string(),
        vec![
            AIAgentType::CodeExplanation,
            AIAgentType::TextEnhancement,
        ]
    );

    presets
}