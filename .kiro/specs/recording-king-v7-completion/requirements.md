# 需求文档：Recording King v7.0 完善与修复

## 简介

Recording King v7.0 是一款基于 Tauri 1.5 (Rust) + React 18 + TypeScript 的 macOS 桌面语音转录应用。项目已从 v6.x（150+ 文件，10000+ 行）重构至 v7.0（28 文件，~2000 行），代码可编译、前端可构建，但存在多个关键缺陷需要修复，同时需要补充缺失功能以达到生产可用状态。

本需求文档覆盖：关键缺陷修复、功能完善、构建修复、错误处理、测试覆盖、代码清理。

## 术语表

- **App**: Recording King v7.0 桌面应用程序
- **Backend**: 基于 Tauri 1.5 / Rust 的后端进程（src-tauri/）
- **Frontend**: 基于 React 18 / TypeScript 的前端界面（src/）
- **QuickInputService**: 快速语音输入服务，负责"按住录音、松开转录"的核心流程
- **ShortcutManager**: 全局快捷键管理器，负责检测按键按下和释放事件
- **AppState**: 应用全局状态管理结构体，包含设置、数据库连接、录音状态
- **AppSettings**: 应用设置数据结构，包含 API Key、模型选择、注入配置等
- **Database**: SQLite 数据库服务，负责转录历史的持久化存储
- **TranscriptionService**: OpenAI Whisper API 转录服务
- **InjectionModule**: macOS Core Graphics 文本注入模块
- **FloatingWindow**: 快速输入悬浮状态窗口（quick-input.html）
- **ErrorBoundary**: React 错误边界组件，用于捕获和展示前端错误
- **CGEventTap**: macOS Core Graphics 事件监听 API，可检测按键释放事件
- **rdev**: Rust 跨平台输入设备监听库，支持按键按下和释放事件检测

## 需求

### 需求 1：按键释放检测（关键缺陷修复）

**用户故事：** 作为用户，我希望按住全局快捷键时开始录音、松开时自动停止并转录，以便实现流畅的"按住说话"语音输入体验。

**背景：** 当前 `register_global_shortcut` 使用 Tauri 的 GlobalShortcut API，该 API 仅支持 key-down 回调，不支持 key-up 事件检测。核心的"按住录音、松开转录"流程无法工作。需要使用 rdev 库或 macOS CGEventTap 来实现按键释放检测。

#### 验收标准

1. WHEN 用户按下已注册的全局快捷键, THE ShortcutManager SHALL 通过 rdev 或 CGEventTap 检测到 key-down 事件并触发录音开始
2. WHEN 用户释放已注册的全局快捷键, THE ShortcutManager SHALL 通过 rdev 或 CGEventTap 检测到 key-up 事件并触发录音停止和转录流程
3. WHILE 全局快捷键处于按下状态, THE FloatingWindow SHALL 保持可见并显示录音状态
4. WHEN 按键按下持续时间少于 200 毫秒, THE ShortcutManager SHALL 将该事件识别为短按并忽略（防止误触）
5. WHEN 按键按下持续时间达到或超过 200 毫秒, THE ShortcutManager SHALL 将该事件识别为有效长按并执行转录流程
6. IF rdev 或 CGEventTap 监听线程异常终止, THEN THE Backend SHALL 记录错误日志并通知 Frontend 快捷键监听已失效
7. THE ShortcutManager SHALL 在应用退出时正确释放 rdev 或 CGEventTap 监听资源

### 需求 2：设置持久化

**用户故事：** 作为用户，我希望应用设置（API Key、模型选择、注入配置等）在重启后仍然保留，以便不需要每次启动都重新配置。

**背景：** 当前 `AppSettings` 仅存储在内存中（`Arc<Mutex<AppSettings>>`），应用重启后所有设置丢失，包括 OpenAI API Key。需要将设置持久化到 SQLite 数据库或配置文件。

#### 验收标准

1. WHEN 用户修改并保存设置, THE Backend SHALL 将 AppSettings 持久化到 SQLite 数据库或本地配置文件
2. WHEN App 启动时, THE Backend SHALL 从持久化存储加载已保存的 AppSettings 并应用到 AppState
3. IF 持久化存储不存在或损坏, THEN THE Backend SHALL 使用默认 AppSettings 并记录警告日志
4. THE Backend SHALL 对 OpenAI API Key 进行加密或混淆存储，避免明文保存在磁盘上
5. WHEN 用户保存设置成功, THE Frontend SHALL 显示保存成功的提示信息
6. WHEN 用户保存设置失败, THE Frontend SHALL 显示具体的错误信息

