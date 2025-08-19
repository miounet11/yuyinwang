#!/usr/bin/env python3
"""
Recording King 文本注入测试工具
使用 Python 测试 macOS 的文本注入功能
"""

import subprocess
import time
import sys

def test_clipboard():
    """测试剪贴板功能"""
    print("📋 测试剪贴板功能...")
    
    test_text = "Python测试文本789"
    
    # 写入剪贴板
    try:
        subprocess.run(['pbcopy'], input=test_text, text=True, check=True)
        print(f"✅ 已写入剪贴板: '{test_text}'")
    except subprocess.CalledProcessError as e:
        print(f"❌ 剪贴板写入失败: {e}")
        return False
    
    # 读取剪贴板
    try:
        result = subprocess.run(['pbpaste'], capture_output=True, text=True, check=True)
        clipboard_content = result.stdout.strip()
        if clipboard_content == test_text:
            print("✅ 剪贴板读取正确")
            return True
        else:
            print(f"❌ 剪贴板内容不匹配: 期望='{test_text}', 实际='{clipboard_content}'")
            return False
    except subprocess.CalledProcessError as e:
        print(f"❌ 剪贴板读取失败: {e}")
        return False

def test_applescript_keystroke():
    """测试 AppleScript 按键发送"""
    print("\n⌨️  测试 AppleScript 按键发送...")
    print("⏰ 请在5秒内切换到目标应用（如Safari地址栏或Notes）...")
    
    for i in range(5, 0, -1):
        print(f"⏳ {i}秒...")
        time.sleep(1)
    
    print("🚀 发送 Cmd+V...")
    
    applescript = '''
    tell application "System Events"
        keystroke "v" using {command down}
    end tell
    '''
    
    try:
        result = subprocess.run(['osascript', '-e', applescript], 
                              capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            print("✅ AppleScript 按键发送成功")
            return True
        else:
            print(f"❌ AppleScript 失败: {result.stderr.strip()}")
            return False
    except subprocess.TimeoutExpired:
        print("❌ AppleScript 超时")
        return False
    except subprocess.CalledProcessError as e:
        print(f"❌ AppleScript 执行错误: {e}")
        return False

def test_alternative_methods():
    """测试其他注入方法"""
    print("\n🔄 测试其他方法...")
    
    # 方法1: 使用 shortcuts 命令（如果可用）
    try:
        result = subprocess.run(['which', 'shortcuts'], capture_output=True, text=True)
        if result.returncode == 0:
            print("✅ 发现 shortcuts 命令，可以尝试使用")
        else:
            print("ℹ️  shortcuts 命令不可用")
    except:
        print("ℹ️  shortcuts 命令检查失败")

def main():
    print("🔧 Recording King 文本注入诊断工具")
    print("=" * 50)
    
    # 测试剪贴板
    clipboard_ok = test_clipboard()
    
    if not clipboard_ok:
        print("\n❌ 剪贴板功能异常，无法继续测试")
        return
    
    # 测试按键发送
    keystroke_ok = test_applescript_keystroke()
    
    # 测试其他方法
    test_alternative_methods()
    
    print("\n📊 测试总结:")
    print(f"剪贴板功能: {'✅ 正常' if clipboard_ok else '❌ 异常'}")
    print(f"按键发送功能: {'✅ 正常' if keystroke_ok else '❌ 异常'}")
    
    if not keystroke_ok:
        print("\n💡 解决建议:")
        print("1. 打开 系统偏好设置 → 安全性与隐私 → 隐私 → 输入监控")
        print("2. 添加 Recording King.app 到授权列表")
        print("3. 确保应用已被勾选启用")
        print("4. 重启 Recording King 应用")
        
        # 自动打开设置
        print("\n🚀 自动打开输入监控设置...")
        try:
            subprocess.run(['open', 'x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent'])
        except:
            print("❌ 无法自动打开设置，请手动打开")

if __name__ == "__main__":
    main()