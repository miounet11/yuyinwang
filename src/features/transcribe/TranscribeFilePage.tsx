import React, { useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { useAppStore } from '../../shared/stores/useAppStore';
import './TranscribeFilePage.css';

const SUPPORTED_FORMATS = ['.mp3', '.wav', '.m4a', '.flac', '.mp4', '.mov', '.m4v', '.webm', '.ogg'];

interface TranscribeResult {
  text: string;
  duration?: number;
}

export const TranscribeFilePage: React.FC = () => {
  const { addToast, addHistoryEntry, settings } = useAppStore();
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileName, setFileName] = useState('');
  const [fileSize, setFileSize] = useState('');
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [progress, setProgress] = useState(0);
  const [result, setResult] = useState('');
  const [isDragOver, setIsDragOver] = useState(false);
  const [elapsedTime, setElapsedTime] = useState(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const handleSelectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: '音频/视频文件',
          extensions: ['mp3', 'wav', 'm4a', 'flac', 'mp4', 'mov', 'm4v', 'webm', 'ogg'],
        }],
      });
      if (selected && typeof selected === 'string') {
        setSelectedFile(selected);
        const name = selected.split('/').pop() || selected;
        setFileName(name);
        setResult('');
        setProgress(0);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleTranscribe = async () => {
    if (!selectedFile) return;
    setIsTranscribing(true);
    setProgress(0);
    setResult('');
    setElapsedTime(0);

    timerRef.current = setInterval(() => {
      setElapsedTime(t => t + 1);
    }, 1000);

    const progressTimer = setInterval(() => {
      setProgress(p => Math.min(p + 1.5, 90));
    }, 500);

    try {
      const res = await invoke<TranscribeResult>('transcribe_file', {
        filePath: selectedFile,
        model: settings.selected_model,
      });
      clearInterval(progressTimer);
      setProgress(100);
      setResult(res.text);

      addHistoryEntry({
        id: Date.now().toString(),
        text: res.text,
        timestamp: Date.now(),
        duration: res.duration || 0,
        model: settings.selected_model,
        confidence: 0.95,
        audio_file_path: selectedFile,
      });

      addToast('success', '转录完成');
    } catch (e) {
      clearInterval(progressTimer);
      setProgress(0);
      setResult(`转录失败: ${e}`);
      addToast('error', `转录失败: ${e}`);
    } finally {
      setIsTranscribing(false);
      if (timerRef.current) clearInterval(timerRef.current);
    }
  };

  const handleCopy = () => {
    if (result && !result.startsWith('转录失败')) {
      navigator.clipboard.writeText(result);
      addToast('success', '已复制到剪贴板');
    }
  };

  const handleInject = async () => {
    if (result && !result.startsWith('转录失败')) {
      try {
        await invoke('inject_text', { text: result, delayMs: settings.inject_delay_ms });
        addToast('success', '已注入到当前应用');
      } catch (e) {
        addToast('error', `注入失败: ${e}`);
      }
    }
  };

  const handleClear = () => {
    setSelectedFile(null);
    setFileName('');
    setFileSize('');
    setResult('');
    setProgress(0);
  };

  const formatElapsed = (s: number) => {
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return m > 0 ? `${m}分${sec}秒` : `${sec}秒`;
  };

  const getFileIcon = (name: string) => {
    if (/\.(mp4|mov|m4v|webm)$/i.test(name)) return '🎬';
    return '🎵';
  };

  return (
    <div className="page">
      <h1 className="page-title">转录文件</h1>
      <p className="page-desc">上传音频或视频文件进行转录</p>

      <div className="section">
        <h2 className="section-title">选择文件</h2>
        <p className="section-desc">支持 MP3, WAV, M4A, FLAC, MP4, MOV 等格式</p>

        <div
          className={`drop-zone ${isDragOver ? 'drag-over' : ''} ${selectedFile ? 'has-file' : ''}`}
          onClick={!selectedFile ? handleSelectFile : undefined}
          onDragOver={(e) => { e.preventDefault(); setIsDragOver(true); }}
          onDragLeave={() => setIsDragOver(false)}
          onDrop={(e) => { e.preventDefault(); setIsDragOver(false); }}
        >
          {selectedFile ? (
            <div className="file-info">
              <span className="file-icon">{getFileIcon(fileName)}</span>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div className="file-name">{fileName}</div>
                {fileSize && <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>{fileSize}</div>}
              </div>
              <div style={{ display: 'flex', gap: '6px' }}>
                <button className="file-change" onClick={(e) => { e.stopPropagation(); handleSelectFile(); }}>
                  更换
                </button>
                <button className="file-change" onClick={(e) => { e.stopPropagation(); handleClear(); }}
                  style={{ color: 'var(--danger)' }}>
                  移除
                </button>
              </div>
            </div>
          ) : (
            <div className="drop-content">
              <span className="drop-icon">📁</span>
              <p className="drop-text">点击选择文件</p>
              <p className="drop-hint">或拖拽文件到这里</p>
            </div>
          )}
        </div>

        {selectedFile && !isTranscribing && !result && (
          <button className="transcribe-btn" onClick={handleTranscribe}>
            🎙 开始转录
          </button>
        )}

        {isTranscribing && (
          <div className="transcribe-progress">
            <div className="progress-bar-wrap">
              <div className="progress-bar-fill" style={{ width: `${progress}%` }} />
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span className="progress-text">转录中... {Math.round(progress)}%</span>
              <span className="progress-text">已用时 {formatElapsed(elapsedTime)}</span>
            </div>
          </div>
        )}
      </div>

      {result && (
        <div className="section">
          <div className="result-card">
            <div className="result-header">
              <h3>{result.startsWith('转录失败') ? '❌ 错误' : '✅ 转录结果'}</h3>
              {!result.startsWith('转录失败') && (
                <div style={{ display: 'flex', gap: '6px' }}>
                  <button className="result-copy" onClick={handleInject}>📝 注入</button>
                  <button className="result-copy" onClick={handleCopy}>📋 复制</button>
                </div>
              )}
            </div>
            <div className="result-body">{result}</div>
            {!result.startsWith('转录失败') && (
              <div style={{
                display: 'flex', gap: '8px', padding: '10px 16px',
                borderTop: '1px solid var(--border)', fontSize: '11px', color: 'var(--text-muted)',
              }}>
                <span>{result.length} 字</span>
                <span>•</span>
                <span>{fileName}</span>
              </div>
            )}
          </div>
        </div>
      )}

      <div className="section">
        <h2 className="section-title">支持的格式</h2>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
          {SUPPORTED_FORMATS.map(fmt => (
            <span key={fmt} style={{
              padding: '4px 10px', background: 'var(--bg-card)', border: '1px solid var(--border)',
              borderRadius: '6px', fontSize: '12px', color: 'var(--text-secondary)',
            }}>{fmt}</span>
          ))}
        </div>
      </div>
    </div>
  );
};
