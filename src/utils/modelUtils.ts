// 模型工具函数 - 从App.tsx中提取的性能优化版本
import { transcriptionModels } from '../data/models';
import logger from './logger';

export interface ModelInfo {
  model: string;
  modelType: 'local' | 'online';
}

// 缓存模型信息以提高性能
const modelCache = new Map<string, ModelInfo>();

export const getModelInfo = (modelId: string): ModelInfo => {
  // 先检查缓存
  if (modelCache.has(modelId)) {
    return modelCache.get(modelId)!;
  }

  logger.debug('查找模型ID', modelId);
  const model = transcriptionModels.find(m => m.id === modelId);
  logger.debug('找到的模型', model ? `${model.name} (type: ${model.type})` : 'null');
  
  const result: ModelInfo = {
    model: modelId,
    modelType: model?.type || 'online'
  };
  
  // 缓存结果
  modelCache.set(modelId, result);
  
  logger.debug('返回结果', result);
  return result;
};

// 获取所有可用模型
export const getAvailableModels = () => {
  return transcriptionModels.map(model => ({
    id: model.id,
    name: model.name,
    type: model.type,
    description: model.description,
    provider: model.provider
  }));
};

// 获取特定类型的模型
export const getModelsByType = (type: 'local' | 'online') => {
  return transcriptionModels.filter(model => model.type === type);
};

// 获取推荐模型
export const getRecommendedModels = () => {
  return transcriptionModels.filter(model => model.recommended);
};

// 检查模型是否为本地模型
export const isLocalModel = (modelId: string): boolean => {
  return getModelInfo(modelId).modelType === 'local';
};

// 检查模型是否支持实时转录
export const isRealtimeModel = (modelId: string): boolean => {
  const model = transcriptionModels.find(m => m.id === modelId);
  return model?.realtime || false;
};

// 获取模型的语言支持
export const getModelLanguages = (modelId: string): string[] => {
  const model = transcriptionModels.find(m => m.id === modelId);
  return model?.languages || ['多语言'];
};

// 清除模型缓存（在模型配置更新时使用）
export const clearModelCache = () => {
  modelCache.clear();
};