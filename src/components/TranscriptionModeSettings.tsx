import React, { useState, useEffect } from 'react';
import { useNetworkStore } from '../stores/networkStore';

interface SettingsConfig {
  networkQualityThreshold: number;
  debounceTime: number;
  preferLocal: boolean;
  autoSwitchEnabled: boolean;
  cloudFallbackEnabled: boolean;
  hybridModeEnabled: boolean;
}

interface ModeChangeHistoryItem {
  timestamp: number;
  from_mode: string;
  to_mode: string;
  reason: string;
  network_quality: number;
}

const TranscriptionModeSettings: React.FC = () => {
  const {
    networkStatus,
    transcriptionMode,
    setTranscriptionMode,
    modeHistory
  } = useNetworkStore();

  const [settings, setSettings] = useState<SettingsConfig>({
    networkQualityThreshold: 70,
    debounceTime: 5000,
    preferLocal: true,
    autoSwitchEnabled: true,
    cloudFallbackEnabled: true,
    hybridModeEnabled: false
  });

  const [showHistory, setShowHistory] = useState(false);

  useEffect(() => {
    // 加载保存的设置
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const savedSettings = localStorage.getItem('transcriptionModeSettings');
      if (savedSettings) {
        setSettings(JSON.parse(savedSettings));
      }
    } catch (error) {
      console.error('加载设置失败:', error);
    }
  };

  const saveSettings = async (newSettings: SettingsConfig) => {
    try {
      localStorage.setItem('transcriptionModeSettings', JSON.stringify(newSettings));
      setSettings(newSettings);
      // TODO: 向后端发送设置更新
    } catch (error) {
      console.error('保存设置失败:', error);
    }
  };

  const handleSettingChange = (key: keyof SettingsConfig, value: any) => {
    const newSettings = { ...settings, [key]: value };
    saveSettings(newSettings);
  };

  const getNetworkQualityColor = (quality: number) => {
    if (quality >= 80) return 'text-green-500';
    if (quality >= 50) return 'text-yellow-500';
    return 'text-red-500';
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp).toLocaleString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  };

  const resetToDefaults = () => {
    const defaultSettings: SettingsConfig = {
      networkQualityThreshold: 70,
      debounceTime: 5000,
      preferLocal: true,
      autoSwitchEnabled: true,
      cloudFallbackEnabled: true,
      hybridModeEnabled: false
    };
    saveSettings(defaultSettings);
  };

  return (
    <div className="bg-white rounded-lg shadow-lg p-6 max-w-2xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-800">转录模式设置</h2>
        <button
          onClick={resetToDefaults}
          className="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded-md transition-colors"
        >
          重置默认
        </button>
      </div>

      {/* 当前状态概览 */}
      <div className="bg-gray-50 rounded-lg p-4 mb-6">
        <h3 className="text-lg font-semibold mb-3">当前状态</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <span className="text-sm text-gray-600">当前模式：</span>
            <span className="ml-2 px-2 py-1 bg-blue-100 text-blue-800 rounded text-sm font-medium">
              {transcriptionMode === 'Local' ? '本地' :
               transcriptionMode === 'Cloud' ? '云端' :
               transcriptionMode === 'Auto' ? '自动' : '混合'}
            </span>
          </div>
          <div>
            <span className="text-sm text-gray-600">网络状态：</span>
            <span className={`ml-2 font-medium ${getNetworkQualityColor(networkStatus?.quality_score || 0)}`}>
              {networkStatus?.status === 'Online' ? '在线' :
               networkStatus?.status === 'Limited' ? '受限' :
               networkStatus?.status === 'Offline' ? '离线' : '未知'}
              {networkStatus?.quality_score && ` (${networkStatus.quality_score}%)`}
            </span>
          </div>
        </div>
      </div>

      {/* 自动切换设置 */}
      <div className="mb-6">
        <h3 className="text-lg font-semibold mb-4">自动切换设置</h3>

        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700">启用自动切换</label>
              <p className="text-xs text-gray-500">根据网络状况自动切换转录模式</p>
            </div>
            <input
              type="checkbox"
              checked={settings.autoSwitchEnabled}
              onChange={(e) => handleSettingChange('autoSwitchEnabled', e.target.checked)}
              className="h-4 w-4 text-blue-600 rounded border-gray-300"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700">
                网络质量阈值: {settings.networkQualityThreshold}%
              </label>
              <p className="text-xs text-gray-500">低于此值时切换到本地模式</p>
            </div>
            <input
              type="range"
              min="30"
              max="90"
              value={settings.networkQualityThreshold}
              onChange={(e) => handleSettingChange('networkQualityThreshold', parseInt(e.target.value))}
              className="w-32"
              disabled={!settings.autoSwitchEnabled}
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700">
                切换延迟: {settings.debounceTime / 1000}秒
              </label>
              <p className="text-xs text-gray-500">避免频繁切换的等待时间</p>
            </div>
            <select
              value={settings.debounceTime}
              onChange={(e) => handleSettingChange('debounceTime', parseInt(e.target.value))}
              className="px-3 py-1 border border-gray-300 rounded-md text-sm"
              disabled={!settings.autoSwitchEnabled}
            >
              <option value={3000}>3秒</option>
              <option value={5000}>5秒</option>
              <option value={10000}>10秒</option>
              <option value={15000}>15秒</option>
            </select>
          </div>
        </div>
      </div>

      {/* 模式偏好设置 */}
      <div className="mb-6">
        <h3 className="text-lg font-semibold mb-4">模式偏好</h3>

        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700">优先本地处理</label>
              <p className="text-xs text-gray-500">在网络状况良好时仍优先使用本地模式</p>
            </div>
            <input
              type="checkbox"
              checked={settings.preferLocal}
              onChange={(e) => handleSettingChange('preferLocal', e.target.checked)}
              className="h-4 w-4 text-blue-600 rounded border-gray-300"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700">启用云端备用</label>
              <p className="text-xs text-gray-500">本地处理失败时自动切换到云端</p>
            </div>
            <input
              type="checkbox"
              checked={settings.cloudFallbackEnabled}
              onChange={(e) => handleSettingChange('cloudFallbackEnabled', e.target.checked)}
              className="h-4 w-4 text-blue-600 rounded border-gray-300"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700">启用混合模式</label>
              <p className="text-xs text-gray-500">同时使用本地和云端进行处理</p>
            </div>
            <input
              type="checkbox"
              checked={settings.hybridModeEnabled}
              onChange={(e) => handleSettingChange('hybridModeEnabled', e.target.checked)}
              className="h-4 w-4 text-blue-600 rounded border-gray-300"
            />
          </div>
        </div>
      </div>

      {/* 模式切换历史 */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold">切换历史</h3>
          <button
            onClick={() => setShowHistory(!showHistory)}
            className="text-sm text-blue-600 hover:text-blue-800"
          >
            {showHistory ? '隐藏' : '显示'}
          </button>
        </div>

        {showHistory && (
          <div className="bg-gray-50 rounded-lg p-4 max-h-64 overflow-y-auto">
            {modeHistory && modeHistory.length > 0 ? (
              <div className="space-y-2">
                {modeHistory.slice(0, 10).map((item: ModeChangeHistoryItem, index: number) => (
                  <div key={index} className="flex items-center justify-between py-2 border-b border-gray-200 last:border-b-0">
                    <div className="flex-1">
                      <div className="flex items-center space-x-2">
                        <span className="text-sm font-medium">
                          {item.from_mode} → {item.to_mode}
                        </span>
                        <span className={`text-xs ${getNetworkQualityColor(item.network_quality)}`}>
                          ({item.network_quality}%)
                        </span>
                      </div>
                      <div className="text-xs text-gray-500">{item.reason}</div>
                    </div>
                    <div className="text-xs text-gray-400">
                      {formatTimestamp(item.timestamp)}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center text-gray-500 py-8">
                <p>暂无切换历史</p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* 保存提示 */}
      <div className="mt-6 p-3 bg-blue-50 rounded-lg">
        <p className="text-sm text-blue-700">
          💡 所有设置会自动保存，并在下次启动时生效。
        </p>
      </div>
    </div>
  );
};

export default TranscriptionModeSettings;
