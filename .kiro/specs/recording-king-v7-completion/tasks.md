# 实施计划：Recording King v7.0 完善与修复

## 概述

按照依赖关系组织任务：先完成基础设施（代码清理、构建修复、依赖配置），再实现核心功能（按键监听、设置持久化），然后补充前端错误处理和加载状态，最后完善快速输入流程和生命周期管理，全程穿插属性测试和单元测试。

## Tasks

- [ ] 1. 代码清理与构建基础修复（需求 3、7）
  - [ ] 1.1 删除冗余文件和目录
    - 删除 `old/` 目录及其所有内容
    - 删除 `src/data/localModels.ts`（确认未被引用后删除）
    - 删除空目录：`src/shared/hooks/`、`src/styles/`、`src-tauri/src/utils/`
    - 删除根目录遗留测试文件：`cgevent_test`、`cgevent_test.rs`、`direct_test.py`、`shortcuts_test.py`、`test_api.py`、`test_injection.py`、`LocRecognizerManager.java`
    - 删除根目录过时文档：`SPOKENLY_IMPLEMENTATION_REPORT.md`、`spokenly-clone-docs.md`、`UX_IMPROVEMENTS.md`、`UX_IMPROVEMENTS_SUMMARY.md`、`VOICE_INPUT_FIX.md`
    - 清理 `docs/` 目录中与 v7.0 无关的旧文档
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_
  - [ ] 1.2 清理编译警告并生成图标文件
    - 清理 `src-tauri/` 中所有未使用的 `use`、变量和死代码（11 个编译警告）
    - 确保 `src-tauri/icons/icon.icns` 存在且有效（从现有 PNG 转换或生成占位图标）
    - 验证 `tauri.conf.json` 中所有图标路径引用的文件均存在
    - _Requirements: 3.1, 3.2, 3.4_

- [ ] 2. 测试框架配置与依赖安装
  - [ ] 2.1 配置后端测试依赖
    - 在 `src-tauri/Cargo.toml` 的 `[dev-dependencies]` 中添加 `proptest = "1.4"` 和 `tempfile = "3.9"`
    - 在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加 `rdev = "0.5"` 和 `base64 = "0.22"`
    - _Requirements: 6.1, 6.2, 6.3_
  - [ ] 2.2 配置前端测试框架
    - 在 `package.json` 的 `devDependencies` 中添加 `vitest`、`@testing-library/react`、`@testing-library/jest-dom`、`fast-check`、`jsdom`
    - 创建 `vitest.config.ts`，配置 jsdom 环境和 React 支持
    - _Requirements: 6.5, 6.6_

- [ ] 3. Checkpoint — 确保项目可编译、前端可构建
  - 确保 `cargo build` 零警告通过，`npm run build` 成功，所有新依赖安装正确。如有问题请询问用户。

- [ ] 4. 后端核心：AppSettings 扩展与设置持久化（需求 2）
  - [ ] 4.1 扩展 AppSettings 数据模型和 Database 持久化方法
    - 在 `src-tauri/src/core/types.rs` 中为 `AppSettings` 添加 `shortcut_key: Option<String>` 字段
    - 为 `AppSettings` 实现 `Default` trait 和 `PartialEq`
    - 在 `src-tauri/src/services/database.rs` 中创建 `app_settings` 表（key-value schema）
    - 实现 `Database::save_settings()` 和 `Database::load_settings()` 方法
    - API Key 使用 Base64 编码存储，读取时解码
    - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - [ ]* 4.2 属性测试：AppSettings 序列化往返（Property 5）
    - **Property 5: AppSettings 序列化往返**
    - 使用 `proptest` 为 `AppSettings` 实现 `Arbitrary` 生成器
    - 验证 `serde_json::to_string()` → `serde_json::from_str()` 往返一致性
    - **Validates: Requirements 6.7**
  - [ ]* 4.3 属性测试：AppSettings 数据库持久化往返（Property 3）
    - **Property 3: AppSettings 数据库持久化往返**
    - 使用 `tempfile` 创建临时数据库，`proptest` 生成随机 `AppSettings`
    - 验证 `save_settings()` → `load_settings()` 往返一致性
    - **Validates: Requirements 2.1, 2.2**
  - [ ]* 4.4 属性测试：API Key 非明文存储（Property 4）
    - **Property 4: API Key 非明文存储**
    - 验证保存后数据库中 `openai_api_key` 的 value 字段不等于原始明文
    - **Validates: Requirements 2.4**
  - [ ]* 4.5 单元测试：Database CRUD 和边界情况
    - 测试 `save_settings` / `load_settings` 基本流程
    - 测试数据库不存在或损坏时使用默认设置
    - 测试历史记录 CRUD 操作和搜索功能
    - _Requirements: 6.1, 2.3_

