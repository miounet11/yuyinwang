import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { register, unregister } from '@tauri-apps/api/globalShortcut';
import './AIPromptsEnhanced.css';

// 支持的LLM模型
interface LLMModel {
  id: string;
  name: string;
  provider: string;
  icon: string;
  apiKey?: string;
  endpoint?: string;
  maxTokens: number;
  temperature: number;
  capabilities: string[];
}

interface AIPrompt {
  id: string;
  name: string;
  description: string;
  prompt_text: string;
  is_active: boolean;
  shortcut?: string; // 每个提示可以有自己的快捷键
  llmModel?: string; // 使用的LLM模型
  agentChain: AgentConfig[];
  created_at: number;
  updated_at: number;
}

interface AgentConfig {
  id: string;
  type: string;
  name: string;
  icon: string;
  llmModel?: string; // 每个Agent可以使用不同的LLM
  config: {
    temperature?: number;
    maxTokens?: number;
    systemPrompt?: string;
    [key: string]: any;
  };
  order: number;
}

interface AIPromptsEnhancedProps {
  onEnhancedTextReady?: (text: string) => void;
  transcriptionText?: string;
  isRecording?: boolean;
}

const AIPromptsEnhanced: React.FC<AIPromptsEnhancedProps> = ({ 
  onEnhancedTextReady, 
  transcriptionText, 
  isRecording 
}) => {
  // 预定义的LLM模型
  const [availableLLMs] = useState<LLMModel[]>([
    {
      id: 'gpt-4o',
      name: 'GPT-4o',
      provider: 'OpenAI',
      icon: '🌀',
      maxTokens: 4096,
      temperature: 0.7,
      capabilities: ['text', 'code', 'analysis', 'creative']
    },
    {
      id: 'gpt-4o-mini',
      name: 'GPT-4o Mini',
      provider: 'OpenAI',
      icon: '💫',
      maxTokens: 4096,
      temperature: 0.7,
      capabilities: ['text', 'code', 'fast']
    },
    {
      id: 'claude-3-opus',
      name: 'Claude 3 Opus',
      provider: 'Anthropic',
      icon: '🎭',
      maxTokens: 4096,
      temperature: 0.7,
      capabilities: ['text', 'code', 'analysis', 'creative']
    },
    {
      id: 'claude-3-sonnet',
      name: 'Claude 3 Sonnet',
      provider: 'Anthropic',
      icon: '🎵',
      maxTokens: 4096,
      temperature: 0.7,
      capabilities: ['text', 'code', 'balanced']
    },
    {
      id: 'gemini-pro',
      name: 'Gemini Pro',
      provider: 'Google',
      icon: '✨',
      maxTokens: 4096,
      temperature: 0.7,
      capabilities: ['text', 'multimodal', 'analysis']
    },
    {
      id: 'llama-3',
      name: 'Llama 3',
      provider: 'Meta',
      icon: '🦙',
      maxTokens: 4096,
      temperature: 0.7,
      capabilities: ['text', 'code', 'opensource']
    },
    {
      id: 'mistral-large',
      name: 'Mistral Large',
      provider: 'Mistral',
      icon: '🌊',
      maxTokens: 4096,
      temperature: 0.7,
      capabilities: ['text', 'code', 'multilingual']
    },
    {
      id: 'deepseek-chat',
      name: 'DeepSeek Chat',
      provider: 'DeepSeek',
      icon: '🔍',
      maxTokens: 4096,
      temperature: 0.7,
      capabilities: ['text', 'code', 'reasoning']
    }
  ]);

  // 可用的Agent类型（增强版）
  const [availableAgents] = useState([
    { type: 'transcription', name: '转录优化', icon: '🎙️' },
    { type: 'grammar', name: '语法纠正', icon: '✅' },
    { type: 'rewrite', name: '重写润色', icon: '✨' },
    { type: 'translate', name: '智能翻译', icon: '🌐' },
    { type: 'summarize', name: '生成摘要', icon: '📝' },
    { type: 'expand', name: '内容扩展', icon: '📖' },
    { type: 'simplify', name: '简化表达', icon: '🎯' },
    { type: 'tone', name: '语气调整', icon: '🎭' },
    { type: 'format', name: '格式化', icon: '📋' },
    { type: 'extract', name: '信息提取', icon: '🔍' },
    { type: 'sentiment', name: '情感分析', icon: '💭' },
    { type: 'auto-input', name: '自动输入', icon: '⌨️' }
  ]);

  const [prompts, setPrompts] = useState<AIPrompt[]>([]);
  const [activePrompt, setActivePrompt] = useState<AIPrompt | null>(null);
  const [showConfigDialog, setShowConfigDialog] = useState(false);
  const [selectedPrompt, setSelectedPrompt] = useState<AIPrompt | null>(null);
  const [processingState, setProcessingState] = useState({
    isProcessing: false,
    currentStep: '',
    progress: 0
  });
  
  // 快捷键管理
  const [shortcutMode, setShortcutMode] = useState<'global' | 'prompt'>('prompt');
  const [globalShortcut, setGlobalShortcut] = useState('CommandOrControl+Shift+A');
  const [isRecordingShortcut, setIsRecordingShortcut] = useState(false);
  const [recordingPromptId, setRecordingPromptId] = useState<string | null>(null);

  // 加载保存的提示
  useEffect(() => {
    loadPrompts();
    registerGlobalShortcut();
  }, []);

  // 注册全局快捷键
  const registerGlobalShortcut = async () => {
    try {
      await unregister(globalShortcut);
      await register(globalShortcut, () => {
        if (activePrompt) {
          processWithActivePrompt();
        }
      });
    } catch (error) {
      console.error('注册全局快捷键失败:', error);
    }
  };

  // 注册提示专用快捷键
  const registerPromptShortcuts = async () => {
    for (const prompt of prompts) {
      if (prompt.shortcut) {
        try {
          await register(prompt.shortcut, () => {
            setActivePrompt(prompt);
            processWithPrompt(prompt);
          });
        } catch (error) {
          console.error(`注册提示快捷键失败 ${prompt.name}:`, error);
        }
      }
    }
  };

  const loadPrompts = async () => {
    try {
      // 加载默认提示
      const defaultPrompts: AIPrompt[] = [
        {
          id: 'quick-transcribe',
          name: '快速转录',
          description: '优化语音转文字结果',
          prompt_text: '请优化以下转录文本，修正语法错误并添加标点符号',
          is_active: true,
          shortcut: 'CommandOrControl+Shift+Q',
          llmModel: 'gpt-4o-mini',
          agentChain: [
            {
              id: '1',
              type: 'transcription',
              name: '转录优化',
              icon: '🎙️',
              llmModel: 'gpt-4o-mini',
              config: { temperature: 0.3 },
              order: 0
            },
            {
              id: '2',
              type: 'grammar',
              name: '语法纠正',
              icon: '✅',
              config: {},
              order: 1
            }
          ],
          created_at: Date.now(),
          updated_at: Date.now()
        },
        {
          id: 'professional-email',
          name: '专业邮件',
          description: '将语音转换为专业的邮件格式',
          prompt_text: '请将以下内容改写为专业的商务邮件',
          is_active: false,
          shortcut: 'CommandOrControl+Shift+E',
          llmModel: 'gpt-4o',
          agentChain: [
            {
              id: '1',
              type: 'transcription',
              name: '转录优化',
              icon: '🎙️',
              config: {},
              order: 0
            },
            {
              id: '2',
              type: 'rewrite',
              name: '重写润色',
              icon: '✨',
              llmModel: 'gpt-4o',
              config: { 
                systemPrompt: '你是一位专业的商务写作专家',
                temperature: 0.5 
              },
              order: 1
            },
            {
              id: '3',
              type: 'format',
              name: '格式化',
              icon: '📋',
              config: {},
              order: 2
            }
          ],
          created_at: Date.now(),
          updated_at: Date.now()
        },
        {
          id: 'code-comment',
          name: '代码注释',
          description: '将语音转换为代码注释',
          prompt_text: '请将以下描述转换为清晰的代码注释',
          is_active: false,
          shortcut: 'CommandOrControl+Shift+C',
          llmModel: 'deepseek-chat',
          agentChain: [
            {
              id: '1',
              type: 'transcription',
              name: '转录优化',
              icon: '🎙️',
              config: {},
              order: 0
            },
            {
              id: '2',
              type: 'rewrite',
              name: '代码注释格式化',
              icon: '💻',
              llmModel: 'deepseek-chat',
              config: { 
                systemPrompt: '将描述转换为规范的代码注释格式',
                temperature: 0.2 
              },
              order: 1
            }
          ],
          created_at: Date.now(),
          updated_at: Date.now()
        }
      ];

      setPrompts(defaultPrompts);
      setActivePrompt(defaultPrompts[0]);
      
      // 注册所有提示的快捷键
      await registerPromptShortcuts();
    } catch (error) {
      console.error('加载提示失败:', error);
    }
  };

  // 处理转录文本
  const processWithActivePrompt = async () => {
    if (!activePrompt || !transcriptionText) return;
    await processWithPrompt(activePrompt);
  };

  const processWithPrompt = async (prompt: AIPrompt) => {
    if (!transcriptionText) return;

    setProcessingState({
      isProcessing: true,
      currentStep: '开始处理',
      progress: 0
    });

    let processedText = transcriptionText;
    const totalSteps = prompt.agentChain.length;

    try {
      for (let i = 0; i < prompt.agentChain.length; i++) {
        const agent = prompt.agentChain[i];
        
        setProcessingState({
          isProcessing: true,
          currentStep: agent.name,
          progress: ((i + 1) / totalSteps) * 100
        });

        // 获取使用的LLM模型
        const llmModel = agent.llmModel || prompt.llmModel || 'gpt-4o-mini';
        
        // 处理文本
        const response = await invoke<string>('process_with_llm', {
          text: processedText,
          agentType: agent.type,
          llmModel,
          config: agent.config
        });

        processedText = response;
        
        // 添加延迟以显示进度
        await new Promise(resolve => setTimeout(resolve, 300));
      }

      // 通知父组件
      if (onEnhancedTextReady) {
        onEnhancedTextReady(processedText);
      }

      setProcessingState({
        isProcessing: false,
        currentStep: '完成',
        progress: 100
      });

      setTimeout(() => {
        setProcessingState({
          isProcessing: false,
          currentStep: '',
          progress: 0
        });
      }, 2000);

    } catch (error) {
      console.error('处理失败:', error);
      setProcessingState({
        isProcessing: false,
        currentStep: '处理失败',
        progress: 0
      });
    }
  };

  // 录制快捷键
  const startRecordingShortcut = (promptId: string) => {
    setIsRecordingShortcut(true);
    setRecordingPromptId(promptId);
  };

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (!isRecordingShortcut) return;

    event.preventDefault();
    const keys = [];
    
    if (event.metaKey || event.ctrlKey) keys.push('CommandOrControl');
    if (event.shiftKey) keys.push('Shift');
    if (event.altKey) keys.push('Alt');
    
    if (event.key && !['Control', 'Shift', 'Alt', 'Meta'].includes(event.key)) {
      keys.push(event.key.toUpperCase());
    }

    if (keys.length >= 2) {
      const shortcut = keys.join('+');
      
      if (recordingPromptId) {
        // 更新提示的快捷键
        setPrompts(prev => prev.map(p => 
          p.id === recordingPromptId ? { ...p, shortcut } : p
        ));
      } else {
        // 更新全局快捷键
        setGlobalShortcut(shortcut);
      }
      
      setIsRecordingShortcut(false);
      setRecordingPromptId(null);
    }
  }, [isRecordingShortcut, recordingPromptId]);

  useEffect(() => {
    if (isRecordingShortcut) {
      document.addEventListener('keydown', handleKeyDown);
      return () => document.removeEventListener('keydown', handleKeyDown);
    }
  }, [isRecordingShortcut, handleKeyDown]);

  // 创建新提示
  const createPrompt = () => {
    const newPrompt: AIPrompt = {
      id: `prompt-${Date.now()}`,
      name: '新AI提示',
      description: '',
      prompt_text: '',
      is_active: false,
      llmModel: 'gpt-4o-mini',
      agentChain: [],
      created_at: Date.now(),
      updated_at: Date.now()
    };
    
    setPrompts([...prompts, newPrompt]);
    setSelectedPrompt(newPrompt);
  };

  // 保存提示
  const savePrompt = async (prompt: AIPrompt) => {
    try {
      await invoke('save_ai_prompt', { prompt });
      setPrompts(prev => prev.map(p => p.id === prompt.id ? prompt : p));
      
      // 重新注册快捷键
      await registerPromptShortcuts();
    } catch (error) {
      console.error('保存提示失败:', error);
    }
  };

  // 删除提示
  const deletePrompt = async (promptId: string) => {
    if (!window.confirm('确定要删除这个提示吗？')) return;
    
    try {
      await invoke('delete_ai_prompt', { promptId });
      setPrompts(prev => prev.filter(p => p.id !== promptId));
      
      if (selectedPrompt?.id === promptId) {
        setSelectedPrompt(null);
      }
    } catch (error) {
      console.error('删除提示失败:', error);
    }
  };

  // 添加Agent到链
  const addAgentToChain = (agentType: string) => {
    if (!selectedPrompt) return;
    
    const agent = availableAgents.find(a => a.type === agentType);
    if (!agent) return;

    const newAgent: AgentConfig = {
      id: `agent-${Date.now()}`,
      type: agentType,
      name: agent.name,
      icon: agent.icon,
      config: {},
      order: selectedPrompt.agentChain.length
    };

    const updatedPrompt = {
      ...selectedPrompt,
      agentChain: [...selectedPrompt.agentChain, newAgent]
    };

    setSelectedPrompt(updatedPrompt);
  };

  // 从链中移除Agent
  const removeAgentFromChain = (agentId: string) => {
    if (!selectedPrompt) return;

    const updatedPrompt = {
      ...selectedPrompt,
      agentChain: selectedPrompt.agentChain
        .filter(a => a.id !== agentId)
        .map((a, index) => ({ ...a, order: index }))
    };

    setSelectedPrompt(updatedPrompt);
  };

  // 更新Agent的LLM模型
  const updateAgentLLM = (agentId: string, llmModel: string) => {
    if (!selectedPrompt) return;

    const updatedPrompt = {
      ...selectedPrompt,
      agentChain: selectedPrompt.agentChain.map(a => 
        a.id === agentId ? { ...a, llmModel } : a
      )
    };

    setSelectedPrompt(updatedPrompt);
  };

  return (
    <div className="ai-prompts-enhanced">
      {/* 主界面 */}
      <div className="main-section">
        <div className="section-header">
          <h2>🤖 AI 提示管理</h2>
          <button 
            className="config-btn"
            onClick={() => setShowConfigDialog(true)}
          >
            ⚙️ 配置
          </button>
        </div>

        {/* 活动提示卡片 */}
        <div className="active-prompt-card">
          <div className="card-header">
            <span className="label">当前活动提示</span>
            {activePrompt && (
              <span className="shortcut-badge">
                {activePrompt.shortcut || globalShortcut}
              </span>
            )}
          </div>
          <div className="card-content">
            {activePrompt ? (
              <>
                <h3>{activePrompt.name}</h3>
                <p>{activePrompt.description}</p>
                <div className="agent-flow">
                  {activePrompt.agentChain.map((agent, index) => (
                    <React.Fragment key={agent.id}>
                      <div className="flow-item">
                        <span className="icon">{agent.icon}</span>
                        <span className="name">{agent.name}</span>
                        {agent.llmModel && (
                          <span className="llm-badge">
                            {availableLLMs.find(l => l.id === agent.llmModel)?.icon}
                          </span>
                        )}
                      </div>
                      {index < activePrompt.agentChain.length - 1 && (
                        <span className="flow-arrow">→</span>
                      )}
                    </React.Fragment>
                  ))}
                </div>
              </>
            ) : (
              <p className="empty-state">未选择活动提示</p>
            )}
          </div>
        </div>

        {/* 快速切换提示 */}
        <div className="quick-switch">
          <h3>快速切换</h3>
          <div className="prompt-grid">
            {prompts.map(prompt => (
              <button
                key={prompt.id}
                className={`prompt-tile ${activePrompt?.id === prompt.id ? 'active' : ''}`}
                onClick={() => setActivePrompt(prompt)}
              >
                <div className="tile-header">
                  <span className="name">{prompt.name}</span>
                  {prompt.shortcut && (
                    <span className="shortcut">{prompt.shortcut}</span>
                  )}
                </div>
                <div className="tile-footer">
                  <span className="agent-count">
                    {prompt.agentChain.length} 个Agent
                  </span>
                  {prompt.llmModel && (
                    <span className="llm-icon">
                      {availableLLMs.find(l => l.id === prompt.llmModel)?.icon}
                    </span>
                  )}
                </div>
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* 配置对话框 */}
      {showConfigDialog && (
        <div className="config-dialog-overlay" onClick={() => setShowConfigDialog(false)}>
          <div className="config-dialog" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-header">
              <h2>AI 提示配置</h2>
              <button 
                className="close-btn"
                onClick={() => setShowConfigDialog(false)}
              >
                ✕
              </button>
            </div>

            <div className="dialog-content">
              {/* 左侧提示列表 */}
              <div className="prompts-sidebar">
                <div className="sidebar-header">
                  <h3>提示列表</h3>
                  <button 
                    className="add-btn"
                    onClick={createPrompt}
                  >
                    + 新建
                  </button>
                </div>
                <div className="prompts-list">
                  {prompts.map(prompt => (
                    <div
                      key={prompt.id}
                      className={`prompt-item ${selectedPrompt?.id === prompt.id ? 'selected' : ''}`}
                      onClick={() => setSelectedPrompt(prompt)}
                    >
                      <div className="item-content">
                        <span className="name">{prompt.name}</span>
                        {prompt.is_active && <span className="active-badge">活动</span>}
                      </div>
                      <button
                        className="delete-btn"
                        onClick={(e) => {
                          e.stopPropagation();
                          deletePrompt(prompt.id);
                        }}
                      >
                        🗑
                      </button>
                    </div>
                  ))}
                </div>
              </div>

              {/* 右侧配置区 */}
              <div className="config-area">
                {selectedPrompt ? (
                  <>
                    {/* 基本信息 */}
                    <div className="config-section">
                      <h3>基本信息</h3>
                      <div className="form-group">
                        <label>名称</label>
                        <input
                          type="text"
                          value={selectedPrompt.name}
                          onChange={(e) => setSelectedPrompt({
                            ...selectedPrompt,
                            name: e.target.value
                          })}
                        />
                      </div>
                      <div className="form-group">
                        <label>描述</label>
                        <textarea
                          value={selectedPrompt.description}
                          onChange={(e) => setSelectedPrompt({
                            ...selectedPrompt,
                            description: e.target.value
                          })}
                          rows={3}
                        />
                      </div>
                      <div className="form-group">
                        <label>快捷键</label>
                        <div className="shortcut-input">
                          <input
                            type="text"
                            value={selectedPrompt.shortcut || '未设置'}
                            readOnly
                          />
                          <button
                            className={`record-btn ${isRecordingShortcut ? 'recording' : ''}`}
                            onClick={() => startRecordingShortcut(selectedPrompt.id)}
                          >
                            {isRecordingShortcut && recordingPromptId === selectedPrompt.id ? '录制中...' : '录制'}
                          </button>
                        </div>
                      </div>
                      <div className="form-group">
                        <label>默认LLM模型</label>
                        <select
                          value={selectedPrompt.llmModel || ''}
                          onChange={(e) => setSelectedPrompt({
                            ...selectedPrompt,
                            llmModel: e.target.value
                          })}
                        >
                          <option value="">使用全局默认</option>
                          {availableLLMs.map(llm => (
                            <option key={llm.id} value={llm.id}>
                              {llm.icon} {llm.name} ({llm.provider})
                            </option>
                          ))}
                        </select>
                      </div>
                    </div>

                    {/* Agent链配置 */}
                    <div className="config-section">
                      <h3>处理链配置</h3>
                      <div className="agent-chain-editor">
                        {selectedPrompt.agentChain.length === 0 ? (
                          <div className="empty-chain">
                            <p>还没有配置处理链</p>
                            <p className="hint">从下方选择Agent添加到处理链</p>
                          </div>
                        ) : (
                          <div className="chain-list">
                            {selectedPrompt.agentChain.map((agent, index) => (
                              <div key={agent.id} className="chain-item">
                                <div className="item-header">
                                  <span className="order">{index + 1}</span>
                                  <span className="icon">{agent.icon}</span>
                                  <span className="name">{agent.name}</span>
                                  <select
                                    className="llm-select"
                                    value={agent.llmModel || ''}
                                    onChange={(e) => updateAgentLLM(agent.id, e.target.value)}
                                    onClick={(e) => e.stopPropagation()}
                                  >
                                    <option value="">继承默认</option>
                                    {availableLLMs.map(llm => (
                                      <option key={llm.id} value={llm.id}>
                                        {llm.icon} {llm.name}
                                      </option>
                                    ))}
                                  </select>
                                  <button
                                    className="remove-btn"
                                    onClick={() => removeAgentFromChain(agent.id)}
                                  >
                                    ×
                                  </button>
                                </div>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>

                      {/* 可用Agents */}
                      <div className="available-agents">
                        <h4>可用的Agents</h4>
                        <div className="agents-grid">
                          {availableAgents.map(agent => (
                            <button
                              key={agent.type}
                              className="agent-tile"
                              onClick={() => addAgentToChain(agent.type)}
                            >
                              <span className="icon">{agent.icon}</span>
                              <span className="name">{agent.name}</span>
                            </button>
                          ))}
                        </div>
                      </div>
                    </div>

                    {/* 操作按钮 */}
                    <div className="config-actions">
                      <button
                        className="activate-btn"
                        onClick={() => setActivePrompt(selectedPrompt)}
                      >
                        设为活动提示
                      </button>
                      <button
                        className="save-btn"
                        onClick={() => savePrompt(selectedPrompt)}
                      >
                        保存
                      </button>
                    </div>
                  </>
                ) : (
                  <div className="empty-config">
                    <p>选择一个提示进行配置</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 处理状态指示器 */}
      {processingState.isProcessing && (
        <div className="processing-overlay">
          <div className="processing-card">
            <div className="processing-spinner"></div>
            <div className="processing-info">
              <h3>AI处理中</h3>
              <p>{processingState.currentStep}</p>
              <div className="progress-bar">
                <div 
                  className="progress-fill"
                  style={{ width: `${processingState.progress}%` }}
                />
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AIPromptsEnhanced;