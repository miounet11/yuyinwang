#!/usr/bin/env python3
"""
使用 shortcuts 命令测试文本注入
"""

import subprocess
import time

def test_shortcuts_injection():
    """使用 shortcuts 命令测试文本注入"""
    print("🚀 使用 shortcuts 命令测试文本注入...")
    
    # 设置测试文本
    test_text = "Shortcuts注入测试888"
    print(f"📝 测试文本: '{test_text}'")
    
    # 写入剪贴板
    subprocess.run(['pbcopy'], input=test_text, text=True)
    print("✅ 文本已写入剪贴板")
    
    print("⏰ 请在5秒内切换到目标应用...")
    for i in range(5, 0, -1):
        print(f"⏳ {i}...")
        time.sleep(1)
    
    print("🚀 尝试使用 shortcuts 发送粘贴命令...")
    
    try:
        # 尝试创建和运行一个简单的快捷指令
        shortcut_script = """
on run
    tell application "System Events"
        keystroke "v" using {command down}
    end tell
end run
"""
        
        # 先尝试简单的按键发送
        result = subprocess.run([
            'shortcuts', 'run', 'Paste'
        ], capture_output=True, text=True, timeout=5)
        
        if result.returncode == 0:
            print("✅ Shortcuts 粘贴成功！")
            return True
        else:
            print(f"❌ Shortcuts 失败: {result.stderr}")
            
            # 如果预设的快捷指令不存在，尝试其他方法
            print("🔄 尝试其他方法...")
            return False
            
    except subprocess.TimeoutExpired:
        print("❌ Shortcuts 超时")
        return False
    except Exception as e:
        print(f"❌ 异常: {e}")
        return False

if __name__ == "__main__":
    test_shortcuts_injection()