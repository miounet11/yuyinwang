import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './PermissionManager.css';

interface PermissionStatus {
  accessibility: boolean;
  input_monitoring: boolean;
  microphone: boolean;
  all_granted: boolean;
}

interface PermissionGuide {
  step: number;
  title: string;
  description: string;
  action: string;
  image_path?: string;
  is_critical: boolean;
}

interface PermissionInfo {
  status: PermissionStatus;
  guide: PermissionGuide[];
  missing_permissions: string[];
  critical_issue: boolean;
  can_use_shortcuts: boolean;
  can_record_audio: boolean;
  next_action: string;
}

interface PermissionManagerProps {
  onPermissionChange?: (hasAllPermissions: boolean) => void;
  showModal?: boolean;
  onClose?: () => void;
}

const PermissionManager: React.FC<PermissionManagerProps> = ({
  onPermissionChange,
  showModal = false,
  onClose
}) => {
  const [permissionInfo, setPermissionInfo] = useState<PermissionInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [currentStep, setCurrentStep] = useState(0);
  const [showGuide, setShowGuide] = useState(false);
  const [isChecking, setIsChecking] = useState(false);

  const checkPermissions = async () => {
    setIsChecking(true);
    try {
      const result = await invoke<PermissionInfo>('check_all_permissions');
      setPermissionInfo(result);
      
      if (onPermissionChange) {
        onPermissionChange(result.status.all_granted);
      }
      
      // 如果缺少关键权限，自动显示指南
      if (result.critical_issue || result.missing_permissions.length > 0) {
        setShowGuide(true);
      }
    } catch (error) {
      console.error('检查权限失败:', error);
    } finally {
      setIsChecking(false);
    }
  };

  useEffect(() => {
    const initCheck = async () => {
      await checkPermissions();
      setLoading(false);
    };
    initCheck();

    // 每30秒检查一次权限状态（减少日志频率）
    const interval = setInterval(checkPermissions, 30000);
    return () => clearInterval(interval);
  }, []);

  const openSystemSettings = async (panel: string) => {
    try {
      await invoke('open_permission_settings', { panel });
    } catch (error) {
      console.error('打开系统设置失败:', error);
    }
  };

  const showWarningDialog = async () => {
    if (!permissionInfo) return;
    
    try {
      await invoke('show_permission_warning_dialog', {
        missingPermissions: permissionInfo.missing_permissions
      });
    } catch (error) {
      console.error('显示警告对话框失败:', error);
    }
  };

  const nextStep = () => {
    if (permissionInfo && currentStep < permissionInfo.guide.length - 1) {
      setCurrentStep(currentStep + 1);
    }
  };

  const prevStep = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const getStatusIcon = (hasPermission: boolean) => {
    return hasPermission ? '✅' : '❌';
  };

  const getStatusColor = (hasPermission: boolean) => {
    return hasPermission ? '#4CAF50' : '#F44336';
  };

  if (loading) {
    return (
      <div className="permission-manager loading">
        <div className="loading-spinner"></div>
        <p>正在检查权限状态...</p>
      </div>
    );
  }

  if (!permissionInfo) {
    return (
      <div className="permission-manager error">
        <h3>⚠️ 权限检查失败</h3>
        <p>无法获取系统权限状态，请重试</p>
        <button onClick={checkPermissions} className="retry-button">
          重新检查
        </button>
      </div>
    );
  }

  const isModalView = showModal;

  return (
    <div className={`permission-manager ${isModalView ? 'modal' : ''}`}>
      {isModalView && (
        <div className="modal-overlay" onClick={onClose}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            {onClose && (
              <button className="modal-close" onClick={onClose}>×</button>
            )}
            <PermissionContent />
          </div>
        </div>
      )}
      
      {!isModalView && <PermissionContent />}
    </div>
  );

  function PermissionContent() {
    return (
      <>
        <div className="permission-header">
          <h2>🔒 系统权限管理</h2>
          <p className="subtitle">Recording King 需要以下权限才能正常工作</p>
        </div>

        <div className="permission-status">
          <div className="permission-item">
            <div className="permission-info">
              <span className="permission-icon">🎤</span>
              <div>
                <h3>麦克风权限</h3>
                <p>用于录制音频和语音识别</p>
              </div>
            </div>
            <div 
              className="permission-status-indicator"
              style={{ color: getStatusColor(permissionInfo.status.microphone) }}
            >
              {getStatusIcon(permissionInfo.status.microphone)}
            </div>
          </div>

          <div className="permission-item">
            <div className="permission-info">
              <span className="permission-icon">📱</span>
              <div>
                <h3>辅助功能权限</h3>
                <p>用于系统集成和自动化操作</p>
              </div>
            </div>
            <div 
              className="permission-status-indicator"
              style={{ color: getStatusColor(permissionInfo.status.accessibility) }}
            >
              {getStatusIcon(permissionInfo.status.accessibility)}
            </div>
          </div>

          <div className="permission-item critical">
            <div className="permission-info">
              <span className="permission-icon">⌨️</span>
              <div>
                <h3>输入监控权限 <span className="critical-badge">必需</span></h3>
                <p>用于全局快捷键功能（核心功能）</p>
              </div>
            </div>
            <div 
              className="permission-status-indicator"
              style={{ color: getStatusColor(permissionInfo.status.input_monitoring) }}
            >
              {getStatusIcon(permissionInfo.status.input_monitoring)}
            </div>
          </div>
        </div>

        {!permissionInfo.status.all_granted && (
          <div className="permission-alert">
            <div className="alert-icon">⚠️</div>
            <div className="alert-content">
              <h3>权限配置不完整</h3>
              <p>
                缺少权限：{permissionInfo.missing_permissions.join('、')}
                {permissionInfo.critical_issue && (
                  <span className="critical-warning">
                    <br />⚠️ 输入监控权限缺失将导致快捷键功能完全无法使用
                  </span>
                )}
              </p>
            </div>
          </div>
        )}

        {permissionInfo.status.all_granted && (
          <div className="permission-success">
            <div className="success-icon">✅</div>
            <div className="success-content">
              <h3>所有权限已配置</h3>
              <p>Recording King 已获得所有必要权限，功能完整可用</p>
            </div>
          </div>
        )}

        <div className="permission-actions">
          {!permissionInfo.status.all_granted && (
            <>
              <button 
                className="primary-button"
                onClick={() => setShowGuide(true)}
              >
                📋 查看配置指南
              </button>
              
              <button 
                className="secondary-button"
                onClick={() => openSystemSettings('security')}
              >
                🔧 打开系统设置
              </button>

              <button 
                className="secondary-button"
                onClick={showWarningDialog}
              >
                ⚠️ 显示权限警告
              </button>
            </>
          )}

          <button 
            className={isChecking ? "checking-button" : "refresh-button"}
            onClick={checkPermissions}
            disabled={isChecking}
          >
            {isChecking ? '🔄 检查中...' : '🔄 重新检查'}
          </button>
        </div>

        {showGuide && (
          <div className="permission-guide">
            <div className="guide-header">
              <h3>📋 权限配置指南</h3>
              <div className="guide-progress">
                步骤 {currentStep + 1} / {permissionInfo.guide.length}
              </div>
            </div>

            <div className="guide-content">
              {permissionInfo.guide[currentStep] && (
                <div className="guide-step">
                  <h4>{permissionInfo.guide[currentStep].title}</h4>
                  <p>{permissionInfo.guide[currentStep].description}</p>
                  
                  {permissionInfo.guide[currentStep].action === 'open_system_preferences' && (
                    <button 
                      className="action-button"
                      onClick={() => openSystemSettings('security')}
                    >
                      🔧 打开系统偏好设置
                    </button>
                  )}
                  
                  {permissionInfo.guide[currentStep].action === 'enable_input_monitoring' && (
                    <button 
                      className="action-button"
                      onClick={() => openSystemSettings('input_monitoring')}
                    >
                      ⌨️ 打开输入监控设置
                    </button>
                  )}
                  
                  {permissionInfo.guide[currentStep].action === 'enable_accessibility' && (
                    <button 
                      className="action-button"
                      onClick={() => openSystemSettings('accessibility')}
                    >
                      📱 打开辅助功能设置
                    </button>
                  )}
                  
                  {permissionInfo.guide[currentStep].action === 'enable_microphone' && (
                    <button 
                      className="action-button"
                      onClick={() => openSystemSettings('microphone')}
                    >
                      🎤 打开麦克风设置
                    </button>
                  )}
                </div>
              )}
            </div>

            <div className="guide-navigation">
              <button 
                onClick={prevStep}
                disabled={currentStep === 0}
                className="nav-button"
              >
                ← 上一步
              </button>
              
              <button 
                onClick={() => setShowGuide(false)}
                className="nav-button close-guide"
              >
                关闭指南
              </button>
              
              <button 
                onClick={nextStep}
                disabled={currentStep === permissionInfo.guide.length - 1}
                className="nav-button"
              >
                下一步 →
              </button>
            </div>
          </div>
        )}
      </>
    );
  }
};

export default PermissionManager;