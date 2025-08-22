/**
 * 历史记录页面
 * 完美复刻 Spokenly 第四张截图的设计
 */

import React, { useState } from 'react';
import './HistoryRecords.css';
import { SpokenlyCard, SpokenlyButton, SpokenlyInput } from '../ui';

interface HistoryItem {
  id: string;
  type: 'dictation' | 'file' | 'diary';
  content: string;
  timestamp: string;
  duration: string;
  expanded?: boolean;
}

interface HistoryRecordsProps {}

const HistoryRecords: React.FC<HistoryRecordsProps> = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedFilter, setSelectedFilter] = useState('全部');
  const [sortBy, setSortBy] = useState('最新的在前');
  const [historyItems, setHistoryItems] = useState<HistoryItem[]>([
    {
      id: '1',
      type: 'dictation',
      content: '目前设置是有广告问题的，页面打开了之后很多的指标，我们联系的设置是有很大的问题，你需要对己进行了个设置，包括我们在新增模型和初次模型的时候经过个页面里面都没有显示，你们需要让所有的agent配合一起解决这个问题。',
      timestamp: '2w ago',
      duration: '29 seconds'
    },
    {
      id: '2',
      type: 'dictation',
      content: '有问题',
      timestamp: '2w ago',
      duration: '2 seconds'
    },
    {
      id: '3',
      type: 'dictation',
      content: '',
      timestamp: '2w ago',
      duration: '1 second'
    },
    {
      id: '4',
      type: 'dictation',
      content: '',
      timestamp: '2w ago',
      duration: '2 seconds'
    }
  ]);

  const filters = ['全部', '听写', '文件', '日记'];

  const filteredItems = historyItems.filter(item => {
    const matchesFilter = selectedFilter === '全部' || 
      (selectedFilter === '听写' && item.type === 'dictation') ||
      (selectedFilter === '文件' && item.type === 'file') ||
      (selectedFilter === '日记' && item.type === 'diary');
    
    const matchesSearch = item.content.toLowerCase().includes(searchQuery.toLowerCase());
    
    return matchesFilter && matchesSearch;
  });

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'dictation':
        return '🎤';
      case 'file':
        return '📄';
      case 'diary':
        return '📔';
      default:
        return '🎤';
    }
  };

  const toggleItemExpansion = (id: string) => {
    setHistoryItems(items =>
      items.map(item =>
        item.id === id ? { ...item, expanded: !item.expanded } : item
      )
    );
  };

  const deleteItem = (id: string) => {
    setHistoryItems(items => items.filter(item => item.id !== id));
  };

  return (
    <div className="spokenly-page">
      <div className="spokenly-page-header">
        <h1>历史记录</h1>
        <p>查看存储在您电脑上的转录历史记录</p>
      </div>

      <div className="spokenly-page-content">
        {/* 筛选和搜索区域 */}
        <div className="history-controls">
          {/* 筛选标签 */}
          <div className="filter-tabs">
            {filters.map((filter) => (
              <button
                key={filter}
                className={`filter-tab ${selectedFilter === filter ? 'active' : ''}`}
                onClick={() => setSelectedFilter(filter)}
              >
                {filter}
              </button>
            ))}
          </div>

          {/* 搜索和排序 */}
          <div className="search-and-sort">
            <div className="search-box">
              <SpokenlyInput
                type="text"
                placeholder="搜索"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                leftIcon="🔍"
              />
            </div>

            <div className="sort-controls">
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value)}
                className="sort-select"
              >
                <option value="最新的在前">最新的在前</option>
                <option value="最旧的在前">最旧的在前</option>
                <option value="时长">按时长</option>
              </select>
              
              <div className="view-options">
                <button className="view-option active">📋</button>
                <button className="view-option">⚙️</button>
              </div>
            </div>
          </div>
        </div>

        {/* 历史记录列表 */}
        <div className="history-list">
          {filteredItems.length === 0 ? (
            <SpokenlyCard>
              <div className="empty-state">
                <p>没有找到匹配的记录</p>
              </div>
            </SpokenlyCard>
          ) : (
            filteredItems.map((item) => (
              <SpokenlyCard key={item.id} className="history-item">
                <div className="history-item-header">
                  <div className="history-item-meta">
                    <span className="history-type-icon">
                      {getTypeIcon(item.type)}
                    </span>
                    <span className="history-timestamp">{item.timestamp}</span>
                    <span className="history-duration">{item.duration}</span>
                  </div>
                  
                  <div className="history-item-actions">
                    <button 
                      className="action-btn edit"
                      onClick={() => {/* 编辑功能 */}}
                      title="编辑"
                    >
                      ✏️
                    </button>
                    <button 
                      className="action-btn share"
                      onClick={() => {/* 分享功能 */}}
                      title="分享"
                    >
                      📤
                    </button>
                    <button 
                      className="action-btn delete"
                      onClick={() => deleteItem(item.id)}
                      title="删除"
                    >
                      🗑️
                    </button>
                  </div>
                </div>

                <div className="history-item-content">
                  {item.content ? (
                    <div className="content-text">
                      {item.expanded || item.content.length <= 100 ? (
                        <p>{item.content}</p>
                      ) : (
                        <>
                          <p>{item.content.substring(0, 100)}...</p>
                          <button
                            className="expand-btn"
                            onClick={() => toggleItemExpansion(item.id)}
                          >
                            显示更多
                          </button>
                        </>
                      )}
                      
                      {item.expanded && item.content.length > 100 && (
                        <button
                          className="expand-btn"
                          onClick={() => toggleItemExpansion(item.id)}
                        >
                          收起
                        </button>
                      )}
                    </div>
                  ) : (
                    <div className="empty-content">
                      <p className="no-content">无内容</p>
                    </div>
                  )}
                </div>
              </SpokenlyCard>
            ))
          )}
        </div>

        {/* 批量操作 */}
        {filteredItems.length > 0 && (
          <div className="bulk-actions">
            <SpokenlyButton variant="secondary" size="sm">
              选择多个
            </SpokenlyButton>
            
            <SpokenlyButton variant="secondary" size="sm">
              导出所有
            </SpokenlyButton>
            
            <SpokenlyButton variant="danger" size="sm">
              清空历史记录
            </SpokenlyButton>
          </div>
        )}
      </div>
    </div>
  );
};

export default HistoryRecords;