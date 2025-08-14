# Recording King Windows 版本部署指南

## 📋 部署概述

Recording King 已完成跨平台适配，支持 Windows 10/11 系统。本指南详细说明如何在 Windows 环境下构建和部署应用程序。

## 🔧 环境准备

### Windows 构建环境要求

1. **Windows 10/11** (x64)
2. **Visual Studio 2019/2022** 或 **Build Tools for Visual Studio**
   - 安装 "C++ build tools" 工作负载
   - 包含 Windows 10/11 SDK
3. **Node.js 18+** 
4. **Rust** (最新稳定版)
5. **Git**

### 快速环境设置

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-pc-windows-msvc

# 安装 Node.js 依赖
npm install
```

## 🏗️ 构建 Windows 版本

### 1. 克隆项目
```bash
git clone <repository-url>
cd recording-king
```

### 2. 安装依赖
```bash
npm install
```

### 3. 构建应用
```bash
# 完整构建（推荐）
npm run tauri build

# 或指定 Windows 目标
npm run tauri build -- --target x86_64-pc-windows-msvc
```

### 4. 构建输出
成功构建后，Windows 安装包位于：
```
src-tauri/target/release/bundle/
├── msi/              # Windows Installer (.msi)
└── nsis/             # NSIS 安装程序 (.exe)
```

## ⚙️ Windows 特定配置

### 权限系统适配

Recording King 在 Windows 上使用以下权限机制：

#### 1. UAC (用户账户控制)
- 替代 macOS 的 TCC 权限系统
- 处理管理员权限提升请求

#### 2. 系统权限类别
| 权限类型 | Windows 实现 | 状态检测 |
|---------|-------------|----------|
| 麦克风访问 | Windows Privacy Settings | 自动检测 |
| 文件系统访问 | 标准文件权限 | 运行时检查 |
| 全局快捷键 | Windows Hook API | 注册检测 |
| 通知权限 | Windows Notification API | 自动授权 |
| 系统访问 | UAC 权限控制 | 实时验证 |

### 快捷键系统差异

#### macOS vs Windows 快捷键映射
```typescript
// 系统级快捷键冲突检测
const systemShortcuts = {
  windows: [
    'Ctrl+Esc',      // 开始菜单
    'Alt+Tab',       // 应用切换  
    'Alt+F4',        // 关闭窗口
    'Ctrl+Shift+Esc', // 任务管理器
    'Win+L'          // 锁定系统
  ],
  mac: [
    'Cmd+Tab',       // 应用切换
    'Cmd+Space',     // Spotlight
    'Alt+Tab',       // 窗口切换
  ]
}
```

## 📦 安装包配置

### NSIS 安装程序设置
```json
{
  "nsis": {
    "displayLanguageSelector": true,
    "installerIcon": null,
    "installMode": "currentUser",
    "languages": ["English", "SimpChinese"],
    "template": null
  }
}
```

### MSI 安装程序设置
```json
{
  "windows": {
    "certificateThumbprint": null,
    "digestAlgorithm": "sha256",
    "webviewInstallMode": {
      "type": "downloadBootstrapper"
    },
    "allowDowngrades": true
  }
}
```

## 🚀 部署流程

### 1. 自动化构建（推荐）
```bash
# 创建发布脚本
cat > build-windows.bat << 'EOF'
@echo off
echo 正在构建 Recording King Windows 版本...

REM 清理之前的构建
npm run clean

REM 安装依赖
npm install

REM 构建前端
npm run build

REM 构建 Tauri 应用
npm run tauri build

echo 构建完成！安装包位于 src-tauri/target/release/bundle/
pause
EOF
```

### 2. 手动构建步骤
1. **准备环境**：确保所有依赖已安装
2. **构建前端**：`npm run build`
3. **构建应用**：`npm run tauri build`
4. **验证输出**：检查 bundle 目录
5. **测试安装**：在干净的 Windows 系统上测试

## 🔒 代码签名（可选）

### 配置数字证书
```json
{
  "windows": {
    "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
    "timestampUrl": "http://timestamp.comodoca.com"
  }
}
```

### 签名命令
```bash
# 使用 signtool 签名
signtool sign /tr http://timestamp.comodoca.com /td sha256 /fd sha256 /a "Recording-King-Setup.exe"
```

## 📋 部署检查清单

### 构建前检查
- [ ] Windows 10/11 环境
- [ ] Visual Studio Build Tools 已安装
- [ ] Rust 工具链配置正确
- [ ] Node.js 和 npm 版本兼容
- [ ] 项目依赖完整安装

### 构建后验证
- [ ] 安装包文件完整生成
- [ ] 文件大小合理（通常 50-100MB）
- [ ] 数字签名验证（如适用）
- [ ] 在干净系统上测试安装
- [ ] 应用启动和核心功能正常

### 发布前测试
- [ ] 权限请求正常显示
- [ ] 全局快捷键功能正常
- [ ] 语音录制和转录功能
- [ ] 文件保存和读取权限
- [ ] 系统通知功能
- [ ] 多语言界面正确显示

## ⚠️ 常见问题

### 构建问题
1. **link.exe not found**
   - 安装 Visual Studio Build Tools
   - 确保 C++ 构建工具已选中

2. **权限访问被拒绝**
   - 以管理员身份运行命令提示符
   - 检查 Windows Defender 设置

3. **WebView2 运行时缺失**
   - 构建时自动下载引导程序
   - 或预安装 WebView2 运行时

### 运行时问题
1. **全局快捷键不工作**
   - 检查是否被其他应用占用
   - 确认 UAC 权限授予

2. **麦克风权限问题**
   - Windows 设置 > 隐私 > 麦克风
   - 允许桌面应用访问麦克风

## 📞 技术支持

如在 Windows 部署过程中遇到问题：

1. **查看日志文件**：`%APPDATA%/recording-king/logs/`
2. **检查系统要求**：确认 Windows 版本兼容性
3. **社区支持**：GitHub Issues 或官方文档

---

**Recording King - 录音王** 🎤👑
*如果你觉得我好用，那么你就叫我-录音王吧！*