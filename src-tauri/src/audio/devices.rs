use cpal::traits::{DeviceTrait, HostTrait};
use crate::errors::{AppError, AppResult};
use crate::types::AudioDevice;

#[derive(Debug)]
pub struct AudioDeviceManager {
    host: cpal::Host,
}

impl AudioDeviceManager {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// 获取所有可用的输入设备
    pub fn get_input_devices(&self) -> AppResult<Vec<AudioDevice>> {
        let devices = self.host.input_devices()
            .map_err(|e| AppError::AudioDeviceError(format!("获取输入设备失败: {}", e)))?;

        let default_input = self.host.default_input_device();
        let default_input_name = default_input
            .as_ref()
            .and_then(|d| d.name().ok());

        let mut audio_devices = Vec::new();

        for device in devices {
            let name = device.name()
                .map_err(|e| AppError::AudioDeviceError(format!("获取设备名称失败: {}", e)))?;

            let is_default = default_input_name
                .as_ref()
                .map_or(false, |default_name| default_name == &name);

            // 检查设备可用性
            let is_available = self.check_device_availability(&device);

            let device_id = format!("input_{}", name.replace(" ", "_").to_lowercase());

            audio_devices.push(AudioDevice {
                name,
                id: device_id,
                is_default,
                is_available,
            });
        }

        Ok(audio_devices)
    }

    /// 获取所有可用的输出设备
    pub fn get_output_devices(&self) -> AppResult<Vec<AudioDevice>> {
        let devices = self.host.output_devices()
            .map_err(|e| AppError::AudioDeviceError(format!("获取输出设备失败: {}", e)))?;

        let default_output = self.host.default_output_device();
        let default_output_name = default_output
            .as_ref()
            .and_then(|d| d.name().ok());

        let mut audio_devices = Vec::new();

        for device in devices {
            let name = device.name()
                .map_err(|e| AppError::AudioDeviceError(format!("获取设备名称失败: {}", e)))?;

            let is_default = default_output_name
                .as_ref()
                .map_or(false, |default_name| default_name == &name);

            let is_available = self.check_device_availability(&device);
            let device_id = format!("output_{}", name.replace(" ", "_").to_lowercase());

            audio_devices.push(AudioDevice {
                name,
                id: device_id,
                is_default,
                is_available,
            });
        }

        Ok(audio_devices)
    }

    /// 根据设备ID获取输入设备
    pub fn get_input_device_by_id(&self, device_id: &str) -> AppResult<Option<cpal::Device>> {
        let devices = self.host.input_devices()
            .map_err(|e| AppError::AudioDeviceError(format!("获取输入设备失败: {}", e)))?;

        for device in devices {
            let name = device.name()
                .map_err(|e| AppError::AudioDeviceError(format!("获取设备名称失败: {}", e)))?;
            
            let current_device_id = format!("input_{}", name.replace(" ", "_").to_lowercase());
            
            if current_device_id == device_id {
                return Ok(Some(device));
            }
        }

        Ok(None)
    }

    /// 获取默认输入设备
    pub fn get_default_input_device(&self) -> AppResult<Option<cpal::Device>> {
        Ok(self.host.default_input_device())
    }

    /// 获取默认输出设备
    pub fn get_default_output_device(&self) -> AppResult<Option<cpal::Device>> {
        Ok(self.host.default_output_device())
    }

    /// 检查设备可用性
    fn check_device_availability(&self, device: &cpal::Device) -> bool {
        // 尝试获取设备配置来验证可用性
        device.default_input_config().is_ok() || device.default_output_config().is_ok()
    }

    /// 获取设备配置信息
    pub fn get_device_config(&self, device: &cpal::Device) -> AppResult<cpal::SupportedStreamConfig> {
        device.default_input_config()
            .or_else(|_| device.default_output_config())
            .map_err(|e| AppError::AudioDeviceError(format!("获取设备配置失败: {}", e)))
    }

    /// 测试设备是否可以录音
    pub fn test_input_device(&self, device: &cpal::Device) -> AppResult<bool> {
        let config = device.default_input_config()
            .map_err(|e| AppError::AudioDeviceError(format!("获取输入配置失败: {}", e)))?;

        // 尝试创建一个短期的测试流
        let test_stream = device.build_input_stream(
            &config.into(),
            |_data: &[f32], _info: &cpal::InputCallbackInfo| {
                // 什么都不做，只是测试
            },
            |err| {
                eprintln!("测试流错误: {}", err);
            },
            None
        );

        match test_stream {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("测试输入设备失败: {}", e);
                Ok(false)
            }
        }
    }

    /// 获取系统音频权限状态（macOS）
    #[cfg(target_os = "macos")]
    pub fn check_microphone_permission() -> AppResult<bool> {
        use std::process::Command;

        let output = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get microphone access")
            .output()
            .map_err(|e| AppError::PermissionError(format!("检查麦克风权限失败: {}", e)))?;

        Ok(output.status.success())
    }

    /// 请求麦克风权限（macOS）
    #[cfg(target_os = "macos")]
    pub fn request_microphone_permission(&self) -> AppResult<()> {
        // 在macOS上，第一次访问麦克风会自动弹出权限请求
        // 这里我们通过尝试创建一个测试设备来触发权限请求
        let default_input = self.host.default_input_device()
            .ok_or_else(|| AppError::AudioDeviceError("没有默认输入设备".to_string()))?;

        let _config = default_input.default_input_config()
            .map_err(|e| AppError::AudioDeviceError(format!("无法访问输入设备: {}", e)))?;

        println!("🎤 已请求麦克风权限，请在系统设置中允许");
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn check_microphone_permission() -> AppResult<bool> {
        // 其他平台通常不需要显式权限检查
        Ok(true)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn request_microphone_permission() -> AppResult<()> {
        // 其他平台不需要请求权限
        Ok(())
    }
}

impl Default for AudioDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_input_devices() {
        let manager = AudioDeviceManager::new();
        let devices = manager.get_input_devices();
        
        // 测试不应该失败（即使没有设备）
        assert!(devices.is_ok());
    }

    #[test]
    fn test_get_output_devices() {
        let manager = AudioDeviceManager::new();
        let devices = manager.get_output_devices();
        
        // 测试不应该失败（即使没有设备）
        assert!(devices.is_ok());
    }
}