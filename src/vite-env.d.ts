/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_TTS_API_KEY: string
  readonly VITE_STT_API_KEY: string
  readonly VITE_API_BASE_URL: string
  readonly VITE_TTS_SERVER_URL: string
  readonly VITE_NODE_ENV: string
  readonly VITE_DEBUG_MODE: string
  // 更多环境变量...
}