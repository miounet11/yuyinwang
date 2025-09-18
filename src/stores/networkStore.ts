// Story 1.4: Network Status and Transcription Mode Store

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export interface NetworkStatus {
  status: 'Online' | 'Offline' | 'Limited' | 'Unknown';
  is_connected: boolean;
  quality_score: number;
  consecutive_failures: number;
  last_checked: number;
}

export interface TranscriptionModeStatus {
  current_mode: string;
  active_mode: string;
  user_preferred_mode: string;
  network_status: string;
  network_quality: number;
  auto_switch_enabled: boolean;
  recommendation?: string;
}

export interface ModeConfig {
  auto_switch_enabled: boolean;
  cloud_api_timeout_ms: number;
  local_model_priority: boolean;
  network_quality_threshold: number;
  switch_debounce_ms: number;
}

export interface ModeChangeEvent {
  from_mode: string;
  to_mode: string;
  reason: string;
  automatic: boolean;
  timestamp: string;
}

export interface NetworkEvent {
  status: string;
  quality_score: number;
  timestamp: string;
}

interface NetworkStore {
  // 网络状态
  networkStatus: NetworkStatus | null;
  isNetworkLoading: boolean;
  networkError: string | null;

  // 转录模式状态
  modeStatus: TranscriptionModeStatus | null;
  isModeLoading: boolean;
  modeError: string | null;

  // 模式配置
  modeConfig: ModeConfig | null;

  // 事件历史
  modeChangeHistory: ModeChangeEvent[];
  networkChangeHistory: NetworkEvent[];

  // Actions
  fetchNetworkStatus: () => Promise<void>;
  checkNetworkNow: () => Promise<void>;
  testApiEndpoint: (url: string) => Promise<boolean>;

  fetchModeStatus: () => Promise<void>;
  setTranscriptionMode: (mode: string) => Promise<void>;
  updateModeConfig: (config: ModeConfig) => Promise<void>;
  forceReevaluateMode: () => Promise<void>;

  // 事件订阅
  subscribeToNetworkChanges: () => Promise<void>;
  subscribeToModeChanges: () => Promise<void>;

  // 清理
  clearError: () => void;
  clearHistory: () => void;
}

export const useNetworkStore = create<NetworkStore>((set, get) => ({
  // 初始状态
  networkStatus: null,
  isNetworkLoading: false,
  networkError: null,

  modeStatus: null,
  isModeLoading: false,
  modeError: null,

  modeConfig: null,
  modeChangeHistory: [],
  networkChangeHistory: [],

  // 网络状态相关操作
  fetchNetworkStatus: async () => {
    set({ isNetworkLoading: true, networkError: null });
    try {
      const status = await invoke<NetworkStatus>('get_network_status');
      set({ networkStatus: status, isNetworkLoading: false });
    } catch (error) {
      console.error('Failed to fetch network status:', error);
      set({
        networkError: error instanceof Error ? error.message : 'Unknown error',
        isNetworkLoading: false
      });
    }
  },

  checkNetworkNow: async () => {
    set({ isNetworkLoading: true });
    try {
      const statusStr = await invoke<string>('check_network_now');
      // 立即获取最新状态
      await get().fetchNetworkStatus();
      console.log('Network check completed:', statusStr);
    } catch (error) {
      console.error('Failed to check network:', error);
      set({
        networkError: error instanceof Error ? error.message : 'Unknown error',
        isNetworkLoading: false
      });
    }
  },

  testApiEndpoint: async (url: string) => {
    try {
      const result = await invoke<{success: boolean; latency_ms?: number; message: string}>('test_api_endpoint', { url });
      return result.success;
    } catch (error) {
      console.error('Failed to test API endpoint:', error);
      return false;
    }
  },

  // 转录模式相关操作
  fetchModeStatus: async () => {
    set({ isModeLoading: true, modeError: null });
    try {
      const status = await invoke<TranscriptionModeStatus>('get_transcription_mode_status');
      set({ modeStatus: status, isModeLoading: false });
    } catch (error) {
      console.error('Failed to fetch mode status:', error);
      set({
        modeError: error instanceof Error ? error.message : 'Unknown error',
        isModeLoading: false
      });
    }
  },

  setTranscriptionMode: async (mode: string) => {
    set({ isModeLoading: true, modeError: null });
    try {
      await invoke('set_transcription_mode', { mode });
      // 重新获取状态
      await get().fetchModeStatus();
      console.log('Transcription mode set to:', mode);
    } catch (error) {
      console.error('Failed to set transcription mode:', error);
      set({
        modeError: error instanceof Error ? error.message : 'Unknown error',
        isModeLoading: false
      });
    }
  },

  updateModeConfig: async (config: ModeConfig) => {
    try {
      await invoke('update_mode_config', { config });
      set({ modeConfig: config });
      console.log('Mode config updated:', config);
    } catch (error) {
      console.error('Failed to update mode config:', error);
      set({ modeError: error instanceof Error ? error.message : 'Unknown error' });
    }
  },

  forceReevaluateMode: async () => {
    set({ isModeLoading: true });
    try {
      const newMode = await invoke<string>('force_reevaluate_mode');
      await get().fetchModeStatus();
      console.log('Mode reevaluated to:', newMode);
    } catch (error) {
      console.error('Failed to reevaluate mode:', error);
      set({
        modeError: error instanceof Error ? error.message : 'Unknown error',
        isModeLoading: false
      });
    }
  },

  // 事件订阅
  subscribeToNetworkChanges: async () => {
    try {
      // 订阅网络状态变化事件
      await invoke('subscribe_network_changes');

      // 监听网络状态事件
      const unlisten = await listen<NetworkEvent>('network_status_event', (event) => {
        const networkEvent = event.payload;
        console.log('Network status changed:', networkEvent);

        // 更新历史记录
        set((state) => ({
          networkChangeHistory: [
            networkEvent,
            ...state.networkChangeHistory.slice(0, 49) // 保留最近50条
          ]
        }));

        // 刷新网络状态
        get().fetchNetworkStatus();
      });

      return unlisten;
    } catch (error) {
      console.error('Failed to subscribe to network changes:', error);
    }
  },

  subscribeToModeChanges: async () => {
    try {
      // 订阅模式变化事件
      await invoke('subscribe_mode_changes');

      // 监听模式变化事件
      const unlisten = await listen<ModeChangeEvent>('mode_change_event', (event) => {
        const modeEvent = event.payload;
        console.log('Mode changed:', modeEvent);

        // 更新历史记录
        set((state) => ({
          modeChangeHistory: [
            modeEvent,
            ...state.modeChangeHistory.slice(0, 49) // 保留最近50条
          ]
        }));

        // 刷新模式状态
        get().fetchModeStatus();
      });

      return unlisten;
    } catch (error) {
      console.error('Failed to subscribe to mode changes:', error);
    }
  },

  // 清理操作
  clearError: () => {
    set({ networkError: null, modeError: null });
  },

  clearHistory: () => {
    set({
      modeChangeHistory: [],
      networkChangeHistory: []
    });
  }
}));
