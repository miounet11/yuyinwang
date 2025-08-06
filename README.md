# 🎤 Spokenly Clone - 语音王

一个功能强大的实时语音转文字桌面应用，完全复刻Spokenly的专业界面和功能。

![Version](https://img.shields.io/badge/version-1.0.1-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-1.5-orange.svg)
![React](https://img.shields.io/badge/React-18-blue.svg)
![Rust](https://img.shields.io/badge/Rust-2021-red.svg)

## ✨ 功能特性

### 🎨 专业UI设计
- 🌙 深色主题界面，完全匹配Spokenly专业外观
- 📱 现代化响应式布局
- 🎯 直观的用户交互体验

### 🤖 多AI模型支持
- **GPT-4o mini** - OpenAI最新模型，卓越准确性
- **Deepgram Nova-3** - 实时转录，英语优化
- **Mistral Voxtral Mini** - 多语言支持，高质量输出
- **ElevenLabs Scribe** - 高质量录制，先进语言识别

### 🎤 音频处理
- 🔍 自动检测系统音频设备
- 🎛️ 音频设备优先级管理
- 🔊 实时音频输入处理
- 📊 音频质量监控

### 📋 历史记录管理
- 💾 完整转录历史保存
- 🔍 智能搜索和过滤
- 📊 转录准确度显示
- ⏱️ 时间戳和持续时间记录

### ⌨️ 快捷键系统
- 🌐 全局快捷键支持
- ⚙️ 自定义键盘组合
- 🎯 一键录音切换
- 🧪 快捷键测试工具

### 📁 文件转录
- 📤 拖拽文件上传
- 🎵 支持多种音频格式（MP3, WAV, M4A, FLAC）
- 🎬 支持视频格式（MP4, MOV, M4V）
- 🔄 批量文件处理

## 🛠️ 技术栈

### 前端
- **React 18** - 现代化前端框架
- **TypeScript** - 类型安全
- **Zustand** - 轻量级状态管理
- **CSS Variables** - 主题系统

### 后端
- **Rust** - 高性能系统编程语言
- **Tauri** - 现代桌面应用框架
- **CPAL** - 跨平台音频库
- **Tokio** - 异步运行时

### AI集成
- **MCP协议** - 模型连接协议
- **多AI模型** - 支持主流语音识别服务
- **RESTful API** - 标准化接口设计

## 🚀 快速开始

### 环境要求
- Node.js >= 18
- Rust >= 1.70
- 操作系统：macOS, Windows, Linux

### 安装依赖
```bash
npm install
```

### 开发运行
```bash
npm run tauri:dev
```

### 构建应用
```bash
npm run tauri:build
```

## 📱 使用方法

### 基本操作
1. **启动应用** - 运行后会显示专业的深色主题界面
2. **选择AI模型** - 在"听写模型"页面选择合适的AI模型
3. **配置音频设备** - 在"常规设置"中选择麦克风设备
4. **开始录音** - 点击录音按钮或使用快捷键
5. **查看结果** - 转录结果会实时显示并保存到历史记录

### 高级功能
- **文件转录** - 拖拽音视频文件到"转录文件"页面
- **历史管理** - 在"历史记录"页面查看和管理所有转录
- **快捷键配置** - 在"快捷键"页面自定义全局热键
- **设置调整** - 在"常规设置"中调整应用行为

## 🔧 配置选项

### 音频设置
- 麦克风设备选择
- 音频质量配置
- 实时监控开关

### AI模型配置
- 模型选择（准确度vs速度平衡）
- API密钥配置
- 语言设置

### 界面定制
- 主题选择
- 界面语言
- 启动行为

## 🔮 未来计划

- [ ] 本地Whisper模型集成
- [ ] 云端同步功能
- [ ] 实时字幕显示
- [ ] 批量文件处理优化
- [ ] 插件系统支持
- [ ] 多语言界面
- [ ] 高级音频处理

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 🤝 贡献

欢迎提交 Issues 和 Pull Requests！

## 🙏 致谢

- [Tauri](https://tauri.app/) - 现代桌面应用框架
- [React](https://reactjs.org/) - 前端框架
- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [Spokenly](https://spokenly.com/) - 原始灵感来源

---

**开发者**: miounet11  
**邮箱**: 9248293@qq.com  
**版本**: 1.0.1  
**构建时间**: 2025-08-06