import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { appWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import './QuickVoiceInput.css';

interface QuickVoiceInputProps {
  onClose?: () => void;
  onTextReady?: (text: string) => void;
}

const QuickVoiceInput: React.FC<QuickVoiceInputProps> = ({ onClose, onTextReady }) => {
  const [isRecording, setIsRecording] = useState(false);
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [transcriptionText, setTranscriptionText] = useState('');
  const [audioLevel, setAudioLevel] = useState(0);
  const [recordingDuration, setRecordingDuration] = useState(0);
  const [error, setError] = useState('');
  
  const timerRef = useRef<NodeJS.Timeout | null>(null);
  const startTimeRef = useRef<number>(0);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // 设置窗口属性
    const setupWindow = async () => {
      try {
        // 设置窗口始终在最前
        await appWindow.setAlwaysOnTop(true);
        // 设置窗口装饰（无标题栏）
        await appWindow.setDecorations(false);
        // 设置窗口大小
        await appWindow.setSize(new LogicalSize(400, 120));
        // 获取光标位置并设置窗口位置
        const position = await invoke<{ x: number, y: number }>('get_cursor_position');
        await appWindow.setPosition(new LogicalPosition(position.x, position.y - 60));
      } catch (error) {
        console.error('设置窗口属性失败:', error);
      }
    };

    setupWindow();
    
    // 自动开始录音
    startRecording();

    // 监听快捷键释放事件（停止录音）
    const unlistenKeyRelease = listen('quick_voice_key_released', () => {
      if (isRecording) {
        stopRecording();
      }
    });

    // 监听ESC键关闭窗口
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleClose();
      }
    };
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      unlistenKeyRelease.then(fn => fn());
      document.removeEventListener('keydown', handleKeyDown);
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
      // 确保在组件卸载时停止录音
      if (isRecording) {
        invoke('stop_recording').catch(console.error);
      }
    };
  }, [isRecording]); // Add isRecording to dependencies

  const startRecording = async () => {
    try {
      // 先尝试停止任何现有的录音
      try {
        await invoke('stop_recording');
      } catch (e) {
        // 忽略错误，可能没有正在进行的录音
      }
      
      setError('');
      setIsRecording(true);
      setTranscriptionText('');
      startTimeRef.current = Date.now();
      
      // 启动录音
      await invoke('start_recording', {
        deviceId: null
      });

      // 启动计时器
      timerRef.current = setInterval(() => {
        const duration = (Date.now() - startTimeRef.current) / 1000;
        setRecordingDuration(duration);
        
        // 模拟音频电平
        setAudioLevel(Math.random() * 0.8 + 0.2);
      }, 100);
    } catch (error) {
      console.error('开始录音失败:', error);
      setError(`录音失败: ${error}`);
      setIsRecording(false);
    }
  };

  const stopRecording = async () => {
    try {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }

      setIsRecording(false);
      setIsTranscribing(true);
      setAudioLevel(0);

      // 停止录音并获取转录
      const result = await invoke<string>('stop_recording', {
        model: 'luyingwang-online'
      });

      setIsTranscribing(false);
      setTranscriptionText(result);

      // 自动插入文本到当前应用
      if (result && onTextReady) {
        onTextReady(result);
        // 插入文本后自动关闭
        setTimeout(() => {
          handleClose();
        }, 500);
      }
    } catch (error) {
      console.error('停止录音失败:', error);
      setError(`转录失败: ${error}`);
      setIsTranscribing(false);
    }
  };

  const handleClose = () => {
    if (isRecording) {
      invoke('stop_recording').catch(console.error);
    }
    if (onClose) {
      onClose();
    } else {
      appWindow.close();
    }
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="quick-voice-input" ref={containerRef}>
      <div className="voice-input-container">
        {/* 状态指示器 */}
        <div className={`status-indicator ${isRecording ? 'recording' : isTranscribing ? 'transcribing' : ''}`}>
          <div className="status-icon">
            {isRecording ? '🎤' : isTranscribing ? '⏳' : '✅'}
          </div>
          {isRecording && (
            <div className="recording-pulse">
              <div className="pulse-ring"></div>
            </div>
          )}
        </div>

        {/* 主要内容区 */}
        <div className="voice-input-content">
          {isRecording ? (
            <>
              <div className="recording-info">
                <span className="status-text">正在录音...</span>
                <span className="duration">{formatDuration(recordingDuration)}</span>
              </div>
              <div className="audio-level-bar">
                <div 
                  className="audio-level-fill" 
                  style={{ width: `${audioLevel * 100}%` }}
                />
              </div>
              <div className="hint-text">松开快捷键停止录音</div>
            </>
          ) : isTranscribing ? (
            <div className="transcribing-info">
              <span className="status-text">正在转录...</span>
              <div className="loading-spinner"></div>
            </div>
          ) : transcriptionText ? (
            <div className="transcription-result">
              <span className="result-text">{transcriptionText}</span>
            </div>
          ) : error ? (
            <div className="error-info">
              <span className="error-text">{error}</span>
            </div>
          ) : null}
        </div>

        {/* 关闭按钮 */}
        <button className="close-btn" onClick={handleClose} title="关闭 (ESC)">
          ×
        </button>
      </div>

      {/* 快捷键提示 */}
      <div className="shortcut-hint">
        <kbd>ESC</kbd> 取消 · <kbd>按住快捷键</kbd> 录音
      </div>
    </div>
  );
};

export default QuickVoiceInput;