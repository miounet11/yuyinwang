import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

// 类型定义
interface TopicTag {
  name: string;
  confidence: number;
  category: string;
  keywords: string[];
  positions: any[];
}

interface SentimentAnalysis {
  overall_sentiment: "Positive" | "Negative" | "Neutral" | "Mixed";
  intensity: number;
  timeline: SentimentTimePoint[];
  tone_characteristics: ToneCharacteristics;
}

interface SentimentTimePoint {
  timestamp: number;
  sentiment_value: number;
  emotion_labels: string[];
}

interface ToneCharacteristics {
  formality: number;
  confidence: number;
  emotional_stability: number;
  professionalism: number;
}

interface KeyInformation {
  summary: StructuredSummary;
  entities: NamedEntity[];
  action_items: ActionItem[];
  data_points: DataPoint[];
}

interface StructuredSummary {
  main_points: string[];
  conclusions: string[];
  questions_discussed: string[];
  solutions_mentioned: string[];
}

interface NamedEntity {
  text: string;
  entity_type: string;
  confidence: number;
  context: string;
}

interface ActionItem {
  description: string;
  assignee?: string;
  due_date?: string;
  priority: "High" | "Medium" | "Low";
}

interface DataPoint {
  description: string;
  value?: number;
  unit?: string;
  data_type: string;
}

interface ContentClassification {
  suggested_categories: CategorySuggestion[];
  auto_tags: string[];
  similar_content: any[];
  knowledge_base_links: any[];
}

interface CategorySuggestion {
  category: string;
  confidence: number;
  reasoning: string;
}

interface AnalysisMetrics {
  topic_analysis_time_ms: number;
  sentiment_analysis_time_ms: number;
  key_info_extraction_time_ms: number;
  total_analysis_time_ms: number;
  ai_model_used: string;
  api_calls_made: number;
  characters_processed: number;
}

interface ContentAnalysisResult {
  topics: TopicTag[];
  sentiment: SentimentAnalysis;
  key_information: KeyInformation;
  classification: ContentClassification;
  performance_metrics: AnalysisMetrics;
  analyzed_at: string;
}

interface AnalysisRequest {
  text: string;
  content_id?: string;
  options?: {
    enable_topics?: boolean;
    enable_sentiment?: boolean;
    enable_key_info?: boolean;
    enable_classification?: boolean;
    priority?: "Low" | "Normal" | "High" | "Urgent";
  };
}

interface RealTimeAnalysisStatus {
  analysis_id: string;
  status: "Queued" | "Processing" | "Completed" | "Failed" | "Cancelled";
  progress: number;
  current_stage:
    | "Initialization"
    | "TopicAnalysis"
    | "SentimentAnalysis"
    | "KeyInfoExtraction"
    | "Classification"
    | "Finalization";
  estimated_remaining_seconds?: number;
  completed_analyses: string[];
}

interface UseContentAnalysisOptions {
  autoStart?: boolean;
  enableRealTimeUpdates?: boolean;
  onProgress?: (status: RealTimeAnalysisStatus) => void;
  onComplete?: (result: ContentAnalysisResult) => void;
  onError?: (error: string) => void;
}

interface UseContentAnalysisReturn {
  // 分析状态
  isAnalyzing: boolean;
  analysisResult: ContentAnalysisResult | null;
  error: string | null;
  progress: number;
  currentStage: string;

  // 分析操作
  analyzeContent: (
    request: AnalysisRequest,
  ) => Promise<ContentAnalysisResult | null>;
  startRealtimeAnalysis: (text: string) => Promise<string | null>;
  cancelAnalysis: (analysisId: string) => Promise<void>;

  // 快速分析操作
  quickTopicAnalysis: (text: string) => Promise<TopicTag[] | null>;
  quickSentimentAnalysis: (text: string) => Promise<SentimentAnalysis | null>;
  quickKeyInfoExtraction: (text: string) => Promise<KeyInformation | null>;

  // 配置和统计
  updateConfig: (config: any) => Promise<void>;
  getPerformanceStats: () => Promise<any>;
  clearCache: () => Promise<void>;
}

