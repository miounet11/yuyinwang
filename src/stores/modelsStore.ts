import { create } from 'zustand';
import { TranscriptionModel, ModelConfig, DownloadTask } from '../types/models';
import { transcriptionModels } from '../data/models';

interface ModelsStore {
  // 模型列表
  models: TranscriptionModel[];
  selectedModelId: string;
  selectedCategory: string;
  searchQuery: string;
  
  // 配置
  modelConfigs: Record<string, ModelConfig>;
  
  // 下载任务
  downloadTasks: DownloadTask[];
  
  // Actions
  setSelectedModel: (modelId: string) => void;
  setSelectedCategory: (category: string) => void;
  setSearchQuery: (query: string) => void;
  updateModelConfig: (modelId: string, config: Partial<ModelConfig>) => void;
  saveModelConfig: (modelId: string, config: ModelConfig) => void;
  
  // 下载管理
  startDownload: (modelId: string) => void;
  pauseDownload: (modelId: string) => void;
  resumeDownload: (modelId: string) => void;
  cancelDownload: (modelId: string) => void;
  updateDownloadProgress: (modelId: string, progress: number, speed?: string, remaining?: string) => void;
  
  // 模型管理
  installModel: (modelId: string) => void;
  uninstallModel: (modelId: string) => void;
  updateModelStatus: (modelId: string, status: TranscriptionModel['status']) => void;
  
  // 获取过滤后的模型
  getFilteredModels: () => TranscriptionModel[];
}

export const useModelsStore = create<ModelsStore>((set, get) => ({
  models: transcriptionModels.map(model => ({ ...model })),
  selectedModelId: 'gpt-4o-mini',
  selectedCategory: 'all',
  searchQuery: '',
  modelConfigs: {},
  downloadTasks: [],
  
  setSelectedModel: (modelId) => set({ selectedModelId: modelId }),
  
  setSelectedCategory: (category) => set({ selectedCategory: category }),
  
  setSearchQuery: (query) => set({ searchQuery: query }),
  
  updateModelConfig: (modelId, config) => set((state) => ({
    modelConfigs: {
      ...state.modelConfigs,
      [modelId]: {
        ...state.modelConfigs[modelId],
        ...config,
        modelId
      }
    }
  })),
  
  saveModelConfig: (modelId, config) => set((state) => ({
    modelConfigs: {
      ...state.modelConfigs,
      [modelId]: config
    }
  })),
  
  startDownload: (modelId) => {
    const model = get().models.find(m => m.id === modelId);
    if (!model) return;
    
    // 更新模型状态
    set((state) => ({
      models: state.models.map(m => 
        m.id === modelId 
          ? { ...m, downloading: true, status: 'downloading' }
          : m
      ),
      downloadTasks: [
        ...state.downloadTasks.filter(t => t.modelId !== modelId),
        {
          modelId,
          progress: 0,
          speed: '0 MB/s',
          remaining: 'Calculating...',
          status: 'downloading'
        }
      ]
    }));
    
    // 模拟下载进度
    const interval = setInterval(() => {
      const task = get().downloadTasks.find(t => t.modelId === modelId);
      if (!task || task.status !== 'downloading') {
        clearInterval(interval);
        return;
      }
      
      const newProgress = Math.min(task.progress + Math.random() * 10, 100);
      const speed = `${(Math.random() * 10 + 1).toFixed(1)} MB/s`;
      const remaining = newProgress >= 100 ? 'Complete' : `${Math.floor((100 - newProgress) / 2)} seconds`;
      
      get().updateDownloadProgress(modelId, newProgress, speed, remaining);
      
      if (newProgress >= 100) {
        clearInterval(interval);
        get().installModel(modelId);
      }
    }, 1000);
  },
  
  pauseDownload: (modelId) => set((state) => ({
    downloadTasks: state.downloadTasks.map(t =>
      t.modelId === modelId ? { ...t, status: 'paused' } : t
    )
  })),
  
  resumeDownload: (modelId) => {
    set((state) => ({
      downloadTasks: state.downloadTasks.map(t =>
        t.modelId === modelId ? { ...t, status: 'downloading' } : t
      )
    }));
    get().startDownload(modelId);
  },
  
  cancelDownload: (modelId) => set((state) => ({
    models: state.models.map(m =>
      m.id === modelId 
        ? { ...m, downloading: false, status: 'available' }
        : m
    ),
    downloadTasks: state.downloadTasks.filter(t => t.modelId !== modelId)
  })),
  
  updateDownloadProgress: (modelId, progress, speed, remaining) => set((state) => ({
    downloadTasks: state.downloadTasks.map(t =>
      t.modelId === modelId 
        ? { 
            ...t, 
            progress, 
            speed: speed || t.speed, 
            remaining: remaining || t.remaining,
            status: progress >= 100 ? 'completed' : t.status
          }
        : t
    )
  })),
  
  installModel: (modelId) => set((state) => ({
    models: state.models.map(m =>
      m.id === modelId 
        ? { ...m, installed: true, downloading: false, status: 'installed' }
        : m
    ),
    downloadTasks: state.downloadTasks.filter(t => t.modelId !== modelId)
  })),
  
  uninstallModel: (modelId) => set((state) => ({
    models: state.models.map(m =>
      m.id === modelId 
        ? { ...m, installed: false, status: 'available' }
        : m
    )
  })),
  
  updateModelStatus: (modelId, status) => set((state) => ({
    models: state.models.map(m =>
      m.id === modelId ? { ...m, status } : m
    )
  })),
  
  getFilteredModels: () => {
    const state = get();
    let filtered = state.models;
    
    // 按分类筛选
    if (state.selectedCategory !== 'all') {
      filtered = filtered.filter(m => 
        m.category.includes(state.selectedCategory as any)
      );
    }
    
    // 按搜索词筛选
    if (state.searchQuery) {
      const query = state.searchQuery.toLowerCase();
      filtered = filtered.filter(m =>
        m.name.toLowerCase().includes(query) ||
        m.provider.toLowerCase().includes(query) ||
        m.description.toLowerCase().includes(query)
      );
    }
    
    return filtered;
  }
}));