import React, { useState } from 'react';
import { TranscriptionModel, ApiConfig } from '../types/models';
import './ModelConfig.css';

interface ModelConfigProps {
  isVisible: boolean;
  model: TranscriptionModel;
  onClose: () => void;
  onSave: (config: ApiConfig) => void;
}

const ModelConfig: React.FC<ModelConfigProps> = ({ isVisible, model, onClose, onSave }) => {
  const [config, setConfig] = useState<ApiConfig>(model.apiConfig || { apiKey: '' });
  const [isLoading, setIsLoading] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  if (!isVisible) return null;

  const handleSave = () => {
    onSave(config);
    onClose();
  };

  const handleTest = async () => {
    setIsLoading(true);
    setTestResult(null);
    
    try {
      // 这里应该调用实际的API测试逻辑
      await new Promise(resolve => setTimeout(resolve, 1000)); // 模拟API调用
      
      if (config.apiKey && config.apiKey.length > 10) {
        setTestResult({ success: true, message: 'API连接测试成功' });
      } else {
        setTestResult({ success: false, message: 'API密钥无效' });
      }
    } catch (error) {
      setTestResult({ success: false, message: '连接失败，请检查网络和配置' });
    } finally {
      setIsLoading(false);
    }
  };

  const renderConfigFields = () => {
    switch (model.provider) {
      case 'openai':
        return (
          <>
            <div className="config-field">
              <label>API密钥</label>
              <input
                type="password"
                value={config.apiKey}
                onChange={(e) => setConfig({ ...config, apiKey: e.target.value })}
                placeholder="sk-..."
                className="config-input"
              />
              <p className="field-hint">从 OpenAI 控制台获取您的API密钥</p>
            </div>
            
            <div className="config-field">
              <label>模型版本</label>
              <select
                value={config.model || 'whisper-1'}
                onChange={(e) => setConfig({ ...config, model: e.target.value })}
                className="config-select"
              >
                <option value="whisper-1">whisper-1</option>
              </select>
            </div>

            <div className="config-field">
              <label>语言设置</label>
              <select
                value={config.language || 'auto'}
                onChange={(e) => setConfig({ ...config, language: e.target.value })}
                className="config-select"
              >
                <option value="auto">自动检测</option>
                <option value="zh">中文</option>
                <option value="en">English</option>
                <option value="ja">日本語</option>
                <option value="ko">한국어</option>
              </select>
            </div>
          </>
        );

      case 'deepgram':
        return (
          <>
            <div className="config-field">
              <label>API密钥</label>
              <input
                type="password"
                value={config.apiKey}
                onChange={(e) => setConfig({ ...config, apiKey: e.target.value })}
                placeholder="输入Deepgram API密钥"
                className="config-input"
              />
            </div>

            <div className="config-field">
              <label>模型版本</label>
              <select
                value={config.model || 'nova-2'}
                onChange={(e) => setConfig({ ...config, model: e.target.value })}
                className="config-select"
              >
                <option value="nova-2">Nova-2</option>
                <option value="nova-3">Nova-3</option>
                <option value="enhanced">Enhanced</option>
              </select>
            </div>

            <div className="config-field">
              <label>高级设置</label>
              <div className="checkbox-group">
                <label>
                  <input
                    type="checkbox"
                    checked={config.customSettings?.punctuate || false}
                    onChange={(e) => setConfig({
                      ...config,
                      customSettings: { ...config.customSettings, punctuate: e.target.checked }
                    })}
                  />
                  启用标点符号
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={config.customSettings?.diarize || false}
                    onChange={(e) => setConfig({
                      ...config,
                      customSettings: { ...config.customSettings, diarize: e.target.checked }
                    })}
                  />
                  说话人分离
                </label>
              </div>
            </div>
          </>
        );

      default:
        return (
          <div className="config-field">
            <label>API密钥</label>
            <input
              type="password"
              value={config.apiKey}
              onChange={(e) => setConfig({ ...config, apiKey: e.target.value })}
              placeholder="输入API密钥"
              className="config-input"
            />
          </div>
        );
    }
  };

  return (
    <div className="model-config-overlay">
      <div className="model-config-dialog">
        <div className="config-header">
          <div className="config-title">
            <div className="model-icon">{model.icon}</div>
            <div>
              <h2>{model.name}</h2>
              <p className="model-provider">配置 {model.provider} 设置</p>
            </div>
          </div>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="config-content">
          <div className="model-info">
            <h3>模型信息</h3>
            <div className="info-grid">
              <div className="info-item">
                <span className="label">准确度</span>
                <div className="score-dots">
                  {[...Array(5)].map((_, i) => (
                    <div key={i} className={`dot ${i < model.performance.accuracy ? 'active' : ''}`}></div>
                  ))}
                </div>
              </div>
              <div className="info-item">
                <span className="label">速度</span>
                <div className="score-dots">
                  {[...Array(5)].map((_, i) => (
                    <div key={i} className={`dot ${i < model.performance.speed ? 'active' : ''}`}></div>
                  ))}
                </div>
              </div>
              <div className="info-item">
                <span className="label">实时支持</span>
                <span className={`status ${model.performance.realtime ? 'yes' : 'no'}`}>
                  {model.performance.realtime ? '是' : '否'}
                </span>
              </div>
            </div>
          </div>

          <div className="config-form">
            <h3>配置设置</h3>
            {renderConfigFields()}
          </div>

          {testResult && (
            <div className={`test-result ${testResult.success ? 'success' : 'error'}`}>
              <div className="result-icon">
                {testResult.success ? '✅' : '❌'}
              </div>
              <span>{testResult.message}</span>
            </div>
          )}
        </div>

        <div className="config-actions">
          <button
            className="test-btn"
            onClick={handleTest}
            disabled={isLoading || !config.apiKey}
          >
            {isLoading ? '测试中...' : '测试连接'}
          </button>
          <div className="action-buttons">
            <button className="cancel-btn" onClick={onClose}>取消</button>
            <button 
              className="save-btn" 
              onClick={handleSave}
              disabled={!config.apiKey}
            >
              保存配置
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ModelConfig;