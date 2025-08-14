// TTS 服务 - 集成 OpenAI TTS API
import logger from '../utils/logger';

interface TTSConfig {
  apiKey: string;
  baseUrl: string;
  model: 'tts-1' | 'tts-1-hd';
  voice: 'alloy' | 'echo' | 'fable' | 'onyx' | 'nova' | 'shimmer';
  responseFormat?: 'mp3' | 'opus' | 'aac' | 'flac' | 'wav' | 'pcm';
  speed?: number; // 0.25 - 4.0
}

interface STTConfig {
  apiKey: string;
  baseUrl: string;
  model: 'whisper-1';
  language?: string; // ISO-639-1
  prompt?: string;
  responseFormat?: 'json' | 'text' | 'srt' | 'verbose_json' | 'vtt';
  temperature?: number; // 0 - 1
}

class TTSService {
  private config: TTSConfig;
  private sttConfig: STTConfig;
  private trialEndDate: Date | null = null;
  private isPro: boolean = false;

  constructor() {
    // 安全配置 - 从环境变量读取
    this.config = {
      apiKey: this.getSecureApiKey('TTS'),
      baseUrl: this.getSecureBaseUrl() || 'https://api.openai.com/v1',
      model: 'tts-1',
      voice: 'alloy',
      responseFormat: 'mp3',
      speed: 1.0
    };

    this.sttConfig = {
      apiKey: this.getSecureApiKey('STT'),
      baseUrl: this.getSecureBaseUrl() || 'https://api.openai.com/v1',
      model: 'whisper-1',
      responseFormat: 'json',
      temperature: 0
    };

    this.loadSubscriptionStatus();
    this.checkTrialStatus();
  }

  // 安全获取API密钥 - 优先级：环境变量 > 用户配置 > 空值
  private getSecureApiKey(type: 'TTS' | 'STT'): string {
    // 尝试从环境变量获取 (Vite使用import.meta.env)
    const envKey = type === 'TTS' ? 
      import.meta.env.VITE_TTS_API_KEY : 
      import.meta.env.VITE_STT_API_KEY;
    
    if (envKey) return envKey;
    
    // 尝试从安全存储获取用户设置的密钥
    const userKey = localStorage.getItem(`spokenly_${type.toLowerCase()}_api_key_encrypted`);
    if (userKey) {
      // 这里应该实现解密逻辑
      return this.decryptApiKey(userKey);
    }
    
    // 如果都没有，返回空值并要求用户配置
    logger.warn(`${type} API key not configured. Please set VITE_${type}_API_KEY environment variable.`);
    return '';
  }

  // 安全获取基础URL
  private getSecureBaseUrl(): string | undefined {
    return import.meta.env.VITE_API_BASE_URL;
  }

  // 简单的加密/解密方法（生产环境应使用更强的加密）
  private decryptApiKey(encryptedKey: string): string {
    try {
      // 这里应该实现真正的解密逻辑
      // 当前只是base64解码示例
      return atob(encryptedKey);
    } catch {
      return '';
    }
  }

  // 安全存储API密钥
  public setApiKey(type: 'TTS' | 'STT', apiKey: string): void {
    if (!apiKey) return;
    
    // 简单加密存储（生产环境应使用更强的加密）
    const encrypted = btoa(apiKey);
    localStorage.setItem(`spokenly_${type.toLowerCase()}_api_key_encrypted`, encrypted);
    
    // 更新运行时配置
    if (type === 'TTS') {
      this.config.apiKey = apiKey;
    } else {
      this.sttConfig.apiKey = apiKey;
    }
  }

  // 加载订阅状态
  private loadSubscriptionStatus() {
    const subscription = localStorage.getItem('spokenly_subscription');
    if (subscription) {
      const subData = JSON.parse(subscription);
      this.isPro = subData.isPro;
      if (subData.expiresAt) {
        const expiryDate = new Date(subData.expiresAt);
        if (expiryDate > new Date()) {
          this.isPro = true;
        } else {
          this.isPro = false;
          localStorage.removeItem('spokenly_subscription');
        }
      }
    }
  }

  // 检查试用状态
  private checkTrialStatus() {
    const trialStart = localStorage.getItem('spokenly_trial_start');
    if (!trialStart) {
      // 首次使用，开始3天免费试用
      const startDate = new Date();
      localStorage.setItem('spokenly_trial_start', startDate.toISOString());
      this.trialEndDate = new Date(startDate.getTime() + 3 * 24 * 60 * 60 * 1000); // 3天后
    } else {
      const startDate = new Date(trialStart);
      this.trialEndDate = new Date(startDate.getTime() + 3 * 24 * 60 * 60 * 1000);
    }
  }

  // 检查是否可以使用服务
  public canUseService(): { allowed: boolean; reason?: string } {
    if (this.isPro) {
      return { allowed: true };
    }

    if (this.trialEndDate && new Date() < this.trialEndDate) {
      const daysLeft = Math.ceil((this.trialEndDate.getTime() - new Date().getTime()) / (24 * 60 * 60 * 1000));
      return { allowed: true, reason: `免费试用还剩 ${daysLeft} 天` };
    }

    return { allowed: false, reason: '试用期已结束，请升级到 Pro 版本' };
  }

