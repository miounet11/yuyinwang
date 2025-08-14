// 增强版权限设置向导 - 优化用户体验版本
import React, { useState, useEffect, useCallback } from 'react';
import { permissionManager, Permission, PermissionCheckResult } from '../utils/permissionManager';
import './EnhancedPermissionWizard.css';

interface EnhancedPermissionWizardProps {
  isVisible: boolean;
  onClose: () => void;
  onPermissionsConfigured?: () => void;
  mode?: 'wizard' | 'settings';
}

type WizardStep = 
  | 'welcome' 
  | 'system-check' 
  | 'microphone' 
  | 'accessibility' 
  | 'input-monitoring' 
  | 'files' 
  | 'notifications' 
  | 'screen-recording' 
  | 'complete';

interface StepInfo {
  id: WizardStep;
  title: string;
  description: string;
  icon: string;
  permissions: string[];
  importance: 'critical' | 'important' | 'optional';
  estimatedTime: string;
}

const EnhancedPermissionWizard: React.FC<EnhancedPermissionWizardProps> = ({
  isVisible,
  onClose,
  onPermissionsConfigured,
  mode = 'wizard'
}) => {
  const [currentStep, setCurrentStep] = useState<WizardStep>('welcome');
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [permissionResults, setPermissionResults] = useState<Map<string, PermissionCheckResult>>(new Map());
  const [checkingPermissions, setCheckingPermissions] = useState(false);
  const [completedSteps, setCompletedSteps] = useState<Set<WizardStep>>(new Set());
  const [skippedSteps, setSkippedSteps] = useState<Set<WizardStep>>(new Set());
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [autoMode, setAutoMode] = useState(true);

  const wizardSteps: StepInfo[] = [
    {
      id: 'welcome',
      title: '欢迎使用 Recording King',
      description: '让我们快速配置必需的系统权限，确保您获得最佳体验',
      icon: '👋',
      permissions: [],
      importance: 'critical',
      estimatedTime: '30秒'
    },
    {
      id: 'system-check',
      title: '系统检查',
      description: '正在检查当前权限状态和系统兼容性',
      icon: '🔍',
      permissions: [],
      importance: 'critical',
      estimatedTime: '10秒'
    },
    {
      id: 'microphone',
      title: '麦克风权限',
      description: '语音录制和转录功能需要访问麦克风',
      icon: '🎤',
      permissions: ['microphone'],
      importance: 'critical',
      estimatedTime: '1分钟'
    },
    {
      id: 'accessibility',
      title: '辅助功能权限',
      description: '快捷键和自动化功能需要辅助功能权限',
      icon: '♿',
      permissions: ['accessibility'],
      importance: 'critical',
      estimatedTime: '1分钟'
    },
    {
      id: 'input-monitoring',
      title: '输入监控权限',
      description: '全局快捷键功能需要监控键盘输入',
      icon: '⌨️',
      permissions: ['input-monitoring'],
      importance: 'important',
      estimatedTime: '1分钟'
    },
    {
      id: 'files',
      title: '文件访问权限',
      description: '保存录音和转录文件需要文件系统访问权限',
      icon: '📁',
      permissions: ['files', 'downloads'],
      importance: 'important',
      estimatedTime: '30秒'
    },
    {
      id: 'notifications',
      title: '通知权限',
      description: '后台处理完成时显示系统通知',
      icon: '🔔',
      permissions: ['notifications'],
      importance: 'optional',
      estimatedTime: '30秒'
    },
    {
      id: 'screen-recording',
      title: '屏幕录制权限',
      description: '屏幕共享和截图功能（可选）',
      icon: '🖥️',
      permissions: ['screen-recording'],
      importance: 'optional',
      estimatedTime: '30秒'
    },
    {
      id: 'complete',
      title: '设置完成',
      description: '权限配置完成，Ready to Record!',
      icon: '🎉',
      permissions: [],
      importance: 'critical',
      estimatedTime: ''
    }
  ];

  useEffect(() => {
    if (isVisible && mode === 'wizard') {
      loadPermissions();
      if (currentStep === 'system-check') {
        performSystemCheck();
      }
    }
  }, [isVisible, currentStep, mode]);

  const loadPermissions = useCallback(async () => {
    const allPermissions = permissionManager.getPermissions();
    setPermissions(allPermissions);
    
    if (autoMode && currentStep !== 'welcome' && currentStep !== 'complete') {
      await checkCurrentStepPermissions();
    }
  }, [currentStep, autoMode]);

  const performSystemCheck = useCallback(async () => {
    setCheckingPermissions(true);
    
    // 模拟系统检查过程
    await new Promise(resolve => setTimeout(resolve, 1500));
    
    const results = await permissionManager.checkAllPermissions();
    setPermissionResults(results);
    
    // 检查哪些步骤已经完成
    const newCompletedSteps = new Set<WizardStep>();
    wizardSteps.forEach(step => {
      if (step.permissions.length > 0) {
        const allGranted = step.permissions.every(permId => {
          const result = results.get(permId);
          return result?.status === 'granted';
        });
        if (allGranted) {
          newCompletedSteps.add(step.id);
        }
      }
    });
    
    setCompletedSteps(newCompletedSteps);
    setCheckingPermissions(false);
    
    // 自动进入下一步
    if (autoMode) {
      setTimeout(() => {
        goToNextStep();
      }, 1000);
    }
  }, [autoMode]);

  const checkCurrentStepPermissions = useCallback(async () => {
    const currentStepInfo = wizardSteps.find(s => s.id === currentStep);
    if (!currentStepInfo || currentStepInfo.permissions.length === 0) return;

    setCheckingPermissions(true);
    const results = new Map<string, PermissionCheckResult>();
    
    for (const permId of currentStepInfo.permissions) {
      const result = await permissionManager.checkPermission(permId);
      results.set(permId, result);
    }
    
    setPermissionResults(prev => new Map([...prev, ...results]));
    setCheckingPermissions(false);
  }, [currentStep]);

  const handleRequestPermissions = useCallback(async () => {
    const currentStepInfo = wizardSteps.find(s => s.id === currentStep);
    if (!currentStepInfo) return;

    setCheckingPermissions(true);
    
    let allGranted = true;
    for (const permId of currentStepInfo.permissions) {
      const success = await permissionManager.requestPermission(permId);
      if (!success) {
        allGranted = false;
      }
    }
    
    // 重新检查权限状态
    await checkCurrentStepPermissions();
    
    if (allGranted) {
      setCompletedSteps(prev => new Set([...prev, currentStep]));
      
      if (autoMode) {
        setTimeout(() => {
          goToNextStep();
        }, 1500);
      }
    }
    
    setCheckingPermissions(false);
  }, [currentStep, autoMode]);

  const goToNextStep = useCallback(() => {
    const currentIndex = wizardSteps.findIndex(s => s.id === currentStep);
    if (currentIndex < wizardSteps.length - 1) {
      const nextStep = wizardSteps[currentIndex + 1];
      setCurrentStep(nextStep.id);
    }
  }, [currentStep]);

  const goToPreviousStep = useCallback(() => {
    const currentIndex = wizardSteps.findIndex(s => s.id === currentStep);
    if (currentIndex > 0) {
      const prevStep = wizardSteps[currentIndex - 1];
      setCurrentStep(prevStep.id);
    }
  }, [currentStep]);

  const skipCurrentStep = useCallback(() => {
    setSkippedSteps(prev => new Set([...prev, currentStep]));
    goToNextStep();
  }, [currentStep, goToNextStep]);

  const getStepStatus = useCallback((stepId: WizardStep) => {
    if (completedSteps.has(stepId)) return 'completed';
    if (skippedSteps.has(stepId)) return 'skipped';
    if (stepId === currentStep) return 'current';
    
    const stepIndex = wizardSteps.findIndex(s => s.id === stepId);
    const currentIndex = wizardSteps.findIndex(s => s.id === currentStep);
    return stepIndex < currentIndex ? 'pending' : 'upcoming';
  }, [currentStep, completedSteps, skippedSteps]);

  const getOverallProgress = useCallback(() => {
    const totalSteps = wizardSteps.filter(s => s.permissions.length > 0).length;
    const completedCount = Array.from(completedSteps).filter(stepId => 
      wizardSteps.find(s => s.id === stepId)?.permissions.length! > 0
    ).length;
    return Math.round((completedCount / totalSteps) * 100);
  }, [completedSteps]);

  const handleClose = useCallback(() => {
    if (currentStep === 'complete' && onPermissionsConfigured) {
      onPermissionsConfigured();
    }
    onClose();
  }, [currentStep, onClose, onPermissionsConfigured]);

  const renderWelcomeStep = () => (
    <div className="wizard-welcome">
      <div className="welcome-hero">
        <div className="welcome-icon">👑</div>
        <h1>欢迎使用 Recording King</h1>
        <p>让我们花几分钟时间配置系统权限，确保所有功能正常工作</p>
      </div>
      
      <div className="welcome-features">
        <div className="feature-item">
          <span className="feature-icon">🎤</span>
          <div className="feature-text">
            <h3>智能语音识别</h3>
            <p>实时转录和AI处理</p>
          </div>
        </div>
        <div className="feature-item">
          <span className="feature-icon">⌨️</span>
          <div className="feature-text">
            <h3>全局快捷键</h3>
            <p>随时随地快速录音</p>
          </div>
        </div>
        <div className="feature-item">
          <span className="feature-icon">💬</span>
          <div className="feature-text">
            <h3>智能文本注入</h3>
            <p>直接插入到任何应用</p>
          </div>
        </div>
      </div>
      
      <div className="welcome-options">
        <label className="option-toggle">
          <input 
            type="checkbox" 
            checked={autoMode}
            onChange={(e) => setAutoMode(e.target.checked)}
          />
          <span>自动模式（推荐）</span>
        </label>
        <p className="option-description">
          自动模式将引导您完成所有必需步骤，跳过已配置的权限
        </p>
      </div>
    </div>
  );

  const renderSystemCheckStep = () => (
    <div className="wizard-system-check">
      <div className="check-header">
        <div className="check-icon rotating">🔍</div>
        <h2>系统检查中...</h2>
        <p>正在检查当前权限状态和系统兼容性</p>
      </div>
      
      <div className="check-progress">
        <div className="check-items">
          <div className="check-item">
            <span className="check-status">✓</span>
            <span>检查操作系统版本</span>
          </div>
          <div className="check-item">
            <span className="check-status">✓</span>
            <span>检查系统架构</span>
          </div>
          <div className="check-item active">
            <span className="check-status">⏳</span>
            <span>扫描权限状态</span>
          </div>
          <div className="check-item">
            <span className="check-status">⏳</span>
            <span>生成配置建议</span>
          </div>
        </div>
      </div>
    </div>
  );

  const renderPermissionStep = () => {
    const stepInfo = wizardSteps.find(s => s.id === currentStep);
    if (!stepInfo) return null;

    const stepPermissions = stepInfo.permissions.map(id => 
      permissions.find(p => p.id === id)
    ).filter(Boolean);

    const allGranted = stepPermissions.every(p => {
      const result = permissionResults.get(p!.id);
      return result?.status === 'granted';
    });

    return (
      <div className="wizard-permission-step">
        <div className="step-header">
          <div className="step-icon">{stepInfo.icon}</div>
          <div className="step-info">
            <h2>{stepInfo.title}</h2>
            <p>{stepInfo.description}</p>
            <div className="step-meta">
              <span className={`importance ${stepInfo.importance}`}>
                {stepInfo.importance === 'critical' ? '必需' : 
                 stepInfo.importance === 'important' ? '重要' : '可选'}
              </span>
              <span className="time-estimate">预计 {stepInfo.estimatedTime}</span>
            </div>
          </div>
        </div>

        <div className="step-permissions">
          {stepPermissions.map(permission => {
            if (!permission) return null;
            
            const result = permissionResults.get(permission.id);
            const status = result?.status || 'unknown';
            
            return (
              <div key={permission.id} className={`permission-card ${status}`}>
                <div className="permission-header">
                  <div className="permission-icon">{permission.icon}</div>
                  <div className="permission-info">
                    <h3>{permission.name}</h3>
                    <p>{permission.description}</p>
                  </div>
                  <div className="permission-status">
                    {status === 'granted' && <span className="status-icon granted">✅</span>}
                    {status === 'denied' && <span className="status-icon denied">❌</span>}
                    {status === 'not-determined' && <span className="status-icon pending">⏳</span>}
                  </div>
                </div>
                
                {result?.message && (
                  <div className="permission-message">{result.message}</div>
                )}
              </div>
            );
          })}
        </div>

        {!allGranted && (
          <div className="step-instructions">
            <h3>📋 如何授予权限：</h3>
            <ol>
              <li>点击下方"请求权限"按钮</li>
              <li>在弹出的系统对话框中选择"允许"</li>
              <li>如果没有弹出对话框，请手动到系统设置中开启权限</li>
              <li>修改权限后可能需要重启应用</li>
            </ol>
          </div>
        )}
      </div>
    );
  };

  const renderCompleteStep = () => (
    <div className="wizard-complete">
      <div className="complete-hero">
        <div className="complete-icon">🎉</div>
        <h1>设置完成！</h1>
        <p>Recording King 已准备就绪，开始您的智能录音之旅</p>
      </div>
      
      <div className="complete-summary">
        <h3>✅ 配置摘要</h3>
        <div className="summary-stats">
          <div className="stat-item">
            <span className="stat-number">{completedSteps.size}</span>
            <span className="stat-label">已配置权限</span>
          </div>
          <div className="stat-item">
            <span className="stat-number">{getOverallProgress()}%</span>
            <span className="stat-label">完成度</span>
          </div>
          <div className="stat-item">
            <span className="stat-number">{skippedSteps.size}</span>
            <span className="stat-label">跳过项目</span>
          </div>
        </div>
      </div>
      
      <div className="complete-actions">
        <div className="action-card primary">
          <div className="action-icon">🎤</div>
          <div className="action-text">
            <h3>开始录音</h3>
            <p>按 ⌘⇧R 开始您的第一次录音</p>
          </div>
        </div>
        <div className="action-card">
          <div className="action-icon">⚙️</div>
          <div className="action-text">
            <h3>调整设置</h3>
            <p>根据需要自定义应用配置</p>
          </div>
        </div>
      </div>
    </div>
  );

  const renderStepContent = () => {
    switch (currentStep) {
      case 'welcome': return renderWelcomeStep();
      case 'system-check': return renderSystemCheckStep();
      case 'complete': return renderCompleteStep();
      default: return renderPermissionStep();
    }
  };

  if (!isVisible) return null;

  const currentStepInfo = wizardSteps.find(s => s.id === currentStep);
  const isFirstStep = currentStep === 'welcome';
  const isLastStep = currentStep === 'complete';
  const currentStepIndex = wizardSteps.findIndex(s => s.id === currentStep);

  return (
    <div className="enhanced-wizard-overlay" onClick={handleClose}>
      <div className="enhanced-wizard-dialog" onClick={(e) => e.stopPropagation()}>
        {/* 进度指示器 */}
        {!isFirstStep && (
          <div className="wizard-progress">
            <div className="progress-steps">
              {wizardSteps.slice(1, -1).map((step, index) => {
                const status = getStepStatus(step.id);
                return (
                  <div key={step.id} className={`progress-step ${status}`}>
                    <div className="step-dot">
                      {status === 'completed' ? '✓' : 
                       status === 'skipped' ? '−' : 
                       status === 'current' ? index + 1 : index + 1}
                    </div>
                    <span className="step-title">{step.title}</span>
                  </div>
                );
              })}
            </div>
            <div className="progress-bar">
              <div 
                className="progress-fill"
                style={{ width: `${(currentStepIndex / (wizardSteps.length - 1)) * 100}%` }}
              />
            </div>
          </div>
        )}

        {/* 主要内容区 */}
        <div className="wizard-content">
          {renderStepContent()}
        </div>

        {/* 底部操作栏 */}
        <div className="wizard-footer">
          <div className="footer-left">
            {!isFirstStep && !isLastStep && (
              <button 
                className="wizard-btn secondary"
                onClick={goToPreviousStep}
                disabled={checkingPermissions}
              >
                ← 上一步
              </button>
            )}
          </div>
          
          <div className="footer-center">
            {currentStep === 'system-check' && (
              <div className="checking-status">
                <div className="spinner"></div>
                <span>检查中...</span>
              </div>
            )}
          </div>
          
          <div className="footer-right">
            {isFirstStep && (
              <button 
                className="wizard-btn primary"
                onClick={() => setCurrentStep('system-check')}
              >
                开始设置 →
              </button>
            )}
            
            {!isFirstStep && !isLastStep && currentStep !== 'system-check' && (
              <>
                {currentStepInfo && currentStepInfo.importance !== 'critical' && (
                  <button 
                    className="wizard-btn secondary"
                    onClick={skipCurrentStep}
                    disabled={checkingPermissions}
                  >
                    跳过
                  </button>
                )}
                
                <button 
                  className="wizard-btn primary"
                  onClick={handleRequestPermissions}
                  disabled={checkingPermissions}
                >
                  {checkingPermissions ? '处理中...' : '请求权限'}
                </button>
              </>
            )}
            
            {isLastStep && (
              <button 
                className="wizard-btn primary"
                onClick={handleClose}
              >
                完成设置
              </button>
            )}
          </div>
        </div>

        {/* 关闭按钮 */}
        <button className="wizard-close-btn" onClick={handleClose}>
          ✕
        </button>
      </div>
    </div>
  );
};

export default EnhancedPermissionWizard;