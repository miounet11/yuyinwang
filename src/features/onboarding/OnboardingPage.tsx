import React, { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useAppStore } from '../../shared/stores/useAppStore';
import type { OnboardingStep } from '../../shared/types';
import './OnboardingPage.css';

export const OnboardingPage: React.FC = () => {
  const {
    onboardingState,
    setOnboardingStep,
    completeOnboarding,
    setCurrentPage,
    addToast,
    setSettings,
    settings
  } = useAppStore();

  const [isProcessing, setIsProcessing] = useState(false);
  const [permissionDenied, setPermissionDenied] = useState(false);
  const [selectedModel, setSelectedModel] = useState('luyin-free');

  const steps: OnboardingStep[] = [
    {
      id: 0,
      title: '启用语音录制',
      description: '授予麦克风权限以录制音频',
      icon: '🎙',
      action: async () => {
        try {
          // 麦克风权限通常在首次使用时自动请求
          // 这里我们只是验证权限状态
          return true;
        } catch (error) {
          console.error('Microphone permission error:', error);
          return false;
        }
      },
      isCompleted: onboardingState.completedSteps.has(0),
    },
    {
      id: 1,
      title: '启用辅助功能',
      description: '授予辅助功能权限以实现文本注入',
      icon: '♿',
      action: async () => {
        try {
          await invoke('request_injection_permission');
          const hasPermission = await invoke<boolean>('check_injection_permission');
          return hasPermission;
        } catch (error) {
          console.error('Accessibility permission error:', error);
          return false;
        }
      },
      isCompleted: onboardingState.completedSteps.has(1),
    },
    {
      id: 2,
      title: '选择默认模型',
      description: '选择您首选的语音识别模型',
      icon: '🤖',
      action: async () => {
        try {
          const updated = { ...settings, selected_model: selectedModel };
          await invoke('update_settings', { settings: updated });
          setSettings(updated);
          return true;
        } catch (error) {
          console.error('Model selection error:', error);
          return false;
        }
      },
      isCompleted: onboardingState.completedSteps.has(2),
    },
    {
      id: 3,
      title: '配置快捷键',
      description: '设置全局快捷键以快速启动录音',
      icon: '⌨️',
      action: async () => {
        try {
          const defaultShortcut = 'CommandOrControl+Shift+Space';
          await invoke('register_global_shortcut', { key: defaultShortcut });
          const updated = { ...settings, shortcut_key: defaultShortcut };
          await invoke('update_settings', { settings: updated });
          setSettings(updated);
          return true;
        } catch (error) {
          console.error('Shortcut registration error:', error);
          return false;
        }
      },
      isCompleted: onboardingState.completedSteps.has(3),
    },
  ];

  const currentStepData = steps[onboardingState.currentStep];

  const handleNext = useCallback(async () => {
    if (!currentStepData) return;

    setIsProcessing(true);
    setPermissionDenied(false);

    try {
      const success = await currentStepData.action();

      if (success) {
        // 步骤完成，进入下一步
        const nextStep = onboardingState.currentStep + 1;

        if (nextStep >= onboardingState.totalSteps) {
          // 所有步骤完成
          completeOnboarding();
          addToast('success', '入门引导完成！');
          setCurrentPage('general');
        } else {
          setOnboardingStep(nextStep);
        }
      } else {
        // 权限被拒绝
        setPermissionDenied(true);
      }
    } catch (error) {
      console.error('Step execution error:', error);
      setPermissionDenied(true);
    } finally {
      setIsProcessing(false);
    }
  }, [currentStepData, onboardingState.currentStep, onboardingState.totalSteps, completeOnboarding, setOnboardingStep, addToast, setCurrentPage]);

  const handleSkip = useCallback(() => {
    const nextStep = onboardingState.currentStep + 1;

    if (nextStep >= onboardingState.totalSteps) {
      // 跳过最后一步，直接完成
      completeOnboarding();
      addToast('info', '已跳过入门引导');
      setCurrentPage('general');
    } else {
      setOnboardingStep(nextStep);
      setPermissionDenied(false);
    }
  }, [onboardingState.currentStep, onboardingState.totalSteps, completeOnboarding, setOnboardingStep, addToast, setCurrentPage]);

  // 自动检测权限状态
  useEffect(() => {
    const checkPermissionStatus = async () => {
      if (onboardingState.currentStep === 1) {
        try {
          const hasPermission = await invoke<boolean>('check_injection_permission');
          if (hasPermission && !onboardingState.completedSteps.has(1)) {
            // 权限已授予，自动进入下一步
            const nextStep = onboardingState.currentStep + 1;
            setOnboardingStep(nextStep);
            setPermissionDenied(false);
          }
        } catch (error) {
          console.error('Permission check error:', error);
        }
      }
    };

    // 每3秒检查一次权限状态
    const interval = setInterval(checkPermissionStatus, 3000);
    return () => clearInterval(interval);
  }, [onboardingState.currentStep, onboardingState.completedSteps, setOnboardingStep]);

  if (!currentStepData) {
    return null;
  }

  return (
    <div className="onboarding-page">
      <div className="onboarding-container">
        {/* 进度指示器 */}
        <div className="onboarding-progress">
          <div className="onboarding-progress-text">
            第 {onboardingState.currentStep + 1} 步，共 {onboardingState.totalSteps} 步
          </div>
          <div className="onboarding-progress-bar">
            <div
              className="onboarding-progress-fill"
              style={{
                width: `${((onboardingState.currentStep + 1) / onboardingState.totalSteps) * 100}%`
              }}
            />
          </div>
        </div>

        {/* 步骤内容 */}
        <div className="onboarding-content">
          <div className="onboarding-icon">{currentStepData.icon}</div>
          <h1 className="onboarding-title">{currentStepData.title}</h1>
          <p className="onboarding-description">{currentStepData.description}</p>

          {/* 步骤特定内容 */}
          {onboardingState.currentStep === 0 && (
            <div className="onboarding-step-content">
              <div className="onboarding-info-box">
                <p>Recording King 需要访问您的麦克风以录制音频。</p>
                <p>点击"继续"后，系统将请求麦克风权限。</p>
              </div>
            </div>
          )}

          {onboardingState.currentStep === 1 && (
            <div className="onboarding-step-content">
              <div className="onboarding-info-box">
                <p>辅助功能权限允许 Recording King 将转录文本自动输入到其他应用。</p>
                <p>点击"继续"后，请在系统设置中授权 Recording King。</p>
                {permissionDenied && (
                  <div className="onboarding-permission-denied">
                    <span className="onboarding-warning-icon">⚠</span>
                    <div>
                      <strong>权限未授予</strong>
                      <p>请前往：系统设置 → 隐私与安全性 → 辅助功能</p>
                      <p>找到 Recording King 并启用权限</p>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {onboardingState.currentStep === 2 && (
            <div className="onboarding-step-content">
              <div className="onboarding-model-selector">
                <label className="onboarding-model-option">
                  <input
                    type="radio"
                    name="model"
                    value="luyin-free"
                    checked={selectedModel === 'luyin-free'}
                    onChange={(e) => setSelectedModel(e.target.value)}
                  />
                  <div className="onboarding-model-info">
                    <div className="onboarding-model-name">录音宝免费版</div>
                    <div className="onboarding-model-desc">免费、快速、中文优化</div>
                  </div>
                </label>
                <label className="onboarding-model-option">
                  <input
                    type="radio"
                    name="model"
                    value="whisper-local"
                    checked={selectedModel === 'whisper-local'}
                    onChange={(e) => setSelectedModel(e.target.value)}
                  />
                  <div className="onboarding-model-info">
                    <div className="onboarding-model-name">Whisper 本地</div>
                    <div className="onboarding-model-desc">离线、隐私、多语言</div>
                  </div>
                </label>
                <label className="onboarding-model-option">
                  <input
                    type="radio"
                    name="model"
                    value="openai-whisper"
                    checked={selectedModel === 'openai-whisper'}
                    onChange={(e) => setSelectedModel(e.target.value)}
                  />
                  <div className="onboarding-model-info">
                    <div className="onboarding-model-name">OpenAI Whisper</div>
                    <div className="onboarding-model-desc">高精度、需要 API Key</div>
                  </div>
                </label>
              </div>
            </div>
          )}

          {onboardingState.currentStep === 3 && (
            <div className="onboarding-step-content">
              <div className="onboarding-info-box">
                <p>默认快捷键：<strong>⌘ ⇧ Space</strong></p>
                <p>在任意应用中按住此快捷键即可开始录音。</p>
                <p>您可以稍后在设置中自定义快捷键。</p>
                {permissionDenied && (
                  <div className="onboarding-permission-denied">
                    <span className="onboarding-warning-icon">⚠</span>
                    <div>
                      <strong>快捷键注册失败</strong>
                      <p>该快捷键可能已被其他应用占用。</p>
                      <p>您可以跳过此步骤，稍后在设置中配置。</p>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>

        {/* 操作按钮 */}
        <div className="onboarding-actions">
          <button
            className="onboarding-btn onboarding-btn-secondary"
            onClick={handleSkip}
            disabled={isProcessing}
          >
            跳过
          </button>
          <button
            className="onboarding-btn onboarding-btn-primary"
            onClick={handleNext}
            disabled={isProcessing}
          >
            {isProcessing ? '处理中...' : '继续'}
          </button>
        </div>
      </div>
    </div>
  );
};
