import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { unregisterAll } from '@tauri-apps/api/globalShortcut';
import { open } from '@tauri-apps/api/dialog';
import './App.css';
import './styles/micro-interactions.css';
import { transcriptionModels } from './data/models';
import logger from './utils/logger';

// 扩展 Window 接口以包含全局录音函数
declare global {
  interface Window {
    appToggleRecording?: () => Promise<void>;
  }
}

// 获取模型信息的帮助函数
const getModelInfo = (modelId: string) => {
  logger.debug('查找模型ID', modelId);
  const model = transcriptionModels.find(m => m.id === modelId);
  logger.debug('找到的模型', model ? `${model.name} (type: ${model.type})` : 'null');
  const result = {
    model: modelId,
    modelType: model?.type || 'online'
  };
  logger.debug('返回结果', result);
  return result;
};

// Components
import FloatingDialog from './components/FloatingDialog';
import AppSelector from './components/AppSelector';
import ShortcutEditor from './components/ShortcutEditor';
import ShortcutPage from './components/ShortcutPage';
import AdvancedShortcutEditor from './components/AdvancedShortcutEditor';
import HistorySettings from './components/HistorySettings';
import TranscriptionModelsPage from './components/TranscriptionModelsPage';
import FeatureTestPanel from './components/FeatureTestPanel';
import AudioInputTest from './components/AudioInputTest';
import DiagnosticButton from './components/DiagnosticButton';
import PermissionSettings from './components/PermissionSettings';
import PermissionIndicator from './components/PermissionIndicator';
import FirstLaunchWizard from './components/FirstLaunchWizard';
import SubscriptionManager from './components/SubscriptionManager';
import AIPrompts from './components/AIPrompts';
import AIPromptsEnhanced from './components/AIPromptsEnhanced';
import TranscriptionDetailView from './components/TranscriptionDetailView';
import EnhancedHistoryPage from './components/EnhancedHistoryPage';
import TextInjectionSettings from './components/TextInjectionSettings';
import RecordingStatusIndicator from './components/RecordingStatusIndicator';
import EnhancedShortcutManager from './components/EnhancedShortcutManager';
import { shortcutManager } from './utils/shortcutManager';
import { permissionManager } from './utils/permissionManager';
// import SystemChecker from './utils/systemCheck';
import { ttsService } from './services/ttsService';

// Types and Stores
// import { ApiConfig } from './types/models';
import { useModelsStore } from './stores/modelsStore';
import { enhancedShortcutManager } from './utils/enhancedShortcutManager';
import { recordingTimer } from './utils/recordingTimer';

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
  useEnhancedAIPrompts: boolean;
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
  setUseEnhancedAIPrompts: (use: boolean) => void;
}

export const useStore = create<AppStore>((set) => ({
  isRecording: false,
  transcriptionText: '',
  audioDevices: [],
  selectedDevice: null,
  language: 'en',
  hotkey: 'CommandOrControl+Shift+Space',
  currentPage: 'general',
  selectedModel: 'luyingwang-online', // 默认使用鲁音网在线模型
  transcriptionHistory: [],
  mcpConfig: {
    enabled: true,
    server_url: import.meta.env.VITE_TTS_SERVER_URL || 'https://api.openai.com/v1',
    api_key: import.meta.env.VITE_TTS_API_KEY || '',
    model: 'whisper-1',
  },
  showFloatingDialog: false,
  aiProcessingActive: false,
  useEnhancedAIPrompts: false, // 默认使用原版
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
  setUseEnhancedAIPrompts: (use) => set({ useEnhancedAIPrompts: use }),
}));

