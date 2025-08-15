// AI Agent Pipeline 管理器
// 实现智能链式处理、并发执行和错误恢复机制

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use tokio::sync::{mpsc, Semaphore};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::errors::{AppError, AppResult};
use super::processors::AIProcessor;
use super::types::{AIAgentType, AIAgentRequest};

/// Pipeline执行策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PipelineStrategy {
    /// 顺序执行
    Sequential,
    /// 并行执行
    Parallel,
    /// 智能选择（根据任务类型和资源情况自动决定）
    Smart,
}

/// Agent任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub agent_type: String,
    pub input_text: String,
    pub config: HashMap<String, String>,
    pub dependencies: Vec<String>, // 依赖的其他任务ID
    pub priority: u8, // 0-255，数值越高优先级越高
    pub timeout_seconds: Option<u64>,
    pub retry_count: u8,
}

impl AgentTask {
    pub fn new(agent_type: &str, input_text: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_type: agent_type.to_string(),
            input_text: input_text.to_string(),
            config: HashMap::new(),
            dependencies: Vec::new(),
            priority: 128, // 默认中等优先级
            timeout_seconds: Some(30),
            retry_count: 3,
        }
    }
    
    pub fn with_dependency(mut self, task_id: &str) -> Self {
        self.dependencies.push(task_id.to_string());
        self
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }
}

/// Pipeline执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub pipeline_id: String,
    pub tasks: Vec<TaskResult>,
    pub execution_time: Duration,
    pub total_tokens_used: u32,
    pub success_rate: f64,
    pub errors: Vec<String>,
    pub performance_stats: PerformanceStats,
}

impl PipelineResult {
    pub fn is_successful(&self) -> bool {
        self.success_rate >= 0.8 // 80%成功率认为是成功
    }
    
    pub fn get_final_output(&self) -> Option<&str> {
        // 返回最后一个成功任务的输出
        self.tasks
            .iter()
            .filter(|t| t.success)
            .last()
            .map(|t| t.output.as_str())
    }
}

/// 单个任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_type: String,
    pub success: bool,
    pub output: String,
    pub execution_time: Duration,
    pub tokens_used: u32,
    pub error: Option<String>,
    pub retry_attempts: u8,
}

/// 性能统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub avg_task_duration: Duration,
    pub max_concurrent_tasks: u32,
    pub memory_usage_mb: f64,
    pub api_calls_count: u32,
    pub cache_hit_rate: f64,
}

/// Pipeline执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub pipeline_id: String,
    pub start_time: Instant,
    pub strategy: PipelineStrategy,
    pub max_concurrent_tasks: usize,
    pub task_results: Arc<Mutex<HashMap<String, TaskResult>>>,
    pub execution_stats: Arc<Mutex<PerformanceStats>>,
}

/// AI Agent Pipeline管理器
#[derive(Debug)]
pub struct AgentPipelineManager {
    // AI处理器实例
    ai_processor: Arc<AIProcessor>,
    
    // 执行控制
    semaphore: Arc<Semaphore>,
    max_concurrent_tasks: usize,
    
    // 缓存和性能监控
    result_cache: Arc<Mutex<HashMap<String, String>>>,
    performance_history: Arc<Mutex<Vec<PerformanceStats>>>,
    
    // 配置
    default_timeout: Duration,
    enable_caching: bool,
}

impl AgentPipelineManager {
    /// 创建新的Pipeline管理器
    pub async fn new(
        ai_processor: Arc<AIProcessor>,
        max_concurrent_tasks: usize,
    ) -> AppResult<Self> {
        Ok(Self {
            ai_processor,
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            max_concurrent_tasks,
            result_cache: Arc::new(Mutex::new(HashMap::new())),
            performance_history: Arc::new(Mutex::new(Vec::new())),
            default_timeout: Duration::from_secs(30),
            enable_caching: true,
        })
    }
    
