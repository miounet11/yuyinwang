# 听写模型管理系统技术实现方案

## 概述

本方案为 Spokenly 听写应用设计了一个完整的模型管理系统，支持在线模型API配置、本地模型下载管理、模型性能评分显示以及分类筛选功能。

## 系统架构

### 1. 数据结构设计

#### 核心类型定义 (`/src/types/models.ts`)

```typescript
// 模型类型：在线、本地、API
export type ModelType = 'online' | 'local' | 'api';

// 分类标签：全部、在线、本地、API、快速、准确、标点符号、字幕
export type ModelCategory = 'all' | 'online' | 'local' | 'api' | 'fast' | 'accurate' | 'punctuation' | 'subtitle';

// 模型提供商
export type ModelProvider = 'openai' | 'deepgram' | 'mistral' | 'elevenlabs' | 'whisper' | 'local';

// API配置接口
export interface ApiConfig {
  apiKey: string;
  endpoint?: string;
  model?: string;
  language?: string;
  customSettings?: Record<string, any>;
}

// 扩展的模型接口
export interface TranscriptionModel {
  id: string;
  name: string;
  provider: ModelProvider;
  type: ModelType;
  description: string;
  
  // 性能指标（1-5星评分）
  performance: {
    accuracy: number;
    speed: number;
    realtime: boolean;
    languages: string[];
  };
  
  // 功能特性
  features: {
    punctuation: boolean;
    timestamps: boolean;
    speakerDiarization: boolean;
    streaming: boolean;
    customVocabulary: boolean;
  };
  
  // 配置信息
  apiConfig?: ApiConfig;
  localModel?: LocalModelInfo;
  
  // 显示属性
  icon: string;
  recommended: boolean;
  badge?: string;
  categories: ModelCategory[];
}
```

### 2. 数据管理 (`/src/data/models.ts`)

#### 预定义模型数据
- **在线模型**: GPT-4o mini、Nova-3、Voxtral Mini、ElevenLabs Scribe
- **本地模型**: Whisper Large v3、Whisper Medium、Whisper Small
- **分类配置**: 8个分类标签的配置和描述
- **工具函数**: 模型过滤、排序、搜索功能

### 3. 状态管理 (`/src/stores/modelsStore.ts`)

使用 Zustand 进行状态管理，支持：
- 模型选择和配置
- API配置持久化存储
- 本地模型下载状态追踪
- 分类筛选状态管理

### 4. 核心组件

#### 4.1 主要模型页面 (`TranscriptionModelsPage.tsx`)

**功能特性：**
- 模型卡片展示（性能评分、功能标签、配置状态）
- 分类标签筛选（8个分类）
- 搜索和排序功能
- 统计信息显示
- 响应式设计

**关键功能：**
```typescript
// 模型选择逻辑
const handleModelSelect = (model: TranscriptionModel) => {
  if (model.type === 'online' && !model.apiConfig?.apiKey) {
    // 未配置的在线模型，打开配置对话框
    setConfigModel(model);
  } else {
    // 直接选择模型
    onSelectModel(model.id);
  }
};
```

#### 4.2 API配置对话框 (`ModelConfig.tsx`)

**功能特性：**
- 针对不同提供商的专用配置界面
- API连接测试功能
- 实时验证和错误提示
- 配置保存和加载

**支持的提供商配置：**
- OpenAI: API密钥、模型版本、语言设置
- Deepgram: API密钥、模型版本、高级设置（标点、说话人分离）
- 通用配置: 基础API密钥配置

#### 4.3 本地模型管理器 (`LocalModelManager.tsx`)

**功能特性：**
- 本地模型列表展示
- 下载进度追踪和控制（暂停/恢复/取消）
- 存储空间信息显示
- 模型文件夹路径管理
- 已下载模型删除功能

**下载状态管理：**
```typescript
export interface DownloadStatus {
  modelId: string;
  progress: number;
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'paused';
  error?: string;
}
```

### 5. API集成 (`/src/utils/modelApi.ts`)

#### Tauri后端集成功能：
- API连接测试
- 配置保存和加载
- 本地模型下载管理
- 存储信息获取
- 实时事件监听（下载进度、完成、错误）
- 模型转录接口

#### 关键API方法：
```typescript
// 测试API连接
static async testApiConnection(modelId: string, config: ApiConfig)

// 下载本地模型
static async downloadLocalModel(modelId: string)

// 设置下载事件监听
static setupDownloadListeners(
  onProgress: (modelId: string, progress: number) => void,
  onComplete: (modelId: string) => void,
  onError: (modelId: string, error: string) => void
)

// 使用模型进行转录
static async transcribeWithModel(modelId: string, audioData: ArrayBuffer, options?)
```

## 界面设计特点

### 1. 模型卡片设计
- **性能评分**: 5星点状评分系统（准确度、速度）
- **功能标签**: 支持的特性标签（标点符号、时间戳、说话人分离等）
- **状态指示**: 配置状态、下载状态、推荐标签
- **交互反馈**: 悬停效果、选中状态、禁用状态

### 2. 分类筛选系统
- **8个分类标签**: 全部、在线、本地、API、快速、准确、标点符号、字幕
- **动态计数**: 每个分类显示模型数量
- **图标标识**: 每个分类有专用图标
- **响应式布局**: 移动端横向滚动

### 3. 搜索和排序
- **实时搜索**: 支持模型名称、描述、提供商搜索
- **多重排序**: 默认、准确度、速度排序
- **结果统计**: 显示搜索结果数量

## 技术实现亮点

### 1. 类型安全
- 完整的 TypeScript 类型定义
- 严格的接口约束
- 泛型支持

### 2. 状态管理
- Zustand 轻量级状态管理
- 持久化存储（本地配置保存）
- 响应式状态更新

### 3. 用户体验
- 加载状态管理
- 错误处理和提示
- 渐进式功能展示
- 响应式设计

### 4. 可扩展性
- 模块化组件设计
- 插件式模型添加
- 配置化分类系统
- 抽象API层

## 部署和集成

### 1. 文件结构
```
src/
├── types/models.ts              # 类型定义
├── data/models.ts              # 模型数据
├── stores/modelsStore.ts       # 状态管理
├── utils/modelApi.ts           # API集成
└── components/
    ├── TranscriptionModelsPage.tsx  # 主页面
    ├── ModelConfig.tsx              # 配置对话框
    ├── LocalModelManager.tsx        # 本地模型管理
    └── *.css                        # 样式文件
```

### 2. 依赖项
- React 18+
- TypeScript
- Zustand (状态管理)
- Tauri (桌面应用框架)
- CSS变量系统

### 3. 集成步骤
1. 添加新的类型定义和数据文件
2. 创建状态管理store
3. 实现核心组件
4. 集成到主应用中
5. 添加Tauri后端API支持

## 后续优化建议

### 1. 功能增强
- 模型性能基准测试
- 自定义模型添加
- 模型使用统计
- 配置导入/导出

### 2. 用户体验
- 模型推荐算法
- 使用历史分析
- 智能配置建议
- 批量操作支持

### 3. 技术优化
- 虚拟滚动（大量模型时）
- WebWorker后台处理
- 增量更新机制
- 错误恢复机制

这个听写模型管理系统提供了完整的模型选择、配置和管理功能，支持多种模型类型和提供商，具有良好的用户体验和可扩展性。