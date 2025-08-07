import React, { useState, useEffect } from 'react';
import { permissionManager, Permission, PermissionCheckResult } from '../utils/permissionManager';
import './PermissionSettings.css';

interface PermissionSettingsProps {
  isVisible: boolean;
  onClose: () => void;
  onPermissionsConfigured?: () => void;
}

const PermissionSettings: React.FC<PermissionSettingsProps> = ({
  isVisible,
  onClose,
  onPermissionsConfigured
}) => {
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [checkingPermissions, setCheckingPermissions] = useState(false);
  const [activeCategory, setActiveCategory] = useState<'all' | 'system' | 'audio' | 'file' | 'notification' | 'screen'>('all');
  const [permissionResults, setPermissionResults] = useState<Map<string, PermissionCheckResult>>(new Map());

  useEffect(() => {
    if (isVisible) {
      loadPermissions();
      checkAllPermissions();
      
      // 开始监控权限变化
      permissionManager.startPermissionMonitoring(3000);
      
      // 监听权限变化事件
      const handlePermissionChange = (id: string, oldStatus: string, newStatus: string) => {
        console.log(`权限 ${id} 状态变化: ${oldStatus} -> ${newStatus}`);
        checkAllPermissions(); // 重新检查所有权限
      };
      
      permissionManager.on('permission-changed', handlePermissionChange);
      
      return () => {
        permissionManager.off('permission-changed', handlePermissionChange);
      };
    }
  }, [isVisible]);

  const loadPermissions = () => {
    const allPermissions = permissionManager.getPermissions();
    setPermissions(allPermissions);
  };

  const checkAllPermissions = async () => {
    setCheckingPermissions(true);
    const results = await permissionManager.checkAllPermissions();
    setPermissionResults(results);
    setCheckingPermissions(false);
  };

  const handleRequestPermission = async (permissionId: string) => {
    const success = await permissionManager.requestPermission(permissionId);
    if (success) {
      // 等待用户设置权限后重新检查
      setTimeout(() => {
        checkAllPermissions();
      }, 2000);
    }
  };

  const handleQuickSetup = async () => {
    const success = await permissionManager.showPermissionWizard();
    if (success) {
      checkAllPermissions();
      if (onPermissionsConfigured) {
        onPermissionsConfigured();
      }
    }
  };

  const getFilteredPermissions = () => {
    if (activeCategory === 'all') {
      return permissions;
    }
    return permissionManager.getPermissionsByCategory(activeCategory);
  };

  const getStatusIcon = (status: Permission['status']) => {
    switch (status) {
      case 'granted': return '✅';
      case 'denied': return '❌';
      case 'not-determined': return '⏳';
      default: return '❓';
    }
  };

  const getStatusColor = (status: Permission['status']) => {
    switch (status) {
      case 'granted': return '#4caf50';
      case 'denied': return '#f44336';
      case 'not-determined': return '#ff9800';
      default: return '#808080';
    }
  };

  const getCategoryCount = (category: string) => {
    const categoryPermissions = category === 'all' 
      ? permissions 
      : permissions.filter(p => p.category === category);
    
    const grantedCount = categoryPermissions.filter(p => {
      const result = permissionResults.get(p.id);
      return result?.status === 'granted';
    }).length;
    
    return `${grantedCount}/${categoryPermissions.length}`;
  };

  if (!isVisible) return null;

  return (
    <div className="permission-settings-overlay" onClick={onClose}>
      <div className="permission-settings-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="permission-header">
          <div className="header-content">
            <h2>🔐 权限设置</h2>
            <p>管理 Recording King 所需的系统权限，确保所有功能正常工作</p>
          </div>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        {/* 主内容滚动容器 */}
        <div className="permission-main-content">
          {/* 快速设置横幅 */}
          <div className="permission-banner">
            <div className="banner-content">
              <div className="banner-icon">⚡</div>
              <div className="banner-text">
                <h3>快速设置向导</h3>
                <p>一键配置所有必需权限，确保最佳体验</p>
              </div>
              <button className="quick-setup-btn" onClick={handleQuickSetup}>
                开始设置
              </button>
            </div>
          </div>

          {/* 分类标签 */}
          <div className="permission-tabs">
            <button 
              className={`permission-tab ${activeCategory === 'all' ? 'active' : ''}`}
              onClick={() => setActiveCategory('all')}
            >
              全部 ({getCategoryCount('all')})
            </button>
            <button 
              className={`permission-tab ${activeCategory === 'system' ? 'active' : ''}`}
              onClick={() => setActiveCategory('system')}
            >
              🖥️ 系统 ({getCategoryCount('system')})
            </button>
            <button 
              className={`permission-tab ${activeCategory === 'audio' ? 'active' : ''}`}
              onClick={() => setActiveCategory('audio')}
            >
              🎤 音频 ({getCategoryCount('audio')})
            </button>
            <button 
              className={`permission-tab ${activeCategory === 'file' ? 'active' : ''}`}
              onClick={() => setActiveCategory('file')}
            >
              📁 文件 ({getCategoryCount('file')})
            </button>
            <button 
              className={`permission-tab ${activeCategory === 'notification' ? 'active' : ''}`}
              onClick={() => setActiveCategory('notification')}
            >
              🔔 通知 ({getCategoryCount('notification')})
            </button>
            <button 
              className={`permission-tab ${activeCategory === 'screen' ? 'active' : ''}`}
              onClick={() => setActiveCategory('screen')}
            >
              🖥️ 屏幕 ({getCategoryCount('screen')})
            </button>
          </div>

          {/* 权限列表 */}
          <div className="permission-list">
            {getFilteredPermissions().map(permission => {
              const result = permissionResults.get(permission.id);
              const status = result?.status || permission.status;
              
              return (
                <div 
                  key={permission.id} 
                  className={`permission-item ${status} ${permission.required ? 'required' : ''}`}
                >
                  <div className="permission-left">
                    <div className="permission-icon">{permission.icon}</div>
                    <div className="permission-info">
                      <div className="permission-name">
                        {permission.name}
                        {permission.required && <span className="required-badge">必需</span>}
                      </div>
                      <div className="permission-description">{permission.description}</div>
                    </div>
                  </div>
                  
                  <div className="permission-right">
                    <div 
                      className="permission-status"
                      style={{ color: getStatusColor(status) }}
                    >
                      {getStatusIcon(status)} {result?.message || '检查中...'}
                    </div>
                    
                    {status !== 'granted' && (
                      <button 
                        className="grant-btn"
                        onClick={() => handleRequestPermission(permission.id)}
                        disabled={checkingPermissions}
                      >
                        {status === 'denied' ? '重新请求' : '授予权限'}
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {/* 帮助信息 */}
          <div className="permission-help">
            <div className="help-section">
              <h4>🍎 macOS 权限设置路径</h4>
              <ul>
                <li><strong>辅助功能</strong>: 系统偏好设置 → 安全性与隐私 → 隐私 → 辅助功能</li>
                <li><strong>麦克风</strong>: 系统偏好设置 → 安全性与隐私 → 隐私 → 麦克风</li>
                <li><strong>输入监控</strong>: 系统偏好设置 → 安全性与隐私 → 隐私 → 输入监控</li>
                <li><strong>文件访问</strong>: 系统偏好设置 → 安全性与隐私 → 隐私 → 文件和文件夹</li>
                <li><strong>屏幕录制</strong>: 系统偏好设置 → 安全性与隐私 → 隐私 → 屏幕录制</li>
                <li><strong>通知</strong>: 系统偏好设置 → 通知与专注模式 → 通知 → Recording King</li>
              </ul>
            </div>
            
            <div className="help-section">
              <h4>⚠️ 常见问题与解决方案</h4>
              <ul>
                <li>• 修改权限后可能需要重启应用才能生效</li>
                <li>• 某些权限设置需要输入管理员密码</li>
                <li>• 快捷键功能需要辅助功能和输入监控权限</li>
                <li>• 首次请求权限时系统会弹出确认对话框</li>
                <li>• 如果权限被拒绝，需要手动到系统设置中开启</li>
                <li>• 语音识别功能需要麦克风权限</li>
                <li>• 文件保存和加载需要文件访问权限</li>
                <li>• 屏幕共享功能需要屏幕录制权限</li>
              </ul>
            </div>
          </div>
        </div>

        {/* 底部操作栏 */}
        <div className="permission-footer">
          <div className="footer-left">
            <button 
              className="refresh-btn"
              onClick={checkAllPermissions}
              disabled={checkingPermissions}
            >
              {checkingPermissions ? '检查中...' : '🔄 刷新状态'}
            </button>
          </div>
          
          <div className="footer-right">
            <button className="done-btn" onClick={onClose}>
              完成
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PermissionSettings;