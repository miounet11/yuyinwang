/**
 * 转录文件页面
 * 完美复刻 Spokenly 第三张截图的设计
 */

import React, { useState, useCallback } from 'react';
import './FileTranscription.css';
import { SpokenlyCard, SpokenlyButton, SpokenlyTag } from '../ui';

interface FileTranscriptionProps {}

const FileTranscription: React.FC<FileTranscriptionProps> = () => {
  const [isDragging, setIsDragging] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [currentModel, setCurrentModel] = useState('Online Whisper v3 Turbo');

  const supportedFormats = [
    { extension: 'MP3', color: '#FF6B6B' },
    { extension: 'WAV', color: '#4ECDC4' },
    { extension: 'M4A', color: '#45B7D1' },
    { extension: 'FLAC', color: '#96CEB4' },
    { extension: 'MP4', color: '#FECA57' },
    { extension: 'MOV', color: '#FF9FF3' },
    { extension: 'M4V', color: '#54A0FF' }
  ];

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    
    const files = Array.from(e.dataTransfer.files);
    const validFiles = files.filter(file => {
      const ext = file.name.split('.').pop()?.toLowerCase();
      return supportedFormats.some(format => format.extension.toLowerCase() === ext);
    });
    
    setSelectedFiles(validFiles);
  }, []);

  const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    setSelectedFiles(files);
  }, []);

  return (
    <div className="spokenly-page">
      <div className="spokenly-page-header">
        <h1>转录文件</h1>
        <p>将音频或视频文件转换为文本，Spokenly 将为您进行转录。</p>
      </div>

      <div className="spokenly-page-content">
        {/* 文件上传区域 */}
        <div className="upload-section">
          <div
            className={`upload-area ${isDragging ? 'dragging' : ''}`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            onClick={() => document.getElementById('file-input')?.click()}
          >
            <div className="upload-content">
              <div className="upload-icon">
                <svg width="64" height="64" viewBox="0 0 64 64" fill="none">
                  <path
                    d="M20 24L32 12L44 24M32 12V40M8 40V52C8 54.2091 9.79086 56 12 56H52C54.2091 56 56 54.2091 56 52V40"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              </div>
              <h3>将文件拖放到此处</h3>
              <p>或点击选择文件</p>
              
              {/* 支持格式标签 */}
              <div className="format-tags">
                {supportedFormats.map((format) => (
                  <SpokenlyTag 
                    key={format.extension}
                    variant="secondary"
                    style={{ backgroundColor: format.color, color: 'white' }}
                  >
                    {format.extension}
                  </SpokenlyTag>
                ))}
              </div>
            </div>
          </div>

          <input
            id="file-input"
            type="file"
            multiple
            accept={supportedFormats.map(f => `.${f.extension.toLowerCase()}`).join(',')}
            onChange={handleFileSelect}
            style={{ display: 'none' }}
          />
        </div>

        {/* 当前模型显示 */}
        <div className="current-model-section">
          <span className="model-label">当前使用模型:</span>
          <span className="model-name">{currentModel}</span>
        </div>

        {/* 已选择文件列表 */}
        {selectedFiles.length > 0 && (
          <SpokenlyCard>
            <h3>选择的文件</h3>
            <div className="selected-files">
              {selectedFiles.map((file, index) => (
                <div key={index} className="file-item">
                  <div className="file-info">
                    <span className="file-name">{file.name}</span>
                    <span className="file-size">
                      {(file.size / 1024 / 1024).toFixed(1)} MB
                    </span>
                  </div>
                  <button
                    className="remove-file"
                    onClick={() => {
                      setSelectedFiles(files => files.filter((_, i) => i !== index));
                    }}
                  >
                    ×
                  </button>
                </div>
              ))}
            </div>
          </SpokenlyCard>
        )}

        {/* 底部操作按钮 */}
        <div className="action-buttons">
          <SpokenlyButton variant="secondary" size="md">
            🎤 录制音频
          </SpokenlyButton>
          
          <SpokenlyButton variant="secondary" size="md">
            🔄 更换模型
          </SpokenlyButton>
          
          <SpokenlyButton variant="secondary" size="md">
            ⚙️ 本地 Whisper 设置
          </SpokenlyButton>
          
          <SpokenlyButton variant="secondary" size="md">
            📥 导入 Spokenly 项目
          </SpokenlyButton>
        </div>

        {/* 开始转录按钮 */}
        {selectedFiles.length > 0 && (
          <div className="transcribe-section">
            <SpokenlyButton 
              variant="primary" 
              size="lg" 
              className="transcribe-button"
            >
              开始转录 ({selectedFiles.length} 个文件)
            </SpokenlyButton>
          </div>
        )}
      </div>
    </div>
  );
};

export default FileTranscription;