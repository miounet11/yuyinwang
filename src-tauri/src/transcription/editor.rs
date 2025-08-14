// 转录文本编辑器
// 提供段落分割、合并、智能编辑等功能

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use regex::Regex;
use crate::errors::{AppError, AppResult};
use crate::types::TranscriptionEntry;

/// 文本段落结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextParagraph {
    pub id: String,
    pub content: String,
    pub start_timestamp: Option<f64>, // 相对于音频开始的时间戳（秒）
    pub end_timestamp: Option<f64>,
    pub confidence: f64,
    pub speaker_id: Option<String>, // 说话人标识
    pub is_edited: bool,             // 是否被手动编辑过
    pub edit_history: Vec<EditOperation>,
}

/// 编辑操作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    pub operation_type: EditOperationType,
    pub timestamp: u64,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub paragraph_ids: Vec<String>, // 涉及的段落ID
    pub user_note: Option<String>,
}

/// 编辑操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditOperationType {
    Insert,      // 插入文本
    Delete,      // 删除文本
    Replace,     // 替换文本
    Split,       // 分割段落
    Merge,       // 合并段落
    Reorder,     // 重新排序
    AddSpeaker,  // 添加说话人
    RemoveSpeaker, // 移除说话人
    TimestampAdjust, // 调整时间戳
}

/// 文档结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionDocument {
    pub entry_id: String,
    pub title: String,
    pub paragraphs: Vec<TextParagraph>,
    pub metadata: DocumentMetadata,
    pub version: u32,
    pub last_modified: u64,
    pub is_dirty: bool, // 是否有未保存的更改
}

/// 文档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub original_text: String,
    pub total_duration: f64,
    pub model_used: String,
    pub language: String,
    pub word_count: usize,
    pub paragraph_count: usize,
    pub average_confidence: f64,
    pub speakers: HashMap<String, SpeakerInfo>,
}

/// 说话人信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerInfo {
    pub name: String,
    pub color: String, // 用于UI显示的颜色
    pub segments_count: usize,
    pub total_duration: f64,
}

/// 段落分割选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphSplitOptions {
    pub split_by: SplitMethod,
    pub min_paragraph_length: usize,      // 最小段落字符数
    pub max_paragraph_length: usize,      // 最大段落字符数
    pub sentence_break_threshold: f64,    // 句子断点阈值
    pub pause_detection_threshold: f64,   // 停顿检测阈值（秒）
    pub preserve_speaker_boundaries: bool, // 保持说话人边界
    pub smart_punctuation: bool,          // 智能标点符号处理
}

/// 分割方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitMethod {
    Sentences,     // 按句子分割
    TimeInterval,  // 按时间间隔分割
    PauseDetection, // 按停顿检测分割
    FixedLength,   // 按固定长度分割
    Manual,        // 手动指定分割点
    Smart,         // 智能分割（综合多种方法）
}

/// 合并选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphMergeOptions {
    pub merge_criteria: MergeCriteria,
    pub max_merged_length: usize,
    pub preserve_timestamps: bool,
    pub handle_speaker_conflicts: SpeakerConflictStrategy,
}

/// 合并条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeCriteria {
    Sequential,     // 按顺序合并
    SameSpeaker,   // 相同说话人
    SimilarContent, // 相似内容
    TimeProximity, // 时间接近
    Custom(String), // 自定义条件
}

/// 说话人冲突策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeakerConflictStrategy {
    KeepFirst,      // 保留第一个
    KeepMajority,   // 保留占多数的
    Merge,          // 标记为混合
    AskUser,        // 询问用户
}

/// 搜索和替换选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindReplaceOptions {
    pub find_text: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub scope: SearchScope,
}

/// 搜索范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchScope {
    CurrentParagraph,
    SelectedParagraphs(Vec<String>),
    AllParagraphs,
    BySpeaker(String),
    ByTimeRange { start: f64, end: f64 },
}

/// 转录文本编辑器
#[derive(Debug)]
pub struct TranscriptionEditor {
    documents: Arc<Mutex<HashMap<String, TranscriptionDocument>>>,
    auto_save_interval: Duration,
    max_undo_levels: usize,
    undo_stacks: Arc<Mutex<HashMap<String, Vec<TranscriptionDocument>>>>, // 撤销栈
    redo_stacks: Arc<Mutex<HashMap<String, Vec<TranscriptionDocument>>>>, // 重做栈
}

