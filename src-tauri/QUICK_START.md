# Recording King v7.0 快速开始

## 环境要求

- Node.js 18+
- Rust 1.70+
- macOS 12+ (主要支持平台)

## 安装步骤

### 1. 克隆仓库
```bash
git clone <repository-url>
cd yuyinwang
```

### 2. 安装依赖
```bash
# 安装前端依赖
npm install

# 安装 Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 3. 开发模式运行
```bash
npm run tauri:dev
```

### 4. 构建生产版本
```bash
npm run tauri:build
```

## 首次使用配置

### 1. 配置 OpenAI API Key
1. 打开应用
2. 点击侧边栏"设置"
3. 输入 OpenAI API Key
4. 点击"保存"

### 2. 配置全局快捷键（可选）
1. 在设置页面配置快捷键（如 `Ctrl+Shift+V`）
2. 重启应用自动生效

### 3. 授予 macOS 权限
1. 系统偏好设置 > 安全性与隐私 > 辅助功能
2. 添加 Recording King 到允许列表
3. 重启应用

## 功能使用

### 录音转录
1. 点击"录音"页面
2. 点击"开始录音"按钮
3. 说话
4. 点击"停止录音"
5. 等待转录完成
6. 查看转录结果

### 快速输入（全局快捷键）
1. 在任意应用中按下配置的快捷键
2. 悬浮窗口出现，开始录音
3. 松开快捷键，自动转录
4. 转录结果显示 2 秒后自动隐藏
5. 如果启用自动注入，文本会自动输入到当前应用

### 查看历史记录
1. 点击"历史"页面
2. 浏览所有转录记录
3. 搜索特定内容
4. 删除不需要的记录

## 测试

### 运行前端测试
```bash
# 单元测试
npm run test:run

# 监听模式
npm test
```

### 运行后端测试
```bash
cd src-tauri

# 单元测试
cargo test

# 属性测试
cargo test --features proptest
```

## 故障排除

### 应用无法启动
- 检查 Rust 是否正确安装：`rustc --version`
- 检查 Node.js 版本：`node --version`
- 删除 `node_modules` 和 `target` 目录后重新安装

### 录音失败
- 检查麦克风权限
- 在设置中选择正确的音频设备
- 查看控制台错误日志

### 转录失败
- 检查 API Key 是否正确配置
- 检查网络连接
- 查看错误 Toast 提示信息

### 快捷键不工作
- 检查是否授予辅助功能权限
- 检查快捷键是否与其他应用冲突
- 重启应用

### 文本注入不工作
- 必须授予辅助功能权限
- 系统偏好设置 > 安全性与隐私 > 辅助功能
- 添加 Recording King 到允许列表

## 开发指南

### 项目结构
```
├── src/              # React 前端
├── src-tauri/        # Rust 后端
├── quick-input.html  # 悬浮窗口
└── docs/             # 文档
```

### 添加新功能
1. 后端：在 `src-tauri/src/commands/` 添加命令
2. 前端：在 `src/features/` 添加页面组件
3. 添加测试：`*.test.tsx` 或 `*.rs` 中的 `#[cfg(test)]`
4. 更新文档

### 代码风格
- Rust: `cargo fmt` + `cargo clippy`
- TypeScript: ESLint + Prettier（如果配置）

## 更多信息

- 完整实施报告：`IMPLEMENTATION_COMPLETE.md`
- 任务规划：`.kiro/specs/recording-king-v7-completion/tasks.md`
- 设计文档：`.kiro/specs/recording-king-v7-completion/design.md`
