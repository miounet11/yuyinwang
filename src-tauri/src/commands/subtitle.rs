// 字幕生成相关的Tauri命令
use tauri::State;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::subtitle::{
    SubtitleGenerator, SubtitleFormat, SubtitleOptions, SubtitleStatistics
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateSubtitleRequest {
    pub transcription_id: String,
    pub format: String, // "srt", "vtt", "ass", "txt"
    pub output_path: String,
    pub options: Option<SubtitleOptionsRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchGenerateSubtitleRequest {
    pub transcription_ids: Vec<String>,
    pub format: String,
    pub output_directory: String,
    pub options: Option<SubtitleOptionsRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeSubtitleRequest {
    pub transcription_ids: Vec<String>,
    pub format: String,
    pub output_path: String,
    pub options: Option<SubtitleOptionsRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewSubtitleRequest {
    pub transcription_id: String,
    pub format: String,
    pub max_lines: Option<usize>,
    pub options: Option<SubtitleOptionsRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtitleOptionsRequest {
    pub max_duration: Option<f64>,
    pub min_duration: Option<f64>,
    pub max_chars_per_line: Option<usize>,
    pub max_lines_per_subtitle: Option<usize>,
    pub gap_duration: Option<f64>,
    pub split_by_punctuation: Option<bool>,
    pub max_words_per_subtitle: Option<usize>,
}

impl From<SubtitleOptionsRequest> for SubtitleOptions {
    fn from(req: SubtitleOptionsRequest) -> Self {
        let mut options = SubtitleOptions::default();
        
        if let Some(max_duration) = req.max_duration {
            options.max_duration = max_duration;
        }
        if let Some(min_duration) = req.min_duration {
            options.min_duration = min_duration;
        }
        if let Some(max_chars_per_line) = req.max_chars_per_line {
            options.max_chars_per_line = max_chars_per_line;
        }
        if let Some(max_lines_per_subtitle) = req.max_lines_per_subtitle {
            options.max_lines_per_subtitle = max_lines_per_subtitle;
        }
        if let Some(gap_duration) = req.gap_duration {
            options.gap_duration = gap_duration;
        }
        if let Some(split_by_punctuation) = req.split_by_punctuation {
            options.split_by_punctuation = split_by_punctuation;
        }
        if let Some(max_words_per_subtitle) = req.max_words_per_subtitle {
            options.max_words_per_subtitle = max_words_per_subtitle;
        }
        
        options
    }
}

fn parse_subtitle_format(format_str: &str) -> Result<SubtitleFormat, String> {
    match format_str.to_lowercase().as_str() {
        "srt" => Ok(SubtitleFormat::SRT),
        "vtt" => Ok(SubtitleFormat::VTT),
        "ass" => Ok(SubtitleFormat::ASS),
        "txt" => Ok(SubtitleFormat::TXT),
        _ => Err(format!("不支持的字幕格式: {}", format_str)),
    }
}

/// 生成单个字幕文件
#[tauri::command]
pub async fn generate_subtitle_file(
    state: State<'_, AppState>,
    request: GenerateSubtitleRequest,
) -> Result<usize, String> {
    let format = parse_subtitle_format(&request.format)?;
    let options = request.options.map(|o| o.into()).unwrap_or_default();
    let generator = SubtitleGenerator::new(options);

    // 从数据库获取转录记录
    let entry = state.database.get_transcription_by_id(&request.transcription_id)
        .map_err(|e| format!("获取转录记录失败: {}", e))?
        .ok_or("转录记录不存在")?;

    match generator.generate_subtitle_file(&entry, format, &request.output_path).await {
        Ok(subtitles) => {
            println!("✅ 字幕文件生成成功: {}", request.output_path);
            Ok(subtitles.len())
        }
        Err(e) => {
            eprintln!("❌ 字幕生成失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 批量生成字幕文件
#[tauri::command]
pub async fn batch_generate_subtitles(
    state: State<'_, AppState>,
    request: BatchGenerateSubtitleRequest,
) -> Result<Vec<String>, String> {
    let format = parse_subtitle_format(&request.format)?;
    let options = request.options.map(|o| o.into()).unwrap_or_default();
    let generator = SubtitleGenerator::new(options);

    // 获取所有转录记录
    let mut entries = Vec::new();
    for id in &request.transcription_ids {
        match state.database.get_transcription_by_id(id) {
            Ok(Some(entry)) => entries.push(entry),
            Ok(None) => eprintln!("⚠️  转录记录不存在: {}", id),
            Err(e) => eprintln!("❌ 获取转录记录失败 {}: {}", id, e),
        }
    }

    if entries.is_empty() {
        return Err("没有找到有效的转录记录".to_string());
    }

    match generator.batch_generate(&entries, format, &request.output_directory).await {
        Ok(files) => {
            println!("✅ 批量字幕生成完成，共生成 {} 个文件", files.len());
            Ok(files)
        }
        Err(e) => {
            eprintln!("❌ 批量字幕生成失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 合并多个转录为单个字幕文件
#[tauri::command]
pub async fn merge_subtitles(
    state: State<'_, AppState>,
    request: MergeSubtitleRequest,
) -> Result<usize, String> {
    let format = parse_subtitle_format(&request.format)?;
    let options = request.options.map(|o| o.into()).unwrap_or_default();
    let generator = SubtitleGenerator::new(options);

    // 获取所有转录记录
    let mut entries = Vec::new();
    for id in &request.transcription_ids {
        match state.database.get_transcription_by_id(id) {
            Ok(Some(entry)) => entries.push(entry),
            Ok(None) => return Err(format!("转录记录不存在: {}", id)),
            Err(e) => return Err(format!("获取转录记录失败: {}", e)),
        }
    }

    match generator.generate_merged_subtitle_file(&entries, format, &request.output_path).await {
        Ok(subtitles) => {
            println!("✅ 合并字幕文件生成成功: {}", request.output_path);
            Ok(subtitles.len())
        }
        Err(e) => {
            eprintln!("❌ 合并字幕生成失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 预览字幕内容
#[tauri::command]
pub async fn preview_subtitle(
    state: State<'_, AppState>,
    request: PreviewSubtitleRequest,
) -> Result<String, String> {
    let format = parse_subtitle_format(&request.format)?;
    let options = request.options.map(|o| o.into()).unwrap_or_default();
    let generator = SubtitleGenerator::new(options);

    // 从数据库获取转录记录
    let entry = state.database.get_transcription_by_id(&request.transcription_id)
        .map_err(|e| format!("获取转录记录失败: {}", e))?
        .ok_or("转录记录不存在")?;

    match generator.preview_subtitles(&entry, format, request.max_lines) {
        Ok(preview) => Ok(preview),
        Err(e) => {
            eprintln!("❌ 字幕预览失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取字幕生成统计信息
#[tauri::command]
pub async fn get_subtitle_statistics(
    state: State<'_, AppState>,
    transcription_id: String,
    options: Option<SubtitleOptionsRequest>,
) -> Result<SubtitleStatisticsResponse, String> {
    let subtitle_options = options.map(|o| o.into()).unwrap_or_default();
    let generator = SubtitleGenerator::new(subtitle_options);

    // 从数据库获取转录记录
    let entry = state.database.get_transcription_by_id(&transcription_id)
        .map_err(|e| format!("获取转录记录失败: {}", e))?
        .ok_or("转录记录不存在")?;

    match generator.estimate_statistics(&entry) {
        Ok(stats) => Ok(SubtitleStatisticsResponse::from(stats)),
        Err(e) => {
            eprintln!("❌ 统计信息计算失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取支持的字幕格式列表
#[tauri::command]
pub async fn get_supported_subtitle_formats() -> Result<Vec<String>, String> {
    Ok(vec![
        "srt".to_string(),
        "vtt".to_string(),
        "ass".to_string(),
        "txt".to_string(),
    ])
}

/// 获取默认字幕选项
#[tauri::command]
pub async fn get_default_subtitle_options() -> Result<SubtitleOptionsResponse, String> {
    let options = SubtitleOptions::default();
    Ok(SubtitleOptionsResponse::from(options))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtitleStatisticsResponse {
    pub total_subtitles: usize,
    pub total_duration: f64,
    pub avg_duration_per_subtitle: f64,
    pub avg_chars_per_subtitle: usize,
    pub longest_subtitle_chars: usize,
    pub estimated_reading_time: f64,
}

impl From<SubtitleStatistics> for SubtitleStatisticsResponse {
    fn from(stats: SubtitleStatistics) -> Self {
        Self {
            total_subtitles: stats.total_subtitles,
            total_duration: stats.total_duration,
            avg_duration_per_subtitle: stats.avg_duration_per_subtitle,
            avg_chars_per_subtitle: stats.avg_chars_per_subtitle,
            longest_subtitle_chars: stats.longest_subtitle_chars,
            estimated_reading_time: stats.estimated_reading_time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtitleOptionsResponse {
    pub max_duration: f64,
    pub min_duration: f64,
    pub max_chars_per_line: usize,
    pub max_lines_per_subtitle: usize,
    pub gap_duration: f64,
    pub split_by_punctuation: bool,
    pub max_words_per_subtitle: usize,
}

impl From<SubtitleOptions> for SubtitleOptionsResponse {
    fn from(options: SubtitleOptions) -> Self {
        Self {
            max_duration: options.max_duration,
            min_duration: options.min_duration,
            max_chars_per_line: options.max_chars_per_line,
            max_lines_per_subtitle: options.max_lines_per_subtitle,
            gap_duration: options.gap_duration,
            split_by_punctuation: options.split_by_punctuation,
            max_words_per_subtitle: options.max_words_per_subtitle,
        }
    }
}