import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { unregisterAll } from '@tauri-apps/api/globalShortcut';
import { open } from '@tauri-apps/api/dialog';
import './App.css';
import './styles/micro-interactions.css';

// Components
import FloatingDialog from './components/FloatingDialog';
import AppSelector from './components/AppSelector';
import ShortcutEditor from './components/ShortcutEditor';
import HistorySettings from './components/HistorySettings';
import TranscriptionModelsPage from './components/TranscriptionModelsPage';
import FeatureTestPanel from './components/FeatureTestPanel';
import PermissionSettings from './components/PermissionSettings';
import PermissionIndicator from './components/PermissionIndicator';
import FirstLaunchWizard from './components/FirstLaunchWizard';
import SubscriptionManager from './components/SubscriptionManager';
import AIPrompts from './components/AIPrompts';
import { shortcutManager } from './utils/shortcutManager';
import { permissionManager } from './utils/permissionManager';
// import SystemChecker from './utils/systemCheck';
import { ttsService } from './services/ttsService';

// Types and Stores
// import { ApiConfig } from './types/models';
// import { useModelsStore } from './stores/modelsStore';

// Zustand Store
import { create } from 'zustand';

interface TranscriptionEntry {
  id: string;
  text: string;
  timestamp: number;
  duration: number;
  model: string;
  confidence: number;
  audio_file_path?: string;
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
  showFloatingDialog: boolean;
  aiProcessingActive: boolean;
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
  setShowFloatingDialog: (show: boolean) => void;
  setAiProcessingActive: (active: boolean) => void;
}

