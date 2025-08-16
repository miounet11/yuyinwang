#!/usr/bin/env python3
import requests
import time
import json
import sys

# API配置
BASE_URL = "https://ly.gl173.com"
BEARER_TOKEN = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJodHRwczovL3JlY29yZC10by10ZXh0LmNvbS9hcGkvdjEvbG9nb3V0IiwiaWF0IjoxNzUzODU4NzIxLCJleHAiOjE3NjI0OTg3MjEsIm5iZiI6MTc1Mzg1ODcyMSwianRpIjoiNTlZQjBUMExqWGV4NGZqdiIsInN1YiI6IjEiLCJwcnYiOiIyM2JkNWM4OTQ5ZjYwMGFkYjM5ZTcwMWM0MDA4NzJkYjdhNTk3NmY3IiwiZGV2aWNlX2lkIjoiYmYyZTdkODU4NWU0YmM3YTFjY2VmNWE0YzI2OTkxZDQiLCJpc19sb2dpbiI6MH0.NxgG2hysvK7we4QuyNwpNoX5etfvHTW4ZqL8s1T-5oc"

headers = {
    "Authorization": f"Bearer {BEARER_TOKEN}"
}

def test_api_health():
    """测试API是否可以访问"""
    print("🔍 测试API连接...")
    try:
        # 先创建一个简单的音频文件用于测试
        import subprocess
        import os
        
        # 使用macOS的say命令生成一个测试音频
        test_file = "/tmp/test_audio.wav"
        subprocess.run(["say", "-o", test_file, "--data-format=LEF32@22050", "Hello, this is a test"], check=True)
        print(f"✅ 创建测试音频文件: {test_file}")
        
        # 1. 上传文件
        print("\n📤 步骤1: 上传文件...")
        with open(test_file, 'rb') as f:
            files = {'file[]': ('test_audio.wav', f, 'audio/wav')}
            response = requests.post(
                f"{BASE_URL}/api/v1/upload-file",
                headers=headers,
                files=files,
                timeout=30
            )
        
        print(f"状态码: {response.status_code}")
        upload_result = response.json()
        print(f"响应: {json.dumps(upload_result, indent=2, ensure_ascii=False)}")
        
        if upload_result.get('code') != 200:
            print(f"❌ 上传失败: {upload_result}")
            return False
            
        file_id = upload_result['data'][0]['file_id']
        print(f"✅ 文件上传成功，file_id: {file_id}")
        
        # 2. 创建转换任务
        print("\n🔄 步骤2: 创建转换任务...")
        response = requests.post(
            f"{BASE_URL}/api/v1/task-add",
            headers=headers,
            data={'file_id': str(file_id)},
            timeout=30
        )
        
        print(f"状态码: {response.status_code}")
        task_result = response.json()
        print(f"响应: {json.dumps(task_result, indent=2, ensure_ascii=False)}")
        
        if task_result.get('code') != 200:
            print(f"❌ 创建任务失败: {task_result}")
            return False
            
        task_id = task_result['data']['task_id']
        print(f"✅ 任务创建成功，task_id: {task_id}")
        
        # 3. 查询任务进度
        print("\n⏳ 步骤3: 查询任务进度...")
        max_attempts = 30
        for attempt in range(max_attempts):
            time.sleep(2)  # 等待2秒
            
            response = requests.post(
                f"{BASE_URL}/api/v1/task-progress",
                headers=headers,
                data={'task_id': task_id},
                timeout=30
            )
            
            progress_result = response.json()
            print(f"尝试 {attempt+1}/{max_attempts}: {progress_result.get('code')}")
            
            if progress_result.get('code') != 200:
                print(f"❌ 查询失败: {progress_result}")
                continue
                
            progress = progress_result['data'].get('progress', 0)
            print(f"进度: {progress * 100}%")
            
            if progress == 1:
                result_text = progress_result['data'].get('result', '')
                print(f"\n✅ 转换完成！")
                print(f"转录结果: {result_text}")
                return True
            elif progress == 0:
                print("仍在转换中...")
            else:
                print(f"未知进度值: {progress}")
                
        print("❌ 转换超时")
        return False
        
    except requests.exceptions.RequestException as e:
        print(f"❌ 网络请求错误: {e}")
        return False
    except Exception as e:
        print(f"❌ 发生错误: {e}")
        return False
    finally:
        # 清理测试文件
        if 'test_file' in locals() and os.path.exists(test_file):
            os.remove(test_file)
            print(f"\n🧹 清理测试文件: {test_file}")

if __name__ == "__main__":
    print("=" * 50)
    print("录音王 API 测试工具")
    print("=" * 50)
    
    success = test_api_health()
    
    print("\n" + "=" * 50)
    if success:
        print("✅ API 测试通过！所有功能正常工作")
    else:
        print("❌ API 测试失败！请检查Token或网络连接")
    print("=" * 50)
    
    sys.exit(0 if success else 1)