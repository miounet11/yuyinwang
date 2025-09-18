import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useStore } from '../App';
import './ProductionRecordingControls.css';

interface AudioLevel {
  peak: number;
  rms: number;
}

interface RecordingState {
  isRecording: boolean;
  isPaused: boolean;
  duration: number;
  audioLevel: AudioLevel;
  error: string | null;
}

const ProductionRecordingControls: React.FC = () => {
  const {
    isRecording,
    setRecording,
    transcriptionText,
    setTranscription,
    selectedModel,
    hasAllPermissions
  } = useStore();

  const [recordingState, setRecordingState] = useState<RecordingState>({
    isRecording: false,
    isPaused: false,
    duration: 0,
    audioLevel: { peak: 0, rms: 0 },
    error: null
  });

  const [isProcessing, setIsProcessing] = useState(false);
  const [recordingQuality, setRecordingQuality] = useState<'good' | 'fair' | 'poor'>('good');
  const durationIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const audioLevelIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // 同步录制状态
  useEffect(() => {
    setRecordingState(prev => ({
      ...prev,
      isRecording
    }));
  }, [isRecording]);

  // 录制时长计时器
  useEffect(() => {
    if (recordingState.isRecording && !recordingState.isPaused) {
      durationIntervalRef.current = setInterval(() => {
        setRecordingState(prev => ({
          ...prev,
          duration: prev.duration + 1
        }));
      }, 1000);
    } else {
      if (durationIntervalRef.current) {
        clearInterval(durationIntervalRef.current);
        durationIntervalRef.current = null;
      }
    }

    return () => {
      if (durationIntervalRef.current) {
        clearInterval(durationIntervalRef.current);
      }
    };
  }, [recordingState.isRecording, recordingState.isPaused]);

  // 音频级别监控
  useEffect(() => {
    if (recordingState.isRecording) {
      audioLevelIntervalRef.current = setInterval(async () => {
        try {
          const level = await invoke<AudioLevel>('get_audio_level');
          setRecordingState(prev => ({
            ...prev,
            audioLevel: level
          }));

          // 根据音频级别判断录制质量
          if (level.rms > 0.7) {
            setRecordingQuality('good');
          } else if (level.rms > 0.3) {
            setRecordingQuality('fair');
          } else {
            setRecordingQuality('poor');
          }
        } catch (error) {
          console.error('获取音频级别失败:', error);
        }
      }, 100);
    } else {
      if (audioLevelIntervalRef.current) {
        clearInterval(audioLevelIntervalRef.current);
        audioLevelIntervalRef.current = null;
      }
    }

    return () => {
      if (audioLevelIntervalRef.current) {
        clearInterval(audioLevelIntervalRef.current);
      }
    };
  }, [recordingState.isRecording]);

  // 开始录制
  const handleStartRecording = async () => {
    if (!hasAllPermissions) {
      setRecordingState(prev => ({
        ...prev,
        error: '请先授予必要的权限'
      }));
      return;
    }

    try {
      setRecordingState(prev => ({
        ...prev,
        error: null,
        duration: 0
      }));

      await invoke('start_recording');
      setRecording(true);
      setTranscription(''); // 清空之前的转录文本
    } catch (error) {
      console.error('开始录制失败:', error);
      setRecordingState(prev => ({
        ...prev,
        error: `录制失败: ${error}`
      }));
    }
  };

  // 暂停/恢复录制
  const handlePauseResume = async () => {
    try {
      if (recordingState.isPaused) {
        await invoke('resume_recording');
        setRecordingState(prev => ({
          ...prev,
          isPaused: false
        }));
      } else {
        await invoke('pause_recording');
        setRecordingState(prev => ({
          ...prev,
          isPaused: true
        }));
      }
    } catch (error) {
      console.error('暂停/恢复录制失败:', error);
      setRecordingState(prev => ({
        ...prev,
        error: `操作失败: ${error}`
      }));
    }
  };

  // 停止录制
  const handleStopRecording = async () => {
    try {
      setIsProcessing(true);
      await invoke('stop_recording');
      setRecording(false);

      setRecordingState(prev => ({
        ...prev,
        isRecording: false,
        isPaused: false,
        duration: 0,
        audioLevel: { peak: 0, rms: 0 }
      }));
    } catch (error) {
      console.error('停止录制失败:', error);
      setRecordingState(prev => ({
        ...prev,
        error: `停止录制失败: ${error}`
      }));
    } finally {
      setIsProcessing(false);
    }
  };

  // 格式化时长显示
  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // 获取录制质量显示文本和颜色
  const getQualityInfo = () => {
    switch (recordingQuality) {
      case 'good':
        return { text: '音质良好', color: '#10b981' };
      case 'fair':
        return { text: '音质一般', color: '#f59e0b' };
      case 'poor':
        return { text: '音质较差', color: '#ef4444' };
      default:
        return { text: '检测中', color: '#6b7280' };
    }
  };

  const qualityInfo = getQualityInfo();

  return (
    <div className="production-recording-controls">
      {/* 主录制按钮 */}
      <div className="main-control-section">
        <div className="record-button-container">
          {!recordingState.isRecording ? (
            <button
              className="record-button start"
              onClick={handleStartRecording}
              disabled={!hasAllPermissions || isProcessing}
              title={!hasAllPermissions ? '需要授予权限' : '开始录制'}
            >
              <div className="button-inner">
                <span className="record-icon">🎙️</span>
                <span className="button-text">开始录制</span>
              </div>
            </button>
          ) : (
            <div className="recording-controls-group">
              <button
                className={`record-button ${recordingState.isPaused ? 'resume' : 'pause'}`}
                onClick={handlePauseResume}
                title={recordingState.isPaused ? '恢复录制' : '暂停录制'}
              >
                <span className="button-icon">
                  {recordingState.isPaused ? '▶️' : '⏸️'}
                </span>
                <span className="button-text">
                  {recordingState.isPaused ? '恢复' : '暂停'}
                </span>
              </button>

              <button
                className="record-button stop"
                onClick={handleStopRecording}
                disabled={isProcessing}
                title="停止录制"
              >
                <span className="button-icon">⏹️</span>
                <span className="button-text">停止</span>
              </button>
            </div>
          )}
        </div>

        {/* 录制状态指示器 */}
        {recordingState.isRecording && (
          <div className="recording-status">
            <div className="recording-indicator">
              <span className="recording-dot pulsing"></span>
              <span className="status-text">
                {recordingState.isPaused ? '已暂停' : '录制中'}
              </span>
            </div>

            <div className="recording-duration">
              {formatDuration(recordingState.duration)}
            </div>
          </div>
        )}
      </div>

      {/* 音频监控面板 */}
      {recordingState.isRecording && (
        <div className="audio-monitor-panel">
          <div className="audio-level-container">
            <label className="monitor-label">音频级别</label>
            <div className="audio-level-bar">
              <div
                className="level-fill"
                style={{
                  width: `${Math.min(recordingState.audioLevel.rms * 100, 100)}%`,
                  backgroundColor: qualityInfo.color
                }}
              />
              <div
                className="peak-indicator"
                style={{
                  left: `${Math.min(recordingState.audioLevel.peak * 100, 100)}%`
                }}
              />
            </div>
            <div
              className="quality-indicator"
              style={{ color: qualityInfo.color }}
            >
              {qualityInfo.text}
            </div>
          </div>
        </div>
      )}

      {/* 录制信息面板 */}
      <div className="recording-info-panel">
        <div className="info-item">
          <span className="info-label">当前模型:</span>
          <span className="info-value">{selectedModel || '默认模型'}</span>
        </div>

        {transcriptionText && (
          <div className="info-item">
            <span className="info-label">已转录:</span>
            <span className="info-value">{transcriptionText.length} 字符</span>
          </div>
        )}

        {isProcessing && (
          <div className="info-item processing">
            <span className="processing-spinner">⏳</span>
            <span className="info-value">处理中...</span>
          </div>
        )}
      </div>

      {/* 错误提示 */}
      {recordingState.error && (
        <div className="error-panel">
          <span className="error-icon">⚠️</span>
          <span className="error-message">{recordingState.error}</span>
          <button
            className="dismiss-error"
            onClick={() => setRecordingState(prev => ({ ...prev, error: null }))}
          >
            ✕
          </button>
        </div>
      )}

      {/* 快捷键提示 */}
      <div className="shortcut-hints">
        <div className="hint-item">
          <kbd>Cmd/Ctrl + Shift + Space</kbd>
          <span>快速录制</span>
        </div>
      </div>
    </div>
  );
};

export default ProductionRecordingControls;
