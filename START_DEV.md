# 🚀 启动开发环境

## 当前状态

✅ 前端服务器已启动：http://localhost:1420/
⏳ 后端需要 Rust 环境

---

## 方案 1：完整开发环境（推荐）

### 安装 Rust

```bash
# 运行安装脚本
./install-rust.sh

# 或手动安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
cargo install tauri-cli
```

### 启动完整应用

```bash
npm run tauri:dev
```

这将启动：
- ✅ Vite 前端服务器（已运行）
- ✅ Tauri 桌面应用
- ✅ Rust 后端
- ✅ 完整功能测试

---

## 方案 2：仅前端预览（当前）

### 当前运行中

```
✅ Vite 服务器：http://localhost:1420/
```

### 可以测试

- ✅ UI 界面
- ✅ 页面导航
- ✅ 组件样式
- ❌ 录音功能（需要后端）
- ❌ 快捷键功能（需要后端）
- ❌ 数据库功能（需要后端）

### 访问方式

在浏览器打开：http://localhost:1420/

---

## 测试清单

### 前端测试（当前可测试）

- [ ] 打开 http://localhost:1420/
- [ ] 查看主界面
- [ ] 点击侧边栏导航
  - [ ] 录音页面
  - [ ] 历史页面
  - [ ] 设置页面
- [ ] 检查 UI 样式
  - [ ] 黑色侧边栏
  - [ ] 白色内容区
  - [ ] 圆形录音按钮
  - [ ] CAPS 按钮标签

### 完整功能测试（需要 Rust）

- [ ] 快速语音输入
  - [ ] 注册快捷键
  - [ ] 按住录音
  - [ ] 松开转录
  - [ ] 自动输入
- [ ] 录音转录
  - [ ] 点击 REC
  - [ ] 录音
  - [ ] 点击 STOP
  - [ ] 查看结果
- [ ] 历史记录
  - [ ] 查看列表
  - [ ] 搜索
  - [ ] 删除
- [ ] 设置管理
  - [ ] 配置 API Key
  - [ ] 选择模型
  - [ ] 配置快捷键

---

## 快速命令

```bash
# 查看前端
open http://localhost:1420/

# 停止前端服务器
lsof -ti:1420 | xargs kill -9

# 重启前端
npm run dev

# 安装 Rust（如果需要）
./install-rust.sh

# 启动完整应用
npm run tauri:dev

# 构建生产版本
npm run tauri:build
```

---

## 故障排除

### 端口被占用

```bash
lsof -ti:1420 | xargs kill -9
npm run dev
```

### Rust 未安装

```bash
./install-rust.sh
```

### 前端构建失败

```bash
rm -rf node_modules dist
npm install
npm run build
```

---

**当前状态**：前端服务器运行中 ✅
**下一步**：在浏览器打开 http://localhost:1420/ 查看 UI
