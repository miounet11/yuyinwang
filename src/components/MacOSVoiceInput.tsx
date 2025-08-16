import React, { useState, useRef, useEffect } from 'react';
import { appWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
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
  
  const containerRef = useRef<HTMLDivElement>(null);
  const animationRef = useRef<number>();
  const noSoundTimeoutRef = useRef<number | null>(null);
  const silenceTimeoutRef = useRef<number | null>(null);
  const autoCloseTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    // 设置窗口属性 - 模拟 macOS 原生样式
    const setupWindow = async () => {
      await appWindow.setAlwaysOnTop(true);
      await appWindow.setDecorations(false);
      await appWindow.setResizable(false);
      await appWindow.setSkipTaskbar(true);
      
      // 设置窗口大小和位置 - 像 macOS 语音输入一样小巧
      await appWindow.setSize(new LogicalSize(380, 120));
      
      // 居中显示在屏幕底部
      try {
        const screenWidth = window.screen.width;
        const screenHeight = window.screen.height;
        const x = Math.floor((screenWidth - 380) / 2);
        const y = Math.floor(screenHeight - 200); // 屏幕底部位置
        await appWindow.setPosition(new LogicalPosition(x, y));
      } catch (error) {
        console.error('设置窗口位置失败:', error);
      }

      // 获取当前活动应用信息
      try {
        const appInfo = await invoke<ActiveAppInfo>('get_active_app_info_for_voice');
        setActiveApp(appInfo);
      } catch (error) {
        console.error('获取活动应用信息失败:', error);
      }
    };
    
    setupWindow();

    // 监听语音输入触发事件
    const unlistenTrigger = listen('voice_input_triggered', async () => {
      console.log('语音输入被触发');
      setState('idle');
      setTranscribedText('');
      setHasAudioInput(false);
      
      // 重新获取活动应用信息
      try {
        const appInfo = await invoke<ActiveAppInfo>('get_active_app_info_for_voice');
        setActiveApp(appInfo);
      } catch (error) {
        console.error('获取活动应用信息失败:', error);
      }
      
      // 显示窗口并自动开始录音
      await appWindow.show();
      await appWindow.setFocus();
      
      // 延迟一点开始录音，确保窗口已经显示
      setTimeout(() => {
        startListening();
      }, 100);
    });

    // 监听实时转录结果
    const unlistenTranscription = listen<string>('realtime_transcription', (event) => {
      setTranscribedText(event.payload);
      if (event.payload && event.payload.trim()) {
        setHasAudioInput(true);
        resetSilenceTimeout();
      }
    });

    // 监听音频电平
    const unlistenAudioLevel = listen<number>('audio_level', (event) => {
      setAudioLevel(event.payload);
      
      // 如果检测到声音
      if (event.payload > 0.1) {
        if (!hasAudioInput) {
          setHasAudioInput(true);
          // 清除无声音超时
          if (noSoundTimeoutRef.current) {
            clearTimeout(noSoundTimeoutRef.current);
            noSoundTimeoutRef.current = null;
          }
        }
        // 重置静音超时
        resetSilenceTimeout();
      }
    });

    // 监听 ESC 键关闭窗口
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleCancel();
      } else if (e.key === 'Enter' && isRecording) {
        stopListening();
      }
    };
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      unlistenTrigger.then(fn => fn());
      unlistenTranscription.then(fn => fn());
      unlistenAudioLevel.then(fn => fn());
      document.removeEventListener('keydown', handleKeyDown);
      clearAllTimeouts();
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
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
  };

  // 重置静音超时
  const resetSilenceTimeout = () => {
    if (silenceTimeoutRef.current) {
      clearTimeout(silenceTimeoutRef.current);
    }
    
    // 2秒静音后自动完成
    silenceTimeoutRef.current = setTimeout(() => {
      if (hasAudioInput && isRecording) {
        console.log('检测到静音，自动完成转录');
        stopListening();
      }
    }, 2000);
  };

  // 开始监听语音
  const startListening = async () => {
    try {
      clearAllTimeouts();
      setState('listening');
      setIsRecording(true);
      setHasAudioInput(false);
      
      // 调用后端开始录音（启用实时模式）
      await invoke('start_voice_recording', {
        device_id: 'default',
        realtime: true
      });
      
      // 设置无声音检测超时（3秒内没有检测到声音则自动关闭）
      noSoundTimeoutRef.current = setTimeout(() => {
        console.log('检查声音输入状态:', hasAudioInput);
        if (!hasAudioInput) {
          console.log('3秒内未检测到声音，自动关闭窗口');
          handleCancel();
        }
      }, 3000);
      
      // 开始音频波形动画
      animateWaveform();
    } catch (error) {
      console.error('开始录音失败:', error);
      setState('idle');
      setIsRecording(false);
    }
  };

  // 停止监听并处理
  const stopListening = async () => {
    try {
      clearAllTimeouts();
      setIsRecording(false);
      
      // 如果没有音频输入，直接关闭
      if (!hasAudioInput) {
        await handleCancel();
        return;
      }
      
      setState('processing');
      
      // 停止录音并获取转录结果
      const finalText = await invoke<string>('stop_voice_recording');
      
      if (finalText && finalText.trim()) {
        setState('injecting');
        setTranscribedText(finalText);
        
        // 注入文本到当前应用
        await invoke('inject_text_to_active_app', { text: finalText });
        
        // 显示成功状态后自动关闭
        autoCloseTimeoutRef.current = setTimeout(() => {
          closeWindow();
        }, 800);
      } else {
        // 没有识别到内容，直接关闭
        closeWindow();
      }
    } catch (error) {
      console.error('处理录音失败:', error);
      setState('idle');
      closeWindow();
    }
  };

  // 取消操作
  const handleCancel = async () => {
    clearAllTimeouts();
    
    if (isRecording) {
      try {
        await invoke('stop_voice_recording');
      } catch (error) {
        console.error('停止录音失败:', error);
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
    await appWindow.hide();
  };

  // 音频波形动画
  const animateWaveform = () => {
    if (!isRecording) return;
    
    // 更新波形动画
    const bars = containerRef.current?.querySelectorAll('.waveform-bar');
    if (bars) {
      bars.forEach((bar: any) => {
        const height = 20 + audioLevel * 30 + Math.random() * 10;
        bar.style.height = `${height}px`;
      });
    }
    
    animationRef.current = requestAnimationFrame(animateWaveform);
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
    if (transcribedText) return transcribedText;
    
    switch (state) {
      case 'listening':
        return hasAudioInput ? '正在聆听...' : '请开始说话...';
      case 'processing':
        return '处理中...';
      case 'injecting':
        return '正在输入...';
      default:
        return '准备就绪';
    }
  };

  return (
    <div className="macos-voice-input" ref={containerRef}>
      <div className="voice-input-container">
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
                {[...Array(20)].map((_, i) => (
                  <div 
                    key={i} 
                    className="waveform-bar"
                    style={{
                      animationDelay: `${i * 0.05}s`
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
              <div className="processing-text">处理中...</div>
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
          {state === 'listening' && isRecording && hasAudioInput && (
            <button 
              className="done-button"
              onClick={stopListening}
              title="完成 (Enter)"
            >
              完成
            </button>
          )}
          
          <button 
            className="close-button"
            onClick={handleCancel}
            title="关闭 (ESC)"
          >
            ×
          </button>
        </div>
      </div>

      {/* 底部提示 */}
      <div className="bottom-hint">
        <span className="hint-text">
          {hasAudioInput 
            ? '说完后点击"完成"或等待自动识别' 
            : '请开始说话，3秒内无声音将自动关闭'}
        </span>
      </div>
    </div>
  );
};

export default MacOSVoiceInput;