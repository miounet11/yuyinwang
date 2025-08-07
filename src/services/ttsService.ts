// TTS 服务 - 集成 OpenAI TTS API
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
    // 默认配置 - 免费试用 API
    this.config = {
      apiKey: 'sk-vJToQKskNEIaFNM3GjTGh1YrN9kGZ33pw2D1AEZUXL0prLjw',
      baseUrl: 'https://ttkk.inping.com/v1',
      model: 'tts-1',
      voice: 'alloy',
      responseFormat: 'mp3',
      speed: 1.0
    };

    this.sttConfig = {
      apiKey: 'sk-vJToQKskNEIaFNM3GjTGh1YrN9kGZ33pw2D1AEZUXL0prLjw',
      baseUrl: 'https://ttkk.inping.com/v1',
      model: 'whisper-1',
      responseFormat: 'json',
      temperature: 0
    };

    this.loadSubscriptionStatus();
    this.checkTrialStatus();
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
      console.error('TTS 错误:', error);
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
      console.error('STT 错误:', error);
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
      console.error('翻译错误:', error);
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