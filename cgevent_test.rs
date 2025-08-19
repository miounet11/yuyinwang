// CGEvent 文本注入测试程序
// 独立测试 CGEvent API 是否能正常工作

use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    println!("🧪 CGEvent 文本注入测试程序");
    println!("{}", "=".repeat(50));

    // 1. 设置测试文本到剪贴板
    let test_text = "CGEvent测试文本777";
    println!("📝 测试文本: '{}'", test_text);
    
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn pbcopy");
        
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(test_text.as_bytes()).expect("Failed to write to stdin");
    }
    
    let output = child.wait();
        
    match output {
        Ok(_) => println!("✅ 文本已写入剪贴板"),
        Err(e) => {
            println!("❌ 剪贴板写入失败: {}", e);
            return;
        }
    }

    // 验证剪贴板
    let verify = Command::new("pbpaste").output();
    match verify {
        Ok(output) => {
            let content = String::from_utf8_lossy(&output.stdout);
            if content.trim() == test_text {
                println!("✅ 剪贴板验证成功");
            } else {
                println!("❌ 剪贴板验证失败");
                return;
            }
        }
        Err(e) => {
            println!("❌ 剪贴板验证失败: {}", e);
            return;
        }
    }

    println!("\n⏰ 请在5秒内切换到目标应用（如Safari地址栏或Notes）...");
    for i in (1..=5).rev() {
        println!("⏳ {}秒...", i);
        thread::sleep(Duration::from_secs(1));
    }

    println!("🚀 开始发送 CGEvent Cmd+V...");
    
    // 这里我们会使用 Objective-C 的 CGEvent API
    // 但由于这是独立程序，我们先用简化的方法测试
    
    unsafe {
        test_cgevent();
    }
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    // CGEvent 相关函数
    fn CGEventSourceCreate(state_id: i32) -> *mut std::ffi::c_void;
    fn CGEventCreateKeyboardEvent(source: *mut std::ffi::c_void, virtual_key: u16, key_down: bool) -> *mut std::ffi::c_void;
    fn CGEventSetFlags(event: *mut std::ffi::c_void, flags: u64);
    fn CGEventPost(tap_location: i32, event: *mut std::ffi::c_void);
    fn CFRelease(obj: *mut std::ffi::c_void);
}

const kCGHIDEventTap: i32 = 0;
const kCGEventSourceStateHIDSystemState: i32 = 1;
const kCGEventFlagMaskCommand: u64 = 0x00100000;

unsafe fn test_cgevent() {
    println!("🔧 创建 CGEvent 源...");
    
    let source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
    if source.is_null() {
        println!("❌ 无法创建 CGEvent 源");
        return;
    }
    println!("✅ CGEvent 源创建成功");

    println!("⌨️ 创建键盘事件...");
    
    // 创建 V 键按下事件 (keycode 9)
    let key_down_event = CGEventCreateKeyboardEvent(source, 9, true);
    if key_down_event.is_null() {
        println!("❌ 无法创建按键按下事件");
        CFRelease(source);
        return;
    }
    
    // 设置 Command 修饰键
    CGEventSetFlags(key_down_event, kCGEventFlagMaskCommand);
    println!("✅ 按键按下事件创建成功");

    // 创建 V 键释放事件
    let key_up_event = CGEventCreateKeyboardEvent(source, 9, false);
    if key_up_event.is_null() {
        println!("❌ 无法创建按键释放事件");
        CFRelease(key_down_event);
        CFRelease(source);
        return;
    }
    
    CGEventSetFlags(key_up_event, kCGEventFlagMaskCommand);
    println!("✅ 按键释放事件创建成功");

    println!("📤 发送按键事件...");
    
    // 发送按键按下事件
    CGEventPost(kCGHIDEventTap, key_down_event);
    thread::sleep(Duration::from_millis(50));
    
    // 发送按键释放事件
    CGEventPost(kCGHIDEventTap, key_up_event);
    
    println!("✅ CGEvent 按键事件已发送");

    // 清理资源
    CFRelease(key_up_event);
    CFRelease(key_down_event);
    CFRelease(source);
    
    println!("🧹 资源已清理");
    println!("\n📊 测试完成！如果文本被成功粘贴，说明 CGEvent 方法可行。");
}