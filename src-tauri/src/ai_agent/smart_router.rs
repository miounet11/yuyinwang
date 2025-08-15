// 智能Agent路由器
// 基于上下文分析自动选择最优Agent链和执行策略

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use crate::errors::{AppError, AppResult};
use super::{AgentTask, PipelineStrategy};

/// 文本分析上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    /// 文本语言
    pub language: String,
    /// 文本长度
    pub text_length: usize,
    /// 内容类型（邮件、代码、文档等）
    pub content_type: ContentType,
    /// 用户意图
    pub user_intent: Vec<UserIntent>,
    /// 质量要求（速度优先 vs 质量优先）
    pub quality_preference: QualityPreference,
    /// 目标受众
    pub target_audience: Option<String>,
    /// 格式要求
    pub format_requirements: Vec<String>,
}

/// 内容类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    /// 普通文本
    PlainText,
    /// 邮件内容
    Email,
    /// 代码
    Code,
    /// 学术论文
    Academic,
    /// 商务文档
    Business,
    /// 创意写作
    Creative,
    /// 技术文档
    Technical,
    /// 聊天消息
    Chat,
    /// 新闻文章
    News,
    /// 社交媒体
    SocialMedia,
}

/// 用户意图
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserIntent {
    /// 语法增强
    GrammarImprovement,
    /// 语言翻译
    Translation,
    /// 内容摘要
    Summarization,
    /// 风格改写
    StyleRewriting,
    /// 长度调整
    LengthAdjustment,
    /// 格式转换
    FormatConversion,
    /// 语调修改
    ToneAdjustment,
    /// 专业术语优化
    TerminologyOptimization,
    /// 可读性提升
    ReadabilityImprovement,
    /// 创意增强
    CreativityEnhancement,
}

/// 质量偏好
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum QualityPreference {
    /// 速度优先
    Speed,
    /// 平衡
    Balanced,
    /// 质量优先
    Quality,
}

/// 路由建议结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRecommendation {
    /// 推荐的Agent任务链
    pub recommended_tasks: Vec<AgentTask>,
    /// 推荐的执行策略
    pub strategy: PipelineStrategy,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 预估执行时间
    pub estimated_duration: Duration,
    /// 预估成本（token数量）
    pub estimated_tokens: u32,
    /// 推荐理由
    pub reasoning: Vec<String>,
    /// 替代方案
    pub alternatives: Vec<AlternativeRoute>,
}

/// 替代路由方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeRoute {
    pub tasks: Vec<AgentTask>,
    pub strategy: PipelineStrategy,
    pub confidence: f64,
    pub trade_off: String, // 描述与主方案的权衡
}

/// 历史性能数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub context_fingerprint: String,
    pub tasks: Vec<String>, // Agent类型列表
    pub strategy: PipelineStrategy,
    pub execution_time: Duration,
    pub success_rate: f64,
    pub user_satisfaction: Option<f64>, // 0.0-1.0
    pub timestamp: u64,
}

/// 智能Agent路由器
#[derive(Debug)]
pub struct SmartAgentRouter {
    /// 内容分析器
    content_analyzer: Arc<ContentAnalyzer>,
    /// 性能历史数据
    performance_history: Arc<Mutex<Vec<PerformanceRecord>>>,
    /// 用户偏好学习
    user_preferences: Arc<Mutex<HashMap<String, QualityPreference>>>,
    /// 路由规则
    routing_rules: Arc<RoutingRuleEngine>,
}

impl SmartAgentRouter {
    /// 创建新的智能路由器
    pub fn new() -> Self {
        Self {
            content_analyzer: Arc::new(ContentAnalyzer::new()),
            performance_history: Arc::new(Mutex::new(Vec::new())),
            user_preferences: Arc::new(Mutex::new(HashMap::new())),
            routing_rules: Arc::new(RoutingRuleEngine::new()),
        }
    }
    
