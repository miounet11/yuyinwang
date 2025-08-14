// 历史记录管理器 - 高级浏览、搜索和批量管理功能
// 提供完整的转录历史记录管理解决方案

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::errors::{AppError, AppResult};
use crate::types::TranscriptionEntry;
use super::{DatabaseManager, SearchFilter, SearchResult, Tag};

/// 高级搜索选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSearchOptions {
    /// 基础搜索过滤器
    pub filter: SearchFilter,
    /// 排序选项
    pub sort_by: SortOption,
    /// 排序方向
    pub sort_order: SortOrder,
    /// 分页大小
    pub page_size: usize,
    /// 页码（从0开始）
    pub page: usize,
    /// 是否包含已删除的记录
    pub include_deleted: bool,
    /// 全文搜索选项
    pub full_text_search: Option<FullTextSearchOptions>,
    /// 分组选项
    pub group_by: Option<GroupByOption>,
}

/// 排序选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOption {
    Timestamp,
    Duration,
    Confidence,
    Model,
    TextLength,
    CreatedAt,
    UpdatedAt,
    Relevance, // 用于全文搜索
}

/// 排序方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// 全文搜索选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTextSearchOptions {
    /// 搜索查询
    pub query: String,
    /// 是否启用模糊搜索
    pub fuzzy_search: bool,
    /// 高亮匹配文本
    pub highlight: bool,
    /// 搜索范围
    pub search_fields: Vec<SearchField>,
    /// 最小匹配度
    pub min_score: f64,
}

/// 搜索字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchField {
    Text,
    Tags,
    Metadata,
    AudioFileName,
}

/// 分组选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupByOption {
    Date,        // 按日期分组
    Model,       // 按模型分组
    Duration,    // 按时长分组
    Confidence,  // 按置信度分组
    Tags,        // 按标签分组
}

/// 批量操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulkOperation {
    Delete,
    Archive,
    Unarchive,
    AddTag { tag: String },
    RemoveTag { tag: String },
    UpdateModel { new_model: String },
    Export { format: ExportFormat },
}

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Txt,
    Srt, // 字幕格式
    Docx,
}

/// 批量操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResult {
    pub operation: BulkOperation,
    pub total_items: usize,
    pub successful_items: usize,
    pub failed_items: usize,
    pub errors: Vec<String>,
    pub execution_time_ms: u64,
}

/// 分组搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedSearchResult {
    pub groups: HashMap<String, Vec<TranscriptionEntry>>,
    pub total_count: usize,
    pub group_counts: HashMap<String, usize>,
}

/// 历史记录统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatistics {
    pub total_entries: usize,
    pub total_duration_hours: f64,
    pub average_confidence: f64,
    pub most_used_models: Vec<(String, usize)>,
    pub entries_by_date: HashMap<String, usize>, // YYYY-MM-DD
    pub entries_by_hour: HashMap<u8, usize>,     // 0-23
    pub tag_usage: HashMap<String, usize>,
    pub recent_activity: Vec<RecentActivity>,
}

/// 最近活动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivity {
    pub activity_type: ActivityType,
    pub entry_id: String,
    pub timestamp: i64,
    pub description: String,
}

/// 活动类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    Created,
    Updated,
    Deleted,
    Tagged,
    Exported,
}

/// 智能建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartSuggestions {
    pub suggested_tags: Vec<String>,
    pub similar_entries: Vec<String>, // Entry IDs
    pub cleanup_suggestions: Vec<CleanupSuggestion>,
}

/// 清理建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupSuggestion {
    pub suggestion_type: CleanupType,
    pub affected_entries: Vec<String>,
    pub description: String,
    pub potential_savings: String, // 例如 "节省 50MB 存储空间"
}

/// 清理类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupType {
    DuplicateText,
    LowConfidenceEntries,
    OldUntaggedEntries,
    LargeAudioFiles,
    BrokenAudioLinks,
}

/// 高级历史记录管理器
#[derive(Debug)]
pub struct HistoryManager {
    db_manager: Arc<DatabaseManager>,
    search_cache: Arc<Mutex<HashMap<String, SearchResult>>>,
    recent_activities: Arc<Mutex<Vec<RecentActivity>>>,
    max_cache_size: usize,
    max_recent_activities: usize,
}

