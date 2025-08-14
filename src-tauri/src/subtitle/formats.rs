// 字幕格式处理
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::errors::{AppError, AppResult};

/// 字幕格式类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubtitleFormat {
    SRT,
    VTT,
    ASS,
    TXT,
}

impl fmt::Display for SubtitleFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubtitleFormat::SRT => write!(f, "srt"),
            SubtitleFormat::VTT => write!(f, "vtt"),
            SubtitleFormat::ASS => write!(f, "ass"),
            SubtitleFormat::TXT => write!(f, "txt"),
        }
    }
}

/// 字幕条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleEntry {
    pub index: u32,
    pub start_time: f64, // 秒
    pub end_time: f64,   // 秒
    pub text: String,
    pub speaker: Option<String>,
    pub confidence: Option<f32>,
}

impl SubtitleEntry {
    /// 格式化时间为SRT格式 (HH:MM:SS,mmm)
    pub fn format_time_srt(seconds: f64) -> String {
        let hours = (seconds / 3600.0) as u32;
        let minutes = ((seconds % 3600.0) / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        let millis = ((seconds % 1.0) * 1000.0) as u32;
        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
    }

    /// 格式化时间为VTT格式 (HH:MM:SS.mmm)
    pub fn format_time_vtt(seconds: f64) -> String {
        let hours = (seconds / 3600.0) as u32;
        let minutes = ((seconds % 3600.0) / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        let millis = ((seconds % 1.0) * 1000.0) as u32;
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, millis)
    }

    /// 转换为SRT格式
    pub fn to_srt(&self) -> String {
        format!(
            "{}\n{} --> {}\n{}\n\n",
            self.index,
            Self::format_time_srt(self.start_time),
            Self::format_time_srt(self.end_time),
            self.text
        )
    }

    /// 转换为VTT格式
    pub fn to_vtt(&self) -> String {
        let speaker_prefix = if let Some(speaker) = &self.speaker {
            format!("<v {}>", speaker)
        } else {
            String::new()
        };

        format!(
            "{} --> {}\n{}{}\n\n",
            Self::format_time_vtt(self.start_time),
            Self::format_time_vtt(self.end_time),
            speaker_prefix,
            self.text
        )
    }

    /// 转换为ASS格式
    pub fn to_ass(&self) -> String {
        let start = Self::format_time_ass(self.start_time);
        let end = Self::format_time_ass(self.end_time);
        let speaker = self.speaker.as_deref().unwrap_or("Default");
        
        format!(
            "Dialogue: 0,{},{},{},,,0,0,0,,{}",
            start, end, speaker, self.text
        )
    }

    /// 格式化时间为ASS格式 (H:MM:SS.cc)
    fn format_time_ass(seconds: f64) -> String {
        let hours = (seconds / 3600.0) as u32;
        let minutes = ((seconds % 3600.0) / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        let centisecs = ((seconds % 1.0) * 100.0) as u32;
        format!("{}:{:02}:{:02}.{:02}", hours, minutes, secs, centisecs)
    }
}

/// 字幕文件生成器
pub struct SubtitleFormatter;

impl SubtitleFormatter {
    /// 生成完整的SRT字幕文件内容
    pub fn to_srt(entries: &[SubtitleEntry]) -> String {
        let mut content = String::new();
        for entry in entries {
            content.push_str(&entry.to_srt());
        }
        content
    }

    /// 生成完整的VTT字幕文件内容
    pub fn to_vtt(entries: &[SubtitleEntry]) -> String {
        let mut content = String::from("WEBVTT\n\n");
        for entry in entries {
            content.push_str(&entry.to_vtt());
        }
        content
    }

    /// 生成完整的ASS字幕文件内容
    pub fn to_ass(entries: &[SubtitleEntry]) -> String {
        let header = r#"[Script Info]
Title: Generated Subtitles
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,16,&H00ffffff,&H000000ff,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

        let mut content = String::from(header);
        for entry in entries {
            content.push_str(&entry.to_ass());
            content.push('\n');
        }
        content
    }

    /// 生成简单的TXT格式
    pub fn to_txt(entries: &[SubtitleEntry]) -> String {
        let mut content = String::new();
        for entry in entries {
            content.push_str(&format!(
                "[{} - {}] {}\n",
                Self::format_time_simple(entry.start_time),
                Self::format_time_simple(entry.end_time),
                entry.text
            ));
        }
        content
    }

    /// 简单时间格式 (MM:SS)
    fn format_time_simple(seconds: f64) -> String {
        let minutes = (seconds / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        format!("{:02}:{:02}", minutes, secs)
    }

    /// 根据格式生成字幕内容
    pub fn format_subtitles(entries: &[SubtitleEntry], format: &SubtitleFormat) -> AppResult<String> {
        match format {
            SubtitleFormat::SRT => Ok(Self::to_srt(entries)),
            SubtitleFormat::VTT => Ok(Self::to_vtt(entries)),
            SubtitleFormat::ASS => Ok(Self::to_ass(entries)),
            SubtitleFormat::TXT => Ok(Self::to_txt(entries)),
        }
    }

    /// 保存字幕文件
    pub async fn save_subtitle_file(
        entries: &[SubtitleEntry],
        format: &SubtitleFormat,
        file_path: &str,
    ) -> AppResult<()> {
        let content = Self::format_subtitles(entries, format)?;
        
        tokio::fs::write(file_path, content)
            .await
            .map_err(|e| AppError::FileSystemError(format!("保存字幕文件失败: {}", e)))?;
        
        println!("✅ 字幕文件已保存: {}", file_path);
        Ok(())
    }
}