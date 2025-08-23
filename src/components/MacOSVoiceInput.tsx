import React, { useState, useRef, useEffect } from 'react';
import { appWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
// Removed complex framer-motion dependencies for better performance
import { 
  startPerformanceMonitoring, 
  stopPerformanceMonitoring, 
  measureVoiceInputLatency 
} from '../utils/performanceMonitor';
import './MacOSVoiceInput.css';

interface ActiveAppInfo {
  name: string;
  icon?: string;
  bundleId?: string;
}

type InputState = 'idle' | 'listening' | 'processing' | 'injecting';

const MacOSVoiceInput: React.FC = () => {
  const [state, setState] = useState<InputState>('idle');
  const [transcribedText, setTranscribedText] = useState('');
  const [activeApp, setActiveApp] = useState<ActiveAppInfo>({ name: '未知应用' });
  const [audioLevel, setAudioLevel] = useState(0);
  const [isRecording, setIsRecording] = useState(false);
  const [hasAudioInput, setHasAudioInput] = useState(false);
  const [currentModel, setCurrentModel] = useState<string>('loading...'); // 当前使用的模型
  const [isProcessing, setIsProcessing] = useState(false); // 防止重复处理
  const [isProcessingTrigger, setIsProcessingTrigger] = useState(false); // 防止重复触发事件
  
  const containerRef = useRef<HTMLDivElement>(null);
  const animationRef = useRef<number>();
  const noSoundTimeoutRef = useRef<number | null>(null);
  const silenceTimeoutRef = useRef<number | null>(null);
  const autoCloseTimeoutRef = useRef<number | null>(null);
  const processingTimeoutRef = useRef<number | null>(null);  // 处理超时
  const retryCountRef = useRef<number>(0);  // 重试计数
  
  // 智能VAD状态追踪
  const lastSoundTimeRef = useRef<number>(Date.now());
  const recordingStartTimeRef = useRef<number>(0);
  const continuousSilenceDurationRef = useRef<number>(0);
  
  // 音频分析和自适应参数
  const audioLevelHistoryRef = useRef<number[]>([]);
  const noiseFloorRef = useRef<number>(0.03);  // 初始噪音基线设为0.03（适应新的范围）
  const smoothedLevelRef = useRef<number>(0);
  const vadStateRef = useRef<'waiting' | 'speech' | 'silence'>('waiting');
  const speechDetectedRef = useRef<boolean>(false);
  

  useEffect(() => {
      // Simplified audio level handling without complex animator
    
    // 获取当前模型信息
    const fetchModelInfo = async () => {
      try {
        const model = await invoke<string>('get_current_model_info');
        setCurrentModel(model);
      } catch (error) {
        setCurrentModel('unknown');
      }
    };
    fetchModelInfo();
    
    // 设置窗口属性 - 模拟 macOS 原生样式
    const setupWindow = async () => {
      
      await appWindow.setAlwaysOnTop(true);
      await appWindow.setDecorations(false);
      await appWindow.setResizable(false);
      await appWindow.setSkipTaskbar(true);
      
      // 设置窗口大小和位置 - 像 macOS 语音输入一样小巧
      await appWindow.setSize(new LogicalSize(280, 60));
      
      // 居中显示在屏幕底部
      try {
        const screenWidth = window.screen.width;
        const screenHeight = window.screen.height;
        const x = Math.floor((screenWidth - 280) / 2);
        const y = Math.floor(screenHeight - 100); // 屏幕底部位置
        await appWindow.setPosition(new LogicalPosition(x, y));
      } catch (error) {
      }

      // 初始化时不获取活动应用，等待事件触发时传递
      // 活动应用信息将由快捷键触发时传递
    };
    
    setupWindow();

    // 监听语音输入触发事件
    const unlistenTrigger = listen<ActiveAppInfo>('voice_input_triggered', async (event) => {
      // 防止重复触发
      if (isProcessingTrigger) {
        return;
      }
      
      setIsProcessingTrigger(true);
      
      setState('idle');
      setTranscribedText('');
      setHasAudioInput(false);
      
      // 使用事件中传递的活动应用信息（这是触发前的原始活动应用）
      if (event.payload && event.payload.name) {
        setActiveApp(event.payload);
      } else {
        // 如果没有传递活动应用信息，则尝试获取（兼容旧版本）
        try {
          const appInfo = await invoke<ActiveAppInfo>('get_active_app_info_for_voice');
          setActiveApp(appInfo);
        } catch (error) {
        }
      }
      
      // 显示窗口并自动开始录音（按住事件到来时再真正开始）
      await appWindow.show();
      await appWindow.setFocus();
      
      // 允许后续触发
      setTimeout(() => {
        setIsProcessingTrigger(false);
      }, 300);
    });

    // 新增：监听长按开始/结束事件（来自原生 Fn/F1 监听）
    const unlistenHoldStart = listen('voice_input_hold_start', () => {
      if (!isRecording) {
        startListening();
      }
    });
    const unlistenHoldEnd = listen('voice_input_hold_end', () => {
      if (isRecording) {
        stopListening();
      }
    });

    // 监听实时转录结果（渐进式部分）
    const unlistenStreaming = listen<any>('streaming_transcription', (event) => {
      const payload: any = event.payload || {};
      const text: string = payload.text || '';
      setTranscribedText(text);
      if (text && text.trim()) {
        setHasAudioInput(true);
        resetSilenceTimeout();
      }
    });

    // 监听流式完成（可用于显示最终状态）
    const unlistenStreamingComplete = listen<string>('streaming_complete', () => {
      // 保持 stopListening 逻辑主导，完成事件仅用于 UI 提示
    });

    // 智能VAD音频电平监听 - 多层检测算法 + 动画集成
    const unlistenAudioLevel = listen<number>('audio_level', (event) => {
      const rawLevel = event.payload;
      const now = Date.now();
      
      // Direct audio level update for better performance
      
      // 🎯 VAD 配置参数 - 适配新的音频电平范围
      const VAD_CONFIG = {
        // 阈值设置（根据新的RMS计算方法调整）
        SOUND_THRESHOLD: 0.15,        // 主声音阈值（正常说话约0.1-0.3）
        SILENCE_THRESHOLD: 0.05,      // 静音阈值（环境噪音通常<0.05）
        NOISE_GATE: 0.02,             // 噪音门限（极低背景噪音）
        
        // 时间控制
        MIN_SPEECH_DURATION: 500,     // 最短有效语音时长（减少到500ms，更灵敏）
        SILENCE_DURATION: 1500,       // 静音等待时间（1.5秒）
        CONFIRMATION_DELAY: 200,      // 确认延迟（200ms）
        
        // 自适应参数
        LEVEL_SMOOTHING: 0.4,         // 音频电平平滑系数（增加平滑度）
        NOISE_FLOOR_SAMPLES: 50,      // 噪音基线采样数量（增加样本数）
      };
      
      // 📈 音频电平平滑处理
      smoothedLevelRef.current = smoothedLevelRef.current * (1 - VAD_CONFIG.LEVEL_SMOOTHING) + 
                                rawLevel * VAD_CONFIG.LEVEL_SMOOTHING;
      const level = smoothedLevelRef.current;
      setAudioLevel(level);
      
      // 📊 噪音基线自适应学习
      audioLevelHistoryRef.current.push(level);
      if (audioLevelHistoryRef.current.length > VAD_CONFIG.NOISE_FLOOR_SAMPLES) {
        audioLevelHistoryRef.current.shift();
        // 计算噪音基线（取历史数据的25%分位数）
        const sorted = [...audioLevelHistoryRef.current].sort((a, b) => a - b);
        noiseFloorRef.current = sorted[Math.floor(sorted.length * 0.25)];
      }
      
      // 🎤 动态阈值计算（基于噪音基线）
      const dynamicThreshold = Math.max(
        VAD_CONFIG.SOUND_THRESHOLD, 
        noiseFloorRef.current * 2.5  // 语音应该比噪音高2.5倍
      );
      
      const dynamicSilenceThreshold = Math.max(
        VAD_CONFIG.SILENCE_THRESHOLD,
        noiseFloorRef.current * 1.2  // 静音阈值略高于噪音基线
      );
      
      // 🧠 VAD 状态机逻辑
      const isSound = level > dynamicThreshold;
      const isSilence = level < dynamicSilenceThreshold;
      
      if (isSound) {
        // 🔊 检测到声音
        lastSoundTimeRef.current = now;
        
        // 状态转换：waiting -> speech 或保持 speech
        if (vadStateRef.current !== 'speech') {
          vadStateRef.current = 'speech';
          speechDetectedRef.current = true;
          
          if (!hasAudioInput) {
            setHasAudioInput(true);
            
            // 清除无声音超时
            if (noSoundTimeoutRef.current) {
              clearTimeout(noSoundTimeoutRef.current);
              noSoundTimeoutRef.current = null;
            }
          }
        }
        
        // 清除静音检测定时器
        if (silenceTimeoutRef.current) {
          clearTimeout(silenceTimeoutRef.current);
          silenceTimeoutRef.current = null;
        }
        
        // 重置静音计时
        continuousSilenceDurationRef.current = 0;
        
      } else if (isSilence && speechDetectedRef.current) {
        // 🔇 检测到静音（但之前有过语音）
        const silenceDuration = now - lastSoundTimeRef.current;
        continuousSilenceDurationRef.current = silenceDuration;
        
        // 状态转换：speech -> silence
        if (vadStateRef.current === 'speech') {
          vadStateRef.current = 'silence';
        }
        
        // 录音时间检查
        if (hasAudioInput && isRecording) {
          const recordingDuration = now - recordingStartTimeRef.current;
          
          // 满足最短语音时长要求
          if (recordingDuration > VAD_CONFIG.MIN_SPEECH_DURATION) {
            // 静音持续足够长时间
            if (silenceDuration > VAD_CONFIG.SILENCE_DURATION && !silenceTimeoutRef.current) {
              
              // 确认延迟，避免误触发
              silenceTimeoutRef.current = setTimeout(() => {
                const currentSilence = Date.now() - lastSoundTimeRef.current;
                if (isRecording && currentSilence > VAD_CONFIG.SILENCE_DURATION) {
                  stopListening();
                }
              }, VAD_CONFIG.CONFIRMATION_DELAY);
            }
          }
        }
        
      } else {
        // 📊 中间状态（介于声音和静音之间）
        // 在中间状态时，如果正在说话中，应该重置静音计时
        if (vadStateRef.current === 'speech') {
          // 还在说话范围内，重置静音计时
          lastSoundTimeRef.current = now;
          continuousSilenceDurationRef.current = 0;
          
          // 清除静音检测定时器
          if (silenceTimeoutRef.current) {
            clearTimeout(silenceTimeoutRef.current);
            silenceTimeoutRef.current = null;
          }
        } else {
          // 真正的静音状态
          const silenceDuration = now - lastSoundTimeRef.current;
          continuousSilenceDurationRef.current = silenceDuration;
        }
      }
      
    });

    // 监听 ESC 键关闭窗口
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleCancel();
      }
      // 移除 Enter 键触发，因为我们是全自动的
    };
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      unlistenTrigger.then(fn => fn());
      unlistenHoldStart.then(fn => fn());
      unlistenHoldEnd.then(fn => fn());
      unlistenStreaming.then(fn => fn());
      unlistenStreamingComplete.then(fn => fn());
      unlistenAudioLevel.then(fn => fn());
      document.removeEventListener('keydown', handleKeyDown);
      clearAllTimeouts();
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
      stopPerformanceMonitoring();
    };
  }, [hasAudioInput, isRecording]);

  // 清理所有超时
  const clearAllTimeouts = () => {
    if (noSoundTimeoutRef.current) {
      clearTimeout(noSoundTimeoutRef.current);
      noSoundTimeoutRef.current = null;
    }
    if (silenceTimeoutRef.current) {
      clearTimeout(silenceTimeoutRef.current);
      silenceTimeoutRef.current = null;
    }
    if (autoCloseTimeoutRef.current) {
      clearTimeout(autoCloseTimeoutRef.current);
      autoCloseTimeoutRef.current = null;
    }
    if (processingTimeoutRef.current) {
      clearTimeout(processingTimeoutRef.current);
      processingTimeoutRef.current = null;
    }
  };

  // 重置静音超时（保留但简化，主要逻辑在音频电平监听中）
  const resetSilenceTimeout = () => {
    if (silenceTimeoutRef.current) {
      clearTimeout(silenceTimeoutRef.current);
      silenceTimeoutRef.current = null;
    }
  };

  // 开始监听语音
  const startListening = async () => {
    const startTime = performance.now();
    
    try {
      clearAllTimeouts();
      setState('listening');
      setIsRecording(true);
      setHasAudioInput(false);
      
      // 启动性能监控
      startPerformanceMonitoring();
      
      // 🔄 重置VAD状态和错误处理状态
      recordingStartTimeRef.current = Date.now();
      lastSoundTimeRef.current = Date.now();
      continuousSilenceDurationRef.current = 0;
      speechDetectedRef.current = false;
      vadStateRef.current = 'waiting';
      smoothedLevelRef.current = 0;
      audioLevelHistoryRef.current = [];
      noiseFloorRef.current = 0.03;  // 重置为合理的初始噪音基线
      retryCountRef.current = 0;  // 重置重试计数
      
      
      // 调用后端开始录音（启用实时模式）
      await invoke('start_voice_recording', {
        deviceId: 'default',  // 修复：使用驼峰命名 deviceId 而不是 device_id
        realtime: true
      });
      
      // 设置无声音检测超时（5秒内没有检测到声音则自动关闭）
      noSoundTimeoutRef.current = setTimeout(() => {
        if (!hasAudioInput) {
          handleCancel();
        }
      }, 5000);
      
      // 开始音频反应动画 - 现在通过 Motion 组件处理
      // animateWaveform(); // 移除旧动画，使用新的 Motion 系统
      
      // 测量语音输入启动延迟
      measureVoiceInputLatency(startTime);
    } catch (error) {
      setState('idle');
      setIsRecording(false);
    }
  };

  // 停止监听并处理
  const stopListening = async () => {
    // 防止重复调用
    if (isProcessing) {
      return;
    }
    
    setIsProcessing(true);
    
    try {
      clearAllTimeouts();
      setIsRecording(false);
      
      // 如果没有音频输入，直接关闭
      if (!hasAudioInput) {
        await handleCancel();
        return;
      }
      
      setState('processing');
      
      // 设置处理超时 - 8秒后自动重试或失败
      processingTimeoutRef.current = setTimeout(async () => {
        retryCountRef.current++;
        
        if (retryCountRef.current <= 2) {
          // 最多重试2次
          setTranscribedText(`重试中... (${retryCountRef.current}/2)`);
          
          try {
            // 再次尝试停止录音
            const retryText = await invoke<string>('stop_voice_recording');
            
            // 清除超时
            if (processingTimeoutRef.current) {
              clearTimeout(processingTimeoutRef.current);
              processingTimeoutRef.current = null;
            }
            
            if (retryText && retryText.trim()) {
              setState('injecting');
              setTranscribedText(retryText);
              
              // 先隐藏窗口，恢复原始应用焦点，然后注入文本
              await appWindow.hide();
              await new Promise(resolve => setTimeout(resolve, 300));
              
              // 如果有原始应用信息，激活它
              if (activeApp && activeApp.bundleId) {
                await invoke('activate_app_by_bundle_id', { bundleId: activeApp.bundleId });
                await new Promise(resolve => setTimeout(resolve, 500));
              }
              
              await invoke('inject_text_to_active_app', { 
                text: retryText, 
                targetBundleId: activeApp.bundleId 
              });
              
              // 窗口已隐藏，直接清理状态
              setTimeout(() => {
                setTranscribedText('');
                setHasAudioInput(false);
                setState('idle');
                setIsProcessing(false);
                setIsProcessingTrigger(false);
              }, 100);
            } else {
              closeWindow();
            }
          } catch (retryError) {
            // 继续等待下一次超时重试
          }
        } else {
          // 重试次数用完，优雅失败
          setState('idle');
          setTranscribedText('处理超时，操作已取消');
          setTimeout(() => {
            closeWindow();
          }, 2000);
        }
      }, 8000); // 8秒超时
      
      // 尝试停止录音并获取转录结果
      const finalText = await invoke<string>('stop_voice_recording');
      
      // 如果成功完成，清除超时
      if (processingTimeoutRef.current) {
        clearTimeout(processingTimeoutRef.current);
        processingTimeoutRef.current = null;
      }
      
      
      if (finalText && finalText.trim()) {
        setState('injecting');
        setTranscribedText(finalText);
        
        // 先隐藏窗口
        await appWindow.hide();
        
        // 等待一小段时间确保窗口完全隐藏
        await new Promise(resolve => setTimeout(resolve, 300));
        
        // 如果有原始应用信息，激活它
        if (activeApp && activeApp.bundleId) {
          try {
            await invoke('activate_app_by_bundle_id', { bundleId: activeApp.bundleId });
            // 增加等待时间确保应用完全激活
            await new Promise(resolve => setTimeout(resolve, 800));
          } catch (error) {
            // 激活失败，继续执行
          }
        }
        
        // 注入文本到当前活动应用
        try {
          await invoke('inject_text_to_active_app', { 
            text: finalText, 
            targetBundleId: activeApp.bundleId 
          });
          
          // 额外验证：等待一下看是否真的成功
          await new Promise(resolve => setTimeout(resolve, 300));
        } catch (error) {
          throw error; // 重新抛出错误以便上层处理
        }
        
        // 窗口已隐藏，直接清理状态
        setTimeout(() => {
          setTranscribedText('');
          setHasAudioInput(false);
          setState('idle');
          setIsProcessing(false);
          setIsProcessingTrigger(false);
        }, 100);
      } else {
        // 没有识别到内容，显示失败
        setState('idle');
        setTranscribedText('未识别到语音内容');
        
        // 不再重试stop_voice_recording，因为录音已停止
        setTimeout(() => {
          closeWindow();
        }, 2000);
            
        // 删除重试逻辑，避免重复调用
      }
    } catch (error) {
      setIsProcessing(false);
      // 清除处理超时
      if (processingTimeoutRef.current) {
        clearTimeout(processingTimeoutRef.current);
        processingTimeoutRef.current = null;
      }
      
      
      // 如果还有重试机会，不直接失败
      retryCountRef.current++;
      if (retryCountRef.current <= 2) {
        setState('processing');
        setTranscribedText(`错误恢复中... (${retryCountRef.current}/2)`);
        
        // 延迟重试
        setTimeout(() => {
          stopListening();
        }, 1000);
      } else {
        setState('idle');
        setTranscribedText('转录失败，请重试');
        setTimeout(() => {
          closeWindow();
        }, 2000);
      }
    }
  };

  // 取消操作
  const handleCancel = async () => {
    clearAllTimeouts();
    stopPerformanceMonitoring();
    
    if (isRecording) {
      try {
        await invoke('stop_voice_recording');
      } catch (error) {
      }
    }
    
    setIsRecording(false);
    setState('idle');
    closeWindow();
  };

  // 关闭窗口
  const closeWindow = async () => {
    clearAllTimeouts();
    setTranscribedText('');
    setHasAudioInput(false);
    setState('idle');
    setIsRecording(false);
    setIsProcessing(false);
    setIsProcessingTrigger(false); // 重置触发标志
    await appWindow.hide();
  };

  // 获取应用图标（如果有）
  const getAppIcon = () => {
    if (activeApp.icon) {
      return <img src={activeApp.icon} alt={activeApp.name} className="app-icon" />;
    }
    // 默认图标
    return <div className="app-icon-placeholder">📝</div>;
  };

  // 获取状态文本
  const getStatusText = () => {
    // 只显示真实的转录文本，不显示模拟数据
    if (transcribedText && transcribedText !== '你好，请问有什么可以帮助你的') {
      return transcribedText;
    }
    
    switch (state) {
      case 'listening':
        return hasAudioInput ? '正在聆听...' : '请开始说话...';
      case 'processing':
        return '正在转录...';
      case 'injecting':
        return '正在输入到目标应用...';
      default:
        return '';
    }
  };

  return (
    <div className="macos-voice-input" ref={containerRef}>
      <div 
        className={`voice-input-container ${state === 'listening' ? 'listening' : ''} ${state === 'processing' ? 'processing' : ''} ${state === 'injecting' ? 'success' : ''}`}
      >
        {/* Audio level indicator for reactive effects */}
        <div 
          className="audio-level-indicator" 
          style={{
            opacity: audioLevel > 0.1 ? audioLevel : 0,
            transform: `scale(${1 + audioLevel * 0.2})`
          }}
        />
        
        {/* 左侧 - 应用图标和信息 */}
        <div className="app-info-section">
          <div className="app-icon-wrapper">
            {getAppIcon()}
          </div>
          <div className="app-name">{activeApp.name}</div>
        </div>

        {/* 中间 - 波形和文字显示 */}
        <div className="voice-content-section">
          {state === 'listening' && (
            <div className="waveform-container">
              <div className="waveform-bars">
                {Array.from({ length: 8 }, (_, i) => (
                  <div 
                    key={i}
                    className="waveform-bar"
                    style={{
                      height: isRecording ? `${8 + audioLevel * 12 + Math.random() * 4}px` : '4px',
                      animationDelay: `${i * 0.1}s`
                    }}
                  />
                ))}
              </div>
              <div className={transcribedText ? 'realtime-text' : 'listening-hint'}>
                {getStatusText()}
              </div>
            </div>
          )}

          {state === 'processing' && (
            <div className="processing-container">
              <div className="processing-spinner" />
              <div className="processing-text">
                <span>处理中</span>
                <span className="processing-dots"></span>
              </div>
            </div>
          )}

          {state === 'injecting' && (
            <div className="success-container">
              <div className="success-icon">✓</div>
              <div className="final-text">{transcribedText}</div>
            </div>
          )}
        </div>

        {/* 右侧 - 控制按钮 */}
        <div className="control-section">
          <button 
            className="close-button"
            onClick={handleCancel}
            title="取消 (ESC)"
            disabled={isProcessing}
          >
            ×
          </button>
        </div>
      </div>

      {/* 底部提示 */}
      <div className="bottom-hint">
        <span className="hint-text">
          {hasAudioInput 
            ? '正在聆听，说完请稍候...' 
            : '请开始说话'}
        </span>
      </div>
    </div>
  );
};

export default MacOSVoiceInput;
