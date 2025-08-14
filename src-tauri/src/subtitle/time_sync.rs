// 字幕时间同步和分割逻辑
use crate::errors::{AppError, AppResult};
use crate::types::TranscriptionEntry;
use super::formats::SubtitleEntry;

/// 字幕生成选项
#[derive(Debug, Clone)]
pub struct SubtitleOptions {
    /// 每个字幕条目的最大持续时间（秒）
    pub max_duration: f64,
    /// 每个字幕条目的最小持续时间（秒）  
    pub min_duration: f64,
    /// 每行最大字符数
    pub max_chars_per_line: usize,
    /// 每个字幕最大行数
    pub max_lines_per_subtitle: usize,
    /// 字幕间隔时间（秒）
    pub gap_duration: f64,
    /// 是否按标点符号分割
    pub split_by_punctuation: bool,
    /// 最大单词数（英文）
    pub max_words_per_subtitle: usize,
}

impl Default for SubtitleOptions {
    fn default() -> Self {
        Self {
            max_duration: 6.0,
            min_duration: 1.0,
            max_chars_per_line: 42,
            max_lines_per_subtitle: 2,
            gap_duration: 0.2,
            split_by_punctuation: true,
            max_words_per_subtitle: 15,
        }
    }
}

/// 字幕时间同步器
pub struct SubtitleSynchronizer;

impl SubtitleSynchronizer {
    /// 从转录结果生成字幕条目
    pub fn generate_from_transcription(
        entry: &TranscriptionEntry,
        options: &SubtitleOptions,
    ) -> AppResult<Vec<SubtitleEntry>> {
        // 如果没有详细的时间戳信息，使用基于文本长度的估算
        let text_segments = Self::split_text(&entry.text, options)?;
        let total_duration = entry.duration.max(1.0);
        
        let mut subtitles = Vec::new();
        let segment_count = text_segments.len() as f64;
        
        for (index, segment) in text_segments.iter().enumerate() {
            let progress = index as f64 / segment_count;
            let next_progress = (index + 1) as f64 / segment_count;
            
            // 根据文本长度分配时间
            let start_time = progress * total_duration;
            let end_time = (next_progress * total_duration)
                .min(start_time + options.max_duration)
                .max(start_time + options.min_duration);
            
            subtitles.push(SubtitleEntry {
                index: (index + 1) as u32,
                start_time,
                end_time,
                text: segment.clone(),
                speaker: None,
                confidence: Some(entry.confidence as f32),
            });
        }
        
        // 调整字幕间隔
        Self::adjust_timing(&mut subtitles, options);
        
        Ok(subtitles)
    }

    /// 从多个转录条目生成字幕
    pub fn generate_from_multiple_transcriptions(
        entries: &[TranscriptionEntry],
        options: &SubtitleOptions,
    ) -> AppResult<Vec<SubtitleEntry>> {
        let mut all_subtitles = Vec::new();
        let mut current_time = 0.0;

        for entry in entries {
            let mut subtitles = Self::generate_from_transcription(entry, options)?;
            
            // 调整时间偏移
            for subtitle in &mut subtitles {
                subtitle.start_time += current_time;
                subtitle.end_time += current_time;
            }
            
            current_time += entry.duration + options.gap_duration;
            all_subtitles.extend(subtitles);
        }

        // 重新编号
        for (index, subtitle) in all_subtitles.iter_mut().enumerate() {
            subtitle.index = (index + 1) as u32;
        }

        Ok(all_subtitles)
    }

    /// 智能文本分割
    fn split_text(text: &str, options: &SubtitleOptions) -> AppResult<Vec<String>> {
        if text.is_empty() {
            return Ok(vec![]);
        }

        let mut segments = Vec::new();
        
        if options.split_by_punctuation {
            // 按标点符号分割
            let sentences = Self::split_by_sentences(text);
            for sentence in sentences {
                if sentence.len() > options.max_chars_per_line {
                    // 如果句子太长，进一步分割
                    let sub_segments = Self::split_long_text(&sentence, options);
                    segments.extend(sub_segments);
                } else {
                    segments.push(sentence);
                }
            }
        } else {
            // 按长度分割
            segments = Self::split_by_length(text, options);
        }

        // 过滤空字符串
        Ok(segments.into_iter().filter(|s| !s.trim().is_empty()).collect())
    }

    /// 按句子分割
    fn split_by_sentences(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        
        let sentence_endings = &['.', '!', '?', '。', '！', '？'];
        
        for ch in text.chars() {
            current_sentence.push(ch);
            
            if sentence_endings.contains(&ch) {
                // 检查后面是否有空格或换行
                sentences.push(current_sentence.trim().to_string());
                current_sentence.clear();
            }
        }
        
        // 添加剩余文本
        if !current_sentence.trim().is_empty() {
            sentences.push(current_sentence.trim().to_string());
        }
        
        sentences
    }

