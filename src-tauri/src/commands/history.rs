// 历史记录管理相关的Tauri命令
// 提供高级搜索、批量操作和统计分析功能

use std::sync::Arc;
use tauri::State;
use crate::errors::AppResult;
use crate::types::TranscriptionEntry;
use crate::database::{
    HistoryManager, 
    AdvancedSearchOptions, 
    SearchResult, 
    GroupedSearchResult,
    BulkOperation, 
    BulkOperationResult,
    HistoryStatistics,
    SmartSuggestions,
    CleanupSuggestion,
    CleanupType,
    ExportFormat,
    SortOption,
    SortOrder,
    GroupByOption,
    FullTextSearchOptions,
    SearchField,
};

/// 高级搜索命令
#[tauri::command]
pub async fn advanced_search_entries(
    history_manager: State<'_, Arc<HistoryManager>>,
    options: AdvancedSearchOptions,
) -> Result<SearchResult, String> {
    history_manager
        .advanced_search(&options)
        .await
        .map_err(|e| e.to_string())
}

/// 分组搜索命令
#[tauri::command]
pub async fn grouped_search_entries(
    history_manager: State<'_, Arc<HistoryManager>>,
    options: AdvancedSearchOptions,
) -> Result<GroupedSearchResult, String> {
    history_manager
        .grouped_search(&options)
        .await
        .map_err(|e| e.to_string())
}

/// 批量操作命令
#[tauri::command]
pub async fn bulk_operation_entries(
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_ids: Vec<String>,
    operation: BulkOperation,
) -> Result<BulkOperationResult, String> {
    history_manager
        .bulk_operation(&entry_ids, operation)
        .await
        .map_err(|e| e.to_string())
}

/// 获取历史统计命令
#[tauri::command]
pub async fn get_history_statistics(
    history_manager: State<'_, Arc<HistoryManager>>,
) -> Result<HistoryStatistics, String> {
    history_manager
        .get_history_statistics()
        .await
        .map_err(|e| e.to_string())
}

/// 获取智能建议命令
#[tauri::command]
pub async fn get_smart_suggestions(
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_id: Option<String>,
) -> Result<SmartSuggestions, String> {
    let entry_id_ref = entry_id.as_deref();
    history_manager
        .get_smart_suggestions(entry_id_ref)
        .await
        .map_err(|e| e.to_string())
}

/// 导出历史记录命令
#[tauri::command]
pub async fn export_history_entries(
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_ids: Vec<String>,
    format: ExportFormat,
    output_path: String,
) -> Result<(), String> {
    history_manager
        .export_entries(&entry_ids, format, &output_path)
        .await
        .map_err(|e| e.to_string())
}

/// 清理历史记录命令
#[tauri::command]
pub async fn cleanup_history_entries(
    history_manager: State<'_, Arc<HistoryManager>>,
    cleanup_type: CleanupType,
    dry_run: bool,
) -> Result<CleanupSuggestion, String> {
    history_manager
        .cleanup_history(cleanup_type, dry_run)
        .await
        .map_err(|e| e.to_string())
}

/// 快速搜索命令（简化版）
#[tauri::command]
pub async fn quick_search_entries(
    history_manager: State<'_, Arc<HistoryManager>>,
    query: String,
    limit: Option<usize>,
) -> Result<SearchResult, String> {
    let options = AdvancedSearchOptions {
        full_text_search: Some(FullTextSearchOptions {
            query: query.clone(),
            fuzzy_search: true,
            highlight: true,
            search_fields: vec![SearchField::Text, SearchField::Tags],
            min_score: 0.1,
        }),
        sort_by: SortOption::Relevance,
        sort_order: SortOrder::Descending,
        page_size: limit.unwrap_or(20),
        page: 0,
        ..Default::default()
    };

    history_manager
        .advanced_search(&options)
        .await
        .map_err(|e| e.to_string())
}

/// 按日期搜索命令
#[tauri::command]
pub async fn search_entries_by_date(
    history_manager: State<'_, Arc<HistoryManager>>,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
    page_size: Option<usize>,
    page: Option<usize>,
) -> Result<SearchResult, String> {
    let mut filter = crate::database::SearchFilter::default();
    filter.start_date = start_timestamp;
    filter.end_date = end_timestamp;

    let options = AdvancedSearchOptions {
        filter,
        sort_by: SortOption::Timestamp,
        sort_order: SortOrder::Descending,
        page_size: page_size.unwrap_or(50),
        page: page.unwrap_or(0),
        ..Default::default()
    };

    history_manager
        .advanced_search(&options)
        .await
        .map_err(|e| e.to_string())
}

