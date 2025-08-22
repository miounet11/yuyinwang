import React, { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import './ProgressiveTranscriptionStatus.css';

interface TranscriptionStatus {
  isActive: boolean;
  currentText: string;
  queueLength: number;
  injectionCount: number;
  transcriptorActive: boolean;
  injectorActive: boolean;
  targetApp?: string;
  confidence?: number;
}

interface ProgressiveTriggerConfig {
  shortcut: string;
  enabled: boolean;
  realTimeInjection: boolean;
  autoDetectApp: boolean;
}

const ProgressiveTranscriptionStatus: React.FC = () => {
  const [status, setStatus] = useState<TranscriptionStatus>({
    isActive: false,
    currentText: '',
    queueLength: 0,
    injectionCount: 0,
    transcriptorActive: false,
    injectorActive: false,
  });

  const [config, setConfig] = useState<ProgressiveTriggerConfig>({
    shortcut: 'Option+Space',
    enabled: true,
    realTimeInjection: true,
    autoDetectApp: true,
  });

  const [isMonitoring, setIsMonitoring] = useState(false);
  const [lastActivity, setLastActivity] = useState<string>('');
  const [error, setError] = useState<string>('');
  
  // 音频可视化相关
  const [audioLevel, setAudioLevel] = useState(0);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>();
  
  // 文本动画相关
  const [displayText, setDisplayText] = useState('');
  const [textAnimation, setTextAnimation] = useState<'typing' | 'backspace' | 'idle'>('idle');

  useEffect(() => {
    const setupEventListeners = async () => {
      // 监听渐进式语音输入状态
      const unlistenStatus = await listen<any>('progressive_voice_input_status', (event) => {
        console.log('📊 渐进式状态更新:', event.payload);
        setStatus(prev => ({
          ...prev,
          transcriptorActive: event.payload.transcriptor_active,
          injectorActive: event.payload.injector_active,
          queueLength: event.payload.queue_length || 0,
          currentText: event.payload.last_injected || '',
          isActive: event.payload.transcriptor_active || event.payload.injector_active,
        }));
        
        setLastActivity(`状态更新: ${new Date().toLocaleTimeString()}`);
      });

      // 监听流式转录事件
      const unlistenStreaming = await listen<any>('streaming_transcription', (event) => {
        console.log('🔄 流式转录:', event.payload);
        const { text, is_partial, confidence } = event.payload;
        
        setStatus(prev => ({
          ...prev,
          currentText: text,
          confidence: confidence,
        }));
        
        // 启动打字动画
        if (is_partial) {
          setTextAnimation('typing');
          setTimeout(() => setTextAnimation('idle'), 500);
        }
        
        setDisplayText(text);
        setLastActivity(`转录: ${text.substring(0, 20)}${text.length > 20 ? '...' : ''}`);
      });

      // 监听触发器激活事件
      const unlistenTrigger = await listen<any>('progressive_trigger_activated', (event) => {
        console.log('🚀 触发器激活:', event.payload);
        setStatus(prev => ({
          ...prev,
          isActive: true,
          targetApp: event.payload.target_app,
        }));
        setLastActivity(`触发: ${event.payload.shortcut}`);
        setError('');
      });

      // 监听完成事件
      const unlistenComplete = await listen<any>('progressive_voice_input_complete', (event) => {
        console.log('✅ 语音输入完成:', event.payload);
        setStatus(prev => ({
          ...prev,
          isActive: false,
          transcriptorActive: false,
          injectorActive: false,
          injectionCount: prev.injectionCount + 1,
        }));
        setLastActivity(`完成: ${event.payload.injected_text || '无文本'}`);
        
        // 延迟重置显示文本
        setTimeout(() => {
          setDisplayText('');
          setTextAnimation('idle');
        }, 3000);
      });

      // 监听错误事件
      const unlistenError = await listen<any>('progressive_voice_input_error', (event) => {
        console.log('❌ 语音输入错误:', event.payload);
        setError(event.payload.error || '未知错误');
        setStatus(prev => ({
          ...prev,
          isActive: false,
          transcriptorActive: false,
          injectorActive: false,
        }));
        setLastActivity(`错误: ${event.payload.error}`);
      });

      return () => {
        unlistenStatus();
        unlistenStreaming();
        unlistenTrigger();
        unlistenComplete();
        unlistenError();
      };
    };

    setupEventListeners();
  }, []);

  // 音频可视化
  useEffect(() => {
    const drawAudioVisualization = () => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      
      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      const { width, height } = canvas;
      ctx.clearRect(0, 0, width, height);

      if (status.isActive) {
        // 绘制音频波形
        ctx.fillStyle = status.transcriptorActive 
          ? 'rgba(34, 197, 94, 0.3)' 
          : 'rgba(156, 163, 175, 0.3)';
        
        const barCount = 12;
        const barWidth = width / barCount;
        
        for (let i = 0; i < barCount; i++) {
          const barHeight = Math.random() * height * (status.transcriptorActive ? audioLevel : 0.1);
          ctx.fillRect(i * barWidth + 2, height - barHeight, barWidth - 4, barHeight);
        }

        // 绘制中心脉冲
        if (status.transcriptorActive) {
          const pulseRadius = 20 + Math.sin(Date.now() * 0.01) * 5;
          ctx.fillStyle = 'rgba(34, 197, 94, 0.6)';
          ctx.beginPath();
          ctx.arc(width / 2, height / 2, pulseRadius, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      animationRef.current = requestAnimationFrame(drawAudioVisualization);
    };

    if (status.isActive) {
      drawAudioVisualization();
    } else {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [status.isActive, status.transcriptorActive, audioLevel]);

  // 启动监听
  const startMonitoring = async () => {
    try {
      setError('');
      const result = await invoke<string>('start_progressive_trigger_monitoring', {
        config: {
          shortcut: config.shortcut,
          long_press_threshold_ms: 800,
          enabled: config.enabled,
          enable_real_time_injection: config.realTimeInjection,
          trigger_sound_enabled: true,
          auto_detect_target_app: config.autoDetectApp,
        }
      });
      console.log('✅ 监听启动成功:', result);
      setIsMonitoring(true);
      setLastActivity('监听已启动');
    } catch (err) {
      console.error('❌ 启动监听失败:', err);
      setError(String(err));
    }
  };

  // 测试触发
  const testTrigger = async () => {
    try {
      setError('');
      const result = await invoke<string>('test_progressive_trigger', {
        targetBundleId: status.targetApp,
      });
      console.log('🧪 测试成功:', result);
      setLastActivity('测试触发成功');
    } catch (err) {
      console.error('❌ 测试失败:', err);
      setError(String(err));
    }
  };

  return (
    <div className="progressive-transcription-status">
      {/* 主状态显示 */}
      <div className={`status-main ${status.isActive ? 'active' : 'idle'}`}>
        <div className="status-header">
          <div className="status-title">
            <span className="icon">🎙️</span>
            <h3>渐进式语音输入</h3>
            <div className={`status-indicator ${status.isActive ? 'active' : 'idle'}`}>
              {status.isActive ? '活跃' : '空闲'}
            </div>
          </div>
          <div className="control-buttons">
            <button 
              onClick={startMonitoring} 
              disabled={isMonitoring}
              className="btn btn-primary"
            >
              {isMonitoring ? '监听中' : '启动监听'}
            </button>
            <button 
              onClick={testTrigger}
              className="btn btn-secondary"
            >
              测试触发
            </button>
          </div>
        </div>

        {/* 音频可视化 */}
        <div className="audio-visualization">
          <canvas 
            ref={canvasRef}
            width={300}
            height={60}
            className="audio-canvas"
          />
        </div>

        {/* 实时文本显示 */}
        <div className={`text-display ${textAnimation}`}>
          <div className="text-label">当前转录:</div>
          <div className="text-content">
            {displayText || '等待语音输入...'}
            {status.confidence && (
              <span className="confidence">
                置信度: {(status.confidence * 100).toFixed(0)}%
              </span>
            )}
          </div>
        </div>
      </div>

      {/* 详细状态信息 */}
      <div className="status-details">
        <div className="detail-grid">
          <div className="detail-item">
            <span className="detail-label">转录器:</span>
            <span className={`detail-value ${status.transcriptorActive ? 'active' : 'inactive'}`}>
              {status.transcriptorActive ? '运行中' : '停止'}
            </span>
          </div>
          
          <div className="detail-item">
            <span className="detail-label">注入器:</span>
            <span className={`detail-value ${status.injectorActive ? 'active' : 'inactive'}`}>
              {status.injectorActive ? '运行中' : '停止'}
            </span>
          </div>
          
          <div className="detail-item">
            <span className="detail-label">队列长度:</span>
            <span className="detail-value">{status.queueLength}</span>
          </div>
          
          <div className="detail-item">
            <span className="detail-label">注入次数:</span>
            <span className="detail-value">{status.injectionCount}</span>
          </div>
          
          {status.targetApp && (
            <div className="detail-item">
              <span className="detail-label">目标应用:</span>
              <span className="detail-value">{status.targetApp}</span>
            </div>
          )}
        </div>
      </div>

      {/* 配置面板 */}
      <div className="config-panel">
        <h4>快捷键配置</h4>
        <div className="config-grid">
          <div className="config-item">
            <label>快捷键:</label>
            <input
              type="text"
              value={config.shortcut}
              onChange={(e) => setConfig(prev => ({ ...prev, shortcut: e.target.value }))}
              className="config-input"
            />
          </div>
          
          <div className="config-item">
            <label>
              <input
                type="checkbox"
                checked={config.realTimeInjection}
                onChange={(e) => setConfig(prev => ({ ...prev, realTimeInjection: e.target.checked }))}
              />
              实时注入
            </label>
          </div>
          
          <div className="config-item">
            <label>
              <input
                type="checkbox"
                checked={config.autoDetectApp}
                onChange={(e) => setConfig(prev => ({ ...prev, autoDetectApp: e.target.checked }))}
              />
              自动检测应用
            </label>
          </div>
        </div>
      </div>

      {/* 活动日志和错误显示 */}
      <div className="activity-log">
        <div className="log-header">
          <span>最后活动:</span>
          <span className="activity-time">{lastActivity}</span>
        </div>
        {error && (
          <div className="error-message">
            <span className="error-icon">⚠️</span>
            {error}
          </div>
        )}
      </div>
    </div>
  );
};

export default ProgressiveTranscriptionStatus;