 录音转文字功能总结

域名https://ly.gl173.com/

  核心API接口（3个主要接口）

  1. 文件上传接口 - POST /api/v1/upload-file
    - 上传音频文件到服务器
    - 返回file_id用于后续转换
参数
file[] 文件
响应
{
    "code": 200,
    "data": [
        {
            "uid": 1,
            "product_id": "1090",
            "file_name": "飞书20250722-114157.mp4",
            "file_time": 22,
            "file_size": 3704778,
            "file_ext": "mp4",
            "file_path": "http://file.inping.com:9000/luyin/fileToTxt/1/飞书20250722-114157.mp4",
            "create_time": 1755140127,
            "file_id": 3022852
        }
    ],
    "message": "一共备份了1个文件，成功了1个文件"
}

  2. 创建转换任务接口 - POST /api/v1/task-add
    - 根据file_id创建转换任务
    - 返回task_id用于查询进度
传参
file_id
响应
{
    "code": 200,
    "data": {
        "task_id": "7e5df68e00b4fa3291419f0a65536a43"
    },
    "message": "success"
}
  3. 查询转换进度接口 - POST /api/v1/task-progress
    - 根据task_id查询转换进度和结果
    - 轮询调用直到转换完成
传参
task_id
响应 progress 1 完成 0 转换中 result 转换结果
{
    "code": 200,
    "data": {
        "task_id": "ef43b329f2539e1a6168c4a6e426c3c9",
        "progress": 1,
        "result": "10秒钟,6秒,7秒。"
    },
    "message": "success"
}
  转换流程逻辑

  用户点击转换 → 文件上传 → 创建任务 → 轮询进度 → 获取结果

  具体步骤：

  1. 文件上传 (uploadAudioFile)
    - 将本地音频文件封装为MultipartBody
    - 调用上传API获取file_id
  2. 创建转换任务 (createConversionTask)
    - 使用file_id创建转换任务
    - 获取task_id
  3. 轮询进度查询 (pollTaskProgress)
    - 每3秒查询一次转换进度
    - 最多轮询60次（3分钟超时）
    - progress=0：转换中
    - progress=1：转换完成，获取result

  3. 状态管理

  - ConversionStatus枚举：
    - UPLOADING：上传中
    - UPLOADED：上传完成
    - CONVERTING：转换中
    - COMPLETED：转换完成
    - FAILED：转换失败

  4. UI更新逻辑

  - 转换过程中禁用按钮，显示进度
  - 成功后保存结果到数据库，显示转换文字
  - 失败后恢复按钮状态，显示错误信息


  这个设计采用了异步+轮询的方式处理长时间的音频转换任务，确保用户体验的同时保证转换的可靠性。