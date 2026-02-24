import '@testing-library/jest-dom';

// Mock Tauri API
global.window = global.window || {};
(global.window as any).__TAURI__ = {
  invoke: vi.fn(),
  event: {
    listen: vi.fn(),
    emit: vi.fn(),
  },
};
