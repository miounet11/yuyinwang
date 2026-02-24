import { describe, it, beforeEach } from 'vitest';
import { fc, test } from 'fast-check';
import { useAppStore } from './useAppStore';

describe('useAppStore Property Tests', () => {
  beforeEach(() => {
    const store = useAppStore.getState();
    store.toasts.forEach(t => store.removeToast(t.id));
  });

  it('Property 7: Tauri command errors visible in UI', () => {
    test.prop([fc.string({ minLength: 5, maxLength: 50 })])((errorMessage) => {
      const { addToast } = useAppStore.getState();

      addToast('error', errorMessage);

      const state = useAppStore.getState();
      expect(state.toasts.length).toBeGreaterThan(0);
      expect(state.toasts[0].type).toBe('error');
      expect(state.toasts[0].message).toBeTruthy();
    });
  });

  it('Property: Toast messages are unique by ID', () => {
    test.prop([fc.array(fc.string(), { minLength: 1, maxLength: 10 })])((messages) => {
      const { addToast } = useAppStore.getState();

      messages.forEach(msg => addToast('info', msg));

      const state = useAppStore.getState();
      const ids = state.toasts.map(t => t.id);
      const uniqueIds = new Set(ids);

      expect(uniqueIds.size).toBe(ids.length);
    });
  });
});
