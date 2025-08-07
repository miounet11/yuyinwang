import React, { useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { ttsService } from '../services/ttsService';
import './ModelTestDialog.css';

interface ModelTestDialogProps {
  isVisible: boolean;
  modelId: string;
  modelName: string;
  onClose: () => void;
}

const ModelTestDialog: React.FC<ModelTestDialogProps> = ({
  isVisible,
  modelId,
  modelName,
  onClose
}) => {
  const [isRecording, setIsRecording] = useState(false);
  const [testText, setTestText] = useState('');
  const [transcriptionResult, setTranscriptionResult] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [testMode, setTestMode] = useState<'record' | 'text'>('record');
  const audioRef = useRef<HTMLAudioElement>(null);

  // 开始录音测试
  const startRecordingTest = async () => {
    try {
      setIsRecording(true);
      setTranscriptionResult('');
      await invoke('start_recording');
      
      // 5秒后自动停止
      setTimeout(async () => {
        await stopRecordingTest();
      }, 5000);
    } catch (error) {
      console.error('开始录音失败:', error);
      setIsRecording(false);
    }
  };

  // 停止录音并获取转录结果
  const stopRecordingTest = async () => {
    try {
      setIsRecording(false);
      setIsProcessing(true);
      
      // 停止录音并获取音频数据
      const audioData = await invoke<string>('stop_recording');
      
      // 使用选定的模型进行转录
      const result = await invoke<string>('transcribe_with_model', {
        modelId,
        audioData
      });
      
      setTranscriptionResult(result);
    } catch (error) {
      console.error('停止录音失败:', error);
      setTranscriptionResult(`测试失败: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  };

  // 测试文本转语音
  const testTextToSpeech = async () => {
    if (!testText.trim()) {
      alert('请输入测试文本');
      return;
    }

    try {
      setIsProcessing(true);
      
      // 使用 TTS 服务将文本转换为语音
      const audioBuffer = await ttsService.textToSpeech(testText, {
        model: 'tts-1',
        voice: 'alloy'
      });
      
      // 创建音频 URL 并播放
      const blob = new Blob([audioBuffer], { type: 'audio/mp3' });
      const audioUrl = URL.createObjectURL(blob);
      
      if (audioRef.current) {
        audioRef.current.src = audioUrl;
        audioRef.current.play();
      }
      
      setTranscriptionResult('语音合成成功，正在播放...');
    } catch (error) {
      console.error('TTS 测试失败:', error);
      setTranscriptionResult(`TTS 测试失败: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  };

  if (!isVisible) return null;

  return (
    <div className="model-test-overlay" onClick={onClose}>
      <div className="model-test-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="test-header">
          <h2>测试 {modelName} 模型</h2>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        <div className="test-tabs">
          <button 
            className={`test-tab ${testMode === 'record' ? 'active' : ''}`}
            onClick={() => setTestMode('record')}
          >
            🎤 录音测试
          </button>
          <button 
            className={`test-tab ${testMode === 'text' ? 'active' : ''}`}
            onClick={() => setTestMode('text')}
          >
            📝 文本测试
          </button>
        </div>

        <div className="test-content">
          {testMode === 'record' ? (
            <div className="record-test">
              <div className="test-instructions">
                <p>点击按钮开始录音，说出一段话来测试转录准确性</p>
                <p className="hint">录音将在5秒后自动停止</p>
              </div>

              <div className="record-controls">
                {!isRecording ? (
                  <button 
                    className="record-btn"
                    onClick={startRecordingTest}
                    disabled={isProcessing}
                  >
                    <span className="record-icon">🎙️</span>
                    <span>{isProcessing ? '处理中...' : '开始录音'}</span>
                  </button>
                ) : (
                  <button 
                    className="record-btn recording"
                    onClick={stopRecordingTest}
                  >
                    <span className="record-icon pulse">🔴</span>
                    <span>正在录音...</span>
                  </button>
                )}
              </div>

              {transcriptionResult && (
                <div className="test-result">
                  <h3>转录结果：</h3>
                  <div className="result-text">{transcriptionResult}</div>
                </div>
              )}
            </div>
          ) : (
            <div className="text-test">
              <div className="test-instructions">
                <p>输入文本测试语音合成功能</p>
              </div>

              <textarea
                className="test-input"
                placeholder="输入要转换为语音的文本..."
                value={testText}
                onChange={(e) => setTestText(e.target.value)}
                rows={4}
              />

              <button 
                className="test-btn"
                onClick={testTextToSpeech}
                disabled={isProcessing || !testText.trim()}
              >
                {isProcessing ? '处理中...' : '🔊 转换为语音'}
              </button>

              {transcriptionResult && (
                <div className="test-result">
                  <p>{transcriptionResult}</p>
                </div>
              )}

              <audio ref={audioRef} controls style={{ display: 'none' }} />
            </div>
          )}
        </div>

        <div className="test-footer">
          <div className="model-info">
            <span className="info-label">当前模型：</span>
            <span className="info-value">{modelName}</span>
          </div>
          <button className="done-btn" onClick={onClose}>
            完成测试
          </button>
        </div>
      </div>
    </div>
  );
};

export default ModelTestDialog;