import React, { useState, useEffect, useRef, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './ProductionTranscriptionDisplay.css';

interface TranscriptionDisplayProps {
  text: string;
  isRealtime?: boolean;
  showTimestamps?: boolean;
  language?: string;
  confidence?: number;
  onTextEdit?: (newText: string) => void;
  onExport?: (format: 'txt' | 'md' | 'docx') => void;
}

interface TranscriptionSegment {
  text: string;
  timestamp: number;
  confidence: number;
  isNew?: boolean;
}

interface ExportOptions {
  format: 'txt' | 'md' | 'docx';
  includeTimestamps: boolean;
  includeConfidence: boolean;
}

const ProductionTranscriptionDisplay: React.FC<TranscriptionDisplayProps> = ({
  text,
  isRealtime = false,
  showTimestamps = false,
  language = 'zh-CN',
  confidence = 0,
  onTextEdit,
  onExport
}) => {
  const [editableText, setEditableText] = useState(text);
  const [isEditing, setIsEditing] = useState(false);
  const [showExportMenu, setShowExportMenu] = useState(false);
  const [wordCount, setWordCount] = useState(0);
  const [charCount, setCharCount] = useState(0);
  const [segments, setSegments] = useState<TranscriptionSegment[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedText, setSelectedText] = useState('');

  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const displayRef = useRef<HTMLDivElement>(null);
  const lastTextLengthRef = useRef(0);

  // 文本统计
  useEffect(() => {
    const chars = text.length;
    const words = text.trim() ? text.trim().split(/\s+/).length : 0;
    setCharCount(chars);
    setWordCount(words);
  }, [text]);

  // 同步文本变化
  useEffect(() => {
    setEditableText(text);

    // 如果是实时转录，检测新增内容
    if (isRealtime && text.length > lastTextLengthRef.current) {
      const newText = text.slice(lastTextLengthRef.current);
      highlightNewText(newText);
      scrollToBottom();
    }
    lastTextLengthRef.current = text.length;
  }, [text, isRealtime]);

  // 分段处理文本（用于时间戳显示）
  const processedSegments = useMemo(() => {
    if (!showTimestamps) return [];

    // 模拟将文本分段，实际应该从后端获取
    const sentences = text.split(/[。！？.!?]+/).filter(s => s.trim());
    return sentences.map((sentence, index) => ({
      text: sentence.trim(),
      timestamp: index * 5, // 模拟时间戳
      confidence: Math.random() * 0.3 + 0.7 // 模拟置信度
    }));
  }, [text, showTimestamps]);

  // 高亮新文本
  const highlightNewText = (newText: string) => {
    // 添加高亮动画效果
    if (displayRef.current) {
      const lastChild = displayRef.current.lastElementChild;
      if (lastChild) {
        lastChild.classList.add('new-text-highlight');
        setTimeout(() => {
          lastChild.classList.remove('new-text-highlight');
        }, 2000);
      }
    }
  };

  // 滚动到底部
  const scrollToBottom = () => {
    if (displayRef.current) {
      displayRef.current.scrollTop = displayRef.current.scrollHeight;
    }
  };

  // 处理文本编辑
  const handleTextChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newText = e.target.value;
    setEditableText(newText);
    if (onTextEdit) {
      onTextEdit(newText);
    }
  };

  // 开始编辑
  const startEditing = () => {
    setIsEditing(true);
    setTimeout(() => {
      if (textAreaRef.current) {
        textAreaRef.current.focus();
        textAreaRef.current.select();
      }
    }, 100);
  };

  // 完成编辑
  const finishEditing = () => {
    setIsEditing(false);
    if (onTextEdit) {
      onTextEdit(editableText);
    }
  };

  // 处理键盘事件
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      setIsEditing(false);
      setEditableText(text); // 恢复原文本
    } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      finishEditing();
    }
  };

  // 复制文本
  const copyText = async () => {
    try {
      await navigator.clipboard.writeText(text);
      // 显示复制成功提示
      showToast('文本已复制到剪贴板');
    } catch (error) {
      console.error('复制失败:', error);
      showToast('复制失败', 'error');
    }
  };

  // 导出文本
  const handleExport = async (format: 'txt' | 'md' | 'docx') => {
    try {
      const options: ExportOptions = {
        format,
        includeTimestamps: showTimestamps,
        includeConfidence: true
      };

      await invoke('export_transcription', {
        text: editableText,
        options
      });

      showToast(`已导出为 ${format.toUpperCase()} 格式`);
      setShowExportMenu(false);
    } catch (error) {
      console.error('导出失败:', error);
      showToast('导出失败', 'error');
    }
  };

  // 搜索文本
  const highlightSearchResults = (text: string, query: string) => {
    if (!query.trim()) return text;

    const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
    return text.replace(regex, '<mark class="search-highlight">$1</mark>');
  };

  // 文本选择处理
  const handleTextSelection = () => {
    const selection = window.getSelection();
    if (selection && selection.toString().trim()) {
      setSelectedText(selection.toString().trim());
    } else {
      setSelectedText('');
    }
  };

  // 显示提示
  const showToast = (message: string, type: 'success' | 'error' = 'success') => {
    // 这里应该集成一个Toast组件
    console.log(`${type}: ${message}`);
  };

  // 格式化时间戳
  const formatTimestamp = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // 获取置信度颜色
  const getConfidenceColor = (confidence: number): string => {
    if (confidence >= 0.8) return '#10b981'; // 绿色
    if (confidence >= 0.6) return '#f59e0b'; // 黄色
    return '#ef4444'; // 红色
  };

  return (
    <div className="production-transcription-display">
      {/* 工具栏 */}
      <div className="transcription-toolbar">
        <div className="toolbar-left">
          <div className="text-stats">
            <span className="stat-item">
              <span className="stat-label">字符:</span>
              <span className="stat-value">{charCount.toLocaleString()}</span>
            </span>
            <span className="stat-item">
              <span className="stat-label">词数:</span>
              <span className="stat-value">{wordCount.toLocaleString()}</span>
            </span>
            {confidence > 0 && (
              <span className="stat-item">
                <span className="stat-label">置信度:</span>
                <span
                  className="stat-value confidence"
                  style={{ color: getConfidenceColor(confidence) }}
                >
                  {Math.round(confidence * 100)}%
                </span>
              </span>
            )}
          </div>
        </div>

        <div className="toolbar-center">
          <div className="search-container">
            <input
              type="text"
              className="search-input"
              placeholder="搜索文本..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            <span className="search-icon">🔍</span>
          </div>
        </div>

        <div className="toolbar-right">
          <button
            className="toolbar-button"
            onClick={startEditing}
            disabled={isRealtime}
            title="编辑文本"
          >
            <span className="button-icon">✏️</span>
            编辑
          </button>

          <button
            className="toolbar-button"
            onClick={copyText}
            title="复制文本"
          >
            <span className="button-icon">📋</span>
            复制
          </button>

          <div className="export-dropdown">
            <button
              className="toolbar-button"
              onClick={() => setShowExportMenu(!showExportMenu)}
              title="导出文本"
            >
              <span className="button-icon">📥</span>
              导出
            </button>

            {showExportMenu && (
              <div className="export-menu">
                <button
                  className="export-option"
                  onClick={() => handleExport('txt')}
                >
                  <span className="option-icon">📄</span>
                  纯文本 (TXT)
                </button>
                <button
                  className="export-option"
                  onClick={() => handleExport('md')}
                >
                  <span className="option-icon">📝</span>
                  Markdown (MD)
                </button>
                <button
                  className="export-option"
                  onClick={() => handleExport('docx')}
                >
                  <span className="option-icon">📘</span>
                  Word (DOCX)
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 主显示区域 */}
      <div className="transcription-content">
        {!text ? (
          <div className="empty-state">
            <div className="empty-icon">🎙️</div>
            <h3>等待转录内容</h3>
            <p>开始录制后，转录文本将在此处显示</p>
          </div>
        ) : isEditing ? (
          <div className="edit-mode">
            <textarea
              ref={textAreaRef}
              className="edit-textarea"
              value={editableText}
              onChange={handleTextChange}
              onKeyDown={handleKeyDown}
              onBlur={finishEditing}
              placeholder="在此编辑转录文本..."
            />
            <div className="edit-hints">
              <span>💡 按 Ctrl/Cmd + Enter 保存，按 Esc 取消</span>
            </div>
          </div>
        ) : showTimestamps && processedSegments.length > 0 ? (
          <div
            ref={displayRef}
            className="segments-view"
            onMouseUp={handleTextSelection}
          >
            {processedSegments.map((segment, index) => (
              <div key={index} className="text-segment">
                <div className="segment-timestamp">
                  {formatTimestamp(segment.timestamp)}
                </div>
                <div className="segment-content">
                  <div
                    className="segment-text"
                    dangerouslySetInnerHTML={{
                      __html: highlightSearchResults(segment.text, searchQuery)
                    }}
                  />
                  <div
                    className="segment-confidence"
                    style={{ color: getConfidenceColor(segment.confidence) }}
                  >
                    {Math.round(segment.confidence * 100)}%
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div
            ref={displayRef}
            className={`text-view ${isRealtime ? 'realtime' : ''}`}
            onMouseUp={handleTextSelection}
          >
            <div
              className="transcription-text"
              dangerouslySetInnerHTML={{
                __html: highlightSearchResults(text, searchQuery)
              }}
            />
            {isRealtime && (
              <div className="realtime-cursor">
                <span className="cursor-blink">|</span>
              </div>
            )}
          </div>
        )}
      </div>

      {/* 选中文本操作面板 */}
      {selectedText && (
        <div className="selection-panel">
          <div className="selection-text">
            已选择: "{selectedText.length > 50 ? selectedText.slice(0, 50) + '...' : selectedText}"
          </div>
          <div className="selection-actions">
            <button
              className="selection-button"
              onClick={() => navigator.clipboard.writeText(selectedText)}
            >
              复制选中
            </button>
            <button
              className="selection-button"
              onClick={() => setSearchQuery(selectedText)}
            >
              搜索相似
            </button>
          </div>
        </div>
      )}

      {/* 实时状态指示器 */}
      {isRealtime && (
        <div className="realtime-indicator">
          <span className="realtime-dot"></span>
          <span className="realtime-text">实时转录中</span>
        </div>
      )}
    </div>
  );
};

export default ProductionTranscriptionDisplay;
