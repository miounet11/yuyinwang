use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AppError {
    // 音频相关错误
    AudioDeviceError(String),
    AudioRecordingError(String),
    AudioProcessingError(String),
    
    // 转录相关错误
    TranscriptionError(String),
    WhisperError(String),
    ApiTranscriptionError(String),
    
    // AI代理相关错误
    AiAgentError(String),
    OpenAiApiError(String),
    PromptProcessingError(String),
    
    // 数据库相关错误
    DatabaseError(String),
    DataSerializationError(String),
    
    // 文件系统相关错误
    FileSystemError(String),
    PathValidationError(String),
    
    // 网络相关错误
    NetworkError(String),
    HttpRequestError(String),
    
    // 权限相关错误
    PermissionError(String),
    AccessibilityError(String),
    SystemIntegrationError(String),
    
    // 配置相关错误
    ConfigurationError(String),
    
    // 快捷键相关错误
    ShortcutError(String),
    
    // 窗口相关错误
    WindowError(String),
    
    // 文本注入相关错误
    InjectionError(String),
    
    // IPC相关错误
    IpcError(String),
    
    // 系统错误
    SystemError(String),
    
    // 通用错误
    ValidationError(String),
    UnexpectedError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::AudioDeviceError(msg) => write!(f, "音频设备错误: {}", msg),
            AppError::AudioRecordingError(msg) => write!(f, "音频录制错误: {}", msg),
            AppError::AudioProcessingError(msg) => write!(f, "音频处理错误: {}", msg),
            AppError::TranscriptionError(msg) => write!(f, "转录错误: {}", msg),
            AppError::WhisperError(msg) => write!(f, "Whisper错误: {}", msg),
            AppError::ApiTranscriptionError(msg) => write!(f, "API转录错误: {}", msg),
            AppError::AiAgentError(msg) => write!(f, "AI代理错误: {}", msg),
            AppError::OpenAiApiError(msg) => write!(f, "OpenAI API错误: {}", msg),
            AppError::PromptProcessingError(msg) => write!(f, "提示词处理错误: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            AppError::DataSerializationError(msg) => write!(f, "数据序列化错误: {}", msg),
            AppError::FileSystemError(msg) => write!(f, "文件系统错误: {}", msg),
            AppError::PathValidationError(msg) => write!(f, "路径验证错误: {}", msg),
            AppError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            AppError::HttpRequestError(msg) => write!(f, "HTTP请求错误: {}", msg),
            AppError::PermissionError(msg) => write!(f, "权限错误: {}", msg),
            AppError::AccessibilityError(msg) => write!(f, "辅助功能权限错误: {}", msg),
            AppError::SystemIntegrationError(msg) => write!(f, "系统集成错误: {}", msg),
            AppError::ConfigurationError(msg) => write!(f, "配置错误: {}", msg),
            AppError::ShortcutError(msg) => write!(f, "快捷键错误: {}", msg),
            AppError::WindowError(msg) => write!(f, "窗口错误: {}", msg),
            AppError::InjectionError(msg) => write!(f, "文本注入错误: {}", msg),
            AppError::IpcError(msg) => write!(f, "IPC通信错误: {}", msg),
            AppError::SystemError(msg) => write!(f, "系统错误: {}", msg),
            AppError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            AppError::UnexpectedError(msg) => write!(f, "未预期的错误: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

// 类型别名，简化Result使用
pub type AppResult<T> = Result<T, AppError>;

// 实现从其他错误类型的转换
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::FileSystemError(error.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError::HttpRequestError(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::DataSerializationError(error.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        AppError::DatabaseError(error.to_string())
    }
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::UnexpectedError(error)
    }
}

impl From<&str> for AppError {
    fn from(error: &str) -> Self {
        AppError::UnexpectedError(error.to_string())
    }
}