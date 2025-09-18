use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 内容分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysisResult {
    /// 主题标签
    pub topics: Vec<TopicTag>,
    /// 情感分析结果
    pub sentiment: SentimentAnalysis,
    /// 关键信息摘要
    pub key_information: KeyInformation,
    /// 智能分类建议
    pub classification: ContentClassification,
    /// 分析性能指标
    pub performance_metrics: AnalysisMetrics,
    /// 分析时间戳
    pub analyzed_at: DateTime<Utc>,
}

/// 主题标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicTag {
    /// 标签名称
    pub name: String,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    /// 标签类型
    pub category: TopicCategory,
    /// 关键词
    pub keywords: Vec<String>,
    /// 在文本中的位置
    pub positions: Vec<TextPosition>,
}

/// 主题类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopicCategory {
    Business,      // 商务
    Technology,    // 技术
    Education,     // 教育
    Entertainment, // 娱乐
    Health,        // 健康
    Finance,       // 金融
    News,          // 新闻
    Personal,      // 个人
    Meeting,       // 会议
    Interview,     // 访谈
    Lecture,       // 讲座
    Conversation,  // 对话
    Other(String), // 其他自定义类别
}

/// 情感分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    /// 整体情感倾向
    pub overall_sentiment: SentimentPolarity,
    /// 情感强度 (0.0-1.0)
    pub intensity: f32,
    /// 情感时间轴
    pub timeline: Vec<SentimentTimePoint>,
    /// 语调特征
    pub tone_characteristics: ToneCharacteristics,
}

/// 情感极性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SentimentPolarity {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

/// 情感时间点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentTimePoint {
    /// 时间戳（秒）
    pub timestamp: f32,
    /// 情感值 (-1.0 到 1.0)
    pub sentiment_value: f32,
    /// 情感标签
    pub emotion_labels: Vec<String>,
}

/// 语调特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneCharacteristics {
    /// 正式程度 (0.0-1.0)
    pub formality: f32,
    /// 自信程度 (0.0-1.0)
    pub confidence: f32,
    /// 情绪稳定性 (0.0-1.0)
    pub emotional_stability: f32,
    /// 专业程度 (0.0-1.0)
    pub professionalism: f32,
}

/// 关键信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInformation {
    /// 结构化摘要
    pub summary: StructuredSummary,
    /// 重要实体
    pub entities: Vec<NamedEntity>,
    /// 行动项
    pub action_items: Vec<ActionItem>,
    /// 关键数据点
    pub data_points: Vec<DataPoint>,
}

/// 结构化摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredSummary {
    /// 主要观点
    pub main_points: Vec<String>,
    /// 核心结论
    pub conclusions: Vec<String>,
    /// 讨论的问题
    pub questions_discussed: Vec<String>,
    /// 提及的解决方案
    pub solutions_mentioned: Vec<String>,
}

/// 命名实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedEntity {
    /// 实体文本
    pub text: String,
    /// 实体类型
    pub entity_type: EntityType,
    /// 置信度
    pub confidence: f32,
    /// 上下文
    pub context: String,
}

/// 实体类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Person,       // 人名
    Organization, // 组织
    Location,     // 地点
    Date,         // 日期
    Time,         // 时间
    Money,        // 金额
    Product,      // 产品
    Technology,   // 技术
    Event,        // 事件
}

/// 行动项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    /// 行动描述
    pub description: String,
    /// 负责人
    pub assignee: Option<String>,
    /// 截止时间
    pub due_date: Option<DateTime<Utc>>,
    /// 优先级
    pub priority: Priority,
}

/// 优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// 数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// 数据描述
    pub description: String,
    /// 数值
    pub value: Option<f64>,
    /// 单位
    pub unit: Option<String>,
    /// 数据类型
    pub data_type: DataType,
}

/// 数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Metric,     // 指标
    Percentage, // 百分比
    Currency,   // 货币
    Count,      // 计数
    Duration,   // 持续时间
    Other(String),
}

