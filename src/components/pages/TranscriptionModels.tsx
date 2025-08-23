/**
 * 听写模型页面
 * 完美复刻 Spokenly 第二张截图的设计
 * 注意：已移除测试功能，适用于正式版本
 */

import React, { useState } from 'react';
import './TranscriptionModels.css';
import { SpokenlyCard, SpokenlyTag, SpokenlyButton } from '../ui';
import { transcriptionModels, getModelsByCategory } from '../../data/models';
import { TranscriptionModel } from '../../types/models';

interface ModelCardProps {
  model: TranscriptionModel;
  isSelected?: boolean;
  isCurrent?: boolean;
  onSelect?: () => void;
}

const ModelCard: React.FC<ModelCardProps> = ({
  model,
  isSelected = false,
  isCurrent = false,
  onSelect
}) => {
  const { id, name, provider, description, accuracy, speed, recommended, features = [], languages = [], type, modelSize, installed } = model;
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
            {renderStatusDots(5, accuracy)}
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
            <span>{languages.length > 0 ? (languages.length === 1 ? languages[0] : `${languages.length}种语言`) : '多语言'}</span>
          </div>
        </div>

        <div className="stat-item">
          <div className="model-tags">
            {type === 'local' && (
              <SpokenlyTag variant="success" size="sm">
                {installed ? '已安装' : '可下载'}
              </SpokenlyTag>
            )}
            {type === 'online' && (
              <SpokenlyTag variant="info" size="sm">
                在线API
              </SpokenlyTag>
            )}
            {recommended && (
              <SpokenlyTag variant="warning" size="sm">
                推荐
              </SpokenlyTag>
            )}
            {modelSize && (
              <SpokenlyTag variant="secondary" size="sm">
                {modelSize}
              </SpokenlyTag>
            )}
            {isCurrent && (
              <SpokenlyTag variant="primary" size="sm">
                当前使用
              </SpokenlyTag>
            )}
          </div>
        </div>
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

  // 使用正确的数据源和过滤逻辑
  const filteredModels = transcriptionModels.filter(model => {
    if (selectedFilter === '全部') return true;
    if (selectedFilter === '在线') return model.type === 'online';
    if (selectedFilter === '本地') return model.type === 'local';
    if (selectedFilter === 'API') return model.type === 'online'; // API模型都是在线模型
    if (selectedFilter === '快速') return model.speed >= 4;
    if (selectedFilter === '准确') return model.accuracy >= 4;
    if (selectedFilter === '标点符号') return model.category.includes('punctuation');
    if (selectedFilter === '字幕') return model.category.includes('subtitle');
    return true;
  });

  // 获取当前分类的描述信息
  const getFilterDescription = () => {
    switch (selectedFilter) {
      case '在线':
        return '需要互联网连接的基于云端API的模型。这些模型通常具有更好的准确性和最新的功能，但需要稳定的网络连接。';
      case '本地':
        return '可以下载到本地运行的模型。这些模型在离线状态下工作，保护隐私，但需要本地计算资源和存储空间。';
      case 'API':
        return '基于云端API的在线转录服务，提供高准确度和快速响应，需要相应的API密钥配置。';
      case '快速':
        return '针对速度优化的模型，适合需要快速转录的场景。';
      case '准确':
        return '高准确度的模型，适合需要精确转录的专业场景。';
      case '标点符号':
        return '支持自动添加标点符号的模型，提供更好的文本格式化。';
      case '字幕':
        return '专为字幕制作优化的模型，支持时间戳和格式化输出。';
      default:
        return '从各种听写模型中选择 - 从云端API到本地模型，选择最适合您听写需求的准确性、隐私性和速度的平衡点。';
    }
  };

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
          {getFilterDescription()}
        </p>

        <div className="models-grid">
          {filteredModels.map((model) => (
            <ModelCard
              key={model.id}
              model={model}
              isSelected={selectedModel === model.id}
              isCurrent={selectedModel === model.id}
              onSelect={() => setSelectedModel(model.id)}
            />
          ))}
        </div>
      </div>
    </div>
  );
};

export default TranscriptionModels;