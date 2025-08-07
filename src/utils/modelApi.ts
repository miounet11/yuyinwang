import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { ApiConfig } from '../types/models';

// 模型管理API
export class ModelAPI {
  // 测试API连接
  static async testApiConnection(modelId: string, config: ApiConfig): Promise<{ success: boolean; message: string }> {
    try {
      const result = await invoke<{ success: boolean; message: string }>('test_api_connection', {
        modelId,
        config
      });
      return result;
    } catch (error) {
      return {
        success: false,
        message: `连接测试失败: ${error}`
      };
    }
  }

  // 保存API配置
  static async saveApiConfig(modelId: string, config: ApiConfig): Promise<void> {
    try {
      await invoke('save_api_config', { modelId, config });
    } catch (error) {
      throw new Error(`保存配置失败: ${error}`);
    }
  }

  // 加载API配置
  static async loadApiConfig(modelId: string): Promise<ApiConfig | null> {
    try {
      const config = await invoke<ApiConfig | null>('load_api_config', { modelId });
      return config;
    } catch (error) {
      console.error('加载配置失败:', error);
      return null;
    }
  }

  // 获取所有已保存的API配置
  static async loadAllApiConfigs(): Promise<Record<string, ApiConfig>> {
    try {
      const configs = await invoke<Record<string, ApiConfig>>('load_all_api_configs');
      return configs;
    } catch (error) {
      console.error('加载所有配置失败:', error);
      return {};
    }
  }

  // 开始下载本地模型
  static async downloadLocalModel(modelId: string): Promise<void> {
    try {
      await invoke('download_local_model', { modelId });
    } catch (error) {
      throw new Error(`开始下载失败: ${error}`);
    }
  }

  // 暂停下载
  static async pauseDownload(modelId: string): Promise<void> {
    try {
      await invoke('pause_model_download', { modelId });
    } catch (error) {
      throw new Error(`暂停下载失败: ${error}`);
    }
  }

  // 恢复下载
  static async resumeDownload(modelId: string): Promise<void> {
    try {
      await invoke('resume_model_download', { modelId });
    } catch (error) {
      throw new Error(`恢复下载失败: ${error}`);
    }
  }

  // 取消下载
  static async cancelDownload(modelId: string): Promise<void> {
    try {
      await invoke('cancel_model_download', { modelId });
    } catch (error) {
      throw new Error(`取消下载失败: ${error}`);
    }
  }

  // 删除本地模型
  static async deleteLocalModel(modelId: string): Promise<void> {
    try {
      await invoke('delete_local_model', { modelId });
    } catch (error) {
      throw new Error(`删除模型失败: ${error}`);
    }
  }

  // 获取本地模型信息
  static async getLocalModelInfo(modelId: string): Promise<{
    exists: boolean;
    size: number;
    path: string;
  }> {
    try {
      const info = await invoke<{
        exists: boolean;
        size: number;
        path: string;
      }>('get_local_model_info', { modelId });
      return info;
    } catch (error) {
      return {
        exists: false,
        size: 0,
        path: ''
      };
    }
  }

  // 设置本地模型存储路径
  static async setLocalModelsPath(path: string): Promise<void> {
    try {
      await invoke('set_local_models_path', { path });
    } catch (error) {
      throw new Error(`设置路径失败: ${error}`);
    }
  }

  // 获取本地模型存储路径
  static async getLocalModelsPath(): Promise<string> {
    try {
      const path = await invoke<string>('get_local_models_path');
      return path;
    } catch (error) {
      console.error('获取路径失败:', error);
      return '';
    }
  }

  // 打开模型文件夹
  static async openModelFolder(): Promise<void> {
    try {
      await invoke('open_model_folder');
    } catch (error) {
      throw new Error(`打开文件夹失败: ${error}`);
    }
  }

  // 获取存储空间信息
  static async getStorageInfo(): Promise<{
    totalSpace: number;
    usedSpace: number;
    availableSpace: number;
  }> {
    try {
      const info = await invoke<{
        totalSpace: number;
        usedSpace: number;
        availableSpace: number;
      }>('get_storage_info');
      return info;
    } catch (error) {
      return {
        totalSpace: 0,
        usedSpace: 0,
        availableSpace: 0
      };
    }
  }

  // 获取已下载的模型列表
  static async getDownloadedModels(): Promise<string[]> {
    try {
      const models = await invoke<string[]>('get_downloaded_models');
      return models;
    } catch (error) {
      console.error('获取已下载模型失败:', error);
      return [];
    }
  }

  // 设置下载事件监听器
  static setupDownloadListeners(
    onProgress: (modelId: string, progress: number) => void,
    onComplete: (modelId: string) => void,
    onError: (modelId: string, error: string) => void
  ): Promise<() => void> {
    return new Promise(async (resolve) => {
      const unlisteners: Array<() => void> = [];

      // 下载进度事件
      const unlistenProgress = await listen<{ modelId: string; progress: number }>(
        'model_download_progress',
        (event) => {
          onProgress(event.payload.modelId, event.payload.progress);
        }
      );
      unlisteners.push(unlistenProgress);

      // 下载完成事件
      const unlistenComplete = await listen<{ modelId: string }>(
        'model_download_complete',
        (event) => {
          onComplete(event.payload.modelId);
        }
      );
      unlisteners.push(unlistenComplete);

      // 下载错误事件
      const unlistenError = await listen<{ modelId: string; error: string }>(
        'model_download_error',
        (event) => {
          onError(event.payload.modelId, event.payload.error);
        }
      );
      unlisteners.push(unlistenError);

      // 返回清理函数
      resolve(() => {
        unlisteners.forEach(unlisten => unlisten());
      });
    });
  }

  // 使用指定模型进行转录
  static async transcribeWithModel(
    modelId: string,
    audioData: ArrayBuffer,
    options?: {
      language?: string;
      enablePunctuation?: boolean;
      enableTimestamps?: boolean;
      enableSpeakerDiarization?: boolean;
    }
  ): Promise<{
    text: string;
    confidence: number;
    segments?: Array<{
      start: number;
      end: number;
      text: string;
      speaker?: string;
    }>;
  }> {
    try {
      const result = await invoke<{
        text: string;
        confidence: number;
        segments?: Array<{
          start: number;
          end: number;
          text: string;
          speaker?: string;
        }>;
      }>('transcribe_with_model', {
        modelId,
        audioData: Array.from(new Uint8Array(audioData)),
        options: options || {}
      });
      return result;
    } catch (error) {
      throw new Error(`转录失败: ${error}`);
    }
  }

  // 流式转录（实时）
  static async startStreamingTranscription(
    modelId: string,
    onResult: (text: string, isFinal: boolean) => void,
    options?: {
      language?: string;
      enablePunctuation?: boolean;
    }
  ): Promise<() => void> {
    try {
      await invoke('start_streaming_transcription', {
        modelId,
        options: options || {}
      });

      const unlisten = await listen<{
        text: string;
        isFinal: boolean;
      }>('streaming_transcription_result', (event) => {
        onResult(event.payload.text, event.payload.isFinal);
      });

      return unlisten;
    } catch (error) {
      throw new Error(`开始流式转录失败: ${error}`);
    }
  }

  // 停止流式转录
  static async stopStreamingTranscription(): Promise<void> {
    try {
      await invoke('stop_streaming_transcription');
    } catch (error) {
      throw new Error(`停止流式转录失败: ${error}`);
    }
  }
}