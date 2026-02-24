import { create } from 'zustand';
import type { ToastData, ToastType } from '../components/Toast';

interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
  is_available: boolean;
}

interface TranscriptionEntry {
  id: string;
  text: string;
  timestamp: number;
  duration: number;
  model: string;
  confidence: number;
  audio_file_path?: string;
}

interface AppSettings {
  openai_api_key?: string;
  luyin_token?: string;
  selected_model: string;
  auto_inject: boolean;
  inject_delay_ms: number;
  shortcut_key?: string;
}

interface AppStore {
  // State
  isRecording: boolean;
  currentPage: string;
  transcriptionText: string;
  audioDevices: AudioDevice[];
  history: TranscriptionEntry[];
  settings: AppSettings;
  toasts: ToastData[];
  isInitializing: boolean;
  initError: string | null;

  // Actions
  setRecording: (value: boolean) => void;
  setCurrentPage: (page: string) => void;
  setTranscriptionText: (text: string) => void;
  setAudioDevices: (devices: AudioDevice[]) => void;
  setHistory: (history: TranscriptionEntry[]) => void;
  addHistoryEntry: (entry: TranscriptionEntry) => void;
  setSettings: (settings: AppSettings) => void;
  addToast: (type: ToastType, message: string, duration?: number) => void;
  removeToast: (id: string) => void;
  setInitializing: (value: boolean) => void;
  setInitError: (error: string | null) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  // Initial state
  isRecording: false,
  currentPage: 'recording',
  transcriptionText: '',
  audioDevices: [],
  history: [],
  settings: {
    selected_model: 'luyin-free',
    auto_inject: false,
    inject_delay_ms: 100,
  },
  toasts: [],
  isInitializing: false,
  initError: null,

  // Actions
  setRecording: (value) => set({ isRecording: value }),
  setCurrentPage: (page) => set({ currentPage: page }),
  setTranscriptionText: (text) => set({ transcriptionText: text }),
  setAudioDevices: (devices) => set({ audioDevices: devices }),
  setHistory: (history) => set({ history }),
  addHistoryEntry: (entry) => set((state) => ({ history: [entry, ...state.history] })),
  setSettings: (settings) => set({ settings }),
  addToast: (type, message, duration) =>
    set((state) => ({
      toasts: [...state.toasts, { id: Date.now().toString(), type, message, duration }],
    })),
  removeToast: (id) =>
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    })),
  setInitializing: (value) => set({ isInitializing: value }),
  setInitError: (error) => set({ initError: error }),
}));
