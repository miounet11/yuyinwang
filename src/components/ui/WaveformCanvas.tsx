import React, { useRef, useEffect, useCallback, useMemo } from "react";
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

interface WaveformCanvasProps {
  /** Canvas 宽度 */
  width: number;
  /** Canvas 高度 */
  height: number;
  /** 渲染模式 */
  renderMode?: "RealTime" | "Static" | "Miniature";
  /** 颜色主题 */
  colorScheme?: WaveformColorScheme;
  /** 是否启用实时更新 */
  enableRealTime?: boolean;
  /** 自定义配置 */
  config?: Partial<WaveformConfig>;
  /** 性能监控回调 */
  onPerformanceUpdate?: (responseTime: number) => void;
  /** 语音活动检测回调 */
  onVoiceActivity?: (detected: boolean) => void;
  /** CSS 类名 */
  className?: string;
  /** 样式 */
  style?: React.CSSProperties;
}

const defaultColorScheme: WaveformColorScheme = {
  low_amplitude: "#4caf50",
  mid_amplitude: "#ff9800",
  high_amplitude: "#f44336",
  background: "#1a1a1a",
  peak_indicator: "#ffeb3b",
};

const defaultConfig: WaveformConfig = {
  buffer_size: 1024,
  sample_rate: 44100,
  render_mode: "RealTime",
  color_scheme: defaultColorScheme,
  max_response_time_ms: 16, // 60 FPS
};

