import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from 'path';

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        'floating-input': resolve(__dirname, 'floating-input.html'),
      },
      output: {
        manualChunks: undefined,
      },
    },
  },
}));
