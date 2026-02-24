import { create } from 'zustand';
import { ToastData, ToastType } from '../components/Toast';

interface AppState {
  // Error state
  error: string | null;
  setError: (error: string | null) => void;

  // Toast state
  toasts: ToastData[];
  addToast: (type: ToastType, message: string, duration?: number) => void;
  removeToast: (id: string) => void;

  // Loading state
  isInitializing: boolean;
  isTranscribing: boolean;
  initError: string | null;
  setInitializing: (value: boolean) => void;
  setTranscribing: (value: boolean) => void;
  setInitError: (error: string | null) => void;
}

const ERROR_MESSAGES: Record<string, string> = {
  'Recording already in progress': '录音已在进行中',
  'Not recording': '未在录音',
  'Failed to start recording': '启动录音失败',
  'Failed to stop recording': '停止录音失败',
  'Transcription failed': '转录失败',
  'API key not configured': 'API 密钥未配置',
  'Network error': '网络错误',
  'Permission denied': '权限被拒绝',
};

const translateError = (error: string): string => {
  return ERROR_MESSAGES[error] || error;
};

export const useAppStore = create<AppState>((set) => ({
  // Error state
  error: null,
  setError: (error) => set({ error }),

  // Toast state
  toasts: [],
  addToast: (type, message, duration) => {
    const id = `${Date.now()}-${Math.random()}`;
    const translatedMessage = translateError(message);
    set((state) => ({
      toasts: [...state.toasts, { id, type, message: translatedMessage, duration }],
    }));
  },
  removeToast: (id) => {
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    }));
  },

  // Loading state
  isInitializing: false,
  isTranscribing: false,
  initError: null,
  setInitializing: (value) => set({ isInitializing: value }),
  setTranscribing: (value) => set({ isTranscribing: value }),
  setInitError: (error) => set({ initError: error }),
}));