    /// 智能路由决策
    pub async fn route_request(
        &self,
        text: &str,
        user_context: Option<&str>,
    ) -> AppResult<RoutingRecommendation> {
        println!("🧠 开始智能Agent路由分析...");
        
        // 1. 分析文本内容
        let analysis_context = self.content_analyzer.analyze_content(text).await?;
        println!("📊 内容分析完成: {:?}", analysis_context.content_type);
        
        // 2. 获取用户偏好
        let user_preference = self.get_user_preference(user_context);
        
        // 3. 应用路由规则生成候选方案
        let candidates = self.routing_rules.generate_candidates(&analysis_context, user_preference).await?;
        
        // 4. 基于历史性能数据评分和排序
        let scored_candidates = self.score_candidates(&candidates, &analysis_context).await?;
        
        // 5. 选择最佳方案
        let best_candidate = scored_candidates.into_iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .ok_or_else(|| AppError::ValidationError("无法生成有效的路由建议".to_string()))?;
        
        println!("✅ 路由决策完成，置信度: {:.2}", best_candidate.confidence);
        
        Ok(best_candidate)
    }
    
    /// 学习用户反馈
    pub async fn learn_from_feedback(
        &self,
        context: &AnalysisContext,
        tasks: &[AgentTask],
        strategy: PipelineStrategy,
        execution_time: Duration,
        success_rate: f64,
        user_satisfaction: Option<f64>,
    ) -> AppResult<()> {
        let record = PerformanceRecord {
            context_fingerprint: self.generate_context_fingerprint(context),
            tasks: tasks.iter().map(|t| t.agent_type.clone()).collect(),
            strategy,
            execution_time,
            success_rate,
            user_satisfaction,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        self.performance_history.lock().push(record);
        
        // 清理过期数据（保留最近30天）
        let thirty_days_ago = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (30 * 24 * 3600);
        
        self.performance_history.lock().retain(|r| r.timestamp > thirty_days_ago);
        
        println!("📚 用户反馈已记录到学习系统");
        Ok(())
    }
    
    /// 获取用户偏好
    fn get_user_preference(&self, user_context: Option<&str>) -> QualityPreference {
        if let Some(context) = user_context {
            if let Some(&preference) = self.user_preferences.lock().get(context) {
                return preference;
            }
        }
        QualityPreference::Balanced // 默认平衡偏好
    }
    
    /// 为候选方案评分
    async fn score_candidates(
        &self,
        candidates: &[RoutingRecommendation],
        context: &AnalysisContext,
    ) -> AppResult<Vec<RoutingRecommendation>> {
        let context_fingerprint = self.generate_context_fingerprint(context);
        let history = self.performance_history.lock();
        
        let mut scored_candidates = Vec::new();
        
        for candidate in candidates {
            let mut score = candidate.confidence;
            
            // 基于历史性能调整评分
            let similar_records: Vec<_> = history
                .iter()
                .filter(|r| {
                    r.context_fingerprint == context_fingerprint ||
                    self.context_similarity(&r.context_fingerprint, &context_fingerprint) > 0.7
                })
                .collect();
            
            if !similar_records.is_empty() {
                let avg_success_rate: f64 = similar_records
                    .iter()
                    .map(|r| r.success_rate)
                    .sum::<f64>() / similar_records.len() as f64;
                
                let avg_satisfaction: f64 = similar_records
                    .iter()
                    .filter_map(|r| r.user_satisfaction)
                    .sum::<f64>() / similar_records.len() as f64;
                
                // 调整置信度
                score = score * 0.6 + avg_success_rate * 0.3 + avg_satisfaction * 0.1;
            }
            
            let mut enhanced_candidate = candidate.clone();
            enhanced_candidate.confidence = score;
            scored_candidates.push(enhanced_candidate);
        }
        
        // 按置信度降序排列
        scored_candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        Ok(scored_candidates)
    }
    
    /// 生成上下文指纹
    fn generate_context_fingerprint(&self, context: &AnalysisContext) -> String {
        format!("{}:{}:{}:{:?}",
                context.language,
                context.content_type.clone() as u8,
                context.text_length / 100, // 粗粒度长度分组
                context.quality_preference)
    }
    
    /// 计算上下文相似度
    fn context_similarity(&self, fp1: &str, fp2: &str) -> f64 {
        let parts1: Vec<&str> = fp1.split(':').collect();
        let parts2: Vec<&str> = fp2.split(':').collect();
        
        if parts1.len() != 4 || parts2.len() != 4 {
            return 0.0;
        }
        
        let mut similarity = 0.0;
        
        // 语言匹配
        if parts1[0] == parts2[0] {
            similarity += 0.4;
        }
        
        // 内容类型匹配
        if parts1[1] == parts2[1] {
            similarity += 0.3;
        }
        
        // 长度相似性
        if let (Ok(len1), Ok(len2)) = (parts1[2].parse::<i32>(), parts2[2].parse::<i32>()) {
            let len_diff = (len1 - len2).abs() as f64;
            similarity += 0.2 * (1.0 - len_diff.min(10.0) / 10.0);
        }
        
        // 质量偏好匹配
        if parts1[3] == parts2[3] {
            similarity += 0.1;
        }
        
        similarity
    }
    
    /// 获取性能统计
    pub fn get_performance_stats(&self) -> HashMap<String, f64> {
        let history = self.performance_history.lock();
        let mut stats = HashMap::new();
        
        if history.is_empty() {
            return stats;
        }
        
        let total_records = history.len() as f64;
        let avg_success_rate = history.iter().map(|r| r.success_rate).sum::<f64>() / total_records;
        let avg_satisfaction = history
            .iter()
            .filter_map(|r| r.user_satisfaction)
            .sum::<f64>() / total_records;
        
        stats.insert("total_decisions".to_string(), total_records);
        stats.insert("avg_success_rate".to_string(), avg_success_rate);
        stats.insert("avg_satisfaction".to_string(), avg_satisfaction);
        
        stats
    }
}

/// 内容分析器
#[derive(Debug)]
pub struct ContentAnalyzer {
    language_detector: Arc<LanguageDetector>,
    intent_classifier: Arc<IntentClassifier>,
    content_classifier: Arc<ContentTypeClassifier>,
}

impl ContentAnalyzer {
    pub fn new() -> Self {
        Self {
            language_detector: Arc::new(LanguageDetector::new()),
            intent_classifier: Arc::new(IntentClassifier::new()),
            content_classifier: Arc::new(ContentTypeClassifier::new()),
        }
    }
    
