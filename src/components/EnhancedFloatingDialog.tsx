// 增强版浮动对话框 - 优化用户体验版本（React性能优化）
import React, { useState, useEffect, useRef, useCallback, useMemo, memo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './EnhancedFloatingDialog.css';

interface EnhancedFloatingDialogProps {
  isVisible: boolean;
  isRecording: boolean;
  transcriptionText: string;
  onClose: () => void;
  onToggleRecording: () => void;
  onSubmitPrompt: (prompt: string) => void;
  onOpenSettings?: () => void;
  onShowHistory?: () => void;
}

type DialogMode = 'idle' | 'listening' | 'processing' | 'result' | 'settings';
type FloatingPosition = 'center' | 'top' | 'bottom' | 'left' | 'right';

const EnhancedFloatingDialog: React.FC<EnhancedFloatingDialogProps> = memo(({
  isVisible,
  isRecording,
  transcriptionText,
  onClose,
  onToggleRecording,
  onSubmitPrompt,
  onOpenSettings,
  onShowHistory
}) => {
  const [prompt, setPrompt] = useState('');
  const [mode, setMode] = useState<DialogMode>('idle');
  const [currentApp, setCurrentApp] = useState({ name: '未知应用', icon: '📱' });
  const [isExpanded, setIsExpanded] = useState(false);
  const [position, setPosition] = useState<FloatingPosition>('center');
  const [showQuickActions, setShowQuickActions] = useState(false);
  const [confidence, setConfidence] = useState<number>(0);
  const [wordCount, setWordCount] = useState(0);
  const [isMinimized, setIsMinimized] = useState(false);
  
  const inputRef = useRef<HTMLInputElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const recordingStartTime = useRef<number>(0);
  const [recordingDuration, setRecordingDuration] = useState(0);

  // 获取当前应用信息
  useEffect(() => {
    if (isVisible) {
      getCurrentAppInfo();
      focusInput();
      setIsMinimized(false);
    }
  }, [isVisible]);

  // 监听录音和转录状态
  useEffect(() => {
    if (isRecording) {
      setMode('listening');
      setPrompt('');
      recordingStartTime.current = Date.now();
      setRecordingDuration(0);
    } else if (transcriptionText && transcriptionText !== prompt) {
      setMode('result');
      setPrompt(transcriptionText);
      setWordCount(transcriptionText.split(/\s+/).length);
      // 模拟置信度计算
      setConfidence(Math.random() * 0.3 + 0.7);
    }
  }, [isRecording, transcriptionText]);

  // 录音时长计时器
  useEffect(() => {
    let interval: number;
    if (isRecording) {
      interval = setInterval(() => {
        setRecordingDuration((Date.now() - recordingStartTime.current) / 1000);
      }, 100);
    }
    return () => clearInterval(interval);
  }, [isRecording]);

  const getCurrentAppInfo = async () => {
    try {
      const appInfo = await invoke<{[key: string]: string}>('get_current_app_info');
      setCurrentApp({
        name: appInfo.name || '未知应用',
        icon: appInfo.icon || '📱'
      });
    } catch (error) {
      console.error('获取当前应用信息失败:', error);
    }
  };

  const focusInput = useCallback(() => {
    setTimeout(() => {
      if (inputRef.current && !isRecording) {
        inputRef.current.focus();
        inputRef.current.select();
      }
    }, 100);
  }, [isRecording]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    } else if (e.key === 'Escape') {
      if (isExpanded) {
        setIsExpanded(false);
      } else {
        onClose();
      }
    } else if (e.key === 'Tab') {
      e.preventDefault();
      setIsExpanded(!isExpanded);
    }
  };

  const handleSubmit = () => {
    if (prompt.trim()) {
      onSubmitPrompt(prompt.trim());
      setMode('processing');
      setShowQuickActions(false);
      // 提交后稍微延迟关闭，让用户看到处理状态
      setTimeout(() => {
        onClose();
      }, 800);
    }
  };

  const handleQuickAction = (action: string) => {
    const quickPrompts = {
      'enhance': '请帮我润色优化这段文字，让它更加清晰专业',
      'translate': '请将这段内容翻译成英文',
      'summarize': '请为这段内容生成一个简洁的摘要',
      'expand': '请详细展开这个话题，提供更多相关信息',
    };
    
    if (quickPrompts[action as keyof typeof quickPrompts]) {
      const actionPrompt = quickPrompts[action as keyof typeof quickPrompts];
      if (prompt.trim()) {
        setPrompt(`${prompt}\n\n${actionPrompt}`);
      } else {
        setPrompt(actionPrompt);
      }
      focusInput();
    }
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const handleMinimize = () => {
    setIsMinimized(!isMinimized);
    setIsExpanded(false);
  };

  const getStatusMessage = () => {
    switch (mode) {
      case 'listening':
        return `🎙️ 正在录音... ${formatDuration(recordingDuration)}`;
      case 'processing':
        return '⚡ AI处理中...';
      case 'result':
        return `✅ 转录完成 · ${wordCount} 词 · ${Math.round(confidence * 100)}% 置信度`;
      default:
        return '💬 说出你的想法，或输入文字';
    }
  };

  if (!isVisible) return null;

  return (
    <div className={`enhanced-dialog-overlay ${position}`} onClick={onClose}>
      <div 
        ref={dialogRef}
        className={`enhanced-dialog ${isExpanded ? 'expanded' : ''} ${isMinimized ? 'minimized' : ''} ${mode}`}
        onClick={(e) => e.stopPropagation()}
        data-mode={mode}
      >
        {/* 最小化状态 */}
        {isMinimized && (
          <div className="minimized-view">
            <button className="minimized-content" onClick={handleMinimize}>
              <div className="minimized-icon">{isRecording ? '🔴' : '🎤'}</div>
              <div className="minimized-text">
                {isRecording ? formatDuration(recordingDuration) : 'Recording King'}
              </div>
            </button>
          </div>
        )}

        {/* 完整状态 */}
        {!isMinimized && (
          <>
            {/* 标题栏 */}
            <div className="dialog-titlebar">
              <div className="titlebar-left">
                <div className="app-context">
                  <span className="app-icon">{currentApp.icon}</span>
                  <span className="app-name">{currentApp.name}</span>
                </div>
              </div>
              
              <div className="titlebar-right">
                <button 
                  className="titlebar-btn minimize-btn" 
                  onClick={handleMinimize}
                  title="最小化"
                >
                  ➖
                </button>
                <button 
                  className="titlebar-btn expand-btn" 
                  onClick={() => setIsExpanded(!isExpanded)}
                  title={isExpanded ? "收起" : "展开更多选项"}
                >
                  {isExpanded ? '⬇️' : '⬆️'}
                </button>
                <button 
                  className="titlebar-btn close-btn" 
                  onClick={onClose}
                  title="关闭"
                >
                  ✕
                </button>
              </div>
            </div>

            {/* 主要内容区 */}
            <div className="dialog-main">
              {/* 状态指示器 */}
              <div className={`status-bar ${mode}`}>
                <div className="status-indicator">
                  <div className={`status-dot ${mode}`}></div>
                  <span className="status-text">{getStatusMessage()}</span>
                </div>
                
                {mode === 'result' && (
                  <div className="result-stats">
                    <span className="word-count">{wordCount} 词</span>
                    <span className="confidence">{Math.round(confidence * 100)}%</span>
                  </div>
                )}
              </div>

              {/* 输入区域 */}
              <div className="input-section">
                <div className="input-container">
                  <textarea
                    ref={inputRef as any}
                    className={`main-input ${mode}`}
                    placeholder={
                      mode === 'listening' 
                        ? "正在监听您的语音..." 
                        : mode === 'processing' 
                        ? "AI正在处理中..." 
                        : "输入您的问题或按住麦克风说话..."
                    }
                    value={prompt}
                    onChange={(e) => setPrompt(e.target.value)}
                    onKeyDown={handleKeyDown}
                    disabled={mode === 'listening' || mode === 'processing'}
                    rows={isExpanded ? 4 : 2}
                  />
                  
                  {/* 输入状态指示器 */}
                  {mode === 'listening' && (
                    <div className="input-indicator listening">
                      <div className="wave-animation">
                        <span></span><span></span><span></span><span></span>
                      </div>
                    </div>
                  )}
                  
                  {mode === 'processing' && (
                    <div className="input-indicator processing">
                      <div className="processing-spinner"></div>
                    </div>
                  )}
                </div>

                {/* 操作按钮组 */}
                <div className="action-buttons">
                  <button 
                    className={`record-button ${isRecording ? 'recording' : ''}`}
                    onClick={onToggleRecording}
                    title={isRecording ? "停止录音" : "开始录音"}
                  >
                    <div className="record-icon">
                      {isRecording ? '⏹️' : '🎤'}
                    </div>
                    {isRecording && (
                      <div className="record-pulse"></div>
                    )}
                  </button>

                  {prompt.trim() && mode !== 'processing' && (
                    <button 
                      className="submit-button"
                      onClick={handleSubmit}
                      title="发送 (Enter)"
                    >
                      ➤
                    </button>
                  )}

                  <button 
                    className="quick-actions-button"
                    onClick={() => setShowQuickActions(!showQuickActions)}
                    title="快速操作"
                  >
                    ⚡
                  </button>
                </div>
              </div>

              {/* 快速操作面板 */}
              {showQuickActions && (
                <div className="quick-actions-panel">
                  <div className="quick-actions-grid">
                    <button 
                      className="quick-action-btn enhance"
                      onClick={() => handleQuickAction('enhance')}
                    >
                      <span className="action-icon">✨</span>
                      <span className="action-text">润色</span>
                    </button>
                    <button 
                      className="quick-action-btn translate"
                      onClick={() => handleQuickAction('translate')}
                    >
                      <span className="action-icon">🌐</span>
                      <span className="action-text">翻译</span>
                    </button>
                    <button 
                      className="quick-action-btn summarize"
                      onClick={() => handleQuickAction('summarize')}
                    >
                      <span className="action-icon">📝</span>
                      <span className="action-text">摘要</span>
                    </button>
                    <button 
                      className="quick-action-btn expand"
                      onClick={() => handleQuickAction('expand')}
                    >
                      <span className="action-icon">📖</span>
                      <span className="action-text">详述</span>
                    </button>
                  </div>
                </div>
              )}

              {/* 展开区域 */}
              {isExpanded && (
                <div className="expanded-section">
                  {/* 历史记录快速访问 */}
                  <div className="history-section">
                    <h4>📋 最近记录</h4>
                    <div className="recent-items">
                      <button className="recent-item">
                        <span className="recent-text">上次的会议纪要...</span>
                        <span className="recent-time">5分钟前</span>
                      </button>
                      <button className="recent-item">
                        <span className="recent-text">翻译文档内容...</span>
                        <span className="recent-time">1小时前</span>
                      </button>
                    </div>
                  </div>

                  {/* 快速设置 */}
                  <div className="settings-section">
                    <h4>⚙️ 快速设置</h4>
                    <div className="setting-toggles">
                      <label className="setting-toggle">
                        <input type="checkbox" defaultChecked />
                        <span>自动标点</span>
                      </label>
                      <label className="setting-toggle">
                        <input type="checkbox" />
                        <span>连续识别</span>
                      </label>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 底部快捷键提示 */}
            <div className="dialog-footer">
              <div className="shortcuts">
                <span className="shortcut"><kbd>⌘⇧R</kbd> 快捷录音</span>
                <span className="shortcut"><kbd>↵</kbd> 发送</span>
                <span className="shortcut"><kbd>⇥</kbd> 展开</span>
                <span className="shortcut"><kbd>esc</kbd> 关闭</span>
              </div>
              
              <div className="footer-actions">
                {onShowHistory && (
                  <button className="footer-btn" onClick={onShowHistory}>
                    📋 历史
                  </button>
                )}
                {onOpenSettings && (
                  <button className="footer-btn" onClick={onOpenSettings}>
                    ⚙️ 设置
                  </button>
                )}
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
});

EnhancedFloatingDialog.displayName = 'EnhancedFloatingDialog';

export default EnhancedFloatingDialog;