impl Default for ParagraphSplitOptions {
    fn default() -> Self {
        Self {
            split_by: SplitMethod::Smart,
            min_paragraph_length: 50,
            max_paragraph_length: 500,
            sentence_break_threshold: 0.8,
            pause_detection_threshold: 1.0,
            preserve_speaker_boundaries: true,
            smart_punctuation: true,
        }
    }
}

impl Default for ParagraphMergeOptions {
    fn default() -> Self {
        Self {
            merge_criteria: MergeCriteria::Sequential,
            max_merged_length: 1000,
            preserve_timestamps: true,
            handle_speaker_conflicts: SpeakerConflictStrategy::KeepMajority,
        }
    }
}

impl TranscriptionEditor {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
            auto_save_interval: Duration::from_secs(30),
            max_undo_levels: 50,
            undo_stacks: Arc::new(Mutex::new(HashMap::new())),
            redo_stacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 从转录条目创建可编辑文档
    pub async fn create_document_from_entry(
        &self,
        entry: &TranscriptionEntry,
    ) -> AppResult<String> {
        let document_id = Uuid::new_v4().to_string();
        
        // 智能分割原始文本为段落
        let paragraphs = self.smart_split_text(&entry.text, None)?;
        
        let metadata = DocumentMetadata {
            original_text: entry.text.clone(),
            total_duration: entry.duration,
            model_used: entry.model.clone(),
            language: "auto".to_string(), // 可以从entry元数据中获取
            word_count: entry.text.split_whitespace().count(),
            paragraph_count: paragraphs.len(),
            average_confidence: entry.confidence,
            speakers: HashMap::new(),
        };

        let paragraph_count = metadata.paragraph_count;
        
        let document = TranscriptionDocument {
            entry_id: entry.id.clone(),
            title: format!("转录文档 - {}", 
                chrono::DateTime::from_timestamp(entry.timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "未知时间".to_string())
            ),
            paragraphs,
            metadata,
            version: 1,
            last_modified: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            is_dirty: false,
        };

        self.documents.lock().insert(document_id.clone(), document);
        self.save_to_undo_stack(&document_id);

        println!("📄 创建文档: {} ({} 个段落)", document_id, paragraph_count);
        Ok(document_id)
    }

    /// 获取文档
    pub fn get_document(&self, document_id: &str) -> AppResult<TranscriptionDocument> {
        self.documents.lock()
            .get(document_id)
            .cloned()
            .ok_or_else(|| AppError::ValidationError(format!("文档不存在: {}", document_id)))
    }

    /// 智能分割文本为段落
    pub fn smart_split_text(
        &self,
        text: &str,
        options: Option<ParagraphSplitOptions>,
    ) -> AppResult<Vec<TextParagraph>> {
        let options = options.unwrap_or_default();
        
        match options.split_by {
            SplitMethod::Smart => self.smart_split_algorithm(text, &options),
            SplitMethod::Sentences => self.split_by_sentences(text, &options),
            SplitMethod::FixedLength => self.split_by_length(text, &options),
            SplitMethod::TimeInterval => self.split_by_time_interval(text, &options),
            SplitMethod::PauseDetection => self.split_by_pause_detection(text, &options),
            SplitMethod::Manual => Ok(vec![self.create_single_paragraph(text)]),
        }
    }

    /// 智能分割算法（综合多种方法）
    fn smart_split_algorithm(
        &self,
        text: &str,
        options: &ParagraphSplitOptions,
    ) -> AppResult<Vec<TextParagraph>> {
        let mut paragraphs = Vec::new();
        
        // 第一步：按句号分割
        let sentences = self.split_by_sentence_endings(text);
        
        let mut current_paragraph = String::new();
        let mut paragraph_id = 0;
        
        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }
            
            // 检查是否应该开始新段落
            let should_start_new = 
                current_paragraph.len() + sentence.len() > options.max_paragraph_length ||
                (current_paragraph.len() > options.min_paragraph_length && 
                 self.is_paragraph_boundary(sentence));
            
