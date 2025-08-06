import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { register, unregisterAll } from '@tauri-apps/api/globalShortcut';
import './App.css';

// Zustand Store
import { create } from 'zustand';

interface TranscriptionEntry {
  id: string;
  text: string;
  timestamp: number;
  duration: number;
  model: string;
  confidence: number;
}

interface AudioDevice {
  name: string;
  id: string;
  is_default: boolean;
  is_available: boolean;
}

interface McpConfig {
  enabled: boolean;
  server_url: string;
  api_key: string;
  model: string;
}

interface AppStore {
  isRecording: boolean;
  transcriptionText: string;
  audioDevices: AudioDevice[];
  selectedDevice: string | null;
  language: string;
  hotkey: string;
  currentPage: string;
  selectedModel: string;
  transcriptionHistory: TranscriptionEntry[];
  mcpConfig: McpConfig;
  setRecording: (value: boolean) => void;
  setTranscription: (text: string) => void;
  setDevices: (devices: AudioDevice[]) => void;
  setSelectedDevice: (device: string) => void;
  setLanguage: (lang: string) => void;
  setHotkey: (key: string) => void;
  setCurrentPage: (page: string) => void;
  setSelectedModel: (model: string) => void;
  setTranscriptionHistory: (history: TranscriptionEntry[]) => void;
  addTranscriptionEntry: (entry: TranscriptionEntry) => void;
  setMcpConfig: (config: McpConfig) => void;
}

const useStore = create<AppStore>((set) => ({
  isRecording: false,
  transcriptionText: '',
  audioDevices: [],
  selectedDevice: null,
  language: 'en',
  hotkey: 'CommandOrControl+Shift+Space',
  currentPage: 'general',
  selectedModel: 'gpt-4o-mini',
  transcriptionHistory: [],
  mcpConfig: {
    enabled: true,
    server_url: 'https://api.openai.com/v1',
    api_key: '',
    model: 'whisper-1',
  },
  setRecording: (value) => set({ isRecording: value }),
  setTranscription: (text) => set({ transcriptionText: text }),
  setDevices: (devices) => set({ audioDevices: devices }),
  setSelectedDevice: (device) => set({ selectedDevice: device }),
  setLanguage: (lang) => set({ language: lang }),
  setHotkey: (key) => set({ hotkey: key }),
  setCurrentPage: (page) => set({ currentPage: page }),
  setSelectedModel: (model) => set({ selectedModel: model }),
  setTranscriptionHistory: (history) => set({ transcriptionHistory: history }),
  addTranscriptionEntry: (entry) => set((state) => ({
    transcriptionHistory: [entry, ...state.transcriptionHistory]
  })),
  setMcpConfig: (config) => set({ mcpConfig: config }),
}));

// 导航菜单项
const navigationItems = [
  { id: 'general', label: '常规设置', icon: '⚙️' },
  { id: 'transcription', label: '听写模型', icon: '🎤' },
  { id: 'files', label: '转录文件', icon: '📁' },
  { id: 'history', label: '历史记录', icon: '📋' },
  { id: 'shortcuts', label: '快捷键', icon: '⌨️' },
  { id: 'ai-prompts', label: 'AI提示', icon: '🤖' },
  { id: 'contact', label: '联系我们', icon: '📧' },
];

// AI模型列表
const aiModels = [
  {
    id: 'nova-3',
    name: 'Online Real-time Nova-3 (English Only)',
    provider: 'Deepgram Nova-3',
    description: '实时听写具有卓越准确性。纯英语语优化版本。',
    accuracy: 5,
    speed: 5,
    languages: ['仅英语'],
    realtime: true,
    recommended: false,
    icon: '🚀'
  },
  {
    id: 'gpt-4o-mini',
    name: 'Online GPT-4o mini Transcribe',
    provider: 'OpenAI GPT-4o mini',
    description: '卓越准确性和快速处理。比Whisper或Nova模型更准确。',
    accuracy: 5,
    speed: 3,
    languages: ['多语言'],
    realtime: false,
    recommended: true,
    icon: '⚡'
  },
  {
    id: 'voxtral-mini',
    name: 'Online Voxtral Mini',
    provider: 'Mistral AI',
    description: 'fast and accurate transcription model with excellent multilingual support. Delivers high-quality results comparable to GPT-4o mini.',
    accuracy: 4,
    speed: 4,
    languages: ['多语言'],
    realtime: false,
    recommended: false,
    icon: '🌟'
  },
  {
    id: 'elevenlabs',
    name: 'Online ElevenLabs Scribe',
    provider: 'ElevenLabs Scribe',
    description: '高质量录制配备先进语言识别和多语言支持。',
    accuracy: 4,
    speed: 3,
    languages: ['多语言'],
    realtime: false,
    recommended: false,
    icon: '🔊'
  }
];

