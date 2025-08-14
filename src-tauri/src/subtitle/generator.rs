// 字幕生成器主模块
use std::path::Path;
use crate::errors::{AppError, AppResult};
use crate::types::TranscriptionEntry;
use super::formats::{SubtitleFormat, SubtitleFormatter, SubtitleEntry};
use super::time_sync::{SubtitleOptions, SubtitleSynchronizer};

/// 字幕生成器
pub struct SubtitleGenerator {
    options: SubtitleOptions,
}

impl SubtitleGenerator {
    /// 创建新的字幕生成器
    pub fn new(options: SubtitleOptions) -> Self {
        Self { options }
    }

    /// 使用默认配置创建字幕生成器
    pub fn default() -> Self {
        Self::new(SubtitleOptions::default())
    }

    /// 从单个转录条目生成字幕文件
    pub async fn generate_subtitle_file(
        &self,
        entry: &TranscriptionEntry,
        format: SubtitleFormat,
        output_path: &str,
    ) -> AppResult<Vec<SubtitleEntry>> {
        println!("🎬 开始生成字幕文件...");
        println!("   📝 转录ID: {}", entry.id);
        println!("   🎞️  格式: {}", format);
        println!("   📁 输出路径: {}", output_path);

        // 生成字幕条目
        let mut subtitles = SubtitleSynchronizer::generate_from_transcription(entry, &self.options)?;
        
        // 优化字幕时间
        self.optimize_subtitles(&mut subtitles)?;

        // 保存文件
        SubtitleFormatter::save_subtitle_file(&subtitles, &format, output_path).await?;

        println!("✅ 字幕生成完成! 共 {} 个字幕条目", subtitles.len());
        Ok(subtitles)
    }

    /// 从多个转录条目生成合并的字幕文件
    pub async fn generate_merged_subtitle_file(
        &self,
        entries: &[TranscriptionEntry],
        format: SubtitleFormat,
        output_path: &str,
    ) -> AppResult<Vec<SubtitleEntry>> {
        println!("🎬 开始生成合并字幕文件...");
        println!("   📝 转录条目数: {}", entries.len());
        println!("   🎞️  格式: {}", format);
        println!("   📁 输出路径: {}", output_path);

        // 生成字幕条目
        let mut subtitles = SubtitleSynchronizer::generate_from_multiple_transcriptions(entries, &self.options)?;
        
        // 优化字幕时间
        self.optimize_subtitles(&mut subtitles)?;

        // 保存文件
        SubtitleFormatter::save_subtitle_file(&subtitles, &format, output_path).await?;

        println!("✅ 合并字幕生成完成! 共 {} 个字幕条目", subtitles.len());
        Ok(subtitles)
    }

    /// 预览字幕内容（不保存文件）
    pub fn preview_subtitles(
        &self,
        entry: &TranscriptionEntry,
        format: SubtitleFormat,
        max_lines: Option<usize>,
    ) -> AppResult<String> {
        let subtitles = SubtitleSynchronizer::generate_from_transcription(entry, &self.options)?;
        let content = SubtitleFormatter::format_subtitles(&subtitles, &format)?;
        
        if let Some(limit) = max_lines {
            let lines: Vec<&str> = content.lines().collect();
            let preview_lines = lines.into_iter().take(limit).collect::<Vec<_>>();
            Ok(preview_lines.join("\n"))
        } else {
            Ok(content)
        }
    }

