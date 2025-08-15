import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useRecordingStore } from '../stores/useRecordingStore';
import { useTranscriptionStore } from '../stores/useTranscriptionStore';
import { useDeviceStore } from '../stores/useDeviceStore';
import './RecordingControls.css';

const RecordingControls: React.FC = () => {
  const { isRecording, setIsRecording, resetRecording } = useRecordingStore();
  const { selectedModel, setTranscription } = useTranscriptionStore();
  const { selectedDevice } = useDeviceStore();

  const handleToggleRecording = async () => {
    try {
      if (!isRecording) {
        // 开始录音
        await invoke('start_recording', { 
          deviceId: selectedDevice 
        });
        setIsRecording(true);
      } else {
        // 停止录音
        const result = await invoke<string>('stop_recording', {
          model: selectedModel
        });
        setIsRecording(false);
        resetRecording();
        
        if (result) {
          setTranscription(result);
        }
      }
    } catch (error) {
      console.error('录音操作失败:', error);
      setIsRecording(false);
      resetRecording();
    }
  };

  return (
    <div className="recording-controls">
      <button 
        className={`record-button ${isRecording ? 'recording' : ''}`}
        onClick={handleToggleRecording}
      >
        {isRecording ? '⏹ 停止录音' : '🎤 开始录音'}
      </button>
    </div>
  );
};

export default RecordingControls;