- [ ] 5. 后端核心：rdev 全局按键监听（需求 1）
  - [ ] 5.1 实现 GlobalKeyListener 模块
    - 创建 `src-tauri/src/core/global_listener.rs`
    - 实现 `GlobalKeyListener` 结构体：`new()`、`start()`、`stop()`、`is_running()`
    - 使用 `rdev::listen()` 在独立线程中监听全局按键事件
    - 通过 `Arc<AtomicBool>` 控制监听线程生命周期
    - 在 `src-tauri/src/core/mod.rs` 中注册新模块
    - _Requirements: 1.1, 1.2, 1.7_
  - [ ] 5.2 集成 ShortcutManager 与 GlobalKeyListener
    - 修改 `src-tauri/src/core/shortcuts.rs`，添加按键状态追踪（`is_pressed`、`press_start` 时间戳）
    - 实现 `handle_key_down()` 和 `handle_key_up()` 方法
    - 实现 200ms 短按/长按阈值判断逻辑
    - 监听线程异常时记录日志并通知前端
    - _Requirements: 1.1, 1.2, 1.4, 1.5, 1.6_
  - [ ]* 5.3 属性测试：ShortcutManager 按键状态往返（Property 1）
    - **Property 1: ShortcutManager 按键状态往返**
    - 验证 `handle_key_down()` → `handle_key_up()` 后状态回到未按下
    - **Validates: Requirements 1.1, 1.2**
  - [ ]* 5.4 属性测试：按键时长阈值决定事件类型（Property 2）
    - **Property 2: 按键时长阈值决定事件类型**
    - 使用 `proptest` 生成随机时长，验证 >= 200ms 返回 LongPress，< 200ms 返回 ShortPress
    - **Validates: Requirements 1.4, 1.5**

- [ ] 6. 后端核心：设置持久化集成与命令层（需求 2、9）
  - [ ] 6.1 集成 AppState 设置加载/保存
    - 修改 `src-tauri/src/services/state.rs`：`AppState::new()` 启动时调用 `database.load_settings()` 加载设置
    - 新增 `AppState::save_settings()` 方法，同时更新内存和持久化
    - _Requirements: 2.1, 2.2, 9.1_
  - [ ] 6.2 修改设置命令实现持久化
    - 修改 `src-tauri/src/commands/settings.rs`：`update_settings` 调用 `AppState::save_settings()`
    - 保存成功/失败返回明确的 Result
    - _Requirements: 2.1, 2.5, 2.6_

- [ ] 7. Checkpoint — 确保后端核心功能可编译
  - 确保所有后端修改编译通过，属性测试和单元测试通过。如有问题请询问用户。

- [ ] 8. 前端：错误处理组件（需求 4）
  - [ ] 8.1 创建 ErrorBoundary 组件
    - 创建 `src/shared/components/ErrorBoundary.tsx`
    - 实现 React class component ErrorBoundary，捕获渲染异常
    - 显示友好的错误回退界面，包含错误描述和重试按钮
    - _Requirements: 4.1, 4.2_
  - [ ] 8.2 创建 Toast 通知组件
    - 创建 `src/shared/components/Toast.tsx`
    - 实现 Toast 通知组件，支持 success/error/warning/info 类型
    - 支持自动消失（默认 3000ms）和手动关闭
    - _Requirements: 4.3_
  - [ ] 8.3 扩展 useAppStore 错误和 Toast 状态
    - 在 `src/shared/stores/useAppStore.ts` 中添加 `error`、`toasts` 状态
    - 添加 `addToast()`、`removeToast()`、`setError()` actions
    - 添加错误消息映射表（中文用户友好消息）
    - _Requirements: 4.3, 4.4, 4.5, 4.6, 4.7_
  - [ ] 8.4 集成 ErrorBoundary 和 Toast 到 App.tsx
    - 在 `src/App.tsx` 根组件外层包裹 ErrorBoundary
    - 在 App 内渲染 Toast 容器
    - 修改所有 Tauri command 调用，失败时调用 `addToast()` 而非 `console.log`
    - _Requirements: 4.1, 4.3_
  - [ ]* 8.5 属性测试：ErrorBoundary 捕获渲染异常（Property 6）
    - **Property 6: ErrorBoundary 捕获渲染异常**
    - 使用 `fast-check` 生成随机错误消息，验证 ErrorBoundary 渲染回退 UI
    - 创建 `src/shared/components/ErrorBoundary.test.tsx`
    - **Validates: Requirements 4.2**
  - [ ]* 8.6 属性测试：Tauri 命令错误在 UI 中可见（Property 7）
    - **Property 7: Tauri 命令错误在 UI 中可见**
    - 验证 `addToast({ type: 'error', message })` 后 Toast 组件渲染可见错误元素
    - 在 `src/shared/stores/useAppStore.test.ts` 中实现
    - **Validates: Requirements 4.3**

