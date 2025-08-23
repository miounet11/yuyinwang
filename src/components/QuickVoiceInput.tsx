import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { appWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import './QuickVoiceInput.css';

interface QuickVoiceInputProps {
  onClose?: () => void;
  onTextReady?: (text: string) => void;
}

interface ActiveAppInfo {
  name: string;
  bundle_id?: string;
  icon?: string;
}

const QuickVoiceInput: React.FC<QuickVoiceInputProps> = ({ onClose, onTextReady }) => {
  const [isRecording, setIsRecording] = useState(false);
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [transcriptionText, setTranscriptionText] = useState('');
  const [audioLevel, setAudioLevel] = useState(0);
  const [recordingDuration, setRecordingDuration] = useState(0);
  const [error, setError] = useState('');
  const [originalApp, setOriginalApp] = useState<ActiveAppInfo | null>(null);
  
  const timerRef = useRef<number | null>(null);
  const startTimeRef = useRef<number>(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const silenceStartRef = useRef<number>(0);
  const lastAudioLevelRef = useRef<number>(0);

  useEffect(() => {
    // 设置窗口属性
    const setupWindow = async () => {
      try {
        // 在显示窗口之前，先保存当前活动的应用
        try {
          const activeApp = await invoke<ActiveAppInfo>('get_active_app_info_for_voice');
          setOriginalApp(activeApp);
          console.log('保存原始应用:', activeApp);
        } catch (e) {
          console.error('获取活动应用信息失败:', e);
        }

        // 设置窗口始终在最前
        await appWindow.setAlwaysOnTop(true);
        // 设置窗口装饰（无标题栏）
        await appWindow.setDecorations(false);
        // 设置窗口大小
        await appWindow.setSize(new LogicalSize(400, 150));
        
        // 获取屏幕尺寸并居中显示
        const screenWidth = window.screen.width;
        const screenHeight = window.screen.height;
        // 固定在屏幕中间位置
        const x = Math.floor(screenWidth / 2 - 200);
        const y = Math.floor(screenHeight / 2 - 75);
        await appWindow.setPosition(new LogicalPosition(x, y));
      } catch (error) {
        console.error('设置窗口属性失败:', error);
      }
    };

    // 监听从后端发送的原始应用信息
    const unlistenAppInfo = listen<ActiveAppInfo>('voice_input_triggered', (event) => {
      console.log('接收到原始应用信息:', event.payload);
      setOriginalApp(event.payload);
    });

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
      unlistenAppInfo.then(fn => fn());
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
      
      // 启动录音，使用默认设备
      await invoke('start_recording', {
        deviceId: "default"  // 使用默认设备而不是null
      });

      // 启动计时器和音频电平监控
      timerRef.current = window.setInterval(async () => {
        const duration = (Date.now() - startTimeRef.current) / 1000;
        setRecordingDuration(duration);
        
        // 获取实际音频电平
        let currentLevel = 0;
        try {
          currentLevel = await invoke<number>('get_audio_level');
          setAudioLevel(Math.min(1.0, currentLevel));
        } catch {
          // 无法获取电平时使用安全回退
          currentLevel = 0;
          setAudioLevel(0);
        }
        
        // 静音检测（VAD - Voice Activity Detection）
        const SILENCE_THRESHOLD = 0.02; // 静音阈值
        const SILENCE_DURATION = 2000; // 2秒静音后自动停止
        
        if (currentLevel < SILENCE_THRESHOLD) {
          if (silenceStartRef.current === 0) {
            silenceStartRef.current = Date.now();
          } else if (Date.now() - silenceStartRef.current > SILENCE_DURATION) {
            // 检测到持续静音，自动停止录音
            console.log('检测到静音，自动停止录音');
            stopRecording();
          }
        } else {
          // 检测到声音，重置静音计时器
        }
        
        lastAudioLevelRef.current = currentLevel;
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
      const result = await invoke<string>('stop_recording_and_transcribe', {
        model: 'luyingwang-online'
      });

      setIsTranscribing(false);
      setTranscriptionText(result);

      // 自动插入文本到原始应用
      if (result) {
        try {
          console.log('准备注入文本，原始应用:', originalApp);
          
          // 1. 先隐藏窗口
          await appWindow.hide();
          console.log('窗口已隐藏');
          
          // 2. 等待窗口完全隐藏
          await new Promise(resolve => setTimeout(resolve, 300));
          
          // 3. 激活原始应用（如果有）
          if (originalApp && originalApp.bundle_id) {
            console.log('尝试激活原始应用:', originalApp.bundle_id);
            try {
              await invoke('activate_app_by_bundle_id', { bundleId: originalApp.bundle_id });
              console.log('原始应用已激活');
              // 等待应用完全获得焦点
              await new Promise(resolve => setTimeout(resolve, 500));
            } catch (e) {
              console.error('激活原始应用失败:', e);
              // 即使失败也继续尝试注入
            }
          } else {
            console.log('没有原始应用信息，等待系统自动恢复焦点');
            // 给系统更多时间恢复焦点
            await new Promise(resolve => setTimeout(resolve, 500));
          }
          
          // 4. 注入文本
          console.log('开始注入文本:', result);
          await invoke('inject_text_to_active_app', { 
            text: result, 
            targetBundleId: originalApp?.bundle_id 
          });
          console.log('✅ 文本注入成功');
          
          // 5. 调用回调（如果有）
          if (onTextReady) {
            onTextReady(result);
          }
          
          // 6. 延迟关闭窗口
          setTimeout(() => {
            handleClose();
          }, 300);
        } catch (error) {
          console.error('❌ 文本注入失败:', error);
          setError(`插入文本失败: ${error}`);
          
          // 重新显示窗口以便用户看到错误和转录结果
          await appWindow.show();
          
          // 提供手动复制选项
          if (navigator.clipboard) {
            try {
              await navigator.clipboard.writeText(result);
              setError(`文本已复制到剪贴板，请手动粘贴: ${error}`);
            } catch (clipErr) {
              console.error('复制到剪贴板也失败:', clipErr);
            }
          }
        }
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
              {error && <div className="error-text">{error}</div>}
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
        {originalApp && <span className="app-info"> · 目标: {originalApp.name}</span>}
      </div>
    </div>
  );
};

export default QuickVoiceInput;