    /// 分析文本内容
    pub async fn analyze_content(&self, text: &str) -> AppResult<AnalysisContext> {
        // 基础信息
        let text_length = text.len();
        
        // 检测语言
        let language = self.language_detector.detect_language(text).await?;
        
        // 分类内容类型
        let content_type = self.content_classifier.classify_content(text).await?;
        
        // 识别用户意图
        let user_intent = self.intent_classifier.classify_intent(text).await?;
        
        // 默认偏好设置
        let quality_preference = QualityPreference::Balanced;
        
        Ok(AnalysisContext {
            language,
            text_length,
            content_type,
            user_intent,
            quality_preference,
            target_audience: None,
            format_requirements: Vec::new(),
        })
    }
}

/// 语言检测器
#[derive(Debug)]
pub struct LanguageDetector;

impl LanguageDetector {
    pub fn new() -> Self {
        Self
    }
    
    /// 检测文本语言
    pub async fn detect_language(&self, text: &str) -> AppResult<String> {
        // 简单的语言检测逻辑（实际项目中可以使用专门的库）
        let chinese_chars = text.chars().filter(|c| {
            (*c as u32) >= 0x4E00 && (*c as u32) <= 0x9FFF
        }).count();
        
        let total_chars = text.chars().count();
        
        if chinese_chars as f64 / total_chars as f64 > 0.3 {
            Ok("zh".to_string())
        } else {
            Ok("en".to_string())
        }
    }
}

/// 意图分类器
#[derive(Debug)]
pub struct IntentClassifier;

impl IntentClassifier {
    pub fn new() -> Self {
        Self
    }
    
