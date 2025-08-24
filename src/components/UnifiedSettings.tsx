import React, { useState } from 'react';
import VoiceShortcutSettings from './VoiceShortcutSettings';
import HistorySettings from './HistorySettings';
import TextInjectionSettings from './TextInjectionSettings';
import ModelConfig from './ModelConfig';
import './UnifiedSettings.css';
import AboutApp from './AboutApp';

interface UnifiedSettingsProps {
  isVisible: boolean;
  onClose: () => void;
}

type SettingsTab = 'shortcuts' | 'history' | 'injection' | 'models' | 'appearance' | 'about';

const UnifiedSettings: React.FC<UnifiedSettingsProps> = ({ isVisible, onClose }) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>('shortcuts');
  const [settings, setSettings] = useState({
    // 快捷键设置
    shortcuts: {
      enabled: true,
      keyBinding: 'fn+Space',
      triggerMode: 'hold',
      behavior: 'floating'
    },
    // 历史记录设置
    history: {
      autoDelete: false,
      deleteAfterDays: 30,
      maxStorageSize: 1000,
      groupByDate: true,
      showSummaries: true,
      exportFormat: 'txt' as const,
      searchHistoryEnabled: true,
      showConfidenceScores: false,
      showDurations: true,
      enableAutoBackup: false,
      backupInterval: 7
    },
    // 文本注入设置
    injection: {
      method: 'clipboard',
      delay: 100,
      targetApps: [],
      enabled: true
    },
    // 模型配置
    models: {
      provider: 'openai',
      apiKey: '',
      model: 'whisper-1'
    },
    // 外观设置
    appearance: {
      theme: 'system',
      floatingStyle: 'modern',
      opacity: 0.95,
      colorScheme: 'default'
    }
  });

  const tabs = [
    { id: 'shortcuts' as const, name: '快捷键设置', icon: '⌨️' },
    { id: 'history' as const, name: '历史记录', icon: '📝' },
    { id: 'injection' as const, name: '文本注入', icon: '📋' },
    { id: 'models' as const, name: '模型配置', icon: '🤖' },
    { id: 'appearance' as const, name: '外观设置', icon: '🎨' },
    { id: 'about' as const, name: '关于', icon: 'ℹ️' }
  ];

  const updateSettings = (category: string, newSettings: any) => {
    setSettings(prev => ({
      ...prev,
      [category]: { ...prev[category as keyof typeof prev], ...newSettings }
    }));
  };

  if (!isVisible) return null;

  return (
    <div className="unified-settings-overlay" onClick={onClose}>
      <div className="unified-settings-container" onClick={(e) => e.stopPropagation()}>
        {/* 侧边栏导航 */}
        <div className="settings-sidebar">
          <div className="sidebar-header">
            <h2>设置</h2>
            <button className="close-btn" onClick={onClose}>×</button>
          </div>
          <nav className="sidebar-nav">
            {tabs.map(tab => (
              <button
                key={tab.id}
                className={`nav-item ${activeTab === tab.id ? 'active' : ''}`}
                onClick={() => setActiveTab(tab.id)}
              >
                <span className="nav-icon">{tab.icon}</span>
                <span className="nav-text">{tab.name}</span>
              </button>
            ))}
          </nav>
        </div>

        {/* 主内容区域 */}
        <div className="settings-content">
          {activeTab === 'shortcuts' && (
            <div className="settings-panel">
              <h3>快捷键设置</h3>
              <p className="panel-description">配置语音输入的全局快捷键</p>
              {/* 这里可以嵌入现有的VoiceShortcutSettings组件内容 */}
              <div className="setting-group">
                <div className="setting-item">
                  <label>
                    <span>启用快捷键</span>
                    <input 
                      type="checkbox" 
                      checked={settings.shortcuts.enabled}
                      onChange={(e) => updateSettings('shortcuts', { enabled: e.target.checked })}
                    />
                  </label>
                </div>
                <div className="setting-item">
                  <label>
                    <span>快捷键组合</span>
                    <input 
                      type="text" 
                      value={settings.shortcuts.keyBinding}
                      onChange={(e) => updateSettings('shortcuts', { keyBinding: e.target.value })}
                      placeholder="例如: fn+Space"
                    />
                  </label>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'history' && (
            <HistorySettings
              isVisible={true}
              onClose={() => {}}
              settings={settings.history}
              onUpdateSettings={(newSettings) => updateSettings('history', newSettings)}
            />
          )}

          {activeTab === 'injection' && (
            <div className="settings-panel">
              <h3>文本注入设置</h3>
              <p className="panel-description">配置转录文本的注入方式</p>
              <div className="setting-group">
                <div className="setting-item">
                  <label>
                    <span>注入方法</span>
                    <select 
                      value={settings.injection.method}
                      onChange={(e) => updateSettings('injection', { method: e.target.value })}
                    >
                      <option value="clipboard">剪贴板</option>
                      <option value="keyboard">键盘模拟</option>
                    </select>
                  </label>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'models' && (
            <div className="settings-panel">
              <h3>模型配置</h3>
              <p className="panel-description">配置语音识别模型和API</p>
              <div className="setting-group">
                <div className="setting-item">
                  <label>
                    <span>服务提供商</span>
                    <select 
                      value={settings.models.provider}
                      onChange={(e) => updateSettings('models', { provider: e.target.value })}
                    >
                      <option value="openai">OpenAI</option>
                      <option value="local">本地模型</option>
                    </select>
                  </label>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'appearance' && (
            <div className="settings-panel">
              <h3>外观设置</h3>
              <p className="panel-description">自定义悬浮窗口的外观和样式</p>
              <div className="setting-group">
                <div className="setting-item">
                  <label>
                    <span>主题</span>
                    <select 
                      value={settings.appearance.theme}
                      onChange={(e) => updateSettings('appearance', { theme: e.target.value })}
                    >
                      <option value="system">跟随系统</option>
                      <option value="light">浅色</option>
                      <option value="dark">深色</option>
                    </select>
                  </label>
                </div>
                <div className="setting-item">
                  <label>
                    <span>悬浮窗样式</span>
                    <select 
                      value={settings.appearance.floatingStyle}
                      onChange={(e) => updateSettings('appearance', { floatingStyle: e.target.value })}
                    >
                      <option value="modern">现代风格</option>
                      <option value="classic">经典风格</option>
                      <option value="minimal">简约风格</option>
                    </select>
                  </label>
                </div>
                <div className="setting-item">
                  <label>
                    <span>透明度</span>
                    <input 
                      type="range" 
                      min="0.7" 
                      max="1" 
                      step="0.05"
                      value={settings.appearance.opacity}
                      onChange={(e) => updateSettings('appearance', { opacity: parseFloat(e.target.value) })}
                    />
                    <span className="range-value">{Math.round(settings.appearance.opacity * 100)}%</span>
                  </label>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'about' && (
            <div className="settings-panel">
              <h3>关于</h3>
              <AboutApp />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default UnifiedSettings;