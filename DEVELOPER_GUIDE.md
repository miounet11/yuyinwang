# Recording King - 开发者指南

> Spokenly UI 重设计后的开发指南
> 更新时间: 2026-02-24

## 📚 目录

1. [项目概览](#项目概览)
2. [技术栈](#技术栈)
3. [项目结构](#项目结构)
4. [核心概念](#核心概念)
5. [组件开发](#组件开发)
6. [状态管理](#状态管理)
7. [样式规范](#样式规范)
8. [常见任务](#常见任务)
9. [调试技巧](#调试技巧)
10. [最佳实践](#最佳实践)

---

## 项目概览

Recording King 是一个基于 Tauri 的桌面应用，提供 AI 驱动的语音转文字功能。前端采用 React + TypeScript，后端使用 Rust。

### 核心功能

- 🎙️ 实时语音录制与转录
- 🤖 多模型支持（在线/本地/API）
- ⌨️ 全局快捷键控制
- 📝 智能文本注入
- 🎯 AI 提示自动化
- 📁 音视频文件转录

---

## 技术栈

### 前端

- **框架**: React 18.2
- **语言**: TypeScript 5.0
- **状态管理**: Zustand 4.4
- **构建工具**: Vite 5.0
- **样式**: CSS Modules + CSS Variables
- **桌面框架**: Tauri 1.5

### 后端

- **语言**: Rust 2021
- **音频处理**: cpal, hound, whisper-rs
- **数据库**: SQLite (rusqlite)
- **异步运行时**: tokio

---

## 项目结构

```
recording-king/
├── src/
│   ├── shared/              # 共享资源
│   │   ├── types.ts         # 全局类型定义
│   │   ├── utils.ts         # 工具函数库
│   │   ├── stores/          # Zustand 状态管理
│   │   │   └── useAppStore.ts
│   │   └── components/      # 共享组件
│   │       ├── Toast.tsx
│   │       └── icons/       # SVG 图标组件
│   ├── features/            # 功能模块
│   │   ├── settings/        # 常规设置
│   │   │   ├── GeneralSettings.tsx
│   │   │   └── PermissionsPage.tsx
│   │   ├── shortcuts/       # 快捷键设置
│   │   │   ├── ShortcutSettings.tsx
│   │   │   └── RecordShortcutModal.tsx
│   │   ├── models/          # 模型管理
│   │   │   ├── ModelSettings.tsx
│   │   │   └── WordReplacePanel.tsx
│   │   ├── transcribe/      # 文件转录
│   │   │   └── TranscribeFilePage.tsx
│   │   ├── ai-prompts/      # AI 提示
│   │   │   ├── AIPromptsPage.tsx
│   │   │   └── EditPromptModal.tsx
│   │   ├── onboarding/      # 入门引导
│   │   │   └── OnboardingPage.tsx
│   │   ├── history/         # 历史记录
│   │   ├── recording/       # 语音输入
│   │   └── ...
│   ├── App.tsx              # 根组件
│   ├── App.css              # 全局样式
│   └── main.tsx             # 入口文件
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/        # Tauri 命令
│   │   └── core/            # 核心功能
│   └── Cargo.toml
└── package.json
```

---

## 核心概念

### 1. 类型系统

所有类型定义集中在 `src/shared/types.ts`：

```typescript
import type { Page, NavItem, ModelCardData, AIPrompt } from '@/shared/types';
```

**核心类型**:
- `Page` - 页面路由类型
- `AppSettings` - 应用设置
- `ModelCardData` - 模型卡片数据
- `AIPrompt` - AI 提示配置
- `CustomShortcut` - 自定义快捷键

### 2. 工具函数

工具函数库位于 `src/shared/utils.ts`：

```typescript
import { validateCustomShortcut, filterModels, escapeHtml } from '@/shared/utils';

// 快捷键验证
const result = validateCustomShortcut(modifiers, key);

// 模型筛选
const filtered = filterModels(models, activeFilters);

// XSS 转义
const safe = escapeHtml(userInput);
```

### 3. 状态管理

使用 Zustand 进行全局状态管理：

```typescript
import { useAppStore } from '@/shared/stores/useAppStore';

function MyComponent() {
  const { settings, addToast, setShortcutPreset } = useAppStore();

  const handleSave = async () => {
    await setShortcutPreset('right-cmd');
    addToast('success', '保存成功');
  };
}
```

**核心状态**:
- `settings` - 应用设置
- `aiPrompts` - AI 提示列表
- `wordReplacements` - 词替换规则
- `onboardingState` - 入门引导状态

**核心操作**:
- `addToast(type, message)` - 显示通知
- `setShortcutPreset(preset)` - 设置快捷键
- `addAIPrompt(prompt)` - 添加 AI 提示
- `completeOnboarding()` - 完成入门引导

---

## 组件开发

### 创建新组件

1. **选择合适的目录**
   - 功能特定组件 → `src/features/[feature]/`
   - 共享组件 → `src/shared/components/`

2. **组件模板**

```typescript
import React, { useState } from 'react';
import { useAppStore } from '@/shared/stores/useAppStore';
import type { MyDataType } from '@/shared/types';
import './MyComponent.css';

interface MyComponentProps {
  title: string;
  onSave?: (data: MyDataType) => void;
}

export const MyComponent: React.FC<MyComponentProps> = ({ title, onSave }) => {
  const { addToast } = useAppStore();
  const [data, setData] = useState<MyDataType | null>(null);

  const handleSubmit = () => {
    if (!data) return;
    onSave?.(data);
    addToast('success', '保存成功');
  };

  return (
    <div className="my-component">
      <h2>{title}</h2>
      {/* 组件内容 */}
    </div>
  );
};
```

3. **样式文件**

```css
/* MyComponent.css */
.my-component {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 20px;
}

.my-component h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 16px;
}
```

### 使用图标

```typescript
import { SettingsIcon, ModelIcon } from '@/shared/components/icons';

<SettingsIcon size={24} className="my-icon" />
<ModelIcon /> {/* 默认 size=20 */}
```

### 调用 Tauri 命令

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 获取设置
const settings = await invoke<AppSettings>('get_settings');

// 更新设置
await invoke('update_settings', { settings: newSettings });

// 转录文件
const result = await invoke<{ text: string }>('transcribe_file', {
  filePath: '/path/to/file.mp3',
  model: 'whisper-base',
});
```

---

## 状态管理

### 读取状态

```typescript
const { settings, aiPrompts, isRecording } = useAppStore();
```

### 更新状态

```typescript
const { setSettings, addAIPrompt, setRecording } = useAppStore();

// 直接更新
setRecording(true);

// 异步更新（自动调用 Tauri API）
await setShortcutPreset('right-cmd');
await addAIPrompt(newPrompt);
```

### 订阅状态变化

```typescript
import { useAppStore } from '@/shared/stores/useAppStore';

function MyComponent() {
  // 只订阅需要的状态
  const isRecording = useAppStore((state) => state.isRecording);

  return <div>{isRecording ? '录音中...' : '未录音'}</div>;
}
```

---

## 样式规范

### CSS 变量

使用 CSS 变量保持主题一致性：

```css
/* 背景色 */
background: var(--bg-primary);    /* 主背景 */
background: var(--bg-secondary);  /* 次级背景 */
background: var(--bg-card);       /* 卡片背景 */
background: var(--bg-hover);      /* 悬停背景 */

/* 文字颜色 */
color: var(--text-primary);       /* 主文字 */
color: var(--text-secondary);     /* 次级文字 */
color: var(--text-muted);         /* 弱化文字 */

/* 主题色 */
color: var(--accent);             /* 强调色 */
color: var(--accent-hover);       /* 强调色悬停 */
background: var(--accent-light);  /* 强调色浅色背景 */

/* 状态色 */
color: var(--success);            /* 成功 */
color: var(--danger);             /* 危险 */
color: var(--warning);            /* 警告 */

/* 边框 */
border: 1px solid var(--border);
```

### 布局模式

**页面布局**:
```css
.page {
  padding: 40px;
  max-width: 800px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 32px;
}
```

**卡片布局**:
```css
.card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 20px;
}
```

**按钮样式**:
```css
.btn-primary {
  padding: 10px 20px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-primary:hover {
  background: var(--accent-hover);
}
```

---

## 常见任务

### 添加新页面

1. **创建页面组件**
```typescript
// src/features/my-feature/MyPage.tsx
export const MyPage: React.FC = () => {
  return (
    <div className="page">
      <h1 className="page-title">我的页面</h1>
      <p className="page-desc">页面描述</p>
    </div>
  );
};
```

2. **添加到类型定义**
```typescript
// src/shared/types.ts
export type Page =
  | 'general'
  | 'my-feature'  // 新增
  | ...;
```

3. **更新导航**
```typescript
// src/App.tsx
import { MyPage } from './features/my-feature/MyPage';

const NAV_ITEMS: NavItem[] = [
  ...,
  { key: 'my-feature', icon: <MyIcon />, label: '我的功能' },
];

// 添加路由
{currentPage === 'my-feature' && <MyPage />}
```

### 添加新的状态

```typescript
// src/shared/stores/useAppStore.ts
interface AppStore {
  // 新增状态
  myData: MyDataType[];

  // 新增操作
  addMyData: (data: MyDataType) => void;
  updateMyData: (id: string, updates: Partial<MyDataType>) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  // 初始状态
  myData: [],

  // 操作实现
  addMyData: (data) => set((state) => ({
    myData: [...state.myData, data],
  })),

  updateMyData: (id, updates) => set((state) => ({
    myData: state.myData.map((item) =>
      item.id === id ? { ...item, ...updates } : item
    ),
  })),
}));
```

### 添加新的 Tauri 命令

1. **Rust 后端**
```rust
// src-tauri/src/commands/my_command.rs
#[tauri::command]
pub async fn my_command(param: String) -> Result<String, String> {
    // 实现逻辑
    Ok(format!("Result: {}", param))
}
```

2. **注册命令**
```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            my_command,
            // ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

3. **前端调用**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke<string>('my_command', { param: 'test' });
```

---

## 调试技巧

### 前端调试

1. **React DevTools**
   - 安装 React DevTools 浏览器扩展
   - 在 Tauri 窗口中打开开发者工具（F12）

2. **状态调试**
```typescript
// 临时添加日志
const store = useAppStore();
console.log('Current state:', store);
```

3. **Zustand DevTools**
```typescript
import { devtools } from 'zustand/middleware';

export const useAppStore = create<AppStore>()(devtools((set) => ({
  // ...
})));
```

### 后端调试

1. **Rust 日志**
```rust
use log::{info, warn, error};

info!("Processing file: {}", file_path);
warn!("Low memory warning");
error!("Failed to load model: {}", err);
```

2. **查看日志**
```bash
# 开发模式
cargo tauri dev

# 查看 Rust 日志
RUST_LOG=debug cargo tauri dev
```

---

## 最佳实践

### 1. 类型安全

✅ **推荐**:
```typescript
import type { ModelCardData } from '@/shared/types';

const model: ModelCardData = {
  id: 'whisper-base',
  name: 'Whisper Base',
  // TypeScript 会检查所有必需字段
};
```

❌ **避免**:
```typescript
const model: any = { id: 'whisper-base' }; // 失去类型检查
```

### 2. 错误处理

✅ **推荐**:
```typescript
try {
  await invoke('risky_operation');
  addToast('success', '操作成功');
} catch (error) {
  addToast('error', `操作失败: ${error}`);
  console.error('Operation failed:', error);
}
```

❌ **避免**:
```typescript
await invoke('risky_operation'); // 未处理错误
```

### 3. 性能优化

✅ **推荐**:
```typescript
// 只订阅需要的状态
const isRecording = useAppStore((state) => state.isRecording);

// 使用 useMemo 缓存计算结果
const filteredModels = useMemo(
  () => filterModels(models, activeFilters),
  [models, activeFilters]
);
```

❌ **避免**:
```typescript
// 订阅整个 store（导致不必要的重渲染）
const store = useAppStore();

// 每次渲染都重新计算
const filteredModels = filterModels(models, activeFilters);
```

### 4. 组件职责

✅ **推荐**: 单一职责原则
```typescript
// 专注于展示
const ModelCard: React.FC<{ model: ModelCardData }> = ({ model }) => {
  return <div className="model-card">{model.name}</div>;
};

// 专注于逻辑
const ModelList: React.FC = () => {
  const [models, setModels] = useState([]);
  // 数据获取和状态管理
  return models.map((m) => <ModelCard key={m.id} model={m} />);
};
```

❌ **避免**: 混合职责
```typescript
const ModelCard: React.FC = () => {
  // 既负责数据获取，又负责展示
  const [models, setModels] = useState([]);
  useEffect(() => { /* fetch */ }, []);
  return <div>{/* render */}</div>;
};
```

### 5. 样式组织

✅ **推荐**: 使用 CSS 变量
```css
.my-button {
  background: var(--accent);
  color: #fff;
}

.my-button:hover {
  background: var(--accent-hover);
}
```

❌ **避免**: 硬编码颜色
```css
.my-button {
  background: #3b82f6; /* 难以维护 */
  color: #fff;
}
```

---

## 常见问题

### Q: 如何添加新的 CSS 变量？

A: 在 `src/App.css` 的 `:root` 中添加：
```css
:root {
  --my-custom-color: #ff6b6b;
}
```

### Q: 如何处理异步状态更新？

A: 使用 async/await 和错误处理：
```typescript
const handleSave = async () => {
  try {
    await setShortcutPreset('right-cmd');
    addToast('success', '保存成功');
  } catch (error) {
    addToast('error', '保存失败');
  }
};
```

### Q: 如何调试 Tauri 命令？

A: 在 Rust 代码中添加日志：
```rust
use log::info;

#[tauri::command]
pub fn my_command() {
    info!("Command called");
}
```

然后运行：
```bash
RUST_LOG=debug cargo tauri dev
```

---

## 资源链接

- [React 文档](https://react.dev/)
- [TypeScript 文档](https://www.typescriptlang.org/docs/)
- [Zustand 文档](https://docs.pmnd.rs/zustand/getting-started/introduction)
- [Tauri 文档](https://tauri.app/)
- [Vite 文档](https://vitejs.dev/)

---

*最后更新: 2026-02-24*
