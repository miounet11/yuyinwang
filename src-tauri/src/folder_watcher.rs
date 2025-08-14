/**
 * Recording King 文件夹监控模块
 * 监控指定文件夹中的新音频文件并自动转录
 */

use notify::{Watcher, RecursiveMode, Event, RecommendedWatcher, Config};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Manager};
use std::collections::HashSet;

#[derive(Debug)]
pub struct FolderWatcher {
    watched_folders: Arc<Mutex<HashSet<PathBuf>>>,
    supported_extensions: Vec<String>,
    app_handle: Option<AppHandle>,
}

impl FolderWatcher {
    pub fn new() -> Self {
        Self {
            watched_folders: Arc::new(Mutex::new(HashSet::new())),
            supported_extensions: vec![
                "mp3".to_string(),
                "wav".to_string(),
                "m4a".to_string(),
                "flac".to_string(),
                "mp4".to_string(),
                "mov".to_string(),
                "m4v".to_string(),
                "aac".to_string(),
                "ogg".to_string(),
            ],
            app_handle: None,
        }
    }
    
    pub fn initialize(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    pub fn add_folder(&self, folder_path: PathBuf) -> Result<(), String> {
        if self.app_handle.is_none() {
            return Err("文件夹监控器未初始化".to_string());
        }

        if !folder_path.exists() {
            return Err("文件夹不存在".to_string());
        }

        if !folder_path.is_dir() {
            return Err("路径不是文件夹".to_string());
        }

        {
            let mut folders = self.watched_folders.lock().unwrap();
            if folders.contains(&folder_path) {
                return Err("文件夹已在监控中".to_string());
            }
            folders.insert(folder_path.clone());
        }

        self.start_watching(folder_path)?;
        Ok(())
    }

    pub fn remove_folder(&self, folder_path: &PathBuf) -> Result<(), String> {
        let mut folders = self.watched_folders.lock().unwrap();
        if folders.remove(folder_path) {
            Ok(())
        } else {
            Err("文件夹未在监控中".to_string())
        }
    }

    pub fn get_watched_folders(&self) -> Vec<PathBuf> {
        let folders = self.watched_folders.lock().unwrap();
        folders.iter().cloned().collect()
    }

    fn start_watching(&self, folder_path: PathBuf) -> Result<(), String> {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())
            .map_err(|e| format!("创建文件监控器失败: {}", e))?;

        watcher.watch(&folder_path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("监控文件夹失败: {}", e))?;

        let app_handle = self.app_handle.as_ref().unwrap().clone();
        let supported_extensions = self.supported_extensions.clone();
        let watched_folders = Arc::clone(&self.watched_folders);

        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(result) => {
                        match result {
                            Ok(event) => {
                                if let Err(e) = Self::handle_file_event(
                                    event,
                                    &app_handle,
                                    &supported_extensions,
                                    &watched_folders,
                                ) {
                                    eprintln!("处理文件事件失败: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("文件监控事件错误: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("文件监控接收错误: {}", e);
                        break;
                    }
                }
            }
        });

        println!("✅ 开始监控文件夹: {:?}", folder_path);
        Ok(())
    }

    fn handle_file_event(
        event: Event,
        app_handle: &AppHandle,
        supported_extensions: &[String],
        watched_folders: &Arc<Mutex<HashSet<PathBuf>>>,
    ) -> Result<(), String> {
        use notify::EventKind;
        
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                        let ext_lower = extension.to_lowercase();
                        if supported_extensions.contains(&ext_lower) {
                            // 检查文件所在目录是否仍在监控中
                            if let Some(parent) = path.parent() {
                                let folders = watched_folders.lock().unwrap();
                                if folders.contains(parent) {
                                    Self::process_new_audio_file(path, app_handle)?;
                                }
                            }
                        }
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    println!("🗑️ 文件被删除: {:?}", path);
                    // 可以选择从历史记录中标记为已删除
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn process_new_audio_file(file_path: PathBuf, app_handle: &AppHandle) -> Result<(), String> {
        println!("🎵 发现新音频文件: {:?}", file_path);
        
        // 等待文件写入完成（避免处理正在写入的文件）
        thread::sleep(Duration::from_secs(1));
        
        // 检查文件是否可读
        if !file_path.exists() {
            return Err("文件不存在".to_string());
        }

        // 发送事件到前端
        let file_path_str = file_path.to_string_lossy().to_string();
        if let Err(e) = app_handle.emit_all("folder_watcher_new_file", &file_path_str) {
            eprintln!("发送文件事件失败: {}", e);
        }

        // 自动开始转录（如果启用了自动转录）
        Self::auto_transcribe_file(file_path, app_handle)
    }

    fn auto_transcribe_file(file_path: PathBuf, app_handle: &AppHandle) -> Result<(), String> {
        let file_path_str = file_path.to_string_lossy().to_string();
        
        // 获取文件大小，避免处理太大的文件
        if let Ok(metadata) = std::fs::metadata(&file_path) {
            let file_size = metadata.len();
            const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
            
            if file_size > MAX_FILE_SIZE {
                println!("⚠️ 文件过大，跳过自动转录: {:?} ({} bytes)", file_path, file_size);
                return Ok(());
            }
        }

        println!("🚀 开始自动转录文件: {:?}", file_path);
        
        // 发送自动转录事件到前端
        if let Err(e) = app_handle.emit_all("folder_watcher_auto_transcribe", &file_path_str) {
            eprintln!("发送自动转录事件失败: {}", e);
        }

        Ok(())
    }

    pub fn is_watching(&self, folder_path: &PathBuf) -> bool {
        let folders = self.watched_folders.lock().unwrap();
        folders.contains(folder_path)
    }

    pub fn clear_all(&self) {
        let mut folders = self.watched_folders.lock().unwrap();
        folders.clear();
        println!("🧹 已清空所有监控文件夹");
    }

    pub fn get_folder_stats(&self) -> (usize, Vec<String>) {
        let folders = self.watched_folders.lock().unwrap();
        let count = folders.len();
        let paths = folders.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        (count, paths)
    }
}

// 文件夹监控配置
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FolderWatcherConfig {
    pub enabled: bool,
    pub auto_transcribe: bool,
    pub max_file_size_mb: u64,
    pub debounce_seconds: u64,
    pub recursive: bool,
}

impl Default for FolderWatcherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_transcribe: true,
            max_file_size_mb: 100,
            debounce_seconds: 2,
            recursive: false,
        }
    }
}