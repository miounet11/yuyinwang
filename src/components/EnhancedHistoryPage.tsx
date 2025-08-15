import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './EnhancedHistoryPage.css';

// 高级搜索选项接口
interface AdvancedSearchOptions {
  filter?: {
    model?: string;
    min_confidence?: number;
    start_date?: number;
    end_date?: number;
    min_duration?: number;
    max_duration?: number;
  };
  full_text_search?: {
    query: string;
    fuzzy_search: boolean;
    highlight: boolean;
    search_fields: string[];
    min_score: number;
  };
  sort_by: string;
  sort_order: string;
  page_size: number;
  page: number;
  group_by?: string;
}

// 搜索结果接口
interface SearchResult {
  entries: any[];
  total_count: number;
  page: number;
  page_size: number;
  total_pages: number;
  has_next: boolean;
  has_previous: boolean;
}

// 批量操作类型
enum BulkOperation {
  Delete = 'Delete',
  AddTag = 'AddTag',
  RemoveTag = 'RemoveTag',
  Export = 'Export'
}

// 导出格式
enum ExportFormat {
  Json = 'Json',
  Csv = 'Csv',
  Txt = 'Txt',
  Markdown = 'Markdown'
}

interface EnhancedHistoryPageProps {
  isVisible: boolean;
  onClose: () => void;
  onOpenTranscriptionDetail: (entry: any) => void;
}

