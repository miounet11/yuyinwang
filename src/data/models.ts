import { TranscriptionModel, APIProvider } from '../types/models';

// 预定义的转录模型
export const transcriptionModels: TranscriptionModel[] = [
  // 主要在线模型
  {
    id: 'deepgram-nova-3',
    name: 'Online Real-time Nova-3 (English Only)',
    provider: 'Deepgram Nova-3',
    description: '实时听写具有卓越准确性。纯英语优化版本。',
    icon: 'D',
    type: 'online',
    category: ['all', 'online', 'api', 'fast', 'accurate'],
    accuracy: 5,
    speed: 5,
    languages: ['仅英语'],
    realtime: true,
    recommended: false,
    requiresApiKey: true,
    apiKeyField: 'deepgram_api_key',
    features: ['最快', '准确度 •••••', '速度 •••••', '实时']
  },
  {
    id: 'gpt-4o-mini',
    name: 'Online GPT-4o mini Transcribe',
    provider: 'OpenAI GPT-4o mini',
    description: '卓越准确性和快速处理。比Whisper或Nova模型更准确。',
    icon: '🌀',
    type: 'online',
    category: ['all', 'online', 'api', 'accurate', 'punctuation'],
    accuracy: 5,
    speed: 3,
    languages: ['多语言'],
    realtime: false,
    recommended: true,
    requiresApiKey: false,
    features: ['最准确', '准确度 •••••', '速度 •••', '多语言']
  },
  {
    id: 'voxtral-mini',
    name: 'Online Voxtral Mini',
    provider: 'Mistral AI',
    description: 'fast and accurate transcription model with excellent multilingual support. Delivers high-quality results comparable to GPT-4o mini.',
    icon: 'M',
    type: 'online',
    category: ['all', 'online', 'api', 'fast', 'accurate'],
    accuracy: 4,
    speed: 4,
    languages: ['多语言'],
    realtime: false,
    requiresApiKey: false,
    features: ['准确度 ••••', '速度 ••••', '多语言']
  },
  {
    id: 'elevenlabs-scribe',
    name: 'Online ElevenLabs Scribe',
    provider: 'ElevenLabs Scribe',
    description: '由ElevenLabs Scribe驱动 - 高质量转录配备先进语音识别和多语言支持。',
    icon: 'II',
    type: 'online',
    category: ['all', 'online', 'api', 'accurate'],
    accuracy: 4,
    speed: 3,
    languages: ['多语言'],
    realtime: false,
    requiresApiKey: false,
    features: ['准确度 ••••', '速度 •••', '多语言']
  },
];

// API提供商配置
export const apiProviders: APIProvider[] = [
  {
    id: 'deepgram',
    name: 'Deepgram API',
    icon: 'D',
    description: '用于精确语音转录的Deepgram API',
    requiresApiKey: true,
    apiKeyField: 'deepgram_api_key',
    models: ['nova-3', 'nova-2', 'enhanced', 'base'],
    configFields: [
      {
        name: 'api_key',
        label: 'API 密钥',
        type: 'password',
        placeholder: '**********',
        required: true
      },
      {
        name: 'model',
        label: '模型',
        type: 'select',
        options: [
          { value: 'nova-3', label: 'Nova-3 (最新)' },
          { value: 'nova-2', label: 'Nova-2' },
          { value: 'enhanced', label: 'Enhanced' },
          { value: 'base', label: 'Base' }
        ],
        defaultValue: 'nova-3'
      },
      {
        name: 'realtime_transcription',
        label: '启用实时转录',
        type: 'toggle',
        defaultValue: true
      },
      {
        name: 'detect_speaker',
        label: '启用语音检测',
        type: 'toggle',
        defaultValue: false
      }
    ],
    testEndpoint: 'https://api.deepgram.com/v1/listen'
  },
  {
    id: 'openai',
    name: 'OpenAI API',
    icon: '🌀',
    description: '用于转录的OpenAI API服务',
    requiresApiKey: true,
    apiKeyField: 'openai_api_key',
    models: ['gpt-4o-transcribe', 'whisper-1'],
    configFields: [
      {
        name: 'api_key',
        label: 'API 密钥',
        type: 'password',
        placeholder: 'sk-...',
        required: true
      },
      {
        name: 'model',
        label: '模型',
        type: 'select',
        options: [
          { value: 'gpt-4o-transcribe', label: 'GPT-4o Transcribe (推荐)' },
          { value: 'whisper-1', label: 'Whisper-1' }
        ],
        defaultValue: 'gpt-4o-transcribe'
      },
      {
        name: 'temperature',
        label: '温度',
        type: 'slider',
        min: 0,
        max: 1,
        step: 0.1,
        defaultValue: 0
      }
    ],
    testEndpoint: 'https://api.openai.com/v1/audio/transcriptions'
  },
  {
    id: 'mistral',
    name: 'Mistral AI',
    icon: 'M',
    description: '使用Voxtral模型进行转录的Mistral AI API',
    requiresApiKey: true,
    apiKeyField: 'mistral_api_key',
    models: ['voxtral-mini', 'voxtral'],
    configFields: [
      {
        name: 'api_key',
        label: 'API 密钥',
        type: 'password',
        placeholder: '**********',
        required: true
      },
      {
        name: 'model',
        label: '模型',
        type: 'select',
        options: [
          { value: 'voxtral-mini', label: 'Voxtral Mini' },
          { value: 'voxtral', label: 'Voxtral' }
        ],
        defaultValue: 'voxtral-mini'
      }
    ],
    testEndpoint: 'https://api.mistral.ai/v1/audio/transcriptions'
  }
];

// 模型分类
export const modelCategories = [
  { id: 'all', label: '全部' },
  { id: 'online', label: '在线' },
  { id: 'local', label: '本地' },
  { id: 'api', label: 'API' },
  { id: 'fast', label: '快速' },
  { id: 'accurate', label: '准确' },
  { id: 'punctuation', label: '标点符号' },
  { id: 'subtitle', label: '字幕' }
];

// 工具函数
export function getModelsByCategory(category: string): TranscriptionModel[] {
  if (category === 'all') {
    return transcriptionModels;
  }
  return transcriptionModels.filter(model => model.category.includes(category as any));
}

export function getModelById(id: string): TranscriptionModel | undefined {
  return transcriptionModels.find(model => model.id === id);
}

export function getProviderById(id: string): APIProvider | undefined {
  return apiProviders.find(provider => provider.id === id);
}