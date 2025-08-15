import React, { useState } from 'react';
import { useModelsStore } from '../stores/modelsStore';
import { modelCategories } from '../data/models';
import ModelConfigDialog from './ModelConfigDialog';
import LocalModelManager from './LocalModelManager';
import DiagnosticButton from './DiagnosticButton';
import './TranscriptionModelsPage.css';

// 导入主应用的 store
import { useStore } from '../App';

const TranscriptionModelsPage: React.FC = () => {
  const {
    models,
    selectedModelId,
    selectedCategory,
    searchQuery,
    downloadTasks,
    setSelectedModel: setModelStoreModel,
    setSelectedCategory,
    setSearchQuery,
    startDownload,
    getFilteredModels
  } = useModelsStore();
  
  // 获取主应用的 store
  const { setSelectedModel: setMainAppModel } = useStore();

  const [showConfigDialog, setShowConfigDialog] = useState(false);
  const [configModelId, setConfigModelId] = useState<string | null>(null);
  const [showLocalManager, setShowLocalManager] = useState(false);

  const filteredModels = getFilteredModels();

  // 计算每个分类的数量
  const getCategoryCount = (categoryId: string) => {
    if (categoryId === 'all') return models.length;
    return models.filter(m => m.category.includes(categoryId as any)).length;
  };

  const handleModelClick = (modelId: string) => {
    const model = models.find(m => m.id === modelId);
    if (!model) return;

    if (model.type === 'local' && !model.installed) {
      // 本地模型未安装，开始下载
      startDownload(modelId);
    } else if (model.requiresApiKey || model.configurable) {
      // 需要配置API密钥
      setConfigModelId(modelId);
      setShowConfigDialog(true);
    } else {
      // 直接选择模型 - 同时更新两个 store
      setModelStoreModel(modelId);
      setMainAppModel(modelId);
      console.log('🔍 模型选择页面: 已选择模型', modelId);
    }
  };

  const renderModelCard = (model: any) => {
    const isSelected = selectedModelId === model.id;
    const downloadTask = downloadTasks.find(t => t.modelId === model.id);
    const isDownloading = downloadTask && downloadTask.status === 'downloading';

    return (
      <div 
        key={model.id}
        className={`model-card ${isSelected ? 'selected' : ''} ${model.recommended ? 'recommended' : ''}`}
        onClick={() => handleModelClick(model.id)}
      >
        <div className="model-layout">
          {/* 左侧图标 */}
          <div className="model-icon-section">
            <div className="model-icon">{model.icon}</div>
          </div>

          {/* 中间内容 */}
          <div className="model-content">
            <div className="model-header">
              <div className="model-title">
                <h3>{model.name}</h3>
                <div className="model-badges">
                  {model.recommended && <span className="badge recommended">最准确</span>}
                  {model.features.includes('最快') && <span className="badge fastest">最快</span>}
                  {model.realtime && <span className="badge realtime">实时</span>}
                  {model.type === 'local' && model.installed && <span className="badge installed">已安装</span>}
                </div>
              </div>
              <p className="model-provider">
                由{model.provider}驱动 - {model.description}
              </p>
            </div>

            {/* 性能评分 */}
            <div className="model-stats">
              <div className="stat-row">
                <div className="stat">
                  <span className="stat-label">准确度</span>
                  <div className="stat-dots">
                    {[...Array(5)].map((_, i) => (
                      <div key={i} className={`dot ${i < model.accuracy ? 'active' : ''}`}></div>
                    ))}
                  </div>
                </div>
                <div className="stat">
                  <span className="stat-label">速度</span>
                  <div className="stat-dots">
                    {[...Array(5)].map((_, i) => (
                      <div key={i} className={`dot ${i < model.speed ? 'active' : ''}`}></div>
                    ))}
                  </div>
                </div>
              </div>
              <div className="languages">
                <span className="languages-text">{model.languages.join(', ')}</span>
              </div>
            </div>
          </div>

          {/* 右侧操作区 */}
          <div className="model-actions">
            {isSelected ? (
              <div className="selected-indicator">
                <div className="checkmark">✓</div>
                <span className="selected-text">当前使用</span>
              </div>
            ) : (
              <div className="action-buttons">
                {model.type === 'local' && !model.installed ? (
                  isDownloading ? (
                    <div className="download-progress">
                      <div className="progress-bar">
                        <div 
                          className="progress-fill" 
                          style={{ width: `${downloadTask?.progress || 0}%` }}
                        ></div>
                      </div>
                      <div className="progress-info">
                        <span>{Math.round(downloadTask?.progress || 0)}%</span>
                      </div>
                    </div>
                  ) : (
                    <button className="download-btn">
                      下载 {model.modelSize}
                    </button>
                  )
                ) : model.requiresApiKey ? (
                  <div className="api-required">
                    <span className="api-icon">🔑</span>
                    <button className="config-btn">配置</button>
                  </div>
                ) : (
                  <button className="use-model-btn">使用此模型</button>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="transcription-models-page">
      <div className="page-header">
        <div className="header-content">
          <div>
            <h1>听写模型</h1>
            <p>从各种听写模型中选择 - 从云端选项到离线工作的本地模型。选择最适合您听写需求的准确性、隐私性和速度的平衡点。</p>
          </div>
          <div className="header-actions">
            <DiagnosticButton 
              category="model" 
              size="small"
              style="button"
            />
            <button 
              className="manage-local-btn"
              onClick={() => setShowLocalManager(true)}
            >
              🖥️ 管理本地模型
            </button>
          </div>
        </div>
      </div>

      {/* 分类标签 */}
      <div className="model-tabs">
        {modelCategories.map(category => (
          <button 
            key={category.id}
            className={`tab ${selectedCategory === category.id ? 'active' : ''}`}
            onClick={() => setSelectedCategory(category.id)}
          >
            {category.label}
            {getCategoryCount(category.id) > 0 && (
              <span className="tab-count">{getCategoryCount(category.id)}</span>
            )}
          </button>
        ))}
      </div>

      {/* 模型列表 */}
      <div className="models-list">
        {filteredModels.map(model => renderModelCard(model))}
      </div>

      {/* 配置对话框 */}
      {showConfigDialog && configModelId && (
        <ModelConfigDialog
          modelId={configModelId}
          isVisible={showConfigDialog}
          onClose={() => {
            setShowConfigDialog(false);
            setConfigModelId(null);
          }}
        />
      )}

      {/* 本地模型管理器 */}
      {showLocalManager && (
        <LocalModelManager
          isVisible={showLocalManager}
          onClose={() => setShowLocalManager(false)}
        />
      )}

    </div>
  );
};

export default TranscriptionModelsPage;