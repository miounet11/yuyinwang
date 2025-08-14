import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './TranscriptionDetailView.css';

interface TranscriptionEntry {
  id: string;
  text: string;
  timestamp: number;
  duration: number;
  model: string;
  confidence: number;
  audio_file_path?: string;
}

interface TranscriptionDetailViewProps {
  entry: TranscriptionEntry | null;
  isVisible: boolean;
  onClose: () => void;
}

const TranscriptionDetailView: React.FC<TranscriptionDetailViewProps> = ({
  entry,
  isVisible,
  onClose
}) => {
  // 添加null检查
  if (!entry) {
    return null;
  }

  const [editedText, setEditedText] = useState(entry.text || '');
  const [isEditing, setIsEditing] = useState(false);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [audioDuration, setAudioDuration] = useState(0);
  const [volume, setVolume] = useState(0.8);
  const [isLoading, setIsLoading] = useState(false);
  const [audioError, setAudioError] = useState<string | null>(null);
  const [playbackRate, setPlaybackRate] = useState(1);
  const audioRef = useRef<HTMLAudioElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    setEditedText(entry?.text || '');
  }, [entry?.text]);

  useEffect(() => {
    if (isEditing && textareaRef.current) {
      textareaRef.current.focus();
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = textareaRef.current.scrollHeight + 'px';
    }
  }, [isEditing]);

  // 键盘快捷键支持
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (isEditing) {
        // Ctrl+Enter 或 Cmd+Enter 保存
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
          e.preventDefault();
          handleSaveEdit();
        }
        // Escape 取消编辑
        else if (e.key === 'Escape') {
          e.preventDefault();
          setEditedText(entry.text);
          setIsEditing(false);
        }
      }
    };

    if (isEditing) {
      document.addEventListener('keydown', handleKeyDown);
      return () => document.removeEventListener('keydown', handleKeyDown);
    }
  }, [isEditing, editedText, entry.text]);

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatDate = (timestamp: number): string => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString('zh-CN');
  };

  const handlePlayPause = async () => {
    if (!audioRef.current) return;
    
    try {
      setIsLoading(true);
      setAudioError(null);
      
      if (isPlaying) {
        audioRef.current.pause();
        setIsPlaying(false);
      } else {
        await audioRef.current.play();
        setIsPlaying(true);
      }
    } catch (error) {
      console.error('音频播放失败:', error);
      setAudioError('音频播放失败，请检查文件是否存在');
      setIsPlaying(false);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSeek = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!audioRef.current || !audioDuration) return;
    
    const rect = e.currentTarget.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const newTime = (clickX / rect.width) * audioDuration;
    
    audioRef.current.currentTime = newTime;
    setCurrentTime(newTime);
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newVolume = parseFloat(e.target.value);
    setVolume(newVolume);
    if (audioRef.current) {
      audioRef.current.volume = newVolume;
    }
  };

  const handlePlaybackRateChange = (rate: number) => {
    setPlaybackRate(rate);
    if (audioRef.current) {
      audioRef.current.playbackRate = rate;
    }
  };

  const handleSkip = (seconds: number) => {
    if (!audioRef.current) return;
    
    const newTime = Math.max(0, Math.min(audioDuration, currentTime + seconds));
    audioRef.current.currentTime = newTime;
    setCurrentTime(newTime);
  };

  const handleSaveEdit = async () => {
    try {
      // 调用后端API保存编辑后的文本
      await invoke('update_transcription_text', {
        entryId: entry.id,
        newText: editedText
      });
      
      console.log('保存编辑成功:', editedText);
      setIsEditing(false);
      
      // 可以通过props通知父组件刷新数据
      // onTextUpdated?.(entry.id, editedText);
    } catch (error) {
      console.error('保存失败:', error);
      alert('保存失败: ' + error);
    }
  };

  const handleExport = async (format: 'txt' | 'json' | 'srt') => {
    try {
      const result = await invoke<string>('export_transcription', {
        entryId: entry.id,
        exportFormat: format
      });
      console.log('导出成功:', result);
    } catch (error) {
      console.error('导出失败:', error);
    }
  };

  const handleDeleteEntry = async () => {
    if (confirm('确定要删除这条转录记录吗？')) {
      try {
        await invoke('delete_file', { entryId: entry.id });
        onClose();
      } catch (error) {
        console.error('删除失败:', error);
      }
    }
  };

  if (!isVisible) return null;

  return (
    <div className="transcription-detail-overlay">
      <div className="transcription-detail-modal">
        <div className="detail-header">
          <div className="header-info">
            <h2>转录详情</h2>
            <div className="entry-meta">
              <span className="entry-type">
                {entry.audio_file_path ? 'FILE 文件转录' : 'MIC 实时听写'}
              </span>
              <span className="entry-date">{formatDate(entry.timestamp)}</span>
            </div>
          </div>
          <div className="header-actions">
            <button className="action-btn" onClick={() => setIsEditing(!isEditing)}>
              {isEditing ? '取消编辑' : 'EDIT 编辑'}
            </button>
            <button className="action-btn close-btn" onClick={onClose}>
              CLOSE
            </button>
          </div>
        </div>

        <div className="detail-content">
          {/* 音频播放器 */}
          {entry.audio_file_path && (
            <div className="audio-player-section">
              {audioError && (
                <div className="audio-error">
                  WARN {audioError}
                </div>
              )}
              
              <div className="audio-controls">
                <button 
                  className="skip-btn"
                  onClick={() => handleSkip(-10)}
                  title="后退10秒"
                >
                  ⏪
                </button>
                
                <button 
                  className={`play-btn ${isPlaying ? 'playing' : ''} ${isLoading ? 'loading' : ''}`}
                  onClick={handlePlayPause}
                  disabled={isLoading}
                >
                  {isLoading ? '加载' : isPlaying ? '⏸️' : '▶️'}
                </button>
                
                <button 
                  className="skip-btn"
                  onClick={() => handleSkip(10)}
                  title="前进10秒"
                >
                  ⏩
                </button>
                
                <div className="time-display">
                  <span>{formatTime(currentTime)}</span>
                  <span className="time-separator">/</span>
                  <span>{formatTime(audioDuration)}</span>
                </div>
                
                <div className="playback-rate">
                  <label>倍速:</label>
                  <select 
                    value={playbackRate} 
                    onChange={(e) => handlePlaybackRateChange(parseFloat(e.target.value))}
                  >
                    <option value={0.5}>0.5x</option>
                    <option value={0.75}>0.75x</option>
                    <option value={1}>1x</option>
                    <option value={1.25}>1.25x</option>
                    <option value={1.5}>1.5x</option>
                    <option value={2}>2x</option>
                  </select>
                </div>
              </div>
              
              <div 
                className="audio-progress"
                onClick={handleSeek}
                title="点击跳转到指定位置"
              >
                <div 
                  className="progress-bar"
                  style={{ width: `${audioDuration > 0 ? (currentTime / audioDuration) * 100 : 0}%` }}
                />
              </div>
              
              <div className="volume-control">
                <span className="volume-icon">VOL</span>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.1"
                  value={volume}
                  onChange={handleVolumeChange}
                  className="volume-slider"
                />
                <span className="volume-value">{Math.round(volume * 100)}%</span>
              </div>
              
              <audio
                ref={audioRef}
                src={entry.audio_file_path}
                onTimeUpdate={(e) => setCurrentTime(e.currentTarget.currentTime)}
                onLoadedMetadata={(e) => {
                  setAudioDuration(e.currentTarget.duration);
                  e.currentTarget.volume = volume;
                }}
                onEnded={() => setIsPlaying(false)}
                onError={(e) => {
                  console.error('音频加载错误:', e);
                  setAudioError('音频文件加载失败');
                }}
                onLoadStart={() => setIsLoading(true)}
                onCanPlay={() => setIsLoading(false)}
              />
            </div>
          )}

          {/* 转录文本 */}
          <div className="text-section">
            <div className="section-header">
              <h3>转录文本</h3>
              <div className="text-actions">
                {isEditing ? (
                  <div className="edit-actions">
                    <span className="word-count">
                      {editedText.length} 字符
                    </span>
                    <button className="save-btn" onClick={handleSaveEdit}>
                      SAVE 保存
                    </button>
                    <button 
                      className="cancel-btn" 
                      onClick={() => {
                        setEditedText(entry.text);
                        setIsEditing(false);
                      }}
                    >
                      CANCEL 取消
                    </button>
                  </div>
                ) : (
                  <div className="view-actions">
                    <span className="word-count">
                      {entry.text?.length || 0} 字符
                    </span>
                  </div>
                )}
              </div>
            </div>
            
            {isEditing ? (
              <div className="text-edit-container">
                <textarea
                  ref={textareaRef}
                  className="text-editor"
                  value={editedText}
                  onChange={(e) => setEditedText(e.target.value)}
                  placeholder="编辑转录文本..."
                />
                <div className="edit-tips">
                  TIP 提示：Ctrl+Enter 快速保存，Escape 取消编辑
                </div>
              </div>
            ) : (
              <div className="text-display">
                {entry.text || '暂无转录文本'}
              </div>
            )}
          </div>

          {/* 元数据信息 */}
          <div className="metadata-section">
            <h3>详细信息</h3>
            <div className="metadata-grid">
              <div className="metadata-item">
                <span className="label">模型:</span>
                <span className="value">{entry.model}</span>
              </div>
              <div className="metadata-item">
                <span className="label">置信度:</span>
                <span className="value">{Math.round(entry.confidence * 100)}%</span>
              </div>
              <div className="metadata-item">
                <span className="label">时长:</span>
                <span className="value">{entry.duration} 秒</span>
              </div>
              <div className="metadata-item">
                <span className="label">创建时间:</span>
                <span className="value">{formatDate(entry.timestamp)}</span>
              </div>
              {entry.audio_file_path && (
                <div className="metadata-item">
                  <span className="label">音频文件:</span>
                  <span className="value audio-path">{entry.audio_file_path.split('/').pop()}</span>
                </div>
              )}
            </div>
          </div>
        </div>

        <div className="detail-footer">
          <div className="export-section">
            <h4>导出选项</h4>
            <div className="export-buttons">
              <button 
                className="export-btn"
                onClick={() => handleExport('txt')}
              >
                TXT
              </button>
              <button 
                className="export-btn"
                onClick={() => handleExport('json')}
              >
                JSON
              </button>
              <button 
                className="export-btn"
                onClick={() => handleExport('srt')}
              >
                SRT
              </button>
            </div>
          </div>
          
          <div className="danger-section">
            <button 
              className="delete-btn"
              onClick={handleDeleteEntry}
            >
              DEL 删除记录
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default TranscriptionDetailView;