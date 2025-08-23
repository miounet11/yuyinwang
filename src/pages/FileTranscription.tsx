/**
 * FileTranscription - 转录文件页面
 * 复刻第三张截图的设计：文件上传和转录界面
 */

import React, { useState, useCallback } from 'react';
import { motion } from 'framer-motion';
import { 
  SpokenlyContent,
  SpokenlyUploadArea,
  SpokenlyButton,
  SpokenlyTag
} from '../components/ui';
import { invoke } from '@tauri-apps/api/tauri';

// 支持的文件格式
const supportedFormats = [
  { extension: 'MP3', type: 'audio/mpeg' },
  { extension: 'WAV', type: 'audio/wav' },
  { extension: 'M4A', type: 'audio/m4a' },
  { extension: 'FLAC', type: 'audio/flac' },
  { extension: 'MP4', type: 'video/mp4' },
  { extension: 'MOV', type: 'video/quicktime' },
  { extension: 'M4V', type: 'video/x-m4v' }
];

// 上传的文件信息
interface UploadedFile {
  id: string;
  name: string;
  size: number;
  type: string;
  status: 'pending' | 'processing' | 'completed' | 'error';
  progress?: number;
  transcript?: string;
  duration?: number;
  path?: string;
  error?: string;
}

interface FileTranscriptionProps {
  className?: string;
}

