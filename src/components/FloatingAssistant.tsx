import React, { useState, useRef, useEffect } from 'react';
import { appWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import './FloatingAssistant.css';

interface Position {
  x: number;
  y: number;
}

const FloatingAssistant: React.FC = () => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isRecording, setIsRecording] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [position, setPosition] = useState<Position>({ x: window.innerWidth - 80, y: 100 });
  const [showContextMenu, setShowContextMenu] = useState(false);
  const [audioLevel, setAudioLevel] = useState(0);
  const [transcriptionText, setTranscriptionText] = useState('');
  const [isMinimized, setIsMinimized] = useState(false);
  
  const dragRef = useRef<{ startX: number; startY: number; startPosX: number; startPosY: number } | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const recordingTimerRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    // 设置窗口属性
    const setupWindow = async () => {
      await appWindow.setAlwaysOnTop(true);
      await appWindow.setDecorations(false);
      await appWindow.setResizable(false);
      await appWindow.setSkipTaskbar(true);
    };
    
    setupWindow();

    // 监听快捷键事件
    const unlistenShortcut = listen('floating_assistant_toggle', () => {
      setIsExpanded(!isExpanded);
    });

    // 清理
    return () => {
      unlistenShortcut.then(fn => fn());
    };
  }, []);

  // 处理拖拽
  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button === 0) { // 左键
      setIsDragging(true);
      dragRef.current = {
        startX: e.clientX,
        startY: e.clientY,
        startPosX: position.x,
        startPosY: position.y
      };
    }
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (isDragging && dragRef.current) {
      const deltaX = e.clientX - dragRef.current.startX;
      const deltaY = e.clientY - dragRef.current.startY;
      
      let newX = dragRef.current.startPosX + deltaX;
      let newY = dragRef.current.startPosY + deltaY;
      
      // 边界检测和吸附
      const screenWidth = window.innerWidth;
      const screenHeight = window.innerHeight;
      const threshold = 20;
      
      if (newX < threshold) newX = 0;
      if (newX > screenWidth - (isExpanded ? 300 : 60) - threshold) {
        newX = screenWidth - (isExpanded ? 300 : 60);
      }
      if (newY < threshold) newY = 0;
      if (newY > screenHeight - (isExpanded ? 400 : 60) - threshold) {
        newY = screenHeight - (isExpanded ? 400 : 60);
      }
      
      setPosition({ x: newX, y: newY });
    }
  };

  const handleMouseUp = () => {
    setIsDragging(false);
    dragRef.current = null;
  };

  // 处理右键菜单
  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    setShowContextMenu(true);
  };

  // 快速语音输入
  const startQuickVoiceInput = async () => {
    try {
      setIsRecording(true);
      setTranscriptionText('');
      
      await invoke('start_recording', { deviceId: 'default' });
      
      // 监控音频电平
      recordingTimerRef.current = setInterval(async () => {
        try {
          const level = await invoke<number>('get_audio_level');
          setAudioLevel(Math.min(1.0, level));
        } catch {
          setAudioLevel(Math.random() * 0.5 + 0.3);
        }
      }, 100);
    } catch (error) {
      console.error('启动录音失败:', error);
      setIsRecording(false);
    }
  };

  const stopVoiceInput = async () => {
    try {
      if (recordingTimerRef.current) {
        clearInterval(recordingTimerRef.current);
        recordingTimerRef.current = null;
      }
      
      setIsRecording(false);
      const result = await invoke<string>('stop_recording_and_transcribe', {
        model: 'luyingwang-online'
      });
      
      if (result) {
        setTranscriptionText(result);
        // 自动插入文本
        await invoke('insert_text_to_app', { text: result });
        
        // 3秒后清除显示
        setTimeout(() => {
          setTranscriptionText('');
        }, 3000);
      }
    } catch (error) {
      console.error('停止录音失败:', error);
    }
  };

  // 切换录音状态
  const toggleRecording = () => {
    if (isRecording) {
      stopVoiceInput();
    } else {
      startQuickVoiceInput();
    }
  };

  // 快捷操作
  const quickActions = [
    {
      icon: '🎤',
      label: '语音输入',
      action: toggleRecording,
      active: isRecording
    },
    {
      icon: '📝',
      label: '快速笔记',
      action: () => invoke('open_quick_note')
    },
    {
      icon: '📋',
      label: '剪贴板',
      action: () => invoke('show_clipboard_history')
    },
    {
      icon: '🔍',
      label: '搜索',
      action: () => invoke('show_search')
    },
    {
      icon: '⚙️',
      label: '设置',
      action: () => invoke('show_settings')
    }
  ];

  return (
    <div 
      ref={containerRef}
      className={`floating-assistant ${isExpanded ? 'expanded' : ''} ${isMinimized ? 'minimized' : ''} ${isDragging ? 'dragging' : ''}`}
      style={{ 
        left: `${position.x}px`, 
        top: `${position.y}px`,
        position: 'fixed'
      }}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    >
      {/* 悬浮球主体 */}
      <div 
        className="floating-ball"
        onMouseDown={handleMouseDown}
        onContextMenu={handleContextMenu}
        onClick={() => !isDragging && setIsExpanded(!isExpanded)}
      >
        <div className={`ball-icon ${isRecording ? 'recording' : ''}`}>
          {isRecording ? '🔴' : '🎯'}
        </div>
        {isRecording && (
          <div className="recording-pulse">
            <div className="pulse-ring"></div>
          </div>
        )}
      </div>

      {/* 展开面板 */}
      {isExpanded && !isMinimized && (
        <div className="expanded-panel">
          <div className="panel-header">
            <span className="panel-title">快捷助手</span>
            <div className="panel-controls">
              <button 
                className="control-btn minimize-btn"
                onClick={() => setIsMinimized(true)}
              >
                ➖
              </button>
              <button 
                className="control-btn close-btn"
                onClick={() => setIsExpanded(false)}
              >
                ✕
              </button>
            </div>
          </div>

          <div className="panel-content">
            {/* 快捷操作网格 */}
            <div className="quick-actions-grid">
              {quickActions.map((action, index) => (
                <button
                  key={index}
                  className={`action-item ${action.active ? 'active' : ''}`}
                  onClick={action.action}
                >
                  <span className="action-icon">{action.icon}</span>
                  <span className="action-label">{action.label}</span>
                </button>
              ))}
            </div>

            {/* 录音状态显示 */}
            {isRecording && (
              <div className="recording-status">
                <div className="status-text">正在录音...</div>
                <div className="audio-visualizer">
                  <div 
                    className="audio-level-bar"
                    style={{ height: `${audioLevel * 100}%` }}
                  />
                </div>
              </div>
            )}

            {/* 转录结果显示 */}
            {transcriptionText && (
              <div className="transcription-result">
                <div className="result-text">{transcriptionText}</div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* 右键菜单 */}
      {showContextMenu && (
        <div 
          className="context-menu"
          onMouseLeave={() => setShowContextMenu(false)}
        >
          <div className="menu-item" onClick={toggleRecording}>
            {isRecording ? '停止录音' : '开始录音'}
          </div>
          <div className="menu-item" onClick={() => setIsExpanded(!isExpanded)}>
            {isExpanded ? '收起面板' : '展开面板'}
          </div>
          <div className="menu-divider"></div>
          <div className="menu-item" onClick={() => invoke('show_main_window')}>
            打开主窗口
          </div>
          <div className="menu-item" onClick={() => invoke('show_settings')}>
            设置
          </div>
          <div className="menu-divider"></div>
          <div className="menu-item" onClick={() => appWindow.close()}>
            退出悬浮球
          </div>
        </div>
      )}
    </div>
  );
};

export default FloatingAssistant;