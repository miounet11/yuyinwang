import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from './shared/stores/useAppStore';
import { GeneralSettings } from './features/settings/GeneralSettings';
import { PermissionsPage } from './features/settings/PermissionsPage';
import { SettingsPage } from './features/settings/SettingsPage';
import { TranscribeFilePage } from './features/transcribe/TranscribeFilePage';
import { HistoryPage } from './features/history/HistoryPage';
import { RecordingPage } from './features/recording/RecordingPage';
import { ToastContainer } from './shared/components/Toast';
import './App.css';

type Page = 'general' | 'permissions' | 'models' | 'transcribe' | 'history' | 'recording';

const NAV_ITEMS: { key: Page; icon: string; label: string }[] = [
  { key: 'general', icon: '⚙', label: '常规设置' },
  { key: 'permissions', icon: '🛡', label: '权限管理' },
  { key: 'models', icon: '🎙', label: '听写模型' },
  { key: 'transcribe', icon: '📁', label: '转录文件' },
  { key: 'history', icon: '📋', label: '历史记录' },
  { key: 'recording', icon: '🎤', label: '语音输入' },
];

function App() {
  const { toasts, removeToast, isInitializing, setInitializing, setInitError, addToast } = useAppStore();
  const [currentPage, setCurrentPage] = useState<Page>('general');
  const [permissionWarning, setPermissionWarning] = useState(false);

  useEffect(() => {
    initializeApp();
    const unlisten = listen<string>('navigate', (event) => {
      const page = event.payload as Page;
      setCurrentPage(page);
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  const initializeApp = async () => {
    setInitializing(true);
    try {
      await invoke('get_settings');
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
            {currentPage === 'general' && <GeneralSettings />}
            {currentPage === 'permissions' && <PermissionsPage />}
            {currentPage === 'models' && <SettingsPage />}
            {currentPage === 'transcribe' && <TranscribeFilePage />}
            {currentPage === 'history' && <HistoryPage />}
            {currentPage === 'recording' && <RecordingPage />}
          </main>
        </div>
      </div>
    </>
  );
}

export default App;