    /// 执行Pipeline
    pub async fn execute_pipeline(
        &self,
        tasks: Vec<AgentTask>,
        strategy: PipelineStrategy,
    ) -> AppResult<PipelineResult> {
        let pipeline_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();
        
        println!("🚀 开始执行Pipeline: {} (策略: {:?})", pipeline_id, strategy);
        
        // 创建执行上下文
        let context = ExecutionContext {
            pipeline_id: pipeline_id.clone(),
            start_time,
            strategy,
            max_concurrent_tasks: self.max_concurrent_tasks,
            task_results: Arc::new(Mutex::new(HashMap::new())),
            execution_stats: Arc::new(Mutex::new(PerformanceStats {
                avg_task_duration: Duration::from_millis(0),
                max_concurrent_tasks: 0,
                memory_usage_mb: 0.0,
                api_calls_count: 0,
                cache_hit_rate: 0.0,
            })),
        };
        
        // 验证和排序任务
        let validated_tasks = self.validate_and_sort_tasks(tasks)?;
        
        // 根据策略执行任务
        let task_results = match strategy {
            PipelineStrategy::Sequential => {
                self.execute_sequential(&context, validated_tasks).await?
            }
            PipelineStrategy::Parallel => {
                self.execute_parallel(&context, validated_tasks).await?
            }
            PipelineStrategy::Smart => {
                self.execute_smart(&context, validated_tasks).await?
            }
        };
        
        // 构建结果
        let execution_time = start_time.elapsed();
        let success_count = task_results.iter().filter(|r| r.success).count();
        let success_rate = success_count as f64 / task_results.len() as f64;
        
        let pipeline_result = PipelineResult {
            pipeline_id,
            tasks: task_results.clone(),
            execution_time,
            total_tokens_used: task_results.iter().map(|t| t.tokens_used).sum(),
            success_rate,
            errors: task_results
                .iter()
                .filter_map(|t| t.error.clone())
                .collect(),
            performance_stats: context.execution_stats.lock().clone(),
        };
        
        // 更新性能历史
        self.performance_history.lock().push(pipeline_result.performance_stats.clone());
        
        println!("✅ Pipeline执行完成: {:.2}%成功率, 耗时: {:?}", 
                 success_rate * 100.0, execution_time);
        
        Ok(pipeline_result)
    }
    
    /// 验证和排序任务（按依赖关系和优先级）
    fn validate_and_sort_tasks(&self, tasks: Vec<AgentTask>) -> AppResult<Vec<AgentTask>> {
        // 检查循环依赖
        self.check_circular_dependencies(&tasks)?;
        
        // 拓扑排序（确保依赖任务先执行）
        let mut sorted_tasks = Vec::new();
        let mut remaining_tasks: Vec<AgentTask> = tasks;
        let mut task_map: HashMap<String, AgentTask> = HashMap::new();
        
        // 建立任务映射
        for task in &remaining_tasks {
            task_map.insert(task.id.clone(), task.clone());
        }
        
        while !remaining_tasks.is_empty() {
            let mut progress = false;
            let mut i = 0;
            
            while i < remaining_tasks.len() {
                let task = &remaining_tasks[i];
                
                // 检查所有依赖是否已完成
                let dependencies_satisfied = task.dependencies
                    .iter()
                    .all(|dep_id| {
                        sorted_tasks.iter().any(|t: &AgentTask| &t.id == dep_id)
                    });
                
                if dependencies_satisfied {
                    sorted_tasks.push(remaining_tasks.remove(i));
                    progress = true;
                } else {
                    i += 1;
                }
            }
            
            if !progress {
                return Err(AppError::ValidationError(
                    "检测到循环依赖或无效的依赖关系".to_string()
                ));
            }
        }
        
        // 在相同依赖层级内按优先级排序
        sorted_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(sorted_tasks)
    }
    
    /// 检查循环依赖
    fn check_circular_dependencies(&self, tasks: &[AgentTask]) -> AppResult<()> {
        // 构建依赖图
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for task in tasks {
            graph.insert(task.id.clone(), task.dependencies.clone());
        }
        
        // DFS检测循环
        for task_id in graph.keys() {
            let mut visited = std::collections::HashSet::new();
            if self.has_cycle(&graph, task_id, &mut visited) {
                return Err(AppError::ValidationError(
                    format!("检测到循环依赖，涉及任务: {}", task_id)
                ));
            }
        }
        
        Ok(())
    }
    
    /// DFS循环检测
    fn has_cycle(
        &self,
        graph: &HashMap<String, Vec<String>>,
        node: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> bool {
        if visited.contains(node) {
            return true;
        }
        
        visited.insert(node.to_string());
        
        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if self.has_cycle(graph, dep, visited) {
                    return true;
                }
            }
        }
        
