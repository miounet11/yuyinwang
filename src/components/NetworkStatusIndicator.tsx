// Story 1.4: Network Status Real-time Indicator Component

import React, { useEffect, useState } from 'react';
import { useNetworkStore } from '../stores/networkStore';

interface NetworkStatusIndicatorProps {
  showDetails?: boolean;
  className?: string;
  size?: 'small' | 'medium' | 'large';
}

const NetworkStatusIndicator: React.FC<NetworkStatusIndicatorProps> = ({
  showDetails = false,
  className = '',
  size = 'medium'
}) => {
  const {
    networkStatus,
    isNetworkLoading,
    fetchNetworkStatus,
    checkNetworkNow,
    subscribeToNetworkChanges
  } = useNetworkStore();

  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  useEffect(() => {
    // 初始化时获取网络状态
    fetchNetworkStatus();

    // 订阅网络状态变化
    subscribeToNetworkChanges();

    // 每30秒更新一次时间显示
    const interval = setInterval(() => {
      setLastUpdate(new Date());
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Online':
        return '#10B981'; // green-500
      case 'Limited':
        return '#F59E0B'; // amber-500
      case 'Offline':
        return '#EF4444'; // red-500
      case 'Unknown':
      default:
        return '#6B7280'; // gray-500
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'Online':
        return '🟢';
      case 'Limited':
        return '🟡';
      case 'Offline':
        return '🔴';
      case 'Unknown':
      default:
        return '⚫';
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'Online':
        return '网络正常';
      case 'Limited':
        return '网络受限';
      case 'Offline':
        return '网络离线';
      case 'Unknown':
      default:
        return '网络未知';
    }
  };

  const getQualityText = (quality: number) => {
    if (quality >= 0.8) return '优秀';
    if (quality >= 0.6) return '良好';
    if (quality >= 0.4) return '一般';
    if (quality >= 0.2) return '较差';
    return '很差';
  };

  const getSizeClasses = () => {
    switch (size) {
      case 'small':
        return 'text-xs';
      case 'large':
        return 'text-lg';
      case 'medium':
      default:
        return 'text-sm';
    }
  };

  const formatLastUpdate = () => {
    if (!networkStatus) return '';
    const seconds = Math.floor((Date.now() - networkStatus.last_checked * 1000) / 1000);
    if (seconds < 60) return `${seconds}秒前`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}分钟前`;
    const hours = Math.floor(minutes / 60);
    return `${hours}小时前`;
  };

  const handleRefresh = async () => {
    await checkNetworkNow();
    setLastUpdate(new Date());
  };

  if (isNetworkLoading && !networkStatus) {
    return (
      <div className={`flex items-center gap-2 ${getSizeClasses()} ${className}`}>
        <div className="animate-spin rounded-full h-4 w-4 border-2 border-gray-300 border-t-blue-600"></div>
        <span className="text-gray-600">检测网络...</span>
      </div>
    );
  }

  if (!networkStatus) {
    return (
      <div className={`flex items-center gap-2 ${getSizeClasses()} ${className}`}>
        <span>⚫</span>
        <span className="text-gray-400">网络状态未知</span>
      </div>
    );
  }

  return (
    <div className={`flex items-center gap-2 ${getSizeClasses()} ${className}`}>
      {/* 状态指示器 */}
      <div className="flex items-center gap-1">
        <span style={{ color: getStatusColor(networkStatus.status) }}>
          {getStatusIcon(networkStatus.status)}
        </span>
        {showDetails && (
          <span
            className="font-medium"
            style={{ color: getStatusColor(networkStatus.status) }}
          >
            {getStatusText(networkStatus.status)}
          </span>
        )}
      </div>

      {/* 详细信息 */}
      {showDetails && (
        <div className="flex items-center gap-3 text-gray-600">
          {/* 网络质量 */}
          <div className="flex items-center gap-1">
            <span className="text-xs text-gray-500">质量:</span>
            <span className="font-medium">
              {getQualityText(networkStatus.quality_score)}
            </span>
            <span className="text-xs text-gray-400">
              ({Math.round(networkStatus.quality_score * 100)}%)
            </span>
          </div>

          {/* 连接状态 */}
          <div className="flex items-center gap-1">
            <span className="text-xs text-gray-500">
              {networkStatus.is_connected ? '已连接' : '未连接'}
            </span>
          </div>

          {/* 失败次数 */}
          {networkStatus.consecutive_failures > 0 && (
            <div className="flex items-center gap-1">
              <span className="text-xs text-red-500">
                连续失败 {networkStatus.consecutive_failures} 次
              </span>
            </div>
          )}
        </div>
      )}

      {/* 最后更新时间 */}
      {showDetails && (
        <div className="flex items-center gap-1 text-xs text-gray-400">
          <span>更新于</span>
          <span>{formatLastUpdate()}</span>
        </div>
      )}

      {/* 刷新按钮 */}
      {showDetails && (
        <button
          onClick={handleRefresh}
          disabled={isNetworkLoading}
          className="p-1 rounded hover:bg-gray-100 disabled:opacity-50 transition-colors"
          title="刷新网络状态"
        >
          <span className={`text-xs ${isNetworkLoading ? 'animate-spin' : ''}`}>
            🔄
          </span>
        </button>
      )}

      {/* 加载指示器 */}
      {isNetworkLoading && showDetails && (
        <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse"></div>
      )}
    </div>
  );
};

export default NetworkStatusIndicator;