### 需求 3：macOS 构建修复

**用户故事：** 作为开发者，我希望项目能够成功构建 macOS DMG 安装包，以便分发给用户使用。

**背景：** tauri.conf.json 引用了 `icons/icon.icns` 但该文件不存在，导致 DMG 构建失败。此外存在 11 个编译警告需要清理。

#### 验收标准

1. THE App SHALL 在 `src-tauri/icons/` 目录中包含有效的 `icon.icns` 文件
2. THE Backend SHALL 编译时产生零个编译警告（清理所有未使用的导入、变量和死代码）
3. WHEN 执行 `cargo tauri build` 命令, THE App SHALL 成功生成 macOS DMG 安装包
4. THE App SHALL 在 tauri.conf.json 中引用的所有图标文件均存在且格式有效

### 需求 4：前端错误处理

**用户故事：** 作为用户，我希望在操作失败时看到清晰的错误提示，而不是应用无响应或崩溃，以便了解问题并采取相应措施。

**背景：** 当前 Tauri command 调用失败仅记录到 console.log，用户无法感知错误。缺少 React ErrorBoundary，未捕获的异常会导致白屏。

#### 验收标准

1. THE Frontend SHALL 在应用根组件外层包裹 ErrorBoundary 组件
2. WHEN ErrorBoundary 捕获到未处理的渲染异常, THE Frontend SHALL 显示友好的错误回退界面，包含错误描述和重试按钮
3. WHEN Tauri command 调用失败, THE Frontend SHALL 在界面上显示用户可理解的错误提示（非 console.log）
4. WHEN 录音启动失败, THE Frontend SHALL 显示具体的失败原因（如设备不可用、权限不足）
5. WHEN 转录请求失败, THE Frontend SHALL 显示具体的失败原因（如 API Key 无效、网络错误、余额不足）
6. WHEN 文本注入失败, THE Frontend SHALL 显示失败原因（如辅助功能权限未授权）
7. IF API Key 未设置且用户尝试转录, THEN THE Frontend SHALL 提示用户先在设置页面配置 API Key


### 需求 5：加载状态管理

**用户故事：** 作为用户，我希望在应用初始化、录音、转录等耗时操作期间看到明确的加载指示，以便了解应用当前状态。

**背景：** 当前缺少统一的加载状态管理，应用初始化、录音中、转录中等状态没有清晰的视觉反馈。

#### 验收标准

1. WHILE App 正在初始化（加载设置、历史记录、音频设备）, THE Frontend SHALL 显示全局加载指示器
2. WHILE 录音正在进行, THE Frontend SHALL 显示录音时长计时器和音频波形或脉动动画
3. WHILE 转录请求正在处理, THE Frontend SHALL 显示转录进度指示器并禁用录音按钮
4. WHEN 初始化完成, THE Frontend SHALL 隐藏全局加载指示器并显示主界面
5. IF 初始化失败, THEN THE Frontend SHALL 显示错误信息和重试按钮

### 需求 6：基础测试覆盖

**用户故事：** 作为开发者，我希望项目具备基础的自动化测试，以便在修改代码时能够快速验证核心功能未被破坏。

**背景：** 当前项目零测试覆盖，没有任何单元测试或集成测试。

#### 验收标准

1. THE Backend SHALL 包含 Database 模块的单元测试，覆盖 CRUD 操作和搜索功能
2. THE Backend SHALL 包含 AppSettings 序列化和反序列化的单元测试
3. THE Backend SHALL 包含 ShortcutManager 按键状态管理的单元测试
4. THE Backend SHALL 包含 AudioRecorder 配置验证的单元测试
5. THE Frontend SHALL 包含 useAppStore 状态管理的单元测试，覆盖所有 action
6. THE Frontend SHALL 包含各页面组件的基础渲染测试
7. FOR ALL 有效的 AppSettings 对象, 序列化后再反序列化 SHALL 产生等价的对象（往返属性）

### 需求 7：代码清理

**用户故事：** 作为开发者，我希望项目代码库整洁、无冗余文件，以便降低维护成本和新开发者的上手难度。

