import { describe, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { fc, test } from 'fast-check';
import { RecordingPage } from './RecordingPage';
import { useAppStore } from '../../shared/stores/useAppStore';
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

describe('RecordingPage Property Tests', () => {
  it('Property 9: Recording button disabled when transcribing', () => {
    test.prop([fc.boolean()])((isTranscribing) => {
      const { setTranscribing } = useAppStore.getState();
      setTranscribing(isTranscribing);

      render(<RecordingPage />);

      const button = screen.getByRole('button');

      if (isTranscribing) {
        expect(button).toBeDisabled();
        expect(screen.getByText('正在转录...')).toBeInTheDocument();
      } else {
        expect(button).not.toBeDisabled();
      }
    });
  });
});