    /// 批量生成字幕文件
    pub async fn batch_generate(
        &self,
        entries: &[TranscriptionEntry],
        format: SubtitleFormat,
        output_dir: &str,
    ) -> AppResult<Vec<String>> {
        println!("🎬 开始批量生成字幕文件...");
        println!("   📝 转录条目数: {}", entries.len());
        println!("   🎞️  格式: {}", format);
        println!("   📁 输出目录: {}", output_dir);

        // 确保输出目录存在
        tokio::fs::create_dir_all(output_dir).await
            .map_err(|e| AppError::FileSystemError(format!("创建目录失败: {}", e)))?;

        let mut generated_files = Vec::new();

        for (index, entry) in entries.iter().enumerate() {
            let filename = format!("subtitle_{:03}.{}", index + 1, format);
            let output_path = Path::new(output_dir).join(&filename);
            let output_path_str = output_path.to_string_lossy();

            match self.generate_subtitle_file(entry, format.clone(), &output_path_str).await {
                Ok(_) => {
                    generated_files.push(output_path_str.to_string());
                    println!("✅ 已生成: {}", filename);
                }
                Err(e) => {
                    eprintln!("❌ 生成失败 {}: {}", filename, e);
                }
            }
        }

        println!("🎉 批量生成完成! 成功生成 {} 个文件", generated_files.len());
        Ok(generated_files)
    }

    /// 优化字幕条目
    fn optimize_subtitles(&self, subtitles: &mut Vec<SubtitleEntry>) -> AppResult<()> {
        // 合并短字幕
        SubtitleSynchronizer::merge_short_subtitles(subtitles, &self.options)?;
        
        // 验证时间有效性
        SubtitleSynchronizer::validate_timing(subtitles)?;
        
        // 清理文本
        for subtitle in subtitles.iter_mut() {
            subtitle.text = self.clean_text(&subtitle.text);
        }

        Ok(())
    }

    /// 清理文本内容
    fn clean_text(&self, text: &str) -> String {
        text.trim()
            .replace("  ", " ") // 移除多余空格
            .replace("\n\n", "\n") // 移除多余换行
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 设置字幕选项
    pub fn set_options(&mut self, options: SubtitleOptions) {
        self.options = options;
    }

    /// 获取当前字幕选项
    pub fn get_options(&self) -> &SubtitleOptions {
        &self.options
    }

    /// 估算字幕生成统计信息
    pub fn estimate_statistics(&self, entry: &TranscriptionEntry) -> AppResult<SubtitleStatistics> {
        let subtitles = SubtitleSynchronizer::generate_from_transcription(entry, &self.options)?;
        
        let total_duration = subtitles.iter()
            .map(|s| s.end_time - s.start_time)
            .sum::<f64>();
        
        let avg_chars_per_subtitle = subtitles.iter()
            .map(|s| s.text.len())
            .sum::<usize>() as f64 / subtitles.len() as f64;

        let longest_subtitle = subtitles.iter()
            .map(|s| s.text.len())
            .max()
            .unwrap_or(0);

        Ok(SubtitleStatistics {
            total_subtitles: subtitles.len(),
            total_duration,
            avg_duration_per_subtitle: total_duration / subtitles.len() as f64,
            avg_chars_per_subtitle: avg_chars_per_subtitle as usize,
            longest_subtitle_chars: longest_subtitle,
            estimated_reading_time: total_duration * 0.8, // 估算阅读时间
        })
    }
}

/// 字幕统计信息
#[derive(Debug, Clone)]
pub struct SubtitleStatistics {
    pub total_subtitles: usize,
    pub total_duration: f64,
    pub avg_duration_per_subtitle: f64,
    pub avg_chars_per_subtitle: usize,
    pub longest_subtitle_chars: usize,
    pub estimated_reading_time: f64,
}

impl SubtitleStatistics {
    /// 打印统计信息
    pub fn print_summary(&self) {
        println!("📊 字幕生成统计:");
        println!("   🎬 字幕条目总数: {}", self.total_subtitles);
        println!("   ⏱️  总时长: {:.2} 秒", self.total_duration);
        println!("   📏 平均时长: {:.2} 秒/条", self.avg_duration_per_subtitle);
        println!("   📝 平均字符数: {} 字符/条", self.avg_chars_per_subtitle);
        println!("   📜 最长字幕: {} 字符", self.longest_subtitle_chars);
        println!("   👀 估算阅读时间: {:.2} 秒", self.estimated_reading_time);
    }
}