export const FileTranscription: React.FC<FileTranscriptionProps> = ({
  className = ''
}) => {
  const [uploadedFiles, setUploadedFiles] = useState<UploadedFile[]>([]);
  const [isDragActive, setIsDragActive] = useState(false);

  const handleFilesDrop = useCallback((files: FileList) => {
    const newFiles: UploadedFile[] = Array.from(files).map(file => ({
      id: Math.random().toString(36).substr(2, 9),
      name: file.name,
      size: file.size,
      type: file.type,
      status: 'pending'
    }));
    setUploadedFiles(prev => [...prev, ...newFiles]);
    newFiles.forEach(runTranscription);
  }, []);

  const runTranscription = async (file: UploadedFile) => {
    try {
      setUploadedFiles(prev => prev.map(f => f.id === file.id ? { ...f, status: 'processing', progress: 0 } : f));
      // 暂不做分片上传；使用 tauri fs 路径需要先保存到临时文件，此处假设前端已可直接提供路径
      // 为兼容，简化为让用户通过拖拽来自文件系统（Tauri 会提供 path）
      // 如果没有 path，这里直接报错提示
      // 实际生产中可扩展为先写到临时目录，再调用后端
      const path = (file as any).path as string | undefined;
      if (!path) {
        setUploadedFiles(prev => prev.map(f => f.id === file.id ? { ...f, status: 'error', error: '无法获取文件路径，请从系统文件管理器拖拽' } : f));
        return;
      }
      const result = await invoke<{ text: string; confidence?: number }>('transcribe_file', { filePath: path, model: 'whisper-tiny' });
      setUploadedFiles(prev => prev.map(f => f.id === file.id ? { ...f, status: 'completed', progress: 100, transcript: result.text } : f));
    } catch (e: any) {
      setUploadedFiles(prev => prev.map(f => f.id === file.id ? { ...f, status: 'error', error: String(e) } : f));
    }
  };

  // 格式化文件大小
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // 格式化持续时间
  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const pageVariants = {
    initial: { opacity: 0, y: 20 },
    animate: { 
      opacity: 1, 
      y: 0,
      transition: {
        duration: 0.6,
        ease: [0.0, 0.0, 0.2, 1]
      }
    }
  };

  return (
    <SpokenlyContent className={className}>
      <motion.div
        className="file-transcription"
        variants={pageVariants}
        initial="initial"
        animate="animate"
        style={{
          width: '100%',
          maxWidth: '800px',
          margin: '0 auto'
        }}
      >
        {/* 页面标题 */}
        <div 
          className="page-header"
          style={{
            marginBottom: 'var(--spokenly-space-8)',
            paddingBottom: 'var(--spokenly-space-4)',
            borderBottom: '1px solid var(--spokenly-border-subtle)'
          }}
        >
          <h1 
            style={{
              fontSize: 'var(--spokenly-text-2xl)',
              fontWeight: 600,
              color: 'var(--spokenly-text-primary)',
              margin: 0,
              marginBottom: 'var(--spokenly-space-2)'
            }}
          >
            转录文件
          </h1>
          <p 
            style={{
              fontSize: 'var(--spokenly-text-sm)',
              color: 'var(--spokenly-text-secondary)',
              margin: 0
            }}
          >
            将音频或视频文件转换为文本
          </p>
        </div>

        {/* 文件上传区域 */}
        <div style={{ marginBottom: 'var(--spokenly-space-8)' }}>
          <SpokenlyUploadArea
            onFilesDrop={handleFilesDrop}
            onFilesSelect={handleFilesDrop}
            accept={supportedFormats.map(f => f.type).join(',')}
            multiple
            maxSize={100 * 1024 * 1024}
            title="将文件拖放到此处"
            description="或点击选择文件（请从系统文件管理器选择，确保有可访问路径）"
            style={{
              minHeight: '200px',
              border: isDragActive 
                ? '2px dashed var(--spokenly-primary-500)' 
                : '2px dashed var(--spokenly-border-default)',
              backgroundColor: isDragActive 
                ? 'var(--spokenly-primary-50)' 
                : 'var(--spokenly-bg-subtle)',
              transition: 'all 0.2s ease'
            }}
          />
          
          {/* 支持的格式标签 */}
          <div style={{ 
            marginTop: 'var(--spokenly-space-4)',
            display: 'flex',
            flexWrap: 'wrap',
            gap: 'var(--spokenly-space-2)',
            justifyContent: 'center'
          }}>
            <span style={{ 
              fontSize: 'var(--spokenly-text-xs)',
              color: 'var(--spokenly-text-tertiary)',
              marginRight: 'var(--spokenly-space-2)'
            }}>
              支持格式：
            </span>
            {supportedFormats.map(format => (
              <SpokenlyTag 
                key={format.extension}
                variant="default" 
                size="sm"
              >
                {format.extension}
              </SpokenlyTag>
            ))}
          </div>
        </div>

        {/* 上传文件列表 */}
        {uploadedFiles.length > 0 && (
          <div style={{ marginBottom: 'var(--spokenly-space-6)' }}>
            <h3 style={{ 
              fontSize: 'var(--spokenly-text-lg)',
              fontWeight: 600,
              color: 'var(--spokenly-text-primary)',
              margin: '0 0 var(--spokenly-space-4) 0'
            }}>
              处理中的文件
            </h3>
            
            <div style={{ 
              display: 'flex', 
              flexDirection: 'column', 
              gap: 'var(--spokenly-space-3)' 
            }}>
              {uploadedFiles.map(file => (
                <motion.div
                  key={file.id}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  style={{
                    padding: 'var(--spokenly-space-4)',
                    backgroundColor: 'var(--spokenly-bg-subtle)',
                    border: '1px solid var(--spokenly-border-default)',
                    borderRadius: 'var(--spokenly-radius-md)'
                  }}
                >
                  <div style={{ 
                    display: 'flex', 
                    justifyContent: 'space-between', 
                    alignItems: 'flex-start',
                    marginBottom: 'var(--spokenly-space-2)'
                  }}>
                    <div style={{ flex: 1 }}>
                      <h4 style={{ 
                        fontSize: 'var(--spokenly-text-sm)',
                        fontWeight: 500,
                        color: 'var(--spokenly-text-primary)',
                        margin: 0,
                        marginBottom: 'var(--spokenly-space-1)'
                      }}>
                        {file.name}
                      </h4>
                      <div style={{ 
                        fontSize: 'var(--spokenly-text-xs)',
                        color: 'var(--spokenly-text-tertiary)',
                        display: 'flex',
                        gap: 'var(--spokenly-space-2)'
                      }}>
                        <span>{formatFileSize(file.size)}</span>
                        {file.duration && <span>{formatDuration(file.duration)}</span>}
                      </div>
                    </div>
                    
                    <SpokenlyTag
                      variant={
                        file.status === 'completed' ? 'success' :
                        file.status === 'error' ? 'error' :
                        file.status === 'processing' ? 'warning' : 'default'
                      }
                      size="sm"
                    >
                      {file.status === 'pending' ? '等待中' :
                       file.status === 'processing' ? '处理中' :
                       file.status === 'completed' ? '已完成' : '错误'}
                    </SpokenlyTag>
                  </div>
                  
                  {file.status === 'processing' && (
                    <div style={{
                      fontSize: 'var(--spokenly-text-xs)',
                      color: 'var(--spokenly-text-tertiary)'
                    }}>
                      正在转录文件，请稍候…
                    </div>
                  )}
                  
                  {file.status === 'error' && file.error && (
                    <div style={{
                      marginTop: 'var(--spokenly-space-2)',
                      fontSize: 'var(--spokenly-text-xs)',
                      color: 'var(--spokenly-error-600, #b91c1c)'
                    }}>
                      {file.error}
                    </div>
                  )}
                  
                  {/* 转录结果 */}
                  {file.transcript && (
                    <div style={{
                      marginTop: 'var(--spokenly-space-2)',
                      padding: 'var(--spokenly-space-3)',
                      backgroundColor: 'var(--spokenly-bg-primary)',
                      border: '1px solid var(--spokenly-border-subtle)',
                      borderRadius: 'var(--spokenly-radius-sm)'
                    }}>
                      <p style={{
                        fontSize: 'var(--spokenly-text-sm)',
                        color: 'var(--spokenly-text-secondary)',
                        margin: 0,
                        lineHeight: 1.5
                      }}>
                        {file.transcript}
                      </p>
                    </div>
                  )}
                </motion.div>
              ))}
            </div>
          </div>
        )}

        {/* 底部操作按钮 */}
        <div style={{
          display: 'flex',
          flexWrap: 'wrap',
          gap: 'var(--spokenly-space-3)',
          justifyContent: 'center',
          padding: 'var(--spokenly-space-6)',
          backgroundColor: 'var(--spokenly-bg-subtle)',
          borderRadius: 'var(--spokenly-radius-md)',
          border: '1px solid var(--spokenly-border-subtle)'
        }}>
          <SpokenlyButton variant="secondary" size="md">
            录制音频
          </SpokenlyButton>
          
          <SpokenlyButton variant="secondary" size="md">
            更换模型
          </SpokenlyButton>
          
          <SpokenlyButton variant="secondary" size="md">
            本地 Whisper 设置
          </SpokenlyButton>
          
          <SpokenlyButton variant="ghost" size="md">
            导入 Spokenly 项目
          </SpokenlyButton>
        </div>
      </motion.div>
    </SpokenlyContent>
  );
};