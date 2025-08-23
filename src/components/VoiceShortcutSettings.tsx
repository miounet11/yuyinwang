import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './VoiceShortcutSettings.css';

interface VoiceShortcutConfig {
  enabled: boolean;
  shortcut: string;
  auto_insert: boolean;
  use_floating_window: boolean;
  preferred_model: string;
  trigger_mode?: 'press' | 'hold';  // 新增：触发模式
  hold_duration?: number;  // 新增：长按持续时间（毫秒）
  realtime_injection?: boolean; // 新增：是否实时注入
  hold_release_delay_ms?: number; // 新增：松手延迟结束（毫秒）
}

interface VoiceShortcutSettingsProps {
  isVisible: boolean;
  onClose: () => void;
}

const VoiceShortcutSettings: React.FC<VoiceShortcutSettingsProps> = ({ isVisible, onClose }) => {
  const [config, setConfig] = useState<VoiceShortcutConfig>({
    enabled: true,
    shortcut: 'CmdOrCtrl+Shift+Space',
    auto_insert: true,
    use_floating_window: true,
    preferred_model: 'luyingwang-online',
    trigger_mode: 'press',
    hold_duration: 300,
    realtime_injection: true,
    hold_release_delay_ms: 150
  });
  
  const [isRecording, setIsRecording] = useState(false);
  const [pressedKeys, setPressedKeys] = useState<string[]>([]);
  const [saveStatus, setSaveStatus] = useState('');

  // 预设快捷键
  const presetShortcuts = [
    { label: '⌘ + Shift + Space', value: 'CmdOrCtrl+Shift+Space' },
    { label: '⌘ + Shift + V', value: 'CmdOrCtrl+Shift+V' },
    { label: 'Option + Space', value: 'Alt+Space' },
    { label: 'F1', value: 'F1' },
    { label: 'F2', value: 'F2' },
  ];

  // 可用模型列表
  const availableModels = [
    { label: '录音王在线 (推荐)', value: 'luyingwang-online' },
    { label: 'Whisper Tiny (本地)', value: 'whisper-tiny' },
    { label: 'Whisper Base (本地)', value: 'whisper-base' },
    { label: 'GPT-4o Mini', value: 'gpt-4o-mini' },
  ];

  useEffect(() => {
    if (isVisible) {
      loadConfig();
    }
  }, [isVisible]);

  const loadConfig = async () => {
    try {
      // 从后端加载配置
      const savedConfig = await invoke<VoiceShortcutConfig>('load_voice_shortcut_config');
      if (savedConfig) {
        setConfig(savedConfig);
      }
    } catch (error) {
      console.error('加载快捷键配置失败:', error);
    }
  };

  const saveConfig = async () => {
    try {
      setSaveStatus('保存中...');
      await invoke('configure_voice_shortcuts', { config });
      setSaveStatus('已保存');
      setTimeout(() => setSaveStatus(''), 2000);
    } catch (error) {
      console.error('保存配置失败:', error);
      setSaveStatus('保存失败');
    }
  };

  const handleShortcutRecord = () => {
    setIsRecording(true);
    setPressedKeys([]);
    
    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      
      const keys: string[] = [];
      if (e.metaKey || e.ctrlKey) keys.push('CmdOrCtrl');
      if (e.altKey) keys.push('Alt');
      if (e.shiftKey) keys.push('Shift');
      
      // 添加主键
      if (e.key && !['Control', 'Meta', 'Alt', 'Shift'].includes(e.key)) {
        const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
        keys.push(key);
      }
      
      if (keys.length > 0) {
        setPressedKeys(keys);
        const shortcut = keys.join('+');
        setConfig(prev => ({ ...prev, shortcut }));
      }
    };
    
    const handleKeyUp = () => {
      setIsRecording(false);
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('keyup', handleKeyUp);
    };
    
    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('keyup', handleKeyUp);
  };

  const testShortcut = async () => {
    try {
      // 触发一次快速语音输入测试
      await invoke('trigger_voice_input_test');
    } catch (error) {
      console.error('测试快捷键失败:', error);
    }
  };

  if (!isVisible) return null;

  return (
    <div className="voice-shortcut-settings-overlay">
      <div className="voice-shortcut-settings">
        <div className="settings-header">
          <h2>🎤 语音输入快捷键设置</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="settings-content">
          {/* 启用开关 */}
          <div className="setting-group">
            <label className="setting-label">
              <input
                type="checkbox"
                checked={config.enabled}
                onChange={(e) => setConfig(prev => ({ ...prev, enabled: e.target.checked }))}
              />
              <span>启用快速语音输入</span>
            </label>
            <p className="setting-desc">在任何应用中使用快捷键快速输入语音转文字</p>
          </div>

          {/* 快捷键设置 */}
          <div className="setting-group">
            <h3>快捷键</h3>
            <div className="shortcut-input-group">
              <input
                type="text"
                className="shortcut-input"
                value={isRecording ? '请按下快捷键组合...' : config.shortcut}
                readOnly
                onClick={handleShortcutRecord}
              />
              <button 
                className="record-btn"
                onClick={handleShortcutRecord}
              >
                {isRecording ? '录制中...' : '录制'}
              </button>
            </div>
            
            {/* 预设快捷键 */}
            <div className="preset-shortcuts">
              {presetShortcuts.map(preset => (
                <button
                  key={preset.value}
                  className={`preset-btn ${config.shortcut === preset.value ? 'active' : ''}`}
                  onClick={() => setConfig(prev => ({ ...prev, shortcut: preset.value }))}
                >
                  {preset.label}
                </button>
              ))}
            </div>
          </div>

          {/* 模型选择 */}
          <div className="setting-group">
            <h3>转录模型</h3>
            <select
              className="model-select"
              value={config.preferred_model}
              onChange={(e) => setConfig(prev => ({ ...prev, preferred_model: e.target.value }))}
            >
              {availableModels.map(model => (
                <option key={model.value} value={model.value}>
                  {model.label}
                </option>
              ))}
            </select>
            <p className="setting-desc">选择用于快速转录的AI模型</p>
          </div>

          {/* 行为设置 */}
          <div className="setting-group">
            <h3>行为设置</h3>
            
            {/* 触发模式选择 */}
            <div className="trigger-mode-section">
              <label className="setting-label">触发模式</label>
              <div className="trigger-mode-options">
                <label className="radio-label">
                  <input
                    type="radio"
                    name="triggerMode"
                    value="press"
                    checked={config.trigger_mode === 'press'}
                    onChange={() => setConfig(prev => ({ ...prev, trigger_mode: 'press' }))}
                  />
                  <span>单击激活</span>
                </label>
                <label className="radio-label">
                  <input
                    type="radio"
                    name="triggerMode"
                    value="hold"
                    checked={config.trigger_mode === 'hold'}
                    onChange={() => setConfig(prev => ({ ...prev, trigger_mode: 'hold' }))}
                  />
                  <span>长按录音</span>
                </label>
              </div>
              <p className="setting-desc">
                {config.trigger_mode === 'press' 
                  ? '按下快捷键开始录音，检测到静音自动停止' 
                  : '按住快捷键录音，松开即停止'}
              </p>
            </div>
            
            {config.trigger_mode === 'hold' && (
              <div className="hold-section">
                <label className="setting-label">最短按住时长（毫秒）</label>
                <input
                  type="number"
                  min={100}
                  max={2000}
                  value={config.hold_duration || 300}
                  onChange={(e) => setConfig(prev => ({ ...prev, hold_duration: Number(e.target.value) }))}
                  className="number-input"
                />
                <label className="setting-label">松手后延迟结束（毫秒）</label>
                <input
                  type="number"
                  min={0}
                  max={1000}
                  value={config.hold_release_delay_ms || 150}
                  onChange={(e) => setConfig(prev => ({ ...prev, hold_release_delay_ms: Number(e.target.value) }))}
                  className="number-input"
                />
              </div>
            )}

            <div className="toggle-row">
              <label className="setting-label">
                <input
                  type="checkbox"
                  checked={!!config.realtime_injection}
                  onChange={(e) => setConfig(prev => ({ ...prev, realtime_injection: e.target.checked }))}
                />
                <span>实时注入文本</span>
              </label>
            </div>
          </div>

          <div className="setting-actions">
            <button className="test-btn" onClick={testShortcut}>
              测试快捷键
            </button>
            <button className="save-btn" onClick={saveConfig}>
              {saveStatus || '保存设置'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default VoiceShortcutSettings;