const EnhancedHistoryPage: React.FC<EnhancedHistoryPageProps> = ({
  isVisible,
  onClose,
  onOpenTranscriptionDetail
}) => {
  // 基础状态
  const [searchResults, setSearchResults] = useState<SearchResult>({
    entries: [],
    total_count: 0,
    page: 0,
    page_size: 20,
    total_pages: 0,
    has_next: false,
    has_previous: false
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');

  // 搜索状态
  const [searchQuery, setSearchQuery] = useState('');
  const [showAdvancedSearch, setShowAdvancedSearch] = useState(false);
  const [advancedOptions, setAdvancedOptions] = useState<AdvancedSearchOptions>({
    sort_by: 'Timestamp',
    sort_order: 'Descending',
    page_size: 20,
    page: 0
  });

  // 筛选状态
  const [modelFilter, setModelFilter] = useState<string>('all');
  const [confidenceFilter, setConfidenceFilter] = useState<number>(0);
  const [dateFilter, setDateFilter] = useState<{start?: string, end?: string}>({});
  const [durationFilter, setDurationFilter] = useState<{min?: number, max?: number}>({});

  // 批量操作状态
  const [selectedEntries, setSelectedEntries] = useState<Set<string>>(new Set());
  const [showBulkActions, setShowBulkActions] = useState(false);
  const [bulkOperationInProgress, setBulkOperationInProgress] = useState(false);

  // 导出状态
  const [showExportDialog, setShowExportDialog] = useState(false);
  const [exportFormat, setExportFormat] = useState<ExportFormat>(ExportFormat.Txt);
  const [exportInProgress, setExportInProgress] = useState(false);

  // 分页状态
  const [currentPage, setCurrentPage] = useState(0);

  // 统计数据
  const [statistics, setStatistics] = useState<any>(null);

  // 预设搜索
  const [searchPresets, setSearchPresets] = useState<string[]>([]);
  const [selectedPreset, setSelectedPreset] = useState<string>('');

  // 执行搜索
  const performSearch = useCallback(async (options?: Partial<AdvancedSearchOptions>) => {
    setLoading(true);
    setError('');
    
    try {
      const searchOptions: AdvancedSearchOptions = {
        ...advancedOptions,
        ...options,
        page: currentPage,
        // 确保 filter 字段总是存在
        filter: {
          ...(modelFilter !== 'all' && { model: modelFilter }),
          ...(confidenceFilter > 0 && { min_confidence: confidenceFilter / 100 }),
          ...(dateFilter.start && { start_date: new Date(dateFilter.start).getTime() }),
          ...(dateFilter.end && { end_date: new Date(dateFilter.end).getTime() }),
          ...(durationFilter.min && { min_duration: durationFilter.min }),
          ...(durationFilter.max && { max_duration: durationFilter.max })
        }
      };

      // 构建全文搜索
      if (searchQuery.trim()) {
        searchOptions.full_text_search = {
          query: searchQuery.trim(),
          fuzzy_search: true,
          highlight: true,
          search_fields: ['Text', 'Tags', 'Metadata'],
          min_score: 0.1
        };
      }

      const result = await invoke<SearchResult>('advanced_search_entries', {
        options: searchOptions
      });

      setSearchResults(result);
    } catch (err) {
      setError(`搜索失败: ${err}`);
      console.error('搜索失败:', err);
    } finally {
      setLoading(false);
    }
  }, [advancedOptions, currentPage, modelFilter, confidenceFilter, dateFilter, durationFilter, searchQuery]);

  // 快速搜索
  const quickSearch = useCallback(async (query: string) => {
    if (!query.trim()) {
      performSearch();
      return;
    }

    setLoading(true);
    try {
      const result = await invoke<SearchResult>('quick_search_entries', {
        query: query.trim(),
        limit: 20
      });
      setSearchResults(result);
    } catch (err) {
      setError(`快速搜索失败: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [performSearch]);

  // 批量操作
  const performBulkOperation = async (operation: BulkOperation, additionalData?: any) => {
    if (selectedEntries.size === 0) {
      alert('请先选择要操作的记录');
      return;
    }

    setBulkOperationInProgress(true);
    try {
      const entryIds = Array.from(selectedEntries);
      let bulkOp: any = operation;

      // 处理需要额外数据的操作
      if (operation === BulkOperation.AddTag || operation === BulkOperation.RemoveTag) {
        const tag = additionalData?.tag || prompt('请输入标签名称');
        if (!tag) return;
        bulkOp = operation === BulkOperation.AddTag 
          ? { AddTag: { tag } }
          : { RemoveTag: { tag } };
      } else if (operation === BulkOperation.Export) {
        bulkOp = { Export: { format: additionalData?.format || exportFormat } };
      }

      const result = await invoke('bulk_operation_entries', {
        entryIds,
        operation: bulkOp
      });

      alert(`批量操作完成: ${JSON.stringify(result)}`);
      setSelectedEntries(new Set());
      performSearch(); // 刷新搜索结果
    } catch (err) {
      alert(`批量操作失败: ${err}`);
    } finally {
      setBulkOperationInProgress(false);
    }
  };

  // 导出选中的记录
  const exportSelectedEntries = async () => {
    if (selectedEntries.size === 0) {
      alert('请先选择要导出的记录');
      return;
    }

    setExportInProgress(true);
    try {
      const entryIds = Array.from(selectedEntries);
      
      // 生成文件名
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const filename = `transcription_export_${timestamp}`;
      
      await invoke('bulk_export_entries', {
        entryIds,
        format: exportFormat
      });

      alert(`导出完成! 文件已保存为: ${filename}`);
      setShowExportDialog(false);
      setSelectedEntries(new Set());
    } catch (err) {
      alert(`导出失败: ${err}`);
    } finally {
      setExportInProgress(false);
    }
  };

  // 加载搜索预设
  const loadSearchPresets = async () => {
    try {
      const presets = await invoke<string[]>('get_search_presets');
      setSearchPresets(presets);
    } catch (err) {
      console.error('加载搜索预设失败:', err);
    }
  };

  // 应用搜索预设
  const applySearchPreset = async (presetName: string) => {
    try {
      const preset = await invoke<AdvancedSearchOptions>('load_search_preset', {
        presetName
      });
      
      if (preset) {
        setAdvancedOptions(preset);
        performSearch(preset);
      }
    } catch (err) {
      console.error('应用搜索预设失败:', err);
    }
  };

  // 加载统计数据
  const loadStatistics = async () => {
    try {
      const stats = await invoke('get_history_statistics');
      setStatistics(stats);
    } catch (err) {
      console.error('加载统计数据失败:', err);
    }
  };

  // 选择/取消选择条目
  const toggleEntrySelection = (entryId: string) => {
    const newSelected = new Set(selectedEntries);
    if (newSelected.has(entryId)) {
      newSelected.delete(entryId);
    } else {
      newSelected.add(entryId);
    }
    setSelectedEntries(newSelected);
  };

  // 全选/取消全选
  const toggleSelectAll = () => {
    if (selectedEntries.size === searchResults.entries.length) {
      setSelectedEntries(new Set());
    } else {
      setSelectedEntries(new Set(searchResults.entries.map(entry => entry.id)));
    }
  };

  // 翻页
  const goToPage = (page: number) => {
    setCurrentPage(page);
  };

  // 初始化
  useEffect(() => {
    if (isVisible) {
      performSearch();
      loadSearchPresets();
      loadStatistics();
    }
  }, [isVisible, currentPage]);

  // 搜索查询变化时的防抖处理
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (searchQuery) {
        quickSearch(searchQuery);
      } else {
        performSearch();
      }
    }, 300);

    return () => clearTimeout(timeoutId);
  }, [searchQuery, quickSearch, performSearch]);

  if (!isVisible) return null;

  return (
    <div className="enhanced-history-overlay" onClick={onClose}>
      <div className="enhanced-history-panel" onClick={(e) => e.stopPropagation()}>
        <div className="enhanced-history-header">
          <h2>📚 增强历史记录管理</h2>
          <div className="header-actions">
            <button 
              className="preset-btn"
              onClick={() => setShowAdvancedSearch(!showAdvancedSearch)}
            >
              {showAdvancedSearch ? '简单搜索' : '高级搜索'}
            </button>
            <button 
              className="bulk-btn"
              onClick={() => setShowBulkActions(!showBulkActions)}
              disabled={selectedEntries.size === 0}
            >
              批量操作 ({selectedEntries.size})
            </button>
            <button className="close-btn" onClick={onClose}>×</button>
          </div>
        </div>

        <div className="enhanced-history-content">
          {/* 搜索区域 */}
          <div className="search-section">
            <div className="basic-search">
              <input
                type="text"
                className="search-input"
                placeholder="搜索转录内容、标签或元数据..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
              <button 
                className="search-btn"
                onClick={() => performSearch()}
                disabled={loading}
              >
                {loading ? '搜索中...' : '🔍'}
              </button>
            </div>

            {/* 搜索预设 */}
            {searchPresets.length > 0 && (
              <div className="search-presets">
                <label>快速预设:</label>
                <select 
                  value={selectedPreset}
                  onChange={(e) => {
                    setSelectedPreset(e.target.value);
                    if (e.target.value) {
                      applySearchPreset(e.target.value);
                    }
                  }}
                >
                  <option value="">选择预设...</option>
                  {searchPresets.map(preset => (
                    <option key={preset} value={preset}>{preset}</option>
                  ))}
                </select>
              </div>
            )}

            {/* 高级搜索选项 */}
            {showAdvancedSearch && (
              <div className="advanced-search">
                <div className="filter-row">
                  <div className="filter-group">
                    <label>模型筛选:</label>
                    <select 
                      value={modelFilter}
                      onChange={(e) => setModelFilter(e.target.value)}
                    >
                      <option value="all">所有模型</option>
                      <option value="whisper">Whisper</option>
                      <option value="luyin">录音王</option>
                      <option value="openai">OpenAI</option>
                    </select>
                  </div>

                  <div className="filter-group">
                    <label>最低置信度:</label>
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={confidenceFilter}
                      onChange={(e) => setConfidenceFilter(Number(e.target.value))}
                    />
                    <span>{confidenceFilter}%</span>
                  </div>
                </div>

                <div className="filter-row">
                  <div className="filter-group">
                    <label>日期范围:</label>
                    <input
                      type="date"
                      value={dateFilter.start || ''}
                      onChange={(e) => setDateFilter({...dateFilter, start: e.target.value})}
                    />
                    <span>至</span>
                    <input
                      type="date"
                      value={dateFilter.end || ''}
                      onChange={(e) => setDateFilter({...dateFilter, end: e.target.value})}
                    />
                  </div>

                  <div className="filter-group">
                    <label>时长范围:</label>
                    <input
                      type="number"
                      placeholder="最小秒数"
                      value={durationFilter.min || ''}
                      onChange={(e) => setDurationFilter({...durationFilter, min: Number(e.target.value)})}
                    />
                    <span>-</span>
                    <input
                      type="number"
                      placeholder="最大秒数"
                      value={durationFilter.max || ''}
                      onChange={(e) => setDurationFilter({...durationFilter, max: Number(e.target.value)})}
                    />
                  </div>
                </div>

                <div className="filter-row">
                  <div className="filter-group">
                    <label>排序方式:</label>
                    <select
                      value={advancedOptions.sort_by}
                      onChange={(e) => setAdvancedOptions({...advancedOptions, sort_by: e.target.value})}
                    >
                      <option value="Timestamp">时间</option>
                      <option value="Confidence">置信度</option>
                      <option value="Duration">时长</option>
                      <option value="Relevance">相关性</option>
                    </select>
                    <select
                      value={advancedOptions.sort_order}
                      onChange={(e) => setAdvancedOptions({...advancedOptions, sort_order: e.target.value})}
                    >
                      <option value="Descending">降序</option>
                      <option value="Ascending">升序</option>
                    </select>
                  </div>

                  <div className="filter-group">
                    <label>每页显示:</label>
                    <select
                      value={advancedOptions.page_size}
                      onChange={(e) => setAdvancedOptions({...advancedOptions, page_size: Number(e.target.value)})}
                    >
                      <option value={10}>10条</option>
                      <option value={20}>20条</option>
                      <option value={50}>50条</option>
                      <option value={100}>100条</option>
                    </select>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* 批量操作区域 */}
          {showBulkActions && selectedEntries.size > 0 && (
            <div className="bulk-actions">
              <h3>批量操作 ({selectedEntries.size} 项已选择)</h3>
              <div className="bulk-buttons">
                <button 
                  onClick={() => performBulkOperation(BulkOperation.Delete)}
                  disabled={bulkOperationInProgress}
                  className="bulk-delete-btn"
                >
                  删除选中
                </button>
                <button 
                  onClick={() => performBulkOperation(BulkOperation.AddTag)}
                  disabled={bulkOperationInProgress}
                  className="bulk-tag-btn"
                >
                  添加标签
                </button>
                <button 
                  onClick={() => performBulkOperation(BulkOperation.RemoveTag)}
                  disabled={bulkOperationInProgress}
                  className="bulk-tag-btn"
                >
                  移除标签
                </button>
                <button 
                  onClick={() => setShowExportDialog(true)}
                  disabled={bulkOperationInProgress}
                  className="bulk-export-btn"
                >
                  导出选中
                </button>
              </div>
            </div>
          )}

          {/* 统计信息 */}
          {statistics && (
            <div className="statistics-summary">
              <div className="stat-item">
                <span className="stat-label">总记录数:</span>
                <span className="stat-value">{statistics.total_entries}</span>
              </div>
              <div className="stat-item">
                <span className="stat-label">搜索结果:</span>
                <span className="stat-value">{searchResults.total_count}</span>
              </div>
              <div className="stat-item">
                <span className="stat-label">平均置信度:</span>
                <span className="stat-value">{statistics.average_confidence ? `${Math.round(statistics.average_confidence * 100)}%` : 'N/A'}</span>
              </div>
            </div>
          )}

          {/* 结果列表 */}
          <div className="results-section">
            {error && (
              <div className="error-message">
                ❌ {error}
              </div>
            )}

            {loading ? (
              <div className="loading-message">
                🔄 加载中...
              </div>
            ) : (
              <>
                {/* 列表头部 - 包含全选功能 */}
                {searchResults.entries.length > 0 && (
                  <div className="list-header">
                    <label className="select-all">
                      <input
                        type="checkbox"
                        checked={selectedEntries.size === searchResults.entries.length && searchResults.entries.length > 0}
                        onChange={toggleSelectAll}
                      />
                      全选
                    </label>
                    <span className="results-info">
                      显示 {searchResults.entries.length} / {searchResults.total_count} 条记录
                    </span>
                  </div>
                )}

                {/* 条目列表 */}
                <div className="entries-list">
                  {searchResults.entries.length > 0 ? (
                    searchResults.entries.map((entry) => (
                      <div key={entry.id} className="history-entry enhanced">
                        <div className="entry-checkbox">
                          <input
                            type="checkbox"
                            checked={selectedEntries.has(entry.id)}
                            onChange={() => toggleEntrySelection(entry.id)}
                          />
                        </div>
                        
                        <div 
                          className="entry-content"
                          onClick={() => onOpenTranscriptionDetail(entry)}
                        >
                          <div className="entry-text">
                            {entry.text}
                          </div>
                          <div className="entry-metadata">
                            <span className="entry-timestamp">
                              {new Date(entry.timestamp).toLocaleString()}
                            </span>
                            <span className="entry-model">
                              {entry.model}
                            </span>
                            <span className="entry-confidence">
                              {Math.round(entry.confidence * 100)}%
                            </span>
                            <span className="entry-duration">
                              {entry.duration}秒
                            </span>
                            {entry.tags && entry.tags.length > 0 && (
                              <div className="entry-tags">
                                {entry.tags.map((tag: string) => (
                                  <span key={tag} className="tag">{tag}</span>
                                ))}
                              </div>
                            )}
                          </div>
                        </div>

                        <div className="entry-actions">
                          <button 
                            onClick={() => onOpenTranscriptionDetail(entry)}
                            className="view-btn"
                          >
                            查看
                          </button>
                        </div>
                      </div>
                    ))
                  ) : (
                    <div className="empty-results">
                      <p>📭 没有找到匹配的记录</p>
                      {searchQuery && (
                        <button onClick={() => setSearchQuery('')}>
                          清除搜索条件
                        </button>
                      )}
                    </div>
                  )}
                </div>

                {/* 分页控件 */}
                {searchResults.total_pages > 1 && (
                  <div className="pagination">
                    <button
                      onClick={() => goToPage(currentPage - 1)}
                      disabled={!searchResults.has_previous}
                      className="page-btn"
                    >
                      上一页
                    </button>
                    
                    <span className="page-info">
                      第 {currentPage + 1} / {searchResults.total_pages} 页
                    </span>
                    
                    <button
                      onClick={() => goToPage(currentPage + 1)}
                      disabled={!searchResults.has_next}
                      className="page-btn"
                    >
                      下一页
                    </button>
                  </div>
                )}
              </>
            )}
          </div>
        </div>

        {/* 导出对话框 */}
        {showExportDialog && (
          <div className="export-dialog-overlay">
            <div className="export-dialog">
              <h3>导出选中记录</h3>
              <div className="export-options">
                <label>
                  <span>导出格式:</span>
                  <select 
                    value={exportFormat}
                    onChange={(e) => setExportFormat(e.target.value as ExportFormat)}
                  >
                    <option value={ExportFormat.Txt}>纯文本 (.txt)</option>
                    <option value={ExportFormat.Json}>JSON (.json)</option>
                    <option value={ExportFormat.Csv}>CSV (.csv)</option>
                    <option value={ExportFormat.Markdown}>Markdown (.md)</option>
                  </select>
                </label>
              </div>
              <div className="export-actions">
                <button 
                  onClick={() => setShowExportDialog(false)}
                  className="cancel-btn"
                >
                  取消
                </button>
                <button 
                  onClick={exportSelectedEntries}
                  disabled={exportInProgress}
                  className="export-btn"
                >
                  {exportInProgress ? '导出中...' : `导出 ${selectedEntries.size} 条记录`}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default EnhancedHistoryPage;