        visited.remove(node);
        false
    }
    
    /// 顺序执行任务
    async fn execute_sequential(
        &self,
        context: &ExecutionContext,
        tasks: Vec<AgentTask>,
    ) -> AppResult<Vec<TaskResult>> {
        let mut results = Vec::new();
        
        for task in tasks {
            let result = self.execute_single_task(context, &task).await;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 并行执行任务
    async fn execute_parallel(
        &self,
        context: &ExecutionContext,
        tasks: Vec<AgentTask>,
    ) -> AppResult<Vec<TaskResult>> {
        let mut handles = Vec::new();
        let (tx, mut rx) = mpsc::unbounded_channel::<TaskResult>();
        
        // 启动所有任务
        for task in tasks {
            let context_clone = context.clone();
            let tx = tx.clone();
            let self_clone = self.clone();
            
            let handle = tokio::spawn(async move {
                let result = self_clone.execute_single_task(&context_clone, &task).await;
                let _ = tx.send(result);
            });
            
            handles.push(handle);
        }
        
        drop(tx); // 关闭发送端
        
        // 收集结果
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }
        
        // 等待所有任务完成
        for handle in handles {
            let _ = handle.await;
        }
        
        // 按任务ID排序结果
        results.sort_by(|a, b| a.task_id.cmp(&b.task_id));
        
        Ok(results)
    }
    
    /// 智能执行策略
    async fn execute_smart(
        &self,
        context: &ExecutionContext,
        tasks: Vec<AgentTask>,
    ) -> AppResult<Vec<TaskResult>> {
        // 分析任务特征并决定执行策略
        let analysis = self.analyze_tasks(&tasks);
        
        if analysis.has_dependencies {
            // 有依赖关系时使用分层并行执行
            self.execute_layered_parallel(context, tasks).await
        } else if analysis.total_estimated_time > Duration::from_secs(10) {
            // 预计耗时较长时使用并行执行
            self.execute_parallel(context, tasks).await
        } else {
            // 简单任务使用顺序执行
            self.execute_sequential(context, tasks).await
        }
    }
    
    /// 分层并行执行（按依赖层级分组并行）
    async fn execute_layered_parallel(
        &self,
        context: &ExecutionContext,
        tasks: Vec<AgentTask>,
    ) -> AppResult<Vec<TaskResult>> {
        let mut all_results = Vec::new();
        let mut remaining_tasks = tasks;
        
        while !remaining_tasks.is_empty() {
            // 找出当前可以执行的任务（无未完成的依赖）
            let mut ready_tasks = Vec::new();
            let mut i = 0;
            
            while i < remaining_tasks.len() {
                let task = &remaining_tasks[i];
                let dependencies_satisfied = task.dependencies
                    .iter()
                    .all(|dep_id| {
                        all_results.iter().any(|r: &TaskResult| &r.task_id == dep_id && r.success)
                    });
                
                if dependencies_satisfied {
                    ready_tasks.push(remaining_tasks.remove(i));
                } else {
                    i += 1;
                }
            }
            
            if ready_tasks.is_empty() {
                break; // 避免无限循环
            }
            
            // 并行执行当前层的任务
            let layer_results = self.execute_parallel(context, ready_tasks).await?;
            all_results.extend(layer_results);
        }
        
        Ok(all_results)
    }
    
    /// 执行单个任务
    async fn execute_single_task(
        &self,
        _context: &ExecutionContext,
        task: &AgentTask,
    ) -> TaskResult {
        let start_time = Instant::now();
        let mut retry_attempts = 0;
        
        // 检查缓存
        if self.enable_caching {
            let cache_key = format!("{}:{}", task.agent_type, task.input_text);
            if let Some(cached_result) = self.result_cache.lock().get(&cache_key) {
                return TaskResult {
                    task_id: task.id.clone(),
                    agent_type: task.agent_type.clone(),
                    success: true,
                    output: cached_result.clone(),
                    execution_time: Duration::from_millis(1), // 缓存命中
                    tokens_used: 0,
                    error: None,
                    retry_attempts: 0,
                };
            }
        }
        
        // 执行任务（带重试机制）
        while retry_attempts <= task.retry_count {
            // 获取信号量许可
            let _permit = self.semaphore.acquire().await.unwrap();
            
            // 创建AI代理请求
            let agent_type = match task.agent_type.as_str() {
                "enhance" => AIAgentType::TextEnhancement,
                "translate" => AIAgentType::Translation,
                "summarize" => AIAgentType::Summarization,
                "email" => AIAgentType::Custom,
                "code_comment" => AIAgentType::Custom,
                _ => return TaskResult {
                    task_id: task.id.clone(),
                    agent_type: task.agent_type.clone(),
                    success: false,
                    output: String::new(),
                    execution_time: start_time.elapsed(),
                    tokens_used: 0,
                    error: Some(format!("未知的Agent类型: {}", task.agent_type)),
                    retry_attempts,
                },
            };
            
            let ai_request = AIAgentRequest {
                text: task.input_text.clone(),
                agent_type,
                options: task.config.clone(),
                context: None,
            };
            
            let result = self.ai_processor.process(ai_request).await
                .map(|response| response.processed_text);
            
            let execution_time = start_time.elapsed();
            
            match result {
                Ok(output) => {
                    // 缓存成功结果
                    if self.enable_caching {
                        let cache_key = format!("{}:{}", task.agent_type, task.input_text);
                        self.result_cache.lock().insert(cache_key, output.clone());
                    }
                    
                    return TaskResult {
                        task_id: task.id.clone(),
                        agent_type: task.agent_type.clone(),
                        success: true,
                        output,
                        execution_time,
                        tokens_used: 100, // 估算值，实际应从API响应获取
                        error: None,
                        retry_attempts,
                    };
                }
                Err(e) => {
                    retry_attempts += 1;
                    if retry_attempts > task.retry_count {
                        return TaskResult {
                            task_id: task.id.clone(),
                            agent_type: task.agent_type.clone(),
                            success: false,
                            output: String::new(),
                            execution_time,
                            tokens_used: 0,
                            error: Some(e.to_string()),
                            retry_attempts,
                        };
                    }
                    
                    // 重试前等待
                    tokio::time::sleep(Duration::from_millis(1000 * retry_attempts as u64)).await;
                }
            }
        }
        
        // 不应该到达这里
        unreachable!()
    }
    
    /// 分析任务特征
    fn analyze_tasks(&self, tasks: &[AgentTask]) -> TaskAnalysis {
        let has_dependencies = tasks.iter().any(|t| !t.dependencies.is_empty());
        let total_estimated_time = Duration::from_secs(tasks.len() as u64 * 3); // 每任务估算3秒
        let complexity_score = tasks.len() as f64 + 
                              tasks.iter().map(|t| t.dependencies.len()).sum::<usize>() as f64;
        
        TaskAnalysis {
            has_dependencies,
            total_estimated_time,
            complexity_score,
            parallelizable_count: tasks.iter().filter(|t| t.dependencies.is_empty()).count(),
        }
    }
    
    /// 清理缓存
    pub fn clear_cache(&self) {
        self.result_cache.lock().clear();
        println!("🧹 Agent Pipeline缓存已清理");
    }
    
    /// 获取性能统计
    pub fn get_performance_history(&self) -> Vec<PerformanceStats> {
        self.performance_history.lock().clone()
    }
}

