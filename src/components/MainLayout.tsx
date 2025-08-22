/**
 * MainLayout.tsx
 * Recording King 主布局组件
 * 复刻 Spokenly 界面设计
 */

import React, { useState, useCallback } from 'react';
import { SpokenlyLayout, SpokemlySidebar, SpokenlyNavItem, SpokenlyNavSection } from './ui';
import GeneralSettings from './pages/GeneralSettings';
import TranscriptionModels from './pages/TranscriptionModels';
import FileTranscription from './pages/FileTranscription';
import HistoryRecords from './pages/HistoryRecords';
import Shortcuts from './pages/Shortcuts';
import AIPrompts from './pages/AIPrompts';

// 导航菜单配置
const navigationConfig = [
  {
    id: 'settings',
    title: '设置',
    items: [
      {
        id: 'general',
        label: '常规设置',
        icon: 'settings',
        component: GeneralSettings
      },
      {
        id: 'shortcuts',
        label: '快捷键',
        icon: 'keyboard',
        component: Shortcuts
      }
    ]
  },
  {
    id: 'transcription',
    title: '转录',
    items: [
      {
        id: 'models',
        label: '听写模型',
        icon: 'microphone',
        component: TranscriptionModels
      },
      {
        id: 'files',
        label: '转录文件',
        icon: 'folder',
        component: FileTranscription
      }
    ]
  },
  {
    id: 'data',
    title: '数据管理',
    items: [
      {
        id: 'history',
        label: '历史记录',
        icon: 'history',
        component: HistoryRecords
      },
      {
        id: 'ai-prompts',
        label: 'AI提示',
        icon: 'brain',
        component: AIPrompts
      }
    ]
  }
];

interface MainLayoutProps {
  initialPage?: string;
  onPageChange?: (page: string) => void;
}

const MainLayout: React.FC<MainLayoutProps> = ({ initialPage = 'general', onPageChange }) => {
  const [currentPage, setCurrentPage] = useState(initialPage);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  // 查找当前页面组件
  const currentPageData = navigationConfig
    .flatMap(section => section.items)
    .find(item => item.id === currentPage);

  const CurrentPageComponent = currentPageData?.component || GeneralSettings;

  const handleNavItemClick = useCallback((pageId: string) => {
    setCurrentPage(pageId);
    onPageChange?.(pageId);
  }, [onPageChange]);

  const handleSidebarToggle = useCallback(() => {
    setSidebarCollapsed(prev => !prev);
  }, []);

  return (
    <SpokenlyLayout className="recording-king-layout">
      {/* 左侧导航栏 */}
      <SpokemlySidebar 
        isCollapsed={sidebarCollapsed}
        onToggle={handleSidebarToggle}
        width={250}
        className="recording-king-sidebar"
      >
        {/* 应用头部 */}
        <div className="sidebar-header">
          <div className="app-logo">
            <div className="logo-icon">●</div>
            {!sidebarCollapsed && (
              <div className="logo-text">Recording King</div>
            )}
          </div>
        </div>

        {/* 导航菜单 */}
        <nav className="sidebar-nav">
          {navigationConfig.map(section => (
            <SpokenlyNavSection
              key={section.id}
              title={sidebarCollapsed ? '' : section.title}
              collapsed={sidebarCollapsed}
            >
              {section.items.map(item => (
                <SpokenlyNavItem
                  key={item.id}
                  label={sidebarCollapsed ? '' : item.label}
                  icon={item.icon}
                  active={currentPage === item.id}
                  onClick={() => handleNavItemClick(item.id)}
                  collapsed={sidebarCollapsed}
                />
              ))}
            </SpokenlyNavSection>
          ))}
        </nav>

        {/* 底部信息 */}
        {!sidebarCollapsed && (
          <div className="sidebar-footer">
            <div className="version-info">v5.7.0</div>
            <div className="upgrade-link">升级 Pro</div>
          </div>
        )}
      </SpokemlySidebar>

      {/* 主内容区域 */}
      <main className="main-content">
        <div className="page-container">
          <CurrentPageComponent />
        </div>
      </main>

      {/* 内联样式 - 主布局专用 */}
      <style jsx>{`
        .recording-king-layout {
          --sidebar-transition: all 0.25s cubic-bezier(0.4, 0.0, 0.2, 1);
        }

        .sidebar-header {
          padding: var(--spokenly-space-6) var(--spokenly-space-4);
          border-bottom: 1px solid var(--spokenly-border-primary);
          margin-bottom: var(--spokenly-space-4);
        }

        .app-logo {
          display: flex;
          align-items: center;
          gap: var(--spokenly-space-3);
        }

        .logo-icon {
          width: 32px;
          height: 32px;
          background: linear-gradient(135deg, var(--spokenly-primary), var(--spokenly-primary-hover));
          color: var(--spokenly-text-white);
          border-radius: var(--spokenly-radius-base);
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: var(--spokenly-text-lg);
          font-weight: var(--spokenly-font-bold);
        }

        .logo-text {
          font-size: var(--spokenly-text-lg);
          font-weight: var(--spokenly-font-semibold);
          color: var(--spokenly-text-primary);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .sidebar-nav {
          flex: 1;
          overflow-y: auto;
          overflow-x: hidden;
        }

        .sidebar-footer {
          padding: var(--spokenly-space-4);
          border-top: 1px solid var(--spokenly-border-primary);
          display: flex;
          flex-direction: column;
          gap: var(--spokenly-space-2);
        }

        .version-info {
          font-size: var(--spokenly-text-sm);
          color: var(--spokenly-text-secondary);
          text-align: center;
        }

        .upgrade-link {
          padding: var(--spokenly-space-2) var(--spokenly-space-3);
          background: linear-gradient(135deg, var(--spokenly-primary), var(--spokenly-primary-hover));
          color: var(--spokenly-text-white);
          text-align: center;
          border-radius: var(--spokenly-radius-base);
          font-size: var(--spokenly-text-sm);
          font-weight: var(--spokenly-font-medium);
          cursor: pointer;
          transition: all var(--spokenly-duration-fast) var(--spokenly-ease-out);
        }

        .upgrade-link:hover {
          transform: translateY(-1px);
          box-shadow: var(--spokenly-shadow-base);
        }

        .main-content {
          flex: 1;
          background-color: var(--spokenly-bg-content);
          overflow: hidden;
        }

        .page-container {
          height: 100%;
          overflow-y: auto;
          overflow-x: hidden;
        }

        /* 响应式适配 */
        @media (max-width: 768px) {
          .recording-king-sidebar {
            position: absolute;
            top: 0;
            left: 0;
            height: 100vh;
            z-index: var(--spokenly-z-modal);
            box-shadow: var(--spokenly-shadow-xl);
          }

          .sidebar-header {
            padding: var(--spokenly-space-4);
          }

          .logo-text {
            font-size: var(--spokenly-text-base);
          }
        }

        @media (max-width: 640px) {
          .recording-king-sidebar {
            width: 100vw !important;
          }
        }
      `}</style>
    </SpokenlyLayout>
  );
};

export default MainLayout;