/// 内容分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentClassification {
    /// 推荐分类
    pub suggested_categories: Vec<CategorySuggestion>,
    /// 自动标签
    pub auto_tags: Vec<String>,
    /// 相似内容
    pub similar_content: Vec<SimilarContentRef>,
    /// 知识库关联
    pub knowledge_base_links: Vec<KnowledgeLink>,
}

/// 分类建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySuggestion {
    /// 分类名称
    pub category: String,
    /// 置信度
    pub confidence: f32,
    /// 推荐理由
    pub reasoning: String,
}

/// 相似内容引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarContentRef {
    /// 内容ID
    pub content_id: String,
    /// 相似度分数
    pub similarity_score: f32,
    /// 相似内容标题
    pub title: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 知识库链接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeLink {
    /// 链接ID
    pub link_id: String,
    /// 链接标题
    pub title: String,
    /// 相关性分数
    pub relevance_score: f32,
    /// 链接类型
    pub link_type: KnowledgeLinkType,
}

/// 知识库链接类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeLinkType {
    RelatedDocument,
    ConceptExplanation,
    BackgroundInfo,
    FollowUpResource,
}

/// 文本位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    /// 开始位置
    pub start: usize,
    /// 结束位置
    pub end: usize,
    /// 行号
    pub line: Option<usize>,
}

/// 分析性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    /// 主题识别时间（毫秒）
    pub topic_analysis_time_ms: u64,
    /// 情感分析时间（毫秒）
    pub sentiment_analysis_time_ms: u64,
    /// 关键信息提取时间（毫秒）
    pub key_info_extraction_time_ms: u64,
    /// 总分析时间（毫秒）
    pub total_analysis_time_ms: u64,
    /// 使用的AI模型
    pub ai_model_used: String,
    /// API调用次数
    pub api_calls_made: u32,
    /// 处理的字符数
    pub characters_processed: usize,
}

/// AI内容分析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysisConfig {
    /// OpenAI API配置
    pub openai_config: OpenAIConfig,
    /// 分析选项
    pub analysis_options: AnalysisOptions,
    /// 性能设置
    pub performance_settings: PerformanceSettings,
}

/// OpenAI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// API密钥
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// 温度参数
    pub temperature: f32,
    /// 最大token数
    pub max_tokens: u32,
    /// API基础URL
    pub base_url: Option<String>,
}

/// 分析选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    /// 启用主题分析
    pub enable_topic_analysis: bool,
    /// 启用情感分析
    pub enable_sentiment_analysis: bool,
    /// 启用关键信息提取
    pub enable_key_info_extraction: bool,
    /// 启用智能分类
    pub enable_classification: bool,
    /// 最小置信度阈值
    pub min_confidence_threshold: f32,
    /// 自定义提示词
    pub custom_prompts: HashMap<String, String>,
}

/// 性能设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// 并发分析任务数
    pub max_concurrent_analyses: usize,
    /// 分析超时时间（秒）
    pub analysis_timeout_seconds: u64,
    /// 启用缓存
    pub enable_caching: bool,
    /// 缓存过期时间（秒）
    pub cache_expiry_seconds: u64,
}

impl Default for ContentAnalysisConfig {
    fn default() -> Self {
        Self {
            openai_config: OpenAIConfig {
                api_key: String::new(),
                model: "gpt-4o-mini".to_string(),
                temperature: 0.3,
                max_tokens: 2000,
                base_url: None,
            },
            analysis_options: AnalysisOptions {
                enable_topic_analysis: true,
                enable_sentiment_analysis: true,
                enable_key_info_extraction: true,
                enable_classification: true,
                min_confidence_threshold: 0.6,
                custom_prompts: HashMap::new(),
            },
            performance_settings: PerformanceSettings {
                max_concurrent_analyses: 3,
                analysis_timeout_seconds: 45,
                enable_caching: true,
                cache_expiry_seconds: 3600,
            },
        }
    }
}

/// AI内容分析器
#[derive(Debug)]
pub struct ContentAnalyzer {
    /// HTTP客户端
    http_client: Client,
    /// 配置
    config: Arc<Mutex<ContentAnalysisConfig>>,
    /// 分析缓存
    analysis_cache: Arc<Mutex<HashMap<String, ContentAnalysisResult>>>,
    /// 性能统计
    performance_stats: Arc<Mutex<PerformanceStats>>,
}