    /// 按长度分割长文本
    fn split_long_text(text: &str, options: &SubtitleOptions) -> Vec<String> {
        let mut segments = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        let mut current_segment = String::new();
        let mut word_count = 0;
        
        for word in words {
            let test_segment = if current_segment.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_segment, word)
            };
            
            if test_segment.len() > options.max_chars_per_line || 
               word_count >= options.max_words_per_subtitle {
                // 当前段落已满，开始新段落
                if !current_segment.is_empty() {
                    segments.push(current_segment);
                }
                current_segment = word.to_string();
                word_count = 1;
            } else {
                current_segment = test_segment;
                word_count += 1;
            }
        }
        
        // 添加最后一个段落
        if !current_segment.is_empty() {
            segments.push(current_segment);
        }
        
        segments
    }

    /// 按固定长度分割
    fn split_by_length(text: &str, options: &SubtitleOptions) -> Vec<String> {
        let mut segments = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        
        let mut start = 0;
        while start < chars.len() {
            let mut end = (start + options.max_chars_per_line).min(chars.len());
            
            // 尝试在单词边界分割
            if end < chars.len() && !chars[end].is_whitespace() {
                // 向后查找空格
                let mut space_pos = end;
                while space_pos > start && !chars[space_pos].is_whitespace() {
                    space_pos -= 1;
                }
                
                if space_pos > start {
                    end = space_pos;
                }
            }
            
            let segment: String = chars[start..end].iter().collect();
            segments.push(segment.trim().to_string());
            
            start = end;
            // 跳过空格
            while start < chars.len() && chars[start].is_whitespace() {
                start += 1;
            }
        }
        
        segments
    }

    /// 调整字幕时间间隔
    fn adjust_timing(subtitles: &mut [SubtitleEntry], options: &SubtitleOptions) {
        for i in 0..subtitles.len().saturating_sub(1) {
            let current_end = subtitles[i].end_time;
            let next_start = subtitles[i + 1].start_time;
            
            // 如果字幕之间间隔太小，调整时间
            if next_start - current_end < options.gap_duration {
                let midpoint = (current_end + next_start) / 2.0;
                subtitles[i].end_time = midpoint - options.gap_duration / 2.0;
                subtitles[i + 1].start_time = midpoint + options.gap_duration / 2.0;
            }
        }
    }

    /// 合并相邻的短字幕
    pub fn merge_short_subtitles(
        subtitles: &mut Vec<SubtitleEntry>,
        options: &SubtitleOptions,
    ) -> AppResult<()> {
        let mut i = 0;
        while i < subtitles.len().saturating_sub(1) {
            let current_duration = subtitles[i].end_time - subtitles[i].start_time;
            let next_duration = subtitles[i + 1].end_time - subtitles[i + 1].start_time;
            
            // 如果两个连续的字幕都很短，考虑合并
            if current_duration < options.min_duration && 
               next_duration < options.min_duration {
                let combined_text = format!("{} {}", subtitles[i].text, subtitles[i + 1].text);
                
                // 检查合并后的文本长度
                if combined_text.len() <= options.max_chars_per_line {
                    subtitles[i].text = combined_text;
                    subtitles[i].end_time = subtitles[i + 1].end_time;
                    subtitles.remove(i + 1);
                    
                    // 重新编号
                    for (index, subtitle) in subtitles.iter_mut().enumerate() {
                        subtitle.index = (index + 1) as u32;
                    }
                    
                    continue; // 不增加i，检查新合并的字幕
                }
            }
            i += 1;
        }
        
        Ok(())
    }

    /// 验证字幕时间的有效性
    pub fn validate_timing(subtitles: &[SubtitleEntry]) -> AppResult<()> {
        for (i, subtitle) in subtitles.iter().enumerate() {
            // 检查时间有效性
            if subtitle.start_time >= subtitle.end_time {
                return Err(AppError::ValidationError(
                    format!("字幕 #{} 时间无效: start={:.2}, end={:.2}", 
                            subtitle.index, subtitle.start_time, subtitle.end_time)
                ));
            }
            
            // 检查重叠
            if i > 0 && subtitle.start_time < subtitles[i - 1].end_time {
                return Err(AppError::ValidationError(
                    format!("字幕 #{} 与前一个字幕时间重叠", subtitle.index)
                ));
            }
        }
        
        Ok(())
    }
}