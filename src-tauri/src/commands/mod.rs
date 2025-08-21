// 命令模块 - 统一管理所有Tauri命令

use tauri::{State, Manager};
use crate::types::*;
use crate::{AppState, ai_agent};
use std::path::Path;

pub mod history;
pub mod transcription;
pub mod subtitle;
pub mod permissions;
pub mod text_injection;
pub mod shortcut_management;
pub mod shortcuts;
pub mod floating_assistant;
pub mod voice_input;

pub use history::*;
pub use transcription::*;
pub use subtitle::*;
pub use permissions::*;
pub use text_injection::*;
pub use shortcut_management::*;
pub use shortcuts::*;
pub use floating_assistant::*;
pub use voice_input::*;

// 辅助函数

/// 创建WAV文件
fn create_wav_file<P: AsRef<Path>>(
    path: P,
    audio_data: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    use hound::{WavWriter, WavSpec, SampleFormat};
    
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create(path, spec)?;
    
    for &sample in audio_data {
        // 将 f32 转换为 i16
        let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)?;
    }
    
    writer.finalize()?;
    Ok(())
}

/// 重采样音频数据
fn resample_audio(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return input.to_vec();
    }
    
    let ratio = to_rate as f32 / from_rate as f32;
    let output_len = (input.len() as f32 * ratio) as usize;
    let mut output = Vec::with_capacity(output_len);
    
    // 简单的线性插值重采样
    for i in 0..output_len {
        let src_pos = i as f32 / ratio;
        let src_idx = src_pos as usize;
        
        if src_idx >= input.len() - 1 {
            output.push(input[input.len() - 1]);
        } else {
            let frac = src_pos - src_idx as f32;
            let sample = input[src_idx] * (1.0 - frac) + input[src_idx + 1] * frac;
            output.push(sample);
        }
    }
    
    output
}

// 基础功能命令

