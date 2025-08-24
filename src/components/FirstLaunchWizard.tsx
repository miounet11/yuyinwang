import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
// import { permissionManager } from '../utils/permissionManager';
import { shortcutTester, TestResult, KeyCombination } from '../utils/shortcutTester';
import './FirstLaunchWizard.css';
import { invoke } from '@tauri-apps/api/tauri';

interface FirstLaunchWizardProps {
  isVisible: boolean;
  onComplete: () => void;
}

// 进度保存接口
interface WizardProgress {
  currentStep: number;
  microphoneEnabled: boolean;
  accessibilityEnabled: boolean;
  selectedShortcut: string;
  shortcutTestResult: string;
  timestamp: number;
}


const FirstLaunchWizard: React.FC<FirstLaunchWizardProps> = ({
  isVisible,
  onComplete
}) => {
  // 基础状态
  const [currentStep, setCurrentStep] = useState(0);
  const [prevStep, setPrevStep] = useState(0);
  const [microphoneEnabled, setMicrophoneEnabled] = useState(false);
  const [accessibilityEnabled, setAccessibilityEnabled] = useState(false);
  const [inputMonitoringEnabled, setInputMonitoringEnabled] = useState(false);
  const [selectedShortcut, setSelectedShortcut] = useState<string>('Fn');
  const [isTestingShortcut, setIsTestingShortcut] = useState(false);
  const [shortcutTestResult, setShortcutTestResult] = useState<string>('');
  const [keyPressCount, setKeyPressCount] = useState(0);
  
  // 交互状态
  const [isTransitioning, setIsTransitioning] = useState(false);
  const [permissionError, setPermissionError] = useState<string>('');
  const [loadingStates, setLoadingStates] = useState<Record<string, boolean>>({});
  const [testHistory, setTestHistory] = useState<TestResult[]>([]);
  const [realTimeKeyInfo, setRealTimeKeyInfo] = useState<KeyCombination | null>(null);
  
  // 可访问性状态
  const [announceText, setAnnounceText] = useState<string>('');
  
  // Refs
  const testTimeoutRef = useRef<number | null>(null);
  const keyPressTimeoutRef = useRef<number | null>(null);
  const transitionTimeoutRef = useRef<number | null>(null);
  const skipToMainRef = useRef<HTMLButtonElement>(null);

  // 步骤配置（增强版）
  const steps = useMemo(() => [
    {
      id: 'microphone',
      title: '启用语音录制',
      subtitle: '允许应用访问您的麦克风',
      description: '需要麦克风权限来进行语音转录',
      icon: '🎤',
      completed: microphoneEnabled,
      required: true,
      estimatedTime: '30秒',
      helpText: '您的语音数据将仅在本地处理，不会上传到服务器'
    },
    {
      id: 'accessibility',
      title: '启用免提文本插入',
      subtitle: '允许应用控制文本输入',
      description: '需要辅助功能权限来直接插入转录文字',
      icon: '🔐',
      completed: accessibilityEnabled,
      required: true,
      estimatedTime: '1分钟',
      helpText: '安全：只能插入文本，不能读取其他应用数据'
    },
    {
      id: 'shortcut',
      title: '选择并测试您的快捷键',
      subtitle: '选择最适合的快捷键方案',
      description: '选择一个便捷的快捷键来快速开始录制',
      icon: '⌨️',
      completed: shortcutTestResult !== '',
      required: true,
      estimatedTime: '2分钟',
      helpText: '建议使用地球键(Fn)以获得最佳体验'
    },
    {
      id: 'complete',
      title: '一切就绪！',
      subtitle: '开始使用 Recording King - 录音王',
      description: '正式开始使用强大的语音转录功能',
      icon: '🎉',
      completed: false,
      required: false,
      estimatedTime: '立即',
      helpText: '所有功能已经准备就绪'
    }
  ], [microphoneEnabled, accessibilityEnabled, shortcutTestResult]);

  // 快捷键选项（集成shortcutTester的选项）
  const shortcutOptions = useMemo(() => {
    const testerOptions = shortcutTester.getRecommendedShortcuts();
    return [
      {
        key: 'Fn',
        name: '地球键 (Fn)',
        description: '单按地球键开始录制',
        instructions: '按下键盘左下角的地球键 🌐',
        category: 'recommended',
        compatibility: 'high'
      },
      {
        key: 'CommandOrControl+Shift+R',
        name: 'Command + Shift + R',
        description: '经典组合键',
        instructions: '同时按下 ⌘ + ⇧ + R',
        category: 'basic',
        compatibility: 'high'
      },
      {
        key: 'CommandOrControl+Shift+Space',
        name: 'Command + Shift + Space',
        description: '快速组合键',
        instructions: '同时按下 ⌘ + ⇧ + 空格',
        category: 'basic',
        compatibility: 'high'
      },
      {
        key: 'F13',
        name: 'F13 键',
        description: '专用功能键',
        instructions: '按下 F13 功能键',
        category: 'advanced',
        compatibility: 'medium'
      },
      // 从 shortcutTester 获取更多选项
      ...testerOptions.slice(1, 3).map(option => ({
        key: option.key,
        name: option.label,
        description: option.description,
        instructions: shortcutTester.formatShortcutDisplay(option.key),
        category: option.category as 'basic' | 'advanced' | 'recommended',
        compatibility: 'medium' as 'high' | 'medium' | 'low'
      }))
    ];
  }, []);

  // 加载保存的进度
  const loadSavedProgress = useCallback(() => {
    try {
      const saved = localStorage.getItem('spokenly_wizard_progress');
      if (saved) {
        const progress: WizardProgress = JSON.parse(saved);
        // 检查时间戳是否过期（24小时）
        if (Date.now() - progress.timestamp < 24 * 60 * 60 * 1000) {
          setCurrentStep(progress.currentStep);
          setMicrophoneEnabled(progress.microphoneEnabled);
          setAccessibilityEnabled(progress.accessibilityEnabled);
          setSelectedShortcut(progress.selectedShortcut);
          setShortcutTestResult(progress.shortcutTestResult);
        }
      }
    } catch (error) {
      console.error('加载进度失败:', error);
    }
  }, []);

  // 键盘事件处理函数
  const handleEscapeKey = useCallback(() => {
    // ESC键关闭向导或退出当前步骤
    if (isTestingShortcut) {
      stopShortcutTest();
    } else {
      // 可以考虑显示确认对话框
      console.log('ESC键被按下');
    }
  }, [isTestingShortcut]);

  const handleTabNavigation = useCallback((e: KeyboardEvent) => {
    // Tab键导航处理
    const focusableElements = document.querySelectorAll(
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled])'
    );
    if (focusableElements.length > 0) {
      const currentIndex = Array.from(focusableElements).indexOf(document.activeElement as Element);
      const nextIndex = e.shiftKey 
        ? (currentIndex - 1 + focusableElements.length) % focusableElements.length
        : (currentIndex + 1) % focusableElements.length;
      (focusableElements[nextIndex] as HTMLElement).focus();
    }
  }, []);

  const handleEnterKey = useCallback(() => {
    // Enter键确认当前操作
    const activeElement = document.activeElement as HTMLElement;
    if (activeElement && activeElement.click) {
      activeElement.click();
    } else {
      // 默认行为：前进到下一步
      if (currentStep < steps.length - 1) {
        const nextStep = currentStep + 1;
        animatedStepTransition(nextStep);
      }
    }
  }, [currentStep, steps.length]);

  // 动画步骤转换
  const animatedStepTransition = useCallback(async (targetStep: number) => {
    if (isTransitioning) return;
    
    setIsTransitioning(true);
    setPrevStep(currentStep);
    
    // 播放转换动画
    await new Promise(resolve => {
      if (transitionTimeoutRef.current) {
        clearTimeout(transitionTimeoutRef.current);
      }
      transitionTimeoutRef.current = window.setTimeout(resolve, 300);
    });
    
    setCurrentStep(targetStep);
    setIsTransitioning(false);
    
    // 设置焦点到新步骤
    setTimeout(() => {
      const firstButton = document.querySelector('.step-actions button:not([disabled])') as HTMLElement;
      if (firstButton) {
        firstButton.focus();
      }
    }, 100);
  }, [currentStep, isTransitioning]);

  const handleNextStep = useCallback(() => {
    if (currentStep < steps.length - 1) {
      const nextStep = currentStep + 1;
      animatedStepTransition(nextStep);
    }
  }, [currentStep, steps.length, animatedStepTransition]);

  // 初始化和清理
  useEffect(() => {
    if (isVisible) {
      initializeWizard();
      setupShortcutTester();
      loadSavedProgress();
    }
    return () => {
      cleanupWizard();
    };
  }, [isVisible]);

  // 监听键盘事件用于导航
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isVisible) return;
      
      switch (e.key) {
        case 'Escape':
          e.preventDefault();
          handleEscapeKey();
          break;
        case 'Tab':
          handleTabNavigation(e);
          break;
        case 'Enter':
        case ' ':
          e.preventDefault();
          handleEnterKey();
          break;
        case 'ArrowLeft':
          e.preventDefault();
          handlePrevStep();
          break;
        case 'ArrowRight':
          e.preventDefault();
          handleNextStep();
          break;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isVisible, currentStep, isTestingShortcut]);

  // 保存进度函数
  const saveProgress = () => {
    const progress = {
      currentStep,
      microphoneEnabled,
      accessibilityEnabled,
      selectedShortcut,
      shortcutTestResult,
      timestamp: Date.now()
    };
    localStorage.setItem('spokenly_wizard_progress', JSON.stringify(progress));
  };

  // 自动保存进度
  useEffect(() => {
    if (isVisible) {
      saveProgress();
    }
  }, [currentStep, microphoneEnabled, accessibilityEnabled, selectedShortcut, shortcutTestResult]);

  // 实时公告文本更新
  useEffect(() => {
    if (announceText) {
      const timer = setTimeout(() => setAnnounceText(''), 3000);
      return () => clearTimeout(timer);
    }
  }, [announceText]);

  // 初始化向导
  const initializeWizard = useCallback(async () => {
    setLoadingStates(prev => ({ ...prev, initialization: true }));
    try {
      await checkInitialPermissions();
      // 设置初始焦点
      setTimeout(() => {
        if (skipToMainRef.current) {
          skipToMainRef.current.focus();
        }
      }, 100);
    } catch (error) {
      console.error('初始化向导失败:', error);
      setPermissionError('初始化失败，请刷新页面重试');
    } finally {
      setLoadingStates(prev => ({ ...prev, initialization: false }));
    }
  }, []);

  // 一键快速配置
  const handleQuickSetup = useCallback(async () => {
    setLoadingStates(prev => ({ ...prev, quickSetup: true }));
    setPermissionError('');
    try {
      // 1) 检查并请求关键权限
      try { await invoke('check_all_permissions'); } catch {}
      try { await invoke('request_permission', { permission: 'microphone' }); } catch {}
      try { await invoke('request_permission', { permission: 'accessibility' }); } catch {}
      setMicrophoneEnabled(true);
      setAccessibilityEnabled(true);

      // 2) 保存默认快捷键配置（不注册全局快捷键，避免与 rdev 监听重复）
      const defaultConfig = {
        enabled: false,
        shortcut: 'Fn',
        auto_insert: true,
        use_floating_window: true,
        preferred_model: 'luyingwang-online',
        trigger_mode: 'hold',
        hold_duration: 300,
        realtime_injection: true,
        hold_release_delay_ms: 150
      };
      try { await invoke('configure_voice_shortcuts', { config: defaultConfig }); } catch {}

      // 3) 快速自检：启动渐进式语音输入 2 秒后停止
      try {
        await invoke('show_floating_input');
        await invoke('start_progressive_voice_input', { targetBundleId: null, enableRealTimeInjection: true });
        await new Promise(r => setTimeout(r, 1500));
        await invoke('stop_voice_recording');
      } catch {}

      // 4) 标记步骤完成并跳转完成页
      setSelectedShortcut('Fn');
      setShortcutTestResult('测试成功！快捷键响应正常');
      await animatedStepTransition(3);
    } catch (e) {
      setPermissionError('快速配置失败，请手动按步骤完成');
    } finally {
      setLoadingStates(prev => ({ ...prev, quickSetup: false }));
    }
  }, [animatedStepTransition]);

  const checkInitialPermissions = async () => {
    try {
      // 检查麦克风权限
      try {
        const micGranted = await invoke<boolean>('check_permission', { permission_type: 'microphone' });
        setMicrophoneEnabled(!!micGranted);
        if (micGranted) setAnnounceText('麦克风权限已获得');
      } catch (e) {
        // 回退到浏览器检查
        try {
          const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
          if (stream) {
            setMicrophoneEnabled(true);
            setAnnounceText('麦克风权限已获得（浏览器模式）');
            stream.getTracks().forEach(track => track.stop());
          }
        } catch {
          setPermissionError('无法获取麦克风权限，请检查系统设置');
        }
      }
      
      // 检查辅助功能权限（macOS）
      try {
        const accessGranted = await invoke<boolean>('check_permission', { permission_type: 'accessibility' });
        setAccessibilityEnabled(!!accessGranted);
        if (accessGranted) setAnnounceText('辅助功能权限已获得');
      } catch (e) {
        // 非 macOS 或检查失败，保持现状
      }
      
      // 检查输入监控权限（macOS）
      try {
        const listenGranted = await invoke<boolean>('check_permission', { permission_type: 'input-monitoring' });
        setInputMonitoringEnabled(!!listenGranted);
      } catch (e) {}
      
    } catch (error) {
      console.error('检查权限失败:', error);
    }
  };

  // 设置快捷键测试器
  const setupShortcutTester = useCallback(() => {
    shortcutTester.onKeyPress((keys: KeyCombination) => {
      setRealTimeKeyInfo(keys);
      if (isTestingShortcut) {
        setKeyPressCount(prev => prev + 1);
        setAnnounceText(`检测到按键: ${shortcutTester.formatShortcut(keys)}`);
      }
    });

    shortcutTester.onTestComplete((result: TestResult) => {
      setTestHistory(prev => [...prev, result]);
      if (result.success) {
        setShortcutTestResult(`测试成功！快捷键响应正常`);
        setAnnounceText(`快捷键测试成功`);
        setIsTestingShortcut(false);
      }
    });
  }, [isTestingShortcut]);

  // 清理向导
  const cleanupWizard = useCallback(() => {
    if (testTimeoutRef.current) clearTimeout(testTimeoutRef.current);
    if (keyPressTimeoutRef.current) clearTimeout(keyPressTimeoutRef.current);
    if (transitionTimeoutRef.current) clearTimeout(transitionTimeoutRef.current);
    shortcutTester.stopTest();
    setRealTimeKeyInfo(null);
  }, []);

  // 改进的权限请求方法
  const requestMicrophonePermission = useCallback(async () => {
    setLoadingStates(prev => ({ ...prev, microphone: true }));
    setPermissionError('');
    
    try {
      const success = await invoke<boolean>('request_permission', { permission_type: 'microphone' });
      if (success) {
        setMicrophoneEnabled(true);
        setAnnounceText('麦克风权限获取成功');
        await animatedStepTransition(1);
      } else {
        // 回退到浏览器API
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        if (stream) {
          setMicrophoneEnabled(true);
          setAnnounceText('麦克风权限获取成功（浏览器模式）');
          stream.getTracks().forEach(track => track.stop());
          await animatedStepTransition(1);
        }
      }
    } catch (error) {
      console.error('请求麦克风权限失败:', error);
      setPermissionError('无法获取麦克风权限。请在系统设置中允许访问麦克风。');
      setAnnounceText('麦克风权限获取失败');
    } finally {
      setLoadingStates(prev => ({ ...prev, microphone: false }));
    }
  }, [animatedStepTransition]);

  const requestAccessibilityPermission = useCallback(async () => {
    setLoadingStates(prev => ({ ...prev, accessibility: true }));
    setPermissionError('');
    
    try {
      const success = await invoke<boolean>('request_permission', { permission_type: 'accessibility' });
      if (success) {
        setAccessibilityEnabled(true);
        setAnnounceText('辅助功能权限获取成功');
        await animatedStepTransition(2);
      } else {
        setAccessibilityEnabled(false);
        setPermissionError('辅助功能权限未获得，请在系统偏好设置中启用。');
      }
    } catch (error) {
      console.error('请求辅助功能权限失败:', error);
      setAccessibilityEnabled(false);
      setPermissionError('无法获取辅助功能权限。请在系统偏好设置中手动启用。');
      setAnnounceText('辅助功能权限获取失败');
    } finally {
      setLoadingStates(prev => ({ ...prev, accessibility: false }));
    }
  }, [animatedStepTransition]);

  const requestInputMonitoringPermission = useCallback(async () => {
    setLoadingStates(prev => ({ ...prev, inputMonitoring: true }));
    setPermissionError('');
    try {
      const success = await invoke<boolean>('request_permission', { permission_type: 'input-monitoring' });
      if (success) {
        setInputMonitoringEnabled(true);
        setAnnounceText('输入监控权限获取成功');
      } else {
        setInputMonitoringEnabled(false);
        setPermissionError('输入监控权限未获得，请在系统偏好设置中启用。');
      }
    } catch (e) {
      setInputMonitoringEnabled(false);
      setPermissionError('无法获取输入监控权限，请在系统偏好设置中手动启用。');
    } finally {
      setLoadingStates(prev => ({ ...prev, inputMonitoring: false }));
    }
  }, []);

  const handleShortcutTest = () => {
    setIsTestingShortcut(true);
    setShortcutTestResult('');
    setKeyPressCount(0);
    
    // 清除之前的定时器
    if (testTimeoutRef.current) {
      clearTimeout(testTimeoutRef.current);
    }
    if (keyPressTimeoutRef.current) {
      clearTimeout(keyPressTimeoutRef.current);
    }
    
    // 监听键盘事件
    const handleKeyDown = (e: KeyboardEvent) => {
      console.log('按键检测:', {
        key: e.key,
        code: e.code,
        ctrlKey: e.ctrlKey,
        shiftKey: e.shiftKey,
        metaKey: e.metaKey,
        altKey: e.altKey,
        selectedShortcut
      });

      let isCorrectKey = false;
      
      if (selectedShortcut === 'Fn') {
        // 检测地球键 - 在Web环境中很难准确检测
        isCorrectKey = e.key === 'Fn' || e.code === 'Fn' || e.key === 'Dead' || e.code === 'Unidentified';
      } else if (selectedShortcut === 'F13') {
        isCorrectKey = e.key === 'F13' || e.code === 'F13';
      } else if (selectedShortcut === 'Command + Shift + R') {
        // 检测 Cmd+Shift+R
        isCorrectKey = e.metaKey && e.shiftKey && (e.key === 'R' || e.key === 'r') && !e.ctrlKey && !e.altKey;
      } else if (selectedShortcut === 'Command + Shift + Space') {
        // 检测 Cmd+Shift+Space
        isCorrectKey = e.metaKey && e.shiftKey && (e.key === ' ' || e.code === 'Space') && !e.ctrlKey && !e.altKey;
      }
      
      if (isCorrectKey) {
        e.preventDefault();
        setKeyPressCount(prev => prev + 1);
        
        // 清除之前的超时
        if (keyPressTimeoutRef.current) {
          clearTimeout(keyPressTimeoutRef.current);
        }
        
        // 设置新的超时
        keyPressTimeoutRef.current = window.setTimeout(() => {
          setShortcutTestResult('测试成功！快捷键响应正常');
          setIsTestingShortcut(false);
          document.removeEventListener('keydown', handleKeyDown);
        }, 500);
      }
    };
    
    document.addEventListener('keydown', handleKeyDown);
    
    // 10秒后超时
    testTimeoutRef.current = window.setTimeout(() => {
      setShortcutTestResult('测试超时，请重试或选择其他快捷键');
      setIsTestingShortcut(false);
      document.removeEventListener('keydown', handleKeyDown);
    }, 10000);
  };

  const stopShortcutTest = () => {
    setIsTestingShortcut(false);
    if (testTimeoutRef.current) {
      clearTimeout(testTimeoutRef.current);
    }
    if (keyPressTimeoutRef.current) {
      clearTimeout(keyPressTimeoutRef.current);
    }
  };

  // 辅助函数定义完成

  const handlePrevStep = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleComplete = () => {
    localStorage.setItem('spokenly_setup_completed', 'true');
    // 保存选择的快捷键
    localStorage.setItem('spokenly_preferred_shortcut', selectedShortcut);
    onComplete();
  };

  const renderStepContent = () => {
    switch (currentStep) {
      case 0:
        return (
          <div className="wizard-step microphone-step">
            <div className="step-content">
              <div className="step-icon-large">🎤</div>
              <h2 className="step-title">启用语音录制</h2>
              <p className="step-description">
                Recording King 需要访问您的麦克风来进行语音转录。
                您的语音数据将仅在本地处理，不会上传到服务器。
              </p>
              
              {microphoneEnabled ? (
                <div className="success-indicator">
                  <div className="success-icon">✅</div>
                  <div className="success-text">麦克风访问已启用！</div>
                </div>
              ) : (
                <div className="permission-required">
                  <div className="warning-icon">⚠️</div>
                  <div className="warning-text">需要麦克风权限</div>
                </div>
              )}

              <div className="quick-setup">
                <button 
                  className="enable-btn primary"
                  onClick={handleQuickSetup}
                  disabled={!!loadingStates.quickSetup}
                >
                  {loadingStates.quickSetup ? '正在一键配置…' : '一键快速配置'}
                </button>
                <small className="helper-text">将自动完成权限、默认快捷键与一次快速自检</small>
              </div>
              
            </div>
          </div>
        );

      case 1:
        return (
          <div className="wizard-step accessibility-step">
            <div className="step-content">
              <div className="step-icon-large">🔐</div>
              <h2 className="step-title">启用免提文本插入</h2>
              <p className="step-description">
                为了将转录的文字直接插入到任何应用程序中，
                Recording King 需要辅助功能权限与输入监控权限（macOS）。
              </p>
              
              {accessibilityEnabled ? (
                <div className="success-indicator">
                  <div className="success-icon">✅</div>
                  <div className="success-text">已启用辅助功能访问！</div>
                </div>
              ) : (
                <div className="permission-required">
                  <div className="warning-icon">⚠️</div>
                  <div className="warning-text">需要辅助功能权限</div>
                </div>
              )}

              <div className="permission-actions" style={{ display: 'flex', gap: 12, marginTop: 12 }}>
                <button className="enable-btn primary" onClick={requestAccessibilityPermission} disabled={!!loadingStates.accessibility}>
                  {loadingStates.accessibility ? '请求中…' : '启用辅助功能'}
                </button>
                <button className="enable-btn" onClick={requestInputMonitoringPermission} disabled={!!loadingStates.inputMonitoring}>
                  {loadingStates.inputMonitoring ? '请求中…' : inputMonitoringEnabled ? '已启用输入监控' : '启用输入监控'}
                </button>
              </div>
              
              <div className="step-info">
                <div className="info-item">
                  <span className="info-icon">🔒</span>
                  <span>安全：只能插入文本，不能读取其他应用数据</span>
                </div>
                <div className="info-item">
                  <span className="info-icon">⚡</span>
                  <span>高效：无需复制粘贴，直接插入转录文本</span>
                </div>
              </div>
              
            </div>
          </div>
        );

      case 2:
        return (
          <div className="wizard-step shortcut-step">
            <div className="step-content">
              <div className="step-icon-large">⌨️</div>
              <h2 className="step-title">选择并测试您的快捷键</h2>
              <p className="step-description">
                选择一个快捷键来快速开始录制。建议使用地球键(Fn)以获得最佳体验。
              </p>
              
              <div className="shortcut-options">
                {shortcutOptions.map((option) => (
                  <div 
                    key={option.key}
                    className={`shortcut-option ${selectedShortcut === option.key ? 'selected' : ''}`}
                    onClick={() => setSelectedShortcut(option.key)}
                  >
                    <div className="option-header">
                      <div className="option-radio">
                        {selectedShortcut === option.key && <div className="radio-dot"></div>}
                      </div>
                      <div className="option-info">
                        <div className="option-name">{option.name}</div>
                        <div className="option-description">{option.description}</div>
                      </div>
                    </div>
                    <div className="option-instructions">{option.instructions}</div>
                  </div>
                ))}
              </div>
              
              <div className="shortcut-test">
                <h3>测试您的快捷键</h3>
                <div className={`test-area ${isTestingShortcut ? 'testing' : ''}`}>
                  {isTestingShortcut ? (
                    <div className="test-active">
                      <div className="test-spinner">⚡</div>
                      <div className="test-message">正在监听快捷键...</div>
                      <div className="test-instructions">
                        {shortcutOptions.find(opt => opt.key === selectedShortcut)?.instructions}
                      </div>
                      {keyPressCount > 0 && (
                        <div className="key-press-indicator">
                          已检测到 {keyPressCount} 次按键
                        </div>
                      )}
                      <button className="stop-test-btn" onClick={stopShortcutTest}>
                        停止测试
                      </button>
                    </div>
                  ) : (
                    <div className="test-ready">
                      {shortcutTestResult ? (
                        <div className="test-result success">
                          <div className="result-icon">✅</div>
                          <div className="result-text">{shortcutTestResult}</div>
                        </div>
                      ) : (
                        <div className="test-placeholder">
                          点击下方按钮开始测试您选择的快捷键
                        </div>
                      )}
                      <button className="test-btn" onClick={handleShortcutTest}>
                        测试快捷键
                      </button>
                    </div>
                  )}
                </div>
              </div>
              
            </div>
          </div>
        );

      case 3:
        return (
          <div className="wizard-step complete-step">
            <div className="step-content">
              <div className="step-icon-large celebration">🎉</div>
              <h2 className="step-title">一切就绪！</h2>
              <p className="step-description">
                恭喜！Recording King (录音王) 已完成设置，您现在可以享受强大的语音转录功能。
              </p>
              
              <div className="welcome-message">
                <p className="tagline">如果你觉得我好用，那么你就叫我-录音王吧！👑</p>
                <p className="company">由 <strong>miaoda</strong>（AI 科技公司）技术支持 · 官网：<a href="https://miaoda.xin" target="_blank" rel="noreferrer">miaoda.xin</a></p>
              </div>
              
              <div className="feature-showcase">
                <div className="feature-item">
                  <div className="feature-icon">⚡</div>
                  <div className="feature-content">
                    <h4>快速录制</h4>
                    <p>按 {shortcutOptions.find(opt => opt.key === selectedShortcut)?.name} 即可开始录制</p>
                  </div>
                </div>
                <div className="feature-item">
                  <div className="feature-icon">🔄</div>
                  <div className="feature-content">
                    <h4>实时转录</h4>
                    <p>语音实时转换为文字，支持多种语言</p>
                  </div>
                </div>
                <div className="feature-item">
                  <div className="feature-icon">📝</div>
                  <div className="feature-content">
                    <h4>智能插入</h4>
                    <p>转录文字自动插入到任何应用程序中</p>
                  </div>
                </div>
                <div className="feature-item">
                  <div className="feature-icon">🤖</div>
                  <div className="feature-content">
                    <h4>AI 增强</h4>
                    <p>内置AI助手，优化和编辑您的文本</p>
                  </div>
                </div>
              </div>
              
              <div className="tips-section">
                <h3>使用小贴士</h3>
                <div className="tip-item">
                  <span className="tip-icon">💡</span>
                  <span>在任何应用中使用快捷键，转录内容会自动插入光标位置</span>
                </div>
                <div className="tip-item">
                  <span className="tip-icon">⚙️</span>
                  <span>可以随时在设置中更改快捷键和其他配置</span>
                </div>
                <div className="tip-item">
                  <span className="tip-icon">🎯</span>
                  <span>支持多种 AI 模型，可根据需要选择最适合的模型</span>
                </div>
              </div>
              
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  if (!isVisible) return null;

  const renderStepActions = () => {
    switch (currentStep) {
      case 0:
        return microphoneEnabled ? (
          <div className="actions-row">
            <button className="next-btn" onClick={() => setCurrentStep(1)}>按步骤配置</button>
            <button 
              className="next-btn primary" 
              onClick={handleQuickSetup}
              disabled={!!loadingStates.quickSetup}
            >
              {loadingStates.quickSetup ? '配置中…' : '一键快速配置'}
            </button>
          </div>
        ) : (
          <div className="actions-row">
            <button className="enable-btn" onClick={requestMicrophonePermission}>启用麦克风</button>
            <button 
              className="next-btn primary" 
              onClick={handleQuickSetup}
              disabled={!!loadingStates.quickSetup}
            >
              {loadingStates.quickSetup ? '配置中…' : '一键快速配置'}
            </button>
          </div>
        );
      case 1:
        return (
          <>
            <button className="back-btn" onClick={handlePrevStep}>
              返回
            </button>
            {accessibilityEnabled ? (
              <button className="next-btn primary" onClick={() => setCurrentStep(2)}>
                下一步
              </button>
            ) : (
              <button className="enable-btn primary" onClick={requestAccessibilityPermission}>
                启用辅助功能
              </button>
            )}
          </>
        );
      case 2:
        return (
          <>
            <button className="back-btn" onClick={handlePrevStep}>
              返回
            </button>
            <button 
              className="next-btn primary" 
              onClick={() => setCurrentStep(3)}
              disabled={!shortcutTestResult}
            >
              下一步
            </button>
          </>
        );
      case 3:
        return (
          <>
            <button className="back-btn" onClick={handlePrevStep}>
              返回
            </button>
            <button className="complete-btn primary" onClick={handleComplete}>
              完成设置
            </button>
          </>
        );
      default:
        return null;
    }
  };

  return (
    <div className="wizard-overlay">
      <div className="wizard-dialog">
        {/* 步骤进度指示器 */}
        <div className="wizard-header">
          <div className="progress-dots">
            {steps.map((step, index) => (
              <div 
                key={index} 
                className={`progress-dot ${
                  index < currentStep ? 'completed' : 
                  index === currentStep ? 'current' : 
                  'pending'
                } ${step.completed ? 'success' : ''}`}
              >
                <div className="dot-inner">
                  {step.completed && index < currentStep ? '✓' : index + 1}
                </div>
              </div>
            ))}
            <div className="progress-line">
              <div 
                className="progress-fill"
                style={{ width: `${(currentStep / (steps.length - 1)) * 100}%` }}
              ></div>
            </div>
          </div>
          
          <div className="step-info">
            <h1 className="step-title">{steps[currentStep].title}</h1>
            <p className="step-subtitle">{steps[currentStep].subtitle}</p>
          </div>

          <div className="header-actions">
            <button 
              ref={skipToMainRef}
              className="link-btn"
              onClick={onComplete}
            >
              稍后再说
            </button>
          </div>
        </div>

        {/* 步骤内容 */}
        <div className="wizard-content">
          {renderStepContent()}
        </div>

        {/* 固定的底部按钮区域 */}
        <footer className="step-actions">
          <div className="actions-container">
            {renderStepActions()}
          </div>
          <div className="keyboard-shortcuts-help">
            <small className="help-text">
              ←/→: 切换步骤 • Tab: 导航 • Enter: 确认 • Esc: 取消
            </small>
          </div>
        </footer>
      </div>
    </div>
  );
};

export default FirstLaunchWizard;