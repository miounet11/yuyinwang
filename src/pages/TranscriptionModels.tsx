/**
 * TranscriptionModels - 听写模型页面
 * 复刻第二张截图的设计：模型选择和管理界面
 */

import React, { useState, useMemo } from 'react';
import { motion } from 'framer-motion';
import { 
  SpokenlyContent,
  SpokenlyTag,
  SpokenlyModelCard,
  SpokenlyButton
} from '../components/ui';

// 模型数据类型
interface TranscriptionModel {
  id: string;
  title: string;
  description: string;
  provider: 'openai' | 'deepgram' | 'local' | 'azure';
  status: {
    type: 'online' | 'offline' | 'loading' | 'error';
    message?: string;
  };
  tags: string[];
  isSelected?: boolean;
  accuracy: number; // 1-5 准确度等级
  speed: number; // 1-5 速度等级
  pricing?: string;
  features: string[];
}

// 模拟模型数据
const mockModels: TranscriptionModel[] = [
  {
    id: 'whisper-1',
    title: 'Whisper-1',
    description: 'OpenAI 的高质量语音识别模型，支持多种语言',
    provider: 'openai',
    status: { type: 'online' },
    tags: ['最新', '当前使用', '准确', '多语言'],
    isSelected: true,
    accuracy: 5,
    speed: 4,
    pricing: '$0.006/分钟',
    features: ['多语言支持', '高准确度', '标点符号']
  },
  {
    id: 'deepgram-nova-2',
    title: 'Deepgram Nova-2',
    description: '最新的实时语音识别 API，专为快速转录优化',
    provider: 'deepgram',
    status: { type: 'online' },
    tags: ['快速', '实时', '最新'],
    accuracy: 4,
    speed: 5,
    pricing: '$0.0043/分钟',
    features: ['实时转录', '低延迟', '高速处理']
  },
  {
    id: 'whisper-local',
    title: 'Whisper (本地)',
    description: '在您的设备上本地运行的 Whisper 模型',
    provider: 'local',
    status: { type: 'offline', message: '未安装' },
    tags: ['本地', '离线', '隐私'],
    accuracy: 4,
    speed: 3,
    features: ['离线使用', '数据隐私', '无网络要求']
  },
  {
    id: 'azure-speech',
    title: 'Azure 语音服务',
    description: '微软认知服务的语音转文本 API',
    provider: 'azure',
    status: { type: 'online' },
    tags: ['企业级', '稳定'],
    accuracy: 4,
    speed: 4,
    pricing: '$1.00/小时',
    features: ['企业级', '自定义模型', '批量处理']
  },
  {
    id: 'whisper-large',
    title: 'Whisper Large (本地)',
    description: '最大的本地 Whisper 模型，提供最高准确度',
    provider: 'local',
    status: { type: 'offline', message: '可下载' },
    tags: ['本地', '准确', '大模型'],
    accuracy: 5,
    speed: 2,
    features: ['最高准确度', '99种语言', '大词汇量']
  }
];

// 筛选标签
const filterTags = [
  { value: 'all', label: '全部' },
  { value: 'online', label: '在线' },
  { value: 'local', label: '本地' },
  { value: 'api', label: 'API' },
  { value: 'fast', label: '快速' },
  { value: 'accurate', label: '准确' },
  { value: 'punctuation', label: '标点符号' },
  { value: 'subtitle', label: '字幕' }
];

interface TranscriptionModelsProps {
  className?: string;
}

