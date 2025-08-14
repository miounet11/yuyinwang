import React, { useState, useEffect } from 'react';
import { useModelsStore } from '../stores/modelsStore';
import { getModelById, getProviderById } from '../data/models';
import './ModelConfigDialog.css';

interface ModelConfigDialogProps {
  modelId: string;
  isVisible: boolean;
  onClose: () => void;
}

const ModelConfigDialog: React.FC<ModelConfigDialogProps> = ({
  modelId,
  isVisible,
  onClose
}) => {
  const { modelConfigs, saveModelConfig, setSelectedModel } = useModelsStore();
  const [config, setConfig] = useState<any>({});
  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<{
    success: boolean;
    message: string;
  } | null>(null);

  const model = getModelById(modelId);
  const provider = model?.apiKeyField 
    ? getProviderById(model.apiKeyField.replace('_api_key', ''))
    : null;

  useEffect(() => {
    if (modelConfigs[modelId]) {
      setConfig(modelConfigs[modelId]);
    } else {
      // 初始化默认配置
      const defaultConfig: any = { modelId };
      provider?.configFields.forEach(field => {
        defaultConfig[field.name] = field.defaultValue || '';
      });
      setConfig(defaultConfig);
    }
  }, [modelId, modelConfigs, provider]);

  const handleFieldChange = (fieldName: string, value: any) => {
    setConfig((prev: any) => ({
      ...prev,
      [fieldName]: value
    }));
  };

  const handleTestConnection = async () => {
    setIsTesting(true);
    setTestResult(null);

    // 模拟API测试
    setTimeout(() => {
      const hasApiKey = config.api_key && config.api_key.length > 0;
      setTestResult({
        success: hasApiKey,
        message: hasApiKey 
          ? '✅ 连接成功！API密钥有效。'
          : '❌ 连接失败。请检查您的API密钥。'
      });
      setIsTesting(false);
    }, 1500);
  };

  const handleSave = () => {
    saveModelConfig(modelId, config);
    setSelectedModel(modelId);
    onClose();
  };

  if (!isVisible || !model || !provider) return null;

  return (
    <div className="model-config-overlay" onClick={onClose}>
      <div className="model-config-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="config-header">
          <div className="config-title">
            <div className="model-icon">{model.icon}</div>
            <div>
              <h2>配置 {model.name}</h2>
              <p>{provider.description}</p>
            </div>
          </div>
          <button className="close-btn" onClick={onClose}>CLOSE</button>
        </div>

        <div className="config-body">
          {provider.configFields.map(field => (
            <div key={field.name} className="config-field">
              <label>{field.label}</label>
              
              {field.type === 'password' && (
                <input
                  type="password"
                  value={config[field.name] || ''}
                  onChange={(e) => handleFieldChange(field.name, e.target.value)}
                  placeholder={field.placeholder}
                  className="config-input"
                />
              )}
              
              {field.type === 'text' && (
                <input
                  type="text"
                  value={config[field.name] || ''}
                  onChange={(e) => handleFieldChange(field.name, e.target.value)}
                  placeholder={field.placeholder}
                  className="config-input"
                />
              )}
              
              {field.type === 'select' && (
                <select
                  value={config[field.name] || ''}
                  onChange={(e) => handleFieldChange(field.name, e.target.value)}
                  className="config-select"
                >
                  {field.options?.map(option => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              )}
              
              {field.type === 'toggle' && (
                <label className="config-toggle">
                  <input
                    type="checkbox"
                    checked={config[field.name] || false}
                    onChange={(e) => handleFieldChange(field.name, e.target.checked)}
                  />
                  <span className="toggle-slider"></span>
                </label>
              )}
              
              {field.type === 'slider' && (
                <div className="config-slider">
                  <input
                    type="range"
                    min={field.min}
                    max={field.max}
                    step={field.step}
                    value={config[field.name] || field.defaultValue}
                    onChange={(e) => handleFieldChange(field.name, parseFloat(e.target.value))}
                  />
                  <span className="slider-value">{config[field.name] || field.defaultValue}</span>
                </div>
              )}
              
              {field.description && (
                <p className="field-description">{field.description}</p>
              )}
            </div>
          ))}

          {testResult && (
            <div className={`test-result ${testResult.success ? 'success' : 'error'}`}>
              {testResult.message}
            </div>
          )}
        </div>

        <div className="config-footer">
          <div className="footer-left">
            <a href="#" className="help-link">🔑 获取API密钥</a>
            <a href="#" className="help-link">📖 查看文档</a>
          </div>
          <div className="footer-right">
            <button 
              className="test-btn"
              onClick={handleTestConnection}
              disabled={isTesting}
            >
              {isTesting ? '测试中...' : '测试连接'}
            </button>
            <button className="cancel-btn" onClick={onClose}>取消</button>
            <button className="save-btn" onClick={handleSave}>保存配置</button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ModelConfigDialog;