// 开关组件
const Toggle: React.FC<{ checked: boolean; onChange: (checked: boolean) => void; label: string }> = 
  ({ checked, onChange, label }) => (
  <div className="toggle-group">
    <span className="toggle-label">{label}</span>
    <div 
      className={`toggle ${checked ? 'toggle-on' : 'toggle-off'}`}
      onClick={() => onChange(!checked)}
    >
      <div className="toggle-thumb"></div>
    </div>
  </div>
);

// 页面组件
const PageContent: React.FC<{ page: string }> = ({ page }) => {
  const {
    isRecording,
    transcriptionText,
    audioDevices,
    selectedDevice,
    language,
    hotkey,
    setRecording,
    setTranscription,
    setDevices,
    setSelectedDevice,
    setLanguage,
    setHotkey,
  } = useStore();

  const [selectedModel, setSelectedModel] = useState('gpt-4o-mini');
  const [loginOnStartup, setLoginOnStartup] = useState(false);
  const [showInDock, setShowInDock] = useState(false);
  const [showInStatusBar, setShowInStatusBar] = useState(true);
  const [playbackEffects, setPlaybackEffects] = useState(true);
  const [recordingMute, setRecordingMute] = useState(true);
  const [touchBarFeedback, setTouchBarFeedback] = useState(true);

  // 切换录音状态
  const handleToggleRecording = async () => {
    if (isRecording) {
      try {
        await invoke('stop_recording');
        setRecording(false);
        
        // 获取转录结果
        const result = await invoke<TranscriptionEntry>('transcribe_with_mcp', {
          audioData: new Uint8Array(1024), // 模拟音频数据
          model: selectedModel
        });
        
        setTranscription(result.text);
        addTranscriptionEntry(result);
        
        // 保存到历史记录
        await invoke('add_transcription_entry', { entry: result });
      } catch (error) {
        console.error('停止录音失败:', error);
      }
    } else {
      try {
        await invoke('start_recording');
        setRecording(true);
      } catch (error) {
        console.error('开始录音失败:', error);
      }
    }
  };

  // 更新设置
  const handleUpdateSettings = async () => {
    await invoke('update_settings', {
      language,
      hotkey,
      device: selectedDevice,
    });
  };

  // 获取转录结果
  const getTranscriptionResult = async () => {
    try {
      const result = await invoke<string>('get_transcription_result');
      setTranscription(result);
    } catch (error) {
      console.error('获取转录结果失败:', error);
    }
  };

  switch (page) {
    case 'general':
      return (
        <div className="page-content">
          <div className="page-header">
            <h1>常规首选项</h1>
            <p>根据您的工作流程和偏好配置 Spokenly。</p>
          </div>

          <div className="section">
            <h2>行为</h2>
            <div className="settings-list">
              <Toggle
                checked={loginOnStartup}
                onChange={setLoginOnStartup}
                label="登录时启动"
              />
              <Toggle
                checked={showInDock}
                onChange={setShowInDock}
                label="在程序坞中显示"
              />
              <Toggle
                checked={showInStatusBar}
                onChange={setShowInStatusBar}
                label="在状态栏中显示"
              />
            </div>

            <div className="form-group">
              <label>应用界面语言</label>
              <select value="zh" className="select-field">
                <option value="zh">System Default</option>
                <option value="en">English</option>
                <option value="zh">中文</option>
              </select>
            </div>
          </div>

          <div className="section">
            <h2>麦克风优先级设置</h2>
            <div className="device-list">
              {audioDevices.map((device, index) => (
                <div key={device.id} className="device-item">
                  <div className="device-info">
                    <div className="device-icon">🎤</div>
                    <span>{index + 1}. {device.name}</span>
                  </div>
                  <div className={`device-status ${device.is_available ? 'online' : 'offline'}`}></div>
                </div>
              ))}
            </div>
            <p className="device-note">麦克风按优先级顺序会依次使用。推动可重新排序。</p>
          </div>

          <div className="section">
            <h2>音频和反馈</h2>
            <div className="settings-list">
              <Toggle
                checked={playbackEffects}
                onChange={setPlaybackEffects}
                label="播放声音效果"
              />
              <Toggle
                checked={recordingMute}
                onChange={setRecordingMute}
                label="录音时静音"
              />
              <Toggle
                checked={touchBarFeedback}
                onChange={setTouchBarFeedback}
                label="启用触控板反馈"
              />
            </div>
          </div>
        </div>
      );

    case 'transcription':
      return (
        <div className="page-content">
          <div className="page-header">
            <h1>听写模型</h1>
            <p>从各种听写模型中选择 - 从云端选项到离线工作的本地模型。选择最适合您听写需求的准确性、隐私性和速度的平衡。</p>
          </div>

          <div className="model-tabs">
            <button className="tab active">全部</button>
            <button className="tab">在线</button>
            <button className="tab">本地</button>
            <button className="tab">API</button>
            <button className="tab">快速</button>
            <button className="tab">准确</button>
            <button className="tab">标点符号</button>
            <button className="tab">字幕</button>
          </div>

          <p className="models-description">
            需要互联网连接的基于云的模型。这些模型通常提供更高的准确性，但依赖于网络可用性。
          </p>

          <div className="models-list">
            {aiModels.map((model) => (
              <div 
                key={model.id} 
                className={`model-card ${selectedModel === model.id ? 'selected' : ''} ${model.recommended ? 'recommended' : ''}`}
                onClick={() => setSelectedModel(model.id)}
              >
                <div className="model-header">
                  <div className="model-icon">{model.icon}</div>
                  <div className="model-info">
                    <div className="model-title">
                      <h3>{model.name}</h3>
                      {model.recommended && <span className="badge recommended">最准确</span>}
                      {model.realtime && <span className="badge">实时</span>}
                    </div>
                    <p className="model-provider">由{model.provider}驱动 - {model.description}</p>
                  </div>
                  {selectedModel === model.id && <div className="selected-indicator">✓</div>}
                </div>
                
                <div className="model-stats">
                  <div className="stat">
                    <span className="stat-label">准确度</span>
                    <div className="stat-dots">
                      {[...Array(5)].map((_, i) => (
                        <div key={i} className={`dot ${i < model.accuracy ? 'active' : ''}`}></div>
                      ))}
                    </div>
                  </div>
                  <div className="stat">
                    <span className="stat-label">速度</span>
                    <div className="stat-dots">
                      {[...Array(5)].map((_, i) => (
                        <div key={i} className={`dot ${i < model.speed ? 'active' : ''}`}></div>
                      ))}
                    </div>
                  </div>
                  <div className="stat">
                    <span className="stat-label">{model.languages.join(', ')}</span>
                  </div>
                  {model.realtime && (
                    <div className="stat">
                      <span className="stat-label">实时</span>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      );

    case 'files':
      return (
        <div className="page-content">
          <div className="page-header">
            <h1>转录文件</h1>
            <p>将音频或视频文件转换为文本。Spokenly 将为您进行转录。</p>
          </div>

          <div className="file-upload-area">
            <div className="upload-zone">
              <div className="upload-icon">📁</div>
              <h3>将文件拖放到此处</h3>
              <div className="file-types">
                <span className="file-type">MP3</span>
                <span className="file-type">WAV</span>
                <span className="file-type">M4A</span>
                <span className="file-type">FLAC</span>
                <span className="file-type">MP4</span>
                <span className="file-type">MOV</span>
                <span className="file-type">M4V</span>
              </div>
            </div>
            
            <div className="model-info">
              <p>Online Whisper v3 Turbo</p>
            </div>

            <div className="file-actions">
              <button className="action-btn">
                <span>🎤</span>
                录制音频
              </button>
              <button className="action-btn">
                <span>🔄</span>
                更换模型
              </button>
              <button className="action-btn">
                <span>⚙️</span>
                本地 Whisper 设置
              </button>
              <button className="action-btn">
                <span>↗</span>
                导入 Spokenly 项目
              </button>
            </div>
          </div>
        </div>
      );

    case 'history':
      return (
        <div className="page-content">
          <div className="page-header">
            <h1>历史记录</h1>
            <p>查看存储在您电脑上的转录历史记录</p>
          </div>

          <div className="history-controls">
            <div className="filter-tabs">
              <button className="filter-tab active">全部</button>
              <button className="filter-tab">听写</button>
              <button className="filter-tab">文件</button>
              <button className="filter-tab">日记</button>
            </div>
            <div className="history-actions">
              <button className="action-btn">选择</button>
              <button className="action-btn">设置</button>
            </div>
          </div>

          <div className="search-bar">
            <input type="text" placeholder="搜索" className="search-input" />
            <select className="sort-select">
              <option>最新的在前</option>
              <option>最旧的在前</option>
              <option>按名称排序</option>
            </select>
          </div>

          <div className="history-list">
            {transcriptionHistory.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">📋</div>
                <h3>暂无转录记录</h3>
                <p>开始录音后，转录记录将显示在这里</p>
              </div>
            ) : (
              transcriptionHistory.map((entry) => {
                const timeAgo = Math.floor((Date.now() - entry.timestamp * 1000) / 1000);
                const timeLabel = timeAgo < 60 ? `${timeAgo}s ago` : 
                                 timeAgo < 3600 ? `${Math.floor(timeAgo / 60)}m ago` : 
                                 `${Math.floor(timeAgo / 3600)}h ago`;
                
                return (
                  <div key={entry.id} className="history-item">
                    <div className="history-icon">🎤</div>
                    <div className="history-content">
                      <div className="history-text">
                        {entry.text}
                      </div>
                      <div className="history-meta">
                        <span className="history-type">听写</span>
                        <span className="history-time">{timeLabel}</span>
                        <span className="history-duration">{entry.duration} seconds</span>
                        <span className="history-model">{entry.model}</span>
                        <span className="history-confidence">{Math.round(entry.confidence * 100)}%</span>
                      </div>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      );

    case 'shortcuts':
      return (
        <div className="page-content">
          <div className="page-header">
            <h1>快捷键</h1>
            <p>选择您喜欢的键盘修饰键来启动 Spokenly。仅按这些修饰键即可开始录音。</p>
          </div>

          <div className="section">
            <h2>录音快捷键</h2>
            <button className="add-shortcut">+</button>
            
            <div className="shortcut-item">
              <div className="shortcut-display">
                <span className="shortcut-key">快捷键</span>
                <div className="shortcut-combo">
                  <select className="key-select">
                    <option>按住或切换</option>
                    <option>单击</option>
                    <option>双击</option>
                  </select>
                  <select className="key-select">
                    <option>Fn</option>
                    <option>Cmd</option>
                    <option>Ctrl</option>
                    <option>Alt</option>
                  </select>
                </div>
              </div>
            </div>

            <p className="shortcut-description">
              配置快捷键及其激活方式：按住或切换（自动停止）、切换（点击开始/停止），按住（按下时录音）或双击（快速按两次）。
            </p>
          </div>

          <div className="section">
            <h2>提示</h2>
            <div className="warning-box">
              <div className="warning-icon">⚠️</div>
              <div className="warning-content">
                <h3>使用 Fn 键</h3>
                <p>要单独使用 Fn 键：</p>
                <ul>
                  <li>• 打开系统设置 → 键盘</li>
                  <li>• 点击"按下 🌐 键以"下拉菜单</li>
                  <li>• 选择"无操作"</li>
                  <li>• 这允许 Spokenly 检测 Fn 键按下</li>
                </ul>
              </div>
            </div>
          </div>

          <div className="section">
            <h2>测试您的快捷键</h2>
            <div className="test-area">
              <div className="test-instruction">
                🎤 首先点击下方的文本框。
              </div>
              <div className="test-input">
                <textarea 
                  placeholder="在此处测试快捷键..."
                  className="test-textarea"
                />
              </div>
            </div>
          </div>
        </div>
      );

    case 'ai-prompts':
      return (
        <div className="page-content">
          <div className="page-header">
            <h1>AI提示</h1>
            <p>配置AI辅助功能和自定义提示</p>
          </div>
          
          <div className="empty-state">
            <div className="empty-icon">🤖</div>
            <h3>AI功能开发中</h3>
            <p>AI辅助功能和自定义提示即将推出</p>
          </div>
        </div>
      );

    case 'contact':
      return (
        <div className="page-content">
          <div className="page-header">
            <h1>联系我们</h1>
            <p>获取帮助和支持</p>
          </div>

          <div className="contact-info">
            <div className="contact-item">
              <div className="contact-icon">📧</div>
              <div className="contact-details">
                <h3>技术支持</h3>
                <p>support@spokenly.com</p>
              </div>
            </div>
            
            <div className="contact-item">
              <div className="contact-icon">🌐</div>
              <div className="contact-details">
                <h3>官方网站</h3>
                <p>https://spokenly.com</p>
              </div>
            </div>
          </div>
        </div>
      );

    default:
      return <div className="page-content">页面未找到</div>;
  }
};

// 主应用组件
function App() {
  const {
    audioDevices,
    currentPage,
    setDevices,
    setCurrentPage,
    setTranscriptionHistory,
    setTranscription,
    addTranscriptionEntry,
  } = useStore();

  // 初始化
  useEffect(() => {
    const initializeApp = async () => {
      try {
        // 获取音频设备列表
        const devices = await invoke<AudioDevice[]>('get_audio_devices');
        setDevices(devices);

        // 加载转录历史
        const history = await invoke<TranscriptionEntry[]>('get_transcription_history');
        setTranscriptionHistory(history);

        console.log('✅ 应用初始化完成');
      } catch (error) {
        console.error('初始化失败:', error);
      }
    };

    // 监听转录结果
    const setupListeners = async () => {
      try {
        const unlisten = await listen<TranscriptionEntry>('transcription_result', (event) => {
          const entry = event.payload;
          setTranscription(entry.text);
          addTranscriptionEntry(entry);
        });

        return () => {
          unlisten();
          unregisterAll();
        };
      } catch (error) {
        console.error('设置监听器失败:', error);
      }
    };

    initializeApp();
    setupListeners();
  }, [setDevices, setTranscriptionHistory, setTranscription, addTranscriptionEntry]);

  return (
    <div className="app">
      {/* 侧边栏 */}
      <div className="sidebar">
        <div className="sidebar-header">
          <div className="app-logo">
            <span className="logo-icon">🎤</span>
            <span className="logo-text">Spokenly</span>
          </div>
        </div>

        <nav className="sidebar-nav">
          {navigationItems.map((item) => (
            <button
              key={item.id}
              className={`nav-item ${currentPage === item.id ? 'active' : ''}`}
              onClick={() => setCurrentPage(item.id)}
            >
              <span className="nav-icon">{item.icon}</span>
              <span className="nav-label">{item.label}</span>
            </button>
          ))}
        </nav>

        <div className="sidebar-footer">
          <div className="usage-info">
            <div className="usage-text">剩余免费使用量：98%</div>
            <div className="upgrade-btn">🔥 升级到 Pro</div>
          </div>
          <div className="version-info">v2.12.10 (234)</div>
        </div>
      </div>

      {/* 主内容区域 */}
      <div className="main-content">
        <PageContent page={currentPage} />
      </div>
    </div>
  );
}

export default App;