export const TranscriptionModels: React.FC<TranscriptionModelsProps> = ({
  className = ''
}) => {
  const [selectedFilter, setSelectedFilter] = useState('all');
  const [selectedModel, setSelectedModel] = useState('whisper-1');

  // 根据筛选条件过滤模型
  const filteredModels = useMemo(() => {
    if (selectedFilter === 'all') return mockModels;
    
    return mockModels.filter(model => {
      switch (selectedFilter) {
        case 'online':
          return model.status.type === 'online';
        case 'local':
          return model.provider === 'local';
        case 'api':
          return model.provider !== 'local';
        case 'fast':
          return model.speed >= 4;
        case 'accurate':
          return model.accuracy >= 4;
        case 'punctuation':
          return model.features.includes('标点符号') || model.tags.includes('标点符号');
        case 'subtitle':
          return model.tags.includes('字幕') || model.features.includes('字幕');
        default:
          return true;
      }
    });
  }, [selectedFilter]);

  const handleModelSelect = (modelId: string) => {
    setSelectedModel(modelId);
    // 这里可以添加实际的模型切换逻辑
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
        staggerChildren: 0.1
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
        className="transcription-models"
        variants={pageVariants}
        initial="initial"
        animate="animate"
        style={{
          width: '100%',
          maxWidth: '1000px',
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
            听写模型
          </h1>
          
          {/* 筛选标签 */}
          <div 
            style={{
              display: 'flex',
              flexWrap: 'wrap',
              gap: 'var(--spokenly-space-2)',
              marginTop: 'var(--spokenly-space-4)'
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
        </div>

        {/* 模型列表 */}
        <motion.div
          className="models-grid"
          variants={listVariants}
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(400px, 1fr))',
            gap: 'var(--spokenly-space-4)',
            width: '100%'
          }}
        >
          {filteredModels.map(model => (
            <motion.div key={model.id} variants={itemVariants}>
              <SpokenlyModelCard
                title={model.title}
                description={model.description}
                provider={model.provider}
                status={model.status}
                isSelected={selectedModel === model.id}
                onSelect={() => handleModelSelect(model.id)}
                tags={model.tags}
                pricing={model.pricing}
                style={{
                  height: '100%',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                  border: selectedModel === model.id 
                    ? '2px solid var(--spokenly-primary-500)' 
                    : '1px solid var(--spokenly-border-default)'
                }}
                actions={
                  <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spokenly-space-2)' }}>
                    {/* 准确度指示器 */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spokenly-space-1)' }}>
                      <div style={{ 
                        fontSize: 'var(--spokenly-text-xs)', 
                        color: 'var(--spokenly-text-tertiary)' 
                      }}>
                        准确度
                      </div>
                      <div style={{ display: 'flex', gap: '2px' }}>
                        {[1, 2, 3, 4, 5].map(level => (
                          <div
                            key={level}
                            style={{
                              width: '8px',
                              height: '8px',
                              borderRadius: '50%',
                              backgroundColor: level <= model.accuracy 
                                ? 'var(--spokenly-success-500)' 
                                : 'var(--spokenly-gray-300)'
                            }}
                          />
                        ))}
                      </div>
                    </div>
                    
                    {/* 速度指示器 */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spokenly-space-1)' }}>
                      <div style={{ 
                        fontSize: 'var(--spokenly-text-xs)', 
                        color: 'var(--spokenly-text-tertiary)' 
                      }}>
                        速度
                      </div>
                      <div style={{ display: 'flex', gap: '2px' }}>
                        {[1, 2, 3, 4, 5].map(level => (
                          <div
                            key={level}
                            style={{
                              width: '8px',
                              height: '8px',
                              borderRadius: '50%',
                              backgroundColor: level <= model.speed 
                                ? 'var(--spokenly-primary-500)' 
                                : 'var(--spokenly-gray-300)'
                            }}
                          />
                        ))}
                      </div>
                    </div>
                  </div>
                }
              />
            </motion.div>
          ))}
        </motion.div>

        {/* 底部信息 */}
        <div 
          style={{
            marginTop: 'var(--spokenly-space-8)',
            padding: 'var(--spokenly-space-4)',
            backgroundColor: 'var(--spokenly-bg-subtle)',
            borderRadius: 'var(--spokenly-radius-md)',
            border: '1px solid var(--spokenly-border-subtle)'
          }}
        >
          <p style={{ 
            fontSize: 'var(--spokenly-text-sm)', 
            color: 'var(--spokenly-text-secondary)',
            margin: 0,
            lineHeight: 1.5
          }}>
            💡 提示：选择合适的模型可以显著提升转录质量。在线模型通常具有更好的准确性，而本地模型提供更好的隐私保护。
          </p>
        </div>
      </motion.div>
    </SpokenlyContent>
  );
};