/// 为支持clone而实现的Clone trait
impl Clone for AgentPipelineManager {
    fn clone(&self) -> Self {
        Self {
            ai_processor: self.ai_processor.clone(),
            semaphore: Arc::new(Semaphore::new(self.max_concurrent_tasks)),
            max_concurrent_tasks: self.max_concurrent_tasks,
            result_cache: self.result_cache.clone(),
            performance_history: self.performance_history.clone(),
            default_timeout: self.default_timeout,
            enable_caching: self.enable_caching,
        }
    }
}

/// 任务分析结果
#[derive(Debug)]
struct TaskAnalysis {
    has_dependencies: bool,
    total_estimated_time: Duration,
    complexity_score: f64,
    parallelizable_count: usize,
}

/// Pipeline构建器（便于创建复杂Pipeline）
#[derive(Debug)]
pub struct PipelineBuilder {
    tasks: Vec<AgentTask>,
    strategy: PipelineStrategy,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            strategy: PipelineStrategy::Smart,
        }
    }
    
    pub fn add_task(mut self, task: AgentTask) -> Self {
        self.tasks.push(task);
        self
    }
    
    pub fn with_strategy(mut self, strategy: PipelineStrategy) -> Self {
        self.strategy = strategy;
        self
    }
    
    pub fn add_enhancement_task(mut self, text: &str) -> Self {
        self.tasks.push(AgentTask::new("enhance", text));
        self
    }
    
    pub fn add_translation_task(mut self, text: &str, target_lang: &str) -> Self {
        let task = AgentTask::new("translate", text)
            .with_config("target_language", target_lang);
        self.tasks.push(task);
        self
    }
    
    pub fn add_summary_task(mut self, text: &str) -> Self {
        self.tasks.push(AgentTask::new("summarize", text));
        self
    }
    
    pub async fn execute(self, manager: &AgentPipelineManager) -> AppResult<PipelineResult> {
        manager.execute_pipeline(self.tasks, self.strategy).await
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}