import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';
import type { ToastData, ToastType } from '../components/Toast';
import type {
  AppSettings,
  AudioDevice,
  TranscriptionEntry,
  ShortcutSettingsState,
  AIPrompt,
  OnboardingState,
  WordReplacement,
  ShortcutPreset,
  CustomShortcut,
  ActivationMode,
} from '../types';
import { presetToTauriKey, customShortcutToTauriKey, reorderMicrophones as reorderMicrophonesUtil } from '../utils';

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

  // 新增状态
  shortcutSettings: ShortcutSettingsState;
  aiPrompts: AIPrompt[];
  onboardingState: OnboardingState;
  wordReplacements: WordReplacement[];

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

  // 新增操作
  setShortcutPreset: (preset: ShortcutPreset) => Promise<void>;
  setCustomShortcut: (shortcut: CustomShortcut) => Promise<void>;
  setActivationMode: (mode: ActivationMode) => void;
  reorderMicrophones: (fromIndex: number, toIndex: number) => void;
  addAIPrompt: (prompt: AIPrompt) => Promise<void>;
  updateAIPrompt: (id: string, updates: Partial<AIPrompt>) => Promise<void>;
  deleteAIPrompt: (id: string) => Promise<void>;
  setOnboardingStep: (step: number) => void;
  completeOnboarding: () => void;
  addWordReplacement: (replacement: WordReplacement) => void;
  updateWordReplacement: (id: string, updates: Partial<WordReplacement>) => void;
  deleteWordReplacement: (id: string) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
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
    display_style: 'panel',
    appearance: 'system',
    ui_language: 'system',
    launch_at_login: false,
    show_in_dock: true,
    show_in_menu_bar: true,
    esc_to_cancel: true,
    shortcut_preset: 'none',
    activation_mode: 'hold-or-toggle',
    microphone_priority: [],
    onboarding_complete: false,
    word_replacements: [],
    transcription_language: 'auto',
    transcription_prompt: '',
  },
  toasts: [],
  isInitializing: false,
  initError: null,

  // 新增状态初始值
  shortcutSettings: {
    selectedShortcut: 'none',
    activationMode: 'hold-or-toggle',
    escToCancel: true,
    testText: '',
  },
  aiPrompts: [],
  onboardingState: {
    currentStep: 0,
    totalSteps: 4,
    completedSteps: new Set(),
  },
  wordReplacements: [],

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

  // 新增操作实现
  setShortcutPreset: async (preset) => {
    const tauriKey = presetToTauriKey(preset);
    if (tauriKey) {
      try {
        const mode = get().shortcutSettings.activationMode;
        await invoke('register_global_shortcut', { key: tauriKey, activationMode: mode });
        set((state) => ({
          settings: { ...state.settings, shortcut_preset: preset },
          shortcutSettings: { ...state.shortcutSettings, selectedShortcut: preset },
        }));
      } catch (error) {
        get().addToast('error', '快捷键已被其他应用占用，请选择其他组合');
      }
    } else if (preset === 'none') {
      // 取消快捷键
      try {
        const currentKey = presetToTauriKey(
          typeof get().shortcutSettings.selectedShortcut === 'string'
            ? get().shortcutSettings.selectedShortcut as ShortcutPreset
            : 'none'
        );
        if (currentKey) {
          await invoke('unregister_global_shortcut', { key: currentKey });
        }
      } catch (_) {}
      set((state) => ({
        settings: { ...state.settings, shortcut_preset: 'none' },
        shortcutSettings: { ...state.shortcutSettings, selectedShortcut: 'none' },
      }));
    }
  },

  setCustomShortcut: async (shortcut) => {
    const tauriKey = customShortcutToTauriKey(shortcut);
    try {
      const mode = get().shortcutSettings.activationMode;
      await invoke('register_global_shortcut', { key: tauriKey, activationMode: mode });
      set((state) => ({
        settings: { ...state.settings, custom_shortcut: shortcut, shortcut_preset: 'custom' },
        shortcutSettings: { ...state.shortcutSettings, selectedShortcut: shortcut },
      }));
    } catch (error) {
      get().addToast('error', '快捷键已被其他应用占用，请选择其他组合');
    }
  },

  setActivationMode: (mode) => {
    set((state) => ({
      settings: { ...state.settings, activation_mode: mode },
      shortcutSettings: { ...state.shortcutSettings, activationMode: mode },
    }));
    // 持久化到后端并更新监听器
    invoke('update_activation_mode', { mode }).catch(console.error);
  },

  reorderMicrophones: (fromIndex, toIndex) => {
    set((state) => {
      const reordered = reorderMicrophonesUtil(state.audioDevices, fromIndex, toIndex);
      const priorityIds = reordered.map((d) => d.id);
      return {
        audioDevices: reordered,
        settings: { ...state.settings, microphone_priority: priorityIds },
      };
    });
    // 持久化到后端
    const priorityIds = get().audioDevices.map((d) => d.id);
    invoke('update_settings', { settings: { microphone_priority: priorityIds } }).catch(console.error);
  },

  addAIPrompt: async (prompt) => {
    set((state) => ({ aiPrompts: [...state.aiPrompts, prompt] }));
    // 持久化到本地存储
    await invoke('save_ai_prompts', { prompts: get().aiPrompts }).catch(console.error);
  },

  updateAIPrompt: async (id, updates) => {
    set((state) => ({
      aiPrompts: state.aiPrompts.map((p) => (p.id === id ? { ...p, ...updates } : p)),
    }));
    // 持久化到本地存储
    await invoke('save_ai_prompts', { prompts: get().aiPrompts }).catch(console.error);
  },

  deleteAIPrompt: async (id) => {
    set((state) => ({ aiPrompts: state.aiPrompts.filter((p) => p.id !== id) }));
    // 持久化到本地存储
    await invoke('save_ai_prompts', { prompts: get().aiPrompts }).catch(console.error);
  },

  setOnboardingStep: (step) => {
    set((state) => ({
      onboardingState: {
        ...state.onboardingState,
        currentStep: Math.max(state.onboardingState.currentStep, step), // 只能前进
        completedSteps: new Set([...state.onboardingState.completedSteps, step - 1]),
      },
    }));
  },

  completeOnboarding: () => {
    set((state) => ({
      settings: { ...state.settings, onboarding_complete: true },
      onboardingState: {
        ...state.onboardingState,
        currentStep: state.onboardingState.totalSteps,
      },
    }));
    // 持久化到后端
    invoke('update_settings', { settings: { onboarding_complete: true } }).catch(console.error);
  },

  addWordReplacement: (replacement) => {
    set((state) => ({
      wordReplacements: [...state.wordReplacements, replacement],
      settings: { ...state.settings, word_replacements: [...state.wordReplacements, replacement] },
    }));
  },

  updateWordReplacement: (id, updates) => {
    set((state) => {
      const updated = state.wordReplacements.map((r) => (r.id === id ? { ...r, ...updates } : r));
      return {
        wordReplacements: updated,
        settings: { ...state.settings, word_replacements: updated },
      };
    });
  },

  deleteWordReplacement: (id) => {
    set((state) => {
      const filtered = state.wordReplacements.filter((r) => r.id !== id);
      return {
        wordReplacements: filtered,
        settings: { ...state.settings, word_replacements: filtered },
      };
    });
  },
}));
