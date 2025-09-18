import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./RecordingStatusIndicator.css";

interface RecordingStatusIndicatorProps {
  isRecording: boolean;
  recordingDuration: number;
  audioLevel?: number;
  selectedModel?: string;
  onToggleRecording?: () => void;
  shortcutKey?: string;
  showFloating?: boolean;
  position?:
    | "top-left"
    | "top-right"
    | "bottom-left"
    | "bottom-right"
    | "center";
  audioDevices?: any[];
  currentDevice?: string;
}

export default function RecordingStatusIndicator({
  isRecording,
  recordingDuration,
  audioLevel = 0,
  selectedModel = "whisper-tiny",
  onToggleRecording,
  shortcutKey = "Cmd+Shift+R",
  showFloating = false,
  position = "top-right",
  audioDevices = [],
  currentDevice: propCurrentDevice = "",
}: RecordingStatusIndicatorProps) {
  const [isVisible, setIsVisible] = useState(true);
  const [showDetails, setShowDetails] = useState(false);
  const [currentDevice, setCurrentDevice] = useState<string>(propCurrentDevice);

  // 格式化录音时长
  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  // 更新当前设备
  useEffect(() => {
    if (audioDevices.length > 0 && !currentDevice) {
      const defaultDevice = audioDevices.find((d) => d.is_default);
      if (defaultDevice) {
        setCurrentDevice(defaultDevice.name);
      }
    }
  }, [audioDevices, currentDevice]);

  // 更新传入的设备名称
  useEffect(() => {
    if (propCurrentDevice) {
      setCurrentDevice(propCurrentDevice);
    }
  }, [propCurrentDevice]);

  // 计算音频电平条数
  const getAudioLevelBars = (level: number): number => {
    return Math.min(Math.floor(level * 10), 10);
  };

  const handleToggle = () => {
    onToggleRecording?.();
  };

  const indicatorContent = (
    <div
      className={`recording-status-indicator ${isRecording ? "recording" : "idle"} ${showFloating ? "floating" : ""} position-${position}`}
    >
      {/* 主状态区域 */}
      <div className="status-main" onClick={() => setShowDetails(!showDetails)}>
        <div className="status-icon">
          {isRecording ? (
            <div className="recording-pulse">
              <div className="pulse-ring"></div>
              <div className="pulse-dot">🎙️</div>
            </div>
          ) : (
            <div className="idle-icon">⏸️</div>
          )}
        </div>

        <div className="status-info">
          <div className="status-text">{isRecording ? "录音中" : "待机"}</div>
          {isRecording && (
            <div className="recording-duration">
              {formatDuration(recordingDuration)}
            </div>
          )}
        </div>

        {/* 音频电平指示器 */}
        {isRecording && (
          <div className="audio-level-container">
            <div className="audio-level-bars">
              {Array.from({ length: 10 }, (_, i) => (
                <div
                  key={i}
                  className={`level-bar ${i < getAudioLevelBars(audioLevel) ? "active" : ""}`}
                  style={{
                    height: `${(i + 1) * 10}%`,
                    backgroundColor:
                      i < 7 ? "#4caf50" : i < 9 ? "#ff9800" : "#f44336",
                  }}
                />
              ))}
            </div>
          </div>
        )}
      </div>

      {/* 详细信息面板 */}
      {showDetails && (
        <div className="status-details">
          <div className="detail-row">
            <span className="detail-label">设备:</span>
            <span className="detail-value">{currentDevice || "默认设备"}</span>
          </div>
          <div className="detail-row">
            <span className="detail-label">模型:</span>
            <span className="detail-value">{selectedModel}</span>
          </div>
          <div className="detail-row">
            <span className="detail-label">快捷键:</span>
            <span className="detail-value">{shortcutKey}</span>
          </div>
          {isRecording && (
            <div className="detail-row">
              <span className="detail-label">音量:</span>
              <span className="detail-value">
                {Math.round(audioLevel * 100)}%
              </span>
            </div>
          )}
        </div>
      )}

      {/* 控制按钮 */}
      <div className="status-controls">
        <button
          className={`control-btn toggle-btn ${isRecording ? "stop" : "start"}`}
          onClick={handleToggle}
          title={
            isRecording
              ? `停止录音 (${shortcutKey})`
              : `开始录音 (${shortcutKey})`
          }
        >
          {isRecording ? "⏹️" : "⏺️"}
        </button>

        {showFloating && (
          <button
            className="control-btn minimize-btn"
            onClick={() => setIsVisible(!isVisible)}
            title="最小化/展开"
          >
            {isVisible ? "➖" : "➕"}
          </button>
        )}
      </div>

      {/* 快捷键提示 */}
      {!isRecording && (
        <div className="shortcut-hint">
          按 <kbd>{shortcutKey}</kbd> 开始录音
        </div>
      )}
    </div>
  );

  if (showFloating && !isVisible) {
    return (
      <div
        className={`recording-status-minimized position-${position}`}
        onClick={() => setIsVisible(true)}
      >
        <div className={`mini-indicator ${isRecording ? "recording" : "idle"}`}>
          {isRecording ? "🔴" : "⚫"}
        </div>
      </div>
    );
  }

  return indicatorContent;
}
