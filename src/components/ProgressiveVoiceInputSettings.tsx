import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './ProgressiveVoiceInputSettings.css';

interface ProgressiveSettings {
  // 触发设置
  shortcut: string;
  longPressThresholdMs: number;
  enabled: boolean;
  triggerSoundEnabled: boolean;
  autoDetectTargetApp: boolean;
  
  // 注入设置
  enableRealTimeInjection: boolean;
  injectIntervalMs: number;
  minInjectLength: number;
  maxQueueLength: number;
  enableBackspaceCorrection: boolean;
  smartPrefixMerging: boolean;
  
  // 转录设置
  chunkDurationMs: number;
  overlapDurationMs: number;
  minConfidenceThreshold: number;
  silenceTimeoutMs: number;
  maxPartialLength: number;
}

const defaultSettings: ProgressiveSettings = {
  shortcut: 'Option+Space',
  longPressThresholdMs: 800,
  enabled: true,
  triggerSoundEnabled: true,
  autoDetectTargetApp: true,
  
  enableRealTimeInjection: true,
  injectIntervalMs: 150,
  minInjectLength: 1,
  maxQueueLength: 30,
  enableBackspaceCorrection: true,
  smartPrefixMerging: true,
  
  chunkDurationMs: 300,
  overlapDurationMs: 50,
  minConfidenceThreshold: 0.6,
  silenceTimeoutMs: 2500,
  maxPartialLength: 150,
};

