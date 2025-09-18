import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

interface AudioVisualizationData {
  amplitude: number;
  frequency_data: number[];
  time_stamps: number[];
  voice_activity_detected: boolean;
  noise_level: number;
  peak_detected: boolean;
  response_time_ms: number;
}

interface WaveformColorScheme {
  low_amplitude: string;
  mid_amplitude: string;
  high_amplitude: string;
  background: string;
  peak_indicator: string;
}

interface WaveformConfig {
  buffer_size: number;
  sample_rate: number;
  render_mode: "RealTime" | "Static" | "Miniature";
  color_scheme: WaveformColorScheme;
  max_response_time_ms: number;
}

interface VisualizationMetrics {
  buffer_usage_amplitude: number;
  buffer_usage_frequency: number;
  max_buffer_size: number;
  sample_rate: number;
  render_mode: "RealTime" | "Static" | "Miniature";
  max_response_time_ms: number;
}

interface UseAudioVisualizationOptions {
  /** 自动启动订阅 */
  autoStart?: boolean;
  /** 可视化配置 */
  config?: Partial<WaveformConfig>;
  /** 性能监控回调 */
  onPerformanceUpdate?: (responseTime: number) => void;
  /** 语音活动检测回调 */
  onVoiceActivity?: (detected: boolean) => void;
  /** 错误处理回调 */
  onError?: (error: string) => void;
}

interface UseAudioVisualizationReturn {
  /** 当前可视化数据 */
  visualizationData: AudioVisualizationData | null;
  /** 是否已订阅 */
  isSubscribed: boolean;
  /** 性能指标 */
  metrics: VisualizationMetrics | null;
  /** 错误状态 */
  error: string | null;
  /** 开始订阅 */
  startVisualization: () => Promise<void>;
  /** 停止订阅 */
  stopVisualization: () => Promise<void>;
  /** 更新配置 */
  updateConfig: (newConfig: Partial<WaveformConfig>) => Promise<void>;
  /** 清除历史数据 */
  clearHistory: () => Promise<void>;
  /** 获取性能指标 */
  getMetrics: () => Promise<VisualizationMetrics | null>;
}

