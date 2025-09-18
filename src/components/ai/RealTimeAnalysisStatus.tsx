import React, { useState, useEffect } from "react";
import ErrorBoundary from "../ErrorBoundary";

// 调试模式
const DEBUG_MODE = true;

const debugLog = (message: string, data?: any) => {
  if (DEBUG_MODE) {
    console.log(`[RealTimeAnalysisStatus] ${message}`, data || "");
  }
};

// Props接口定义
interface RealTimeAnalysisStatusProps {
  status: AnalysisStatus | null;
  isActive: boolean;
}

// 类型定义
interface AnalysisStatus {
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
  error_message?: string;
}

interface PerformanceStats {
  total_analyses: number;
  average_analysis_time_ms: number;
  cache_hit_rate: number;
  successful_analyses: number;
  failed_analyses: number;
  last_analysis_time?: string;
}

export const RealTimeAnalysisStatus: React.FC<RealTimeAnalysisStatusProps> = ({
  status,
  isActive,
}) => {
  debugLog("组件初始化");

  const [currentStatus, setCurrentStatus] = useState<AnalysisStatus>({
    analysis_id: "",
    status: "Queued",
    progress: 0,
    current_stage: "Initialization",
    completed_analyses: [],
  });

  const [performanceStats, setPerformanceStats] = useState<PerformanceStats>({
    total_analyses: 0,
    average_analysis_time_ms: 0,
    cache_hit_rate: 0,
    successful_analyses: 0,
    failed_analyses: 0,
  });

  const [isConnected, setIsConnected] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  // 模拟实时状态更新
  useEffect(() => {
    debugLog("开始模拟状态更新");

    const interval = setInterval(() => {
      setPerformanceStats((prev) => ({
        ...prev,
        total_analyses: prev.total_analyses + Math.floor(Math.random() * 2),
        average_analysis_time_ms: 1200 + Math.random() * 800,
        cache_hit_rate: 0.65 + Math.random() * 0.3,
        successful_analyses:
          prev.successful_analyses + Math.floor(Math.random() * 2),
        last_analysis_time: new Date().toISOString(),
      }));

      setLastUpdate(new Date());
      setIsConnected(true);
    }, 3000);

    return () => {
      clearInterval(interval);
      debugLog("清理状态更新定时器");
    };
  }, []);

  // 模拟分析状态变化
  const simulateAnalysis = () => {
    const analysisId = `analysis_${Date.now()}`;
    debugLog("开始模拟分析:", analysisId);

    setCurrentStatus({
      analysis_id: analysisId,
      status: "Processing",
      progress: 0,
      current_stage: "Initialization",
      completed_analyses: [],
    });

    const stages: AnalysisStatus["current_stage"][] = [
      "Initialization",
      "TopicAnalysis",
      "SentimentAnalysis",
      "KeyInfoExtraction",
      "Classification",
      "Finalization",
    ];

    let currentStageIndex = 0;
    let progress = 0;

    const progressInterval = setInterval(() => {
      progress += Math.random() * 20;

      if (progress >= 100) {
        progress = 100;
        setCurrentStatus((prev) => ({
          ...prev,
          status: "Completed",
          progress: 100,
          current_stage: "Finalization",
          completed_analyses: [
            "topics",
            "sentiment",
            "keyinfo",
            "classification",
          ],
        }));
        clearInterval(progressInterval);
        debugLog("分析完成:", analysisId);
        return;
      }

      const stageProgress = Math.floor(progress / (100 / stages.length));
      if (stageProgress > currentStageIndex && stageProgress < stages.length) {
        currentStageIndex = stageProgress;
      }

      setCurrentStatus((prev) => ({
        ...prev,
        progress: Math.floor(progress),
        current_stage: stages[currentStageIndex],
        estimated_remaining_seconds: Math.floor((100 - progress) / 10),
      }));
    }, 500);
  };

  const getStatusColor = (status: AnalysisStatus["status"]) => {
    switch (status) {
      case "Queued":
        return "#718096";
      case "Processing":
        return "#3182ce";
      case "Completed":
        return "#38a169";
      case "Failed":
        return "#e53e3e";
      case "Cancelled":
        return "#d69e2e";
      default:
        return "#718096";
    }
  };

  const getStatusIcon = (status: AnalysisStatus["status"]) => {
    switch (status) {
      case "Queued":
        return "⏳";
      case "Processing":
        return "⚡";
      case "Completed":
        return "✅";
      case "Failed":
        return "❌";
      case "Cancelled":
        return "⚠️";
      default:
        return "❓";
    }
  };

  const getStageDescription = (stage: AnalysisStatus["current_stage"]) => {
    switch (stage) {
      case "Initialization":
        return "初始化分析环境";
      case "TopicAnalysis":
        return "识别文本主题";
      case "SentimentAnalysis":
        return "分析情感倾向";
      case "KeyInfoExtraction":
        return "提取关键信息";
      case "Classification":
        return "内容分类处理";
      case "Finalization":
        return "完成分析并整理结果";
      default:
        return "未知阶段";
    }
  };

  return (
    <ErrorBoundary componentName="RealTimeAnalysisStatus">
      <div
        style={{
          padding: "20px",
          border: "1px solid #e2e8f0",
          borderRadius: "8px",
          margin: "10px 0",
          backgroundColor: "#f7fafc",
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: "20px",
          }}
        >
          <h3 style={{ margin: 0, color: "#2d3748" }}>📊 实时分析状态</h3>
          <div style={{ display: "flex", gap: "10px" }}>
            <button
              onClick={simulateAnalysis}
              style={{
                padding: "6px 12px",
                backgroundColor: "#3182ce",
                color: "white",
                border: "none",
                borderRadius: "4px",
                cursor: "pointer",
                fontSize: "12px",
              }}
            >
              🚀 模拟分析
            </button>
            <div
              style={{
                padding: "4px 8px",
                backgroundColor: isConnected ? "#c6f6d5" : "#fed7d7",
                color: isConnected ? "#22543d" : "#c53030",
                borderRadius: "12px",
                fontSize: "12px",
                fontWeight: "bold",
              }}
            >
              {isConnected ? "🟢 已连接" : "🔴 未连接"}
            </div>
          </div>
        </div>

        {DEBUG_MODE && (
          <div
            style={{
              fontSize: "12px",
              color: "#718096",
              backgroundColor: "#edf2f7",
              padding: "8px",
              borderRadius: "4px",
              marginBottom: "15px",
            }}
          >
            🐛 调试信息 | 状态: {currentStatus.status} | 阶段:{" "}
            {currentStatus.current_stage} | 最后更新:{" "}
            {lastUpdate?.toLocaleTimeString() || "未更新"}
          </div>
        )}

        {/* 当前分析状态 */}
        <div style={{ marginBottom: "20px" }}>
          <h4 style={{ margin: "0 0 10px 0", color: "#4a5568" }}>当前分析</h4>
          <div
            style={{
              padding: "15px",
              backgroundColor: "white",
              borderRadius: "6px",
              border: "1px solid #e2e8f0",
            }}
          >
            {currentStatus.analysis_id ? (
              <>
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "8px",
                    marginBottom: "10px",
                  }}
                >
                  <span style={{ fontSize: "20px" }}>
                    {getStatusIcon(currentStatus.status)}
                  </span>
                  <span
                    style={{
                      fontWeight: "bold",
                      color: getStatusColor(currentStatus.status),
                    }}
                  >
                    {currentStatus.status}
                  </span>
                  <span style={{ fontSize: "12px", color: "#718096" }}>
                    (ID: {currentStatus.analysis_id.slice(-8)})
                  </span>
                </div>

                <div style={{ marginBottom: "10px" }}>
                  <div
                    style={{
                      display: "flex",
                      justifyContent: "space-between",
                      marginBottom: "5px",
                    }}
                  >
                    <span style={{ fontSize: "14px", color: "#4a5568" }}>
                      {getStageDescription(currentStatus.current_stage)}
                    </span>
                    <span
                      style={{
                        fontSize: "14px",
                        fontWeight: "bold",
                        color: "#2d3748",
                      }}
                    >
                      {currentStatus.progress}%
                    </span>
                  </div>
                  <div
                    style={{
                      width: "100%",
                      height: "8px",
                      backgroundColor: "#edf2f7",
                      borderRadius: "4px",
                      overflow: "hidden",
                    }}
                  >
                    <div
                      style={{
                        width: `${currentStatus.progress}%`,
                        height: "100%",
                        backgroundColor: getStatusColor(currentStatus.status),
                        transition: "width 0.3s ease",
                      }}
                    />
                  </div>
                </div>

                {currentStatus.estimated_remaining_seconds && (
                  <div style={{ fontSize: "12px", color: "#718096" }}>
                    预计剩余时间: {currentStatus.estimated_remaining_seconds} 秒
                  </div>
                )}

                {currentStatus.completed_analyses.length > 0 && (
                  <div style={{ marginTop: "10px" }}>
                    <span
                      style={{
                        fontSize: "12px",
                        color: "#4a5568",
                        marginRight: "8px",
                      }}
                    >
                      已完成:
                    </span>
                    {currentStatus.completed_analyses.map((analysis, index) => (
                      <span
                        key={index}
                        style={{
                          fontSize: "11px",
                          padding: "2px 6px",
                          backgroundColor: "#c6f6d5",
                          color: "#22543d",
                          borderRadius: "8px",
                          marginRight: "4px",
                        }}
                      >
                        {analysis}
                      </span>
                    ))}
                  </div>
                )}
              </>
            ) : (
              <div
                style={{
                  textAlign: "center",
                  color: "#718096",
                  padding: "20px 0",
                }}
              >
                <div style={{ fontSize: "24px", marginBottom: "8px" }}>⏸️</div>
                <div>暂无进行中的分析任务</div>
                <div style={{ fontSize: "12px", marginTop: "4px" }}>
                  点击上方"模拟分析"按钮开始测试
                </div>
              </div>
            )}
          </div>
        </div>

        {/* 性能统计 */}
        <div>
          <h4 style={{ margin: "0 0 10px 0", color: "#4a5568" }}>系统统计</h4>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
              gap: "15px",
            }}
          >
            <div
              style={{
                padding: "12px",
                backgroundColor: "white",
                borderRadius: "6px",
                border: "1px solid #e2e8f0",
              }}
            >
              <div
                style={{
                  fontSize: "24px",
                  fontWeight: "bold",
                  color: "#3182ce",
                }}
              >
                {performanceStats.total_analyses}
              </div>
              <div style={{ fontSize: "12px", color: "#718096" }}>
                总分析次数
              </div>
            </div>

            <div
              style={{
                padding: "12px",
                backgroundColor: "white",
                borderRadius: "6px",
                border: "1px solid #e2e8f0",
              }}
            >
              <div
                style={{
                  fontSize: "24px",
                  fontWeight: "bold",
                  color: "#38a169",
                }}
              >
                {Math.round(performanceStats.average_analysis_time_ms)}ms
              </div>
              <div style={{ fontSize: "12px", color: "#718096" }}>
                平均处理时间
              </div>
            </div>

            <div
              style={{
                padding: "12px",
                backgroundColor: "white",
                borderRadius: "6px",
                border: "1px solid #e2e8f0",
              }}
            >
              <div
                style={{
                  fontSize: "24px",
                  fontWeight: "bold",
                  color: "#d69e2e",
                }}
              >
                {Math.round(performanceStats.cache_hit_rate * 100)}%
              </div>
              <div style={{ fontSize: "12px", color: "#718096" }}>
                缓存命中率
              </div>
            </div>

            <div
              style={{
                padding: "12px",
                backgroundColor: "white",
                borderRadius: "6px",
                border: "1px solid #e2e8f0",
              }}
            >
              <div
                style={{
                  fontSize: "24px",
                  fontWeight: "bold",
                  color: "#805ad5",
                }}
              >
                {performanceStats.successful_analyses}/
                {performanceStats.failed_analyses}
              </div>
              <div style={{ fontSize: "12px", color: "#718096" }}>
                成功/失败
              </div>
            </div>
          </div>

          {performanceStats.last_analysis_time && (
            <div
              style={{
                marginTop: "10px",
                fontSize: "12px",
                color: "#718096",
                textAlign: "center",
              }}
            >
              最后分析时间:{" "}
              {new Date(performanceStats.last_analysis_time).toLocaleString()}
            </div>
          )}
        </div>
      </div>
    </ErrorBoundary>
  );
};

export default RealTimeAnalysisStatus;