#[tauri::command]
pub async fn transcribe_file(
    state: State<'_, AppState>,
    file_path: String,
    model: String,
) -> Result<TranscriptionResult, String> {
    let config = TranscriptionConfig {
        model_name: model.clone(),
        language: Some(if model.starts_with("whisper-") { "zh".to_string() } else { "auto".to_string() }),
        temperature: Some(0.0),
        is_local: model.starts_with("whisper-") && model != "whisper-1",
        api_endpoint: None,
    };
    
    match state.transcription_service.transcribe_audio(&file_path, &config).await {
        Ok(result) => {
            println!("✅ 转录成功: {}", result.text);
            
            // 保存到数据库
            let entry = TranscriptionEntry {
                id: uuid::Uuid::new_v4().to_string(),
                text: result.text.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64,
                duration: 0.0,
                model: model,
                confidence: 0.95,
                audio_file_path: Some(file_path),
                created_at: None,
                updated_at: None,
                tags: None,
                metadata: None,
            };
            
            if let Err(e) = state.database.insert_transcription(&entry) {
                eprintln!("保存转录记录失败: {}", e);
            }
            
            Ok(result)
        },
        Err(e) => {
            eprintln!("转录失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_transcription_history(
    state: State<'_, AppState>,
) -> Result<Vec<TranscriptionEntry>, String> {
    match state.database.get_all_transcriptions() {
        Ok(history) => Ok(history),
        Err(e) => {
            eprintln!("获取转录历史失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn process_ai_agent(
    state: State<'_, AppState>,
    request: AgentRequest,
) -> Result<AgentResponse, String> {
    let ai_request = ai_agent::AIAgentRequest {
        text: request.input_text,
        agent_type: match request.agent_type.as_str() {
            "text-enhancer" => ai_agent::AIAgentType::TextEnhancement,
            "translator" => ai_agent::AIAgentType::Translation,
            "summarizer" => ai_agent::AIAgentType::Summarization,
            "grammar-check" => ai_agent::AIAgentType::GrammarCorrection,
            _ => ai_agent::AIAgentType::Custom,
        },
        options: request.additional_context.unwrap_or_default(),
        context: None,
    };
    
    let service = state.ai_agent_service.clone();
    let agent_type = request.agent_type.clone();
    
    // 在tokio任务中处理以避免Send问题
    match tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            service.lock().process_agent_request(ai_request).await
        })
    }).await {
        Ok(Ok(response)) => {
            Ok(AgentResponse {
                success: response.success,
                output_text: response.processed_text,
                agent_type,
                processing_time_ms: response.processing_time_ms,
                error: response.error,
            })
        },
        Ok(Err(e)) => {
            eprintln!("AI代理处理失败: {}", e);
            Err(e.to_string())
        },
        Err(e) => {
            eprintln!("AI代理任务执行失败: {}", e);
            Err(format!("任务执行失败: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_audio_devices(
    state: State<'_, AppState>,
) -> Result<Vec<AudioDevice>, String> {
    match state.audio_device_manager.get_input_devices() {
        Ok(devices) => {
            println!("🎤 可用音频输入设备:");
            for (i, device) in devices.iter().enumerate() {
                println!("  {}. {} (ID: {}, 默认: {}, 可用: {})", 
                    i + 1, device.name, device.id, device.is_default, device.is_available);
            }
            Ok(devices)
        },
        Err(e) => {
            eprintln!("获取音频设备失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn test_audio_input(
    state: State<'_, AppState>,
    _device_id: Option<String>,
    duration_seconds: Option<f32>,
) -> Result<String, String> {
    let test_duration = duration_seconds.unwrap_or(3.0);
    println!("🧪 开始音频输入测试，持续时间: {:.1}秒", test_duration);
    
    // 启动录音测试
    let start_result = {
        let mut recorder = state.audio_recorder.lock();
        recorder.start_recording()
    };
    
    match start_result {
        Ok(_) => {
            println!("✅ 录音测试已启动");
            
            // 等待指定时间
            tokio::time::sleep(tokio::time::Duration::from_millis((test_duration * 1000.0) as u64)).await;
            
            // 停止录音并分析
            let stop_result = {
                let mut recorder = state.audio_recorder.lock();
                recorder.stop_recording()
            };
            
            match stop_result {
                Ok(audio_data) => {
                    let audio_max = audio_data.iter().map(|&x| x.abs()).fold(0.0, f32::max);
                    let audio_rms = (audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
                    let sample_count = audio_data.len();
                    
                    println!("📊 音频测试结果:");
                    println!("   - 样本数: {}", sample_count);
                    println!("   - 最大音量: {:.4}", audio_max);
                    println!("   - RMS音量: {:.4}", audio_rms);
                    println!("   - 持续时间: {:.2}秒", sample_count as f32 / 16000.0);
                    
                    let result = if audio_max < 0.01 {
                        "❌ 音频输入异常：音量过低，请检查麦克风设置和权限"
                    } else if audio_rms < 0.005 {
                        "⚠️ 音频输入较弱：建议提高麦克风音量或靠近麦克风"
                    } else {
                        "✅ 音频输入正常"
                    };
                    
                    Ok(format!("{}\n样本数: {}, 最大音量: {:.4}, RMS音量: {:.4}", 
                        result, sample_count, audio_max, audio_rms))
                },
                Err(e) => {
                    Err(format!("停止录音测试失败: {}", e))
                }
            }
        },
        Err(e) => {
            Err(format!("启动录音测试失败: {}", e))
        }
    }
}

#[tauri::command]
pub async fn start_recording(
    state: State<'_, AppState>,
    device_id: Option<String>,
) -> Result<String, String> {
    let mut is_recording = state.is_recording.lock();
    if *is_recording {
        return Err("已在录音中".to_string());
    }
    
    // 获取录音器引用
    let mut recorder = state.audio_recorder.lock();
    
    // 启动真实的录音
    match recorder.start_recording() {
        Ok(_) => {
            *is_recording = true;
            println!("🎙️ 录音已启动，使用设备: {:?}", device_id.as_deref().unwrap_or("默认设备"));
            Ok("录音已开始".to_string())
        },
        Err(e) => {
            Err(format!("启动录音失败: {}", e))
        }
    }
}

#[tauri::command]
pub async fn stop_recording(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    model: Option<String>,
) -> Result<String, String> {
    let mut is_recording = state.is_recording.lock();
    if !*is_recording {
        return Err("当前没有在录音".to_string());
    }
    
    // 获取录音器引用并停止录音
    let mut recorder = state.audio_recorder.lock();
    
    // 获取实际的采样率
    let actual_sample_rate = recorder.get_sample_rate();
    
    match recorder.stop_recording() {
        Ok(audio_data) => {
            *is_recording = false;
            println!("🛑 录音已停止，捕获了 {} 个音频样本", audio_data.len());
            println!("📊 实际采样率: {} Hz", actual_sample_rate);
            
            // 自动进行转录
            if !audio_data.is_empty() {
                println!("🎤 开始转录音频数据...");
                
                // 保存音频数据到临时文件
                let temp_dir = std::env::temp_dir();
                let temp_file = temp_dir.join(format!("recording_{}.wav", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()));
                
                // 音频质量分析
                let audio_max = audio_data.iter().map(|&x| x.abs()).fold(0.0, f32::max);
                let audio_rms = (audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
                println!("🎵 音频质量分析: 最大音量={:.4}, RMS音量={:.4}, 样本数={}", audio_max, audio_rms, audio_data.len());
                
                if audio_max < 0.01 {
                    println!("⚠️ 警告：音频音量过低 (最大值={:.4})，可能影响转录质量", audio_max);
                }
                if audio_rms < 0.005 {
                    println!("⚠️ 警告：音频信号较弱 (RMS={:.4})，建议提高麦克风音量或靠近说话", audio_rms);
                }
                
                // 如果采样率不是16kHz，进行重采样以兼容转录服务
                let (audio_for_transcription, transcription_sample_rate) = if actual_sample_rate != 16000 {
                    println!("🔄 重采样音频从 {} Hz 到 16000 Hz 以兼容转录服务", actual_sample_rate);
                    let resampled = crate::commands::resample_audio(&audio_data, actual_sample_rate, 16000);
                    (resampled, 16000)
                } else {
                    (audio_data.clone(), actual_sample_rate)
                };
                
                // 创建WAV文件 - 使用16kHz采样率以兼容转录服务
                match crate::commands::create_wav_file(&temp_file, &audio_for_transcription, transcription_sample_rate, 1) {
                    Ok(_) => {
                        println!("📁 音频文件已保存: {:?}", temp_file);
                        
                        // 使用用户选择的模型或默认配置进行转录
                        let selected_model = model.unwrap_or_else(|| "whisper-tiny".to_string());
                        println!("🔧 用户选择的模型: {}", selected_model);
                        let config = match selected_model.as_str() {
                            "luyingwang-online" => TranscriptionConfig {
                                model_name: "luyin-api".to_string(),
                                language: Some("auto".to_string()),
                                temperature: Some(0.0),
                                is_local: false,
                                api_endpoint: None,
                            },
                            "gpt-4o-mini" => TranscriptionConfig {
                                model_name: "gpt-4o-mini".to_string(),
                                language: Some("auto".to_string()),
                                temperature: Some(0.0),
                                is_local: false,
                                api_endpoint: None,
                            },
                            model_name if model_name.starts_with("whisper-") => TranscriptionConfig {
                                model_name: model_name.to_string(),
                                language: Some("zh".to_string()), // 指定中文语言，避免误识别为西班牙语
                                temperature: Some(0.0),
                                is_local: true,
                                api_endpoint: None,
                            },
                            _ => TranscriptionConfig {
                                model_name: "whisper-tiny".to_string(),
                                language: Some("zh".to_string()), // 默认也指定中文
                                temperature: Some(0.0),
                                is_local: true,
                                api_endpoint: None,
                            },
                        };
                        
                        // 异步进行转录
                        let transcription_service = state.transcription_service.clone();
                        let temp_file_path = temp_file.to_string_lossy().to_string();
                        let database = state.database.clone();
                        let app_handle_clone = app_handle.clone();
                        
                        tokio::spawn(async move {
                            match transcription_service.transcribe_audio(&temp_file_path, &config).await {
                                Ok(result) => {
                                    println!("✅ 转录完成: {}", result.text);
                                    
                                    // 保存到数据库
                                    let entry = TranscriptionEntry {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        text: result.text.clone(),
                                        timestamp: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap()
                                            .as_millis() as i64,
                                        duration: 0.0,
                                        model: config.model_name,
                                        confidence: 0.95,
                                        audio_file_path: Some(temp_file_path.clone()),
                                        created_at: None,
                                        updated_at: None,
                                        tags: None,
                                        metadata: None,
                                    };
                                    
                                    if let Err(e) = database.insert_transcription(&entry) {
                                        eprintln!("保存转录记录失败: {}", e);
                                    }
                                    
                                    // 发送转录结果事件到前端
                                    match app_handle_clone.emit_all("transcription_result", &entry) { Err(e) => {
                                        eprintln!("发送转录结果事件失败: {}", e);
                                    } _ => {
                                        println!("✅ 转录结果事件已发送到前端");
                                    }}
                                    
                                    // 清理临时文件
                                    if let Err(e) = std::fs::remove_file(&temp_file_path) {
                                        eprintln!("清理临时文件失败: {}", e);
                                    }
                                },
                                Err(e) => {
                                    eprintln!("转录失败: {}", e);
                                    
                                    // 发送转录错误事件到前端
                                    if let Err(emit_error) = app_handle_clone.emit_all("transcription_error", &e.to_string()) {
                                        eprintln!("发送转录错误事件失败: {}", emit_error);
                                    }
                                    
                                    // 清理临时文件
                                    if let Err(e) = std::fs::remove_file(&temp_file_path) {
                                        eprintln!("清理临时文件失败: {}", e);
                                    }
                                }
                            }
                        });
                    },
                    Err(e) => {
                        eprintln!("保存音频文件失败: {}", e);
                    }
                }
            }
            
            Ok(format!("录音已停止，录制了 {:.2} 秒音频，正在转录...", audio_data.len() as f32 / actual_sample_rate as f32))
        },
        Err(e) => {
            *is_recording = false; // 确保状态正确
            Err(format!("停止录音失败: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_app_settings(
    state: State<'_, AppState>,
) -> Result<crate::config::AppSettings, String> {
    Ok(state.settings.lock().clone())
}

#[tauri::command]
pub async fn update_app_settings(
    state: State<'_, AppState>,
    settings: crate::config::AppSettings,
) -> Result<(), String> {
    match settings.save() {
        Ok(_) => {
            *state.settings.lock() = settings;
            Ok(())
        },
        Err(e) => {
            eprintln!("保存设置失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_recording_state(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let is_recording = *state.is_recording.lock();
    println!("📊 获取录音状态: {}", is_recording);
    Ok(is_recording)
}

#[tauri::command]
pub async fn reset_recording_state(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut is_recording = state.is_recording.lock();
    let was_recording = *is_recording;
    *is_recording = false;
    
    // 同时重置录音器状态
    {
        let mut recorder = state.audio_recorder.lock();
        // 强制重置录音器状态，无论当前是否在录音
        recorder.force_reset();
    }
    
    println!("🔄 重置录音状态: {} -> false", was_recording);
    Ok(format!("录音状态已重置: {} -> false", was_recording))
}

#[tauri::command]
pub async fn track_previous_app(
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 获取当前活动应用（不应该是Recording King）
    let injector = crate::system::TextInjector::default();
    match injector.get_active_application_info().await {
        Ok(app_info) => {
            // 检查是否是Recording King自身
            if !app_info.bundle_id.contains("recordingking") && !app_info.name.contains("Recording King") {
                let mut previous_app = state.previous_active_app.lock();
                *previous_app = Some(app_info.clone());
                println!("📱 记录前一个活动应用: {} ({})", app_info.name, app_info.bundle_id);
                Ok(())
            } else {
                // 如果是Recording King，不更新
                println!("⚠️ 检测到Recording King自身，不更新前一个应用");
                Ok(())
            }
        }
        Err(e) => {
            eprintln!("❌ 获取活动应用信息失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn smart_inject_text_with_app_switch(
    state: State<'_, AppState>,
    text: String,
    config: Option<text_injection::TextInjectionConfigDto>,
) -> Result<bool, String> {
    println!("🔄 智能文本注入（带应用切换）");
    
    // 检查是否有记录的前一个应用
    let target_app = {
        let previous_app = state.previous_active_app.lock();
        previous_app.clone()
    };
    
    if let Some(app_info) = target_app {
        println!("🎯 目标应用: {} ({})", app_info.name, app_info.bundle_id);
        
        // 激活目标应用
        #[cfg(target_os = "macos")]
        {
            // 注意：activate_app_by_bundle_id 功能已在简化重构中移除
            // 应用切换将依赖系统自然的窗口焦点切换
            println!("ℹ️ 准备切换到目标应用: {}", app_info.bundle_id);
        }
        
        // 等待应用切换完成
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    } else {
        println!("⚠️ 没有记录的前一个应用，将注入到当前活动应用");
    }
    
    // 调用原有的智能注入功能
    text_injection::smart_inject_text(text, config).await
}