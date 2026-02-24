import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { fc, test } from 'fast-check';
import App from './App';
import { useAppStore } from './shared/stores/useAppStore';

// Mock Tauri API
import { vi } from 'vitest';
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn().mockResolvedValue({}),
}));

describe('App Property Tests', () => {
  beforeEach(() => {
    const store = useAppStore.getState();
    store.setInitializing(false);
    store.setInitError(null);
    store.toasts.forEach(t => store.removeToast(t.id));
  });

  it('Property 8: Loading indicator syncs with initialization state', () => {
    test.prop([fc.boolean()])((isInitializing) => {
      const { setInitializing } = useAppStore.getState();
      setInitializing(isInitializing);

      const { container } = render(<App />);

      if (isInitializing) {
        expect(screen.getByText('加载中...')).toBeInTheDocument();
        expect(screen.queryByText('Recording King')).not.toBeInTheDocument();
      } else {
        expect(screen.queryByText('加载中...')).not.toBeInTheDocument();
      }
    });
  });
});