export function useAudioVisualization(
  options: UseAudioVisualizationOptions = {},
): UseAudioVisualizationReturn {
  const {
    autoStart = false,
    config,
    onPerformanceUpdate,
    onVoiceActivity,
    onError,
  } = options;

  // 状态管理
  const [visualizationData, setVisualizationData] =
    useState<AudioVisualizationData | null>(null);
  const [isSubscribed, setIsSubscribed] = useState(false);
  const [metrics, setMetrics] = useState<VisualizationMetrics | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 引用管理
  const subscriptionIdRef = useRef<string>();
  const unlistenRef = useRef<(() => void) | null>(null);

  // 默认配置
  const defaultConfig: WaveformConfig = {
    buffer_size: 1024,
    sample_rate: 44100,
    render_mode: "RealTime",
    color_scheme: {
      low_amplitude: "#4caf50",
      mid_amplitude: "#ff9800",
      high_amplitude: "#f44336",
      background: "#1a1a1a",
      peak_indicator: "#ffeb3b",
    },
    max_response_time_ms: 16,
  };

  // 合并配置
  const mergedConfig = { ...defaultConfig, ...config };

  // 开始可视化订阅
  const startVisualization = useCallback(async () => {
    if (isSubscribed) {
      return;
    }

    try {
      setError(null);

      const subscriptionId = (await invoke("subscribe_audio_visualization", {
        request: {
          config: mergedConfig,
          enable_real_time: true,
        },
      })) as string;

      subscriptionIdRef.current = subscriptionId;
      setIsSubscribed(true);

      // 监听可视化数据更新
      const unlisten = await listen(
        "audio_visualization_update",
        (event: any) => {
          const updateEvent = event.payload;

          if (updateEvent.subscription_id === subscriptionId) {
            const data: AudioVisualizationData = updateEvent.data;
            setVisualizationData(data);

            // 触发回调
            onPerformanceUpdate?.(data.response_time_ms);
            onVoiceActivity?.(data.voice_activity_detected);
          }
        },
      );

      unlistenRef.current = unlisten;

      // 获取初始性能指标
      await getMetrics();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      onError?.(errorMessage);
      setIsSubscribed(false);
    }
  }, [
    isSubscribed,
    mergedConfig,
    onPerformanceUpdate,
    onVoiceActivity,
    onError,
  ]);

  // 停止可视化订阅
  const stopVisualization = useCallback(async () => {
    if (!isSubscribed || !subscriptionIdRef.current) {
      return;
    }

    try {
      await invoke("unsubscribe_audio_visualization", {
        subscriptionId: subscriptionIdRef.current,
      });

      // 清理监听器
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }

      setIsSubscribed(false);
      subscriptionIdRef.current = undefined;
      setVisualizationData(null);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      onError?.(errorMessage);
    }
  }, [isSubscribed, onError]);

  // 更新配置
  const updateConfig = useCallback(
    async (newConfig: Partial<WaveformConfig>) => {
      if (!isSubscribed || !subscriptionIdRef.current) {
        throw new Error("Not subscribed to visualization");
      }

      try {
        const updatedConfig = { ...mergedConfig, ...newConfig };

        const success = (await invoke("update_visualization_config", {
          subscriptionId: subscriptionIdRef.current,
          newConfig: updatedConfig,
        })) as boolean;

        if (!success) {
          throw new Error("Failed to update visualization config");
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        onError?.(errorMessage);
        throw err;
      }
    },
    [isSubscribed, mergedConfig, onError],
  );

  // 清除历史数据
  const clearHistory = useCallback(async () => {
    if (!isSubscribed || !subscriptionIdRef.current) {
      return;
    }

    try {
      const success = (await invoke("clear_visualization_history", {
        subscriptionId: subscriptionIdRef.current,
      })) as boolean;

      if (!success) {
        throw new Error("Failed to clear visualization history");
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      onError?.(errorMessage);
    }
  }, [isSubscribed, onError]);

  // 获取性能指标
  const getMetrics =
    useCallback(async (): Promise<VisualizationMetrics | null> => {
      if (!isSubscribed || !subscriptionIdRef.current) {
        return null;
      }

      try {
        const metricsData = (await invoke("get_visualization_metrics", {
          subscriptionId: subscriptionIdRef.current,
        })) as VisualizationMetrics;

        setMetrics(metricsData);
        return metricsData;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        onError?.(errorMessage);
        return null;
      }
    }, [isSubscribed, onError]);

  // 自动启动
  useEffect(() => {
    if (autoStart) {
      startVisualization();
    }

    // 清理函数
    return () => {
      if (isSubscribed) {
        stopVisualization();
      }
    };
  }, [autoStart]); // 只在 autoStart 改变时执行

  // 配置变更时更新
  useEffect(() => {
    if (isSubscribed && config) {
      updateConfig(config).catch(console.error);
    }
  }, [config, isSubscribed]); // 在配置或订阅状态改变时执行

  return {
    visualizationData,
    isSubscribed,
    metrics,
    error,
    startVisualization,
    stopVisualization,
    updateConfig,
    clearHistory,
    getMetrics,
  };
}

// 获取预设颜色方案的 hook
export function useWaveformColorSchemes() {
  const [colorSchemes, setColorSchemes] = useState<WaveformColorScheme[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadColorSchemes = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const schemes = (await invoke(
        "get_waveform_color_schemes",
      )) as WaveformColorScheme[];
      setColorSchemes(schemes);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadColorSchemes();
  }, [loadColorSchemes]);

  return {
    colorSchemes,
    loading,
    error,
    reload: loadColorSchemes,
  };
}

// 语音活动检测阈值设置的 hook
export function useVoiceActivityThreshold() {
  const [threshold, setThreshold] = useState<number>(0.1);
  const [error, setError] = useState<string | null>(null);

  const updateThreshold = useCallback(async (newThreshold: number) => {
    if (newThreshold < 0 || newThreshold > 1) {
      setError("Threshold must be between 0 and 1");
      return;
    }

    try {
      await invoke("set_voice_activity_threshold", { threshold: newThreshold });
      setThreshold(newThreshold);
      setError(null);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
    }
  }, []);

  return {
    threshold,
    error,
    updateThreshold,
  };
}
