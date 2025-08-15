import React from 'react';
import { useTranscriptionStore } from '../stores/useTranscriptionStore';
import './TranscriptionDisplay.css';

const TranscriptionDisplay: React.FC = () => {
  const { transcriptionText, clearTranscription } = useTranscriptionStore();

  if (!transcriptionText) {
    return (
      <div className="transcription-display empty">
        <p>等待转录结果...</p>
      </div>
    );
  }

  return (
    <div className="transcription-display">
      <div className="transcription-header">
        <h3>转录结果</h3>
        <button 
          className="clear-button"
          onClick={clearTranscription}
        >
          清除
        </button>
      </div>
      <div className="transcription-content">
        <p>{transcriptionText}</p>
      </div>
    </div>
  );
};

export default TranscriptionDisplay;