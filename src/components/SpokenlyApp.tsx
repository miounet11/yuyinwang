/**
 * SpokenlyApp.tsx
 * Recording King - Spokenly 界面完整集成
 * 像素级精确复刻 Spokenly 设计系统
 */

import React, { useEffect, useState } from 'react';
import MainLayout from './MainLayout';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// 导入设计系统样式
import '../styles/spokenly-design-system.css';

// 全局状态接口
interface AppState {
  initialized: boolean;
  currentPage: string;
  hasPermissions: boolean;
  showPermissionModal: boolean;
}

// 主应用组件
const SpokenlyApp: React.FC = () => {
  const [appState, setAppState] = useState<AppState>({
    initialized: false,
    currentPage: 'general',
    hasPermissions: false,
    showPermissionModal: false
  });

  // 初始化应用
  useEffect(() => {
    const initializeApp = async () => {
      try {
        // 检查权限状态
        const permissionInfo = await invoke('check_all_permissions') as {
          status: {
            all_granted: boolean;
            input_monitoring: boolean;
          }
        };

        const hasAllPermissions = permissionInfo.status.all_granted;
        const hasCriticalPermissions = permissionInfo.status.input_monitoring;

        // 设置初始化状态
        setAppState(prev => ({
          ...prev,
          initialized: true,
          hasPermissions: hasAllPermissions,
          showPermissionModal: !hasCriticalPermissions
        }));

        console.log('📱 Spokenly App 初始化完成');
        console.log('🔐 权限状态:', { hasAllPermissions, hasCriticalPermissions });

      } catch (error) {
        console.error('❌ 应用初始化失败:', error);
        
        // 即使初始化失败，也显示界面
        setAppState(prev => ({
          ...prev,
          initialized: true,
          hasPermissions: false,
          showPermissionModal: true
        }));
      }
    };

    // 设置事件监听器
    const setupEventListeners = async () => {
      try {
        // 监听全局快捷键事件
        const unlistenGlobalShortcut = await listen('global_shortcut_triggered', () => {
          console.log('🎯 全局快捷键触发');
        });

        // 监听转录结果
        const unlistenTranscription = await listen('transcription_result', (event: any) => {
          console.log('📝 收到转录结果:', event.payload);
        });

        // 监听系统托盘事件
        const unlistenTray = await listen('tray_navigate_to', (event: any) => {
          const page = event.payload;
          console.log('🔄 托盘导航至:', page);
          setAppState(prev => ({ ...prev, currentPage: page }));
        });

        // 返回清理函数
        return () => {
          unlistenGlobalShortcut();
          unlistenTranscription();
          unlistenTray();
        };
      } catch (error) {
        console.error('❌ 设置事件监听器失败:', error);
        return () => {}; // 返回空的清理函数
      }
    };

    // 执行初始化
    initializeApp();
    setupEventListeners();

    // 组件卸载时的清理
    return () => {
      console.log('🧹 Spokenly App 清理完成');
    };
  }, []);

  // 处理权限模态框关闭
  const handlePermissionModalClose = () => {
    setAppState(prev => ({ ...prev, showPermissionModal: false }));
  };

  // 处理页面切换
  const handlePageChange = (page: string) => {
    setAppState(prev => ({ ...prev, currentPage: page }));
  };

  // 如果未初始化，显示加载状态
  if (!appState.initialized) {
    return (
      <div className="spokenly-loading">
        <div className="loading-spinner"></div>
        <h2>正在启动 Recording King...</h2>
        <p>初始化 Spokenly 界面系统</p>

        <style jsx>{`
          .spokenly-loading {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 100vh;
            background-color: var(--spokenly-bg-app);
            gap: var(--spokenly-space-4);
            font-family: var(--spokenly-font-family);
          }

          .loading-spinner {
            width: 48px;
            height: 48px;
            border: 4px solid var(--spokenly-border-primary);
            border-top: 4px solid var(--spokenly-primary);
            border-radius: 50%;
            animation: spin 1s linear infinite;
          }

          .spokenly-loading h2 {
            font-size: var(--spokenly-text-xl);
            font-weight: var(--spokenly-font-semibold);
            color: var(--spokenly-text-primary);
            margin: 0;
          }

          .spokenly-loading p {
            font-size: var(--spokenly-text-base);
            color: var(--spokenly-text-secondary);
            margin: 0;
          }

          @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
          }
        `}</style>
      </div>
    );
  }

  return (
    <>
      {/* 主应用布局 */}
      <MainLayout 
        initialPage={appState.currentPage}
        onPageChange={handlePageChange}
      />

      {/* 权限警告模态框 */}
      {appState.showPermissionModal && (
        <div className="permission-modal">
          <div className="permission-modal-content">
            <div className="permission-icon">🔐</div>
            <h3>需要系统权限</h3>
            <p>
              Recording King 需要以下权限才能正常工作：<br/>
              • 输入监控权限（用于全局快捷键）<br/>
              • 麦克风访问权限（用于语音录制）
            </p>
            <div className="permission-actions">
              <button 
                className="permission-btn primary"
                onClick={async () => {
                  try {
                    await invoke('open_system_preferences');
                    handlePermissionModalClose();
                  } catch (error) {
                    console.error('打开系统偏好设置失败:', error);
                  }
                }}
              >
                打开系统偏好设置
              </button>
              <button 
                className="permission-btn secondary"
                onClick={handlePermissionModalClose}
              >
                稍后设置
              </button>
            </div>
          </div>
          
          <style jsx>{`
            .permission-modal {
              position: fixed;
              top: 0;
              left: 0;
              right: 0;
              bottom: 0;
              background-color: rgba(0, 0, 0, 0.5);
              display: flex;
              align-items: center;
              justify-content: center;
              z-index: var(--spokenly-z-modal);
              backdrop-filter: blur(8px);
            }

            .permission-modal-content {
              background-color: var(--spokenly-bg-card);
              border-radius: var(--spokenly-radius-xl);
              padding: var(--spokenly-space-8);
              box-shadow: var(--spokenly-shadow-xl);
              border: 1px solid var(--spokenly-border-primary);
              max-width: 480px;
              width: 90%;
              text-align: center;
            }

            .permission-icon {
              font-size: 48px;
              margin-bottom: var(--spokenly-space-4);
            }

            .permission-modal-content h3 {
              font-size: var(--spokenly-text-xl);
              font-weight: var(--spokenly-font-semibold);
              color: var(--spokenly-text-primary);
              margin: 0 0 var(--spokenly-space-4) 0;
            }

            .permission-modal-content p {
              font-size: var(--spokenly-text-base);
              color: var(--spokenly-text-secondary);
              line-height: var(--spokenly-leading-relaxed);
              margin: 0 0 var(--spokenly-space-6) 0;
              text-align: left;
            }

            .permission-actions {
              display: flex;
              gap: var(--spokenly-space-3);
              justify-content: center;
            }

            .permission-btn {
              padding: var(--spokenly-space-3) var(--spokenly-space-5);
              border: 1px solid var(--spokenly-border-primary);
              border-radius: var(--spokenly-radius-base);
              font-size: var(--spokenly-text-base);
              font-weight: var(--spokenly-font-medium);
              cursor: pointer;
              transition: all var(--spokenly-duration-fast) var(--spokenly-ease-out);
            }

            .permission-btn.primary {
              background-color: var(--spokenly-primary);
              color: var(--spokenly-text-white);
              border-color: var(--spokenly-primary);
            }

            .permission-btn.primary:hover {
              background-color: var(--spokenly-primary-hover);
              transform: translateY(-1px);
              box-shadow: var(--spokenly-shadow-base);
            }

            .permission-btn.secondary {
              background-color: var(--spokenly-bg-card);
              color: var(--spokenly-text-secondary);
            }

            .permission-btn.secondary:hover {
              background-color: var(--spokenly-bg-hover);
              color: var(--spokenly-text-primary);
            }

            @media (max-width: 640px) {
              .permission-modal-content {
                padding: var(--spokenly-space-6);
              }
              
              .permission-actions {
                flex-direction: column;
              }
            }
          `}</style>
        </div>
      )}

      {/* 全局样式重写 */}
      <style jsx global>{`
        /* 确保全局样式正确应用 */
        body {
          margin: 0;
          padding: 0;
          font-family: var(--spokenly-font-family);
          background-color: var(--spokenly-bg-app);
          color: var(--spokenly-text-primary);
          overflow: hidden; /* 防止 body 级别的滚动 */
        }

        #root {
          height: 100vh;
          overflow: hidden;
        }

        /* 滚动条全局样式 */
        ::-webkit-scrollbar {
          width: 8px;
          height: 8px;
        }

        ::-webkit-scrollbar-track {
          background: transparent;
        }

        ::-webkit-scrollbar-thumb {
          background: var(--spokenly-border-secondary);
          border-radius: var(--spokenly-radius-full);
        }

        ::-webkit-scrollbar-thumb:hover {
          background: var(--spokenly-text-tertiary);
        }

        /* 选择文本样式 */
        ::selection {
          background-color: var(--spokenly-primary-light);
          color: var(--spokenly-text-primary);
        }

        /* 焦点样式 */
        :focus-visible {
          outline: 2px solid var(--spokenly-primary);
          outline-offset: 2px;
        }

        /* 按钮重置 */
        button {
          cursor: pointer;
          border: none;
          background: transparent;
          font-family: inherit;
        }

        /* 输入框重置 */
        input, textarea, select {
          font-family: inherit;
          font-size: inherit;
        }
      `}</style>
    </>
  );
};

export default SpokenlyApp;