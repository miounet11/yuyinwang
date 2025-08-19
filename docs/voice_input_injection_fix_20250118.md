# 语音输入文本注入问题修复方案

## 问题描述
用户报告使用快捷键进行语音输入后，对话框消失了，但转换的内容没有注入到输入框，而是出现在"测试转录"区域。

## 问题分析

### 根本原因
1. **文本注入方法问题**：原代码使用了错误的 key code（`key code 9`），在某些情况下可能不起作用
2. **剪贴板类型问题**：使用了不标准的剪贴板类型标识符
3. **缺少错误处理**：没有备用方案和详细的错误日志

### 影响范围
- 所有使用语音输入快捷键的场景
- 影响文本注入到目标应用的功能

## 修复方案

### 1. 修复 AppleScript 命令
**文件**: `src-tauri/src/commands/voice_input.rs`

**修改内容**:
- 将 `key code 9` 改为 `keystroke "v"`，使用更可靠的方式触发粘贴
- 改进 AppleScript 格式，使用多行格式提高可读性

### 2. 修复剪贴板类型
**修改内容**:
- 将 `public.utf8-plain-text` 改为 `NSStringPboardType`
- 这是 macOS 标准的字符串剪贴板类型

### 3. 添加备用方案
**新增功能**:
- 当 AppleScript 执行失败时，使用 `core-graphics` 库的 CGEvent API 作为备用方案
- 直接模拟键盘事件，更底层更可靠

### 4. 增强调试日志
**新增日志**:
- 每个步骤都添加详细的日志输出
- 包括当前活动应用信息、剪贴板操作状态、粘贴执行结果等

### 5. 添加必要依赖
**文件**: `src-tauri/Cargo.toml`

**新增依赖**:
```toml
core-graphics = "0.23"
```

## 具体代码修改

### inject_text_to_active_app 函数优化

1. **增加详细日志**
```rust
println!("🔤 开始注入文本: '{}'", text);
println!("📱 当前活动应用: {} ({})", app_info.name, app_info.bundle_id);
```

2. **改进剪贴板操作**
```rust
// 使用标准的剪贴板类型
let string_type = NSString::alloc(nil).init_str("NSStringPboardType");

// 检查写入是否成功
let success: bool = msg_send![general_pasteboard, setString:text_string forType:string_type];
if !success {
    eprintln!("❌ 写入剪贴板失败");
    return Err("写入剪贴板失败".to_string());
}
```

3. **改进 AppleScript**
```rust
let script = r#"
    tell application "System Events"
        keystroke "v" using {command down}
    end tell
"#;
```

4. **添加备用方案**
```rust
if error != nil {
    println!("🔄 尝试备用方法...");
    
    // 使用 CGEventPost 直接发送键盘事件
    use core_graphics::event::{CGEvent, CGEventFlags, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
    
    // 按下 Cmd+V
    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), 9, true) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(core_graphics::event::CGEventTapLocation::HID);
    }
    
    // 释放 Cmd+V
    if let Ok(event) = CGEvent::new_keyboard_event(source, 9, false) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(core_graphics::event::CGEventTapLocation::HID);
    }
}
```

## 测试步骤

### 1. 编译测试
```bash
cd src-tauri
cargo build --release
```

### 2. 功能测试

#### 测试场景 1：文本编辑器
1. 打开任意文本编辑器（如 TextEdit、VS Code）
2. 将光标放在输入区域
3. 使用语音输入快捷键
4. 说话并等待转录
5. 验证文本是否正确注入到编辑器

#### 测试场景 2：浏览器输入框
1. 打开浏览器，进入任意网页的输入框
2. 点击输入框获得焦点
3. 使用语音输入快捷键
4. 说话并等待转录
5. 验证文本是否正确注入到输入框

#### 测试场景 3：聊天应用
1. 打开聊天应用（如 微信、Slack）
2. 选择一个对话
3. 点击消息输入框
4. 使用语音输入快捷键
5. 说话并等待转录
6. 验证文本是否正确注入

### 3. 日志检查
运行应用时查看控制台输出，应该能看到：
- 🔤 开始注入文本
- 📱 当前活动应用信息
- 📋 剪贴板操作步骤
- ⌨️ 粘贴操作执行
- ✅ 文本注入完成

### 4. 错误场景测试
- 测试在 Recording King 自身输入（应该被阻止）
- 测试在没有输入焦点的情况下
- 测试在系统对话框中

## 预期结果

修复后，语音输入功能应该：
1. ✅ 正确将转录文本注入到目标应用
2. ✅ 不会在"测试转录"区域显示
3. ✅ 支持各种应用的输入框
4. ✅ 有详细的日志便于调试
5. ✅ 有备用方案保证可靠性

## 版本信息
- 修复版本：4.6.9
- 修复日期：2025-01-18
- 修复人：Recording King Team

## 后续优化建议

1. **添加配置选项**：允许用户选择注入方式（剪贴板/直接输入）
2. **添加重试机制**：如果注入失败，自动重试
3. **改进焦点管理**：更精确地恢复原应用的焦点和光标位置
4. **添加注入成功反馈**：通过声音或视觉提示告知用户注入成功