const ProgressiveVoiceInputSettings: React.FC = () => {
  const [settings, setSettings] = useState<ProgressiveSettings>(defaultSettings);
  const [hasChanges, setHasChanges] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');
  const [error, setError] = useState<string>('');

  // 加载设置
  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setIsLoading(true);
      const status = await invoke<string>('get_progressive_trigger_status');
      const statusData = JSON.parse(status);
      
      if (statusData.config) {
        setSettings(prev => ({
          ...prev,
          shortcut: statusData.config.shortcut || prev.shortcut,
          longPressThresholdMs: statusData.config.threshold_ms || prev.longPressThresholdMs,
          enabled: statusData.config.enabled ?? prev.enabled,
          triggerSoundEnabled: statusData.config.sound_enabled ?? prev.triggerSoundEnabled,
          autoDetectTargetApp: statusData.config.auto_detect_app ?? prev.autoDetectTargetApp,
          enableRealTimeInjection: statusData.config.real_time_injection ?? prev.enableRealTimeInjection,
        }));
      }
    } catch (err) {
      console.error('加载设置失败:', err);
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const updateSetting = <K extends keyof ProgressiveSettings>(
    key: K,
    value: ProgressiveSettings[K]
  ) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    setHasChanges(true);
    setSaveStatus('idle');
  };

  const saveSettings = async () => {
    try {
      setSaveStatus('saving');
      setError('');
      
      // 保存触发配置
      await invoke('update_progressive_trigger_config', {
        config: {
          shortcut: settings.shortcut,
          long_press_threshold_ms: settings.longPressThresholdMs,
          enabled: settings.enabled,
          enable_real_time_injection: settings.enableRealTimeInjection,
          trigger_sound_enabled: settings.triggerSoundEnabled,
          auto_detect_target_app: settings.autoDetectTargetApp,
        }
      });
      
      setSaveStatus('success');
      setHasChanges(false);
      
      // 3秒后重置状态
      setTimeout(() => {
        setSaveStatus('idle');
      }, 3000);
      
    } catch (err) {
      console.error('保存设置失败:', err);
      setError(String(err));
      setSaveStatus('error');
    }
  };

  const resetToDefaults = () => {
    setSettings(defaultSettings);
    setHasChanges(true);
    setSaveStatus('idle');
  };

  const testConfiguration = async () => {
    try {
      setError('');
      const result = await invoke<string>('test_progressive_trigger', {
        targetBundleId: null,
      });
      alert(`测试成功: ${result}`);
    } catch (err) {
      console.error('测试失败:', err);
      setError(String(err));
    }
  };

  return (
    <div className="progressive-voice-settings">
      <div className="settings-header">
        <h2>渐进式语音输入设置</h2>
        <p>配置如输入法般的实时语音转录和文本注入体验</p>
      </div>

      {isLoading && (
        <div className="loading-indicator">
          <div className="spinner"></div>
          <span>加载设置中...</span>
        </div>
      )}

      <div className="settings-content">
        {/* 触发设置 */}
        <div className="settings-section">
          <h3>快捷键触发</h3>
          <div className="settings-grid">
            <div className="setting-item">
              <label htmlFor="shortcut">快捷键组合</label>
              <input
                id="shortcut"
                type="text"
                value={settings.shortcut}
                onChange={(e) => updateSetting('shortcut', e.target.value)}
                placeholder="例如: Option+Space"
                className="setting-input"
              />
              <small>支持: Cmd, Option, Shift, Ctrl 等修饰键</small>
            </div>

            <div className="setting-item">
              <label htmlFor="threshold">长按阈值 (毫秒)</label>
              <input
                id="threshold"
                type="number"
                min="100"
                max="3000"
                step="50"
                value={settings.longPressThresholdMs}
                onChange={(e) => updateSetting('longPressThresholdMs', Number(e.target.value))}
                className="setting-input"
              />
              <small>长按多久后触发语音输入 (推荐: 800ms)</small>
            </div>

            <div className="setting-item checkbox-item">
              <label>
                <input
                  type="checkbox"
                  checked={settings.enabled}
                  onChange={(e) => updateSetting('enabled', e.target.checked)}
                />
                <span>启用快捷键触发</span>
              </label>
            </div>

            <div className="setting-item checkbox-item">
              <label>
                <input
                  type="checkbox"
                  checked={settings.triggerSoundEnabled}
                  onChange={(e) => updateSetting('triggerSoundEnabled', e.target.checked)}
                />
                <span>触发提示音</span>
              </label>
            </div>

            <div className="setting-item checkbox-item">
              <label>
                <input
                  type="checkbox"
                  checked={settings.autoDetectTargetApp}
                  onChange={(e) => updateSetting('autoDetectTargetApp', e.target.checked)}
                />
                <span>自动检测目标应用</span>
              </label>
            </div>
          </div>
        </div>

        {/* 文本注入设置 */}
        <div className="settings-section">
          <h3>文本注入</h3>
          <div className="settings-grid">
            <div className="setting-item checkbox-item">
              <label>
                <input
                  type="checkbox"
                  checked={settings.enableRealTimeInjection}
                  onChange={(e) => updateSetting('enableRealTimeInjection', e.target.checked)}
                />
                <span>实时文本注入</span>
              </label>
              <small>如输入法般实时显示转录结果</small>
            </div>

            <div className="setting-item">
              <label htmlFor="inject-interval">注入间隔 (毫秒)</label>
              <input
                id="inject-interval"
                type="number"
                min="50"
                max="1000"
                step="25"
                value={settings.injectIntervalMs}
                onChange={(e) => updateSetting('injectIntervalMs', Number(e.target.value))}
                className="setting-input"
              />
              <small>文本注入频率 (推荐: 150ms)</small>
            </div>

            <div className="setting-item">
              <label htmlFor="min-length">最小注入长度</label>
              <input
                id="min-length"
                type="number"
                min="1"
                max="10"
                value={settings.minInjectLength}
                onChange={(e) => updateSetting('minInjectLength', Number(e.target.value))}
                className="setting-input"
              />
              <small>低于此字符数不进行注入</small>
            </div>

            <div className="setting-item">
              <label htmlFor="queue-length">最大队列长度</label>
              <input
                id="queue-length"
                type="number"
                min="10"
                max="100"
                value={settings.maxQueueLength}
                onChange={(e) => updateSetting('maxQueueLength', Number(e.target.value))}
                className="setting-input"
              />
              <small>防止内存过度占用</small>
            </div>

            <div className="setting-item checkbox-item">
              <label>
                <input
                  type="checkbox"
                  checked={settings.enableBackspaceCorrection}
                  onChange={(e) => updateSetting('enableBackspaceCorrection', e.target.checked)}
                />
                <span>退格纠错</span>
              </label>
              <small>自动替换不准确的转录结果</small>
            </div>

            <div className="setting-item checkbox-item">
              <label>
                <input
                  type="checkbox"
                  checked={settings.smartPrefixMerging}
                  onChange={(e) => updateSetting('smartPrefixMerging', e.target.checked)}
                />
                <span>智能前缀合并</span>
              </label>
              <small>避免重复注入相同前缀</small>
            </div>
          </div>
        </div>

        {/* 转录设置 */}
        <div className="settings-section">
          <h3>实时转录</h3>
          <div className="settings-grid">
            <div className="setting-item">
              <label htmlFor="chunk-duration">音频块时长 (毫秒)</label>
              <input
                id="chunk-duration"
                type="number"
                min="100"
                max="1000"
                step="50"
                value={settings.chunkDurationMs}
                onChange={(e) => updateSetting('chunkDurationMs', Number(e.target.value))}
                className="setting-input"
              />
              <small>更短=更快响应，更长=更准确 (推荐: 300ms)</small>
            </div>

            <div className="setting-item">
              <label htmlFor="overlap-duration">重叠时长 (毫秒)</label>
              <input
                id="overlap-duration"
                type="number"
                min="0"
                max="200"
                step="25"
                value={settings.overlapDurationMs}
                onChange={(e) => updateSetting('overlapDurationMs', Number(e.target.value))}
                className="setting-input"
              />
              <small>避免丢失边界词 (推荐: 50ms)</small>
            </div>

            <div className="setting-item">
              <label htmlFor="confidence">最小置信度</label>
              <input
                id="confidence"
                type="number"
                min="0.1"
                max="1.0"
                step="0.05"
                value={settings.minConfidenceThreshold}
                onChange={(e) => updateSetting('minConfidenceThreshold', Number(e.target.value))}
                className="setting-input"
              />
              <small>过滤低质量转录 (推荐: 0.6)</small>
            </div>

            <div className="setting-item">
              <label htmlFor="silence-timeout">静音超时 (毫秒)</label>
              <input
                id="silence-timeout"
                type="number"
                min="1000"
                max="10000"
                step="250"
                value={settings.silenceTimeoutMs}
                onChange={(e) => updateSetting('silenceTimeoutMs', Number(e.target.value))}
                className="setting-input"
              />
              <small>检测到静音后自动停止 (推荐: 2500ms)</small>
            </div>
          </div>
        </div>
      </div>

      {/* 操作按钮 */}
      <div className="settings-actions">
        <div className="action-buttons">
          <button
            onClick={testConfiguration}
            className="btn btn-secondary"
            disabled={isLoading}
          >
            🧪 测试配置
          </button>
          
          <button
            onClick={resetToDefaults}
            className="btn btn-tertiary"
            disabled={isLoading}
          >
            🔄 重置默认
          </button>
          
          <button
            onClick={saveSettings}
            className={`btn btn-primary ${hasChanges ? 'has-changes' : ''}`}
            disabled={!hasChanges || isLoading || saveStatus === 'saving'}
          >
            {saveStatus === 'saving' && <div className="btn-spinner"></div>}
            {saveStatus === 'success' && '✅ '}
            {saveStatus === 'error' && '❌ '}
            {saveStatus === 'saving' ? '保存中...' : '保存设置'}
          </button>
        </div>

        {hasChanges && (
          <div className="changes-indicator">
            <span className="changes-icon">●</span>
            <span>有未保存的更改</span>
          </div>
        )}

        {error && (
          <div className="error-message">
            <span className="error-icon">⚠️</span>
            <span>{error}</span>
          </div>
        )}
      </div>
    </div>
  );
};

export default ProgressiveVoiceInputSettings;