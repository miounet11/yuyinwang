#!/usr/bin/env python3
"""
直接测试文本注入 - 绕过 Recording King
"""

import subprocess
import time

def test_direct_injection():
    """直接测试文本注入"""
    print("🧪 直接测试文本注入...")
    
    # 设置测试文本
    test_text = "直接注入测试999"
    print(f"📝 测试文本: '{test_text}'")
    
    # 写入剪贴板
    subprocess.run(['pbcopy'], input=test_text, text=True)
    print("✅ 文本已写入剪贴板")
    
    print("⏰ 请在5秒内切换到目标应用...")
    for i in range(5, 0, -1):
        print(f"⏳ {i}...")
        time.sleep(1)
    
    print("🚀 现在尝试用系统快捷键...")
    
    # 使用系统的 shortcuts 命令（如果可用）
    try:
        # 尝试直接发送系统事件
        result = subprocess.run([
            'osascript', '-e', 
            'tell application "System Events" to key code 9 using {command down}'
        ], capture_output=True, text=True)
        
        if result.returncode == 0:
            print("✅ 按键发送成功！")
            return True
        else:
            print(f"❌ 按键发送失败: {result.stderr}")
            return False
            
    except Exception as e:
        print(f"❌ 异常: {e}")
        return False

if __name__ == "__main__":
    test_direct_injection()