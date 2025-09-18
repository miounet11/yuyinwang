import React, { useState, useEffect, useCallback, useMemo } from "react";
import { useContentAnalysis } from "../../hooks/useContentAnalysis";
import { RealTimeAnalysisStatus } from "./RealTimeAnalysisStatus";
import ErrorBoundary from "../ErrorBoundary";

// 类型定义
export interface ContentAnalysisResult {
  id: string;
  content: string;
  analysis: {
    topics: string[];
    sentiment: "positive" | "negative" | "neutral";
    keyInformation: string[];
    confidence: number;
    aiModel: string;
    timestamp: string;
  };
  performance?: {
    analysisTime: number;
    cacheHit: boolean;
    modelTokens: number;
  };
}

export interface BatchAnalysisProgress {
  total: number;
  completed: number;
  failed: number;
  currentItem?: string;
  estimatedTimeRemaining?: number;
}

// 调试配置接口
interface DebugConfig {
  enabled: boolean;
  verbose: boolean;
  logLevel: "info" | "debug" | "error";
  simulateErrors: boolean;
}

// 主组件
export const SmartContentAnalysisPanel: React.FC = () => {
  // Hooks
  const {
    analyzeContent,
    batchAnalyzeContent,
    startRealTimeAnalysis,
    stopRealTimeAnalysis,
    isAnalyzing,
    realTimeStatus,
    error: analysisError,
    clearError,
  } = useContentAnalysis();

  // 状态管理
  const [inputContent, setInputContent] = useState<string>("");
  const [analysisResults, setAnalysisResults] = useState<
    ContentAnalysisResult[]
  >([]);
  const [batchFiles, setBatchFiles] = useState<File[]>([]);
  const [batchProgress, setBatchProgress] =
    useState<BatchAnalysisProgress | null>(null);
  const [isRealTimeActive, setIsRealTimeActive] = useState<boolean>(false);
  const [activeTab, setActiveTab] = useState<"single" | "batch" | "realtime">(
    "single",
  );
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // 调试状态
  const [debugConfig, setDebugConfig] = useState<DebugConfig>({
    enabled: process.env.NODE_ENV === "development",
    verbose: false,
    logLevel: "info",
    simulateErrors: false,
  });

  // 调试日志函数
  const debugLog = useCallback(
    (
      message: string,
      level: "info" | "debug" | "error" = "info",
      data?: any,
    ) => {
      if (!debugConfig.enabled) return;

      const timestamp = new Date().toISOString();
      const logMessage = `[SmartContentAnalysis] ${timestamp} [${level.toUpperCase()}] ${message}`;

      if (debugConfig.verbose && data) {
        console.log(logMessage, data);
      } else {
        console.log(logMessage);
      }
    },
    [debugConfig],
  );

  // 错误处理
  const handleError = useCallback(
    (error: Error | string, context?: string) => {
      const errorMessage = typeof error === "string" ? error : error.message;
      const fullError = context ? `${context}: ${errorMessage}` : errorMessage;

      debugLog(`错误发生: ${fullError}`, "error");
      setError(fullError);

      // 自动清除成功消息
      setSuccessMessage(null);
    },
    [debugLog],
  );

  // 成功处理
  const handleSuccess = useCallback(
    (message: string) => {
      debugLog(`操作成功: ${message}`, "info");
      setSuccessMessage(message);
      setError(null);

      // 5秒后自动清除成功消息
      setTimeout(() => setSuccessMessage(null), 5000);
    },
    [debugLog],
  );

  // 清除所有消息
  const clearMessages = useCallback(() => {
    setError(null);
    setSuccessMessage(null);
    clearError();
  }, [clearError]);

  // 单个内容分析
  const handleSingleAnalysis = useCallback(async () => {
    if (!inputContent.trim()) {
      handleError("请输入要分析的内容");
      return;
    }

    debugLog("开始单个内容分析");
    clearMessages();

    try {
      if (debugConfig.simulateErrors) {
        throw new Error("模拟错误: 分析服务暂时不可用");
      }

      const result = await analyzeContent(inputContent);

      const analysisResult: ContentAnalysisResult = {
        id: `single_${Date.now()}`,
        content: inputContent,
        analysis: {
          topics: result.topics || [],
          sentiment: result.sentiment || "neutral",
          keyInformation: result.keyInformation || [],
          confidence: result.confidence || 0,
          aiModel: result.aiModel || "gpt-3.5-turbo",
          timestamp: new Date().toISOString(),
        },
        performance: result.performance,
      };

      setAnalysisResults((prev) => [analysisResult, ...prev]);
      handleSuccess("内容分析完成");
      debugLog("单个分析完成", "info", analysisResult);

      // 清空输入框
      setInputContent("");
    } catch (error) {
      handleError(error as Error, "单个内容分析失败");
    }
  }, [
    inputContent,
    analyzeContent,
    handleError,
    handleSuccess,
    clearMessages,
    debugConfig,
    debugLog,
  ]);

  // 批量文件处理
  const handleFileUpload = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const files = Array.from(event.target.files || []);
      const textFiles = files.filter(
        (file) =>
          file.type.startsWith("text/") ||
          file.name.endsWith(".txt") ||
          file.name.endsWith(".md"),
      );

      if (textFiles.length !== files.length) {
        handleError("只支持文本文件格式(.txt, .md)");
      }

      setBatchFiles(textFiles);
      debugLog(`上传了 ${textFiles.length} 个文件`, "info");
    },
    [handleError, debugLog],
  );

  // 批量分析
  const handleBatchAnalysis = useCallback(async () => {
    if (batchFiles.length === 0) {
      handleError("请先上传要分析的文件");
      return;
    }

    debugLog(`开始批量分析 ${batchFiles.length} 个文件`);
    clearMessages();

    try {
      // 读取文件内容
      const fileContents: string[] = [];
      for (const file of batchFiles) {
        const content = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = (e) => resolve((e.target?.result as string) || "");
          reader.onerror = () =>
            reject(new Error(`无法读取文件: ${file.name}`));
          reader.readAsText(file);
        });
        fileContents.push(content);
      }

      // 初始化进度
      setBatchProgress({
        total: batchFiles.length,
        completed: 0,
        failed: 0,
      });

      // 执行批量分析
      const results = await batchAnalyzeContent(fileContents);

      // 处理结果
      const batchResults: ContentAnalysisResult[] = results.map(
        (result, index) => ({
          id: `batch_${Date.now()}_${index}`,
          content: fileContents[index].substring(0, 200) + "...",
          analysis: {
            topics: result.topics || [],
            sentiment: result.sentiment || "neutral",
            keyInformation: result.keyInformation || [],
            confidence: result.confidence || 0,
            aiModel: result.aiModel || "gpt-3.5-turbo",
            timestamp: new Date().toISOString(),
          },
          performance: result.performance,
        }),
      );

      setAnalysisResults((prev) => [...batchResults, ...prev]);
      setBatchProgress({
        total: batchFiles.length,
        completed: batchFiles.length,
        failed: 0,
      });

      handleSuccess(`批量分析完成: ${batchFiles.length} 个文件`);
      debugLog("批量分析完成", "info", {
        totalFiles: batchFiles.length,
        results: batchResults.length,
      });

      // 清空文件列表
      setBatchFiles([]);

      // 3秒后清除进度
      setTimeout(() => setBatchProgress(null), 3000);
    } catch (error) {
      handleError(error as Error, "批量分析失败");
      setBatchProgress(null);
    }
  }, [
    batchFiles,
    batchAnalyzeContent,
    handleError,
    handleSuccess,
    clearMessages,
    debugLog,
  ]);

  // 实时分析控制
  const handleRealTimeToggle = useCallback(async () => {
    debugLog(`切换实时分析状态: ${isRealTimeActive ? "停止" : "开始"}`);

    try {
      if (isRealTimeActive) {
        await stopRealTimeAnalysis();
        setIsRealTimeActive(false);
        handleSuccess("实时分析已停止");
      } else {
        await startRealTimeAnalysis();
        setIsRealTimeActive(true);
        handleSuccess("实时分析已启动");
      }
    } catch (error) {
      handleError(error as Error, "实时分析状态切换失败");
    }
  }, [
    isRealTimeActive,
    startRealTimeAnalysis,
    stopRealTimeAnalysis,
    handleError,
    handleSuccess,
    debugLog,
  ]);

  // 清除结果
  const clearResults = useCallback(() => {
    setAnalysisResults([]);
    handleSuccess("分析结果已清除");
    debugLog("清除所有分析结果");
  }, [handleSuccess, debugLog]);

  // 导出结果
  const exportResults = useCallback(() => {
    if (analysisResults.length === 0) {
      handleError("没有可导出的分析结果");
      return;
    }

    try {
      const exportData = {
        exportTime: new Date().toISOString(),
        totalResults: analysisResults.length,
        results: analysisResults,
      };

      const blob = new Blob([JSON.stringify(exportData, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `content_analysis_${new Date().toISOString().split("T")[0]}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      handleSuccess("分析结果已导出");
      debugLog("导出分析结果", "info", { count: analysisResults.length });
    } catch (error) {
      handleError(error as Error, "导出失败");
    }
  }, [analysisResults, handleError, handleSuccess, debugLog]);

  // 统计数据
  const statistics = useMemo(() => {
    const total = analysisResults.length;
    const sentiments = analysisResults.reduce(
      (acc, result) => {
        acc[result.analysis.sentiment] =
          (acc[result.analysis.sentiment] || 0) + 1;
        return acc;
      },
      {} as Record<string, number>,
    );

    const avgConfidence =
      total > 0
        ? analysisResults.reduce(
            (sum, result) => sum + result.analysis.confidence,
            0,
          ) / total
        : 0;

    return {
      total,
      sentiments,
      avgConfidence: Math.round(avgConfidence * 100) / 100,
    };
  }, [analysisResults]);

  // 监听分析错误
  useEffect(() => {
    if (analysisError) {
      handleError(analysisError, "AI分析服务");
    }
  }, [analysisError, handleError]);

  // 组件卸载时清理
  useEffect(() => {
    return () => {
      if (isRealTimeActive) {
        stopRealTimeAnalysis().catch(console.error);
      }
    };
  }, [isRealTimeActive, stopRealTimeAnalysis]);

  // 渲染函数
  const renderTabContent = () => {
    switch (activeTab) {
      case "single":
        return (
          <div className="analysis-tab-content">
            <div className="input-section">
              <textarea
                className="content-input"
                value={inputContent}
                onChange={(e) => setInputContent(e.target.value)}
                placeholder="请输入要分析的文本内容..."
                rows={6}
                disabled={isAnalyzing}
              />
              <div className="input-actions">
                <button
                  className="analyze-btn primary"
                  onClick={handleSingleAnalysis}
                  disabled={isAnalyzing || !inputContent.trim()}
                >
                  {isAnalyzing ? "分析中..." : "开始分析"}
                </button>
                <span className="char-count">{inputContent.length} 字符</span>
              </div>
            </div>
          </div>
        );

      case "batch":
        return (
          <div className="analysis-tab-content">
            <div className="file-upload-section">
              <input
                type="file"
                multiple
                accept=".txt,.md,text/*"
                onChange={handleFileUpload}
                className="file-input"
                disabled={isAnalyzing}
              />
              <div className="file-list">
                {batchFiles.map((file, index) => (
                  <div key={index} className="file-item">
                    <span className="file-name">{file.name}</span>
                    <span className="file-size">
                      {(file.size / 1024).toFixed(1)} KB
                    </span>
                  </div>
                ))}
              </div>
              {batchFiles.length > 0 && (
                <div className="batch-actions">
                  <button
                    className="analyze-btn primary"
                    onClick={handleBatchAnalysis}
                    disabled={isAnalyzing}
                  >
                    {isAnalyzing
                      ? "批量分析中..."
                      : `分析 ${batchFiles.length} 个文件`}
                  </button>
                  <button
                    className="clear-btn"
                    onClick={() => setBatchFiles([])}
                    disabled={isAnalyzing}
                  >
                    清除文件
                  </button>
                </div>
              )}
              {batchProgress && (
                <div className="batch-progress">
                  <div className="progress-info">
                    <span>
                      进度: {batchProgress.completed}/{batchProgress.total}
                    </span>
                    {batchProgress.failed > 0 && (
                      <span className="failed-count">
                        失败: {batchProgress.failed}
                      </span>
                    )}
                  </div>
                  <div className="progress-bar">
                    <div
                      className="progress-fill"
                      style={{
                        width: `${(batchProgress.completed / batchProgress.total) * 100}%`,
                      }}
                    />
                  </div>
                </div>
              )}
            </div>
          </div>
        );

      case "realtime":
        return (
          <div className="analysis-tab-content">
            <div className="realtime-section">
              <div className="realtime-controls">
                <button
                  className={`realtime-toggle ${isRealTimeActive ? "active" : ""}`}
                  onClick={handleRealTimeToggle}
                  disabled={isAnalyzing && !isRealTimeActive}
                >
                  {isRealTimeActive ? "停止实时分析" : "启动实时分析"}
                </button>
              </div>
              <RealTimeAnalysisStatus
                status={realTimeStatus}
                isActive={isRealTimeActive}
              />
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <ErrorBoundary>
      <div className="smart-content-analysis-panel">
        {/* 头部 */}
        <div className="panel-header">
          <h2>智能内容分析</h2>
          <div className="header-actions">
            {debugConfig.enabled && (
              <button
                className="debug-toggle"
                onClick={() =>
                  setDebugConfig((prev) => ({
                    ...prev,
                    verbose: !prev.verbose,
                  }))
                }
                title="切换调试模式"
              >
                🐛 {debugConfig.verbose ? "详细" : "简化"}
              </button>
            )}
          </div>
        </div>

        {/* 消息提示 */}
        {(error || successMessage) && (
          <div className="message-container">
            {error && (
              <div className="error-message">
                <span>❌ {error}</span>
                <button onClick={clearMessages} className="close-btn">
                  ×
                </button>
              </div>
            )}
            {successMessage && (
              <div className="success-message">
                <span>✅ {successMessage}</span>
                <button
                  onClick={() => setSuccessMessage(null)}
                  className="close-btn"
                >
                  ×
                </button>
              </div>
            )}
          </div>
        )}

        {/* 标签页导航 */}
        <div className="tab-navigation">
          <button
            className={`tab-btn ${activeTab === "single" ? "active" : ""}`}
            onClick={() => setActiveTab("single")}
          >
            单个分析
          </button>
          <button
            className={`tab-btn ${activeTab === "batch" ? "active" : ""}`}
            onClick={() => setActiveTab("batch")}
          >
            批量分析
          </button>
          <button
            className={`tab-btn ${activeTab === "realtime" ? "active" : ""}`}
            onClick={() => setActiveTab("realtime")}
          >
            实时分析
          </button>
        </div>

        {/* 标签页内容 */}
        {renderTabContent()}

        {/* 统计信息 */}
        {analysisResults.length > 0 && (
          <div className="statistics-section">
            <h3>分析统计</h3>
            <div className="stats-grid">
              <div className="stat-item">
                <span className="stat-label">总分析数</span>
                <span className="stat-value">{statistics.total}</span>
              </div>
              <div className="stat-item">
                <span className="stat-label">平均置信度</span>
                <span className="stat-value">{statistics.avgConfidence}</span>
              </div>
              <div className="stat-item">
                <span className="stat-label">正面情感</span>
                <span className="stat-value">
                  {statistics.sentiments.positive || 0}
                </span>
              </div>
              <div className="stat-item">
                <span className="stat-label">负面情感</span>
                <span className="stat-value">
                  {statistics.sentiments.negative || 0}
                </span>
              </div>
            </div>
          </div>
        )}

        {/* 结果列表 */}
        {analysisResults.length > 0 && (
          <div className="results-section">
            <div className="results-header">
              <h3>分析结果 ({analysisResults.length})</h3>
              <div className="results-actions">
                <button className="export-btn" onClick={exportResults}>
                  导出结果
                </button>
                <button className="clear-btn" onClick={clearResults}>
                  清除全部
                </button>
              </div>
            </div>
            <div className="results-list">
              {analysisResults.map((result) => (
                <div key={result.id} className="result-item">
                  <div className="result-header">
                    <span className="result-time">
                      {new Date(result.analysis.timestamp).toLocaleString()}
                    </span>
                    <span
                      className={`sentiment-badge ${result.analysis.sentiment}`}
                    >
                      {result.analysis.sentiment === "positive"
                        ? "正面"
                        : result.analysis.sentiment === "negative"
                          ? "负面"
                          : "中性"}
                    </span>
                  </div>
                  <div className="result-content">
                    <p className="content-preview">{result.content}</p>
                  </div>
                  <div className="result-analysis">
                    <div className="analysis-topics">
                      <strong>主题：</strong>
                      {result.analysis.topics.map((topic, index) => (
                        <span key={index} className="topic-tag">
                          {topic}
                        </span>
                      ))}
                    </div>
                    <div className="analysis-key-info">
                      <strong>关键信息：</strong>
                      <ul>
                        {result.analysis.keyInformation.map((info, index) => (
                          <li key={index}>{info}</li>
                        ))}
                      </ul>
                    </div>
                    <div className="analysis-meta">
                      <span>置信度: {result.analysis.confidence}</span>
                      <span>模型: {result.analysis.aiModel}</span>
                      {result.performance && (
                        <span>耗时: {result.performance.analysisTime}ms</span>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 调试信息 */}
        {debugConfig.enabled && debugConfig.verbose && (
          <div className="debug-section">
            <h4>调试信息</h4>
            <div className="debug-info">
              <p>当前标签: {activeTab}</p>
              <p>分析状态: {isAnalyzing ? "进行中" : "空闲"}</p>
              <p>实时分析: {isRealTimeActive ? "激活" : "未激活"}</p>
              <p>结果数量: {analysisResults.length}</p>
              <p>错误模拟: {debugConfig.simulateErrors ? "启用" : "禁用"}</p>
            </div>
          </div>
        )}
      </div>
    </ErrorBoundary>
  );
};

export default SmartContentAnalysisPanel;
