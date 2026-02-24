import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../../shared/stores/useAppStore';
import './RecordingPage.css';

interface TranscriptionResult {
  text: string;
  language?: string;
  duration?: number;
}

export const RecordingPage: React.FC = () => {
  const {
    isRecording,
    transcriptionText,
    settings,
    setRecording,
    setTranscriptionText,
    addHistoryEntry,
    addToast,
  } = useAppStore();

  const [isTranscribing, setIsTranscribing] = useState(false);
  const [recordingTime, setRecordingTime] = useState(0);
  const [waveAmplitude, setWaveAmplitude] = useState(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const waveRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    // Listen for quick-input events to sync state
    const unlistens = [
      listen('quick-input-started', () => { setRecording(true); }),
      listen('quick-input-result', (e: any) => {
        setRecording(false);
        setTranscriptionText(e.payload);
      }),
    ];
    return () => { unlistens.forEach(u => u.then(fn => fn())); };
  }, []);

  useEffect(() => {
    if (isRecording) {
      setRecordingTime(0);
      timerRef.current = setInterval(() => setRecordingTime(t => t + 1), 1000);
      // Simulate waveform amplitude for visual feedback
      waveRef.current = setInterval(() => {
        setWaveAmplitude(0.3 + Math.random() * 0.7);
      }, 150);
    } else {
      if (timerRef.current) clearInterval(timerRef.current);
      if (waveRef.current) clearInterval(waveRef.current);
      setWaveAmplitude(0);
    }
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
      if (waveRef.current) clearInterval(waveRef.current);
    };
  }, [isRecording]);

  const formatTime = (seconds: number) => {
    const m = Math.floor(seconds / 60).toString().padStart(2, '0');
    const s = (seconds % 60).toString().padStart(2, '0');
    return `${m}:${s}`;
  };

  const handleToggleRecording = async () => {
    if (isRecording) {
      try {
        setIsTranscribing(true);
        setTranscriptionText('');
        const result = await invoke<TranscriptionResult>('stop_recording', {
          model: settings.selected_model,
        });
        setRecording(false);
        setIsTranscribing(false);
        setTranscriptionText(result.text);
        addHistoryEntry({
          id: Date.now().toString(),
          text: result.text,
          timestamp: Date.now(),
          duration: result.duration || recordingTime,
          model: settings.selected_model,
          confidence: 0.95,
        });
        if (settings.auto_inject && result.text) {
          await invoke('inject_text', { text: result.text, delayMs: settings.inject_delay_ms });
        }
        addToast('success', '转录完成');
      } catch (error) {
        setTranscriptionText(`转录失败: ${error}`);
        setIsTranscribing(false);
        setRecording(false);
        addToast('error', `转录失败: ${error}`);
      }
    } else {
      try {
        await invoke('start_recording');
        setRecording(true);
        setTranscriptionText('');
      } catch (error) {
        addToast('error', `录音失败: ${error}`);
      }
    }
  };

  const handleCopy = () => {
    if (transcriptionText) {
      navigator.clipboard.writeText(transcriptionText);
      addToast('success', '已复制到剪贴板');
    }
  };

  const handleInject = async () => {
    if (transcriptionText) {
      try {
        await invoke('inject_text', { text: transcriptionText, delayMs: settings.inject_delay_ms });
        addToast('success', '已注入到当前应用');
      } catch (e) {
        addToast('error', `注入失败: ${e}`);
      }
    }
  };

  const getModelLabel = (id: string) => {
    const names: Record<string, string> = {
      'luyin-free': 'LuYinWang (免费)',
      'gpt-4o-mini-transcribe': 'GPT-4o mini',
      'deepgram-nova3': 'Nova-3',
      'whisper-small': 'Whisper Small',
      'whisper-base': 'Whisper Base',
      'whisper-tiny': 'Whisper Tiny',
      'whisper-medium': 'Whisper Medium',
      'whisper-large-v3': 'Whisper Large v3',
      'whisper-large-v3-turbo': 'Large v3 Turbo',
      'whisper-medium-zh': 'Medium 中文',
    };
    return names[id] || id;
  };

  return (
    <div className="recording-page">
      <div className="rec-hero">
        <div className="rec-model-badge">
          <span className="rec-model-dot" />
          {getModelLabel(settings.selected_model)}
        </div>

        <div className={`rec-circle-wrap ${isRecording ? 'recording' : isTranscribing ? 'processing' : 'idle'}`}>
          {isRecording && (
            <>
              <div className="rec-ring ring-1" style={{ animationDuration: `${2.4 - waveAmplitude}s` }} />
              <div className="rec-ring ring-2" style={{ animationDuration: `${2.4 - waveAmplitude * 0.5}s` }} />
              <div className="rec-ring ring-3" />
            </>
          )}
          <button
            className={`rec-circle ${isRecording ? 'recording' : ''}`}
            onClick={handleToggleRecording}
            disabled={isTranscribing}
            aria-label={isRecording ? '停止录音' : isTranscribing ? '转录中' : '开始录音'}
          >
            {isTranscribing ? (
              <div className="rec-spinner" />
            ) : isRecording ? (
              <div className="rec-stop-icon" />
            ) : (
              <div className="rec-mic-icon">🎙</div>
            )}
          </button>
        </div>

        <div className="rec-status">
          {isRecording ? (
            <>
              <span className="rec-dot active" />
              <span className="rec-time">{formatTime(recordingTime)}</span>
              <span className="rec-label">正在录音</span>
            </>
          ) : isTranscribing ? (
            <>
              <span className="rec-dot processing" />
              <span className="rec-label">正在转录...</span>
            </>
          ) : (
            <span className="rec-label idle-label">点击开始录音</span>
          )}
        </div>

        <p className="rec-hint">
          💡 提示：使用快捷键可在任意应用中按住说话，松开自动转录注入
        </p>
      </div>

      {transcriptionText && (
        <div className="rec-result">
          <div className="result-header">
            <h3>转录结果</h3>
            <div style={{ display: 'flex', gap: '6px' }}>
              <button className="copy-btn" onClick={handleInject} title="注入到当前应用">📝 注入</button>
              <button className="copy-btn" onClick={handleCopy} title="复制到剪贴板">📋 复制</button>
            </div>
          </div>
          <div className="result-body">{transcriptionText}</div>
          <div className="result-footer">
            <span>{getModelLabel(settings.selected_model)}</span>
            <span>•</span>
            <span>{transcriptionText.length} 字</span>
          </div>
        </div>
      )}
    </div>
  );
};
