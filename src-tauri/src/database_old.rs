/**
 * Recording King SQLite 数据库管理器
 * 提供持久化存储和数据管理功能
 */

use rusqlite::{Connection, Result, params};
use std::path::Path;
use std::sync::{Arc, Mutex};

// 使用主模块的 TranscriptionEntry
use crate::TranscriptionEntry;

#[derive(Debug)]
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db_manager = DatabaseManager {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db_manager.init_database()?;
        Ok(db_manager)
    }

    fn init_database(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // 创建转录记录表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                duration REAL NOT NULL,
                model TEXT NOT NULL,
                confidence REAL NOT NULL,
                audio_file_path TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                tags TEXT, -- JSON字符串
                metadata TEXT -- JSON字符串
            )",
            [],
        )?;

        // 创建模型使用统计表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS model_usage_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                model_name TEXT NOT NULL,
                usage_count INTEGER DEFAULT 1,
                total_duration REAL DEFAULT 0,
                average_confidence REAL DEFAULT 0,
                last_used DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(model_name)
            )",
            [],
        )?;

        // 创建应用设置表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // 创建标签表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                color TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // 创建索引以提高查询性能
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_timestamp ON transcriptions(timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_model ON transcriptions(model)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_text ON transcriptions(text)",
            [],
        )?;

        println!("✅ 数据库初始化完成");
        Ok(())
    }

    // 插入新的转录记录
    pub fn insert_transcription(&self, entry: &TranscriptionEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO transcriptions (
                id, text, timestamp, duration, model, confidence, 
                audio_file_path, tags, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry.id,
                entry.text,
                entry.timestamp,
                entry.duration,
                entry.model,
                entry.confidence,
                entry.audio_file_path,
                entry.tags,
                entry.metadata
            ],
        )?;

        // 更新模型使用统计
        self.update_model_stats(&entry.model, entry.duration, entry.confidence)?;
        
        println!("✅ 插入转录记录: {}", entry.id);
        Ok(())
    }

    // 更新转录记录
    pub fn update_transcription(&self, id: &str, new_text: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE transcriptions SET text = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![new_text, id],
        )?;
        
        println!("✅ 更新转录记录: {}", id);
        Ok(())
    }

    // 删除转录记录
    pub fn delete_transcription(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute("DELETE FROM transcriptions WHERE id = ?1", params![id])?;
        
        println!("✅ 删除转录记录: {}", id);
        Ok(())
    }

    // 获取所有转录记录
    pub fn get_all_transcriptions(&self) -> Result<Vec<TranscriptionEntry>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, text, timestamp, duration, model, confidence, 
             audio_file_path, created_at, updated_at, tags, metadata 
             FROM transcriptions ORDER BY timestamp DESC"
        )?;

        let transcription_iter = stmt.query_map([], |row| {
            Ok(TranscriptionEntry {
                id: row.get(0)?,
                text: row.get(1)?,
                timestamp: row.get(2)?,
                duration: row.get(3)?,
                model: row.get(4)?,
                confidence: row.get(5)?,
                audio_file_path: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                tags: row.get(9)?,
                metadata: row.get(10)?,
            })
        })?;

        let mut transcriptions = Vec::new();
        for transcription in transcription_iter {
            transcriptions.push(transcription?);
        }

        Ok(transcriptions)
    }

    // 搜索转录记录
    pub fn search_transcriptions(&self, query: &str, limit: Option<usize>) -> Result<Vec<TranscriptionEntry>> {
        let conn = self.conn.lock().unwrap();
        
        let limit_clause = match limit {
            Some(l) => format!(" LIMIT {}", l),
            None => String::new(),
        };
        
        let sql = format!(
            "SELECT id, text, timestamp, duration, model, confidence, 
             audio_file_path, created_at, updated_at, tags, metadata 
             FROM transcriptions 
             WHERE text LIKE ?1 OR model LIKE ?1 OR audio_file_path LIKE ?1
             ORDER BY timestamp DESC{}",
            limit_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let search_pattern = format!("%{}%", query);

        let transcription_iter = stmt.query_map(params![search_pattern], |row| {
            Ok(TranscriptionEntry {
                id: row.get(0)?,
                text: row.get(1)?,
                timestamp: row.get(2)?,
                duration: row.get(3)?,
                model: row.get(4)?,
                confidence: row.get(5)?,
                audio_file_path: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                tags: row.get(9)?,
                metadata: row.get(10)?,
            })
        })?;

        let mut transcriptions = Vec::new();
        for transcription in transcription_iter {
            transcriptions.push(transcription?);
        }

        Ok(transcriptions)
    }

    // 按模型筛选转录记录
    pub fn get_transcriptions_by_model(&self, model: &str) -> Result<Vec<TranscriptionEntry>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, text, timestamp, duration, model, confidence, 
             audio_file_path, created_at, updated_at, tags, metadata 
             FROM transcriptions WHERE model = ?1 ORDER BY timestamp DESC"
        )?;

        let transcription_iter = stmt.query_map(params![model], |row| {
            Ok(TranscriptionEntry {
                id: row.get(0)?,
                text: row.get(1)?,
                timestamp: row.get(2)?,
                duration: row.get(3)?,
                model: row.get(4)?,
                confidence: row.get(5)?,
                audio_file_path: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                tags: row.get(9)?,
                metadata: row.get(10)?,
            })
        })?;

        let mut transcriptions = Vec::new();
        for transcription in transcription_iter {
            transcriptions.push(transcription?);
        }

        Ok(transcriptions)
    }

    // 更新模型使用统计
    fn update_model_stats(&self, model: &str, duration: f64, confidence: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // 尝试更新现有记录
        let updated = conn.execute(
            "UPDATE model_usage_stats 
             SET usage_count = usage_count + 1,
                 total_duration = total_duration + ?1,
                 average_confidence = (average_confidence * (usage_count - 1) + ?2) / usage_count,
                 last_used = CURRENT_TIMESTAMP
             WHERE model_name = ?3",
            params![duration, confidence, model],
        )?;

        // 如果没有更新任何记录，则插入新记录
        if updated == 0 {
            conn.execute(
                "INSERT INTO model_usage_stats (model_name, usage_count, total_duration, average_confidence)
                 VALUES (?1, 1, ?2, ?3)",
                params![model, duration, confidence],
            )?;
        }

        Ok(())
    }

    // 获取模型使用统计
    pub fn get_model_stats(&self) -> Result<Vec<(String, i32, f64, f64)>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT model_name, usage_count, total_duration, average_confidence 
             FROM model_usage_stats ORDER BY usage_count DESC"
        )?;

        let stats_iter = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
            ))
        })?;

        let mut stats = Vec::new();
        for stat in stats_iter {
            stats.push(stat?);
        }

        Ok(stats)
    }

    // 获取应用设置
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    // 设置应用设置
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) 
             VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            params![key, value],
        )?;

        Ok(())
    }

    // 清理旧记录（可选的数据管理功能）
    pub fn cleanup_old_records(&self, days_to_keep: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (days_to_keep * 24 * 60 * 60);
        
        let deleted = conn.execute(
            "DELETE FROM transcriptions WHERE timestamp < ?1",
            params![cutoff_timestamp],
        )?;

        println!("🗑️ 清理了 {} 条旧记录", deleted);
        Ok(deleted)
    }

    // 获取数据库统计信息
    pub fn get_database_stats(&self) -> Result<(usize, f64, usize)> {
        let conn = self.conn.lock().unwrap();
        
        // 总记录数
        let total_records: usize = conn.query_row(
            "SELECT COUNT(*) FROM transcriptions",
            [],
            |row| row.get(0)
        )?;

        // 总转录时长
        let total_duration: f64 = conn.query_row(
            "SELECT COALESCE(SUM(duration), 0) FROM transcriptions",
            [],
            |row| row.get(0)
        )?;

        // 使用的模型数量
        let unique_models: usize = conn.query_row(
            "SELECT COUNT(DISTINCT model) FROM transcriptions",
            [],
            |row| row.get(0)
        )?;

        Ok((total_records, total_duration, unique_models))
    }

    // 导出数据为JSON
    pub fn export_to_json(&self) -> Result<String> {
        let transcriptions = self.get_all_transcriptions()?;
        let json = serde_json::to_string_pretty(&transcriptions)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        Ok(json)
    }
}