import { TranscriptionModel } from '../types/models';

// Whisper 本地模型定义
export const whisperModels: TranscriptionModel[] = [
  // Whisper Tiny
  {
    id: 'whisper-tiny',
    name: 'Whisper Tiny',
    provider: 'OpenAI Whisper',
    description: '最小最快的模型，适合快速草稿和低端设备',
    icon: '🎯',
    type: 'local',
    category: ['all', 'local', 'fast'],
    accuracy: 2,
    speed: 5,
    languages: ['多语言'],
    realtime: false,
    recommended: false,
    requiresApiKey: false,
    modelSize: '39 MB',
    downloadUrl: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin',
    installed: false,
    features: ['超快速', '准确度 ••', '速度 •••••', '离线', '39MB'],
    systemRequirements: {
      minRam: '1GB',
      recommendedRam: '2GB',
      diskSpace: '100MB',
      gpu: false
    }
  },
  
  // Whisper Base
  {
    id: 'whisper-base',
    name: 'Whisper Base',
    provider: 'OpenAI Whisper',
    description: '平衡的基础模型，适合日常使用',
    icon: '⚡',
    type: 'local',
    category: ['all', 'local', 'fast'],
    accuracy: 3,
    speed: 4,
    languages: ['多语言'],
    realtime: false,
    recommended: false,
    requiresApiKey: false,
    modelSize: '74 MB',
    downloadUrl: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin',
    installed: false,
    features: ['快速', '准确度 •••', '速度 ••••', '离线', '74MB'],
    systemRequirements: {
      minRam: '1GB',
      recommendedRam: '2GB',
      diskSpace: '150MB',
      gpu: false
    }
  },
  
  // Whisper Small
  {
    id: 'whisper-small',
    name: 'Whisper Small',
    provider: 'OpenAI Whisper',
    description: '良好的准确性和速度平衡，推荐用于大多数用例',
    icon: '🎤',
    type: 'local',
    category: ['all', 'local', 'accurate'],
    accuracy: 4,
    speed: 3,
    languages: ['多语言'],
    realtime: false,
    recommended: true,
    requiresApiKey: false,
    modelSize: '244 MB',
    downloadUrl: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin',
    installed: false,
    features: ['推荐', '准确度 ••••', '速度 •••', '离线', '244MB'],
    systemRequirements: {
      minRam: '2GB',
      recommendedRam: '4GB',
      diskSpace: '500MB',
      gpu: false
    }
  },
  
  // Whisper Medium
  {
    id: 'whisper-medium',
    name: 'Whisper Medium',
    provider: 'OpenAI Whisper',
    description: '高准确性模型，适合专业转录',
    icon: '💎',
    type: 'local',
    category: ['all', 'local', 'accurate', 'punctuation'],
    accuracy: 4,
    speed: 2,
    languages: ['多语言'],
    realtime: false,
    recommended: false,
    requiresApiKey: false,
    modelSize: '769 MB',
    downloadUrl: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin',
    installed: false,
    features: ['高准确', '准确度 ••••', '速度 ••', '离线', '769MB'],
    systemRequirements: {
      minRam: '4GB',
      recommendedRam: '8GB',
      diskSpace: '1.5GB',
      gpu: false
    }
  },
  
  // Whisper Large
  {
    id: 'whisper-large-v3',
    name: 'Whisper Large v3',
    provider: 'OpenAI Whisper',
    description: '最准确的模型，最新v3版本，支持所有功能',
    icon: '👑',
    type: 'local',
    category: ['all', 'local', 'accurate', 'punctuation', 'subtitle'],
    accuracy: 5,
    speed: 1,
    languages: ['多语言 (99种语言)'],
    realtime: false,
    recommended: false,
    requiresApiKey: false,
    modelSize: '1.55 GB',
    downloadUrl: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin',
    installed: false,
    features: ['最准确', '准确度 •••••', '速度 •', '离线', '1.55GB', '标点符号'],
    systemRequirements: {
      minRam: '8GB',
      recommendedRam: '16GB',
      diskSpace: '3GB',
      gpu: true
    }
  },

  // Whisper Large v3 Turbo
  {
    id: 'whisper-large-v3-turbo',
    name: 'Whisper Large v3 Turbo',
    provider: 'OpenAI Whisper',
    description: '优化版Large模型，速度提升2倍，准确度略有降低',
    icon: '🚀',
    type: 'local',
    category: ['all', 'local', 'accurate', 'fast', 'punctuation'],
    accuracy: 4,
    speed: 3,
    languages: ['多语言 (99种语言)'],
    realtime: false,
    recommended: false,
    requiresApiKey: false,
    modelSize: '809 MB',
    downloadUrl: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin',
    installed: false,
    features: ['快速+准确', '准确度 ••••', '速度 •••', '离线', '809MB'],
    systemRequirements: {
      minRam: '4GB',
      recommendedRam: '8GB',
      diskSpace: '2GB',
      gpu: false
    }
  },

  // 中文优化模型
  {
    id: 'whisper-medium-zh',
    name: 'Whisper Medium 中文优化',
    provider: 'OpenAI Whisper',
    description: '针对中文优化的Medium模型，中文识别准确度提升30%',
    icon: '🇨🇳',
    type: 'local',
    category: ['all', 'local', 'accurate', 'punctuation'],
    accuracy: 5,
    speed: 2,
    languages: ['中文', '英文'],
    realtime: false,
    recommended: true,
    requiresApiKey: false,
    modelSize: '769 MB',
    downloadUrl: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin',
    installed: false,
    features: ['中文优化', '准确度 •••••', '速度 ••', '离线', '769MB'],
    systemRequirements: {
      minRam: '4GB',
      recommendedRam: '8GB',
      diskSpace: '1.5GB',
      gpu: false
    }
  },

  // 英文优化模型
  {
    id: 'whisper-small-en',
    name: 'Whisper Small English',
    provider: 'OpenAI Whisper',
    description: '英文专用模型，英文识别准确度极高，速度快',
    icon: '🇬🇧',
    type: 'local',
    category: ['all', 'local', 'accurate', 'fast'],
    accuracy: 5,
    speed: 4,
    languages: ['仅英语'],
    realtime: false,
    recommended: false,
    requiresApiKey: false,
    modelSize: '244 MB',
    downloadUrl: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin',
    installed: false,
    features: ['英文专用', '准确度 •••••', '速度 ••••', '离线', '244MB'],
    systemRequirements: {
      minRam: '2GB',
      recommendedRam: '4GB',
      diskSpace: '500MB',
      gpu: false
    }
  }
];

// 获取所有本地模型
export function getLocalModels(): TranscriptionModel[] {
  return whisperModels;
}

// 根据系统配置推荐模型
export function getRecommendedLocalModel(): TranscriptionModel | null {
  // 这里可以根据系统配置推荐合适的模型
  // 暂时返回 whisper-small 作为默认推荐
  return whisperModels.find(m => m.id === 'whisper-small') || null;
}

// 检查模型是否已安装
export async function checkModelInstalled(modelId: string): Promise<boolean> {
  try {
    // 调用 Tauri 后端检查模型文件是否存在
    const { invoke } = await import('@tauri-apps/api/tauri');
    return await invoke<boolean>('check_model_installed', { modelId });
  } catch {
    return false;
  }
}

// 获取模型安装路径
export function getModelPath(modelId: string): string {
  // 模型默认安装在应用数据目录下
  return `models/whisper/${modelId}.bin`;
}