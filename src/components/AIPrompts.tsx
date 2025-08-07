import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './AIPrompts.css';

interface AIPrompt {
  id: string;
  name: string;
  description: string;
  agent_type: string;
  prompt_text: string;
  is_active: boolean;
  created_at: number;
  updated_at: number;
  agentChain: AgentConfig[];
}

interface AgentConfig {
  id: string;
  type: string;
  name: string;
  config: Record<string, any>;
  order: number;
}

interface AIPromptsProps {
  onEnhancedTextReady?: (text: string) => void;
  transcriptionText?: string;
  isRecording?: boolean;
}

interface AgentRequest {
  agent_type: string;
  input_text: string;
  prompt_id?: string;
  additional_context?: Record<string, string>;
}

interface AgentResponse {
  success: boolean;
  output_text: string;
  agent_type: string;
  processing_time_ms: number;
  error?: string;
}

interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message: string;
  duration?: number;
}

interface ProcessingState {
  isProcessing: boolean;
  currentAgent?: string;
  progress: number;
  totalSteps: number;
}

const AIPrompts: React.FC<AIPromptsProps> = ({ onEnhancedTextReady, transcriptionText, isRecording }) => {
  const [prompts, setPrompts] = useState<AIPrompt[]>([]);
  const [activePrompt, setActivePrompt] = useState<AIPrompt | null>(null);
  const [showPromptEditor, setShowPromptEditor] = useState(false);
  const [processingState, setProcessingState] = useState<ProcessingState>({
    isProcessing: false,
    progress: 0,
    totalSteps: 0
  });
  const [workflowStep, setWorkflowStep] = useState<'idle' | 'listening' | 'processing' | 'typing'>('idle');
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [showKeyboardHints, setShowKeyboardHints] = useState(true);
  const [draggedItemId, setDraggedItemId] = useState<string | null>(null);
  const [selectedPromptId, setSelectedPromptId] = useState<string>('');
  const dialogRef = useRef<HTMLDivElement>(null);
  const notificationTimeoutRef = useRef<{ [key: string]: number }>({});
  
  // 新建提示的初始状态
  const [newPrompt, setNewPrompt] = useState<AIPrompt>({
    id: '',
    name: '新提示',
    description: '',
    agent_type: 'text-enhancer',
    prompt_text: '',
    is_active: false,
    created_at: Date.now(),
    updated_at: Date.now(),
    agentChain: []
  });

  // 可用的Agent类型
  const availableAgents = [
    { type: 'speech-to-text', name: '语音转文字', icon: '🎙️' },
    { type: 'text-enhancer', name: '文本增强', icon: '✨' },
    { type: 'translator', name: '翻译', icon: '🌐' },
    { type: 'summarizer', name: '摘要生成', icon: '📝' },
    { type: 'formatter', name: '格式化', icon: '📋' },
    { type: 'grammer-check', name: '语法检查', icon: '✅' },
    { type: 'tone-adjuster', name: '语气调整', icon: '🎭' },
    { type: 'auto-input', name: '自动输入', icon: '⌨️' }
  ];

  // 加载保存的提示
  useEffect(() => {
    loadPrompts();
  }, []);

  // 键盘快捷键支持
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // ESC键关闭弹窗
      if (event.key === 'Escape' && showPromptEditor) {
        setShowPromptEditor(false);
        return;
      }

      // Cmd/Ctrl + E 打开编辑器
      if ((event.metaKey || event.ctrlKey) && event.key === 'e') {
        event.preventDefault();
        setShowPromptEditor(true);
        return;
      }

      // Cmd/Ctrl + S 保存提示 (在编辑器中)
      if ((event.metaKey || event.ctrlKey) && event.key === 's' && showPromptEditor) {
        event.preventDefault();
        if (newPrompt.name && newPrompt.agentChain && newPrompt.agentChain.length > 0) {
          savePrompt(newPrompt);
        }
        return;
      }

      // F1 切换键盘提示
      if (event.key === 'F1') {
        event.preventDefault();
        setShowKeyboardHints(prev => !prev);
        return;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [showPromptEditor, newPrompt]);

  // 点击外部关闭弹窗
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dialogRef.current && !dialogRef.current.contains(event.target as Node)) {
        setShowPromptEditor(false);
      }
    };

    if (showPromptEditor) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [showPromptEditor]);

  // 自动隐藏键盘提示
  useEffect(() => {
    const timer = setTimeout(() => {
      setShowKeyboardHints(false);
    }, 10000); // 10秒后自动隐藏

    return () => clearTimeout(timer);
  }, []);

  // 添加通知
  const addNotification = useCallback((notification: Omit<Notification, 'id'>) => {
    const id = `notification-${Date.now()}`;
    const newNotification = { ...notification, id };
    
    setNotifications(prev => [...prev, newNotification]);
    
    // 自动删除通知
    const duration = notification.duration || 5000;
    notificationTimeoutRef.current[id] = setTimeout(() => {
      removeNotification(id);
    }, duration);
  }, []);

  // 移除通知
  const removeNotification = useCallback((id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id));
    if (notificationTimeoutRef.current[id]) {
      clearTimeout(notificationTimeoutRef.current[id]);
      delete notificationTimeoutRef.current[id];
    }
  }, []);

  const loadPrompts = async () => {
    try {
      const savedPrompts = await invoke<AIPrompt[]>('get_ai_prompts');
      setPrompts(savedPrompts);
      // 设置默认激活的提示
      const active = savedPrompts.find(p => p.is_active);
      if (active) {
        setActivePrompt(active);
      }
      addNotification({
        type: 'success',
        title: '加载成功',
        message: `已加载 ${savedPrompts.length} 个AI提示配置`,
        duration: 3000
      });
    } catch (error) {
      console.error('加载AI提示失败:', error);
      addNotification({
        type: 'error',
        title: '加载失败',
        message: '无法加载AI提示配置，已使用默认设置'
      });
      // 创建默认提示
      const defaultPrompt: AIPrompt = {
        id: 'default',
        name: '默认提示',
        description: '标准语音转文字处理流程',
        agentChain: [
          { id: '1', type: 'speech-to-text', name: '语音转文字', config: {}, order: 0 },
          { id: '2', type: 'text-enhancer', name: '文本增强', config: {}, order: 1 },
          { id: '3', type: 'auto-input', name: '自动输入', config: {}, order: 2 }
        ],
        agent_type: 'text-enhancer',
        prompt_text: '请优化并增强以下文本',
        is_active: true,
        created_at: Date.now(),
        updated_at: Date.now()
      };
      setPrompts([defaultPrompt]);
      setActivePrompt(defaultPrompt);
    }
  };

  // 保存提示
  const savePrompt = async (prompt: AIPrompt) => {
    try {
      await invoke('save_ai_prompt', { prompt });
      await loadPrompts();
      setShowPromptEditor(false);
      addNotification({
        type: 'success',
        title: '保存成功',
        message: `提示 "${prompt.name}" 已成功保存`
      });
    } catch (error) {
      console.error('保存提示失败:', error);
      addNotification({
        type: 'error',
        title: '保存失败',
        message: '无法保存提示配置，请检查系统权限'
      });
    }
  };

  // 删除提示
  const deletePrompt = async (promptId: string) => {
    const promptToDelete = prompts.find(p => p.id === promptId);
    if (!promptToDelete) return;

    if (window.confirm(`确定要删除提示 "${promptToDelete.name}" 吗？`)) {
      try {
        await invoke('delete_ai_prompt', { promptId });
        await loadPrompts();
        addNotification({
          type: 'success',
          title: '删除成功',
          message: `提示 "${promptToDelete.name}" 已删除`
        });
      } catch (error) {
        console.error('删除提示失败:', error);
        addNotification({
          type: 'error',
          title: '删除失败',
          message: '无法删除提示配置'
        });
      }
    }
  };

  // 激活提示
  const activatePrompt = async (prompt: AIPrompt) => {
    try {
      // 先取消其他激活的提示
      const updatedPrompts = prompts.map(p => ({
        ...p,
        is_active: p.id === prompt.id
      }));
      setPrompts(updatedPrompts);
      setActivePrompt(prompt);
      
      await invoke('activate_ai_prompt', { promptId: prompt.id });
      addNotification({
        type: 'success',
        title: '已激活',
        message: `提示 "${prompt.name}" 现在是活动提示`,
        duration: 3000
      });
    } catch (error) {
      console.error('激活提示失败:', error);
      addNotification({
        type: 'error',
        title: '激活失败',
        message: '无法激活提示配置'
      });
    }
  };

  // 监听转录文本变化，自动处理
  useEffect(() => {
    if (transcriptionText && transcriptionText !== '' && !isRecording) {
      // 当录音停止且有新的转录文本时，自动处理
      processWithAgents(transcriptionText);
    }
  }, [transcriptionText, isRecording]);
  
  // 处理语音输入
  const processWithAgents = async (audioText: string) => {
    if (!activePrompt) return audioText;
    
    const totalSteps = activePrompt.agentChain?.length || 0;
    setProcessingState({
      isProcessing: true,
      progress: 0,
      totalSteps
    });
    setWorkflowStep('processing');
    
    let processedText = audioText;
    
    try {
      // 按顺序执行Agent链
      const chain = activePrompt.agentChain || [];
      for (let i = 0; i < chain.length; i++) {
        const agent = chain[i];
        setProcessingState(prev => ({
          ...prev,
          currentAgent: agent.name,
          progress: i + 1
        }));

        const request: AgentRequest = {
          agent_type: agent.type,
          input_text: processedText,
          prompt_id: activePrompt.id,
          additional_context: agent.config
        };
        
        const response = await invoke<AgentResponse>('process_with_agent', { request });
        
        if (response.success) {
          processedText = response.output_text;
        } else {
          throw new Error(response.error || 'Agent处理失败');
        }

        // 添加一个小延迟来显示进度
        await new Promise(resolve => setTimeout(resolve, 200));
      }
      
      setWorkflowStep('typing');
      // 通知父组件处理完成的文本
      if (onEnhancedTextReady) {
        onEnhancedTextReady(processedText);
      }

      addNotification({
        type: 'success',
        title: 'AI增强完成',
        message: `文本已通过 ${totalSteps} 个Agent处理完成`,
        duration: 3000
      });
      
      // 2秒后重置状态
      setTimeout(() => {
        setWorkflowStep('idle');
      }, 2000);
      
    } catch (error) {
      console.error('Agent处理失败:', error);
      addNotification({
        type: 'error',
        title: 'AI处理失败',
        message: '文本处理过程中出现错误，请检查Agent配置'
      });
      setWorkflowStep('idle');
    } finally {
      setProcessingState({
        isProcessing: false,
        progress: 0,
        totalSteps: 0
      });
    }
    
    return processedText;
  };

  // 添加Agent到链中
  const addAgentToChain = (agentType: string) => {
    const agent = availableAgents.find(a => a.type === agentType);
    if (!agent) return;
    
    const newAgent: AgentConfig = {
      id: `agent-${Date.now()}`,
      type: agentType,
      name: agent.name,
      config: {},
      order: newPrompt.agentChain.length
    };
    
    setNewPrompt({
      ...newPrompt,
      agentChain: [...newPrompt.agentChain, newAgent]
    });
  };

  // 从链中移除Agent
  const removeAgentFromChain = (agentId: string) => {
    setNewPrompt({
      ...newPrompt,
      agentChain: newPrompt.agentChain
        .filter(a => a.id !== agentId)
        .map((a, index) => ({ ...a, order: index }))
    });
  };

  // 拖拽处理
  const handleDragStart = (e: React.DragEvent, agentId: string) => {
    setDraggedItemId(agentId);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  };

  const handleDragEnd = () => {
    setDraggedItemId(null);
  };

  const handleDrop = (e: React.DragEvent, dropAgentId: string) => {
    e.preventDefault();
    if (!draggedItemId || draggedItemId === dropAgentId) return;

    const dragIndex = newPrompt.agentChain.findIndex(a => a.id === draggedItemId);
    const dropIndex = newPrompt.agentChain.findIndex(a => a.id === dropAgentId);
    
    if (dragIndex === -1 || dropIndex === -1) return;

    const draggedAgent = newPrompt.agentChain[dragIndex];
    const newChain = [...newPrompt.agentChain];
    newChain.splice(dragIndex, 1);
    newChain.splice(dropIndex, 0, draggedAgent);
    
    setNewPrompt({
      ...newPrompt,
      agentChain: newChain.map((a, index) => ({ ...a, order: index }))
    });

    addNotification({
      type: 'info',
      title: 'Agent顺序已调整',
      message: `${draggedAgent.name} 已移动到新位置`,
      duration: 2000
    });
  };

  // 重新排序Agent链（向上移动）
  const moveAgentUp = (agentId: string) => {
    const currentIndex = newPrompt.agentChain.findIndex(a => a.id === agentId);
    if (currentIndex > 0) {
      const newChain = [...newPrompt.agentChain];
      [newChain[currentIndex - 1], newChain[currentIndex]] = [newChain[currentIndex], newChain[currentIndex - 1]];
      setNewPrompt({
        ...newPrompt,
        agentChain: newChain.map((a, index) => ({ ...a, order: index }))
      });
    }
  };

  // 重新排序Agent链（向下移动）
  const moveAgentDown = (agentId: string) => {
    const currentIndex = newPrompt.agentChain.findIndex(a => a.id === agentId);
    if (currentIndex < newPrompt.agentChain.length - 1) {
      const newChain = [...newPrompt.agentChain];
      [newChain[currentIndex], newChain[currentIndex + 1]] = [newChain[currentIndex + 1], newChain[currentIndex]];
      setNewPrompt({
        ...newPrompt,
        agentChain: newChain.map((a, index) => ({ ...a, order: index }))
      });
    }
  };

  return (
    <div className="ai-prompts-container">
      {/* 主界面 - 三步工作流程 */}
      <div className="workflow-section">
        <h2>工作原理：</h2>
        
        <div className="workflow-steps">
          {/* 步骤1 */}
          <div className={`workflow-step ${workflowStep === 'listening' ? 'active' : ''}`}>
            <div className="step-number">1</div>
            <div className="step-content">
              <h3>步骤 1：激活并口述</h3>
              <p>使用快捷键或在所选应用中说话</p>
            </div>
          </div>

          <div className="workflow-arrow">→</div>

          {/* 步骤2 */}
          <div className={`workflow-step ${workflowStep === 'processing' ? 'active' : ''}`}>
            <div className="step-number">2</div>
            <div className="step-content">
              <h3>步骤 2：AI增强</h3>
              <p>您的语音用选定的提示进行处理</p>
            </div>
          </div>

          <div className="workflow-arrow">→</div>

          {/* 步骤3 */}
          <div className={`workflow-step ${workflowStep === 'typing' ? 'active' : ''}`}>
            <div className="step-number">3</div>
            <div className="step-content">
              <h3>步骤 3：自动输入</h3>
              <p>AI增强的文本自动输入到应用中</p>
            </div>
          </div>
        </div>

        {/* 编辑主提示按钮 */}
        <div className="prompt-actions">
          <button 
            className="edit-prompt-btn"
            onClick={() => setShowPromptEditor(true)}
          >
            <span className="btn-icon">✏️</span>
            编辑主提示
          </button>
          {activePrompt && (
            <div className="active-prompt-info">
              <span className="active-label">当前激活：</span>
              <span className="active-name">{activePrompt.name}</span>
            </div>
          )}
        </div>
      </div>

      {/* 提示编辑器对话框 */}
      {showPromptEditor && (
        <div className="prompt-editor-overlay">
          <div className="prompt-editor-dialog" ref={dialogRef}>
            <div className="editor-header">
              <h2>Agent配置</h2>
              <button 
                className="close-btn"
                onClick={() => setShowPromptEditor(false)}
              >
                ✕
              </button>
            </div>

            <div className="editor-content">
              {/* 左侧 - 提示列表 */}
              <div className="prompts-list">
                <div className="list-header">
                  <h3>我的提示</h3>
                  <button 
                    className="add-prompt-btn"
                    onClick={() => {
                      const newId = `prompt-${Date.now()}`;
                      setNewPrompt({
                        id: newId,
                        name: '新提示',
                        description: '',
                        agent_type: 'text-enhancer',
                        prompt_text: '',
                        is_active: false,
                        created_at: Date.now(),
                        updated_at: Date.now(),
                        agentChain: []
                      });
                    }}
                  >
                    + 新建
                  </button>
                </div>
                
                <div className="prompts-items">
                  {prompts.map(prompt => (
                    <div 
                      key={prompt.id}
                      className={`prompt-item ${prompt.is_active ? 'active' : ''} ${selectedPromptId === prompt.id ? 'selected' : ''}`}
                      onClick={() => {
                        setNewPrompt(prompt);
                        setSelectedPromptId(prompt.id);
                      }}
                    >
                      <div className="prompt-info">
                        <span className="prompt-name">{prompt.name}</span>
                        {prompt.is_active && <span className="active-badge">激活</span>}
                      </div>
                      <div className="prompt-actions">
                        <button 
                          className="activate-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            activatePrompt(prompt);
                          }}
                        >
                          {prompt.is_active ? '✓' : '○'}
                        </button>
                        <button 
                          className="delete-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            deletePrompt(prompt.id);
                          }}
                        >
                          🗑️
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* 右侧 - Agent配置 */}
              <div className="agent-configuration">
                <div className="config-header">
                  <input 
                    type="text"
                    className="prompt-name-input"
                    value={newPrompt.name}
                    onChange={(e) => setNewPrompt({ ...newPrompt, name: e.target.value })}
                    placeholder="提示名称"
                  />
                  <textarea 
                    className="prompt-description"
                    value={newPrompt.description}
                    onChange={(e) => setNewPrompt({ ...newPrompt, description: e.target.value })}
                    placeholder="描述这个提示的用途..."
                    rows={2}
                  />
                </div>

                <div className="agent-chain">
                  <h3>处理链路</h3>
                  <div className="chain-visualization">
                    {newPrompt.agentChain.length === 0 ? (
                      <div className="empty-chain">
                        <p>还没有添加任何Agent</p>
                        <p className="hint">从下方选择Agent添加到处理链路</p>
                      </div>
                    ) : (
                      newPrompt.agentChain.map((agent, index) => (
                        <div 
                          key={agent.id} 
                          className={`chain-item ${draggedItemId === agent.id ? 'dragging' : ''}`}
                          draggable
                          onDragStart={(e) => handleDragStart(e, agent.id)}
                          onDragOver={handleDragOver}
                          onDrop={(e) => handleDrop(e, agent.id)}
                          onDragEnd={handleDragEnd}
                        >
                          <div className="chain-order">{index + 1}</div>
                          <div className="chain-agent">
                            <span className="agent-name">{agent.name}</span>
                            <div className="agent-controls">
                              <button 
                                className="move-up-btn"
                                onClick={() => moveAgentUp(agent.id)}
                                disabled={index === 0}
                                title="向上移动"
                              >
                                ↑
                              </button>
                              <button 
                                className="move-down-btn"
                                onClick={() => moveAgentDown(agent.id)}
                                disabled={index === newPrompt.agentChain.length - 1}
                                title="向下移动"
                              >
                                ↓
                              </button>
                              <span className="drag-handle" title="拖拽排序">⋮⋮</span>
                              <button 
                                className="remove-agent"
                                onClick={() => removeAgentFromChain(agent.id)}
                                title="删除Agent"
                              >
                                ×
                              </button>
                            </div>
                          </div>
                          {index < newPrompt.agentChain.length - 1 && (
                            <div className="chain-connector">↓</div>
                          )}
                        </div>
                      ))
                    )}
                  </div>
                </div>

                <div className="available-agents">
                  <h3>可用的Agents</h3>
                  <div className="agents-grid">
                    {availableAgents.map(agent => (
                      <button 
                        key={agent.type}
                        className="agent-option"
                        onClick={() => addAgentToChain(agent.type)}
                      >
                        <span className="agent-icon">{agent.icon}</span>
                        <span className="agent-label">{agent.name}</span>
                      </button>
                    ))}
                  </div>
                </div>

                <div className="editor-footer">
                  <button 
                    className="cancel-btn"
                    onClick={() => setShowPromptEditor(false)}
                  >
                    取消
                  </button>
                  <button 
                    className="save-btn"
                    onClick={() => savePrompt(newPrompt)}
                    disabled={!newPrompt.name || newPrompt.agentChain.length === 0}
                  >
                    保存提示
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 处理状态提示 */}
      {processingState.isProcessing && (
        <div className="processing-indicator">
          <div className="processing-spinner"></div>
          <div className="processing-content">
            <div className="processing-text">
              {processingState.currentAgent ? 
                `正在执行: ${processingState.currentAgent}` : 
                '正在使用AI增强文本...'
              }
            </div>
            {processingState.totalSteps > 0 && (
              <div className="processing-progress">
                <div className="progress-bar">
                  <div 
                    className="progress-fill"
                    style={{ width: `${(processingState.progress / processingState.totalSteps) * 100}%` }}
                  ></div>
                </div>
                <span className="progress-text">
                  {processingState.progress}/{processingState.totalSteps}
                </span>
              </div>
            )}
          </div>
        </div>
      )}

      {/* 通知系统 */}
      <div className="notifications-container">
        {notifications.map(notification => (
          <div 
            key={notification.id}
            className={`notification ${notification.type}`}
          >
            <div className="notification-icon">
              {notification.type === 'success' && '✓'}
              {notification.type === 'error' && '✕'}
              {notification.type === 'warning' && '⚠'}
              {notification.type === 'info' && 'ℹ'}
            </div>
            <div className="notification-content">
              <div className="notification-title">{notification.title}</div>
              <div className="notification-message">{notification.message}</div>
            </div>
            <button 
              className="notification-close"
              onClick={() => removeNotification(notification.id)}
            >
              ×
            </button>
          </div>
        ))}
      </div>

      {/* 键盘快捷键提示 */}
      <div className={`keyboard-hints ${showKeyboardHints ? '' : 'hidden'}`}>
        <div className="keyboard-hint">
          <span className="kbd">Cmd/Ctrl</span> + <span className="kbd">E</span>
          <span>打开编辑器</span>
        </div>
        <div className="keyboard-hint">
          <span className="kbd">Cmd/Ctrl</span> + <span className="kbd">S</span>
          <span>保存提示</span>
        </div>
        <div className="keyboard-hint">
          <span className="kbd">Esc</span>
          <span>关闭弹窗</span>
        </div>
        <div className="keyboard-hint">
          <span className="kbd">F1</span>
          <span>显示/隐藏提示</span>
        </div>
      </div>
    </div>
  );
};

export default AIPrompts;