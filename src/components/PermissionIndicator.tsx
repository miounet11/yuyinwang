import React, { useState, useEffect } from 'react';
import { permissionManager } from '../utils/permissionManager';
import './PermissionIndicator.css';

interface PermissionIndicatorProps {
  onOpenSettings: () => void;
}

const PermissionIndicator: React.FC<PermissionIndicatorProps> = ({ onOpenSettings }) => {
  const [missingCount, setMissingCount] = useState(0);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    checkPermissions();
    
    // 监听权限变化
    const handlePermissionChange = () => {
      checkPermissions();
    };
    
    permissionManager.on('permission-granted', handlePermissionChange);
    permissionManager.on('permission-denied', handlePermissionChange);
    
    return () => {
      permissionManager.off('permission-granted', handlePermissionChange);
      permissionManager.off('permission-denied', handlePermissionChange);
    };
  }, []);

  const checkPermissions = async () => {
    setIsLoading(true);
    const missing = await permissionManager.getMissingRequiredPermissions();
    setMissingCount(missing.length);
    setIsLoading(false);
  };

  if (isLoading) {
    return (
      <div className="permission-indicator loading">
        <span className="loading-icon">⏳</span>
      </div>
    );
  }

  if (missingCount === 0) {
    return (
      <div className="permission-indicator success" title="所有权限已授予">
        <span className="success-icon">✅</span>
      </div>
    );
  }

  return (
    <div 
      className="permission-indicator warning"
      onClick={onOpenSettings}
      title={`${missingCount} 个必需权限未授予，点击设置`}
    >
      <span className="warning-icon">⚠️</span>
      <span className="warning-count">{missingCount}</span>
    </div>
  );
};

export default PermissionIndicator;