/// 性能统计
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// 总分析次数
    pub total_analyses: u64,
    /// 平均分析时间（毫秒）
    pub average_analysis_time_ms: f64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// API调用次数
    pub total_api_calls: u64,
}

impl ContentAnalyzer {
    /// 创建新的内容分析器
    pub fn new(config: ContentAnalysisConfig) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(
                config.performance_settings.analysis_timeout_seconds,
            ))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            config: Arc::new(Mutex::new(config)),
            analysis_cache: Arc::new(Mutex::new(HashMap::new())),
            performance_stats: Arc::new(Mutex::new(PerformanceStats::default())),
        }
    }

    /// 分析文本内容
    pub async fn analyze_content(
        &self,
        text: &str,
        content_id: Option<String>,
    ) -> Result<ContentAnalysisResult, AnalysisError> {
        let start_time = std::time::Instant::now();

        // 检查缓存
        let cache_key = self.generate_cache_key(text, content_id.as_deref());
        if let Some(cached_result) = self.get_cached_result(&cache_key).await {
            self.update_performance_stats(start_time.elapsed().as_millis() as u64, true, 0)
                .await;
            return Ok(cached_result);
        }

        let config = self.config.lock().await.clone();
        let mut api_calls = 0;

        // 执行各种分析任务
        let mut topics = Vec::new();
        let mut sentiment = SentimentAnalysis {
            overall_sentiment: SentimentPolarity::Neutral,
            intensity: 0.0,
            timeline: Vec::new(),
            tone_characteristics: ToneCharacteristics {
                formality: 0.5,
                confidence: 0.5,
                emotional_stability: 0.5,
                professionalism: 0.5,
            },
        };
        let mut key_information = KeyInformation {
            summary: StructuredSummary {
                main_points: Vec::new(),
                conclusions: Vec::new(),
                questions_discussed: Vec::new(),
                solutions_mentioned: Vec::new(),
            },
            entities: Vec::new(),
            action_items: Vec::new(),
            data_points: Vec::new(),
        };
        let mut classification = ContentClassification {
            suggested_categories: Vec::new(),
            auto_tags: Vec::new(),
            similar_content: Vec::new(),
            knowledge_base_links: Vec::new(),
        };

        let topic_start = std::time::Instant::now();
        if config.analysis_options.enable_topic_analysis {
            match self.analyze_topics(text, &config).await {
                Ok(result) => {
                    topics = result;
                    api_calls += 1;
                }
                Err(e) => eprintln!("主题分析失败: {}", e),
            }
        }
        let topic_time = topic_start.elapsed().as_millis() as u64;

        let sentiment_start = std::time::Instant::now();
        if config.analysis_options.enable_sentiment_analysis {
            match self.analyze_sentiment(text, &config).await {
                Ok(result) => {
                    sentiment = result;
                    api_calls += 1;
                }
                Err(e) => eprintln!("情感分析失败: {}", e),
            }
        }
        let sentiment_time = sentiment_start.elapsed().as_millis() as u64;

        let key_info_start = std::time::Instant::now();
        if config.analysis_options.enable_key_info_extraction {
            match self.extract_key_information(text, &config).await {
                Ok(result) => {
                    key_information = result;
                    api_calls += 1;
                }
                Err(e) => eprintln!("关键信息提取失败: {}", e),
            }
        }
        let key_info_time = key_info_start.elapsed().as_millis() as u64;

        if config.analysis_options.enable_classification {
            classification = self.classify_content(text, &topics, &sentiment).await;
        }

        let total_time = start_time.elapsed().as_millis() as u64;

        let result = ContentAnalysisResult {
            topics,
            sentiment,
            key_information,
            classification,
            performance_metrics: AnalysisMetrics {
                topic_analysis_time_ms: topic_time,
                sentiment_analysis_time_ms: sentiment_time,
                key_info_extraction_time_ms: key_info_time,
                total_analysis_time_ms: total_time,
                ai_model_used: config.openai_config.model.clone(),
                api_calls_made: api_calls,
                characters_processed: text.len(),
            },
            analyzed_at: Utc::now(),
        };

        // 缓存结果
        if config.performance_settings.enable_caching {
            self.cache_result(&cache_key, &result).await;
        }

        self.update_performance_stats(total_time, false, api_calls)
            .await;

        Ok(result)
    }

    /// 分析主题
    async fn analyze_topics(
        &self,
        text: &str,
        config: &ContentAnalysisConfig,
    ) -> Result<Vec<TopicTag>, AnalysisError> {
        let prompt = format!(
            "分析以下文本的主题，提取主要话题标签。请以JSON格式返回结果，包含话题名称、置信度、类别和关键词：\n\n{}",
            text
        );

        let response = self.call_openai_api(&prompt, config).await?;

        // 解析OpenAI响应并转换为TopicTag结构
        self.parse_topics_response(&response).await
    }

    /// 分析情感
    async fn analyze_sentiment(
        &self,
        text: &str,
        config: &ContentAnalysisConfig,
    ) -> Result<SentimentAnalysis, AnalysisError> {
        let prompt = format!(
            "分析以下文本的情感倾向、语调特征和情绪变化。请以JSON格式返回详细的情感分析结果：\n\n{}",
            text
        );

        let response = self.call_openai_api(&prompt, config).await?;
        self.parse_sentiment_response(&response).await
    }

    /// 提取关键信息
    async fn extract_key_information(
        &self,
        text: &str,
        config: &ContentAnalysisConfig,
    ) -> Result<KeyInformation, AnalysisError> {
        let prompt = format!(
            "提取以下文本的关键信息，包括主要观点、重要实体、行动项和数据点。请以结构化JSON格式返回：\n\n{}",
            text
        );

        let response = self.call_openai_api(&prompt, config).await?;
        self.parse_key_info_response(&response).await
    }

    /// 智能分类
    async fn classify_content(
        &self,
        text: &str,
        topics: &[TopicTag],
        sentiment: &SentimentAnalysis,
    ) -> ContentClassification {
        // 基于主题和情感进行智能分类
        let mut suggested_categories = Vec::new();
        let mut auto_tags = Vec::new();

        // 根据主题生成分类建议
        for topic in topics {
            if topic.confidence > 0.7 {
                suggested_categories.push(CategorySuggestion {
                    category: format!("{:?}", topic.category),
                    confidence: topic.confidence,
                    reasoning: format!("基于主题 '{}' 的高置信度识别", topic.name),
                });
            }
            auto_tags.extend(topic.keywords.clone());
        }

        // 根据情感添加分类
        match sentiment.overall_sentiment {
            SentimentPolarity::Positive => auto_tags.push("积极".to_string()),
            SentimentPolarity::Negative => auto_tags.push("消极".to_string()),
            SentimentPolarity::Neutral => auto_tags.push("中性".to_string()),
            SentimentPolarity::Mixed => auto_tags.push("复杂情感".to_string()),
        }

        ContentClassification {
            suggested_categories,
            auto_tags,
            similar_content: Vec::new(),      // TODO: 实现相似内容检索
            knowledge_base_links: Vec::new(), // TODO: 实现知识库链接
        }
    }

    /// 调用OpenAI API
    async fn call_openai_api(
        &self,
        prompt: &str,
        config: &ContentAnalysisConfig,
    ) -> Result<String, AnalysisError> {
        let url = config
            .openai_config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com")
            .to_string()
            + "/v1/chat/completions";

        let request_body = serde_json::json!({
            "model": config.openai_config.model,
            "messages": [
                {
                    "role": "system",
                    "content": "你是一个专业的内容分析助手，擅长分析文本的主题、情感和关键信息。请始终以JSON格式返回结构化的分析结果。"
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": config.openai_config.temperature,
            "max_tokens": config.openai_config.max_tokens
        });

        let response = self
            .http_client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", config.openai_config.api_key),
            )
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AnalysisError::ApiError(format!("请求失败: {}", e)))?;

        if !response.status().is_success() {
            return Err(AnalysisError::ApiError(format!(
                "API错误: {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AnalysisError::ParseError(format!("响应解析失败: {}", e)))?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| AnalysisError::ParseError("无法获取响应内容".to_string()))?;

        Ok(content.to_string())
    }

    /// 解析主题响应
    async fn parse_topics_response(&self, response: &str) -> Result<Vec<TopicTag>, AnalysisError> {
        // 简化实现 - 在实际项目中应该使用更复杂的JSON解析
        Ok(vec![TopicTag {
            name: "示例主题".to_string(),
            confidence: 0.8,
            category: TopicCategory::Other("未分类".to_string()),
            keywords: vec!["关键词1".to_string(), "关键词2".to_string()],
            positions: vec![],
        }])
    }

    /// 解析情感响应
    async fn parse_sentiment_response(
        &self,
        response: &str,
    ) -> Result<SentimentAnalysis, AnalysisError> {
        Ok(SentimentAnalysis {
            overall_sentiment: SentimentPolarity::Neutral,
            intensity: 0.5,
            timeline: Vec::new(),
            tone_characteristics: ToneCharacteristics {
                formality: 0.7,
                confidence: 0.6,
                emotional_stability: 0.8,
                professionalism: 0.7,
            },
        })
    }

    /// 解析关键信息响应
    async fn parse_key_info_response(
        &self,
        response: &str,
    ) -> Result<KeyInformation, AnalysisError> {
        Ok(KeyInformation {
            summary: StructuredSummary {
                main_points: vec!["主要观点1".to_string(), "主要观点2".to_string()],
                conclusions: vec!["结论1".to_string()],
                questions_discussed: Vec::new(),
                solutions_mentioned: Vec::new(),
            },
            entities: Vec::new(),
            action_items: Vec::new(),
            data_points: Vec::new(),
        })
    }

    /// 生成缓存键
    fn generate_cache_key(&self, text: &str, content_id: Option<&str>) -> String {
        use sha2::{Digest, Sha256};

        let input = match content_id {
            Some(id) => format!("{}:{}", id, text),
            None => text.to_string(),
        };

        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 获取缓存结果
    async fn get_cached_result(&self, cache_key: &str) -> Option<ContentAnalysisResult> {
        self.analysis_cache.lock().await.get(cache_key).cloned()
    }

    /// 缓存结果
    async fn cache_result(&self, cache_key: &str, result: &ContentAnalysisResult) {
        self.analysis_cache
            .lock()
            .await
            .insert(cache_key.to_string(), result.clone());
    }

    /// 更新性能统计
    async fn update_performance_stats(
        &self,
        analysis_time_ms: u64,
        cache_hit: bool,
        api_calls: u32,
    ) {
        let mut stats = self.performance_stats.lock().await;
        stats.total_analyses += 1;

        if cache_hit {
            stats.cache_hits += 1;
        } else {
            stats.total_api_calls += api_calls as u64;
        }

        // 更新平均分析时间
        let total_time = stats.average_analysis_time_ms * (stats.total_analyses - 1) as f64
            + analysis_time_ms as f64;
        stats.average_analysis_time_ms = total_time / stats.total_analyses as f64;
    }

    /// 获取性能统计
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        self.performance_stats.lock().await.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: ContentAnalysisConfig) {
        *self.config.lock().await = new_config;
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        self.analysis_cache.lock().await.clear();
    }
}

/// 分析错误类型
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("API调用错误: {0}")]
    ApiError(String),

    #[error("响应解析错误: {0}")]
    ParseError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("超时错误")]
    TimeoutError,

    #[error("未知错误: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_content_analyzer_creation() {
        let config = ContentAnalysisConfig::default();
        let analyzer = ContentAnalyzer::new(config);

        let stats = analyzer.get_performance_stats().await;
        assert_eq!(stats.total_analyses, 0);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let config = ContentAnalysisConfig::default();
        let analyzer = ContentAnalyzer::new(config);

        let key1 = analyzer.generate_cache_key("test text", Some("id1"));
        let key2 = analyzer.generate_cache_key("test text", Some("id2"));
        let key3 = analyzer.generate_cache_key("different text", Some("id1"));

        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);
    }
}
