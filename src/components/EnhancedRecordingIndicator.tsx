import React, { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import WaveformCanvas from "./ui/WaveformCanvas";
import { useAudioVisualization } from "../hooks/useAudioVisualization";
import "./EnhancedRecordingIndicator.css";

interface EnhancedRecordingIndicatorProps {
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
  /** 可视化模式 */
  visualizationMode?: "compact" | "detailed" | "floating";
  /** 是否显示置信度计量器 */
  showConfidenceMeters?: boolean;
  /** 是否启用高级可视化 */
  enableAdvancedVisualization?: boolean;
}

export default function EnhancedRecordingIndicator({
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
  visualizationMode = "detailed",
  showConfidenceMeters = true,
  enableAdvancedVisualization = true,
}: EnhancedRecordingIndicatorProps) {
  const [isVisible, setIsVisible] = useState(true);
  const [showDetails, setShowDetails] = useState(false);
  const [currentDevice, setCurrentDevice] = useState<string>(propCurrentDevice);
  const [voiceActivity, setVoiceActivity] = useState(false);
  const [responseTime, setResponseTime] = useState(0);

  // 音频可视化hook
  const {
    visualizationData,
    isSubscribed,
    metrics,
    error: visualizationError,
    startVisualization,
    stopVisualization,
  } = useAudioVisualization({
    autoStart: isRecording && enableAdvancedVisualization,
    config: {
      render_mode: visualizationMode === "compact" ? "Miniature" : "RealTime",
      max_response_time_ms: 16, // 60 FPS
      buffer_size: visualizationMode === "compact" ? 256 : 1024,
    },
    onPerformanceUpdate: setResponseTime,
    onVoiceActivity: setVoiceActivity,
    onError: (error) => console.error("Audio visualization error:", error),
  });

  // 格式化录音时长
  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  // 获取音频电平条数（增强版）
  const getAudioLevelBars = (level: number): number => {
    return Math.min(Math.floor(level * 12), 12); // 增加到12个条
  };

  // 获取音频质量指示颜色
  const getAudioQualityColor = (level: number): string => {
    if (level < 0.1) return "#666"; // 静音 - 灰色
    if (level < 0.3) return "#4caf50"; // 良好 - 绿色
    if (level < 0.7) return "#ff9800"; // 警告 - 橙色
    return "#f44336"; // 过载 - 红色
  };

  // 获取响应时间指示颜色
  const getResponseTimeColor = (time: number): string => {
    if (time < 16) return "#4caf50"; // 优秀 - 绿色
    if (time < 33) return "#ff9800"; // 良好 - 橙色
    return "#f44336"; // 需要优化 - 红色
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

  // 管理音频可视化订阅
  useEffect(() => {
    if (isRecording && enableAdvancedVisualization && !isSubscribed) {
      startVisualization();
    } else if (!isRecording && isSubscribed) {
      stopVisualization();
    }
  }, [
    isRecording,
    enableAdvancedVisualization,
    isSubscribed,
    startVisualization,
    stopVisualization,
  ]);

  const handleToggle = () => {
    onToggleRecording?.();
  };

  // 计算可视化canvas的尺寸
  const canvasSize = useMemo(() => {
    switch (visualizationMode) {
      case "compact":
        return { width: 60, height: 20 };
      case "floating":
        return { width: 200, height: 80 };
      default: // detailed
        return { width: 150, height: 40 };
    }
  }, [visualizationMode]);

  const indicatorContent = (
    <div
      className={`enhanced-recording-indicator ${isRecording ? "recording" : "idle"} ${showFloating ? "floating" : ""} position-${position} mode-${visualizationMode}`}
    >
      {/* 主状态区域 */}
      <div className="status-main" onClick={() => setShowDetails(!showDetails)}>
        <div className="status-icon">
          {isRecording ? (
            <div className="recording-pulse">
              <div className="pulse-ring"></div>
              <div className="pulse-dot">🎙️</div>
              {voiceActivity && (
                <div className="voice-activity-indicator">
                  <span className="voice-indicator-dot"></span>
                </div>
              )}
            </div>
          ) : (
            <div className="idle-icon">⏸️</div>
          )}
        </div>

        <div className="status-info">
          <div className="status-text">
            {isRecording ? "录音中" : "待机"}
            {voiceActivity && <span className="voice-label">VOICE</span>}
          </div>
          {isRecording && (
            <div className="recording-duration">
              {formatDuration(recordingDuration)}
            </div>
          )}
        </div>

        {/* 增强音频电平指示器 */}
        {isRecording && (
          <div className="enhanced-audio-level-container">
            {enableAdvancedVisualization && visualizationData ? (
              <div className="advanced-visualization">
                <WaveformCanvas
                  width={canvasSize.width}
                  height={canvasSize.height}
                  renderMode={
                    visualizationMode === "compact" ? "Miniature" : "RealTime"
                  }
                  enableRealTime={true}
                  className="embedded-waveform"
                />
                {showConfidenceMeters && (
                  <div className="confidence-meters">
                    <div
                      className="confidence-bar"
                      style={{
                        width: `${(visualizationData.amplitude || 0) * 100}%`,
                        backgroundColor: getAudioQualityColor(
                          visualizationData.amplitude || 0,
                        ),
                      }}
                    />
                  </div>
                )}
              </div>
            ) : (
              // 传统电平条显示
              <div className="traditional-audio-level-bars">
                {Array.from({ length: 12 }, (_, i) => (
                  <div
                    key={i}
                    className={`level-bar ${i < getAudioLevelBars(audioLevel) ? "active" : ""}`}
                    style={{
                      height: `${(i + 1) * 8}%`,
                      backgroundColor:
                        i < 8 ? "#4caf50" : i < 10 ? "#ff9800" : "#f44336",
                    }}
                  />
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {/* 详细信息面板 */}
      {showDetails && (
        <div className="status-details enhanced">
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
            <>
              <div className="detail-row">
                <span className="detail-label">音量:</span>
                <span className="detail-value">
                  {Math.round(
                    (visualizationData?.amplitude || audioLevel) * 100,
                  )}
                  %
                </span>
              </div>
              {enableAdvancedVisualization && visualizationData && (
                <>
                  <div className="detail-row">
                    <span className="detail-label">噪声级别:</span>
                    <span className="detail-value">
                      {Math.round(visualizationData.noise_level * 100)}%
                    </span>
                  </div>
                  <div className="detail-row">
                    <span className="detail-label">响应时间:</span>
                    <span
                      className="detail-value"
                      style={{ color: getResponseTimeColor(responseTime) }}
                    >
                      {responseTime}ms
                    </span>
                  </div>
                  {metrics && (
                    <div className="detail-row">
                      <span className="detail-label">缓冲区使用:</span>
                      <span className="detail-value">
                        {Math.round(
                          (metrics.buffer_usage_amplitude /
                            metrics.max_buffer_size) *
                            100,
                        )}
                        %
                      </span>
                    </div>
                  )}
                </>
              )}
            </>
          )}

          {visualizationError && (
            <div className="detail-row error">
              <span className="detail-label">可视化错误:</span>
              <span className="detail-value error-text">
                {visualizationError}
              </span>
            </div>
          )}
        </div>
      )}

      {/* 控制按钮 */}
      <div className="status-controls enhanced">
        <button
          className={`control-btn toggle-btn ${isRecording ? "stop" : "start"} ${voiceActivity ? "voice-active" : ""}`}
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

        {/* 可视化模式切换按钮 */}
        {enableAdvancedVisualization && (
          <button
            className={`control-btn visualization-btn ${isSubscribed ? "active" : ""}`}
            onClick={() =>
              isSubscribed ? stopVisualization() : startVisualization()
            }
            title="切换高级可视化"
          >
            📊
          </button>
        )}
      </div>

      {/* 快捷键提示 */}
      {!isRecording && (
        <div className="shortcut-hint enhanced">
          按 <kbd>{shortcutKey}</kbd> 开始录音
          {enableAdvancedVisualization && (
            <div className="visualization-hint">支持实时音频可视化</div>
          )}
        </div>
      )}

      {/* 性能指标显示 */}
      {isRecording && enableAdvancedVisualization && visualizationData && (
        <div className="performance-indicators">
          {visualizationData.peak_detected && (
            <div className="peak-indicator">🔊</div>
          )}
          {voiceActivity && <div className="voice-activity">🗣️</div>}
          {responseTime > 50 && (
            <div
              className="performance-warning"
              title={`响应时间: ${responseTime}ms`}
            >
              ⚠️
            </div>
          )}
        </div>
      )}
    </div>
  );

  if (showFloating && !isVisible) {
    return (
      <div
        className={`recording-status-minimized enhanced position-${position}`}
        onClick={() => setIsVisible(true)}
      >
        <div
          className={`mini-indicator ${isRecording ? "recording" : "idle"} ${voiceActivity ? "voice-active" : ""}`}
        >
          {isRecording ? (voiceActivity ? "🔴🗣️" : "🔴") : "⚫"}
        </div>
        {isRecording && enableAdvancedVisualization && (
          <WaveformCanvas
            width={30}
            height={10}
            renderMode="Miniature"
            enableRealTime={true}
            className="mini-waveform"
          />
        )}
      </div>
    );
  }

  return indicatorContent;
}