  // 获取试用剩余时间
  public getTrialInfo() {
    if (this.isPro) {
      return { isPro: true, daysLeft: -1, message: 'Pro 版本' };
    }

    if (this.trialEndDate) {
      const now = new Date();
      if (now < this.trialEndDate) {
        const daysLeft = Math.ceil((this.trialEndDate.getTime() - now.getTime()) / (24 * 60 * 60 * 1000));
        const hoursLeft = Math.ceil((this.trialEndDate.getTime() - now.getTime()) / (60 * 60 * 1000));
        
        if (hoursLeft < 24) {
          return { isPro: false, daysLeft: 0, hoursLeft, message: `试用还剩 ${hoursLeft} 小时` };
        }
        
        return { isPro: false, daysLeft, message: `试用还剩 ${daysLeft} 天` };
      }
    }

    return { isPro: false, daysLeft: 0, message: '试用期已结束' };
  }

  // 文本转语音
  public async textToSpeech(text: string, options?: Partial<TTSConfig>): Promise<ArrayBuffer> {
    const serviceCheck = this.canUseService();
    if (!serviceCheck.allowed) {
      throw new Error(serviceCheck.reason);
    }

    const config = { ...this.config, ...options };
    
    try {
      const response = await fetch(`${config.baseUrl}/audio/speech`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${config.apiKey}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          model: config.model,
          input: text,
          voice: config.voice,
          response_format: config.responseFormat,
          speed: config.speed
        })
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`TTS API 错误: ${error}`);
      }

      return await response.arrayBuffer();
    } catch (error) {
      logger.error('TTS 错误', error);
      throw error;
    }
  }

  // 语音转文本（转录）
  public async speechToText(audioFile: File | Blob, options?: Partial<STTConfig>): Promise<{ text: string }> {
    const serviceCheck = this.canUseService();
    if (!serviceCheck.allowed) {
      throw new Error(serviceCheck.reason);
    }

    const config = { ...this.sttConfig, ...options };
    
    try {
      const formData = new FormData();
      formData.append('file', audioFile);
      formData.append('model', config.model);
      
      if (config.language) {
        formData.append('language', config.language);
      }
      if (config.prompt) {
        formData.append('prompt', config.prompt);
      }
      if (config.responseFormat) {
        formData.append('response_format', config.responseFormat);
      }
      if (config.temperature !== undefined) {
        formData.append('temperature', config.temperature.toString());
      }

      const response = await fetch(`${config.baseUrl}/audio/transcriptions`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${config.apiKey}`
        },
        body: formData
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`STT API 错误: ${error}`);
      }

      const result = await response.json();
      return result;
    } catch (error) {
      logger.error('STT 错误', error);
      throw error;
    }
  }

  // 音频翻译（翻译为英文）
  public async translateAudio(audioFile: File | Blob, options?: Partial<STTConfig>): Promise<{ text: string }> {
    const serviceCheck = this.canUseService();
    if (!serviceCheck.allowed) {
      throw new Error(serviceCheck.reason);
    }

    const config = { ...this.sttConfig, ...options };
    
    try {
      const formData = new FormData();
      formData.append('file', audioFile);
      formData.append('model', config.model);
      
      if (config.prompt) {
        formData.append('prompt', config.prompt);
      }
      if (config.responseFormat) {
        formData.append('response_format', config.responseFormat);
      }
      if (config.temperature !== undefined) {
        formData.append('temperature', config.temperature.toString());
      }

      const response = await fetch(`${config.baseUrl}/audio/translations`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${config.apiKey}`
        },
        body: formData
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`翻译 API 错误: ${error}`);
      }

      const result = await response.json();
      return result;
    } catch (error) {
      logger.error('翻译错误', error);
      throw error;
    }
  }

  // 升级到 Pro 版本
  public upgradeToPro(plan: 'monthly' | 'yearly') {
    const expiryDate = new Date();
    
    if (plan === 'monthly') {
      expiryDate.setMonth(expiryDate.getMonth() + 1);
    } else {
      expiryDate.setFullYear(expiryDate.getFullYear() + 1);
    }

    const subscription = {
      isPro: true,
      plan,
      startDate: new Date().toISOString(),
      expiresAt: expiryDate.toISOString()
    };

    localStorage.setItem('spokenly_subscription', JSON.stringify(subscription));
    this.isPro = true;
    
    return subscription;
  }

  // 更新配置
  public updateConfig(newConfig: Partial<TTSConfig>) {
    this.config = { ...this.config, ...newConfig };
  }

  // 更新 STT 配置
  public updateSTTConfig(newConfig: Partial<STTConfig>) {
    this.sttConfig = { ...this.sttConfig, ...newConfig };
  }

  // 获取当前配置
  public getConfig() {
    return { ...this.config };
  }

  // 获取 STT 配置
  public getSTTConfig() {
    return { ...this.sttConfig };
  }
}

// 导出单例
export const ttsService = new TTSService();
export default ttsService;