use std::process::{Command, Output};
use std::collections::HashMap;

pub struct SecureCommandExecutor;

impl SecureCommandExecutor {
    /// 安全的AppleScript执行，只允许预定义的脚本
    pub fn execute_applescript(script_type: &str) -> Result<String, String> {
        let script = match script_type {
            "check_accessibility" => "tell application \"System Events\" to get UI elements enabled",
            "get_frontmost_app" => "tell application \"System Events\" to get name of first application process whose frontmost is true",
            "test_notification" => "display notification \"LuYinWang 测试通知\" with title \"权限测试\"",
            _ => return Err("不允许的脚本类型".to_string()),
        };
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("执行AppleScript失败: {}", e))?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }
    
    /// 安全的应用打开，只允许白名单URL
    pub fn open_system_preferences(preference_pane: &str) -> Result<(), String> {
        let allowed_panes = HashMap::from([
            ("microphone", "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"),
            ("accessibility", "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"),
            ("screen_recording", "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"),
            ("notifications", "x-apple.systempreferences:com.apple.preference.notifications"),
            ("automation", "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"),
        ]);
        
        let url = allowed_panes.get(preference_pane)
            .ok_or("不允许的系统偏好设置面板")?;
            
        Command::new("open")
            .arg(url)
            .output()
            .map_err(|e| format!("打开系统偏好设置失败: {}", e))?;
            
        Ok(())
    }
    
    /// 验证是否为允许的系统命令
    fn is_allowed_command(command: &str) -> bool {
        let allowed_commands = [
            "osascript",
            "open",
        ];
        allowed_commands.contains(&command)
    }
    
    /// 安全的命令参数验证
    fn validate_args(args: &[&str]) -> Result<(), String> {
        for arg in args {
            // 检查是否包含危险字符
            if arg.contains("&&") || arg.contains("||") || arg.contains(";") || 
               arg.contains("|") || arg.contains("`") || arg.contains("$(") {
                return Err("参数包含危险字符".to_string());
            }
            
            // 检查路径遍历
            if arg.contains("../") || arg.contains("..\\") {
                return Err("参数包含路径遍历字符".to_string());
            }
        }
        Ok(())
    }
    
    /// 安全执行系统命令的通用方法
    pub fn execute_safe_command(command: &str, args: &[&str]) -> Result<Output, String> {
        // 验证命令白名单
        if !Self::is_allowed_command(command) {
            return Err(format!("不允许执行的命令: {}", command));
        }
        
        // 验证参数安全性
        Self::validate_args(args)?;
        
        // 执行命令
        Command::new(command)
            .args(args)
            .output()
            .map_err(|e| format!("执行命令失败: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_validation() {
        // 测试允许的命令
        assert!(SecureCommandExecutor::is_allowed_command("osascript"));
        assert!(SecureCommandExecutor::is_allowed_command("open"));
        
        // 测试不允许的命令
        assert!(!SecureCommandExecutor::is_allowed_command("rm"));
        assert!(!SecureCommandExecutor::is_allowed_command("curl"));
        assert!(!SecureCommandExecutor::is_allowed_command("bash"));
    }
    
    #[test]
    fn test_args_validation() {
        // 测试安全参数
        assert!(SecureCommandExecutor::validate_args(&["-e", "test"]).is_ok());
        
        // 测试危险参数
        assert!(SecureCommandExecutor::validate_args(&["test && rm -rf /"]).is_err());
        assert!(SecureCommandExecutor::validate_args(&["../etc/passwd"]).is_err());
        assert!(SecureCommandExecutor::validate_args(&["$(whoami)"]).is_err());
    }
}