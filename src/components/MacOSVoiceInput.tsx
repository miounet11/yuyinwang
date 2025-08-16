import React, { useState, useRef, useEffect } from 'react';
import { appWindow, LogicalPosition } from '@tauri-apps/api/window';
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
  const [state, setState] = useState<InputState>('listening');
  const [transcribedText, setTranscribedText] = useState('');
  const [activeApp, setActiveApp] = useState<ActiveAppInfo>({ name: '未知应用' });
  const [audioLevel, setAudioLevel] = useState(0);
  const [isRecording, setIsRecording] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const animationRef = useRef<number>();

  useEffect(() => {
    // 设置窗口属性 - 模拟 macOS 原生样式
    const setupWindow = async () => {
      await appWindow.setAlwaysOnTop(true);
      await appWindow.setDecorations(false);
      await appWindow.setResizable(false);
      await appWindow.setSkipTaskbar(true);
      
      // 设置窗口大小和位置 - 像 macOS 语音输入一样小巧
      await appWindow.setSize({ width: 380, height: 120 });
      
      // 居中显示在屏幕上方
      try {
        const screenWidth = window.screen.width;
        const screenHeight = window.screen.height;
        const x = Math.floor((screenWidth - 380) / 2);
        const y = Math.floor(screenHeight * 0.2); // 屏幕上方 20% 位置
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

      // 自动开始录音
      startListening();
    };
    
    setupWindow();

    // 监听语音输入触发事件
    const unlistenTrigger = listen('voice_input_triggered', () => {
      setState('listening');
      setTranscribedText('');
      startListening();
    });

    // 监听实时转录结果
    const unlistenTranscription = listen<string>('realtime_transcription', (event) => {
      setTranscribedText(event.payload);
    });

    // 监听音频电平
    const unlistenAudioLevel = listen<number>('audio_level', (event) => {
      setAudioLevel(event.payload);
    });

    // 监听 ESC 键关闭窗口
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        stopListening();
        appWindow.hide();
      }
    };
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      unlistenTrigger.then(fn => fn());
      unlistenTranscription.then(fn => fn());
      unlistenAudioLevel.then(fn => fn());
      document.removeEventListener('keydown', handleKeyDown);
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  // 开始监听语音
  const startListening = async () => {
    try {
      setState('listening');
      setIsRecording(true);
      await invoke('start_voice_recording', { 
        deviceId: 'default',
        realtime: true 
      });
      
      // 开始音频波形动画
      animateWaveform();
    } catch (error) {
      console.error('开始录音失败:', error);
      setState('idle');
    }
  };

  // 停止监听并处理
  const stopListening = async () => {
    try {
      setIsRecording(false);
      setState('processing');
      
      const finalText = await invoke<string>('stop_voice_recording');
      
      if (finalText && finalText.trim()) {
        setState('injecting');
        setTranscribedText(finalText);
        
        // 注入文本到当前应用
        await invoke('inject_text_to_active_app', { text: finalText });
        
        // 显示成功状态后关闭
        setTimeout(() => {
          appWindow.hide();
          setState('idle');
          setTranscribedText('');
        }, 800);
      } else {
        // 没有识别到内容，直接关闭
        appWindow.hide();
        setState('idle');
      }
    } catch (error) {
      console.error('处理录音失败:', error);
      setState('idle');
    }
  };

  // 音频波形动画
  const animateWaveform = () => {
    if (!isRecording) return;
    
    // 这里可以添加波形动画逻辑
    animationRef.current = requestAnimationFrame(animateWaveform);
  };

  // 点击完成按钮
  const handleDoneClick = () => {
    stopListening();
  };

  // 获取应用图标（如果有）
  const getAppIcon = () => {
    if (activeApp.icon) {
      return <img src={activeApp.icon} alt={activeApp.name} className="app-icon" />;
    }
    // 默认图标
    return <div className="app-icon-placeholder">📝</div>;
  };

  return (
    <div className="macos-voice-input">
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
                      height: `${20 + audioLevel * 30 + Math.random() * 10}px`,
                      animationDelay: `${i * 0.05}s`
                    }}
                  />
                ))}
              </div>
              {transcribedText && (
                <div className="realtime-text">{transcribedText}</div>
              )}
              {!transcribedText && (
                <div className="listening-hint">正在聆听...</div>
              )}
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
          {state === 'listening' && (
            <button 
              className="done-button"
              onClick={handleDoneClick}
              title="完成"
            >
              完成
            </button>
          )}
          
          <button 
            className="close-button"
            onClick={() => {
              stopListening();
              appWindow.hide();
            }}
            title="关闭"
          >
            ×
          </button>
        </div>
      </div>

      {/* 底部提示 */}
      <div className="bottom-hint">
        <span className="hint-text">说完后点击"完成"或按 ESC 退出</span>
      </div>
    </div>
  );
};

export default MacOSVoiceInput;