// 拆分后的Zustand Store - 性能优化版本
import { create } from 'zustand';

export interface TranscriptionEntry {
  id: string;
  text: string;
  timestamp: number;
  duration: number;
  model: string;
  confidence: number;
  audio_file_path?: string;
}

export interface AudioDevice {
  name: string;
  id: string;
  is_default: boolean;
  is_available: boolean;
}

export interface McpConfig {
  enabled: boolean;
  server_url: string;
  api_key: string;
  model: string;
}

export interface AppStore {
  // 录音状态
  isRecording: boolean;
  transcriptionText: string;
  audioDevices: AudioDevice[];
  selectedDevice: string | null;
  
  // 应用配置
  language: string;
  hotkey: string;
  currentPage: string;
  selectedModel: string;
  
  // 历史和数据
  transcriptionHistory: TranscriptionEntry[];
  
  // AI和集成
  mcpConfig: McpConfig;
  aiProcessingActive: boolean;
  useEnhancedAIPrompts: boolean;
  
  // UI状态
  showFloatingDialog: boolean;
  
  // Actions
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

export const useStore = create<AppStore>((set, get) => ({
  // 初始状态
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
  
  // 优化的Actions - 使用函数式更新避免不必要的重渲染
  setRecording: (value) => set((state) => 
    state.isRecording !== value ? { isRecording: value } : {}
  ),
  
  setTranscription: (text) => set((state) => 
    state.transcriptionText !== text ? { transcriptionText: text } : {}
  ),
  
  setDevices: (devices) => set({ audioDevices: devices }),
  
  setSelectedDevice: (device) => set((state) => 
    state.selectedDevice !== device ? { selectedDevice: device } : {}
  ),
  
  setLanguage: (lang) => set((state) => 
    state.language !== lang ? { language: lang } : {}
  ),
  
  setHotkey: (key) => set((state) => 
    state.hotkey !== key ? { hotkey: key } : {}
  ),
  
  setCurrentPage: (page) => set((state) => 
    state.currentPage !== page ? { currentPage: page } : {}
  ),
  
  setSelectedModel: (model) => set((state) => 
    state.selectedModel !== model ? { selectedModel: model } : {}
  ),
  
  setTranscriptionHistory: (history) => set({ transcriptionHistory: history }),
  
  addTranscriptionEntry: (entry) => set((state) => ({
    transcriptionHistory: [entry, ...state.transcriptionHistory]
  })),
  
  setMcpConfig: (config) => set({ mcpConfig: config }),
  
  setShowFloatingDialog: (show) => set((state) => 
    state.showFloatingDialog !== show ? { showFloatingDialog: show } : {}
  ),
  
  setAiProcessingActive: (active) => set((state) => 
    state.aiProcessingActive !== active ? { aiProcessingActive: active } : {}
  ),
  
  setUseEnhancedAIPrompts: (use) => set((state) => 
    state.useEnhancedAIPrompts !== use ? { useEnhancedAIPrompts: use } : {}
  ),
}));

// 选择器函数 - 用于性能优化
export const selectRecordingState = (state: AppStore) => ({
  isRecording: state.isRecording,
  transcriptionText: state.transcriptionText,
  selectedModel: state.selectedModel,
});

export const selectAudioDevices = (state: AppStore) => ({
  audioDevices: state.audioDevices,
  selectedDevice: state.selectedDevice,
});

export const selectHistory = (state: AppStore) => ({
  transcriptionHistory: state.transcriptionHistory,
});

export const selectUIState = (state: AppStore) => ({
  currentPage: state.currentPage,
  showFloatingDialog: state.showFloatingDialog,
  aiProcessingActive: state.aiProcessingActive,
});

export const selectConfig = (state: AppStore) => ({
  language: state.language,
  hotkey: state.hotkey,
  mcpConfig: state.mcpConfig,
  useEnhancedAIPrompts: state.useEnhancedAIPrompts,
});