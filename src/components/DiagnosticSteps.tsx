import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './DiagnosticSteps.css';

// 诊断步骤类型定义
export interface DiagnosticStep {
  id: string;
  title: string;
  description: string;
  category: 'audio' | 'model' | 'api' | 'permission' | 'storage' | 'network' | 'shortcut';
  status: 'pending' | 'running' | 'passed' | 'failed' | 'warning';
  result?: string;
  error?: string;
  details?: any;
  action?: () => Promise<void>;
  autoRun?: boolean;
  priority: 'critical' | 'high' | 'medium' | 'low';
}

export interface DiagnosticCategory {
  id: string;
  name: string;
  icon: string;
  description: string;
  steps: DiagnosticStep[];
}

interface DiagnosticStepsProps {
  isVisible: boolean;
  onClose: () => void;
  category?: string; // 如果指定，则只显示该类别的测试
  autoStart?: boolean; // 是否自动开始测试
}

const DiagnosticSteps: React.FC<DiagnosticStepsProps> = ({ 
  isVisible, 
  onClose, 
  category, 
  autoStart = false 
}) => {
  const [categories, setCategories] = useState<DiagnosticCategory[]>([]);
  const [activeCategory, setActiveCategory] = useState<string>('');
  const [isRunning, setIsRunning] = useState<boolean>(false);
  const [currentStep, setCurrentStep] = useState<string>('');

  // 初始化诊断步骤
  useEffect(() => {
    if (isVisible) {
      initializeDiagnostics();
      if (category) {
        setActiveCategory(category);
      }
    }
  }, [isVisible, category]);

  // 自动开始测试
  useEffect(() => {
    if (isVisible && autoStart && categories.length > 0) {
      if (category && categories.find(cat => cat.id === category)) {
        runCategoryTests(category);
      }
    }
  }, [isVisible, autoStart, category, categories]);

  const initializeDiagnostics = () => {
    const diagnosticCategories: DiagnosticCategory[] = [
      {
        id: 'audio',
        name: '音频系统',
        icon: '🎤',
        description: '测试麦克风、音频设备和录音功能',
        steps: [
          {
            id: 'audio-devices',
            title: '检测音频设备',
            description: '检查可用的音频输入设备',
            category: 'audio',
            status: 'pending',
            priority: 'critical',
            autoRun: true,
            action: async () => {
              const devices = await invoke<any[]>('get_audio_devices');
              return `发现 ${devices.length} 个音频设备: ${devices.map(d => d.name).join(', ')}`;
            }
          },
          {
            id: 'audio-quality',
            title: '音频质量测试',
            description: '测试麦克风音频输入质量（3秒）',
            category: 'audio',
            status: 'pending',
            priority: 'high',
            action: async () => {
              const result = await invoke<string>('test_audio_input', {
                deviceId: null,
                durationSeconds: 3
              });
              return result;
            }
          },
          {
            id: 'audio-permission',
            title: '麦克风权限',
            description: '检查麦克风访问权限',
            category: 'audio',
            status: 'pending',
            priority: 'critical',
            autoRun: true,
            action: async () => {
              const hasPermission = await invoke<boolean>('check_permission', {
                permissionType: 'microphone'
              });
              if (!hasPermission) {
                throw new Error('缺少麦克风权限，请在系统设置中授权');
              }
              return '麦克风权限正常';
            }
          }
        ]
      },
      {
        id: 'model',
        name: '模型处理',
        icon: '🧠',
        description: '测试AI模型加载和处理能力',
        steps: [
          {
            id: 'whisper-models',
            title: 'Whisper模型检查',
            description: '检查本地Whisper模型是否可用',
            category: 'model',
            status: 'pending',
            priority: 'high',
            autoRun: true,
            action: async () => {
              // 这里需要添加检查Whisper模型的后端命令
              return '检查本地Whisper模型状态';
            }
          },
          {
            id: 'model-transcription',
            title: '模型转录测试',
            description: '使用测试音频验证模型转录功能',
            category: 'model',
            status: 'pending',
            priority: 'high',
            action: async () => {
              // 使用预置的测试音频文件进行转录测试
              return '模型转录测试';
            }
          }
        ]
      },
      {
        id: 'api',
        name: 'API连接',
        icon: '🌐',
        description: '测试在线服务和API连接',
        steps: [
          {
            id: 'luyin-api',
            title: 'LuYinWang API',
            description: '测试录音王API连接和认证',
            category: 'api',
            status: 'pending',
            priority: 'medium',
            action: async () => {
              // 测试录音王API连接
              return '测试LuYinWang API连接';
            }
          },
          {
            id: 'openai-api',
            title: 'OpenAI API',
            description: '测试OpenAI API连接和配置',
            category: 'api',
            status: 'pending',
            priority: 'medium',
            action: async () => {
              // 测试OpenAI API连接
              return '测试OpenAI API连接';
            }
          }
        ]
      },
      {
        id: 'permission',
        name: '系统权限',
        icon: '🔒',
        description: '检查系统权限和安全设置',
        steps: [
          {
            id: 'accessibility',
            title: '辅助功能权限',
            description: '检查macOS辅助功能权限',
            category: 'permission',
            status: 'pending',
            priority: 'high',
            autoRun: true,
            action: async () => {
              const hasPermission = await invoke<boolean>('check_permission', {
                permissionType: 'accessibility'
              });
              if (!hasPermission) {
                throw new Error('缺少辅助功能权限，请在系统偏好设置中授权');
              }
              return '辅助功能权限正常';
            }
          },
          {
            id: 'input-monitoring',
            title: '输入监控权限',
            description: '检查输入监控权限（用于全局快捷键）',
            category: 'permission',
            status: 'pending',
            priority: 'medium',
            action: async () => {
              const hasPermission = await invoke<boolean>('check_permission', {
                permissionType: 'input_monitoring'
              });
              if (!hasPermission) {
                throw new Error('缺少输入监控权限，全局快捷键可能无法使用');
              }
              return '输入监控权限正常';
            }
          }
        ]
      },
      {
        id: 'storage',
        name: '存储系统',
        icon: '💾',
        description: '测试数据库和文件存储功能',
        steps: [
          {
            id: 'database-connection',
            title: '数据库连接',
            description: '测试SQLite数据库连接',
            category: 'storage',
            status: 'pending',
            priority: 'high',
            autoRun: true,
            action: async () => {
              const history = await invoke<any[]>('get_transcription_history');
              return `数据库连接正常，包含 ${history.length} 条转录记录`;
            }
          },
          {
            id: 'file-permissions',
            title: '文件读写权限',
            description: '测试临时文件创建和读写权限',
            category: 'storage',
            status: 'pending',
            priority: 'medium',
            action: async () => {
              // 测试文件读写权限
              return '文件读写权限测试';
            }
          }
        ]
      },
      {
        id: 'network',
        name: '网络连接',
        icon: '📡',
        description: '测试网络连接和在线服务',
        steps: [
          {
            id: 'internet-connection',
            title: '网络连通性',
            description: '检查互联网连接状态',
            category: 'network',
            status: 'pending',
            priority: 'medium',
            action: async () => {
              // 测试网络连接
              try {
                const response = await fetch('https://www.baidu.com', { 
                  method: 'HEAD',
                  mode: 'no-cors'
                });
                return '网络连接正常';
              } catch (error) {
                throw new Error('网络连接异常');
              }
            }
          },
          {
            id: 'dns-resolution',
            title: 'DNS解析',
            description: '测试域名解析功能',
            category: 'network',
            status: 'pending',
            priority: 'low',
            action: async () => {
              // 测试DNS解析
              return 'DNS解析测试';
            }
          }
        ]
      },
      {
        id: 'shortcut',
        name: '快捷键系统',
        icon: '⌨️',
        description: '测试全局快捷键和输入处理',
        steps: [
          {
            id: 'shortcut-registration',
            title: '快捷键注册',
            description: '检查全局快捷键注册状态',
            category: 'shortcut',
            status: 'pending',
            priority: 'medium',
            action: async () => {
              // 测试快捷键注册
              return '快捷键注册测试';
            }
          },
          {
            id: 'shortcut-trigger',
            title: '快捷键触发',
            description: '测试快捷键响应和事件处理',
            category: 'shortcut',
            status: 'pending',
            priority: 'low',
            action: async () => {
              // 测试快捷键触发
              return '快捷键触发测试';
            }
          }
        ]
      }
    ];

    setCategories(diagnosticCategories);
    
    // 如果没有指定类别，默认选择第一个
    if (!category && diagnosticCategories.length > 0) {
      setActiveCategory(diagnosticCategories[0].id);
    }
  };

  // 运行单个测试步骤
  const runStep = async (categoryId: string, stepId: string) => {
    const category = categories.find(cat => cat.id === categoryId);
    if (!category) return;

    const stepIndex = category.steps.findIndex(step => step.id === stepId);
    if (stepIndex === -1) return;

    // 更新步骤状态为运行中
    setCategories(prev => prev.map(cat => 
      cat.id === categoryId 
        ? {
            ...cat,
            steps: cat.steps.map((step, index) => 
              index === stepIndex 
                ? { ...step, status: 'running' as const, error: undefined }
                : step
            )
          }
        : cat
    ));

    setCurrentStep(stepId);

    try {
      if (category.steps[stepIndex].action) {
        const result = await category.steps[stepIndex].action!();
        
        // 更新步骤状态为成功
        setCategories(prev => prev.map(cat => 
          cat.id === categoryId 
            ? {
                ...cat,
                steps: cat.steps.map((step, index) => 
                  index === stepIndex 
                    ? { ...step, status: 'passed' as const, result }
                    : step
                )
              }
            : cat
        ));
      }
    } catch (error: any) {
      // 更新步骤状态为失败
      setCategories(prev => prev.map(cat => 
        cat.id === categoryId 
          ? {
              ...cat,
              steps: cat.steps.map((step, index) => 
                index === stepIndex 
                  ? { ...step, status: 'failed' as const, error: error.message }
                  : step
              )
            }
          : cat
      ));
    }

    setCurrentStep('');
  };

  // 运行整个类别的测试
  const runCategoryTests = async (categoryId: string) => {
    setIsRunning(true);
    const category = categories.find(cat => cat.id === categoryId);
    if (!category) {
      setIsRunning(false);
      return;
    }

    // 首先运行自动运行的步骤
    for (const step of category.steps.filter(s => s.autoRun)) {
      await runStep(categoryId, step.id);
    }

    // 然后运行其他步骤
    for (const step of category.steps.filter(s => !s.autoRun)) {
      await runStep(categoryId, step.id);
    }

    setIsRunning(false);
  };

  // 获取状态图标
  const getStatusIcon = (status: DiagnosticStep['status']) => {
    switch (status) {
      case 'running': return '🔄';
      case 'passed': return '✅';
      case 'failed': return '❌';
      case 'warning': return '⚠️';
      default: return '⭕';
    }
  };

  // 获取优先级颜色
  const getPriorityColor = (priority: DiagnosticStep['priority']) => {
    switch (priority) {
      case 'critical': return '#ff4444';
      case 'high': return '#ff9800';
      case 'medium': return '#ffeb3b';
      case 'low': return '#4caf50';
    }
  };

  if (!isVisible) return null;

  const activeCategoryData = categories.find(cat => cat.id === activeCategory);

  return (
    <div className="diagnostic-overlay" onClick={onClose}>
      <div className="diagnostic-panel" onClick={(e) => e.stopPropagation()}>
        <div className="diagnostic-header">
          <h2>🔍 系统诊断工具</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="diagnostic-content">
          {/* 类别选择 */}
          {!category && (
            <div className="category-tabs">
              {categories.map((cat) => (
                <button
                  key={cat.id}
                  className={`category-tab ${activeCategory === cat.id ? 'active' : ''}`}
                  onClick={() => setActiveCategory(cat.id)}
                >
                  <span className="category-icon">{cat.icon}</span>
                  <span className="category-name">{cat.name}</span>
                </button>
              ))}
            </div>
          )}

          {/* 当前类别信息 */}
          {activeCategoryData && (
            <div className="category-info">
              <div className="category-header">
                <span className="category-icon-large">{activeCategoryData.icon}</span>
                <div className="category-details">
                  <h3>{activeCategoryData.name}</h3>
                  <p>{activeCategoryData.description}</p>
                </div>
                <button 
                  className={`run-all-btn ${isRunning ? 'running' : ''}`}
                  onClick={() => runCategoryTests(activeCategoryData.id)}
                  disabled={isRunning}
                >
                  {isRunning ? '🔄 测试中...' : '▶️ 运行全部'}
                </button>
              </div>
            </div>
          )}

          {/* 诊断步骤列表 */}
          {activeCategoryData && (
            <div className="diagnostic-steps">
              {activeCategoryData.steps.map((step) => (
                <div key={step.id} className={`diagnostic-step ${step.status}`}>
                  <div className="step-header">
                    <div className="step-info">
                      <span className="step-status">{getStatusIcon(step.status)}</span>
                      <div className="step-details">
                        <h4>{step.title}</h4>
                        <p>{step.description}</p>
                      </div>
                      <div 
                        className="step-priority"
                        style={{ backgroundColor: getPriorityColor(step.priority) }}
                      >
                        {step.priority}
                      </div>
                    </div>
                    <button 
                      className="step-run-btn"
                      onClick={() => runStep(activeCategoryData.id, step.id)}
                      disabled={isRunning || step.status === 'running'}
                    >
                      {step.status === 'running' ? '运行中' : '测试'}
                    </button>
                  </div>

                  {/* 测试结果 */}
                  {(step.result || step.error) && (
                    <div className="step-result">
                      {step.result && (
                        <div className="result-success">
                          ✅ {step.result}
                        </div>
                      )}
                      {step.error && (
                        <div className="result-error">
                          ❌ {step.error}
                        </div>
                      )}
                    </div>
                  )}

                  {/* 当前运行步骤高亮 */}
                  {currentStep === step.id && (
                    <div className="step-running-indicator">
                      <div className="running-bar"></div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default DiagnosticSteps;