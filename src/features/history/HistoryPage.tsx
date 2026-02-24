import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useAppStore } from '../../shared/stores/useAppStore';
import './HistoryPage.css';

export const HistoryPage: React.FC = () => {
  const { history, setHistory, addToast, settings } = useAppStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  useEffect(() => { loadHistory(); }, []);

  const loadHistory = async () => {
    setIsLoading(true);
    try {
      const entries = await invoke('get_history', { limit: 100 });
      setHistory(entries as any[]);
    } catch (e) { console.error(e); }
    finally { setIsLoading(false); }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) { loadHistory(); return; }
    try {
      const results = await invoke('search_history', { query: searchQuery, limit: 50 });
      setHistory(results as any[]);
    } catch (e) { console.error(e); }
  };

  const handleDelete = async (id: string) => {
    if (confirmDeleteId !== id) {
      setConfirmDeleteId(id);
      setTimeout(() => setConfirmDeleteId(null), 3000);
      return;
    }
    try {
      await invoke('delete_entry', { id });
      setHistory(history.filter((e) => e.id !== id));
      addToast('success', '已删除');
      setConfirmDeleteId(null);
    } catch (e) { addToast('error', '删除失败'); }
  };

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
    addToast('success', '已复制');
  };

  const handleInject = async (text: string) => {
    try {
      await invoke('inject_text', { text, delayMs: settings.inject_delay_ms });
      addToast('success', '已注入到当前应用');
    } catch (e) {
      addToast('error', `注入失败: ${e}`);
    }
  };

  const handleClearAll = async () => {
    if (!confirm('确定要清空所有历史记录吗？此操作不可撤销。')) return;
    try {
      for (const entry of history) {
        await invoke('delete_entry', { id: entry.id });
      }
      setHistory([]);
      addToast('success', '历史记录已清空');
    } catch (e) {
      addToast('error', '清空失败');
    }
  };

  const formatTime = (timestamp: number) => {
    const ms = timestamp < 1e12 ? timestamp * 1000 : timestamp;
    const date = new Date(ms);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    if (diff < 60000) return '刚刚';
    if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`;
    if (diff < 172800000) return '昨天 ' + date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
    return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' }) + ' ' +
           date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  };

  const formatDuration = (seconds: number) => {
    if (!seconds || seconds <= 0) return '';
    if (seconds < 60) return `${Math.round(seconds)}秒`;
    return `${Math.floor(seconds / 60)}分${Math.round(seconds % 60)}秒`;
  };

  return (
    <div className="history-page">
      <div className="history-header">
        <div>
          <h1>历史记录</h1>
          <span className="entry-count">共 {history.length} 条记录</span>
        </div>
        {history.length > 0 && (
          <button className="clear-all-btn" onClick={handleClearAll}>清空</button>
        )}
      </div>

      <div className="search-bar">
        <span className="search-icon">🔍</span>
        <input
          type="text"
          placeholder="搜索转录内容..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
        />
        {searchQuery && (
          <button className="search-clear" onClick={() => { setSearchQuery(''); loadHistory(); }}>✕</button>
        )}
      </div>

      <div className="history-list">
        {isLoading ? (
          <div className="empty-state">
            <div className="loading-dots">
              <span /><span /><span />
            </div>
            <p style={{ color: 'var(--text-muted)', marginTop: '12px' }}>加载中...</p>
          </div>
        ) : history.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">📝</div>
            <p className="empty-title">{searchQuery ? '未找到匹配结果' : '暂无转录记录'}</p>
            <p className="empty-desc">{searchQuery ? '尝试其他关键词' : '开始录音后，转录结果会显示在这里'}</p>
          </div>
        ) : (
          history.map((entry) => {
            const isExpanded = expandedId === entry.id;
            return (
              <div key={entry.id} className={`history-card ${isExpanded ? 'expanded' : ''}`}>
                <div className="card-text"
                  onClick={() => setExpandedId(isExpanded ? null : entry.id)}
                  style={{ cursor: 'pointer', WebkitLineClamp: isExpanded ? 'unset' : 3 }}
                >
                  {entry.text}
                </div>
                <div className="card-meta">
                  {entry.audio_file_path && <span className="card-source">📁 文件转录</span>}
                  <span>⏱ {formatTime(entry.timestamp)}</span>
                  {entry.duration > 0 && <span>🕐 {formatDuration(entry.duration)}</span>}
                  <span className="card-model">{entry.model}</span>
                  <span>{Math.round(entry.confidence * 100)}%</span>
                </div>
                <div className="card-actions">
                  <button className="action-icon-btn" title="注入到当前应用" onClick={() => handleInject(entry.text)}>📝</button>
                  <button className="action-icon-btn" title="复制" onClick={() => handleCopy(entry.text)}>📋</button>
                  <button
                    className={`action-icon-btn ${confirmDeleteId === entry.id ? 'confirm-delete' : 'danger'}`}
                    title={confirmDeleteId === entry.id ? '再次点击确认删除' : '删除'}
                    onClick={() => handleDelete(entry.id)}
                  >
                    {confirmDeleteId === entry.id ? '确认?' : '🗑'}
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
