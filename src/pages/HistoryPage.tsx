import React, { useState, useCallback, memo, useMemo } from 'react';
import { useStore } from '../stores/appStore';
import TranscriptionDetailView from '../components/TranscriptionDetailView';

interface TranscriptionEntry {
  id: string;
  text: string;
  timestamp: number;
  duration: number;
  model: string;
  confidence: number;
  audio_file_path?: string;
}

interface HistoryPageProps {
  onOpenHistorySettings: () => void;
}

const HistoryPage: React.FC<HistoryPageProps> = memo(({ onOpenHistorySettings }) => {
  const { transcriptionHistory } = useStore();
  
  const [selectedEntry, setSelectedEntry] = useState<TranscriptionEntry | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<'timestamp' | 'confidence' | 'duration'>('timestamp');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [filterModel, setFilterModel] = useState<string>('all');

  // 筛选和排序历史记录
  const filteredAndSortedHistory = useMemo(() => {
    let filtered = transcriptionHistory.filter(entry => {
      const matchesSearch = entry.text.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesModel = filterModel === 'all' || entry.model === filterModel;
      return matchesSearch && matchesModel;
    });

    return filtered.sort((a, b) => {
      let comparison = 0;
      switch (sortBy) {
        case 'timestamp':
          comparison = a.timestamp - b.timestamp;
          break;
        case 'confidence':
          comparison = a.confidence - b.confidence;
          break;
        case 'duration':
          comparison = a.duration - b.duration;
          break;
      }
      return sortOrder === 'asc' ? comparison : -comparison;
    });
  }, [transcriptionHistory, searchQuery, sortBy, sortOrder, filterModel]);

  // 获取所有使用过的模型
  const usedModels = useMemo(() => {
    const models = [...new Set(transcriptionHistory.map(entry => entry.model))];
    return models.sort();
  }, [transcriptionHistory]);

  const handleEntryClick = useCallback((entry: TranscriptionEntry) => {
    setSelectedEntry(entry);
  }, []);

  const handleCloseDetail = useCallback(() => {
    setSelectedEntry(null);
  }, []);

  const handleSearch = useCallback((query: string) => {
    setSearchQuery(query);
  }, []);

  const formatDuration = useCallback((duration: number) => {
    const minutes = Math.floor(duration / 60);
    const seconds = Math.round(duration % 60);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }, []);

  const formatFileSize = useCallback((sizeInBytes: number) => {
    if (sizeInBytes < 1024) return `${sizeInBytes}B`;
    if (sizeInBytes < 1024 * 1024) return `${(sizeInBytes / 1024).toFixed(1)}KB`;
    return `${(sizeInBytes / (1024 * 1024)).toFixed(1)}MB`;
  }, []);

  const getConfidenceClass = useCallback((confidence: number) => {
    if (confidence >= 0.9) return 'confidence-high';
    if (confidence >= 0.7) return 'confidence-medium';
    return 'confidence-low';
  }, []);

  return (
    <div className="history-page">
      <div className="history-header">
        <h2>转录历史</h2>
        <div className="history-actions">
          <button onClick={onOpenHistorySettings} className="settings-btn">
            ⚙️ 历史设置
          </button>
        </div>
      </div>

      {/* 搜索和筛选控件 */}
      <div className="history-controls">
        <div className="search-bar">
          <input
            type="text"
            placeholder="搜索转录内容..."
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
            className="search-input"
          />
          <span className="search-icon">🔍</span>
        </div>

        <div className="filter-controls">
          <select
            value={filterModel}
            onChange={(e) => setFilterModel(e.target.value)}
            className="model-filter"
          >
            <option value="all">所有模型</option>
            {usedModels.map(model => (
              <option key={model} value={model}>{model}</option>
            ))}
          </select>

          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="sort-by"
          >
            <option value="timestamp">按时间</option>
            <option value="confidence">按置信度</option>
            <option value="duration">按时长</option>
          </select>

          <button
            onClick={() => setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')}
            className="sort-order-btn"
          >
            {sortOrder === 'asc' ? '↑' : '↓'}
          </button>
        </div>
      </div>

      {/* 统计信息 */}
      <div className="history-stats">
        <div className="stat-item">
          <span className="stat-label">总数:</span>
          <span className="stat-value">{transcriptionHistory.length}</span>
        </div>
        <div className="stat-item">
          <span className="stat-label">筛选后:</span>
          <span className="stat-value">{filteredAndSortedHistory.length}</span>
        </div>
        <div className="stat-item">
          <span className="stat-label">总时长:</span>
          <span className="stat-value">
            {formatDuration(transcriptionHistory.reduce((sum, entry) => sum + entry.duration, 0))}
          </span>
        </div>
        <div className="stat-item">
          <span className="stat-label">平均置信度:</span>
          <span className="stat-value">
            {transcriptionHistory.length > 0 
              ? Math.round(transcriptionHistory.reduce((sum, entry) => sum + entry.confidence, 0) / transcriptionHistory.length * 100)
              : 0}%
          </span>
        </div>
      </div>

      {/* 历史记录列表 */}
      <div className="history-list">
        {filteredAndSortedHistory.length > 0 ? (
          filteredAndSortedHistory.map((entry) => (
            <div 
              key={entry.id}
              className="history-entry"
              onClick={() => handleEntryClick(entry)}
            >
              <div className="entry-content">
                <div className="entry-text">
                  {entry.text.substring(0, 200)}
                  {entry.text.length > 200 && '...'}
                </div>
                <div className="entry-meta">
                  <div className="meta-row">
                    <span className="timestamp">
                      📅 {new Date(entry.timestamp).toLocaleString()}
                    </span>
                    <span className="duration">
                      ⏱️ {formatDuration(entry.duration)}
                    </span>
                  </div>
                  <div className="meta-row">
                    <span className="model-badge">
                      🤖 {entry.model}
                    </span>
                    <span className={`confidence ${getConfidenceClass(entry.confidence)}`}>
                      📊 {Math.round(entry.confidence * 100)}%
                    </span>
                  </div>
                  {entry.audio_file_path && (
                    <div className="meta-row">
                      <span className="file-path">
                        🔗 {entry.audio_file_path.split('/').pop()}
                      </span>
                    </div>
                  )}
                </div>
              </div>
              <div className="entry-actions">
                <button className="action-btn view-btn">查看</button>
                <button className="action-btn copy-btn">复制</button>
                <button className="action-btn export-btn">导出</button>
              </div>
            </div>
          ))
        ) : (
          <div className="empty-history">
            {searchQuery || filterModel !== 'all' ? (
              <div className="no-results">
                <h3>未找到匹配的记录</h3>
                <p>尝试调整搜索条件或筛选器</p>
                <button 
                  onClick={() => {
                    setSearchQuery('');
                    setFilterModel('all');
                  }}
                  className="clear-filters-btn"
                >
                  清除筛选条件
                </button>
              </div>
            ) : (
              <div className="no-history">
                <h3>暂无转录历史</h3>
                <p>开始录音后，您的转录历史将在这里显示</p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* 详情视图模态框 */}
      {selectedEntry && (
        <div className="detail-modal-overlay" onClick={handleCloseDetail}>
          <div className="detail-modal" onClick={(e) => e.stopPropagation()}>
            <TranscriptionDetailView
              entry={selectedEntry}
              onClose={handleCloseDetail}
              isVisible={true}
            />
          </div>
        </div>
      )}
    </div>
  );
});

HistoryPage.displayName = 'HistoryPage';

export default HistoryPage;