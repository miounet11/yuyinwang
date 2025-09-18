import React, { useState, useEffect } from "react";
import { useStore } from "../App";
import "./MainLayout.css";

// 导入生产级核心组件
import ProductionRecordingControls from "./ProductionRecordingControls";
import ProductionTranscriptionDisplay from "./ProductionTranscriptionDisplay";
import EnhancedHistoryPage from "./EnhancedHistoryPage";
import PermissionManager from "./PermissionManager";
import RecordingStatusIndicator from "./RecordingStatusIndicator";
import NetworkStatusIndicator from "./NetworkStatusIndicator";

// 导入设置相关组件
import TranscriptionModelsPage from "./TranscriptionModelsPage";
import VoiceShortcutSettings from "./VoiceShortcutSettings";
import TextInjectionSettings from "./TextInjectionSettings";
import TranscriptionModeSettings from "./TranscriptionModeSettings";

interface MainLayoutProps {
  children?: React.ReactNode;
}

const MainLayout: React.FC<MainLayoutProps> = ({ children }) => {
  const {
    currentPage,
    setCurrentPage,
    isRecording,
    transcriptionText,
    hasAllPermissions,
    showPermissionModal,
  } = useStore();

  // 侧边栏导航配置
  const navigationItems = [
    {
      id: "recording",
      name: "录制",
      icon: "🎙️",
      description: "语音录制和转录",
    },
    {
      id: "history",
      name: "历史",
      icon: "📝",
      description: "转录历史和管理",
    },
    {
      id: "models",
      name: "模型",
      icon: "🤖",
      description: "转录模型配置",
    },
    {
      id: "shortcuts",
      name: "快捷键",
      icon: "⌨️",
      description: "语音快捷键设置",
    },
    {
      id: "injection",
      name: "文本注入",
      icon: "📋",
      description: "智能文本注入",
    },
    {
      id: "transcription-mode",
      name: "转录模式",
      icon: "⚙️",
      description: "转录模式设置",
    },
  ];

  // 获取当前页面标题和描述
  const getCurrentPageInfo = () => {
    const current = navigationItems.find((item) => item.id === currentPage);
    return (
      current || { name: "录制", description: "语音录制和转录", icon: "🎙️" }
    );
  };

  const pageInfo = getCurrentPageInfo();

  // 渲染主内容区域
  const renderMainContent = () => {
    if (!hasAllPermissions && showPermissionModal) {
      return <PermissionManager />;
    }

    switch (currentPage) {
      case "recording":
        return (
          <div className="recording-main-content">
            <div className="recording-section">
              <ProductionRecordingControls />
            </div>
            <div className="transcription-section">
              <ProductionTranscriptionDisplay
                text={transcriptionText}
                isRealtime={isRecording}
                showTimestamps={true}
                language="zh-CN"
                onTextEdit={(newText) => {
                  // 处理文本编辑
                  console.log("文本已编辑:", newText);
                }}
                onExport={(format) => {
                  // 处理导出
                  console.log("导出格式:", format);
                }}
              />
            </div>
          </div>
        );

      case "history":
        return <EnhancedHistoryPage />;

      case "models":
        return <TranscriptionModelsPage />;

      case "shortcuts":
        return <VoiceShortcutSettings />;

      case "injection":
        return <TextInjectionSettings />;

      case "transcription-mode":
        return <TranscriptionModeSettings />;

      default:
        return (
          <div className="recording-main-content">
            <div className="recording-section">
              <ProductionRecordingControls />
            </div>
            <div className="transcription-section">
              <ProductionTranscriptionDisplay
                text={transcriptionText}
                isRealtime={isRecording}
                showTimestamps={true}
                language="zh-CN"
                onTextEdit={(newText) => {
                  // 处理文本编辑
                  console.log("文本已编辑:", newText);
                }}
                onExport={(format) => {
                  // 处理导出
                  console.log("导出格式:", format);
                }}
              />
            </div>
          </div>
        );
    }
  };

  return (
    <div className="main-layout">
      {/* 顶部标题栏 */}
      <header className="main-header">
        <div className="header-left">
          <div className="app-logo">
            <span className="logo-icon">🎙️</span>
            <span className="app-name">Recording King</span>
          </div>
        </div>

        <div className="header-center">
          <h1 className="page-title">
            <span className="page-icon">{pageInfo.icon}</span>
            {pageInfo.name}
          </h1>
          <p className="page-description">{pageInfo.description}</p>
        </div>

        <div className="header-right">
          <NetworkStatusIndicator />
          {isRecording && (
            <div className="recording-indicator pulse">
              <span className="recording-dot"></span>
              录制中
            </div>
          )}
        </div>
      </header>

      <div className="main-container">
        {/* 左侧导航栏 */}
        <nav className="sidebar">
          <div className="nav-items">
            {navigationItems.map((item) => (
              <button
                key={item.id}
                className={`nav-item ${currentPage === item.id ? "active" : ""}`}
                onClick={() => setCurrentPage(item.id)}
                title={item.description}
              >
                <span className="nav-icon">{item.icon}</span>
                <span className="nav-label">{item.name}</span>
                {item.id === "recording" && isRecording && (
                  <span className="recording-badge"></span>
                )}
              </button>
            ))}
          </div>

          {/* 侧边栏底部信息 */}
          <div className="sidebar-footer">
            <div className="version-info">
              <span>v3.4.3</span>
            </div>
          </div>
        </nav>

        {/* 主内容区域 */}
        <main className="main-content">
          <div className="content-wrapper">{renderMainContent()}</div>
        </main>
      </div>

      {/* 权限提示模态框 */}
      {showPermissionModal && !hasAllPermissions && (
        <div className="permission-overlay">
          <PermissionManager />
        </div>
      )}
    </div>
  );
};

export default MainLayout;