impl Default for AdvancedSearchOptions {
    fn default() -> Self {
        Self {
            filter: SearchFilter::default(),
            sort_by: SortOption::Timestamp,
            sort_order: SortOrder::Descending,
            page_size: 50,
            page: 0,
            include_deleted: false,
            full_text_search: None,
            group_by: None,
        }
    }
}

impl HistoryManager {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            db_manager,
            search_cache: Arc::new(Mutex::new(HashMap::new())),
            recent_activities: Arc::new(Mutex::new(Vec::new())),
            max_cache_size: 100,
            max_recent_activities: 1000,
        }
    }

    /// 高级搜索功能
    pub async fn advanced_search(
        &self,
        options: &AdvancedSearchOptions,
    ) -> AppResult<SearchResult> {
        // 生成缓存键
        let cache_key = self.generate_cache_key(options);
        
        // 检查缓存
        if let Some(cached_result) = self.search_cache.lock().get(&cache_key) {
            return Ok(cached_result.clone());
        }

        // 构建搜索参数
        let limit = Some(options.page_size);
        let offset = Some(options.page * options.page_size);
        
        // 执行基础搜索
        let query = if let Some(full_text) = &options.full_text_search {
            &full_text.query
        } else {
            ""
        };
        
        let mut result = self.db_manager.search_transcriptions(
            query, 
            &options.filter, 
            limit, 
            offset
        )?;

        // 应用高级搜索功能
        if let Some(full_text_opts) = &options.full_text_search {
            result = self.apply_full_text_search(result, full_text_opts).await?;
        }

        // 应用排序
        self.apply_sorting(&mut result, &options.sort_by, &options.sort_order);

        // 应用高亮
        if let Some(full_text_opts) = &options.full_text_search {
            if full_text_opts.highlight {
                self.apply_highlighting(&mut result, &full_text_opts.query);
            }
        }

        // 缓存结果
        self.cache_search_result(cache_key, result.clone());

        println!("🔍 高级搜索完成: 找到 {} 个结果", result.entries.len());
        Ok(result)
    }

    /// 分组搜索
    pub async fn grouped_search(
        &self,
        options: &AdvancedSearchOptions,
    ) -> AppResult<GroupedSearchResult> {
        if options.group_by.is_none() {
            return Err(AppError::ValidationError("分组选项未指定".to_string()));
        }

        // 获取所有匹配的记录（不分页）
        let all_options = AdvancedSearchOptions {
            page_size: 10000, // 获取大量结果
            page: 0,
            ..options.clone()
        };
        
        let search_result = self.advanced_search(&all_options).await?;
        let group_by = options.group_by.as_ref().unwrap();

        let mut groups: HashMap<String, Vec<TranscriptionEntry>> = HashMap::new();
        let mut group_counts: HashMap<String, usize> = HashMap::new();

        for entry in search_result.entries {
            let group_key = self.get_group_key(&entry, group_by);
            
            let group_entries = groups.entry(group_key.clone()).or_insert_with(Vec::new);
            group_entries.push(entry);
            
            *group_counts.entry(group_key).or_insert(0) += 1;
        }

        println!("📊 分组搜索完成: {} 个分组", groups.len());
        
        Ok(GroupedSearchResult {
            groups,
            total_count: search_result.total_count,
            group_counts,
        })
    }

    /// 批量操作
    pub async fn bulk_operation(
        &self,
        entry_ids: &[String],
        operation: BulkOperation,
    ) -> AppResult<BulkOperationResult> {
        let start_time = SystemTime::now();
        let mut successful_items = 0;
        let mut failed_items = 0;
        let mut errors = Vec::new();

        for entry_id in entry_ids {
            match self.execute_single_operation(entry_id, &operation).await {
                Ok(_) => {
                    successful_items += 1;
                    self.record_activity(ActivityType::Updated, entry_id.clone(), "批量操作".to_string());
                }
                Err(e) => {
                    failed_items += 1;
                    errors.push(format!("条目 {}: {}", entry_id, e));
                }
            }
        }

        let execution_time_ms = start_time.elapsed()
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        println!("⚡ 批量操作完成: {}/{} 成功", successful_items, entry_ids.len());

        Ok(BulkOperationResult {
            operation,
            total_items: entry_ids.len(),
            successful_items,
            failed_items,
            errors,
            execution_time_ms,
        })
    }

    /// 获取历史统计
    pub async fn get_history_statistics(&self) -> AppResult<HistoryStatistics> {
        let all_entries = self.db_manager.get_all_transcriptions()?;
        
        let total_entries = all_entries.len();
        let total_duration_hours: f64 = all_entries.iter().map(|e| e.duration).sum::<f64>() / 3600.0;
        let average_confidence = if total_entries > 0 {
            all_entries.iter().map(|e| e.confidence).sum::<f64>() / total_entries as f64
        } else {
            0.0
        };

        // 模型使用统计
        let mut model_counts: HashMap<String, usize> = HashMap::new();
        for entry in &all_entries {
            *model_counts.entry(entry.model.clone()).or_insert(0) += 1;
        }
        let mut most_used_models: Vec<(String, usize)> = model_counts.into_iter().collect();
        most_used_models.sort_by(|a, b| b.1.cmp(&a.1));
        most_used_models.truncate(10);

        // 按日期统计
        let mut entries_by_date: HashMap<String, usize> = HashMap::new();
        let mut entries_by_hour: HashMap<u8, usize> = HashMap::new();
        
        for entry in &all_entries {
            // 转换时间戳为日期
            let date = self.timestamp_to_date(entry.timestamp);
            let hour = self.timestamp_to_hour(entry.timestamp);
            
            *entries_by_date.entry(date).or_insert(0) += 1;
            *entries_by_hour.entry(hour).or_insert(0) += 1;
        }

        // 标签使用统计
        let tag_usage = self.calculate_tag_usage(&all_entries);

        // 最近活动
        let recent_activity = self.recent_activities.lock().clone();

        Ok(HistoryStatistics {
            total_entries,
            total_duration_hours,
            average_confidence,
            most_used_models,
            entries_by_date,
            entries_by_hour,
            tag_usage,
            recent_activity,
        })
    }

    /// 获取智能建议
    pub async fn get_smart_suggestions(&self, entry_id: Option<&str>) -> AppResult<SmartSuggestions> {
        let all_entries = self.db_manager.get_all_transcriptions()?;

        // 建议标签
        let suggested_tags = self.generate_tag_suggestions(&all_entries, entry_id);

        // 相似条目
        let similar_entries = if let Some(id) = entry_id {
            self.find_similar_entries(&all_entries, id)?
        } else {
            Vec::new()
        };

        // 清理建议
        let cleanup_suggestions = self.generate_cleanup_suggestions(&all_entries);

        Ok(SmartSuggestions {
            suggested_tags,
            similar_entries,
            cleanup_suggestions,
        })
    }

    /// 导出历史记录
    pub async fn export_entries(
        &self,
        entry_ids: &[String],
        format: ExportFormat,
        output_path: &str,
    ) -> AppResult<()> {
        let entries: Vec<TranscriptionEntry> = entry_ids.iter()
            .filter_map(|id| {
                // 这里需要实现根据ID获取条目的功能
                // 目前先从所有条目中过滤
                if let Ok(all_entries) = self.db_manager.get_all_transcriptions() {
                    all_entries.into_iter().find(|e| &e.id == id)
                } else {
                    None
                }
            })
            .collect();

        match format {
            ExportFormat::Json => {
                let json_data = serde_json::to_string_pretty(&entries)
                    .map_err(|e| AppError::DataSerializationError(format!("JSON序列化失败: {}", e)))?;
                std::fs::write(output_path, json_data)
                    .map_err(|e| AppError::FileSystemError(format!("写入文件失败: {}", e)))?;
            }
            ExportFormat::Csv => {
                self.export_to_csv(&entries, output_path)?;
            }
            ExportFormat::Txt => {
                self.export_to_txt(&entries, output_path)?;
            }
            ExportFormat::Srt => {
                self.export_to_srt(&entries, output_path)?;
            }
            ExportFormat::Docx => {
                return Err(AppError::ValidationError("DOCX导出暂未实现".to_string()));
            }
        }

        // 记录导出活动
        for entry_id in entry_ids {
            self.record_activity(ActivityType::Exported, entry_id.clone(), format!("导出为 {:?}", format));
        }

        println!("📤 导出完成: {} 个条目导出到 {}", entries.len(), output_path);
        Ok(())
    }

    /// 清理历史记录
    pub async fn cleanup_history(&self, cleanup_type: CleanupType, dry_run: bool) -> AppResult<CleanupSuggestion> {
        let all_entries = self.db_manager.get_all_transcriptions()?;
        
        let (affected_entries, description, savings) = match cleanup_type {
            CleanupType::DuplicateText => {
                let duplicates = self.find_duplicate_text_entries(&all_entries);
                (
                    duplicates,
                    "删除重复的转录文本".to_string(),
                    "节省存储空间".to_string(),
                )
            }
            CleanupType::LowConfidenceEntries => {
                let low_confidence: Vec<String> = all_entries
                    .iter()
                    .filter(|e| e.confidence < 0.3)
                    .map(|e| e.id.clone())
                    .collect();
                (
                    low_confidence.clone(),
                    format!("删除 {} 个低置信度条目", low_confidence.len()),
                    "提升数据质量".to_string(),
                )
            }
            CleanupType::OldUntaggedEntries => {
                let cutoff_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 - (30 * 24 * 3600); // 30天前
                let old_untagged: Vec<String> = all_entries
                    .iter()
                    .filter(|e| e.timestamp < cutoff_time && (e.tags.is_none() || e.tags.as_ref().unwrap().trim().is_empty()))
                    .map(|e| e.id.clone())
                    .collect();
                (
                    old_untagged.clone(),
                    format!("删除 {} 个30天前的无标签条目", old_untagged.len()),
                    "整理历史记录".to_string(),
                )
            }
            CleanupType::LargeAudioFiles => {
                // 这里需要检查音频文件大小，暂时返回空
                (Vec::new(), "清理大音频文件".to_string(), "节省磁盘空间".to_string())
            }
            CleanupType::BrokenAudioLinks => {
                let broken_links = self.find_broken_audio_links(&all_entries);
                (
                    broken_links.clone(),
                    format!("修复 {} 个损坏的音频链接", broken_links.len()),
                    "修复数据完整性".to_string(),
                )
            }
        };

        if !dry_run && !affected_entries.is_empty() {
            // 执行实际清理
            match cleanup_type {
                CleanupType::DuplicateText | 
                CleanupType::LowConfidenceEntries | 
                CleanupType::OldUntaggedEntries => {
                    for entry_id in &affected_entries {
                        let _ = self.db_manager.delete_transcription(entry_id);
                    }
                }
                _ => {
                    // 其他类型的清理逻辑
                }
            }
        }

        Ok(CleanupSuggestion {
            suggestion_type: cleanup_type,
            affected_entries,
            description,
            potential_savings: savings,
        })
    }

    // =================== 私有辅助方法 ===================

    /// 生成搜索缓存键
    fn generate_cache_key(&self, options: &AdvancedSearchOptions) -> String {
        format!("{:?}", options) // 简化的缓存键生成
    }

    /// 缓存搜索结果
    fn cache_search_result(&self, key: String, result: SearchResult) {
        let mut cache = self.search_cache.lock();
        
        // 限制缓存大小
        if cache.len() >= self.max_cache_size {
            // 简单的LRU清理，删除第一个条目
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }
        
        cache.insert(key, result);
    }

    /// 应用全文搜索
    async fn apply_full_text_search(
        &self,
        mut result: SearchResult,
        options: &FullTextSearchOptions,
    ) -> AppResult<SearchResult> {
        if options.fuzzy_search {
            // 实现模糊搜索逻辑
            result.entries = result.entries.into_iter()
                .filter(|entry| self.fuzzy_match(&entry.text, &options.query))
                .collect();
        }

        // 根据匹配度重新排序
        if options.min_score > 0.0 {
            result.entries = result.entries.into_iter()
                .map(|entry| {
                    let score = self.calculate_relevance_score(&entry.text, &options.query);
                    (entry, score)
                })
                .filter(|(_, score)| *score >= options.min_score)
                .map(|(entry, _)| entry)
                .collect();
        }

        Ok(result)
    }

    /// 应用排序
    fn apply_sorting(&self, result: &mut SearchResult, sort_by: &SortOption, sort_order: &SortOrder) {
        result.entries.sort_by(|a, b| {
            let cmp = match sort_by {
                SortOption::Timestamp => a.timestamp.cmp(&b.timestamp),
                SortOption::Duration => a.duration.partial_cmp(&b.duration).unwrap_or(std::cmp::Ordering::Equal),
                SortOption::Confidence => a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal),
                SortOption::Model => a.model.cmp(&b.model),
                SortOption::TextLength => a.text.len().cmp(&b.text.len()),
                SortOption::CreatedAt => a.created_at.cmp(&b.created_at),
                SortOption::UpdatedAt => a.updated_at.cmp(&b.updated_at),
                SortOption::Relevance => std::cmp::Ordering::Equal, // 需要额外的相关性分数
            };

            match sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    /// 应用高亮
    fn apply_highlighting(&self, result: &mut SearchResult, query: &str) {
        let query_lower = query.to_lowercase();
        for entry in &mut result.entries {
            if entry.text.to_lowercase().contains(&query_lower) {
                // 简单的高亮实现
                entry.text = entry.text.replace(query, &format!("<mark>{}</mark>", query));
            }
        }
    }

    /// 获取分组键
    fn get_group_key(&self, entry: &TranscriptionEntry, group_by: &GroupByOption) -> String {
        match group_by {
            GroupByOption::Date => self.timestamp_to_date(entry.timestamp),
            GroupByOption::Model => entry.model.clone(),
            GroupByOption::Duration => {
                if entry.duration < 60.0 {
                    "短时间 (<1分钟)".to_string()
                } else if entry.duration < 300.0 {
                    "中等时间 (1-5分钟)".to_string()
                } else {
                    "长时间 (>5分钟)".to_string()
                }
            }
            GroupByOption::Confidence => {
                if entry.confidence < 0.5 {
                    "低置信度 (<50%)".to_string()
                } else if entry.confidence < 0.8 {
                    "中等置信度 (50-80%)".to_string()
                } else {
                    "高置信度 (>80%)".to_string()
                }
            }
            GroupByOption::Tags => {
                entry.tags.clone().unwrap_or_else(|| "无标签".to_string())
            }
        }
    }

    /// 执行单个操作
    async fn execute_single_operation(&self, entry_id: &str, operation: &BulkOperation) -> AppResult<()> {
        match operation {
            BulkOperation::Delete => {
                self.db_manager.delete_transcription(entry_id)?;
            }
            BulkOperation::AddTag { tag } => {
                // 这里需要实现标签添加逻辑
                // 当前数据库模型中标签是JSON字符串，需要解析和更新
                println!("为条目 {} 添加标签: {}", entry_id, tag);
            }
            BulkOperation::RemoveTag { tag } => {
                println!("从条目 {} 移除标签: {}", entry_id, tag);
            }
            BulkOperation::UpdateModel { new_model } => {
                // 需要添加更新模型的数据库方法
                println!("更新条目 {} 的模型为: {}", entry_id, new_model);
            }
            BulkOperation::Archive => {
                // 需要添加归档功能
                println!("归档条目: {}", entry_id);
            }
            BulkOperation::Unarchive => {
                println!("取消归档条目: {}", entry_id);
            }
            BulkOperation::Export { format } => {
                // 单个条目导出
                println!("导出条目 {} 为 {:?} 格式", entry_id, format);
            }
        }
        Ok(())
    }

    /// 记录活动
    fn record_activity(&self, activity_type: ActivityType, entry_id: String, description: String) {
        let activity = RecentActivity {
            activity_type,
            entry_id,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            description,
        };

        let mut activities = self.recent_activities.lock();
        activities.push(activity);

        // 限制活动记录数量
        if activities.len() > self.max_recent_activities {
            let len = activities.len();
            activities.drain(0..len - self.max_recent_activities);
        }
    }

    /// 时间戳转日期
    fn timestamp_to_date(&self, timestamp: i64) -> String {
        // 简化的日期转换，实际应用中应使用proper的时间库
        let date = std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64);
        let datetime = date.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let days = datetime / 86400;
        format!("{}-{:02}-{:02}", 1970 + days / 365, (days % 365) / 30 + 1, days % 30 + 1)
    }

    /// 时间戳转小时
    fn timestamp_to_hour(&self, timestamp: i64) -> u8 {
        ((timestamp % 86400) / 3600) as u8
    }

    /// 计算标签使用统计
    fn calculate_tag_usage(&self, entries: &[TranscriptionEntry]) -> HashMap<String, usize> {
        let mut tag_counts = HashMap::new();
        
        for entry in entries {
            if let Some(tags_str) = &entry.tags {
                // 假设标签是逗号分隔的字符串
                for tag in tags_str.split(',').map(|s| s.trim()) {
                    if !tag.is_empty() {
                        *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        
        tag_counts
    }

    /// 生成标签建议
    fn generate_tag_suggestions(&self, entries: &[TranscriptionEntry], entry_id: Option<&str>) -> Vec<String> {
        // 基于文本内容和历史标签生成建议
        let mut suggestions = Vec::new();
        
        // 从历史记录中提取常用标签
        let tag_usage = self.calculate_tag_usage(entries);
        let mut popular_tags: Vec<(String, usize)> = tag_usage.into_iter().collect();
        popular_tags.sort_by(|a, b| b.1.cmp(&a.1));
        
        for (tag, _) in popular_tags.into_iter().take(10) {
            suggestions.push(tag);
        }
        
        suggestions
    }

    /// 查找相似条目
    fn find_similar_entries(&self, entries: &[TranscriptionEntry], entry_id: &str) -> AppResult<Vec<String>> {
        let target_entry = entries.iter()
            .find(|e| e.id == entry_id)
            .ok_or_else(|| AppError::ValidationError("目标条目未找到".to_string()))?;

        let mut similar = Vec::new();
        
        for entry in entries {
            if entry.id != entry_id {
                let similarity = self.calculate_text_similarity(&target_entry.text, &entry.text);
                if similarity > 0.7 {
                    similar.push(entry.id.clone());
                }
            }
        }
        
        similar.truncate(5); // 最多返回5个相似条目
        Ok(similar)
    }

    /// 生成清理建议
    fn generate_cleanup_suggestions(&self, entries: &[TranscriptionEntry]) -> Vec<CleanupSuggestion> {
        let mut suggestions = Vec::new();

        // 重复文本检测
        let duplicates = self.find_duplicate_text_entries(entries);
        if !duplicates.is_empty() {
            suggestions.push(CleanupSuggestion {
                suggestion_type: CleanupType::DuplicateText,
                affected_entries: duplicates.clone(),
                description: format!("发现 {} 个重复文本条目", duplicates.len()),
                potential_savings: "节省存储空间".to_string(),
            });
        }

        // 低置信度条目
        let low_confidence: Vec<String> = entries
            .iter()
            .filter(|e| e.confidence < 0.3)
            .map(|e| e.id.clone())
            .collect();
        
        if !low_confidence.is_empty() {
            suggestions.push(CleanupSuggestion {
                suggestion_type: CleanupType::LowConfidenceEntries,
                affected_entries: low_confidence.clone(),
                description: format!("发现 {} 个低置信度条目", low_confidence.len()),
                potential_savings: "提升数据质量".to_string(),
            });
        }

        suggestions
    }

    /// 查找重复文本条目
    fn find_duplicate_text_entries(&self, entries: &[TranscriptionEntry]) -> Vec<String> {
        let mut text_to_ids: HashMap<String, Vec<String>> = HashMap::new();
        let mut duplicates = Vec::new();

        for entry in entries {
            let normalized_text = entry.text.trim().to_lowercase();
            text_to_ids.entry(normalized_text).or_insert_with(Vec::new).push(entry.id.clone());
        }

        for (_, ids) in text_to_ids {
            if ids.len() > 1 {
                duplicates.extend(ids.into_iter().skip(1)); // 保留第一个，其余标记为重复
            }
        }

        duplicates
    }

    /// 查找损坏的音频链接
    fn find_broken_audio_links(&self, entries: &[TranscriptionEntry]) -> Vec<String> {
        let mut broken = Vec::new();
        
        for entry in entries {
            if let Some(audio_path) = &entry.audio_file_path {
                if !std::path::Path::new(audio_path).exists() {
                    broken.push(entry.id.clone());
                }
            }
        }
        
        broken
    }

    /// 模糊匹配
    fn fuzzy_match(&self, text: &str, query: &str) -> bool {
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();
        
        // 简单的模糊匹配：检查所有查询词是否都在文本中
        query_lower.split_whitespace()
            .all(|word| text_lower.contains(word))
    }

    /// 计算相关性分数
    fn calculate_relevance_score(&self, text: &str, query: &str) -> f64 {
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        
        if query_words.is_empty() {
            return 0.0;
        }
        
        let matches = query_words.iter()
            .filter(|word| text_lower.contains(*word))
            .count();
            
        matches as f64 / query_words.len() as f64
    }

    /// 计算文本相似度
    fn calculate_text_similarity(&self, text1: &str, text2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// 导出到CSV
    fn export_to_csv(&self, entries: &[TranscriptionEntry], output_path: &str) -> AppResult<()> {
        let mut csv_content = String::from("ID,Text,Timestamp,Duration,Model,Confidence,AudioFile,CreatedAt,UpdatedAt,Tags\n");
        
        for entry in entries {
            let line = format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                entry.id,
                entry.text.replace(",", "，").replace("\n", " "), // 转义逗号和换行
                entry.timestamp,
                entry.duration,
                entry.model,
                entry.confidence,
                entry.audio_file_path.as_deref().unwrap_or(""),
                entry.created_at.as_deref().unwrap_or(""),
                entry.updated_at.as_deref().unwrap_or(""),
                entry.tags.as_deref().unwrap_or("")
            );
            csv_content.push_str(&line);
        }
        
        std::fs::write(output_path, csv_content)
            .map_err(|e| AppError::FileSystemError(format!("写入CSV文件失败: {}", e)))?;
            
        Ok(())
    }

    /// 导出到TXT
    fn export_to_txt(&self, entries: &[TranscriptionEntry], output_path: &str) -> AppResult<()> {
        let mut txt_content = String::new();
        
        for entry in entries {
            txt_content.push_str(&format!(
                "=== {} ===\n时间: {}\n模型: {}\n置信度: {:.2}\n内容:\n{}\n\n",
                entry.id,
                entry.created_at.as_deref().unwrap_or(""),
                entry.model,
                entry.confidence,
                entry.text
            ));
        }
        
        std::fs::write(output_path, txt_content)
            .map_err(|e| AppError::FileSystemError(format!("写入TXT文件失败: {}", e)))?;
            
        Ok(())
    }

    /// 导出到SRT字幕格式
    fn export_to_srt(&self, entries: &[TranscriptionEntry], output_path: &str) -> AppResult<()> {
        let mut srt_content = String::new();
        
        for (index, entry) in entries.iter().enumerate() {
            let start_time = "00:00:00,000"; // 需要根据实际时间戳计算
            let end_time = format!("00:00:{:02},{:03}", 
                (entry.duration as u32) / 60, 
                ((entry.duration * 1000.0) as u32) % 1000);
            
            srt_content.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                index + 1,
                start_time,
                end_time,
                entry.text
            ));
        }
        
        std::fs::write(output_path, srt_content)
            .map_err(|e| AppError::FileSystemError(format!("写入SRT文件失败: {}", e)))?;
            
        Ok(())
    }
}