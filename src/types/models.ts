// 模型类型定义
export interface TranscriptionModel {
  id: string;
  name: string;
  provider: string;
  description: string;
  icon: string;
  type: 'online' | 'local';
  category: ('all' | 'online' | 'local' | 'api' | 'fast' | 'accurate' | 'punctuation' | 'subtitle' | 'realtime' | 'dictation')[];
  accuracy: number; // 1-5
  speed: number; // 1-5
  languages: string[];
  realtime?: boolean;
  recommended?: boolean;
  requiresApiKey?: boolean;
  apiKeyField?: string;
  modelSize?: string; // e.g., "564 MB"
  downloadUrl?: string;
  installed?: boolean;
  downloading?: boolean;
  downloadProgress?: number;
  configurable?: boolean;
  features?: string[];
  status?: 'available' | 'downloading' | 'installed' | 'error';
  apiConfig?: {
    apiKey?: string;
    endpoint?: string;
  };
  performance?: {
    speed: number;
    accuracy: number;
    quality: number;
    realtime?: number;
  };
  systemRequirements?: {
    minRam: string;
    recommendedRam: string;
    diskSpace: string;
    gpu: boolean;
  };
}

export interface ModelConfig {
  modelId: string;
  apiKey?: string;
  apiUrl?: string;
  language?: string;
  translation?: boolean;
  realtimeTranscription?: boolean;
  detectSpeaker?: boolean;
  maxSentenceLength?: number;
  threads?: number;
  temperature?: number;
  prompt?: string;
  exportRewriting?: boolean;
  customModelPath?: string;
}

export interface APIProvider {
  id: string;
  name: string;
  icon: string;
  description: string;
  requiresApiKey: boolean;
  apiKeyField: string;
  models: string[];
  configFields: ConfigField[];
  testEndpoint?: string;
}

export interface ConfigField {
  name: string;
  label: string;
  type: 'text' | 'password' | 'select' | 'toggle' | 'number' | 'slider' | 'file';
  placeholder?: string;
  options?: { value: string; label: string }[];
  min?: number;
  max?: number;
  step?: number;
  defaultValue?: any;
  required?: boolean;
  description?: string;
}

export interface DownloadTask {
  modelId: string;
  progress: number;
  speed: string;
  remaining: string;
  status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error';
  error?: string;
}

export interface DownloadStatus {
  modelId: string;
  progress: number;
  status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error';
  speed?: string;
  remaining?: string;
  error?: string;
}

export interface ApiConfig {
  apiKey?: string;
  apiUrl?: string;
  model?: string;
  language?: string;
  temperature?: number;
  customSettings?: Record<string, any>;
}

export interface ModelCategory {
  id: string;
  label: string;
  count?: number;
  active?: boolean;
}