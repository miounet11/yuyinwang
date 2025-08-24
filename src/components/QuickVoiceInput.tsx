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
  const [isPartial, setIsPartial] = useState(false);
  const [usingStreaming, setUsingStreaming] = useState(true);
  
  const timerRef = useRef<number | null>(null);
  const startTimeRef = useRef<number>(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const silenceStartRef = useRef<number>(0);
  const lastAudioLevelRef = useRef<number>(0);

  useEffect(() => {
    const setupWindow = async () => {
      try {
        try {
          const activeApp = await invoke<ActiveAppInfo>('get_active_app_info_for_voice');
          setOriginalApp(activeApp);
        } catch (e) {}
        await appWindow.setAlwaysOnTop(true);
        await appWindow.setDecorations(false);
        await appWindow.setSize(new LogicalSize(400, 150));
        const screenWidth = window.screen.width;
        const screenHeight = window.screen.height;
        const x = Math.floor(screenWidth / 2 - 200);
        const y = Math.floor(screenHeight / 2 - 75);
        await appWindow.setPosition(new LogicalPosition(x, y));
      } catch {}
    };

    const unlistenAppInfo = listen<ActiveAppInfo>('voice_input_triggered', (event) => {
      setOriginalApp(event.payload);
    });

    setupWindow();
    
    // 自动开始录音（使用渐进式流）
    startRecording();

    // 停止录音：快捷键松开
    const unlistenKeyRelease = listen('quick_voice_key_released', () => {
      if (isRecording) {
        stopRecording();
      }
    });

    // 监听流式事件
    const unlistenStreaming = listen<any>('streaming_transcription', (event) => {
      const payload: any = event.payload || {};
      const text: string = payload.text || '';
      const partial: boolean = !!payload.is_partial;
      setTranscriptionText(text);
      setIsPartial(partial);
      if (text) setIsTranscribing(false);
    });
    const unlistenFinal = listen<string>('final_transcription', (event) => {
      const finalText = (event.payload || '').toString();
      if (finalText) {
        setTranscriptionText(finalText);
        setIsPartial(false);
      }
    });
    const unlistenComplete = listen<any>('progressive_voice_input_complete', async () => {
      // 渐进式注入已完成，关闭窗口
      try {
        await appWindow.hide();
      } catch {}
      if (onClose) onClose(); else appWindow.close();
    });

    // ESC 关闭
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleClose();
      }
    };
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      unlistenAppInfo.then(fn => fn());
      unlistenKeyRelease.then(fn => fn());
      unlistenStreaming.then(fn => fn());
      unlistenFinal.then(fn => fn());
      unlistenComplete.then(fn => fn());
      document.removeEventListener('keydown', handleKeyDown);
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
      if (isRecording) {
        invoke('stop_voice_recording').catch(() => {});
      }
    };
  }, [isRecording]);

  const startRecording = async () => {
    try {
      // 停止现有录音
      try { await invoke('stop_voice_recording'); } catch {}
      setError('');
      setIsRecording(true);
      setTranscriptionText('');
      setIsPartial(false);
      setUsingStreaming(true);
      startTimeRef.current = Date.now();

      // 启动渐进式语音输入（仅最终注入，避免重复注入）
      await invoke('start_progressive_voice_input', { targetBundleId: null, enableRealTimeInjection: false });

      // 计时器与电平（尽量使用真实API，失败回退0）
      timerRef.current = window.setInterval(async () => {
        const duration = (Date.now() - startTimeRef.current) / 1000;
        setRecordingDuration(duration);
        let currentLevel = 0;
        try {
          currentLevel = await invoke<number>('get_audio_level');
          setAudioLevel(Math.min(1.0, currentLevel));
        } catch { setAudioLevel(0); }
        lastAudioLevelRef.current = currentLevel;
      }, 100);
    } catch (error) {
      setError(`录音失败: ${error}`);
      setIsRecording(false);
    }
  };

  const stopRecording = async () => {
    try {
      if (timerRef.current) { clearInterval(timerRef.current); timerRef.current = null; }
      setIsRecording(false);
      setIsTranscribing(true);
      setAudioLevel(0);
      // 使用渐进式停止，不做本地注入，避免重复
      await invoke('stop_voice_recording');
    } catch (error) {
      setError(`停止录音失败: ${error}`);
      setIsTranscribing(false);
    }
  };

  const handleClose = () => {
    if (isRecording) {
      invoke('stop_voice_recording').catch(() => {});
    }
    if (onClose) onClose(); else appWindow.close();
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="quick-voice-input" ref={containerRef}>
      <div className="voice-input-container">
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
              <span className={`result-text ${isPartial ? 'partial' : 'final'}`}>{transcriptionText}</span>
              {error && <div className="error-text">{error}</div>}
            </div>
          ) : error ? (
            <div className="error-info">
              <span className="error-text">{error}</span>
            </div>
          ) : null}
        </div>

        <button className="close-btn" onClick={handleClose} title="关闭 (ESC)">×</button>
      </div>

      <div className="shortcut-hint">
        <kbd>ESC</kbd> 取消 · <kbd>按住快捷键</kbd> 录音
        {originalApp && <span className="app-info"> · 目标: {originalApp.name}</span>}
      </div>
    </div>
  );
};

export default QuickVoiceInput;
