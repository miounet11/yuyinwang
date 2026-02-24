use crate::core::error::Result;

#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
#[cfg(target_os = "macos")]
use std::process::Command;

// ============================================================
// 三层降级注入策略
// Layer 1: CGEvent Unicode（最快，不碰剪贴板）
// Layer 2: 剪贴板 + Cmd+V（兼容性最好）
// Layer 3: AppleScript keystroke（最后兜底）
// ============================================================

#[cfg(target_os = "macos")]
pub fn inject_text(text: &str, delay_ms: u64) -> Result<()> {
    use std::thread;
    use std::time::Duration;

    if text.is_empty() {
        return Ok(());
    }

    if delay_ms > 0 {
        thread::sleep(Duration::from_millis(delay_ms));
    }

    // 先检查权限
    if !check_accessibility_permission() {
        // 没有权限，尝试请求并用 AppleScript 兜底
        println!("⚠️ 无辅助功能权限，尝试 AppleScript 兜底");
        return inject_via_applescript(text);
    }

    // Layer 1: CGEvent Unicode
    match inject_via_cgevent(text) {
        Ok(()) => {
            println!("✅ CGEvent 注入成功: {}字", text.chars().count());
            return Ok(());
        }
        Err(e) => {
            println!("⚠️ CGEvent 注入失败: {}, 降级到剪贴板", e);
        }
    }

    // Layer 2: 剪贴板 + Cmd+V
    match inject_via_clipboard(text) {
        Ok(()) => {
            println!("✅ 剪贴板注入成功: {}字", text.chars().count());
            return Ok(());
        }
        Err(e) => {
            println!("⚠️ 剪贴板注入失败: {}, 降级到 AppleScript", e);
        }
    }

    // Layer 3: AppleScript
    inject_via_applescript(text)
}

// ============================================================
// Layer 1: CGEvent Unicode 直接注入
// 最快，不碰剪贴板，每批 20 个 UTF-16 code unit
// ============================================================
#[cfg(target_os = "macos")]
fn inject_via_cgevent(text: &str) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| crate::core::error::AppError::Permission(
            "CGEventSource creation failed".into()
        ))?;

    // 按 Unicode 字符边界分块，避免 Emoji 被错误分割
    let chars: Vec<char> = text.chars().collect();
    for chunk in chars.chunks(20) {
        let chunk_str: String = chunk.iter().collect();
        let utf16: Vec<u16> = chunk_str.encode_utf16().collect();

        let event = CGEvent::new_keyboard_event(source.clone(), 0, true)
            .map_err(|_| crate::core::error::AppError::Other(
                "CGEvent creation failed".into()
            ))?;

        event.set_string_from_utf16_unchecked(&utf16);
        event.post(CGEventTapLocation::HID);

        if chars.len() > 20 {
            std::thread::sleep(std::time::Duration::from_micros(500));
        }
    }

    Ok(())
}

// ============================================================
// Layer 2: 剪贴板 + Cmd+V
// 兼容性最好，几乎所有应用都支持粘贴
// ============================================================
#[cfg(target_os = "macos")]
fn inject_via_clipboard(text: &str) -> Result<()> {
    use std::io::Write;
    use std::thread;
    use std::time::Duration;

    // 保存原始剪贴板
    let original = Command::new("pbpaste")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());

    // 写入文本到剪贴板
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| crate::core::error::AppError::Other(format!("pbcopy: {}", e)))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(text.as_bytes())
            .map_err(|e| crate::core::error::AppError::Other(format!("write: {}", e)))?;
    }
    child.wait().map_err(|e| crate::core::error::AppError::Other(format!("wait: {}", e)))?;

    thread::sleep(Duration::from_millis(30));

    // Cmd+V
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| crate::core::error::AppError::Permission("CGEventSource failed".into()))?;

    // key code 9 = V
    let down = CGEvent::new_keyboard_event(source.clone(), 9, true)
        .map_err(|_| crate::core::error::AppError::Other("key down failed".into()))?;
    down.set_flags(CGEventFlags::CGEventFlagCommand);
    down.post(CGEventTapLocation::HID);

    thread::sleep(Duration::from_millis(20));

    let up = CGEvent::new_keyboard_event(source, 9, false)
        .map_err(|_| crate::core::error::AppError::Other("key up failed".into()))?;
    up.set_flags(CGEventFlags::CGEventFlagCommand);
    up.post(CGEventTapLocation::HID);

    // 恢复原始剪贴板（延迟，确保粘贴完成）
    // 增加延迟到 300ms 以兼容慢速应用（如 Electron）
    if let Some(orig) = original {
        thread::sleep(Duration::from_millis(300));
        if let Ok(mut c) = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn() {
            if let Some(stdin) = c.stdin.as_mut() {
                let _ = stdin.write_all(orig.as_bytes());
            }
            let _ = c.wait();
        }
    }

    Ok(())
}

