/**
 * HistoryRecords - 历史记录页面
 * 复刻第四张截图的设计：历史记录查看和管理界面
 */

import React, { useState, useMemo } from 'react';
import { motion } from 'framer-motion';
import { 
  SpokenlyContent,
  SpokenlyTag,
  SpokenlyHistoryItem,
  SpokenlySearchBox,
  SpokenlyButton
} from '../components/ui';

// 历史记录类型
interface HistoryRecord {
  id: string;
  title: string;
  content: string;
  timestamp: Date;
  duration: number;
  fileSize?: number;
  format?: string;
  type: 'dictation' | 'file' | 'journal';
}

// 模拟历史记录数据
const mockHistoryRecords: HistoryRecord[] = [
  {
    id: '1',
    title: '会议纪要',
    content: '今天的产品会议讨论了新功能的开发进度，预计在下个月完成第一版的设计稿...',
    timestamp: new Date(Date.now() - 2 * 7 * 24 * 60 * 60 * 1000), // 2 weeks ago
    duration: 29,
    type: 'dictation'
  },
  {
    id: '2',
    title: '项目总结',
    content: '本季度项目总结：完成了用户界面的重新设计，提升了用户体验和界面美观度...',
    timestamp: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000), // 3 days ago
    duration: 124,
    type: 'dictation'
  },
  {
    id: '3',
    title: '客户访谈录音',
    content: '用户反馈产品功能丰富，但希望能够简化操作流程，特别是初次使用的引导...',
    timestamp: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000), // 5 days ago
    duration: 1842,
    fileSize: 15.2 * 1024 * 1024,
    format: 'MP3',
    type: 'file'
  },
  {
    id: '4',
    title: '日记 - 产品思考',
    content: '思考如何让语音输入更加自然，用户不应该感觉到技术的存在，而是专注于内容创作...',
    timestamp: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000), // 1 day ago
    duration: 67,
    type: 'journal'
  },
  {
    id: '5',
    title: '技术讨论',
    content: '关于 AI 模型的选择和优化策略，需要平衡准确度、速度和成本之间的关系...',
    timestamp: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000), // 1 week ago
    duration: 205,
    type: 'dictation'
  },
  {
    id: '6',
    title: '播客录音转录',
    content: '今天的播客讨论了人工智能在创作工具中的应用，以及未来可能的发展方向...',
    timestamp: new Date(Date.now() - 10 * 24 * 60 * 60 * 1000), // 10 days ago
    duration: 3245,
    fileSize: 28.7 * 1024 * 1024,
    format: 'M4A',
    type: 'file'
  }
];

// 筛选标签
const filterTags = [
  { value: 'all', label: '全部' },
  { value: 'dictation', label: '听写' },
  { value: 'file', label: '文件' },
  { value: 'journal', label: '日记' }
];

interface HistoryRecordsProps {
  className?: string;
}

