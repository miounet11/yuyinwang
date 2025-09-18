import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { unregisterAll } from "@tauri-apps/api/globalShortcut";
import "./App.css";
import "./styles/micro-interactions.css";
import logger from "./utils/logger";

// 导入主布局和全局状态
import MainLayout from "./components/MainLayout";
import { useStore } from "./App";
import {
  InteractionFeedbackProvider,
} from "./components/feedback/InteractionFeedbackProvider";
import ErrorBoundary from "./components/ErrorBoundary";

// 导入必要的组件
import FirstLaunchWizard from "./components/FirstLaunchWizard";
import PermissionManager from "./components/PermissionManager";
import FloatingDialog from "./components/FloatingDialog";

declare global {
  interface Window {
    appToggleRecording?: () => Promise<void>;
  }
}

const ProductionApp: React.FC = () => {
  const {
    isRecording,
    transcriptionText,
    setRecording,
    setTranscription,
    setDevices,
    setTranscriptionHistory,
    addTranscriptionEntry,
    showFloatingDialog,
    setShowFloatingDialog,
    hasAllPermissions,
    setHasAllPermissions,
    showPermissionModal,
    setShowPermissionModal,
    permissionIssueDetected,
    setPermissionIssueDetected,
  } = useStore();

  const [showFirstLaunchWizard, setShowFirstLaunchWizard] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);

  // 初始化应用
  useEffect(() => {
    const initializeApp = async () => {
      try {
        logger.info("初始化生产级 Recording King 应用");

        // 检查权限状态
        await checkPermissions();

        // 获取音频设备
        await loadAudioDevices();

        // 加载转录历史
        await loadTranscriptionHistory();

        // 检查首次启动
        checkFirstLaunch();

        setIsInitialized(true);
        logger.info("应用初始化完成");
      } catch (error) {
        logger.error("应用初始化失败:", error);
      }
    };

    initializeApp();
  }, []);

  // 检查权限
  const checkPermissions = async () => {
    try {
      const permissions = await invoke<{
        microphone: boolean;
        accessibility: boolean;
      }>("check_all_permissions");

      const hasAll = permissions.microphone && permissions.accessibility;
      setHasAllPermissions(hasAll);

      if (!hasAll) {
        setShowPermissionModal(true);
        setPermissionIssueDetected(true);
      }

      logger.info("权限检查完成:", permissions);
    } catch (error) {
      logger.error("权限检查失败:", error);
      setHasAllPermissions(false);
      setShowPermissionModal(true);
    }
  };

  // 加载音频设备
  const loadAudioDevices = async () => {
    try {
      const devices = await invoke<any[]>("get_audio_devices");
      setDevices(devices);
      logger.info(`加载了 ${devices.length} 个音频设备`);
    } catch (error) {
      logger.error("加载音频设备失败:", error);
    }
  };

  // 加载转录历史
  const loadTranscriptionHistory = async () => {
    try {
      const history = await invoke<any[]>("get_transcription_history");
      setTranscriptionHistory(history);
      logger.info(`加载了 ${history.length} 条历史记录`);
    } catch (error) {
      logger.error("加载转录历史失败:", error);
    }
  };

  // 检查首次启动
  const checkFirstLaunch = () => {
    const hasCompletedSetup = localStorage.getItem('recording_king_setup_completed');
    const hasSeenWizard = localStorage.getItem('recording_king_wizard_seen');

    if (!hasCompletedSetup && !hasSeenWizard) {
      logger.info("首次启动，显示设置向导");
      localStorage.setItem("recording_king_wizard_seen", "true");
      setTimeout(() => {
        setShowFirstLaunchWizard(true);
      }, 1500);
    }
  };

  // 设置事件监听器
  useEffect(() => {
    const setupListeners = async () => {
      try {
        // 监听转录文本更新
        const unlisten1 = await listen<{ text: string }>(
          "transcription_update",
          (event) => {
            logger.debug("收到转录更新:", event.payload.text);
            setTranscription(event.payload.text);
          }
        );

        // 监听录制状态变化
        const unlisten2 = await listen<{ is_recording: boolean }>(
          "recording_state_changed",
          (event) => {
            logger.debug("录制状态变化:", event.payload.is_recording);
            setRecording(event.payload.is_recording);
          }
        );

        // 监听转录完成
        const unlisten3 = await listen<any>(
          "transcription_completed",
          (event) => {
            logger.info("转录完成:", event.payload);
            addTranscriptionEntry(event.payload);
          }
        );

        // 监听权限变化
        const unlisten4 = await listen<{ has_permissions: boolean }>(
          "permissions_changed",
          (event) => {
            logger.info("权限状态变化:", event.payload.has_permissions);
            setHasAllPermissions(event.payload.has_permissions);
            setShowPermissionModal(!event.payload.has_permissions);
          }
        );

        // 返回清理函数
        return () => {
          unlisten1();
          unlisten2();
          unlisten3();
          unlisten4();
          unregisterAll();
        };
      } catch (error) {
        logger.error("设置监听器失败:", error);
      }
    };

    if (isInitialized) {
      setupListeners();
    }
  }, [isInitialized]);

  // 设置全局录制切换函数
  useEffect(() => {
    window.appToggleRecording = async () => {
      try {
        if (!hasAllPermissions) {
          logger.warn("权限不足，无法切换录制状态");
          setShowPermissionModal(true);
          return;
        }

        if (isRecording) {
          await invoke("stop_recording");
          setRecording(false);
          logger.info("通过全局快捷键停止录制");
        } else {
          await invoke("start_recording");
          setRecording(true);
          logger.info("通过全局快捷键开始录制");
        }
      } catch (error) {
        logger.error("全局录制切换失败:", error);
      }
    };

    return () => {
      delete window.appToggleRecording;
    };
  }, [isRecording, hasAllPermissions]);

  // 处理权限变化
  const handlePermissionChange = (hasAll: boolean) => {
    setHasAllPermissions(hasAll);
    setShowPermissionModal(!hasAll);
    setPermissionIssueDetected(!hasAll);

    if (hasAll) {
      logger.info("所有权限已授予");
    } else {
      logger.warn("权限不足");
    }
  };

  // 处理向导完成
  const handleWizardComplete = () => {
    setShowFirstLaunchWizard(false);
    localStorage.setItem("recording_king_setup_completed", "true");
    logger.info("设置向导完成");
  };

  // 如果未初始化，显示加载状态
  if (!isInitialized) {
    return (
      <div className="app-loading">
        <div className="loading-spinner">
          <div className="spinner-icon">🎙️</div>
          <div className="loading-text">初始化 Recording King...</div>
        </div>
      </div>
    );
  }

  return (
    <ErrorBoundary componentName="ProductionApp">
      <InteractionFeedbackProvider
        performanceTarget={100}
        enableRipple={true}
        enableHover={true}
        enableMetrics={true}
        onPerformanceWarning={(metrics) => {
          if (metrics.responseTime > 200) {
            logger.warn("UI性能警告:", metrics);
          }
        }}
      >
        <div className="production-app">
          {/* 主界面布局 */}
          <MainLayout />

          {/* 悬浮对话框 */}
          {showFloatingDialog && (
            <FloatingDialog
              onClose={() => setShowFloatingDialog(false)}
              transcriptionText={transcriptionText}
              isRecording={isRecording}
            />
          )}

          {/* 首次启动向导 */}
          {showFirstLaunchWizard && (
            <div className="wizard-overlay">
              <FirstLaunchWizard onComplete={handleWizardComplete} />
            </div>
          )}

          {/* 权限管理模态框 */}
          {showPermissionModal && !hasAllPermissions && (
            <div className="permission-modal-overlay">
              <div className="permission-modal">
                <PermissionManager onPermissionChange={handlePermissionChange} />
              </div>
            </div>
          )}
        </div>
      </InteractionFeedbackProvider>
    </ErrorBoundary>
  );
};

export default ProductionApp;
