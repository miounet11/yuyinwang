import React, { useState } from 'react';
import ModelConfigDialog from './ModelConfigDialog';
import LocalModelManager from './LocalModelManager';
import { useModelsStore } from '../stores/modelsStore';

/**
 * Example component showing how to use the ModelConfigDialog and LocalModelManager components
 * This demonstrates the integration with the useModelsStore and proper state management
 */
const ComponentUsageExample: React.FC = () => {
  const { models } = useModelsStore();
  
  // Dialog states
  const [showModelConfig, setShowModelConfig] = useState(false);
  const [showLocalModelManager, setShowLocalModelManager] = useState(false);
  const [selectedModel, setSelectedModel] = useState<string | null>(null);

  // Get models by type for demonstration
  const onlineModels = models.filter(model => model.type === 'online' && model.requiresApiKey);
  const localModels = models.filter(model => model.type === 'local');

  const handleConfigureModel = (modelId: string) => {
    setSelectedModel(modelId);
    setShowModelConfig(true);
  };

  const handleCloseConfig = () => {
    setShowModelConfig(false);
    setSelectedModel(null);
  };

  const handleOpenLocalManager = () => {
    setShowLocalModelManager(true);
  };

  const handleCloseLocalManager = () => {
    setShowLocalModelManager(false);
  };

  const selectedModelData = models.find(m => m.id === selectedModel);

  return (
    <div className="component-usage-example">
      <div className="demo-section">
        <h2>Model Configuration Example</h2>
        <p>Configure API keys and settings for online models that require authentication:</p>
        
        <div className="model-grid">
          {onlineModels.map(model => (
            <div key={model.id} className="model-card">
              <div className="model-header">
                <span className="model-icon">{model.icon}</span>
                <div>
                  <h3>{model.name}</h3>
                  <p>{model.provider}</p>
                </div>
              </div>
              <p className="model-description">{model.description}</p>
              <button 
                className="config-btn"
                onClick={() => handleConfigureModel(model.id)}
              >
                Configure API
              </button>
            </div>
          ))}
        </div>
      </div>

      <div className="demo-section">
        <h2>Local Model Manager Example</h2>
        <p>Manage local model downloads, storage, and installation:</p>
        
        <div className="local-models-summary">
          <div className="summary-stats">
            <div className="stat">
              <span className="stat-number">{localModels.length}</span>
              <span className="stat-label">Available Models</span>
            </div>
            <div className="stat">
              <span className="stat-number">{localModels.filter(m => m.installed).length}</span>
              <span className="stat-label">Installed</span>
            </div>
            <div className="stat">
              <span className="stat-number">{localModels.filter(m => m.downloading).length}</span>
              <span className="stat-label">Downloading</span>
            </div>
          </div>
          
          <button 
            className="manage-btn primary"
            onClick={handleOpenLocalManager}
          >
            Manage Local Models
          </button>
        </div>

        <div className="local-models-preview">
          {localModels.slice(0, 3).map(model => (
            <div key={model.id} className="model-preview">
              <span className="model-icon">{model.icon}</span>
              <div className="model-info">
                <h4>{model.name}</h4>
                <p>{model.modelSize}</p>
              </div>
              <span className={`status-indicator ${model.installed ? 'installed' : 'available'}`}>
                {model.installed ? '✓' : '○'}
              </span>
            </div>
          ))}
          {localModels.length > 3 && (
            <div className="more-models">
              +{localModels.length - 3} more models
            </div>
          )}
        </div>
      </div>

      {/* Model Configuration Dialog */}
      <ModelConfigDialog
        isVisible={showModelConfig}
        modelId={selectedModelData?.id || ''}
        onClose={handleCloseConfig}
      />

      {/* Local Model Manager */}
      <LocalModelManager
        isVisible={showLocalModelManager}
        onClose={handleCloseLocalManager}
      />
    </div>
  );
};

export default ComponentUsageExample;