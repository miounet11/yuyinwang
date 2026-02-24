import { describe, it, expect, beforeEach } from 'vitest';
import { useAppStore } from './useAppStore';

describe('useAppStore', () => {
  beforeEach(() => {
    // Reset store state
    const store = useAppStore.getState();
    store.setError(null);
    store.toasts.forEach(t => store.removeToast(t.id));
    store.setInitializing(false);
    store.setTranscribing(false);
    store.setInitError(null);
  });

  describe('Error state', () => {
    it('should set and clear error', () => {
      const { setError, error } = useAppStore.getState();
      
      setError('Test error');
      expect(useAppStore.getState().error).toBe('Test error');
      
      setError(null);
      expect(useAppStore.getState().error).toBeNull();
    });
  });

  describe('Toast management', () => {
    it('should add toast', () => {
      const { addToast, toasts } = useAppStore.getState();
      
      addToast('success', 'Test message');
      
      const state = useAppStore.getState();
      expect(state.toasts).toHaveLength(1);
      expect(state.toasts[0].type).toBe('success');
      expect(state.toasts[0].message).toBe('Test message');
    });

    it('should translate error messages', () => {
      const { addToast } = useAppStore.getState();
      
      addToast('error', 'API key not configured');
      
      const state = useAppStore.getState();
      expect(state.toasts[0].message).toBe('API 密钥未配置');
    });

    it('should remove toast', () => {
      const { addToast, removeToast } = useAppStore.getState();
      
      addToast('info', 'Test');
      const toastId = useAppStore.getState().toasts[0].id;
      
      removeToast(toastId);
      expect(useAppStore.getState().toasts).toHaveLength(0);
    });

    it('should add multiple toasts', () => {
      const { addToast } = useAppStore.getState();
      
      addToast('success', 'Message 1');
      addToast('error', 'Message 2');
      addToast('warning', 'Message 3');
      
      expect(useAppStore.getState().toasts).toHaveLength(3);
    });
  });

  describe('Loading state', () => {
    it('should set initializing state', () => {
      const { setInitializing } = useAppStore.getState();
      
      setInitializing(true);
      expect(useAppStore.getState().isInitializing).toBe(true);
      
      setInitializing(false);
      expect(useAppStore.getState().isInitializing).toBe(false);
    });

    it('should set transcribing state', () => {
      const { setTranscribing } = useAppStore.getState();
      
      setTranscribing(true);
      expect(useAppStore.getState().isTranscribing).toBe(true);
      
      setTranscribing(false);
      expect(useAppStore.getState().isTranscribing).toBe(false);
    });

    it('should set init error', () => {
      const { setInitError } = useAppStore.getState();
      
      setInitError('Init failed');
      expect(useAppStore.getState().initError).toBe('Init failed');
      
      setInitError(null);
      expect(useAppStore.getState().initError).toBeNull();
    });
  });
});