export function useContentAnalysis(
  options: UseContentAnalysisOptions = {},
): UseContentAnalysisReturn {
  const {
    autoStart = false,
    enableRealTimeUpdates = true,
    onProgress,
    onComplete,
    onError,
  } = options;

  // 状态管理
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [analysisResult, setAnalysisResult] =
    useState<ContentAnalysisResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);
  const [currentStage, setCurrentStage] = useState("");
  const [currentAnalysisId, setCurrentAnalysisId] = useState<string | null>(
    null,
  );

  // 监听实时分析状态更新
  useEffect(() => {
    if (!enableRealTimeUpdates) return;

    const setupListeners = async () => {
      const unlistenStatus = await listen(
        "realtime_analysis_status",
        (event: any) => {
          const status: RealTimeAnalysisStatus = event.payload;

          if (currentAnalysisId && status.analysis_id === currentAnalysisId) {
            setProgress(status.progress);
            setCurrentStage(status.current_stage);
            onProgress?.(status);

            if (status.status === "Failed") {
              setIsAnalyzing(false);
              setError("分析失败");
            } else if (status.status === "Cancelled") {
              setIsAnalyzing(false);
              setError("分析已取消");
            }
          }
        },
      );

      const unlistenCompleted = await listen(
        "realtime_analysis_completed",
        (event: any) => {
          const result: ContentAnalysisResult = event.payload;
          setAnalysisResult(result);
          setIsAnalyzing(false);
          setProgress(100);
          setCurrentStage("已完成");
          onComplete?.(result);
        },
      );

      const unlistenError = await listen(
        "realtime_analysis_error",
        (event: any) => {
          const errorMessage: string = event.payload;
          setError(errorMessage);
          setIsAnalyzing(false);
          setProgress(0);
          onError?.(errorMessage);
        },
      );

      return () => {
        unlistenStatus();
        unlistenCompleted();
        unlistenError();
      };
    };

    setupListeners();
  }, [
    currentAnalysisId,
    enableRealTimeUpdates,
    onProgress,
    onComplete,
    onError,
  ]);

  // 分析内容
  const analyzeContent = useCallback(
    async (request: AnalysisRequest): Promise<ContentAnalysisResult | null> => {
      try {
        setIsAnalyzing(true);
        setError(null);
        setProgress(0);

        const result = (await invoke("analyze_content", {
          request,
        })) as ContentAnalysisResult;

        setAnalysisResult(result);
        setProgress(100);
        setCurrentStage("已完成");
        onComplete?.(result);

        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        onError?.(errorMessage);
        return null;
      } finally {
        setIsAnalyzing(false);
      }
    },
    [onComplete, onError],
  );

  // 开始实时分析
  const startRealtimeAnalysis = useCallback(
    async (text: string): Promise<string | null> => {
      try {
        setIsAnalyzing(true);
        setError(null);
        setProgress(0);
        setCurrentStage("初始化中");

        const analysisId = (await invoke("start_realtime_analysis", {
          text,
        })) as string;
        setCurrentAnalysisId(analysisId);

        return analysisId;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        setIsAnalyzing(false);
        onError?.(errorMessage);
        return null;
      }
    },
    [onError],
  );

  // 取消分析
  const cancelAnalysis = useCallback(
    async (analysisId: string): Promise<void> => {
      try {
        await invoke("cancel_realtime_analysis", { analysisId });
        setIsAnalyzing(false);
        setCurrentAnalysisId(null);
        setProgress(0);
        setCurrentStage("已取消");
      } catch (err) {
        console.error("取消分析失败:", err);
      }
    },
    [],
  );

  // 快速主题分析
  const quickTopicAnalysis = useCallback(
    async (text: string): Promise<TopicTag[] | null> => {
      try {
        const topics = (await invoke("quick_topic_identification", {
          text,
        })) as TopicTag[];
        return topics;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        onError?.(errorMessage);
        return null;
      }
    },
    [onError],
  );

  // 快速情感分析
  const quickSentimentAnalysis = useCallback(
    async (text: string): Promise<SentimentAnalysis | null> => {
      try {
        const sentiment = (await invoke("quick_sentiment_analysis", {
          text,
        })) as SentimentAnalysis;
        return sentiment;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        onError?.(errorMessage);
        return null;
      }
    },
    [onError],
  );

  // 快速关键信息提取
  const quickKeyInfoExtraction = useCallback(
    async (text: string): Promise<KeyInformation | null> => {
      try {
        const keyInfo = (await invoke("quick_key_info_extraction", {
          text,
        })) as KeyInformation;
        return keyInfo;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        onError?.(errorMessage);
        return null;
      }
    },
    [onError],
  );

  // 更新配置
  const updateConfig = useCallback(
    async (config: any): Promise<void> => {
      try {
        await invoke("update_analysis_config", { request: config });
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        onError?.(errorMessage);
      }
    },
    [onError],
  );

  // 获取性能统计
  const getPerformanceStats = useCallback(async (): Promise<any> => {
    try {
      return await invoke("get_analysis_performance_stats");
    } catch (err) {
      console.error("获取性能统计失败:", err);
      return null;
    }
  }, []);

  // 清除缓存
  const clearCache = useCallback(async (): Promise<void> => {
    try {
      await invoke("clear_analysis_cache");
    } catch (err) {
      console.error("清除缓存失败:", err);
    }
  }, []);

  return {
    // 状态
    isAnalyzing,
    analysisResult,
    error,
    progress,
    currentStage,

    // 操作
    analyzeContent,
    startRealtimeAnalysis,
    cancelAnalysis,

    // 快速分析
    quickTopicAnalysis,
    quickSentimentAnalysis,
    quickKeyInfoExtraction,

    // 配置和统计
    updateConfig,
    getPerformanceStats,
    clearCache,
  };
}

// 批量分析hook
export function useBatchContentAnalysis() {
  const [isBatchAnalyzing, setIsBatchAnalyzing] = useState(false);
  const [batchProgress, setBatchProgress] = useState(0);
  const [batchResults, setBatchResults] = useState<any>(null);
  const [batchError, setBatchError] = useState<string | null>(null);

  // 监听批量分析进度
  useEffect(() => {
    const setupBatchListeners = async () => {
      const unlistenProgress = await listen(
        "batch_analysis_progress",
        (event: any) => {
          setBatchProgress(event.payload);
        },
      );

      const unlistenCompleted = await listen(
        "batch_analysis_completed",
        (event: any) => {
          setBatchResults(event.payload);
          setIsBatchAnalyzing(false);
        },
      );

      return () => {
        unlistenProgress();
        unlistenCompleted();
      };
    };

    setupBatchListeners();
  }, []);

  const startBatchAnalysis = useCallback(
    async (requests: AnalysisRequest[]): Promise<any> => {
      try {
        setIsBatchAnalyzing(true);
        setBatchError(null);
        setBatchProgress(0);

        const batchRequest = {
          items: requests,
          batch_options: {
            max_concurrent: 3,
            continue_on_error: true,
            progress_interval: 1,
          },
        };

        const result = await invoke("batch_analyze_content", {
          request: batchRequest,
        });
        setBatchResults(result);

        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setBatchError(errorMessage);
        return null;
      } finally {
        setIsBatchAnalyzing(false);
      }
    },
    [],
  );

  return {
    isBatchAnalyzing,
    batchProgress,
    batchResults,
    batchError,
    startBatchAnalysis,
  };
}