const useStore = create<AppStore>((set) => ({
  isRecording: false,
  transcriptionText: '',
  audioDevices: [],
  selectedDevice: null,
  language: 'en',
  hotkey: 'CommandOrControl+Shift+Space',
  currentPage: 'general',
  selectedModel: 'whisper-1', // 默认使用听写模型
  transcriptionHistory: [],
  mcpConfig: {
    enabled: true,
    server_url: 'https://ttkk.inping.com/v1', // 使用免费 TTS API
    api_key: 'sk-vJToQKskNEIaFNM3GjTGh1YrN9kGZ33pw2D1AEZUXL0prLjw',
    model: 'whisper-1',
  },
  showFloatingDialog: false,
  aiProcessingActive: false,
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
  setShowFloatingDialog: (show) => set({ showFloatingDialog: show }),
  setAiProcessingActive: (active) => set({ aiProcessingActive: active }),
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
/* const aiModels = [
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
]; */

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
const PageContent: React.FC<{ 
  page: string;
  setShowShortcutEditor?: (show: boolean) => void;
  setShowAppSelector?: (show: boolean) => void;
  setShowHistorySettings?: (show: boolean) => void;
  audioDevices?: AudioDevice[];
  trialInfo?: any;
  setShowSubscriptionManager?: (show: boolean) => void;
  onEnhancedTextReady?: (text: string) => void;
  isRecording?: boolean;
}> = ({ page, setShowShortcutEditor, setShowAppSelector, setShowHistorySettings, audioDevices = [], onEnhancedTextReady, isRecording }) => {
  const {
    transcriptionText,
    transcriptionHistory,
    setTranscription,
    setTranscriptionHistory,
  } = useStore();

  const [selectedModel] = useState('gpt-4o-mini');
  const [loginOnStartup, setLoginOnStartup] = useState(false);
  const [showInDock, setShowInDock] = useState(false);
  const [showInStatusBar, setShowInStatusBar] = useState(true);
  const [playbackEffects, setPlaybackEffects] = useState(true);
  const [recordingMute, setRecordingMute] = useState(true);
  const [touchBarFeedback, setTouchBarFeedback] = useState(true);
  const [isUploading, setIsUploading] = useState(false);
  const [supportedFormats, setSupportedFormats] = useState<string[]>([]);


  // 获取支持的文件格式
  const getSupportedFormats = async () => {
    try {
      const formats = await invoke<string[]>('get_supported_formats');
      setSupportedFormats(formats);
    } catch (error) {
      console.error('获取支持格式失败:', error);
    }
  };

  // 文件上传处理
  const handleFileUpload = async () => {
    try {
      setIsUploading(true);
      
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: '音频/视频文件',
            extensions: supportedFormats.length > 0 ? supportedFormats : ['mp3', 'wav', 'm4a', 'flac', 'mp4', 'mov', 'm4v']
          }
        ]
      });

      if (selected && typeof selected === 'string') {
        console.log('选择的文件:', selected);
        
        const result = await invoke<string>('upload_file', { 
          filePath: selected 
        });
        
        console.log('上传结果:', result);
        
        // 显示上传成功消息
        setTranscription(`文件上传成功: ${selected.split('/').pop()}`);
      }
    } catch (error) {
      console.error('文件上传失败:', error);
      setTranscription(`文件上传失败: ${error}`);
    } finally {
      setIsUploading(false);
    }
  };

  // 删除转录记录
  const handleDeleteEntry = async (entryId: string) => {
    try {
      await invoke('delete_file', { entryId });
      // 刷新历史记录
      const history = await invoke<TranscriptionEntry[]>('get_transcription_history');
      setTranscriptionHistory(history);
    } catch (error) {
      console.error('删除记录失败:', error);
    }
  };

  // 导出转录结果
  const handleExportEntry = async (entryId: string, format: string) => {
    try {
      const exportPath = await invoke<string>('export_transcription', { 
        entryId, 
        exportFormat: format 
      });
      console.log('导出成功:', exportPath);
      setTranscription(`导出成功: ${exportPath}`);
    } catch (error) {
      console.error('导出失败:', error);
    }
  };

  // 处理AI助手提示 - 暂时注释掉，后续可能需要
  // const handleSubmitPromptLocal = async (prompt: string) => {
  //   console.log('AI助手提示:', prompt);
  //   setTranscription(`AI助手处理: ${prompt}`);
  // };
  
  // 如果有外部传入的handleSubmitPrompt则使用，否则使用本地的
  // const submitPrompt = handleSubmitPrompt || handleSubmitPromptLocal;

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
              <select defaultValue="zh" className="select-field" onChange={(e) => console.log('语言切换:', e.target.value)}>
                <option value="system">System Default</option>
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
      return <TranscriptionModelsPage />;

    case 'files':
      return (
        <div className="page-content">
          <div className="page-header">
            <h1>转录文件</h1>
            <p>将音频或视频文件转换为文本。Spokenly 将为您进行转录。</p>
          </div>

          <div className="file-upload-area">
            <div 
              className={`upload-zone ${isUploading ? 'uploading' : ''}`}
              onClick={handleFileUpload}
              style={{ cursor: isUploading ? 'not-allowed' : 'pointer' }}
            >
              <div className="upload-icon">
                {isUploading ? '⏳' : '📁'}
              </div>
              <h3>
                {isUploading ? '正在上传文件...' : '点击选择文件或将文件拖放到此处'}
              </h3>
              <div className="file-types">
                {supportedFormats.length > 0 ? (
                  supportedFormats.map(format => (
                    <span key={format} className="file-type">{format.toUpperCase()}</span>
                  ))
                ) : (
                  <>
                    <span className="file-type">MP3</span>
                    <span className="file-type">WAV</span>
                    <span className="file-type">M4A</span>
                    <span className="file-type">FLAC</span>
                    <span className="file-type">MP4</span>
                    <span className="file-type">MOV</span>
                    <span className="file-type">M4V</span>
                  </>
                )}
              </div>
              {transcriptionText && (
                <div className="upload-status">
                  <p>{transcriptionText}</p>
                </div>
              )}
            </div>
            
            <div className="model-info">
              <p>当前模型: {selectedModel}</p>
            </div>

            <div className="file-actions">
              <button 
                className="action-btn"
                onClick={handleFileUpload}
                disabled={isUploading}
              >
                <span>📁</span>
                {isUploading ? '上传中...' : '选择文件'}
              </button>
              <button className="action-btn" onClick={getSupportedFormats}>
                <span>🔄</span>
                刷新支持格式
              </button>
              <button className="action-btn" onClick={() => setTranscription('')}>
                <span>🗑️</span>
                清除状态
              </button>
              <button className="action-btn">
                <span>⚙️</span>
                转录设置
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
              <button className="action-btn" onClick={() => setShowAppSelector?.(true)}>选择</button>
              <button className="action-btn" onClick={() => setShowHistorySettings?.(true)}>设置</button>
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
                    <div className="history-icon">
                      {entry.audio_file_path ? '📁' : '🎤'}
                    </div>
                    <div className="history-content">
                      <div className="history-text">
                        {entry.text}
                      </div>
                      <div className="history-meta">
                        <span className="history-type">
                          {entry.audio_file_path ? '文件转录' : '实时听写'}
                        </span>
                        <span className="history-time">{timeLabel}</span>
                        <span className="history-duration">{entry.duration} seconds</span>
                        <span className="history-model">{entry.model}</span>
                        <span className="history-confidence">{Math.round(entry.confidence * 100)}%</span>
                      </div>
                    </div>
                    <div className="history-actions">
                      <button 
                        className="action-btn small"
                        onClick={() => handleExportEntry(entry.id, 'txt')}
                        title="导出为TXT"
                      >
                        📄
                      </button>
                      <button 
                        className="action-btn small"
                        onClick={() => handleExportEntry(entry.id, 'json')}
                        title="导出为JSON"
                      >
                        📋
                      </button>
                      <button 
                        className="action-btn small danger"
                        onClick={() => handleDeleteEntry(entry.id)}
                        title="删除记录"
                      >
                        🗑️
                      </button>
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
            <button className="add-shortcut" onClick={() => setShowShortcutEditor?.(true)}>+</button>
            
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
        <AIPrompts 
          onEnhancedTextReady={onEnhancedTextReady}
          transcriptionText={transcriptionText}
          isRecording={isRecording}
        />
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
    currentPage,
    isRecording,
    transcriptionText,
    showFloatingDialog,
    audioDevices,
    setDevices,
    setCurrentPage,
    setTranscriptionHistory,
    setTranscription,
    addTranscriptionEntry,
    setRecording,
    setShowFloatingDialog,
  } = useStore();

  // 新增的状态管理
  const [showAppSelector, setShowAppSelector] = useState(false);
  const [showShortcutEditor, setShowShortcutEditor] = useState(false);
  const [showHistorySettings, setShowHistorySettings] = useState(false);
  const [showTestPanel, setShowTestPanel] = useState(false);
  const [showPermissionSettings, setShowPermissionSettings] = useState(false);
  const [showFirstLaunchWizard, setShowFirstLaunchWizard] = useState(false);
  const [showSubscriptionManager, setShowSubscriptionManager] = useState(false);
  const [trialInfo, setTrialInfo] = useState<any>(null);
  // const [aiPromptsRef, setAiPromptsRef] = useState<any>(null);
  const [shortcuts, setShortcuts] = useState<any[]>([
    {
      id: '1',
      name: '快捷键',
      key: 'Fn',
      modifiers: [],
      mode: 'toggle',
      assigned: true
    }
  ]);
  const [historySettings, setHistorySettings] = useState({
    autoDelete: false,
    deleteAfterDays: 30,
    maxStorageSize: 1000,
    groupByDate: true,
    showSummaries: true,
    exportFormat: 'txt' as const
  });

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

        // 获取支持的文件格式
        const formats = await invoke<string[]>('get_supported_formats');
        console.log('✅ 应用初始化完成');
        console.log('支持的文件格式:', formats);

        // 初始化快捷键管理器
        await initializeShortcuts();
        
        // 检查权限
        await checkPermissions();
        
        // 检查是否首次启动
        checkFirstLaunch();
        
        // 检查 TTS 服务试用状态
        checkTTSTrialStatus();
      } catch (error) {
        console.error('初始化失败:', error);
      }
    };

    // 检查 TTS 试用状态
    const checkTTSTrialStatus = () => {
      const info = ttsService.getTrialInfo();
      setTrialInfo(info);
      
      // 如果试用期即将结束，显示提醒
      if (!info.isPro && info.daysLeft <= 1) {
        setTimeout(() => {
          setShowSubscriptionManager(true);
        }, 3000);
      }
    };

    // 监听转录结果
    const setupListeners = async () => {
      try {
        const unlisten1 = await listen<TranscriptionEntry>('transcription_result', (event) => {
          const entry = event.payload;
          setTranscription(entry.text);
          addTranscriptionEntry(entry);
          
          // 如果AI处理处于激活状态且在AI提示页面，处理语音转录
          // if (currentPage === 'ai-prompts' && aiPromptsRef?.processWithAgents) {
          //   setAiProcessingActive(true);
          //   aiPromptsRef.processWithAgents(entry.text);
          // }
        });

        const unlisten2 = await listen<TranscriptionEntry>('file_transcription_result', (event) => {
          const entry = event.payload;
          console.log('收到文件转录结果:', entry);
          setTranscription(`文件转录完成: ${entry.text}`);
          addTranscriptionEntry(entry);
        });

        const unlisten3 = await listen<string>('file_transcription_error', (event) => {
          const error = event.payload;
          console.error('文件转录错误:', error);
          setTranscription(`文件转录失败: ${error}`);
        });

        // 监听全局快捷键事件
        const unlisten4 = await listen('global_shortcut_triggered', (event: any) => {
          console.log('全局快捷键触发:', event);
          // 打开AI助手对话框
          setShowFloatingDialog(true);
        });

        // 监听 Fn 键或其他特殊快捷键
        const unlisten5 = await listen('shortcut_pressed', (event: any) => {
          console.log('快捷键按下:', event.payload);
          const { shortcut } = event.payload || {};
          
          if (shortcut === 'Fn' || shortcut === 'CommandOrControl+Shift+Space') {
            // 切换录音状态
            handleFloatingDialogToggleRecording();
          }
        });

        // 监听系统托盘事件
        const unlisten6 = await listen('tray_toggle_recording', () => {
          console.log('托盘录音切换');
          handleFloatingDialogToggleRecording();
        });

        const unlisten7 = await listen<string>('tray_navigate_to', (event) => {
          console.log('托盘导航到:', event.payload);
          setCurrentPage(event.payload);
        });

        const unlisten8 = await listen('tray_show_permissions', () => {
          console.log('托盘权限设置');
          setShowPermissionSettings(true);
        });

        return () => {
          unlisten1();
          unlisten2();
          unlisten3();
          unlisten4();
          unlisten5();
          unlisten6();
          unlisten7();
          unlisten8();
          unregisterAll();
        };
      } catch (error) {
        console.error('设置监听器失败:', error);
      }
    };

    initializeApp();
    setupListeners();

    // 清理函数
    return () => {
      shortcutManager.unregisterAllShortcuts();
    };
  }, [setDevices, setTranscriptionHistory, setTranscription, addTranscriptionEntry]);

  // 处理悬浮对话框的录音切换
  // 检查首次启动
  const checkFirstLaunch = () => {
    const hasCompletedSetup = localStorage.getItem('spokenly_setup_completed');
    const hasSeenWizard = localStorage.getItem('spokenly_wizard_seen');
    const hasSeenSubscription = localStorage.getItem('spokenly_subscription_seen');
    
    console.log('检查首次启动:', {
      hasCompletedSetup,
      hasSeenWizard,
      hasSeenSubscription
    });
    
    // 开发模式下的快捷重置功能 (Shift+Cmd+R+E+S+E+T)
    const setupDevKeyListener = () => {
      let keySequence = '';
      const targetSequence = 'RESET';
      
      const handleKeyPress = (e: KeyboardEvent) => {
        if (e.shiftKey && e.metaKey) {
          keySequence += e.key.toUpperCase();
          if (keySequence.includes(targetSequence)) {
            console.log('🔄 开发者重置：清除首次启动状态');
            localStorage.removeItem('spokenly_setup_completed');
            localStorage.removeItem('spokenly_wizard_seen');
            localStorage.removeItem('spokenly_subscription_seen');
            localStorage.removeItem('spokenly_preferred_shortcut');
            setTimeout(() => {
              window.location.reload();
            }, 100);
          }
          // 重置序列如果不匹配
          setTimeout(() => { keySequence = ''; }, 2000);
        }
      };
      
      document.addEventListener('keydown', handleKeyPress);
      return () => document.removeEventListener('keydown', handleKeyPress);
    };
    
    // 仅在开发环境启用
    try {
      setupDevKeyListener();
    } catch (error) {
      console.log('开发者重置功能初始化失败:', error);
    }
    
    // 如果从未完成设置向导，显示首次启动向导
    if (!hasCompletedSetup && !hasSeenWizard) {
      console.log('🎯 首次启动，显示向导');
      localStorage.setItem('spokenly_wizard_seen', 'true');
      setTimeout(() => {
        setShowFirstLaunchWizard(true);
      }, 1500);
    } else if (hasCompletedSetup && !hasSeenSubscription) {
      // 如果已完成向导但还没看到订阅选项，显示订阅管理器
      console.log('💎 显示订阅选项');
      localStorage.setItem('spokenly_subscription_seen', 'true');
      setTimeout(() => {
        setShowSubscriptionManager(true);
      }, 2000);
    }
  };
  
  // 检查权限
  const checkPermissions = async () => {
    const missingPermissions = await permissionManager.getMissingRequiredPermissions();
    if (missingPermissions.length > 0) {
      console.log('⚠️ 发现缺失的必需权限:', missingPermissions.map(p => p.name).join(', '));
      
      // 如果不是首次启动且缺少关键权限，显示权限设置
      const hasCompletedSetup = localStorage.getItem('spokenly_setup_completed');
      if (hasCompletedSetup && missingPermissions.some(p => p.required)) {
        setTimeout(() => {
          setShowPermissionSettings(true);
        }, 2000);
      }
    }
  };

  // 初始化快捷键
  const initializeShortcuts = async () => {
    // 注册快捷键事件监听器
    shortcutManager.on('toggle-recording', async () => {
      await handleFloatingDialogToggleRecording();
    });

    shortcutManager.on('quick-transcribe', async () => {
      if (!isRecording) {
        await invoke('start_recording');
        setRecording(true);
        // 3秒后自动停止
        setTimeout(async () => {
          await invoke('stop_recording');
          setRecording(false);
        }, 3000);
      }
    });

    shortcutManager.on('open-ai-assistant', () => {
      setShowFloatingDialog(true);
    });

    shortcutManager.on('switch-to-history', () => {
      setCurrentPage('history');
    });

    shortcutManager.on('switch-to-models', () => {
      setCurrentPage('transcription');
    });

    shortcutManager.on('switch-to-settings', () => {
      setCurrentPage('general');
    });

    // 监听首次启动向导事件
    shortcutManager.on('show-first-launch-wizard', () => {
      setShowFirstLaunchWizard(true);
    });

    shortcutManager.on('suggest-permission-check', () => {
      // 温和的权限检查提醒
      console.log('💡 建议用户检查权限设置');
    });

    shortcutManager.on('copy-transcription', async () => {
      if (transcriptionText) {
        await navigator.clipboard.writeText(transcriptionText);
        console.log('✅ 已复制转录文本');
      }
    });

    shortcutManager.on('export-transcription', async () => {
      const history = useStore.getState().transcriptionHistory;
      if (history.length > 0) {
        const latest = history[0];
        await invoke('export_transcription', {
          entryId: latest.id,
          exportFormat: 'txt'
        });
      }
    });

    // 注册所有快捷键
    await shortcutManager.registerAllShortcuts();
    console.log('✅ 快捷键系统已初始化');
  };

  const handleFloatingDialogToggleRecording = async () => {
    if (isRecording) {
      try {
        await invoke('stop_recording');
        setRecording(false);
        // 更新托盘图标为非录音状态
        await invoke('set_tray_icon_recording', { isRecording: false });
      } catch (error) {
        console.error('停止录音失败:', error);
      }
    } else {
      try {
        await invoke('start_recording');
        setRecording(true);
        // 更新托盘图标为录音状态
        await invoke('set_tray_icon_recording', { isRecording: true });
      } catch (error) {
        console.error('开始录音失败:', error);
      }
    }
  };

  // 处理AI增强后的文本
  const handleEnhancedTextReady = async (enhancedText: string) => {
    try {
      // 更新转录文本为增强后的版本
      setTranscription(enhancedText);
      
      // 自动输入到目标应用（如果需要）
      // await invoke('auto_input_text', { text: enhancedText });
      
      // 重置AI处理状态
      // setAiProcessingActive(false);
      
      console.log('✅ AI增强文本已处理完成:', enhancedText);
    } catch (error) {
      console.error('处理增强文本失败:', error);
      // setAiProcessingActive(false);
    }
  };

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
              onClick={() => {
                setCurrentPage(item.id);
                // 如果点击快捷键设置，同时检查权限
                if (item.id === 'shortcuts') {
                  checkPermissions();
                }
              }}
            >
              <span className="nav-icon">{item.icon}</span>
              <span className="nav-label">{item.label}</span>
            </button>
          ))}
        </nav>

        <div className="sidebar-footer">
          <PermissionIndicator onOpenSettings={() => setShowPermissionSettings(true)} />
          <div className="upgrade-link" onClick={() => setShowSubscriptionManager(true)}>
            升级 Pro
          </div>
          <div className="version-info" onClick={() => setShowTestPanel(true)} style={{ cursor: 'pointer' }}>v2.12.10</div>
        </div>
      </div>

      {/* 主内容区域 */}
      <div className="main-content">
        <PageContent 
          page={currentPage} 
          setShowShortcutEditor={setShowShortcutEditor}
          setShowAppSelector={setShowAppSelector}
          setShowHistorySettings={setShowHistorySettings}
          audioDevices={audioDevices}
          trialInfo={trialInfo}
          setShowSubscriptionManager={setShowSubscriptionManager}
          onEnhancedTextReady={handleEnhancedTextReady}
          isRecording={isRecording}
        />
      </div>

      {/* AI助手悬浮对话框 */}
      <FloatingDialog
        isVisible={showFloatingDialog}
        isRecording={isRecording}
        transcriptionText={transcriptionText}
        onClose={() => setShowFloatingDialog(false)}
        onToggleRecording={handleFloatingDialogToggleRecording}
        onSubmitPrompt={(prompt) => {
          console.log('AI助手提示:', prompt);
          setTranscription(`AI助手处理: ${prompt}`);
          setTimeout(() => {
            setTranscription(`AI助手回复: 已收到您的指令"${prompt}"，正在处理...`);
          }, 1000);
          setShowFloatingDialog(false);
        }}
      />

      {/* 应用选择器对话框 */}
      <AppSelector
        isVisible={showAppSelector}
        onClose={() => setShowAppSelector(false)}
        onSelectApp={(app) => {
          console.log('选择的应用:', app);
        }}
      />

      {/* 快捷键编辑器对话框 */}
      <ShortcutEditor
        isVisible={showShortcutEditor}
        onClose={() => setShowShortcutEditor(false)}
        shortcuts={shortcuts}
        onUpdateShortcut={(shortcut) => {
          setShortcuts(shortcuts.map(s => s.id === shortcut.id ? shortcut : s));
        }}
        onAddShortcut={() => {
          const newShortcut = {
            id: `${shortcuts.length + 1}`,
            name: '新快捷键',
            key: '未指定',
            modifiers: [],
            mode: 'toggle' as const,
            assigned: false
          };
          setShortcuts([...shortcuts, newShortcut]);
        }}
      />

      {/* 历史记录设置对话框 */}
      <HistorySettings
        isVisible={showHistorySettings}
        onClose={() => setShowHistorySettings(false)}
        settings={historySettings}
        onUpdateSettings={(settings) => {
          setHistorySettings(settings);
          console.log('更新历史记录设置:', settings);
        }}
      />

      {/* 功能测试面板 */}
      <FeatureTestPanel
        isVisible={showTestPanel}
        onClose={() => setShowTestPanel(false)}
      />

      {/* 权限设置对话框 */}
      <PermissionSettings
        isVisible={showPermissionSettings}
        onClose={() => setShowPermissionSettings(false)}
        onPermissionsConfigured={() => {
          console.log('✅ 权限已配置');
          // 重新注册快捷键
          shortcutManager.registerAllShortcuts();
        }}
      />

      {/* 首次启动向导 */}
      <FirstLaunchWizard
        isVisible={showFirstLaunchWizard}
        onComplete={() => {
          setShowFirstLaunchWizard(false);
          console.log('✅ 首次设置完成');
          // 重新注册快捷键
          shortcutManager.registerAllShortcuts();
        }}
      />

      {/* 订阅管理 */}
      <SubscriptionManager
        isVisible={showSubscriptionManager}
        onClose={() => setShowSubscriptionManager(false)}
        isFirstLaunch={!localStorage.getItem('spokenly_subscription_seen')}
        onUpgradeSuccess={() => {
          // 刷新试用状态
          const info = ttsService.getTrialInfo();
          setTrialInfo(info);
        }}
      />

      {/* 试用状态提示 - 已移除以避免过度商业化 */}
    </div>
  );
}

export default App;