// 导航菜单项
const navigationItems = [
  { id: 'general', label: '常规设置', icon: '•' },
  { id: 'transcription', label: '听写模型', icon: '•' },
  { id: 'files', label: '转录文件', icon: '•' },
  { id: 'history', label: '历史记录', icon: '•' },
  { id: 'shortcuts', label: '快捷键', icon: '•' },
  { id: 'ai-prompts', label: 'AI提示', icon: '•' },
  { id: 'contact', label: '联系我们', icon: '•' },
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
  selectedModel?: string;
  setShowShortcutEditor?: (show: boolean) => void;
  setShowAppSelector?: (show: boolean) => void;
  setShowHistorySettings?: (show: boolean) => void;
  setShowEnhancedHistory?: (show: boolean) => void;
  setShowTextInjectionSettings?: (show: boolean) => void;
  setShowEnhancedShortcutManager?: (show: boolean) => void;
  audioDevices?: AudioDevice[];
  trialInfo?: any;
  setShowSubscriptionManager?: (show: boolean) => void;
  onEnhancedTextReady?: (text: string) => void;
  isRecording?: boolean;
  useAdvancedShortcuts?: boolean;
  setUseAdvancedShortcuts?: (value: boolean) => void;
  useEnhancedAIPrompts?: boolean;
  setUseEnhancedAIPrompts?: (value: boolean) => void;
  selectedEntry?: TranscriptionEntry | null;
  setSelectedEntry?: (entry: TranscriptionEntry | null) => void;
  handleFloatingDialogToggleRecording?: () => Promise<void>;
  isTranscribing?: boolean;
}> = ({ page, selectedModel: propSelectedModel, setShowShortcutEditor, setShowAppSelector, setShowHistorySettings, setShowEnhancedHistory, setShowTextInjectionSettings, setShowEnhancedShortcutManager, audioDevices = [], onEnhancedTextReady, isRecording: propIsRecording, useAdvancedShortcuts, setUseAdvancedShortcuts, useEnhancedAIPrompts, setUseEnhancedAIPrompts, setSelectedEntry, handleFloatingDialogToggleRecording, isTranscribing }) => {
  const {
    transcriptionText,
    transcriptionHistory,
    selectedModel,
    setTranscription,
    setTranscriptionHistory,
    setRecording,
    isRecording,
  } = useStore();

  const [loginOnStartup, setLoginOnStartup] = useState(false);
  const [showInDock, setShowInDock] = useState(false);
  const [showInStatusBar, setShowInStatusBar] = useState(true);
  const [playbackEffects, setPlaybackEffects] = useState(true);
  const [recordingMute, setRecordingMute] = useState(true);
  const [touchBarFeedback, setTouchBarFeedback] = useState(true);
  const [isUploading, setIsUploading] = useState(false);
  const [supportedFormats, setSupportedFormats] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedFilter, setSelectedFilter] = useState<'all' | 'listening' | 'file' | 'journal'>('all');
  const [sortBy, setSortBy] = useState<'newest' | 'oldest' | 'name'>('newest');


  // 获取支持的文件格式
  const getSupportedFormats = async () => {
    try {
      // 暂时使用默认格式，命令不存在
      const formats = ['mp3', 'wav', 'm4a', 'flac', 'mp4', 'mov', 'm4v'];
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
        logger.info('选择的文件', selected);
        
        const result = await invoke<string>('upload_file', { 
          filePath: selected 
        });
        
        logger.info('上传结果', result);
        
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
      logger.info('导出成功', exportPath);
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

  // 搜索和过滤逻辑
  const filteredAndSortedHistory = React.useMemo(() => {
    let filtered = transcriptionHistory;

    // 按类型过滤
    if (selectedFilter !== 'all') {
      filtered = filtered.filter(entry => {
        switch (selectedFilter) {
          case 'listening':
            return !entry.audio_file_path; // 实时听写
          case 'file':
            return !!entry.audio_file_path; // 文件转录
          case 'journal':
            // 这里可以根据特定标记或长度判断是否为日记
            return entry.text.length > 100; // 假设超过100字符的为日记
          default:
            return true;
        }
      });
    }

    // 搜索过滤
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(entry => 
        entry.text.toLowerCase().includes(query) ||
        entry.model.toLowerCase().includes(query) ||
        (entry.audio_file_path && entry.audio_file_path.toLowerCase().includes(query))
      );
    }

    // 排序
    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'newest':
          return b.timestamp - a.timestamp;
        case 'oldest':
          return a.timestamp - b.timestamp;
        case 'name':
          return a.text.localeCompare(b.text);
        default:
          return b.timestamp - a.timestamp;
      }
    });

    return filtered;
  }, [transcriptionHistory, selectedFilter, searchQuery, sortBy]);

  switch (page) {
    case 'general':
      return (
        <div className="page-content">
          <div className="page-header">
            <h1>常规首选项</h1>
            <p>根据您的工作流程和偏好配置 Recording King。</p>
          </div>

          <div className="section">
            <h2>录音测试</h2>
            <div className="recording-test-container">
              
              {/* 当前模型信息 */}
              <div className="current-model-info">
                <div className="model-display">
                  <span className="model-label">当前模型:</span>
                  <span className="model-name">{selectedModel}</span>
                  <span className={`model-type ${getModelInfo(selectedModel).modelType}`}>
                    {getModelInfo(selectedModel).modelType === 'local' ? '本地' : '在线'}
                  </span>
                </div>
              </div>

              {/* 录音控制区 */}
              <div className="recording-controls">
                <p className="recording-description">点击按钮测试麦克风录音和转录功能：</p>
                
                {/* 音频诊断工具按钮 */}
                <div className="audio-test-actions">
                  <DiagnosticButton 
                    category="audio" 
                    size="medium"
                    autoStart={true}
                  />
                </div>
                
                <button 
                  className={`recording-button ${isRecording ? 'recording' : 'idle'}`}
                  onClick={() => {
                    console.log('🔥 REC 按钮被点击!');
                    console.log('handleFloatingDialogToggleRecording 是否存在:', !!handleFloatingDialogToggleRecording);
                    if (handleFloatingDialogToggleRecording) {
                      handleFloatingDialogToggleRecording();
                    } else {
                      console.error('❌ handleFloatingDialogToggleRecording 函数不存在!');
                      alert('录音函数未找到，请检查控制台');
                    }
                  }}
                >
                  <span className="button-icon">
                    {isRecording ? 'STOP' : 'REC'}
                  </span>
                  <span className="button-text">
                    {isRecording ? '停止录音' : '开始录音'}
                  </span>
                </button>

                <div className="recording-status">
                  <div className={`status-indicator ${isRecording ? 'active' : isTranscribing ? 'processing' : 'inactive'}`}></div>
                  <span className="status-text">
                    {isRecording ? '正在录音...' : isTranscribing ? '正在转录...' : '未录音'}
                  </span>
                </div>
              </div>

              {/* 转录结果显示区 */}
              {transcriptionText && (
                <div className="transcription-result">
                  <h3>转录结果</h3>
                  <div className="result-content">
                    <p>{transcriptionText}</p>
                  </div>
                </div>
              )}
              
            </div>
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
              <select defaultValue="zh" className="select-field" onChange={(e) => logger.debug('语言切换', e.target.value)}>
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
                    <div className="device-icon">MIC</div>
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
            <p>将音频或视频文件转换为文本。Recording King 将为您进行转录。</p>
          </div>

          <div className="file-upload-area">
            <div 
              className={`upload-zone ${isUploading ? 'uploading' : ''}`}
              onClick={handleFileUpload}
              style={{ cursor: isUploading ? 'not-allowed' : 'pointer' }}
            >
              <div className="upload-icon">
                {isUploading ? 'UPLOADING' : 'SELECT'}
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
              <button onClick={() => {
                logger.debug('当前 selectedModel', selectedModel);
                logger.debug('所有可用模型', transcriptionModels.map(m => `${m.id} (${m.type})`));
                const { model, modelType } = getModelInfo(selectedModel);
                logger.debug('当前模型信息', { model, modelType });
              }}>
                调试模型状态
              </button>
            </div>

            <div className="file-actions">
              <button 
                className="action-btn"
                onClick={handleFileUpload}
                disabled={isUploading}
              >
                <span>BROWSE</span>
                {isUploading ? '上传中...' : '选择文件'}
              </button>
              <button className="action-btn" onClick={getSupportedFormats}>
                <span>REFRESH</span>
                刷新支持格式
              </button>
              <button className="action-btn" onClick={() => setTranscription('')}>
                <span>CLEAR</span>
                清除状态
              </button>
              <button className="action-btn">
                <span>CONFIG</span>
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
            <div className="header-content">
              <div>
                <h1>历史记录</h1>
                <p>查看存储在您电脑上的转录历史记录</p>
              </div>
              <div className="header-actions">
                <DiagnosticButton 
                  category="storage" 
                  size="small"
                  style="button"
                />
              </div>
            </div>
          </div>

          <div className="history-controls">
            <div className="filter-tabs">
              <button 
                className={`filter-tab ${selectedFilter === 'all' ? 'active' : ''}`}
                onClick={() => setSelectedFilter('all')}
              >
                全部
              </button>
              <button 
                className={`filter-tab ${selectedFilter === 'listening' ? 'active' : ''}`}
                onClick={() => setSelectedFilter('listening')}
              >
                听写
              </button>
              <button 
                className={`filter-tab ${selectedFilter === 'file' ? 'active' : ''}`}
                onClick={() => setSelectedFilter('file')}
              >
                文件
              </button>
              <button 
                className={`filter-tab ${selectedFilter === 'journal' ? 'active' : ''}`}
                onClick={() => setSelectedFilter('journal')}
              >
                日记
              </button>
            </div>
            <div className="history-actions">
              <button className="action-btn enhanced-history-btn" onClick={() => setShowEnhancedHistory(true)}>
                <span>🚀</span>
                增强搜索
              </button>
              <button className="action-btn text-injection-btn" onClick={() => setShowTextInjectionSettings(true)}>
                <span>🎯</span>
                文本注入
              </button>
              <button className="action-btn shortcut-manager-btn" onClick={() => setShowEnhancedShortcutManager(true)}>
                <span>⌨️</span>
                快捷键
              </button>
              <button className="action-btn" onClick={() => setShowAppSelector?.(true)}>选择</button>
              <button className="action-btn" onClick={() => setShowHistorySettings?.(true)}>设置</button>
            </div>
          </div>

          <div className="search-bar">
            <div className="search-input-container">
              <input 
                type="text" 
                placeholder="搜索转录内容、模型名称或文件名..." 
                className="search-input"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
              {searchQuery && (
                <button 
                  className="clear-search-btn"
                  onClick={() => setSearchQuery('')}
                  title="清除搜索"
                >
                  ✕
                </button>
              )}
            </div>
            <select 
              className="sort-select"
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as 'newest' | 'oldest' | 'name')}
            >
              <option value="newest">最新的在前</option>
              <option value="oldest">最旧的在前</option>
              <option value="name">按内容排序</option>
            </select>
          </div>

          {/* 搜索结果统计 */}
          {(searchQuery || selectedFilter !== 'all') && (
            <div className="search-results-info">
              <span>
                找到 {filteredAndSortedHistory.length} 条记录
                {searchQuery && ` (搜索: "${searchQuery}")`}
                {selectedFilter !== 'all' && ` (筛选: ${selectedFilter})`}
              </span>
              {(searchQuery || selectedFilter !== 'all') && (
                <button 
                  className="clear-filters-btn"
                  onClick={() => {
                    setSearchQuery('');
                    setSelectedFilter('all');
                  }}
                >
                  清除筛选
                </button>
              )}
            </div>
          )}

          <div className="history-list">
            {filteredAndSortedHistory.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">EMPTY</div>
                <h3>
                  {transcriptionHistory.length === 0 
                    ? '暂无转录记录' 
                    : '未找到匹配的记录'
                  }
                </h3>
                <p>
                  {transcriptionHistory.length === 0 
                    ? '开始录音后，转录记录将显示在这里' 
                    : '尝试调整搜索关键词或筛选条件'
                  }
                </p>
              </div>
            ) : (
              filteredAndSortedHistory.map((entry) => {
                const timeAgo = Math.floor((Date.now() - entry.timestamp * 1000) / 1000);
                const timeLabel = timeAgo < 60 ? `${timeAgo}s ago` : 
                                 timeAgo < 3600 ? `${Math.floor(timeAgo / 60)}m ago` : 
                                 `${Math.floor(timeAgo / 3600)}h ago`;
                
                return (
                  <div key={entry.id} className="history-item">
                    <div className="history-icon">
                      {entry.audio_file_path ? 'FILE' : 'LIVE'}
                    </div>
                    <div 
                      className="history-content"
                      onClick={() => setSelectedEntry?.(entry)}
                      style={{ cursor: 'pointer' }}
                      title="点击查看详情"
                    >
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
                        onClick={(e) => {
                          e.stopPropagation();
                          setSelectedEntry?.(entry);
                        }}
                        title="查看详情"
                      >
                        VIEW
                      </button>
                      <button 
                        className="action-btn small"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleExportEntry(entry.id, 'txt');
                        }}
                        title="导出为TXT"
                      >
                        COPY
                      </button>
                      <button 
                        className="action-btn small"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleExportEntry(entry.id, 'json');
                        }}
                        title="导出为JSON"
                      >
                        COPY
                      </button>
                      <button 
                        className="action-btn small danger"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteEntry(entry.id);
                        }}
                        title="删除记录"
                      >
                        DEL
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
      return <ShortcutPage />;

    case 'ai-prompts':
      return (
        <div className="page-content">
          <div className="page-header">
            <div className="header-content">
              <div>
                <h1>AI 提示管理</h1>
                <p>选择和配置AI提示处理模式</p>
              </div>
              <div className="header-actions">
                <DiagnosticButton 
                  category="api" 
                  size="small"
                  style="button"
                />
                <DiagnosticButton 
                  category="network" 
                  size="small"
                  style="button"
                />
              </div>
            </div>
          </div>

          <div className="section">
            <h2>模式选择</h2>
            <div className="mode-selector">
              <div className="mode-toggle">
                <label className="toggle-option">
                  <input
                    type="radio"
                    name="aiPromptsMode"
                    checked={!useEnhancedAIPrompts}
                    onChange={() => setUseEnhancedAIPrompts?.(false)}
                  />
                  <span className="toggle-label">
                    <span className="toggle-icon">BASIC</span>
                    <div className="toggle-info">
                      <span className="toggle-name">基础模式</span>
                      <span className="toggle-desc">简单易用的Agent链配置</span>
                    </div>
                  </span>
                </label>
                
                <label className="toggle-option">
                  <input
                    type="radio"
                    name="aiPromptsMode"
                    checked={useEnhancedAIPrompts}
                    onChange={() => setUseEnhancedAIPrompts?.(true)}
                  />
                  <span className="toggle-label">
                    <span className="toggle-icon">PRO</span>
                    <div className="toggle-info">
                      <span className="toggle-name">增强模式</span>
                      <span className="toggle-desc">支持多种LLM模型和快捷键</span>
                    </div>
                  </span>
                </label>
              </div>
            </div>
          </div>

          <div className="ai-prompts-wrapper">
            {useEnhancedAIPrompts ? (
              <AIPromptsEnhanced
                onEnhancedTextReady={onEnhancedTextReady}
                transcriptionText={transcriptionText}
                isRecording={isRecording}
              />
            ) : (
              <AIPrompts 
                onEnhancedTextReady={onEnhancedTextReady}
                transcriptionText={transcriptionText}
                isRecording={isRecording}
              />
            )}
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
              <div className="contact-icon">EMAIL</div>
              <div className="contact-details">
                <h3>技术支持</h3>
                <p>support@spokenly.com</p>
              </div>
            </div>
            
            <div className="contact-item">
              <div className="contact-icon">WEB</div>
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
    selectedModel,
    showFloatingDialog,
    audioDevices,
    useEnhancedAIPrompts,
    setDevices,
    setCurrentPage,
    setTranscriptionHistory,
    setTranscription,
    addTranscriptionEntry,
    setRecording,
    setShowFloatingDialog,
    setUseEnhancedAIPrompts,
  } = useStore();
  
  // Models Store
  const { saveModelConfig } = useModelsStore();

  // 新增的状态管理
  const [showAppSelector, setShowAppSelector] = useState(false);
  const [showShortcutEditor, setShowShortcutEditor] = useState(false);
  const [useAdvancedShortcuts, setUseAdvancedShortcuts] = useState(false); // 默认使用精简版快捷键编辑器
  const [showHistorySettings, setShowHistorySettings] = useState(false);
  const [showEnhancedHistory, setShowEnhancedHistory] = useState(false);
  const [showTextInjectionSettings, setShowTextInjectionSettings] = useState(false);
  const [recordingDuration, setRecordingDuration] = useState(0);
  const [audioLevel, setAudioLevel] = useState(0);
  const [showFloatingIndicator, setShowFloatingIndicator] = useState(false);
  const [showEnhancedShortcutManager, setShowEnhancedShortcutManager] = useState(false);
  const [showTestPanel, setShowTestPanel] = useState(false);
  const [showAudioInputTest, setShowAudioInputTest] = useState(false);
  const [showPermissionSettings, setShowPermissionSettings] = useState(false);
  const [showFirstLaunchWizard, setShowFirstLaunchWizard] = useState(false);
  const [showSubscriptionManager, setShowSubscriptionManager] = useState(false);
  const [trialInfo, setTrialInfo] = useState<any>(null);
  const [selectedEntry, setSelectedEntry] = useState<TranscriptionEntry | null>(null);
  const [isTranscribing, setIsTranscribing] = useState(false);
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

  // 状态同步函数
  const syncRecordingState = async () => {
    try {
      console.log('🔄 同步录音状态...');
      const backendState = await invoke('get_recording_state') as boolean;
      console.log('📊 后端录音状态:', backendState, '前端录音状态:', isRecording);
      
      if (backendState !== isRecording) {
        console.log('⚠️ 检测到前后端状态不一致，正在同步...');
        setRecording(backendState);
        
        if (backendState) {
          // 如果后端在录音但前端不知道，启动前端计时器
          const sessionId = recordingTimer.startRecording(selectedModel, 'sync');
          console.log(`🔄 同步录音会话: ${sessionId}`);
        } else {
          // 如果后端没在录音，停止前端计时器
          recordingTimer.stopRecording();
        }
      }
      console.log('✅ 录音状态同步完成');
    } catch (error) {
      console.error('❌ 同步录音状态失败:', error);
    }
  };

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
        logger.info('应用初始化完成');
        logger.info('支持的文件格式', formats);

        // 初始化快捷键管理器
        await initializeShortcuts();
        
        // 检查权限
        await checkPermissions();
        
        // 检查是否首次启动
        checkFirstLaunch();
        
        // 检查 TTS 服务试用状态
        checkTTSTrialStatus();
        
        // 初始化 LuYinWang 模型配置
        initializeLuYinWangConfig();
        
        // 同步录音状态
        setTimeout(syncRecordingState, 1000);
      } catch (error) {
        console.error('初始化失败:', error);
      }
    };

    // 初始化 LuYinWang 模型配置
    const initializeLuYinWangConfig = () => {
      try {
        const luyinwangConfig = {
          modelId: 'luyingwang-online',
          bearer_token: 'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJodHRwczovL3JlY29yZC10by10ZXh0LmNvbS9hcGkvdjEvbG9nb3V0IiwiaWF0IjoxNzUzODU4NzIxLCJleHAiOjE3NjI0OTg3MjEsIm5iZiI6MTc1Mzg1ODcyMSwianRpIjoiNTlZQjBUMExqWGV4NGZqdiIsInN1YiI6IjEiLCJwcnYiOiIyM2JkNWM4OTQ5ZjYwMGFkYjM5ZTcwMWM0MDA4NzJkYjdhNTk3NmY3IiwiZGV2aWNlX2lkIjoiYmYyZTdkODU4NWU0YmM3YTFjY2VmNWE0YzI2OTkxZDQiLCJpc19sb2dpbiI6MH0.NxgG2hysvK7we4QuyNwpNoX5etfvHTW4ZqL8s1T-5oc'
        };
        
        saveModelConfig('luyingwang-online', luyinwangConfig);
        logger.info('✅ LuYinWang 模型配置已初始化 - Bearer Token 已设置');
      } catch (error) {
        logger.error('❌ 初始化 LuYinWang 配置失败:', error);
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
        console.log('🚀 开始设置所有监听器...');
        // 监听录音转录结果（从 stop_recording 命令发出）
        const unlisten1 = await listen<TranscriptionEntry>('transcription_result', (event) => {
          const entry = event.payload;
          logger.transcription('收到录音转录结果', entry);
          setTranscription(entry.text);
          addTranscriptionEntry(entry);
          // setIsTranscribing(false); // 转录完成，清除进度状态
          
          // 如果AI处理处于激活状态且在AI提示页面，处理语音转录
          // if (currentPage === 'ai-prompts' && aiPromptsRef?.processWithAgents) {
          //   setAiProcessingActive(true);
          //   aiPromptsRef.processWithAgents(entry.text);
          // }
        });

        // 监听文件转录结果
        const unlisten2b = await listen<TranscriptionEntry>('file_transcription_result', (event) => {
          const entry = event.payload;
          logger.transcription('收到文件转录结果', entry);
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
          logger.debug('全局快捷键触发', event);
          // 打开AI助手对话框
          setShowFloatingDialog(true);
        });

        // 设置增强快捷键管理器 - 添加延迟确保后端快捷键注册完成
        console.log('🔧 设置 enhancedShortcutManager 事件订阅...');
        console.log('🔍 检查 enhancedShortcutManager 实例:', enhancedShortcutManager);
        
        // 等待一下确保后端快捷键注册完成
        await new Promise(resolve => setTimeout(resolve, 500));
        console.log('⏳ 延迟完成，开始设置事件监听器...');
        
        // 手动设置 enhancedShortcutManager 的事件监听器
        await enhancedShortcutManager.setupEventListeners();
        
        const unsubscribeRecording = enhancedShortcutManager.on('toggle_recording', () => {
          console.log('🎯 快捷键触发录音切换');
          handleFloatingDialogToggleRecording();
        });
        console.log('✅ toggle_recording 事件已订阅');
        
        // 测试快捷键监听器是否工作 
        console.log('🧪 测试快捷键监听器...');
        setTimeout(async () => {
          console.log('🧪 调用后端测试命令');
          try {
            await invoke('test_shortcut', { 
              shortcut: 'CommandOrControl+Shift+R', 
              action: 'toggle_recording' 
            });
          } catch (error) {
            console.error('❌ 测试快捷键命令失败:', error);
          }
          
          console.log('🧪 模拟快捷键触发测试');
          enhancedShortcutManager.simulateShortcut('CommandOrControl+Shift+R');
        }, 1000);

        const unsubscribeStartRecording = enhancedShortcutManager.on('start_recording', () => {
          console.log('🎙️ 快捷键触发开始录音');
          if (!isRecording) {
            handleFloatingDialogToggleRecording();
          }
        });

        const unsubscribeStopRecording = enhancedShortcutManager.on('stop_recording', () => {
          console.log('⏹️ 快捷键触发停止录音');
          if (isRecording) {
            handleFloatingDialogToggleRecording();
          }
        });

        const unsubscribeShowHistory = enhancedShortcutManager.on('show_history', () => {
          console.log('📚 快捷键触发显示历史记录');
          setCurrentPage('history');
        });

        const unsubscribeToggleVisibility = enhancedShortcutManager.on('toggle_visibility', () => {
          console.log('👁️ 快捷键触发切换窗口显示');
          // 这里可以添加窗口显示/隐藏逻辑
        });

        const unsubscribeTextInjection = enhancedShortcutManager.on('toggle_text_injection', () => {
          console.log('🎯 快捷键触发文本注入设置');
          setShowTextInjectionSettings(true);
        });

        // 设置录音计时器监听器
        const unsubscribeTimer = recordingTimer.addListener(({ duration, isActive }) => {
          setRecordingDuration(duration);
          if (!isActive) {
            // 录音结束时的处理
            console.log(`📊 录音结束，总时长: ${duration.toFixed(2)}秒`);
          }
        });

        // 监听系统托盘事件
        const unlisten6 = await listen('tray_toggle_recording', () => {
          logger.debug('托盘录音切换');
          handleFloatingDialogToggleRecording();
        });

        const unlisten7 = await listen<string>('tray_navigate_to', (event) => {
          logger.debug('托盘导航到', event.payload);
          setCurrentPage(event.payload);
        });

        const unlisten8 = await listen('tray_show_permissions', () => {
          logger.debug('托盘权限设置');
          setShowPermissionSettings(true);
        });

        return () => {
          unlisten1();
          unlisten2b();
          unlisten3();
          unlisten4();
          unlisten6();
          unlisten7();
          unlisten8();
          unregisterAll();
          
          // 清理增强快捷键管理器订阅
          unsubscribeRecording();
          unsubscribeStartRecording();
          unsubscribeStopRecording();
          unsubscribeShowHistory();
          unsubscribeToggleVisibility();
          unsubscribeTextInjection();
          unsubscribeTimer();
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
    // 暂时跳过向导，直接进入主界面
    const hasCompletedSetup = true; // localStorage.getItem('spokenly_setup_completed');
    const hasSeenWizard = true; // localStorage.getItem('spokenly_wizard_seen');
    const hasSeenSubscription = true; // localStorage.getItem('spokenly_subscription_seen');
    
    logger.info('跳过向导，直接进入主界面');
    setShowFirstLaunchWizard(false);
    setShowSubscriptionManager(false);
    return;
    
    // 开发模式下的快捷重置功能 (Shift+Cmd+R+E+S+E+T)
    const setupDevKeyListener = () => {
      let keySequence = '';
      const targetSequence = 'RESET';
      
      const handleKeyPress = (e: KeyboardEvent) => {
        if (e.shiftKey && e.metaKey) {
          keySequence += e.key.toUpperCase();
          if (keySequence.includes(targetSequence)) {
            logger.debug('开发者重置：清除首次启动状态');
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
      logger.error('开发者重置功能初始化失败', error);
    }
    
    // 如果从未完成设置向导，显示首次启动向导
    if (!hasCompletedSetup && !hasSeenWizard) {
      logger.info('首次启动，显示向导');
      localStorage.setItem('spokenly_wizard_seen', 'true');
      setTimeout(() => {
        setShowFirstLaunchWizard(true);
      }, 1500);
    } else if (hasCompletedSetup && !hasSeenSubscription) {
      // 如果已完成向导但还没看到订阅选项，显示订阅管理器
      logger.info('显示订阅选项');
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
      logger.warn('发现缺失的必需权限', missingPermissions.map(p => p.name).join(', '));
      
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
      logger.debug('快速转录快捷键触发', { isRecording });
      if (!isRecording) {
        try {
          logger.audio('开始快速转录');
          await invoke('start_recording');
          setRecording(true);
          logger.audio('录音已开始');
          
          // 3秒后自动停止
          setTimeout(async () => {
            try {
              logger.audio('自动停止录音');
              const currentModelId = selectedModel || 'gpt-4o-mini';
              const { model, modelType } = getModelInfo(currentModelId);
              await invoke('stop_recording', { 
                model: model, 
                modelType: modelType 
              });
              setRecording(false);
              logger.audio('录音已停止');
            } catch (error) {
              logger.error('停止录音失败', error);
            }
          }, 3000);
        } catch (error) {
          logger.error('开始录音失败', error);
        }
      } else {
        logger.warn('已经在录音中，忽略快捷键');
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
      logger.warn('建议用户检查权限设置');
    });

    shortcutManager.on('copy-transcription', async () => {
      if (transcriptionText) {
        await navigator.clipboard.writeText(transcriptionText);
        logger.info('已复制转录文本');
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

    // 处理快捷键冲突检测
    shortcutManager.on('shortcut-conflicts-detected', (failedShortcuts: string[]) => {
      logger.warn('检测到快捷键冲突', failedShortcuts);
      // 可以在这里显示提示或打开快捷键设置页面
      setTimeout(() => {
        setCurrentPage('shortcuts');
      }, 1000);
    });

    // 不在应用启动时自动注册全局快捷键，改为在快捷键编辑器中点击“应用更改”时注册
    logger.info('快捷键事件监听已就绪');
  };

  const handleFloatingDialogToggleRecording = async () => {
    console.log('🎯 handleFloatingDialogToggleRecording 被调用, 当前状态:', { isRecording });
    
    if (isRecording) {
      console.log('🛑 执行停止录音逻辑...');
      try {
        const { model, modelType } = getModelInfo(selectedModel || 'gpt-4o-mini');
        
        // 停止录音计时器
        const session = recordingTimer.stopRecording();
        console.log(`📊 录音会话结束:`, session);
        
        // 显示转录中状态
        setIsTranscribing(true);
        setTranscription('正在转录中，请稍候...');
        
        // 停止录音并获取转录结果
        const result = await invoke('stop_recording', { 
          model: model, 
          modelType: modelType 
        });
        
        console.log('🔄 设置 setRecording(false)...');
        setRecording(false);
        setIsTranscribing(false);
        
        // 处理转录结果
        if (result && typeof result === 'string') {
          setTranscription(result);
          logger.transcription('录音已停止，转录结果', result);
          
          // 添加到历史记录
          addTranscriptionEntry({
            id: Date.now().toString(),
            text: result,
            timestamp: Date.now(),
            model: selectedModel || 'gpt-4o-mini',
            confidence: 0.95,
            duration: session?.duration ? Math.round(session.duration / 1000) : 0
          });
        } else {
          setTranscription('转录完成，但未获取到结果');
        }
        
        // 重置音频电平
        setAudioLevel(0);
        
        // 更新托盘图标为非录音状态 (暂时跳过，命令不存在)
        // await invoke('set_tray_icon_recording', { isRecording: false });
      } catch (error) {
        console.error('停止录音失败:', error);
        setTranscription(`停止录音失败: ${error}`);
        setIsTranscribing(false);
        // 确保计时器停止
        recordingTimer.stopRecording();
        setRecording(false);
        setAudioLevel(0);
      }
    } else {
      console.log('🎙️ 执行开始录音逻辑...');
      try {
        await invoke('start_recording');
        console.log('🔄 设置 setRecording(true)...');
        setRecording(true);
        
        // 启动录音计时器
        const sessionId = recordingTimer.startRecording(selectedModel, 'default');
        console.log(`🎙️ 录音会话开始: ${sessionId}`);
        
        // 更新托盘图标为录音状态 (暂时跳过，命令不存在)
        // await invoke('set_tray_icon_recording', { isRecording: true });
        
        // 开始模拟音频电平（实际项目中应该从后端获取真实音频数据）
        const levelInterval = setInterval(() => {
          if (recordingTimer.isRecording()) {
            // 模拟音频电平变化
            const randomLevel = Math.random() * 0.8 + 0.1;
            setAudioLevel(randomLevel);
          } else {
            clearInterval(levelInterval);
            setAudioLevel(0);
          }
        }, 100);
        
      } catch (error) {
        console.error('开始录音失败:', error);
        
        // 检查是否是"已在录音中"的错误
        if (error && typeof error === 'string' && error.includes('已在录音中')) {
          console.log('🔄 检测到状态不同步，尝试重置后端状态...');
          try {
            // 重置后端录音状态
            await invoke('reset_recording_state');
            console.log('✅ 后端状态已重置，重新尝试开始录音...');
            
            // 重新尝试开始录音
            await invoke('start_recording');
            setRecording(true);
            
            // 启动录音计时器
            const sessionId = recordingTimer.startRecording(selectedModel, 'default');
            console.log(`🎙️ 录音会话开始 (重试后): ${sessionId}`);
            
            // 开始模拟音频电平
            const levelInterval = setInterval(() => {
              if (recordingTimer.isRecording()) {
                const randomLevel = Math.random() * 0.8 + 0.1;
                setAudioLevel(randomLevel);
              } else {
                clearInterval(levelInterval);
                setAudioLevel(0);
              }
            }, 100);
            
            return; // 成功重试，直接返回
          } catch (retryError) {
            console.error('重试开始录音失败:', retryError);
          }
        }
        
        setRecording(false);
        recordingTimer.stopRecording();
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
      
      logger.ai('AI增强文本已处理完成', enhancedText);
    } catch (error) {
      console.error('处理增强文本失败:', error);
      // setAiProcessingActive(false);
    }
  };

  // 暴露统一的录音切换函数给子组件使用
  useEffect(() => {
    window.appToggleRecording = handleFloatingDialogToggleRecording;
    return () => {
      delete window.appToggleRecording;
    };
  }, []);

  return (
    <div className="app">
      {/* 侧边栏 */}
      <div className="sidebar">
        <div className="sidebar-header">
          <div className="app-logo">
            <span className="logo-icon">●</span>
            <span className="logo-text">Recording King</span>
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
          selectedModel={selectedModel}
          setShowShortcutEditor={setShowShortcutEditor}
          setShowAppSelector={setShowAppSelector}
          setShowHistorySettings={setShowHistorySettings}
          setShowEnhancedHistory={setShowEnhancedHistory}
          setShowTextInjectionSettings={setShowTextInjectionSettings}
          setShowEnhancedShortcutManager={setShowEnhancedShortcutManager}
          audioDevices={audioDevices}
          trialInfo={trialInfo}
          setShowSubscriptionManager={setShowSubscriptionManager}
          onEnhancedTextReady={handleEnhancedTextReady}
          isRecording={isRecording}
          useAdvancedShortcuts={useAdvancedShortcuts}
          setUseAdvancedShortcuts={setUseAdvancedShortcuts}
          useEnhancedAIPrompts={useEnhancedAIPrompts}
          setUseEnhancedAIPrompts={setUseEnhancedAIPrompts}
          setSelectedEntry={setSelectedEntry}
          handleFloatingDialogToggleRecording={handleFloatingDialogToggleRecording}
          isTranscribing={isTranscribing}
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
          logger.ai('AI助手提示', prompt);
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
          logger.debug('选择的应用', app);
        }}
      />

      {/* 快捷键编辑器对话框 - 根据设置选择标准或高级版本 */}
      {useAdvancedShortcuts ? (
        <AdvancedShortcutEditor
          isVisible={showShortcutEditor}
          onClose={() => setShowShortcutEditor(false)}
        />
      ) : (
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
      )}

      {/* 历史记录设置对话框 */}
      <HistorySettings
        isVisible={showHistorySettings}
        onClose={() => setShowHistorySettings(false)}
        settings={historySettings}
        onUpdateSettings={(settings) => {
          setHistorySettings(settings);
          logger.debug('更新历史记录设置', settings);
        }}
      />

      {/* 功能测试面板 */}
      <FeatureTestPanel
        isVisible={showTestPanel}
        onClose={() => setShowTestPanel(false)}
      />

      {/* 音频输入测试对话框 */}
      <AudioInputTest
        isVisible={showAudioInputTest}
        onClose={() => setShowAudioInputTest(false)}
      />

      {/* 权限设置对话框 */}
      <PermissionSettings
        isVisible={showPermissionSettings}
        onClose={() => setShowPermissionSettings(false)}
        onPermissionsConfigured={() => {
          logger.info('权限已配置');
          // 重新注册快捷键
          shortcutManager.registerAllShortcuts();
        }}
      />

      {/* 首次启动向导 */}
      <FirstLaunchWizard
        isVisible={showFirstLaunchWizard}
        onComplete={() => {
          setShowFirstLaunchWizard(false);
          logger.info('首次设置完成');
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

      {/* 转录详情查看器 */}
      <TranscriptionDetailView
        entry={selectedEntry}
        isVisible={!!selectedEntry}
        onClose={() => setSelectedEntry(null)}
      />

      {/* 增强历史记录页面 */}
      <EnhancedHistoryPage
        isVisible={showEnhancedHistory}
        onClose={() => setShowEnhancedHistory(false)}
        onOpenTranscriptionDetail={(entry) => {
          setSelectedEntry(entry);
          setShowEnhancedHistory(false);
        }}
      />

      {/* 文本注入设置 */}
      <TextInjectionSettings
        isVisible={showTextInjectionSettings}
        onClose={() => setShowTextInjectionSettings(false)}
        onConfigChange={(config) => {
          console.log('文本注入配置更新:', config);
        }}
      />

      {/* 录音状态指示器 - 临时禁用避免状态冲突 */}
      {false && (
        <RecordingStatusIndicator
          isRecording={isRecording}
          recordingDuration={recordingDuration}
          audioLevel={audioLevel}
          selectedModel={selectedModel}
          onToggleRecording={handleFloatingDialogToggleRecording}
          shortcutKey="Cmd+Shift+R"
          showFloating={false}
          position="bottom-right"
        />
      )}

      {/* 增强快捷键管理器 */}
      <EnhancedShortcutManager
        isVisible={showEnhancedShortcutManager}
        onClose={() => setShowEnhancedShortcutManager(false)}
      />

      {/* 试用状态提示 - 已移除以避免过度商业化 */}
    </div>
  );
}

export default App;
