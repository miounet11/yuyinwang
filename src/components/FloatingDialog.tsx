import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './FloatingDialog.css';

interface FloatingDialogProps {
  isVisible: boolean;
  isRecording: boolean;
  transcriptionText: string;
  onClose: () => void;
  onToggleRecording: () => void;
  onSubmitPrompt: (prompt: string) => void;
}

const FloatingDialog: React.FC<FloatingDialogProps> = ({
  isVisible,
  isRecording,
  transcriptionText,
  onClose,
  onToggleRecording,
  onSubmitPrompt
}) => {
  const [prompt, setPrompt] = useState('');
  const [mode, setMode] = useState<'idle' | 'listening' | 'processing' | 'result'>('idle');
  const [currentApp, setCurrentApp] = useState({ name: '未知应用', icon: '📱' });
  // 保留扩展功能供将来使用
  // const [isExpanded, setIsExpanded] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // 获取当前应用信息
  useEffect(() => {
    if (isVisible) {
      getCurrentAppInfo();
      if (inputRef.current) {
        inputRef.current.focus();
      }
    }
  }, [isVisible]);

  // 监听录音和转录状态
  useEffect(() => {
    if (isRecording) {
      setMode('listening');
      setPrompt('');
    } else if (transcriptionText && transcriptionText !== prompt) {
      setMode('result');
      setPrompt(transcriptionText);
    }
  }, [isRecording, transcriptionText]);

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

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    } else if (e.key === 'Escape') {
      onClose();
    }
  };

  const handleSubmit = () => {
    if (prompt.trim()) {
      onSubmitPrompt(prompt.trim());
      setMode('processing');
      setTimeout(() => {
        onClose();
      }, 1000);
    }
  };

  // 保留扩展功能供将来使用
  // const toggleExpanded = () => {
  //   setIsExpanded(!isExpanded);
  // };

  if (!isVisible) return null;

  return (
    <div className="floating-dialog-overlay" onClick={onClose}>
      <div className="floating-dialog" onClick={(e) => e.stopPropagation()}>
        {/* 应用图标和主提示区域 */}
        <div className="dialog-main">
          <div className="app-info">
            <div className="app-icon">{currentApp.icon}</div>
            <div className="app-details">
              <div className="app-name">{currentApp.name}</div>
              <div className="spokenly-brand">👑 Recording King</div>
            </div>
          </div>

          {/* 主要输入框 */}
          <div className="input-section">
            <div className="input-container">
              <input
                ref={inputRef}
                type="text"
                className={`main-input ${mode === 'listening' ? 'listening' : ''}`}
                placeholder={
                  mode === 'listening' 
                    ? "正在监听..." 
                    : mode === 'processing' 
                    ? "转写中..." 
                    : "You 你好你好你好?"
                }
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                onKeyDown={handleKeyDown}
                disabled={mode === 'listening'}
              />
              
              {mode === 'listening' && (
                <div className="listening-indicator">
                  <div className="pulse-animation"></div>
                </div>
              )}
              
              {mode === 'processing' && (
                <div className="processing-indicator">
                  <div className="loading-dots">
                    <span></span>
                    <span></span>
                    <span></span>
                  </div>
                </div>
              )}
            </div>

            {/* 录音按钮 */}
            <button 
              className={`record-button ${isRecording ? 'recording' : ''}`}
              onClick={onToggleRecording}
            >
              {isRecording ? '🎙️' : '🎤'}
            </button>
          </div>
        </div>

        {/* 底部状态指示 */}
        <div className="dialog-footer">
          <div className="status-info">
            {mode === 'listening' && '🎙️ 正在监听...'}
            {mode === 'processing' && '⚡ AI处理中...'}
            {mode === 'result' && '✅ 转录完成'}
            {mode === 'idle' && '按住快捷键开始录音'}
          </div>
          
          <div className="footer-actions">
            <span className="hint">ESC 关闭</span>
            <span className="hint">Enter 发送</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default FloatingDialog;