    /// 分类用户意图
    pub async fn classify_intent(&self, text: &str) -> AppResult<Vec<UserIntent>> {
        let mut intents = Vec::new();
        let text_lower = text.to_lowercase();
        
        // 基于关键词的简单意图识别
        if text_lower.contains("translate") || text_lower.contains("翻译") {
            intents.push(UserIntent::Translation);
        }
        
        if text_lower.contains("summary") || text_lower.contains("摘要") || text_lower.contains("summarize") {
            intents.push(UserIntent::Summarization);
        }
        
        if text_lower.contains("improve") || text_lower.contains("enhance") || text_lower.contains("优化") {
            intents.push(UserIntent::GrammarImprovement);
        }
        
        if text_lower.contains("professional") || text_lower.contains("business") || text_lower.contains("formal") {
            intents.push(UserIntent::ToneAdjustment);
        }
        
        // 默认意图
        if intents.is_empty() {
            intents.push(UserIntent::GrammarImprovement);
        }
        
        Ok(intents)
    }
}

/// 内容类型分类器
#[derive(Debug)]
pub struct ContentTypeClassifier;

impl ContentTypeClassifier {
    pub fn new() -> Self {
        Self
    }
    
    /// 分类内容类型
    pub async fn classify_content(&self, text: &str) -> AppResult<ContentType> {
        let text_lower = text.to_lowercase();
        
        // 基于内容特征的简单分类
        if text_lower.contains("dear") || text_lower.contains("sincerely") || 
           text_lower.contains("best regards") || text_lower.contains("您好") {
            return Ok(ContentType::Email);
        }
        
        if text_lower.contains("function") || text_lower.contains("class") || 
           text_lower.contains("import") || text_lower.contains("def ") {
            return Ok(ContentType::Code);
        }
        
        if text_lower.contains("abstract") || text_lower.contains("introduction") || 
           text_lower.contains("methodology") || text_lower.contains("论文") {
            return Ok(ContentType::Academic);
        }
        
        if text_lower.contains("meeting") || text_lower.contains("agenda") || 
           text_lower.contains("proposal") || text_lower.contains("商务") {
            return Ok(ContentType::Business);
        }
        
        // 默认类型
        Ok(ContentType::PlainText)
    }
}

/// 路由规则引擎
#[derive(Debug)]
pub struct RoutingRuleEngine;

impl RoutingRuleEngine {
    pub fn new() -> Self {
        Self
    }
    
    /// 生成候选路由方案
    pub async fn generate_candidates(
        &self,
        context: &AnalysisContext,
        user_preference: QualityPreference,
    ) -> AppResult<Vec<RoutingRecommendation>> {
        let mut candidates = Vec::new();
        
        // 根据内容类型和用户意图生成方案
        for intent in &context.user_intent {
            match intent {
                UserIntent::GrammarImprovement => {
                    candidates.push(self.create_enhancement_route(context, user_preference));
                }
                UserIntent::Translation => {
                    candidates.push(self.create_translation_route(context, user_preference));
                }
                UserIntent::Summarization => {
                    candidates.push(self.create_summary_route(context, user_preference));
                }
                UserIntent::StyleRewriting => {
                    candidates.push(self.create_style_rewrite_route(context, user_preference));
                }
                _ => {} // 其他意图的路由规则
            }
        }
        
        // 如果没有生成候选方案，创建默认方案
        if candidates.is_empty() {
            candidates.push(self.create_default_route(context, user_preference));
        }
        
        Ok(candidates)
    }
    
