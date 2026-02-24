import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { RecordingPage } from './RecordingPage';
import { useAppStore } from '../../shared/stores/useAppStore';
import { invoke } from '@tauri-apps/api/tauri';

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

describe('RecordingPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    const store = useAppStore.getState();
    store.setTranscribing(false);
    store.toasts.forEach(t => store.removeToast(t.id));
  });

  it('should render recording button', () => {
    render(<RecordingPage />);
    
    expect(screen.getByRole('button', { name: '开始录音' })).toBeInTheDocument();
  });

  it('should disable recording button when transcribing', () => {
    const { setTranscribing } = useAppStore.getState();
    setTranscribing(true);

    render(<RecordingPage />);
    
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
    expect(screen.getByText('正在转录...')).toBeInTheDocument();
  });

  it('should enable recording button when not transcribing', () => {
    const { setTranscribing } = useAppStore.getState();
    setTranscribing(false);

    render(<RecordingPage />);
    
    const button = screen.getByRole('button');
    expect(button).not.toBeDisabled();
  });

  it('should call start_recording when button clicked', async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue(undefined);

    render(<RecordingPage />);
    
    const button = screen.getByRole('button', { name: '开始录音' });
    fireEvent.click(button);

    expect(mockInvoke).toHaveBeenCalledWith('start_recording');
  });

  it('should show progress indicator when transcribing', () => {
    const { setTranscribing } = useAppStore.getState();
    setTranscribing(true);

    render(<RecordingPage />);
    
    expect(screen.getByText('正在转录...')).toBeInTheDocument();
  });
});
