import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import App from './App';
import { useAppStore } from './shared/stores/useAppStore';

// Mock Tauri API
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn().mockResolvedValue({}),
}));

describe('App', () => {
  beforeEach(() => {
    const store = useAppStore.getState();
    store.setInitializing(false);
    store.setInitError(null);
    store.toasts.forEach(t => store.removeToast(t.id));
  });

  it('should show loading indicator when initializing', () => {
    const { setInitializing } = useAppStore.getState();
    setInitializing(true);

    render(<App />);
    
    expect(screen.getByText('加载中...')).toBeInTheDocument();
    expect(screen.queryByText('Recording King')).not.toBeInTheDocument();
  });

  it('should show main content when not initializing', async () => {
    const { setInitializing } = useAppStore.getState();
    setInitializing(false);

    render(<App />);
    
    await waitFor(() => {
      expect(screen.getByText('Recording King')).toBeInTheDocument();
    });
    expect(screen.queryByText('加载中...')).not.toBeInTheDocument();
  });

  it('should render ToastContainer', () => {
    render(<App />);
    
    // ToastContainer is always rendered (even if empty)
    const { addToast } = useAppStore.getState();
    addToast('success', 'Test toast');
    
    expect(screen.getByText('Test toast')).toBeInTheDocument();
  });
});
