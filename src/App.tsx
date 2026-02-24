import { useEffect, useState, lazy, Suspense } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from './shared/stores/useAppStore';
import { ToastContainer } from './shared/components/Toast';
import type { Page, NavItem } from './shared/types';
import {
  SettingsIcon,
  ShortcutIcon,
  ModelIcon,
  TranscribeIcon,
  AIPromptsIcon,
  HistoryIcon,
  PermissionsIcon,
  RecordingIcon,
} from './shared/components/icons';
import './App.css';

// Lazy load page components for code splitting
const GeneralSettings = lazy(() => import('./features/settings/GeneralSettings').then(m => ({ default: m.GeneralSettings })));
const PermissionsPage = lazy(() => import('./features/settings/PermissionsPage').then(m => ({ default: m.PermissionsPage })));
const ModelSettings = lazy(() => import('./features/models/ModelSettings').then(m => ({ default: m.ModelSettings })));
const TranscribeFilePage = lazy(() => import('./features/transcribe/TranscribeFilePage').then(m => ({ default: m.TranscribeFilePage })));
const HistoryPage = lazy(() => import('./features/history/HistoryPage').then(m => ({ default: m.HistoryPage })));
const RecordingPage = lazy(() => import('./features/recording/RecordingPage').then(m => ({ default: m.RecordingPage })));
const AIPromptsPage = lazy(() => import('./features/ai-prompts').then(m => ({ default: m.AIPromptsPage })));
const ShortcutSettings = lazy(() => import('./features/shortcuts/ShortcutSettings').then(m => ({ default: m.ShortcutSettings })));
const OnboardingPage = lazy(() => import('./features/onboarding/OnboardingPage').then(m => ({ default: m.OnboardingPage })));

const NAV_ITEMS: NavItem[] = [
  { key: 'general', icon: <SettingsIcon />, label: '常规设置' },
  { key: 'shortcuts', icon: <ShortcutIcon />, label: '快捷键设置' },
  { key: 'models', icon: <ModelIcon />, label: '听写模型' },
  { key: 'transcribe', icon: <TranscribeIcon />, label: '转录文件' },
  { key: 'ai-prompts', icon: <AIPromptsIcon />, label: 'AI 提示' },
  { key: 'history', icon: <HistoryIcon />, label: '历史记录' },
  { key: 'permissions', icon: <PermissionsIcon />, label: '权限管理' },
  { key: 'recording', icon: <RecordingIcon />, label: '语音输入' },
];

function App() {
  const { toasts, removeToast, isInitializing, setInitializing, setInitError, addToast, settings } = useAppStore();
  const [currentPage, setCurrentPage] = useState<Page>('general');
  const [permissionWarning, setPermissionWarning] = useState(false);

  useEffect(() => {
    initializeApp();

    // 监听导航事件
    const unlistenNavigate = listen<string>('navigate', (event) => {
      const page = event.payload as Page;
      setCurrentPage(page);
    });

    // 监听快捷键事件
    const unlistenStarted = listen('quick-input-started', () => {
      console.log('🎤 快捷键录音已开始');
    });

    const unlistenResult = listen<string>('quick-input-result', (event) => {
      console.log('✅ 转录完成:', event.payload);
      addToast('success', `转录完成: ${event.payload}`);
    });

    const unlistenError = listen<string>('quick-input-error', (event) => {
      console.error('❌ 快捷键错误:', event.payload);
      addToast('error', event.payload);
    });

    const unlistenInjectionFailed = listen<string>('quick-input-injection-failed', (event) => {
      console.error('❌ 文本注入失败:', event.payload);
      addToast('error', event.payload);
    });

    return () => {
      unlistenNavigate.then(fn => fn());
      unlistenStarted.then(fn => fn());
      unlistenResult.then(fn => fn());
      unlistenError.then(fn => fn());
      unlistenInjectionFailed.then(fn => fn());
    };
  }, [addToast]);

  const initializeApp = async () => {
    setInitializing(true);
    try {
      const loadedSettings = await invoke('get_settings') as any;

      // Check if onboarding is complete
      if (!loadedSettings.onboarding_complete) {
        setCurrentPage('onboarding');
      }

      // Check permissions
      try {
        const hasPerm = await invoke<boolean>('check_injection_permission');
        if (!hasPerm) setPermissionWarning(true);
      } catch {}
    } catch (error) {
      setInitError(String(error));
      addToast('error', String(error));
    } finally {
      setInitializing(false);
    }
  };

  if (isInitializing) {
    return (
      <div className="loading-screen">
        <div className="loading-spinner" />
        <p className="loading-text">Recording King</p>
      </div>
    );
  }

  return (
    <>
      <ToastContainer toasts={toasts} onClose={removeToast} />
      <div className="app">
        <nav className="sidebar">
          <div className="sidebar-header">
            <span className="sidebar-logo">🎙</span>
            <span className="sidebar-title">Recording King</span>
          </div>
          <div className="nav-items">
            {NAV_ITEMS.map((item) => (
              <button
                key={item.key}
                className={`nav-item ${currentPage === item.key ? 'active' : ''}`}
                onClick={() => setCurrentPage(item.key)}
              >
                <span className="nav-icon">{item.icon}</span>
                <span className="nav-label">{item.label}</span>
                {item.badge !== undefined && item.badge > 0 && (
                  <span className="nav-badge">{item.badge}</span>
                )}
              </button>
            ))}
          </div>
          <div className="sidebar-footer">
            <span className="version-text">版本 7.0.0</span>
          </div>
        </nav>

        <div className="main-area">
          {permissionWarning && (
            <div className="permission-banner">
              <span className="banner-icon">⚠️</span>
              <span className="banner-text">检测到权限问题，快捷键功能可能无法正常工作</span>
              <button className="banner-btn" onClick={() => { setCurrentPage('permissions'); setPermissionWarning(false); }}>
                配置权限
              </button>
              <button className="banner-close" onClick={() => setPermissionWarning(false)}>✕</button>
            </div>
          )}
          <main className="main-content">
            <Suspense fallback={
              <div className="loading-screen">
                <div className="loading-spinner" />
                <p className="loading-text">加载中...</p>
              </div>
            }>
              {currentPage === 'general' && <GeneralSettings />}
              {currentPage === 'shortcuts' && <ShortcutSettings />}
              {currentPage === 'models' && <ModelSettings />}
              {currentPage === 'transcribe' && <TranscribeFilePage />}
              {currentPage === 'ai-prompts' && <AIPromptsPage />}
              {currentPage === 'history' && <HistoryPage />}
              {currentPage === 'permissions' && <PermissionsPage />}
              {currentPage === 'recording' && <RecordingPage />}
              {currentPage === 'onboarding' && <OnboardingPage />}
            </Suspense>
          </main>
        </div>
      </div>
    </>
  );
}

export default App;
