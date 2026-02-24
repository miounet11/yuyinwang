import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useAppStore } from '../../shared/stores/useAppStore';

export const RecordingPage: React.FC = () => {
  const { isTranscribing, addToast } = useAppStore();
  const [isRecording, setIsRecording] = React.useState(false);

  const handleStartRecording = async () => {
    try {
      await invoke('start_recording');
      setIsRecording(true);
      addToast('success', '录音已开始');
    } catch (error) {
      addToast('error', String(error));
    }
  };

  const handleStopRecording = async () => {
    try {
      await invoke('stop_recording');
      setIsRecording(false);
      addToast('success', '录音已停止');
    } catch (error) {
      addToast('error', String(error));
    }
  };

  return (
    <div style={{ padding: '20px' }}>
      <h1>录音</h1>
      <button
        onClick={isRecording ? handleStopRecording : handleStartRecording}
        disabled={isTranscribing}
        style={{
          padding: '12px 24px',
          fontSize: '16px',
          cursor: isTranscribing ? 'not-allowed' : 'pointer',
          opacity: isTranscribing ? 0.5 : 1,
        }}
      >
        {isRecording ? '停止录音' : '开始录音'}
      </button>
      {isTranscribing && <p>正在转录...</p>}
    </div>
  );
};
