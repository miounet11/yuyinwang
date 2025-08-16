# Recording King v4.1.0 安装指南

## 📥 下载文件

- **DMG 安装包**: `Recording King_4.1.0_aarch64.dmg` (8.7 MB)
- **应用程序**: `Recording King.app`
- **SHA256 校验**: `9cccc046b32c8ea5791a6075d262ceaeaea200745a29519dfe1d6525740026cf`

## 🚀 快速安装

### 方法 1：使用 DMG 安装包（推荐）

1. **下载 DMG 文件**
   - 文件名：`Recording King_4.1.0_aarch64.dmg`

2. **验证文件完整性**（可选）
   ```bash
   shasum -a 256 "Recording King_4.1.0_aarch64.dmg"
   # 应该显示：9cccc046b32c8ea5791a6075d262ceaeaea200745a29519dfe1d6525740026cf
   ```

3. **安装步骤**
   - 双击 DMG 文件打开
   - 将 Recording King 图标拖到 Applications 文件夹
   - 弹出 DMG 磁盘镜像

4. **首次运行**
   - 在应用程序文件夹中找到 Recording King
   - 右键点击选择"打开"（绕过 Gatekeeper）
   - 或者在终端运行：
     ```bash
     xattr -cr "/Applications/Recording King.app"
     ```

### 方法 2：直接复制应用程序

1. **复制应用到应用程序文件夹**
   ```bash
   cp -r "Recording King.app" /Applications/
   ```

2. **清除隔离属性**
   ```bash
   xattr -cr "/Applications/Recording King.app"
   ```

## 🔐 权限配置

首次启动时，需要授予以下权限：

### 1. 麦克风权限
- 系统会自动弹出权限请求
- 点击"允许"即可

### 2. 辅助功能权限（快捷键功能）
1. 打开"系统设置" → "隐私与安全性" → "辅助功能"
2. 点击锁图标解锁
3. 添加 Recording King 到列表
4. 勾选启用

### 3. 输入监控权限（可选）
1. 打开"系统设置" → "隐私与安全性" → "输入监控"
2. 添加 Recording King
3. 勾选启用

## ⌨️ 快捷键

- **Cmd + Shift + K**: 唤起语音输入窗口
- **Option + L**: 备用语音输入快捷键
- **ESC**: 关闭当前窗口

## 🎯 测试功能

1. **测试语音输入**
   - 按 Cmd+Shift+K
   - 对着麦克风说话
   - 查看实时转录结果

2. **测试文本注入**
   - 打开任意文本编辑器
   - 使用语音输入
   - 文本会自动输入到编辑器

## ❓ 常见问题

### 无法打开应用（提示来自未识别的开发者）
```bash
sudo xattr -cr "/Applications/Recording King.app"
sudo spctl --add "/Applications/Recording King.app"
```

### 快捷键不生效
- 确保已授予"辅助功能"权限
- 重启应用程序
- 检查是否有其他应用占用相同快捷键

### 麦克风无声音
- 检查系统音频输入设置
- 确保已授予麦克风权限
- 尝试选择其他音频设备

## 📱 系统要求

- **操作系统**: macOS 10.13 或更高版本
- **处理器**: Apple Silicon (M1/M2/M3) 或 Intel
- **内存**: 建议 4GB 或以上
- **存储**: 至少 200MB 可用空间

## 🔄 卸载方法

1. **删除应用程序**
   ```bash
   rm -rf "/Applications/Recording King.app"
   ```

2. **清理应用数据**（可选）
   ```bash
   rm -rf ~/Library/Application\ Support/recording-king
   rm -rf ~/Library/Application\ Support/spokenly-clone
   rm -rf ~/Library/Caches/com.recordingking.app
   ```

3. **移除权限设置**
   - 在系统设置中移除相关权限授权

## 💡 提示

- 建议将 Recording King 添加到登录项，开机自动启动
- 可以在菜单栏图标中快速访问常用功能
- 定期检查更新以获得最新功能和修复

---
如有任何问题，请查看发布说明或联系技术支持。