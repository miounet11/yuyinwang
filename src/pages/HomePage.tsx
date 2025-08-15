import React, { useState, useCallback, memo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useStore } from '../stores/appStore';
import { getModelInfo } from '../utils/modelUtils';
import logger from '../utils/logger';

interface HomePageProps {
  onOpenAppSelector: () => void;
  onOpenShortcutEditor: () => void;
  onOpenPermissionSettings: () => void;
  onOpenTestPanel: () => void;
  onOpenTranscriptionDetail: (entry: any) => void;
}

const HomePage: React.FC<HomePageProps> = memo(({
  onOpenAppSelector,
  onOpenShortcutEditor,
  onOpenPermissionSettings,
  onOpenTestPanel,
  onOpenTranscriptionDetail
}) => {
  const {
    isRecording,
    transcriptionText,
    selectedModel,
    audioDevices,
    transcriptionHistory,
    setRecording,
    setTranscription,
    addTranscriptionEntry
  } = useStore();

  const [isTranscribing, setIsTranscribing] = useState(false);
  const [recordingDuration, setRecordingDuration] = useState(0);

  // 优化的录音处理函数 - 使用统一的录音逻辑
  const handleStartRecording = useCallback(async () => {
    // 使用主应用的统一录音逻辑，避免状态不同步
    if (window.appToggleRecording) {
      await window.appToggleRecording();
    } else {
      // 降级处理
      logger.audio('开始录音测试');
      try {
        await invoke('start_recording');
        setRecording(true);
        logger.audio('录音已开始');
      } catch (error) {
        logger.error('开始录音失败', error);
        alert('开始录音失败: ' + error);
      }
    }
  }, [setRecording]);

  const handleStopRecording = useCallback(async () => {
    // 使用统一的录音逻辑
    if (window.appToggleRecording) {
      await window.appToggleRecording();
    } else {
      // 降级处理
      logger.audio('停止录音测试');
      try {
        setRecording(false);
        setIsTranscribing(true);
        setTranscription('正在转录中，请稍候...');
        
        const currentModelId = selectedModel || 'gpt-4o-mini';
        const { model, modelType } = getModelInfo(currentModelId);
        const result = await invoke('stop_recording', { 
          model: model, 
          modelType: modelType 
        });
        
        logger.transcription('录音已停止，转录结果', result);
      } catch (error) {
        logger.error('停止录音失败', error);
        setTranscription(`停止录音失败: ${error}`);
        alert('停止录音失败: ' + error);
      } finally {
        setIsTranscribing(false);
      }
    }
  }, [selectedModel, setRecording, setTranscription]);

  const handleRecordingToggle = useCallback(async () => {
    if (!isRecording) {
      await handleStartRecording();
    } else {
      await handleStopRecording();
    }
  }, [isRecording, handleStartRecording, handleStopRecording]);

  return (
    <div className="home-page">
      <div className="main-settings-content">
        <h2>录音测试</h2>
        <div className="recording-test-container">
          
          {/* 当前模型信息 */}
          <div className="current-model-info">
            <div className="model-display">
              <span className="model-label">当前模型:</span>
              <span className="model-name">{selectedModel}</span>
              <span className={`model-type ${getModelInfo(selectedModel).modelType}`}>
                {getModelInfo(selectedModel).modelType === 'local' ? '本地' : '在线'}
              </span>
            </div>
          </div>

          {/* 录音控制区 */}
          <div className="recording-controls">
            <p className="recording-description">点击按钮测试麦克风录音和转录功能：</p>
            
            <button 
              className={`recording-button ${isRecording ? 'recording' : 'idle'}`}
              onClick={handleRecordingToggle}
              disabled={isTranscribing}
            >
              <div className="record-icon">
                {isRecording ? '■' : '●'}
              </div>
              <span className="record-text">
                {isRecording ? '停止录音' : '开始录音'}
              </span>
              {isRecording && (
                <div className="recording-pulse"></div>
              )}
            </button>

            {/* 录音状态指示器 */}
            <div className="recording-status">
              {isRecording && (
                <div className="status-indicator recording">
                  <span className="status-dot"></span>
                  <span>正在录音...</span>
                  <span className="duration">{recordingDuration}s</span>
                </div>
              )}
              {isTranscribing && (
                <div className="status-indicator processing">
                  <span className="spinner"></span>
                  <span>正在转录...</span>
                </div>
              )}
            </div>
          </div>

          {/* 转录结果显示区 */}
          <div className="transcription-result">
            <h3>转录结果:</h3>
            <div className="result-text-container">
              <textarea
                className="result-text"
                value={transcriptionText}
                readOnly
                placeholder="转录文本将在这里显示..."
              />
            </div>
          </div>
        </div>

        {/* 音频设备信息 */}
        <div className="audio-device-section">
          <h3>音频设备</h3>
          <div className="device-list">
            {audioDevices.length > 0 ? (
              audioDevices.map((device, index) => (
                <div key={index} className="device-item">
                  <span className="device-name">{device.name}</span>
                  {device.is_default && <span className="default-badge">默认</span>}
                  <span className={`status ${device.is_available ? 'available' : 'unavailable'}`}>
                    {device.is_available ? '可用' : '不可用'}
                  </span>
                </div>
              ))
            ) : (
              <div className="no-devices">
                <p>未检测到音频设备</p>
                <button onClick={onOpenPermissionSettings} className="permission-btn">
                  检查权限设置
                </button>
              </div>
            )}
          </div>
        </div>

        {/* 快速操作区 */}
        <div className="quick-actions">
          <h3>快速操作</h3>
          <div className="action-buttons">
            <button onClick={onOpenAppSelector} className="action-btn">
              <span className="btn-icon">📱</span>
              应用管理
            </button>
            <button onClick={onOpenShortcutEditor} className="action-btn">
              <span className="btn-icon">⌨️</span>
              快捷键设置
            </button>
            <button onClick={onOpenPermissionSettings} className="action-btn">
              <span className="btn-icon">🔒</span>
              权限设置
            </button>
            <button onClick={onOpenTestPanel} className="action-btn">
              <span className="btn-icon">🧪</span>
              功能测试
            </button>
          </div>
        </div>

        {/* 最近转录历史 */}
        <div className="recent-history">
          <h3>最近转录</h3>
          <div className="history-list">
            {transcriptionHistory.slice(0, 5).map((entry) => (
              <div 
                key={entry.id} 
                className="history-item" 
                onClick={() => onOpenTranscriptionDetail(entry)}
              >
                <div className="history-text">
                  {entry.text.substring(0, 60)}
                  {entry.text.length > 60 && '...'}
                </div>
                <div className="history-meta">
                  <span className="timestamp">
                    {new Date(entry.timestamp).toLocaleString()}
                  </span>
                  <span className="model-badge">{entry.model}</span>
                  <span className="confidence">
                    {Math.round(entry.confidence * 100)}%
                  </span>
                </div>
              </div>
            ))}
            {transcriptionHistory.length === 0 && (
              <div className="no-history">
                <p>暂无转录历史</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
});

HomePage.displayName = 'HomePage';

export default HomePage;