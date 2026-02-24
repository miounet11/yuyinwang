import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { SettingsPage } from './SettingsPage';
import { invoke } from '@tauri-apps/api/tauri';
import { useAppStore } from '../../shared/stores/useAppStore';

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

describe('SettingsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    const store = useAppStore.getState();
    store.toasts.forEach(t => store.removeToast(t.id));
  });

  it('should render settings page', () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue({
      openai_api_key: null,
      selected_model: 'whisper-1',
      auto_inject: false,
      inject_delay_ms: 100,
      shortcut_key: null,
    });

    render(<SettingsPage />);
    
    expect(screen.getByText('设置')).toBeInTheDocument();
    expect(screen.getByLabelText('OpenAI API Key:')).toBeInTheDocument();
  });

  it('should load settings on mount', async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue({
      openai_api_key: 'test-key',
      selected_model: 'whisper-1',
      auto_inject: false,
      inject_delay_ms: 100,
      shortcut_key: null,
    });

    render(<SettingsPage />);
    
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('get_settings');
    });
  });

  it('should show success toast when settings saved', async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue({
      openai_api_key: null,
      selected_model: 'whisper-1',
      auto_inject: false,
      inject_delay_ms: 100,
      shortcut_key: null,
    });

    render(<SettingsPage />);
    
    await waitFor(() => {
      expect(screen.getByRole('button', { name: '保存' })).toBeInTheDocument();
    });

    mockInvoke.mockResolvedValueOnce(undefined);
    const saveButton = screen.getByRole('button', { name: '保存' });
    fireEvent.click(saveButton);

    await waitFor(() => {
      const toasts = useAppStore.getState().toasts;
      expect(toasts.some(t => t.message === '设置已保存')).toBe(true);
    });
  });

  it('should show error toast when settings save fails', async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue({
      openai_api_key: null,
      selected_model: 'whisper-1',
      auto_inject: false,
      inject_delay_ms: 100,
      shortcut_key: null,
    });

    render(<SettingsPage />);
    
    await waitFor(() => {
      expect(screen.getByRole('button', { name: '保存' })).toBeInTheDocument();
    });

    mockInvoke.mockRejectedValueOnce(new Error('Save failed'));
    const saveButton = screen.getByRole('button', { name: '保存' });
    fireEvent.click(saveButton);

    await waitFor(() => {
      const toasts = useAppStore.getState().toasts;
      expect(toasts.some(t => t.message === '保存设置失败')).toBe(true);
    });
  });
});