- [ ] 9. 前端：加载状态管理（需求 5）
  - [ ] 9.1 扩展 useAppStore 加载状态
    - 添加 `isInitializing`、`isTranscribing`、`initError` 状态
    - 添加 `setInitializing()`、`setTranscribing()`、`setInitError()` actions
    - _Requirements: 5.1, 5.3_
  - [ ] 9.2 实现 App 初始化加载流程
    - 修改 `src/App.tsx`：`initializeApp()` 设置 `isInitializing = true`，完成后设为 `false`
    - 初始化期间显示全局 Loading 组件
    - 初始化失败显示错误信息和重试按钮
    - _Requirements: 5.1, 5.4, 5.5_
  - [ ] 9.3 实现录音和转录状态 UI 反馈
    - 修改 `src/features/recording/RecordingPage.tsx`：转录中禁用录音按钮，显示进度指示器
    - 录音中显示时长计时器和脉动动画
    - _Requirements: 5.2, 5.3_
  - [ ]* 9.4 属性测试：加载指示器与初始化状态同步（Property 8）
    - **Property 8: 加载指示器与初始化状态同步**
    - 验证 `isInitializing === true` 时加载指示器可见且主内容隐藏，反之亦然
    - 创建 `src/App.test.tsx`
    - **Validates: Requirements 5.1, 5.4**
  - [ ]* 9.5 属性测试：转录中禁用录音按钮（Property 9）
    - **Property 9: 转录中禁用录音按钮**
    - 验证 `isTranscribing === true` 时录音按钮 disabled 且显示进度指示器
    - 创建 `src/features/recording/RecordingPage.test.tsx`
    - **Validates: Requirements 5.3**

- [ ] 10. Checkpoint — 确保前端组件可构建、测试通过
  - 确保 `npm run build` 成功，所有前端属性测试和单元测试通过。如有问题请询问用户。

- [ ] 11. 快速输入流程完善（需求 8）
  - [ ] 11.1 实现快速输入转录结果保存和 FloatingWindow 状态更新
    - 修改 `src-tauri/src/services/quick_input.rs`：转录完成后保存到 Database 历史记录
    - 添加 `quick-input-transcribing` 和 `quick-input-result` 事件 emit
    - 实现错误恢复：录音/转录失败时隐藏 FloatingWindow 并通知前端
    - _Requirements: 8.1, 8.3, 8.4, 8.5, 8.6_
  - [ ] 11.2 修改 quick_input 命令使用 GlobalKeyListener
    - 修改 `src-tauri/src/commands/quick_input.rs`：`register_global_shortcut` 改用 `GlobalKeyListener`
    - `unregister_global_shortcut` 调用 `GlobalKeyListener::stop()`
    - auto_inject 启用时在指定延迟后注入文本
    - _Requirements: 1.1, 1.2, 8.2_
  - [ ] 11.3 更新 FloatingWindow 前端状态展示
    - 修改 `quick-input.html`：监听 `quick-input-transcribing` 事件显示"正在转录..."
    - 监听 `quick-input-result` 事件短暂显示转录结果后自动隐藏
    - _Requirements: 8.5, 8.6_
  - [ ]* 11.4 属性测试：快速输入转录结果持久化到历史（Property 10）
    - **Property 10: 快速输入转录结果持久化到历史**
    - 使用 `proptest` 生成随机转录文本，验证保存后出现在 `get_history()` 结果中
    - 在 `src-tauri/src/services/database.rs` 测试模块中实现
    - **Validates: Requirements 8.1**

- [ ] 12. 应用启动与生命周期管理（需求 9）
  - [ ] 12.1 实现应用启动自动恢复和窗口关闭最小化
    - 修改 `src-tauri/src/main.rs` setup 闭包：加载持久化设置，自动注册之前配置的全局快捷键
    - 添加辅助功能权限检查，未授权时提示用户
    - 添加 `on_window_event` 处理：`CloseRequested` 时隐藏窗口而非退出（`api.prevent_close()`）
    - 系统托盘"退出"事件中释放 GlobalKeyListener、关闭数据库连接后退出
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [ ] 13. 前端补充测试（需求 6）
  - [ ]* 13.1 useAppStore 单元测试
    - 创建 `src/shared/stores/useAppStore.test.ts`
    - 测试所有 action：`addToast`、`removeToast`、`setError`、`setInitializing`、`setTranscribing`
    - _Requirements: 6.5_
  - [ ]* 13.2 页面组件基础渲染测试
    - 创建 `src/features/history/HistoryPage.test.tsx`：基础渲染测试
    - 创建 `src/features/settings/SettingsPage.test.tsx`：基础渲染测试、保存成功/失败提示
    - _Requirements: 6.6_
  - [ ]* 13.3 后端 AudioRecorder 配置验证测试
    - 在 `src-tauri/src/core/audio.rs` 中添加 `#[cfg(test)] mod tests`
    - 测试 RecordingConfig 验证逻辑
    - _Requirements: 6.4_

- [ ] 14. Final Checkpoint — 确保所有测试通过、构建成功
  - 确保 `cargo build` 零警告，`cargo test` 全部通过，`npm run build` 成功，`npx vitest --run` 全部通过。如有问题请询问用户。

## Notes

- 标记 `*` 的子任务为可选测试任务，可跳过以加速 MVP
- 每个任务引用具体需求编号以确保可追溯性
- 属性测试验证设计文档中的 10 个正确性属性
- Checkpoint 任务确保增量验证，避免问题累积
- 任务按依赖关系排序：清理 → 基础设施 → 后端核心 → 前端 → 集成 → 生命周期