            if should_start_new && !current_paragraph.is_empty() {
                // 完成当前段落
                paragraphs.push(TextParagraph {
                    id: format!("para_{}", paragraph_id),
                    content: current_paragraph.trim().to_string(),
                    start_timestamp: None,
                    end_timestamp: None,
                    confidence: 0.9, // 默认置信度
                    speaker_id: None,
                    is_edited: false,
                    edit_history: Vec::new(),
                });
                
                current_paragraph.clear();
                paragraph_id += 1;
            }
            
            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(sentence);
        }
        
        // 处理最后一个段落
        if !current_paragraph.trim().is_empty() {
            paragraphs.push(TextParagraph {
                id: format!("para_{}", paragraph_id),
                content: current_paragraph.trim().to_string(),
                start_timestamp: None,
                end_timestamp: None,
                confidence: 0.9,
                speaker_id: None,
                is_edited: false,
                edit_history: Vec::new(),
            });
        }
        
        println!("🧠 智能分割完成: {} 个段落", paragraphs.len());
        Ok(paragraphs)
    }

    /// 按句子分割
    fn split_by_sentences(
        &self,
        text: &str,
        _options: &ParagraphSplitOptions,
    ) -> AppResult<Vec<TextParagraph>> {
        let sentences = self.split_by_sentence_endings(text);
        let paragraphs = sentences
            .into_iter()
            .enumerate()
            .filter_map(|(i, sentence)| {
                let trimmed = sentence.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(TextParagraph {
                        id: format!("sent_{}", i),
                        content: trimmed.to_string(),
                        start_timestamp: None,
                        end_timestamp: None,
                        confidence: 0.9,
                        speaker_id: None,
                        is_edited: false,
                        edit_history: Vec::new(),
                    })
                }
            })
            .collect();
            
        Ok(paragraphs)
    }

    /// 按固定长度分割
    fn split_by_length(
        &self,
        text: &str,
        options: &ParagraphSplitOptions,
    ) -> AppResult<Vec<TextParagraph>> {
        let mut paragraphs = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_paragraph = String::new();
        let mut paragraph_id = 0;
        
        for word in words {
            if current_paragraph.len() + word.len() + 1 > options.max_paragraph_length {
                if !current_paragraph.is_empty() {
                    paragraphs.push(TextParagraph {
                        id: format!("len_{}", paragraph_id),
                        content: current_paragraph.trim().to_string(),
                        start_timestamp: None,
                        end_timestamp: None,
                        confidence: 0.9,
                        speaker_id: None,
                        is_edited: false,
                        edit_history: Vec::new(),
                    });
                    
                    current_paragraph.clear();
                    paragraph_id += 1;
                }
            }
            
            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(word);
        }
        
        if !current_paragraph.trim().is_empty() {
            paragraphs.push(TextParagraph {
                id: format!("len_{}", paragraph_id),
                content: current_paragraph.trim().to_string(),
                start_timestamp: None,
                end_timestamp: None,
                confidence: 0.9,
                speaker_id: None,
                is_edited: false,
                edit_history: Vec::new(),
            });
        }
        
        Ok(paragraphs)
    }

    /// 按时间间隔分割（需要时间戳信息）
    fn split_by_time_interval(
        &self,
        text: &str,
        _options: &ParagraphSplitOptions,
    ) -> AppResult<Vec<TextParagraph>> {
        // 这个方法需要更多的时间戳信息，现在简单分割
        Ok(vec![self.create_single_paragraph(text)])
    }

    /// 按停顿检测分割（需要音频分析）
    fn split_by_pause_detection(
        &self,
        text: &str,
        _options: &ParagraphSplitOptions,
    ) -> AppResult<Vec<TextParagraph>> {
        // 这个方法需要音频分析，现在简单分割
        Ok(vec![self.create_single_paragraph(text)])
    }

    /// 分割段落
    pub async fn split_paragraph(
        &self,
        document_id: &str,
        paragraph_id: &str,
        split_position: usize,
    ) -> AppResult<Vec<String>> {
        self.save_to_undo_stack(document_id);
        
        let mut documents = self.documents.lock();
        let document = documents.get_mut(document_id)
            .ok_or_else(|| AppError::ValidationError("文档不存在".to_string()))?;
        
        let paragraph_index = document.paragraphs
            .iter()
            .position(|p| p.id == paragraph_id)
            .ok_or_else(|| AppError::ValidationError("段落不存在".to_string()))?;
        
        // 先获取原始段落信息，避免借用冲突
        let original_content = document.paragraphs[paragraph_index].content.clone();
        let original_end_timestamp = document.paragraphs[paragraph_index].end_timestamp;
        let original_confidence = document.paragraphs[paragraph_index].confidence;
        let original_speaker_id = document.paragraphs[paragraph_index].speaker_id.clone();
        
        if split_position >= original_content.len() {
            return Err(AppError::ValidationError("分割位置超出段落范围".to_string()));
        }
        
        let first_part = original_content[..split_position].trim().to_string();
        let second_part = original_content[split_position..].trim().to_string();
        
        if first_part.is_empty() || second_part.is_empty() {
            return Err(AppError::ValidationError("分割后的段落不能为空".to_string()));
        }
        
        // 创建新的段落ID
        let new_paragraph_id = format!("{}_split", paragraph_id);
        
        // 记录编辑操作
        let edit_op = EditOperation {
            operation_type: EditOperationType::Split,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            old_content: Some(original_content),
            new_content: None,
            paragraph_ids: vec![paragraph_id.to_string(), new_paragraph_id.clone()],
            user_note: Some(format!("在位置 {} 分割段落", split_position)),
        };
        
        // 更新第一个段落
        document.paragraphs[paragraph_index].content = first_part;
        document.paragraphs[paragraph_index].is_edited = true;
        document.paragraphs[paragraph_index].edit_history.push(edit_op.clone());
        
        // 创建第二个段落
        let second_paragraph = TextParagraph {
            id: new_paragraph_id.clone(),
            content: second_part,
            start_timestamp: original_end_timestamp,
            end_timestamp: original_end_timestamp,
            confidence: original_confidence,
            speaker_id: original_speaker_id,
            is_edited: true,
            edit_history: vec![edit_op],
        };
        
        // 插入第二个段落
        document.paragraphs.insert(paragraph_index + 1, second_paragraph);
        
        // 更新文档状态
        document.is_dirty = true;
        document.version += 1;
        document.last_modified = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        document.metadata.paragraph_count = document.paragraphs.len();
        
        let result_ids = vec![paragraph_id.to_string(), new_paragraph_id];
        
        println!("✂️ 分割段落: {} -> {:?}", paragraph_id, result_ids);
        Ok(result_ids)
    }

    /// 合并段落
    pub async fn merge_paragraphs(
        &self,
        document_id: &str,
        paragraph_ids: &[String],
        options: Option<ParagraphMergeOptions>,
    ) -> AppResult<String> {
        if paragraph_ids.len() < 2 {
            return Err(AppError::ValidationError("至少需要两个段落进行合并".to_string()));
        }
        
        let options = options.unwrap_or_default();
        self.save_to_undo_stack(document_id);
        
        let mut documents = self.documents.lock();
        let document = documents.get_mut(document_id)
            .ok_or_else(|| AppError::ValidationError("文档不存在".to_string()))?;
        
        // 查找要合并的段落
        let mut paragraphs_to_merge: Vec<TextParagraph> = Vec::new();
        let mut indices_to_remove: Vec<usize> = Vec::new();
        
        for paragraph_id in paragraph_ids {
            if let Some(index) = document.paragraphs.iter().position(|p| &p.id == paragraph_id) {
                paragraphs_to_merge.push(document.paragraphs[index].clone());
                indices_to_remove.push(index);
            }
        }
        
        if paragraphs_to_merge.is_empty() {
            return Err(AppError::ValidationError("未找到要合并的段落".to_string()));
        }
        
        // 按索引排序以保持顺序
        indices_to_remove.sort();
        paragraphs_to_merge.sort_by_key(|p| {
            document.paragraphs.iter().position(|dp| dp.id == p.id).unwrap_or(0)
        });
        
        // 合并内容
        let merged_content = paragraphs_to_merge
            .iter()
            .map(|p| p.content.trim())
            .collect::<Vec<&str>>()
            .join(" ");
        
        if merged_content.len() > options.max_merged_length {
            return Err(AppError::ValidationError(
                format!("合并后的段落长度 ({}) 超过限制 ({})", 
                        merged_content.len(), options.max_merged_length)
            ));
        }
        
        // 创建合并后的段落
        let merged_paragraph_id = format!("merged_{}", Uuid::new_v4());
        let first_paragraph = &paragraphs_to_merge[0];
        let last_paragraph = paragraphs_to_merge.last().unwrap();
        
        let edit_op = EditOperation {
            operation_type: EditOperationType::Merge,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            old_content: Some(paragraphs_to_merge.iter()
                .map(|p| p.content.clone())
                .collect::<Vec<String>>()
                .join(" | ")),
            new_content: Some(merged_content.clone()),
            paragraph_ids: paragraph_ids.to_vec(),
            user_note: Some(format!("合并 {} 个段落", paragraph_ids.len())),
        };
        
        let merged_paragraph = TextParagraph {
            id: merged_paragraph_id.clone(),
            content: merged_content,
            start_timestamp: first_paragraph.start_timestamp,
            end_timestamp: if options.preserve_timestamps { 
                last_paragraph.end_timestamp 
            } else { 
                None 
            },
            confidence: paragraphs_to_merge.iter()
                .map(|p| p.confidence)
                .sum::<f64>() / paragraphs_to_merge.len() as f64,
            speaker_id: self.resolve_speaker_conflict(&paragraphs_to_merge, &options.handle_speaker_conflicts),
            is_edited: true,
            edit_history: vec![edit_op],
        };
        
        // 删除原段落（从后往前删以避免索引变化）
        for &index in indices_to_remove.iter().rev() {
            document.paragraphs.remove(index);
        }
        
        // 插入合并后的段落到第一个原段落的位置
        let insert_index = indices_to_remove[0];
        document.paragraphs.insert(insert_index, merged_paragraph);
        
        // 更新文档状态
        document.is_dirty = true;
        document.version += 1;
        document.last_modified = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        document.metadata.paragraph_count = document.paragraphs.len();
        
        println!("🔗 合并段落: {:?} -> {}", paragraph_ids, merged_paragraph_id);
        Ok(merged_paragraph_id)
    }

    /// 编辑段落内容
    pub async fn edit_paragraph_content(
        &self,
        document_id: &str,
        paragraph_id: &str,
        new_content: &str,
        user_note: Option<String>,
    ) -> AppResult<()> {
        self.save_to_undo_stack(document_id);
        
        let mut documents = self.documents.lock();
        let document = documents.get_mut(document_id)
            .ok_or_else(|| AppError::ValidationError("文档不存在".to_string()))?;
        
        let paragraph = document.paragraphs
            .iter_mut()
            .find(|p| p.id == paragraph_id)
            .ok_or_else(|| AppError::ValidationError("段落不存在".to_string()))?;
        
        let old_content = paragraph.content.clone();
        
        let edit_op = EditOperation {
            operation_type: EditOperationType::Replace,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            old_content: Some(old_content),
            new_content: Some(new_content.to_string()),
            paragraph_ids: vec![paragraph_id.to_string()],
            user_note,
        };
        
        paragraph.content = new_content.to_string();
        paragraph.is_edited = true;
        paragraph.edit_history.push(edit_op);
        
        // 更新文档状态
        document.is_dirty = true;
        document.version += 1;
        document.last_modified = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        document.metadata.word_count = document.paragraphs
            .iter()
            .map(|p| p.content.split_whitespace().count())
            .sum();
        
        println!("✏️ 编辑段落: {}", paragraph_id);
        Ok(())
    }

    /// 搜索和替换
    pub async fn find_and_replace(
        &self,
        document_id: &str,
        options: FindReplaceOptions,
    ) -> AppResult<usize> {
        self.save_to_undo_stack(document_id);
        
        let mut documents = self.documents.lock();
        let document = documents.get_mut(document_id)
            .ok_or_else(|| AppError::ValidationError("文档不存在".to_string()))?;
        
        let mut replacements_count = 0;
        let find_text = if options.case_sensitive {
            options.find_text.clone()
        } else {
            options.find_text.to_lowercase()
        };
        
        // 确定搜索范围
        let paragraph_indices: Vec<usize> = match &options.scope {
            SearchScope::AllParagraphs => (0..document.paragraphs.len()).collect(),
            SearchScope::CurrentParagraph => vec![0], // 需要传入当前段落索引
            SearchScope::SelectedParagraphs(ids) => {
                ids.iter()
                    .filter_map(|id| document.paragraphs.iter().position(|p| &p.id == id))
                    .collect()
            }
            SearchScope::BySpeaker(speaker_id) => {
                document.paragraphs
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.speaker_id.as_ref() == Some(speaker_id))
                    .map(|(i, _)| i)
                    .collect()
            }
            SearchScope::ByTimeRange { start, end } => {
                document.paragraphs
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| {
                        if let (Some(p_start), Some(p_end)) = (p.start_timestamp, p.end_timestamp) {
                            p_start >= *start && p_end <= *end
                        } else {
                            false
                        }
                    })
                    .map(|(i, _)| i)
                    .collect()
            }
        };
        
        for &index in &paragraph_indices {
            let paragraph = &mut document.paragraphs[index];
            let content = if options.case_sensitive {
                paragraph.content.clone()
            } else {
                paragraph.content.to_lowercase()
            };
            
            if options.use_regex {
                // 正则表达式替换
                if let Ok(regex) = Regex::new(&find_text) {
                    let new_content = regex.replace_all(&paragraph.content, &options.replace_text);
                    if new_content != paragraph.content {
                        let edit_op = EditOperation {
                            operation_type: EditOperationType::Replace,
                            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                            old_content: Some(paragraph.content.clone()),
                            new_content: Some(new_content.to_string()),
                            paragraph_ids: vec![paragraph.id.clone()],
                            user_note: Some(format!("正则替换: {} -> {}", options.find_text, options.replace_text)),
                        };
                        
                        paragraph.content = new_content.to_string();
                        paragraph.is_edited = true;
                        paragraph.edit_history.push(edit_op);
                        replacements_count += 1;
                    }
                }
            } else if options.whole_word {
                // 整词替换
                let words: Vec<&str> = paragraph.content.split_whitespace().collect();
                let mut new_words: Vec<String> = Vec::new();
                let mut changed = false;
                
                for word in words {
                    let check_word = if options.case_sensitive { word } else { &word.to_lowercase() };
                    if check_word == find_text {
                        new_words.push(options.replace_text.clone());
                        changed = true;
                        replacements_count += 1;
                    } else {
                        new_words.push(word.to_string());
                    }
                }
                
                if changed {
                    let edit_op = EditOperation {
                        operation_type: EditOperationType::Replace,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        old_content: Some(paragraph.content.clone()),
                        new_content: Some(new_words.join(" ")),
                        paragraph_ids: vec![paragraph.id.clone()],
                        user_note: Some(format!("整词替换: {} -> {}", options.find_text, options.replace_text)),
                    };
                    
                    paragraph.content = new_words.join(" ");
                    paragraph.is_edited = true;
                    paragraph.edit_history.push(edit_op);
                }
            } else {
                // 简单字符串替换
                if content.contains(&find_text) {
                    let new_content = if options.case_sensitive {
                        paragraph.content.replace(&options.find_text, &options.replace_text)
                    } else {
                        // 大小写不敏感替换需要更复杂的逻辑
                        paragraph.content.clone() // 简化处理
                    };
                    
                    if new_content != paragraph.content {
                        let edit_op = EditOperation {
                            operation_type: EditOperationType::Replace,
                            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                            old_content: Some(paragraph.content.clone()),
                            new_content: Some(new_content.clone()),
                            paragraph_ids: vec![paragraph.id.clone()],
                            user_note: Some(format!("文本替换: {} -> {}", options.find_text, options.replace_text)),
                        };
                        
                        paragraph.content = new_content;
                        paragraph.is_edited = true;
                        paragraph.edit_history.push(edit_op);
                        replacements_count += 1;
                    }
                }
            }
        }
        
        if replacements_count > 0 {
            // 更新文档状态
            document.is_dirty = true;
            document.version += 1;
            document.last_modified = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        }
        
        println!("🔍 搜索替换完成: {} 处替换", replacements_count);
        Ok(replacements_count)
    }

    /// 撤销操作
    pub async fn undo(&self, document_id: &str) -> AppResult<()> {
        let mut undo_stacks = self.undo_stacks.lock();
        let mut redo_stacks = self.redo_stacks.lock();
        
        if let Some(undo_stack) = undo_stacks.get_mut(document_id) {
            if let Some(previous_state) = undo_stack.pop() {
                // 将当前状态保存到重做栈
                let mut documents = self.documents.lock();
                if let Some(current_document) = documents.get(document_id) {
                    redo_stacks.entry(document_id.to_string())
                        .or_insert_with(Vec::new)
                        .push(current_document.clone());
                }
                
                // 恢复到之前的状态
                documents.insert(document_id.to_string(), previous_state);
                
                println!("↩️ 撤销操作: {}", document_id);
                return Ok(());
            }
        }
        
        Err(AppError::ValidationError("没有可撤销的操作".to_string()))
    }

    /// 重做操作
    pub async fn redo(&self, document_id: &str) -> AppResult<()> {
        let mut undo_stacks = self.undo_stacks.lock();
        let mut redo_stacks = self.redo_stacks.lock();
        
        if let Some(redo_stack) = redo_stacks.get_mut(document_id) {
            if let Some(next_state) = redo_stack.pop() {
                // 将当前状态保存到撤销栈
                let mut documents = self.documents.lock();
                if let Some(current_document) = documents.get(document_id) {
                    undo_stacks.entry(document_id.to_string())
                        .or_insert_with(Vec::new)
                        .push(current_document.clone());
                }
                
                // 恢复到下一个状态
                documents.insert(document_id.to_string(), next_state);
                
                println!("↪️ 重做操作: {}", document_id);
                return Ok(());
            }
        }
        
        Err(AppError::ValidationError("没有可重做的操作".to_string()))
    }

    /// 保存文档到数据库（同步转录条目）
    pub async fn save_document(
        &self,
        document_id: &str,
        database: &crate::database::DatabaseManager,
    ) -> AppResult<()> {
        let mut documents = self.documents.lock();
        let document = documents.get_mut(document_id)
            .ok_or_else(|| AppError::ValidationError("文档不存在".to_string()))?;
        
        // 重新组合段落内容
        let combined_text = document.paragraphs
            .iter()
            .map(|p| p.content.trim())
            .collect::<Vec<&str>>()
            .join("\n\n");
        
        // 更新转录条目
        database.update_transcription(&document.entry_id, &combined_text)?;
        
        // 标记为已保存
        document.is_dirty = false;
        
        println!("💾 保存文档: {} -> 条目: {}", document_id, document.entry_id);
        Ok(())
    }

    // =================== 私有辅助方法 ===================

    /// 保存到撤销栈
    fn save_to_undo_stack(&self, document_id: &str) {
        if let Some(document) = self.documents.lock().get(document_id).cloned() {
            let mut undo_stacks = self.undo_stacks.lock();
            let stack = undo_stacks.entry(document_id.to_string()).or_insert_with(Vec::new);
            
            stack.push(document);
            
            // 限制撤销栈大小
            if stack.len() > self.max_undo_levels {
                stack.remove(0);
            }
            
            // 清空重做栈（因为有了新操作）
            self.redo_stacks.lock().remove(document_id);
        }
    }

    /// 创建单个段落
    fn create_single_paragraph(&self, text: &str) -> TextParagraph {
        TextParagraph {
            id: Uuid::new_v4().to_string(),
            content: text.to_string(),
            start_timestamp: None,
            end_timestamp: None,
            confidence: 0.9,
            speaker_id: None,
            is_edited: false,
            edit_history: Vec::new(),
        }
    }

    /// 按句号等结束符分割
    fn split_by_sentence_endings(&self, text: &str) -> Vec<String> {
        let sentence_endings = ['。', '.', '!', '?', '！', '？', '；', ';'];
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        
        for char in text.chars() {
            current_sentence.push(char);
            
            if sentence_endings.contains(&char) {
                sentences.push(current_sentence.trim().to_string());
                current_sentence.clear();
            }
        }
        
        // 处理最后一句
        if !current_sentence.trim().is_empty() {
            sentences.push(current_sentence.trim().to_string());
        }
        
        sentences
    }

    /// 判断是否为段落边界
    fn is_paragraph_boundary(&self, sentence: &str) -> bool {
        // 简单的段落边界判断逻辑
        let boundary_indicators = [
            "第一", "第二", "第三", "首先", "其次", "最后", "总之", "因此",
            "然而", "但是", "不过", "另外", "此外", "再者"
        ];
        
        boundary_indicators.iter().any(|&indicator| sentence.starts_with(indicator))
    }

    /// 解决说话人冲突
    fn resolve_speaker_conflict(
        &self,
        paragraphs: &[TextParagraph],
        strategy: &SpeakerConflictStrategy,
    ) -> Option<String> {
        match strategy {
            SpeakerConflictStrategy::KeepFirst => {
                paragraphs.first()?.speaker_id.clone()
            }
            SpeakerConflictStrategy::KeepMajority => {
                let mut speaker_counts: HashMap<String, usize> = HashMap::new();
                for paragraph in paragraphs {
                    if let Some(speaker) = &paragraph.speaker_id {
                        *speaker_counts.entry(speaker.clone()).or_insert(0) += 1;
                    }
                }
                
                speaker_counts.into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(speaker, _)| speaker)
            }
            SpeakerConflictStrategy::Merge => {
                Some("多人".to_string())
            }
            SpeakerConflictStrategy::AskUser => {
                // 在实际应用中应该显示对话框让用户选择
                None
            }
        }
    }
}