**背景：** 项目中存在多处冗余：`old/` 目录包含 150+ 文件的遗留代码、`src/data/localModels.ts` 未被使用、`src/shared/components/`、`src/shared/hooks/`、`src/styles/` 为空目录、根目录存在大量遗留文档和测试脚本。

#### 验收标准

1. THE App SHALL 删除 `old/` 目录及其所有内容
2. THE App SHALL 删除或整合 `src/data/localModels.ts`（若前端需要模型列表则整合使用，否则删除）
3. THE App SHALL 删除所有空目录（`src/shared/components/`、`src/shared/hooks/`、`src/styles/`）
4. THE App SHALL 清理根目录中不再需要的遗留文件（如 `cgevent_test`、`cgevent_test.rs`、`direct_test.py`、`shortcuts_test.py`、`test_api.py`、`test_injection.py`、`LocRecognizerManager.java` 等）
5. THE App SHALL 清理根目录中过时的文档文件（如 `SPOKENLY_IMPLEMENTATION_REPORT.md`、`spokenly-clone-docs.md`、`UX_IMPROVEMENTS.md`、`UX_IMPROVEMENTS_SUMMARY.md`、`VOICE_INPUT_FIX.md` 等旧版文档）
6. THE App SHALL 清理 `docs/` 目录中与 v7.0 无关的旧版文档
7. THE Backend SHALL 在 `src-tauri/src/utils/` 空目录中添加实用工具或删除该目录

### 需求 8：快速输入流程完善

**用户故事：** 作为用户，我希望快速语音输入的完整流程（按住快捷键 → 录音 → 松开 → 转录 → 注入文本）稳定可靠，以便在任何应用中高效使用语音输入。

**背景：** 快速输入流程依赖需求 1（按键释放检测）的修复。除此之外，当前流程缺少转录结果保存到历史记录、错误恢复等功能。

#### 验收标准

1. WHEN 快速输入转录完成, THE QuickInputService SHALL 将转录结果保存到 Database 历史记录中
2. WHEN 快速输入转录完成且 auto_inject 启用, THE QuickInputService SHALL 在指定延迟后将文本注入到当前活跃应用
3. IF 快速输入过程中录音失败, THEN THE QuickInputService SHALL 隐藏 FloatingWindow 并通过事件通知 Frontend 显示错误
4. IF 快速输入过程中转录失败, THEN THE QuickInputService SHALL 隐藏 FloatingWindow 并通过事件通知 Frontend 显示错误
5. WHILE 快速输入正在转录, THE FloatingWindow SHALL 显示"正在转录..."状态
6. WHEN 快速输入转录成功, THE FloatingWindow SHALL 短暂显示转录结果后自动隐藏

### 需求 9：应用启动与生命周期管理

**用户故事：** 作为用户，我希望应用启动时自动恢复之前的配置状态（包括全局快捷键注册），关闭窗口时应用最小化到系统托盘而非退出。

#### 验收标准

1. WHEN App 启动时, THE Backend SHALL 自动加载持久化的设置并注册之前配置的全局快捷键
2. WHEN 用户关闭主窗口, THE App SHALL 最小化到系统托盘而非退出进程
3. WHEN 用户点击系统托盘"退出"菜单, THE App SHALL 正确释放所有资源（快捷键监听、音频设备、数据库连接）后退出
4. WHEN 用户点击系统托盘"显示"菜单, THE App SHALL 恢复并聚焦主窗口
5. IF App 启动时检测到辅助功能权限未授权, THEN THE App SHALL 提示用户授权辅助功能权限

---

## 已完成功能（仅供参考，无需实现）

以下功能在 v7.0 重构中已完成，列出以供上下文参考：

1. ✅ 使用 cpal 进行音频录制（16kHz Mono）
2. ✅ OpenAI Whisper API 转录集成
3. ✅ SQLite 历史记录管理（CRUD + 搜索）
4. ✅ macOS Core Graphics 文本注入
5. ✅ 全局快捷键注册/注销（仅 key-down，需求 1 将修复 key-up）
6. ✅ 快速语音输入服务框架（QuickInputService）
7. ✅ 悬浮状态窗口（quick-input.html）
8. ✅ 设置管理 UI（API Key、模型、设备、注入配置）
9. ✅ 系统托盘（显示/退出菜单）
10. ✅ 三页面 UI（录音、历史、设置）
11. ✅ Zustand 状态管理
12. ✅ macOS Entitlements 和图标配置