/// 按模型搜索命令
#[tauri::command]
pub async fn search_entries_by_model(
    history_manager: State<'_, Arc<HistoryManager>>,
    model_name: String,
    page_size: Option<usize>,
    page: Option<usize>,
) -> Result<SearchResult, String> {
    let mut filter = crate::database::SearchFilter::default();
    filter.model = Some(model_name);

    let options = AdvancedSearchOptions {
        filter,
        sort_by: SortOption::Timestamp,
        sort_order: SortOrder::Descending,
        page_size: page_size.unwrap_or(50),
        page: page.unwrap_or(0),
        ..Default::default()
    };

    history_manager
        .advanced_search(&options)
        .await
        .map_err(|e| e.to_string())
}

/// 获取最近条目命令
#[tauri::command]
pub async fn get_recent_entries(
    history_manager: State<'_, Arc<HistoryManager>>,
    limit: Option<usize>,
) -> Result<SearchResult, String> {
    let options = AdvancedSearchOptions {
        sort_by: SortOption::Timestamp,
        sort_order: SortOrder::Descending,
        page_size: limit.unwrap_or(20),
        page: 0,
        ..Default::default()
    };

    history_manager
        .advanced_search(&options)
        .await
        .map_err(|e| e.to_string())
}

/// 按置信度过滤命令
#[tauri::command]
pub async fn filter_entries_by_confidence(
    history_manager: State<'_, Arc<HistoryManager>>,
    min_confidence: f64,
    page_size: Option<usize>,
    page: Option<usize>,
) -> Result<SearchResult, String> {
    let mut filter = crate::database::SearchFilter::default();
    filter.min_confidence = Some(min_confidence);

    let options = AdvancedSearchOptions {
        filter,
        sort_by: SortOption::Confidence,
        sort_order: SortOrder::Descending,
        page_size: page_size.unwrap_or(50),
        page: page.unwrap_or(0),
        ..Default::default()
    };

    history_manager
        .advanced_search(&options)
        .await
        .map_err(|e| e.to_string())
}

/// 按时长过滤命令
#[tauri::command]
pub async fn filter_entries_by_duration(
    history_manager: State<'_, Arc<HistoryManager>>,
    min_duration: Option<f64>,
    max_duration: Option<f64>,
    page_size: Option<usize>,
    page: Option<usize>,
) -> Result<SearchResult, String> {
    let mut filter = crate::database::SearchFilter::default();
    filter.min_duration = min_duration;
    filter.max_duration = max_duration;

    let options = AdvancedSearchOptions {
        filter,
        sort_by: SortOption::Duration,
        sort_order: SortOrder::Descending,
        page_size: page_size.unwrap_or(50),
        page: page.unwrap_or(0),
        ..Default::default()
    };

    history_manager
        .advanced_search(&options)
        .await
        .map_err(|e| e.to_string())
}

/// 批量删除命令
#[tauri::command]
pub async fn bulk_delete_entries(
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_ids: Vec<String>,
) -> Result<BulkOperationResult, String> {
    history_manager
        .bulk_operation(&entry_ids, BulkOperation::Delete)
        .await
        .map_err(|e| e.to_string())
}

/// 批量添加标签命令
#[tauri::command]
pub async fn bulk_add_tag(
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_ids: Vec<String>,
    tag: String,
) -> Result<BulkOperationResult, String> {
    history_manager
        .bulk_operation(&entry_ids, BulkOperation::AddTag { tag })
        .await
        .map_err(|e| e.to_string())
}

/// 批量移除标签命令
#[tauri::command]
pub async fn bulk_remove_tag(
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_ids: Vec<String>,
    tag: String,
) -> Result<BulkOperationResult, String> {
    history_manager
        .bulk_operation(&entry_ids, BulkOperation::RemoveTag { tag })
        .await
        .map_err(|e| e.to_string())
}