export default function WaveformCanvas({
  width,
  height,
  renderMode = "RealTime",
  colorScheme = defaultColorScheme,
  enableRealTime = true,
  config,
  onPerformanceUpdate,
  onVoiceActivity,
  className,
  style,
}: WaveformCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationFrameRef = useRef<number>();
  const subscriptionIdRef = useRef<string>();
  const lastRenderTimeRef = useRef<number>(Date.now());

  // 合并配置
  const mergedConfig = useMemo(
    () => ({
      ...defaultConfig,
      render_mode: renderMode,
      color_scheme: colorScheme,
      ...config,
    }),
    [renderMode, colorScheme, config],
  );

  // 音频可视化数据状态
  const [visualizationData, setVisualizationData] =
    React.useState<AudioVisualizationData | null>(null);
  const [isSubscribed, setIsSubscribed] = React.useState(false);
  const [performanceMetrics, setPerformanceMetrics] = React.useState({
    responseTime: 0,
    fps: 0,
  });

  // 绘制波形
  const drawWaveform = useCallback(
    (ctx: CanvasRenderingContext2D, data: AudioVisualizationData) => {
      const { width: canvasWidth, height: canvasHeight } = ctx.canvas;

      // 清除画布
      ctx.fillStyle = mergedConfig.color_scheme.background;
      ctx.fillRect(0, 0, canvasWidth, canvasHeight);

      const {
        frequency_data,
        amplitude,
        peak_detected,
        voice_activity_detected,
      } = data;

      if (!frequency_data || frequency_data.length === 0) {
        return;
      }

      // 根据渲染模式选择绘制方式
      switch (mergedConfig.render_mode) {
        case "RealTime":
          drawRealtimeWaveform(ctx, data);
          break;
        case "Static":
          drawStaticWaveform(ctx, data);
          break;
        case "Miniature":
          drawMiniatureWaveform(ctx, data);
          break;
      }

      // 绘制峰值指示器
      if (peak_detected) {
        drawPeakIndicator(ctx, amplitude);
      }

      // 绘制语音活动指示器
      if (voice_activity_detected) {
        drawVoiceActivityIndicator(ctx);
      }
    },
    [mergedConfig],
  );

  // 实时波形绘制
  const drawRealtimeWaveform = useCallback(
    (ctx: CanvasRenderingContext2D, data: AudioVisualizationData) => {
      const { width: canvasWidth, height: canvasHeight } = ctx.canvas;
      const { frequency_data } = data;

      const barWidth = canvasWidth / frequency_data.length;
      const centerY = canvasHeight / 2;

      frequency_data.forEach((amplitude, index) => {
        const barHeight = amplitude * centerY;
        const x = index * barWidth;

        // 根据振幅选择颜色
        const color = getAmplitudeColor(amplitude);
        ctx.fillStyle = color;

        // 绘制对称的条形
        ctx.fillRect(x, centerY - barHeight / 2, barWidth - 1, barHeight);
      });
    },
    [],
  );

  // 静态波形绘制
  const drawStaticWaveform = useCallback(
    (ctx: CanvasRenderingContext2D, data: AudioVisualizationData) => {
      const { width: canvasWidth, height: canvasHeight } = ctx.canvas;
      const { frequency_data } = data;

      ctx.strokeStyle = mergedConfig.color_scheme.mid_amplitude;
      ctx.lineWidth = 2;
      ctx.beginPath();

      const stepX = canvasWidth / frequency_data.length;
      const centerY = canvasHeight / 2;

      frequency_data.forEach((amplitude, index) => {
        const x = index * stepX;
        const y = centerY - amplitude * centerY * 0.8;

        if (index === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      });

      ctx.stroke();
    },
    [mergedConfig],
  );

  // 小型波形绘制
  const drawMiniatureWaveform = useCallback(
    (ctx: CanvasRenderingContext2D, data: AudioVisualizationData) => {
      const { width: canvasWidth, height: canvasHeight } = ctx.canvas;
      const { amplitude } = data;

      // 简单的振幅指示圆
      const centerX = canvasWidth / 2;
      const centerY = canvasHeight / 2;
      const maxRadius = Math.min(centerX, centerY) * 0.8;
      const radius = amplitude * maxRadius;

      const color = getAmplitudeColor(amplitude);

      // 绘制外圈
      ctx.strokeStyle = mergedConfig.color_scheme.background;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(centerX, centerY, maxRadius, 0, 2 * Math.PI);
      ctx.stroke();

      // 绘制振幅圆
      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.arc(centerX, centerY, radius, 0, 2 * Math.PI);
      ctx.fill();
    },
    [mergedConfig],
  );

  // 根据振幅获取颜色
  const getAmplitudeColor = useCallback(
    (amplitude: number): string => {
      if (amplitude < 0.3) {
        return mergedConfig.color_scheme.low_amplitude;
      } else if (amplitude < 0.7) {
        return mergedConfig.color_scheme.mid_amplitude;
      } else {
        return mergedConfig.color_scheme.high_amplitude;
      }
    },
    [mergedConfig],
  );

  // 绘制峰值指示器
  const drawPeakIndicator = useCallback(
    (ctx: CanvasRenderingContext2D, amplitude: number) => {
      const { width: canvasWidth, height: canvasHeight } = ctx.canvas;

      ctx.fillStyle = mergedConfig.color_scheme.peak_indicator;
      ctx.fillRect(canvasWidth - 10, 5, 5, canvasHeight * amplitude);
    },
    [mergedConfig],
  );

  // 绘制语音活动指示器
  const drawVoiceActivityIndicator = useCallback(
    (ctx: CanvasRenderingContext2D) => {
      const { width: canvasWidth } = ctx.canvas;

      ctx.fillStyle = mergedConfig.color_scheme.peak_indicator;
      ctx.fillRect(5, 5, 10, 10);

      // 添加"VOICE"文字
      ctx.fillStyle = "white";
      ctx.font = "10px Arial";
      ctx.fillText("VOICE", 20, 14);
    },
    [mergedConfig],
  );

  // 订阅音频可视化数据
  const subscribeToVisualization = useCallback(async () => {
    if (isSubscribed || !enableRealTime) {
      return;
    }

    try {
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
            setVisualizationData(updateEvent.data);

            // 性能监控
            const now = Date.now();
            const fps = 1000 / (now - lastRenderTimeRef.current);
            lastRenderTimeRef.current = now;

            const metrics = {
              responseTime: updateEvent.data.response_time_ms,
              fps: Math.round(fps),
            };
            setPerformanceMetrics(metrics);

            // 回调通知
            onPerformanceUpdate?.(updateEvent.data.response_time_ms);
            onVoiceActivity?.(updateEvent.data.voice_activity_detected);
          }
        },
      );

      // 清理函数
      return unlisten;
    } catch (error) {
      console.error("订阅音频可视化失败:", error);
      setIsSubscribed(false);
    }
  }, [
    isSubscribed,
    enableRealTime,
    mergedConfig,
    onPerformanceUpdate,
    onVoiceActivity,
  ]);

  // 取消订阅
  const unsubscribeFromVisualization = useCallback(async () => {
    if (!isSubscribed || !subscriptionIdRef.current) {
      return;
    }

    try {
      await invoke("unsubscribe_audio_visualization", {
        subscriptionId: subscriptionIdRef.current,
      });

      setIsSubscribed(false);
      subscriptionIdRef.current = undefined;
    } catch (error) {
      console.error("取消订阅音频可视化失败:", error);
    }
  }, [isSubscribed]);

  // 渲染循环
  const renderLoop = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas || !visualizationData) {
      return;
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }

    drawWaveform(ctx, visualizationData);

    if (enableRealTime && isSubscribed) {
      animationFrameRef.current = requestAnimationFrame(renderLoop);
    }
  }, [visualizationData, drawWaveform, enableRealTime, isSubscribed]);

  // 初始化和清理
  useEffect(() => {
    if (enableRealTime) {
      subscribeToVisualization();
    }

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      unsubscribeFromVisualization();
    };
  }, [enableRealTime, subscribeToVisualization, unsubscribeFromVisualization]);

  // 开始渲染循环
  useEffect(() => {
    if (visualizationData && isSubscribed) {
      renderLoop();
    }
  }, [visualizationData, isSubscribed, renderLoop]);

  // 处理配置变更
  useEffect(() => {
    if (isSubscribed && subscriptionIdRef.current) {
      invoke("update_visualization_config", {
        subscriptionId: subscriptionIdRef.current,
        newConfig: mergedConfig,
      }).catch(console.error);
    }
  }, [mergedConfig, isSubscribed]);

  return (
    <div className={className} style={style}>
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        style={{
          width: `${width}px`,
          height: `${height}px`,
          backgroundColor: mergedConfig.color_scheme.background,
          borderRadius: "4px",
        }}
      />

      {/* 性能指标显示（开发模式） */}
      {process.env.NODE_ENV === "development" && (
        <div
          style={{
            position: "absolute",
            top: 0,
            right: 0,
            background: "rgba(0,0,0,0.7)",
            color: "white",
            padding: "4px 8px",
            fontSize: "10px",
            borderRadius: "0 0 0 4px",
          }}
        >
          {performanceMetrics.responseTime}ms | {performanceMetrics.fps}fps
        </div>
      )}
    </div>
  );
}