    /// 创建增强路由
    fn create_enhancement_route(
        &self,
        context: &AnalysisContext,
        preference: QualityPreference,
    ) -> RoutingRecommendation {
        let task = AgentTask::new("enhance", "");
        
        let (confidence, strategy, duration, tokens) = match preference {
            QualityPreference::Speed => (0.8, PipelineStrategy::Parallel, Duration::from_secs(5), 200),
            QualityPreference::Balanced => (0.9, PipelineStrategy::Smart, Duration::from_secs(8), 300),
            QualityPreference::Quality => (0.95, PipelineStrategy::Sequential, Duration::from_secs(12), 500),
        };
        
        RoutingRecommendation {
            recommended_tasks: vec![task],
            strategy,
            confidence,
            estimated_duration: duration,
            estimated_tokens: tokens,
            reasoning: vec![
                format!("检测到{}内容类型", match context.content_type {
                    ContentType::Email => "邮件",
                    ContentType::Business => "商务",
                    _ => "通用",
                }),
                format!("用户偏好: {:?}", preference),
            ],
            alternatives: Vec::new(),
        }
    }
    
    /// 创建翻译路由
    fn create_translation_route(
        &self,
        _context: &AnalysisContext,
        preference: QualityPreference,
    ) -> RoutingRecommendation {
        let mut tasks = vec![AgentTask::new("translate", "")];
        
        // 高质量模式添加后处理
        if preference == QualityPreference::Quality {
            tasks.push(AgentTask::new("enhance", "").with_dependency(&tasks[0].id));
        }
        
        let confidence = match preference {
            QualityPreference::Speed => 0.85,
            QualityPreference::Balanced => 0.9,
            QualityPreference::Quality => 0.95,
        };
        
        RoutingRecommendation {
            recommended_tasks: tasks,
            strategy: PipelineStrategy::Sequential,
            confidence,
            estimated_duration: Duration::from_secs(10),
            estimated_tokens: 400,
            reasoning: vec!["检测到翻译需求".to_string()],
            alternatives: Vec::new(),
        }
    }
    
    /// 创建摘要路由
    fn create_summary_route(
        &self,
        context: &AnalysisContext,
        preference: QualityPreference,
    ) -> RoutingRecommendation {
        let task = AgentTask::new("summarize", "");
        
        // 长文本使用并行策略
        let strategy = if context.text_length > 5000 {
            PipelineStrategy::Parallel
        } else {
            PipelineStrategy::Sequential
        };
        
        let confidence = match preference {
            QualityPreference::Speed => 0.8,
            QualityPreference::Balanced => 0.85,
            QualityPreference::Quality => 0.9,
        };
        
        RoutingRecommendation {
            recommended_tasks: vec![task],
            strategy,
            confidence,
            estimated_duration: Duration::from_secs(6),
            estimated_tokens: 250,
            reasoning: vec!["检测到摘要需求".to_string()],
            alternatives: Vec::new(),
        }
    }
    
    /// 创建风格改写路由
    fn create_style_rewrite_route(
        &self,
        _context: &AnalysisContext,
        preference: QualityPreference,
    ) -> RoutingRecommendation {
        let tasks = vec![
            AgentTask::new("enhance", ""),
            AgentTask::new("email", "").with_dependency("enhance_task_id"),
        ];
        
        let confidence = match preference {
            QualityPreference::Speed => 0.7,
            QualityPreference::Balanced => 0.8,
            QualityPreference::Quality => 0.9,
        };
        
        RoutingRecommendation {
            recommended_tasks: tasks,
            strategy: PipelineStrategy::Sequential,
            confidence,
            estimated_duration: Duration::from_secs(15),
            estimated_tokens: 600,
            reasoning: vec!["检测到风格改写需求".to_string()],
            alternatives: Vec::new(),
        }
    }
    
    /// 创建默认路由
    fn create_default_route(
        &self,
        _context: &AnalysisContext,
        _preference: QualityPreference,
    ) -> RoutingRecommendation {
        let task = AgentTask::new("enhance", "");
        
        RoutingRecommendation {
            recommended_tasks: vec![task],
            strategy: PipelineStrategy::Smart,
            confidence: 0.7,
            estimated_duration: Duration::from_secs(8),
            estimated_tokens: 300,
            reasoning: vec!["使用默认增强处理".to_string()],
            alternatives: Vec::new(),
        }
    }
}