/// 批量导出命令
#[tauri::command]
pub async fn bulk_export_entries(
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_ids: Vec<String>,
    format: ExportFormat,
) -> Result<BulkOperationResult, String> {
    history_manager
        .bulk_operation(&entry_ids, BulkOperation::Export { format })
        .await
        .map_err(|e| e.to_string())
}

/// 获取数据完整性报告
#[tauri::command]
pub async fn get_data_integrity_report(
    history_manager: State<'_, Arc<HistoryManager>>,
) -> Result<Vec<CleanupSuggestion>, String> {
    let mut suggestions = Vec::new();
    
    // 检查各种数据完整性问题
    let cleanup_types = vec![
        CleanupType::DuplicateText,
        CleanupType::LowConfidenceEntries,
        CleanupType::OldUntaggedEntries,
        CleanupType::BrokenAudioLinks,
    ];
    
    for cleanup_type in cleanup_types {
        match history_manager.cleanup_history(cleanup_type, true).await {
            Ok(suggestion) => suggestions.push(suggestion),
            Err(_) => continue,
        }
    }
    
    Ok(suggestions)
}

/// 构建高级搜索选项的辅助函数
#[tauri::command]
pub async fn build_search_options(
    query: Option<String>,
    model_filter: Option<String>,
    min_confidence: Option<f64>,
    start_date: Option<i64>,
    end_date: Option<i64>,
    min_duration: Option<f64>,
    max_duration: Option<f64>,
    sort_by: Option<SortOption>,
    sort_order: Option<SortOrder>,
    page_size: Option<usize>,
    page: Option<usize>,
    group_by: Option<GroupByOption>,
) -> Result<AdvancedSearchOptions, String> {
    let mut filter = crate::database::SearchFilter::default();
    filter.model = model_filter;
    filter.min_confidence = min_confidence;
    filter.start_date = start_date;
    filter.end_date = end_date;
    filter.min_duration = min_duration;
    filter.max_duration = max_duration;

    let full_text_search = if let Some(q) = query {
        Some(FullTextSearchOptions {
            query: q,
            fuzzy_search: true,
            highlight: true,
            search_fields: vec![SearchField::Text, SearchField::Tags, SearchField::Metadata],
            min_score: 0.1,
        })
    } else {
        None
    };

    Ok(AdvancedSearchOptions {
        filter,
        sort_by: sort_by.unwrap_or(SortOption::Timestamp),
        sort_order: sort_order.unwrap_or(SortOrder::Descending),
        page_size: page_size.unwrap_or(50),
        page: page.unwrap_or(0),
        include_deleted: false,
        full_text_search,
        group_by,
    })
}

/// 获取搜索建议（自动完成）
#[tauri::command]
pub async fn get_search_suggestions(
    history_manager: State<'_, Arc<HistoryManager>>,
    partial_query: String,
    max_suggestions: Option<usize>,
) -> Result<Vec<String>, String> {
    // 这里可以基于历史搜索和内容生成建议
    // 目前返回简单的建议
    let suggestions = vec![
        format!("{}*", partial_query), // 通配符搜索
        format!("\"{}\"", partial_query), // 精确匹配
    ];
    
    let max = max_suggestions.unwrap_or(10);
    Ok(suggestions.into_iter().take(max).collect())
}

/// 保存搜索预设
#[tauri::command]
pub async fn save_search_preset(
    preset_name: String,
    options: AdvancedSearchOptions,
) -> Result<(), String> {
    // 这里可以将搜索预设保存到配置文件或数据库
    println!("保存搜索预设: {} -> {:?}", preset_name, options);
    Ok(())
}

/// 加载搜索预设
#[tauri::command]
pub async fn load_search_preset(
    preset_name: String,
) -> Result<Option<AdvancedSearchOptions>, String> {
    // 从配置文件或数据库加载搜索预设
    println!("加载搜索预设: {}", preset_name);
    Ok(None) // 暂时返回空
}

/// 获取可用的搜索预设列表
#[tauri::command]
pub async fn get_search_presets() -> Result<Vec<String>, String> {
    // 返回可用的搜索预设名称列表
    Ok(vec![
        "最近一周".to_string(),
        "高置信度记录".to_string(),
        "长时间录音".to_string(),
        "需要整理的记录".to_string(),
    ])
}