/**
 * 听写模型页面
 * 完美复刻 Spokenly 第二张截图的设计
 * 注意：已移除测试功能，适用于正式版本
 */

import React, { useState } from 'react';
import './TranscriptionModels.css';
import { SpokenlyCard, SpokenlyTag, SpokenlyButton } from '../ui';

interface ModelCardProps {
  id: string;
  name: string;
  provider: string;
  description: string;
  accuracy: number;
  speed: number;
  isSelected?: boolean;
  isRecommended?: boolean;
  isCurrent?: boolean;
  tags?: string[];
  languages?: string[];
  onSelect?: () => void;
}

const ModelCard: React.FC<ModelCardProps> = ({
  id,
  name,
  provider,
  description,
  accuracy,
  speed,
  isSelected = false,
  isRecommended = false,
  isCurrent = false,
  tags = [],
  languages = [],
  onSelect
}) => {
  const renderStatusDots = (count: number, filled: number) => {
    return Array.from({ length: count }, (_, i) => (
      <span
        key={i}
        className={`status-dot ${i < filled ? 'filled' : 'empty'}`}
      />
    ));
  };

  return (
    <SpokenlyCard className={`model-card ${isSelected ? 'selected' : ''}`}>
      <div className="model-header">
        <div className="model-info">
          <h4 className="model-name">{name}</h4>
          <p className="model-provider">由{provider}</p>
          <p className="model-description">{description}</p>
        </div>
        {isCurrent && (
          <div className="current-indicator">
            <div className="current-dot"></div>
          </div>
        )}
      </div>

      <div className="model-stats">
        <div className="stat-item">
          <span className="stat-label">准确度</span>
          <div className="stat-dots">
            {renderStatusDots(4, accuracy)}
          </div>
        </div>
        
        <div className="stat-item">
          <span className="stat-label">速度</span>
          <div className="stat-dots">
            {renderStatusDots(5, speed)}
          </div>
        </div>

        <div className="stat-item">
          <span className="stat-label">支持语言</span>
          <div className="language-count">
            <span>{languages.length > 0 ? `${languages.length}种语言` : '多语言'}</span>
          </div>
        </div>

        {tags.length > 0 && (
          <div className="stat-item">
            <div className="model-tags">
              {tags.map((tag, index) => (
                <SpokenlyTag key={index} variant="info" size="sm">
                  {tag}
                </SpokenlyTag>
              ))}
            </div>
          </div>
        )}
      </div>
    </SpokenlyCard>
  );
};

const TranscriptionModels: React.FC = () => {
  const [selectedFilter, setSelectedFilter] = useState('全部');
  const [selectedModel, setSelectedModel] = useState('gpt-4o-mini');

  const filters = [
    '全部', '在线', '本地', 'API', '快速', '准确', '标点符号', '字幕'
  ];

  const models = [
    {
      id: 'nova-3-english',
      name: 'Online Real-time Nova-3 (English Only)',
      provider: 'Deepgram Nova-3驱动',
      description: '实时听写具有卓越准确性，比英语优化版本。',
      accuracy: 4,
      speed: 5,
      isRecommended: true,
      tags: ['最新'],
      languages: ['English']
    },
    {
      id: 'gpt-4o-mini',
      name: 'Online GPT-4o mini Transcribe',
      provider: 'OpenAI GPT-4o mini驱动',
      description: '卓越准确性和较处理，比Whisper或Nova模型更高。',
      accuracy: 4,
      speed: 2,
      isCurrent: true,
      tags: ['当前使用'],
      languages: ['多语言']
    },
    {
      id: 'voxtral-mini',
      name: 'Online Voxtral Mini',
      provider: 'Mistral AI',
      description: "Mistral AI's fast and accurate transcription model with excellent multilingual support. Delivers high-quality results comparable to GPT-4o mini.",
      accuracy: 4,
      speed: 4,
      tags: ['多语言'],
      languages: ['多语言']
    },
    {
      id: 'elevenlabs-scribe',
      name: 'Online ElevenLabs Scribe',
      provider: 'ElevenLabs Scribe驱动',
      description: '高质量转录和音频处理各种语音识别和多语言支持。',
      accuracy: 4,
      speed: 4,
      tags: ['多语言'],
      languages: ['多语言']
    },
    {
      id: 'whisper-v3-turbo',
      name: 'Online Whisper v3 Turbo',
      provider: 'Groq Whisper v3 Turbo驱动',
      description: '高质量转录各类处理，微妙处理语音具体性短音。',
      accuracy: 3,
      speed: 5,
      tags: ['多语言'],
      languages: ['多语言']
    },
    {
      id: 'whisper-large-v3',
      name: 'Online Real-time Whisper Large v3',
      provider: 'Fireworks AI Whisper v3驱动',
      description: '实时听写具有卓越准确性，持续或式脑即时提供文本。',
      accuracy: 5,
      speed: 3,
      tags: ['实时', '多语言'],
      languages: ['多语言']
    },
    {
      id: 'nova-3-realtime',
      name: 'Online Real-time Nova-3',
      provider: 'Deepgram Nova-3驱动',
      description: '实时听写模型具有卓越准确性，支持多语言。',
      accuracy: 4,
      speed: 5,
      tags: ['多语言'],
      languages: ['多语言']
    },
    {
      id: 'nova-3-medical',
      name: 'Online Nova-3 Medical (Real-time)',
      provider: 'Deepgram Nova-3 Medical驱动',
      description: '专为医疗保健和医学术语优化的专业模型。',
      accuracy: 4,
      speed: 5,
      tags: ['医疗', '实时'],
      languages: ['医疗']
    }
  ];

  const filteredModels = models.filter(model => {
    if (selectedFilter === '全部') return true;
    if (selectedFilter === '在线') return model.id.includes('online');
    if (selectedFilter === '本地') return model.id.includes('local');
    if (selectedFilter === 'API') return true; // 所有都是API模型
    if (selectedFilter === '快速') return model.speed >= 4;
    if (selectedFilter === '准确') return model.accuracy >= 4;
    if (selectedFilter === '多语言') return model.languages.includes('多语言');
    return true;
  });

  return (
    <div className="spokenly-page">
      <div className="spokenly-page-header">
        <h1>听写模型</h1>
        <p>从各种听写模型中选择 - 从云端选择到本地模型，选择最适合您听写需求的准确性、隐私性和速度的平衡点。</p>
      </div>

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

      <div className="spokenly-page-content">
        <p className="model-description">
          需要互联网连接的基于云的模型。这些模型通常具有更好的准确性，但在网络中断时可用。
        </p>

        <div className="models-grid">
          {filteredModels.map((model) => (
            <ModelCard
              key={model.id}
              {...model}
              isSelected={selectedModel === model.id}
              onSelect={() => setSelectedModel(model.id)}
            />
          ))}
        </div>
      </div>
    </div>
  );
};

export default TranscriptionModels;