// ============================================================
// Layer 3: AppleScript 兜底
// 不需要辅助功能权限，但只支持 ASCII + 基本字符
// 对中文使用剪贴板方式
// ============================================================
#[cfg(target_os = "macos")]
fn inject_via_applescript(text: &str) -> Result<()> {
    use std::io::Write;

    // 中文或特殊字符用剪贴板 + AppleScript 粘贴
    let has_non_ascii = text.chars().any(|c| !c.is_ascii() || c == '"' || c == '\\');

    if has_non_ascii {
        // 写入剪贴板
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| crate::core::error::AppError::Other(format!("pbcopy: {}", e)))?;
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();

        std::thread::sleep(std::time::Duration::from_millis(30));

        // AppleScript Cmd+V
        let script = r#"tell application "System Events" to keystroke "v" using command down"#;
        Command::new("osascript")
            .args(&["-e", script])
            .output()
            .map_err(|e| crate::core::error::AppError::Other(format!("osascript: {}", e)))?;
    } else {
        // 纯 ASCII 直接 keystroke
        let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            r#"tell application "System Events" to keystroke "{}""#,
            escaped
        );
        Command::new("osascript")
            .args(&["-e", &script])
            .output()
            .map_err(|e| crate::core::error::AppError::Other(format!("osascript: {}", e)))?;
    }

    println!("✅ AppleScript 注入成功: {}字", text.chars().count());
    Ok(())
}

// ============================================================
// 焦点管理
// ============================================================

/// 激活指定应用
#[cfg(target_os = "macos")]
pub fn activate_app(bundle_id: &str) -> Result<()> {
    let script = format!(
        r#"tell application id "{}" to activate"#,
        bundle_id
    );
    let output = Command::new("osascript")
        .args(&["-e", &script])
        .output()
        .map_err(|e| crate::core::error::AppError::Other(format!("activate: {}", e)))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        println!("⚠️ activate_app 警告: {}", err);
    }
    Ok(())
}

/// 获取当前前台应用 bundle id
#[cfg(target_os = "macos")]
pub fn get_frontmost_app_bundle_id() -> Result<String> {
    let output = Command::new("osascript")
        .args(&["-e", r#"tell application "System Events" to get bundle identifier of first process whose frontmost is true"#])
        .output()
        .map_err(|e| crate::core::error::AppError::Other(format!("get app: {}", e)))?;

    let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if bundle_id.is_empty() {
        return Err(crate::core::error::AppError::Other("No frontmost app".into()));
    }
    Ok(bundle_id)
}

// ============================================================
// 权限检测 + 引导
// ============================================================

/// 检查辅助功能权限（真正的 API 调用）
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

/// 请求辅助功能权限（弹出系统设置引导）
#[cfg(target_os = "macos")]
pub fn request_accessibility_permission() -> bool {
    // AXIsProcessTrustedWithOptions 传入 kAXTrustedCheckOptionPrompt = true 会弹出系统提示
    extern "C" {
        fn CFDictionaryCreate(
            allocator: *const std::ffi::c_void,
            keys: *const *const std::ffi::c_void,
            values: *const *const std::ffi::c_void,
            num_values: i64,
            key_callbacks: *const std::ffi::c_void,
            value_callbacks: *const std::ffi::c_void,
        ) -> *const std::ffi::c_void;
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
        fn CFRelease(cf: *const std::ffi::c_void);
        static kAXTrustedCheckOptionPrompt: *const std::ffi::c_void;
        static kCFBooleanTrue: *const std::ffi::c_void;
        static kCFTypeDictionaryKeyCallBacks: std::ffi::c_void;
        static kCFTypeDictionaryValueCallBacks: std::ffi::c_void;
    }

    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let options = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks as *const _ as *const std::ffi::c_void,
            &kCFTypeDictionaryValueCallBacks as *const _ as *const std::ffi::c_void,
        );
        let trusted = AXIsProcessTrustedWithOptions(options);
        CFRelease(options);
        trusted
    }
}

// ============================================================
// 非 macOS 平台 fallback
// ============================================================

#[cfg(not(target_os = "macos"))]
pub fn inject_text(_text: &str, _delay_ms: u64) -> Result<()> {
    Err(crate::core::error::AppError::Other(
        "Text injection only supported on macOS".into(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn request_accessibility_permission() -> bool {
    true
}
