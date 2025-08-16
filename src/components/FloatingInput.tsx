import React, { useState, useRef, useEffect } from 'react';
import { appWindow, LogicalPosition } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import './FloatingInput.css';

type InputState = 'idle' | 'recording' | 'processing' | 'success';

const FloatingInput: React.FC = () => {
  const [state, setState] = useState<InputState>('idle');
  const [inputText, setInputText] = useState('');
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // 设置窗口属性
    const setupWindow = async () => {
      await appWindow.setAlwaysOnTop(true);
      await appWindow.setDecorations(false);
      await appWindow.setResizable(false);
      await appWindow.setSkipTaskbar(true);
      
      // 居中显示
      try {
        // 使用屏幕尺寸来居中
        const screenWidth = window.screen.width;
        const screenHeight = window.screen.height;
        const x = Math.floor((screenWidth - 600) / 2);
        const y = Math.floor(screenHeight * 0.15);
        await appWindow.setPosition(new LogicalPosition(x, y));
      } catch (error) {
        console.error('设置窗口位置失败:', error);
      }
    };
    
    setupWindow();

    // 监听快捷键触发
    const unlistenTrigger = listen('floating_input_triggered', () => {
      setState('idle');
      setInputText('');
      inputRef.current?.focus();
    });

    // 自动聚焦
    setTimeout(() => {
      inputRef.current?.focus();
    }, 100);

    return () => {
      unlistenTrigger.then(fn => fn());
    };
  }, []);

  // 使用Tauri的startDragging API处理窗口拖拽
  const handleStartDrag = async () => {
    try {
      await appWindow.startDragging();
    } catch (error) {
      console.error('拖拽失败:', error);
    }
  };

  // 开始录音
  const startRecording = async () => {
    try {
      setState('recording');
      setInputText('录音中...');
      
      await invoke('start_recording', { deviceId: 'default' });
      
      // 可选：添加音频电平监控
      // 你可以在这里添加定时器来获取音频电平
      
    } catch (error) {
      console.error('开始录音失败:', error);
      setState('idle');
      setInputText('录音失败，请检查麦克风权限');
    }
  };

  // 停止录音并处理
  const stopRecording = async () => {
    try {
      setState('processing');
      setInputText('转写中...');
      
      // 停止录音并获取转录结果
      const result = await invoke<string>('stop_recording_and_transcribe', {
        model: 'luyingwang-online'
      });
      
      if (result) {
        setState('success');
        setInputText(result);
        
        // 自动插入文本到当前应用
        try {
          await invoke('insert_text_to_app', { text: result });
          console.log('文本已插入到当前应用');
        } catch (error) {
          console.error('插入文本失败:', error);
        }
        
        // 2秒后自动关闭窗口
        setTimeout(() => {
          appWindow.hide();
          setState('idle');
          setInputText('');
        }, 2000);
      } else {
        setState('idle');
        setInputText('');
      }
      
    } catch (error) {
      console.error('停止录音失败:', error);
      setState('idle');
      setInputText('转录失败，请重试');
    }
  };

  // 处理键盘事件
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      appWindow.hide();
    } else if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (state === 'idle' && inputText.trim()) {
        // 处理文本输入
        handleTextSubmit();
      }
    }
  };

  // 处理文本提交
  const handleTextSubmit = async () => {
    setState('processing');
    
    // 模拟处理
    setTimeout(() => {
      setState('success');
      setTimeout(() => {
        setState('idle');
        setInputText('');
      }, 1500);
    }, 1000);
  };

  // 获取状态图标
  const getStatusIcon = () => {
    switch (state) {
      case 'recording':
        return '🎤';
      case 'processing':
        return '⏳';
      case 'success':
        return '✅';
      default:
        return '💬';
    }
  };

  // 获取占位符文本
  const getPlaceholder = () => {
    switch (state) {
      case 'recording':
        return '正在录音...';
      case 'processing':
        return '正在处理...';
      case 'success':
        return '完成！';
      default:
        return '输入文字或点击麦克风录音';
    }
  };

  return (
    <div 
      ref={containerRef}
      className={`floating-input-container ${state}`}
    >
      <div className="floating-input-wrapper">
        {/* 拖动把手 */}
        <div 
          className="drag-handle" 
          onMouseDown={handleStartDrag}
        >
          <span className="drag-dots">⋮⋮⋮</span>
        </div>

        {/* 主输入区域 */}
        <div className="input-area">
          <div className="status-icon">
            {getStatusIcon()}
          </div>
          
          <input
            ref={inputRef}
            type="text"
            className="text-input"
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={getPlaceholder()}
            disabled={state !== 'idle'}
          />
          
          {/* 操作按钮 */}
          <div className="action-buttons">
            {state === 'idle' && (
              <button 
                className="mic-button"
                onClick={startRecording}
                title="语音输入"
              >
                🎙️
              </button>
            )}
            
            {state === 'recording' && (
              <button 
                className="stop-button pulse"
                onClick={stopRecording}
                title="停止录音"
              >
                ⏹️
              </button>
            )}
            
            {state === 'processing' && (
              <div className="spinner"></div>
            )}
            
            {state === 'success' && (
              <button 
                className="copy-button"
                onClick={() => {
                  navigator.clipboard.writeText(inputText);
                }}
                title="复制"
              >
                📋
              </button>
            )}
          </div>
        </div>

        {/* 关闭按钮 */}
        <button 
          className="close-button"
          onClick={() => appWindow.hide()}
          title="关闭"
        >
          ×
        </button>
      </div>

      {/* 提示信息 */}
      {state === 'idle' && (
        <div className="hint-text">
          按 Enter 发送 · Esc 退出 · 拖动移动位置
        </div>
      )}

      {/* 处理动画 */}
      {state === 'processing' && (
        <div className="processing-animation">
          <div className="processing-bar"></div>
        </div>
      )}

      {/* 成功动画 */}
      {state === 'success' && (
        <div className="success-animation">
          <div className="success-check">✓</div>
        </div>
      )}
    </div>
  );
};

export default FloatingInput;