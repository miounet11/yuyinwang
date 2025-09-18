// Story 1.4: Transcription Mode Selector Component

import React, { useEffect, useState } from 'react';
import { useNetworkStore } from '../stores/networkStore';
import NetworkStatusIndicator from './NetworkStatusIndicator';

interface TranscriptionModeSelectorProps {
  className?: string;
  showRecommendation?: boolean;
  compact?: boolean;
}

const TranscriptionModeSelector: React.FC<TranscriptionModeSelectorProps> = ({
  className = '',
  showRecommendation = true,
  compact = false
}) => {
  const {
    modeStatus,
    isModeLoading,
    modeError,
    fetchModeStatus,
    setTranscriptionMode,
    forceReevaluateMode,
    subscribeToModeChanges
  } = useNetworkStore();

  const [localSelectedMode, setLocalSelectedMode] = useState<string>('');

  useEffect(() => {
    // 初始化时获取模式状态
    fetchModeStatus();

    // 订阅模式变化
    subscribeToModeChanges();
  }, []);

  useEffect(() => {
    // 同步本地选择状态
    if (modeStatus?.current_mode && !localSelectedMode) {
      setLocalSelectedMode(modeStatus.current_mode);
    }
  }, [modeStatus?.current_mode]);

  const modes = [
    {
      id: 'auto',
      name: '智能模式',
      description: '根据网络状况自动选择最佳转录方式',
      icon: '🤖',
      color: 'blue'
    },
    {
      id: 'local',
      name: '本地模式',
      description: '使用本地Whisper模型进行转录',
      icon: '💻',
      color: 'green'
    },
    {
      id: 'cloud',
      name: '云端模式',
      description: '使用在线API进行转录',
      icon: '☁️',
      color: 'purple'
    },
    {
      id: 'hybrid',
      name: '混合模式',
      description: '同时使用本地和云端，选择最佳结果',
      icon: '⚡',
      color: 'orange'
    }
  ];

  const handleModeChange = async (modeId: string) => {
    if (isModeLoading) return;

    setLocalSelectedMode(modeId);
    try {
      await setTranscriptionMode(modeId);
    } catch (error) {
      // 如果失败，恢复之前的选择
      setLocalSelectedMode(modeStatus?.current_mode || '');
    }
  };

  const handleReevaluate = async () => {
    try {
      await forceReevaluateMode();
    } catch (error) {
      console.error('Failed to reevaluate mode:', error);
    }
  };

  const getModeColor = (modeId: string) => {
    const mode = modes.find(m => m.id === modeId);
    return mode?.color || 'gray';
  };

  const getColorClasses = (color: string, isSelected: boolean) => {
    const baseClasses = 'transition-all duration-200';

    if (isSelected) {
      switch (color) {
        case 'blue':
          return `${baseClasses} bg-blue-50 border-blue-200 text-blue-700`;
        case 'green':
          return `${baseClasses} bg-green-50 border-green-200 text-green-700`;
        case 'purple':
          return `${baseClasses} bg-purple-50 border-purple-200 text-purple-700`;
        case 'orange':
          return `${baseClasses} bg-orange-50 border-orange-200 text-orange-700`;
        default:
          return `${baseClasses} bg-gray-50 border-gray-200 text-gray-700`;
      }
    }

    return `${baseClasses} bg-white border-gray-200 text-gray-600 hover:bg-gray-50 hover:border-gray-300`;
  };

  if (compact) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <select
          value={localSelectedMode}
          onChange={(e) => handleModeChange(e.target.value)}
          disabled={isModeLoading}
          className="px-3 py-1 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:opacity-50"
        >
          {modes.map((mode) => (
            <option key={mode.id} value={mode.id}>
              {mode.icon} {mode.name}
            </option>
          ))}
        </select>

        {isModeLoading && (
          <div className="w-4 h-4 border-2 border-gray-300 border-t-blue-600 rounded-full animate-spin"></div>
        )}
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* 标题和状态 */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-gray-900">转录模式</h3>
        <div className="flex items-center gap-2">
          <NetworkStatusIndicator size="small" />
          {modeStatus?.current_mode !== modeStatus?.active_mode && (
            <span className="text-xs px-2 py-1 bg-yellow-100 text-yellow-700 rounded-full">
              切换中
            </span>
          )}
        </div>
      </div>

      {/* 错误提示 */}
      {modeError && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-md">
          <p className="text-sm text-red-700">{modeError}</p>
        </div>
      )}

      {/* 当前状态信息 */}
      {modeStatus && (
        <div className="p-3 bg-gray-50 rounded-md space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-600">当前模式:</span>
            <span className="font-medium text-gray-900">
              {modes.find(m => m.id.toLowerCase() === modeStatus.active_mode.toLowerCase())?.name || modeStatus.active_mode}
            </span>
          </div>

          {showRecommendation && modeStatus.recommendation && (
            <div className="text-xs text-blue-600 bg-blue-50 p-2 rounded border">
              💡 {modeStatus.recommendation}
            </div>
          )}
        </div>
      )}

      {/* 模式选择 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {modes.map((mode) => {
          const isSelected = localSelectedMode.toLowerCase() === mode.id.toLowerCase();
          const isActive = modeStatus?.active_mode.toLowerCase() === mode.id.toLowerCase();

          return (
            <button
              key={mode.id}
              onClick={() => handleModeChange(mode.id)}
              disabled={isModeLoading}
              className={`
                relative p-4 border-2 rounded-lg text-left
                ${getColorClasses(mode.color, isSelected)}
                disabled:opacity-50 disabled:cursor-not-allowed
                focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500
              `}
            >
              {/* 活动指示器 */}
              {isActive && (
                <div className="absolute top-2 right-2 w-2 h-2 bg-green-500 rounded-full"></div>
              )}

              <div className="flex items-start gap-3">
                <span className="text-2xl">{mode.icon}</span>
                <div className="flex-1">
                  <h4 className="font-medium">{mode.name}</h4>
                  <p className="text-sm opacity-75 mt-1">{mode.description}</p>
                </div>
              </div>

              {/* 选中指示器 */}
              {isSelected && (
                <div className="absolute bottom-2 right-2">
                  <span className="text-sm">✓</span>
                </div>
              )}
            </button>
          );
        })}
      </div>

      {/* 操作按钮 */}
      <div className="flex items-center gap-2 pt-2">
        <button
          onClick={handleReevaluate}
          disabled={isModeLoading}
          className="px-4 py-2 text-sm border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 transition-colors"
        >
          🔄 重新评估
        </button>

        {modeStatus?.auto_switch_enabled && (
          <div className="flex items-center gap-1 text-sm text-green-600">
            <span>🤖</span>
            <span>智能切换已启用</span>
          </div>
        )}

        {isModeLoading && (
          <div className="flex items-center gap-2 text-sm text-gray-500">
            <div className="w-4 h-4 border-2 border-gray-300 border-t-blue-600 rounded-full animate-spin"></div>
            <span>正在切换...</span>
          </div>
        )}
      </div>
    </div>
  );
};

export default TranscriptionModeSelector;