export const HistoryRecords: React.FC<HistoryRecordsProps> = ({
  className = ''
}) => {
  const [selectedFilter, setSelectedFilter] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedRecords, setSelectedRecords] = useState<string[]>([]);

  // 根据筛选条件和搜索查询过滤记录
  const filteredRecords = useMemo(() => {
    let filtered = mockHistoryRecords;

    // 按类型筛选
    if (selectedFilter !== 'all') {
      filtered = filtered.filter(record => record.type === selectedFilter);
    }

    // 按搜索查询筛选
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(record => 
        record.title.toLowerCase().includes(query) ||
        record.content.toLowerCase().includes(query)
      );
    }

    // 按时间排序（最新的在前）
    return filtered.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
  }, [selectedFilter, searchQuery]);

  // 格式化时间显示
  const formatTimeAgo = (date: Date): string => {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const diffWeeks = Math.floor(diffDays / 7);

    if (diffDays === 0) return '今天';
    if (diffDays === 1) return '1天前';
    if (diffDays < 7) return `${diffDays}天前`;
    if (diffWeeks === 1) return '1周前';
    if (diffWeeks < 4) return `${diffWeeks}周前`;
    
    const diffMonths = Math.floor(diffDays / 30);
    if (diffMonths === 1) return '1个月前';
    if (diffMonths < 12) return `${diffMonths}个月前`;
    
    const diffYears = Math.floor(diffDays / 365);
    return `${diffYears}年前`;
  };

  // 处理记录选择
  const handleRecordSelect = (recordId: string) => {
    setSelectedRecords(prev => {
      if (prev.includes(recordId)) {
        return prev.filter(id => id !== recordId);
      } else {
        return [...prev, recordId];
      }
    });
  };

  // 处理记录操作
  const handleRecordPlay = (recordId: string) => {
    console.log('播放记录:', recordId);
  };

  const handleRecordDelete = (recordId: string) => {
    console.log('删除记录:', recordId);
  };

  const handleRecordExport = (recordId: string) => {
    console.log('导出记录:', recordId);
  };

  const pageVariants = {
    initial: { opacity: 0, y: 20 },
    animate: { 
      opacity: 1, 
      y: 0,
      transition: {
        duration: 0.6,
        ease: [0.0, 0.0, 0.2, 1]
      }
    }
  };

  const listVariants = {
    animate: {
      transition: {
        staggerChildren: 0.05
      }
    }
  };

  const itemVariants = {
    initial: { opacity: 0, y: 20 },
    animate: { 
      opacity: 1, 
      y: 0,
      transition: {
        duration: 0.4,
        ease: [0.0, 0.0, 0.2, 1]
      }
    }
  };

  return (
    <SpokenlyContent className={className}>
      <motion.div
        className="history-records"
        variants={pageVariants}
        initial="initial"
        animate="animate"
        style={{
          width: '100%',
          maxWidth: '900px',
          margin: '0 auto'
        }}
      >
        {/* 页面标题 */}
        <div 
          className="page-header"
          style={{
            marginBottom: 'var(--spokenly-space-6)',
            paddingBottom: 'var(--spokenly-space-4)',
            borderBottom: '1px solid var(--spokenly-border-subtle)'
          }}
        >
          <h1 
            style={{
              fontSize: 'var(--spokenly-text-2xl)',
              fontWeight: 600,
              color: 'var(--spokenly-text-primary)',
              margin: 0,
              marginBottom: 'var(--spokenly-space-2)'
            }}
          >
            历史记录
          </h1>
          <p 
            style={{
              fontSize: 'var(--spokenly-text-sm)',
              color: 'var(--spokenly-text-secondary)',
              margin: 0
            }}
          >
            查看存储在您电脑上的转录历史记录
          </p>
        </div>

        {/* 筛选和搜索 */}
        <div style={{ marginBottom: 'var(--spokenly-space-6)' }}>
          {/* 筛选标签 */}
          <div 
            style={{
              display: 'flex',
              flexWrap: 'wrap',
              gap: 'var(--spokenly-space-2)',
              marginBottom: 'var(--spokenly-space-4)'
            }}
          >
            {filterTags.map(tag => (
              <SpokenlyTag
                key={tag.value}
                variant={selectedFilter === tag.value ? 'info' : 'default'}
                size="sm"
                style={{
                  cursor: 'pointer',
                  backgroundColor: selectedFilter === tag.value 
                    ? 'var(--spokenly-primary-500)' 
                    : undefined,
                  color: selectedFilter === tag.value 
                    ? 'white' 
                    : undefined
                }}
                onClick={() => setSelectedFilter(tag.value)}
              >
                {tag.label}
              </SpokenlyTag>
            ))}
          </div>

          {/* 搜索框 */}
          <SpokenlySearchBox
            placeholder="搜索历史记录..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onSearch={setSearchQuery}
            onClear={() => setSearchQuery('')}
            showClearButton={searchQuery.length > 0}
            style={{
              maxWidth: '400px'
            }}
          />
        </div>

        {/* 记录列表 */}
        <motion.div
          className="records-list"
          variants={listVariants}
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--spokenly-space-3)'
          }}
        >
          {filteredRecords.length === 0 ? (
            <div style={{
              padding: 'var(--spokenly-space-8)',
              textAlign: 'center',
              color: 'var(--spokenly-text-tertiary)'
            }}>
              <p>没有找到匹配的记录</p>
            </div>
          ) : (
            filteredRecords.map(record => (
              <motion.div key={record.id} variants={itemVariants}>
                <SpokenlyHistoryItem
                  id={record.id}
                  title={record.title}
                  content={record.content}
                  timestamp={record.timestamp}
                  duration={record.duration}
                  fileSize={record.fileSize}
                  format={record.format}
                  isSelected={selectedRecords.includes(record.id)}
                  onSelect={handleRecordSelect}
                  onPlay={handleRecordPlay}
                  onDelete={handleRecordDelete}
                  onExport={handleRecordExport}
                  style={{
                    transition: 'all 0.2s ease',
                    cursor: 'pointer'
                  }}
                />
                
                {/* 自定义时间显示 */}
                <div style={{
                  position: 'absolute',
                  top: 'var(--spokenly-space-3)',
                  right: 'var(--spokenly-space-4)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 'var(--spokenly-space-2)',
                  fontSize: 'var(--spokenly-text-xs)',
                  color: 'var(--spokenly-text-tertiary)',
                  pointerEvents: 'none'
                }}>
                  <span>{formatTimeAgo(record.timestamp)}</span>
                  <span>•</span>
                  <span>
                    {Math.floor(record.duration / 60)}:{(record.duration % 60).toString().padStart(2, '0')} 分钟
                  </span>
                </div>
              </motion.div>
            ))
          )}
        </motion.div>

        {/* 批量操作 */}
        {selectedRecords.length > 0 && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            style={{
              position: 'fixed',
              bottom: 'var(--spokenly-space-6)',
              left: '50%',
              transform: 'translateX(-50%)',
              display: 'flex',
              gap: 'var(--spokenly-space-3)',
              padding: 'var(--spokenly-space-4)',
              backgroundColor: 'var(--spokenly-bg-primary)',
              border: '1px solid var(--spokenly-border-default)',
              borderRadius: 'var(--spokenly-radius-lg)',
              boxShadow: 'var(--spokenly-shadow-lg)',
              zIndex: 10
            }}
          >
            <span style={{ 
              fontSize: 'var(--spokenly-text-sm)',
              color: 'var(--spokenly-text-secondary)',
              alignSelf: 'center'
            }}>
              已选择 {selectedRecords.length} 项
            </span>
            
            <SpokenlyButton variant="secondary" size="sm">
              导出选中
            </SpokenlyButton>
            
            <SpokenlyButton variant="danger" size="sm">
              删除选中
            </SpokenlyButton>
            
            <SpokenlyButton 
              variant="ghost" 
              size="sm"
              onClick={() => setSelectedRecords([])}
            >
              取消
            </SpokenlyButton>
          </motion.div>
        )}
      </motion.div>
    </SpokenlyContent>
  );
};