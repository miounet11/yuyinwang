use super::models::{DatabaseStats, ModelUsageStats, SearchFilter, SearchResult};
use crate::errors::{AppError, AppResult};
use crate::types::TranscriptionEntry;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    pub fn new(db_path: &Path) -> AppResult<Self> {
        let conn = Connection::open(db_path)
            .map_err(|e| AppError::DatabaseError(format!("无法打开数据库: {}", e)))?;

        let db_manager = DatabaseManager {
            conn: Arc::new(Mutex::new(conn)),
        };

        db_manager.init_database()?;
        Ok(db_manager)
    }

    fn init_database(&self) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        self.create_tables(&conn)?;
        self.create_indexes(&conn)?;

        println!("✅ 数据库初始化完成");
        Ok(())
    }

    fn create_tables(&self, conn: &Connection) -> AppResult<()> {
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
        )
        .map_err(|e| AppError::DatabaseError(format!("创建转录表失败: {}", e)))?;

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
        )
        .map_err(|e| AppError::DatabaseError(format!("创建模型统计表失败: {}", e)))?;

        // 创建应用设置表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| AppError::DatabaseError(format!("创建设置表失败: {}", e)))?;

        // 创建标签表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                color TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| AppError::DatabaseError(format!("创建标签表失败: {}", e)))?;

        Ok(())
    }

    fn create_indexes(&self, conn: &Connection) -> AppResult<()> {
        let indexes = vec![
            (
                "idx_transcriptions_timestamp",
                "transcriptions",
                "timestamp",
            ),
            ("idx_transcriptions_model", "transcriptions", "model"),
            ("idx_transcriptions_text", "transcriptions", "text"),
            (
                "idx_model_usage_last_used",
                "model_usage_stats",
                "last_used",
            ),
        ];

        for (index_name, table, column) in indexes {
            conn.execute(
                &format!(
                    "CREATE INDEX IF NOT EXISTS {} ON {}({})",
                    index_name, table, column
                ),
                [],
            )
            .map_err(|e| AppError::DatabaseError(format!("创建索引{}失败: {}", index_name, e)))?;
        }

        Ok(())
    }

    // 转录记录相关操作
    pub fn insert_transcription(&self, entry: &TranscriptionEntry) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

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
        )
        .map_err(|e| AppError::DatabaseError(format!("插入转录记录失败: {}", e)))?;

        // 更新模型使用统计
        drop(conn); // 释放锁
        self.update_model_stats(&entry.model, entry.duration, entry.confidence)?;

        println!("✅ 插入转录记录: {}", entry.id);
        Ok(())
    }

    pub fn update_transcription(&self, id: &str, new_text: &str) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let rows_affected = conn
            .execute(
                "UPDATE transcriptions SET text = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![new_text, id],
            )
            .map_err(|e| AppError::DatabaseError(format!("更新转录记录失败: {}", e)))?;

        if rows_affected == 0 {
            return Err(AppError::ValidationError(format!("转录记录不存在: {}", id)));
        }

        println!("✅ 更新转录记录: {}", id);
        Ok(())
    }

    pub fn get_transcription_by_id(&self, id: &str) -> AppResult<Option<TranscriptionEntry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, text, timestamp, duration, model, confidence,
             audio_file_path, created_at, updated_at, tags, metadata
             FROM transcriptions WHERE id = ?1",
            )
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let result = stmt
            .query_row(params![id], |row| {
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
            })
            .optional()
            .map_err(|e| AppError::DatabaseError(format!("查询转录记录失败: {}", e)))?;

        Ok(result)
    }

    pub fn delete_transcription(&self, id: &str) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let rows_affected = conn
            .execute("DELETE FROM transcriptions WHERE id = ?1", params![id])
            .map_err(|e| AppError::DatabaseError(format!("删除转录记录失败: {}", e)))?;

        if rows_affected == 0 {
            return Err(AppError::ValidationError(format!("转录记录不存在: {}", id)));
        }

        println!("✅ 删除转录记录: {}", id);
        Ok(())
    }

    pub fn get_all_transcriptions(&self) -> AppResult<Vec<TranscriptionEntry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, text, timestamp, duration, model, confidence,
             audio_file_path, created_at, updated_at, tags, metadata
             FROM transcriptions ORDER BY timestamp DESC",
            )
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let transcription_iter = stmt
            .query_map([], |row| {
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
            })
            .map_err(|e| AppError::DatabaseError(format!("执行查询失败: {}", e)))?;

        let mut transcriptions = Vec::new();
        for transcription in transcription_iter {
            transcriptions.push(
                transcription
                    .map_err(|e| AppError::DatabaseError(format!("读取记录失败: {}", e)))?,
            );
        }

        Ok(transcriptions)
    }

    pub fn search_transcriptions(
        &self,
        query: &str,
        filter: &SearchFilter,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> AppResult<SearchResult> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let mut where_clauses = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // 文本搜索
        if !query.trim().is_empty() {
            where_clauses.push("text LIKE ?".to_string());
            params.push(Box::new(format!("%{}%", query)));
        }

        // 应用过滤器
        if let Some(model) = &filter.model {
            where_clauses.push("model = ?".to_string());
            params.push(Box::new(model.clone()));
        }

        if let Some(min_conf) = filter.min_confidence {
            where_clauses.push("confidence >= ?".to_string());
            params.push(Box::new(min_conf));
        }

        if let Some(start_date) = filter.start_date {
            where_clauses.push("timestamp >= ?".to_string());
            params.push(Box::new(start_date));
        }

        if let Some(end_date) = filter.end_date {
            where_clauses.push("timestamp <= ?".to_string());
            params.push(Box::new(end_date));
        }

        if let Some(min_duration) = filter.min_duration {
            where_clauses.push("duration >= ?".to_string());
            params.push(Box::new(min_duration));
        }

        if let Some(max_duration) = filter.max_duration {
            where_clauses.push("duration <= ?".to_string());
            params.push(Box::new(max_duration));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_clauses.join(" AND "))
        };

        let limit_clause = match limit {
            Some(l) => format!(" LIMIT {}", l),
            None => String::new(),
        };

        let offset_clause = match offset {
            Some(o) => format!(" OFFSET {}", o),
            None => String::new(),
        };

        let sql = format!(
            "SELECT id, text, timestamp, duration, model, confidence,
             audio_file_path, created_at, updated_at, tags, metadata
             FROM transcriptions{} ORDER BY timestamp DESC{}{}",
            where_clause, limit_clause, offset_clause
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| AppError::DatabaseError(format!("准备搜索查询失败: {}", e)))?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let transcription_iter = stmt
            .query_map(&param_refs[..], |row| {
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
            })
            .map_err(|e| AppError::DatabaseError(format!("执行搜索查询失败: {}", e)))?;

        let mut entries = Vec::new();
        for entry in transcription_iter {
            entries.push(
                entry.map_err(|e| AppError::DatabaseError(format!("读取搜索结果失败: {}", e)))?,
            );
        }

        // 获取总计数
        let count_sql = format!("SELECT COUNT(*) FROM transcriptions{}", where_clause);
        let mut count_stmt = conn
            .prepare(&count_sql)
            .map_err(|e| AppError::DatabaseError(format!("准备计数查询失败: {}", e)))?;
        let total_count: usize = count_stmt
            .query_row(&param_refs[..], |row| Ok(row.get::<_, i64>(0)? as usize))
            .map_err(|e| AppError::DatabaseError(format!("执行计数查询失败: {}", e)))?;

        let has_more = limit.map_or(false, |l| {
            entries.len() == l && total_count > (offset.unwrap_or(0) + l)
        });

        Ok(SearchResult {
            entries,
            total_count,
            has_more,
        })
    }

    // 模型统计相关操作
    pub fn update_model_stats(
        &self,
        model_name: &str,
        duration: f64,
        confidence: f64,
    ) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        // 使用 INSERT OR REPLACE 来更新统计数据
        conn.execute(
            "INSERT OR REPLACE INTO model_usage_stats (
                model_name, usage_count, total_duration, average_confidence, last_used
            ) VALUES (
                ?1,
                COALESCE((SELECT usage_count FROM model_usage_stats WHERE model_name = ?1), 0) + 1,
                COALESCE((SELECT total_duration FROM model_usage_stats WHERE model_name = ?1), 0) + ?2,
                (COALESCE((SELECT average_confidence FROM model_usage_stats WHERE model_name = ?1), 0) *
                 COALESCE((SELECT usage_count FROM model_usage_stats WHERE model_name = ?1), 0) + ?3) /
                (COALESCE((SELECT usage_count FROM model_usage_stats WHERE model_name = ?1), 0) + 1),
                CURRENT_TIMESTAMP
            )",
            params![model_name, duration, confidence],
        ).map_err(|e| AppError::DatabaseError(format!("更新模型统计失败: {}", e)))?;

        Ok(())
    }

    pub fn get_model_usage_stats(&self) -> AppResult<Vec<ModelUsageStats>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, model_name, usage_count, total_duration, average_confidence, last_used
             FROM model_usage_stats ORDER BY usage_count DESC",
            )
            .map_err(|e| AppError::DatabaseError(format!("准备统计查询失败: {}", e)))?;

        let stats_iter = stmt
            .query_map([], |row| {
                Ok(ModelUsageStats {
                    id: row.get(0)?,
                    model_name: row.get(1)?,
                    usage_count: row.get(2)?,
                    total_duration: row.get(3)?,
                    average_confidence: row.get(4)?,
                    last_used: row.get(5)?,
                })
            })
            .map_err(|e| AppError::DatabaseError(format!("执行统计查询失败: {}", e)))?;

        let mut stats = Vec::new();
        for stat in stats_iter {
            stats.push(
                stat.map_err(|e| AppError::DatabaseError(format!("读取统计数据失败: {}", e)))?,
            );
        }

        Ok(stats)
    }

    // 设置相关操作
    pub fn get_setting(&self, key: &str) -> AppResult<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let result = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                params![key],
                |row| Ok(row.get::<_, String>(0)?),
            )
            .optional()
            .map_err(|e| AppError::DatabaseError(format!("查询设置失败: {}", e)))?;

        Ok(result)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            params![key, value],
        )
        .map_err(|e| AppError::DatabaseError(format!("设置配置失败: {}", e)))?;

        Ok(())
    }

    // 数据库统计信息
    pub fn get_database_stats(&self) -> AppResult<DatabaseStats> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let total_transcriptions: i64 = conn
            .query_row("SELECT COUNT(*) FROM transcriptions", [], |row| {
                Ok(row.get(0)?)
            })
            .map_err(|e| AppError::DatabaseError(format!("查询转录总数失败: {}", e)))?;

        let total_duration: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(duration), 0) FROM transcriptions",
                [],
                |row| Ok(row.get(0)?),
            )
            .map_err(|e| AppError::DatabaseError(format!("查询总时长失败: {}", e)))?;

        let most_used_model: Option<String> = conn
            .query_row(
                "SELECT model_name FROM model_usage_stats ORDER BY usage_count DESC LIMIT 1",
                [],
                |row| Ok(row.get(0)?),
            )
            .optional()
            .map_err(|e| AppError::DatabaseError(format!("查询最常用模型失败: {}", e)))?;

        let average_confidence: f64 = conn
            .query_row(
                "SELECT COALESCE(AVG(confidence), 0) FROM transcriptions",
                [],
                |row| Ok(row.get(0)?),
            )
            .map_err(|e| AppError::DatabaseError(format!("查询平均置信度失败: {}", e)))?;

        // 简化的数据库大小计算
        let database_size_mb = 0.0; // TODO: 实现实际的文件大小计算

        Ok(DatabaseStats {
            total_transcriptions,
            total_duration,
            most_used_model,
            average_confidence,
            database_size_mb,
        })
    }

    // 数据库维护
    pub fn vacuum(&self) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        conn.execute("VACUUM", [])
            .map_err(|e| AppError::DatabaseError(format!("数据库压缩失败: {}", e)))?;

        println!("✅ 数据库压缩完成");
        Ok(())
    }

    // Story 1.4: Local Whisper Model Management Methods

    /// Get all local models from database
    pub async fn get_local_models(&self) -> AppResult<Vec<super::models::LocalModelInfo>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let mut stmt = conn
            .prepare(
                "SELECT model_id, model_name, file_path, size_bytes, download_status,
             download_progress, accuracy_score, performance_rating, supported_languages,
             created_at, last_verified, metadata
             FROM local_whisper_models ORDER BY model_name",
            )
            .map_err(|e| AppError::DatabaseError(format!("准备本地模型查询失败: {}", e)))?;

        let model_iter = stmt
            .query_map([], |row| {
                let download_status_str: String = row.get(4)?;
                let download_progress: f64 = row.get(5)?;

                let download_status = match download_status_str.as_str() {
                    "NotDownloaded" => super::models::DownloadStatus::NotDownloaded,
                    "Downloading" => super::models::DownloadStatus::Downloading {
                        progress: download_progress,
                    },
                    "Downloaded" => super::models::DownloadStatus::Downloaded,
                    "Corrupted" => super::models::DownloadStatus::Corrupted,
                    "UpdateAvailable" => super::models::DownloadStatus::UpdateAvailable,
                    _ => super::models::DownloadStatus::NotDownloaded,
                };

                let file_path_str: Option<String> = row.get(2)?;
                let file_path = file_path_str.map(|p| std::path::PathBuf::from(p));

                let supported_languages_str: String = row.get(8)?;
                let supported_languages: Vec<String> =
                    serde_json::from_str(&supported_languages_str)
                        .unwrap_or_else(|_| vec!["en".to_string()]);

                let created_at_str: String = row.get(9)?;
                let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .with_timezone(&chrono::Utc);

                let last_verified_str: Option<String> = row.get(10)?;
                let last_verified = last_verified_str.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .ok()
                });

                let metadata_str: String = row.get(11)?;
                let metadata = serde_json::from_str(&metadata_str)
                    .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

                Ok(super::models::LocalModelInfo {
                    model_id: row.get(0)?,
                    model_name: row.get(1)?,
                    file_path,
                    size_bytes: row.get(3)?,
                    download_status,
                    accuracy_score: row.get(6)?,
                    performance_rating: row.get(7)?,
                    supported_languages,
                    created_at,
                    last_verified,
                    metadata,
                })
            })
            .map_err(|e| AppError::DatabaseError(format!("执行本地模型查询失败: {}", e)))?;

        let mut models = Vec::new();
        for model in model_iter {
            models
                .push(model.map_err(|e| {
                    AppError::DatabaseError(format!("读取本地模型数据失败: {}", e))
                })?);
        }

        Ok(models)
    }

    /// Get a specific local model by ID
    pub async fn get_local_model(
        &self,
        model_id: &str,
    ) -> AppResult<Option<super::models::LocalModelInfo>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let result = conn
            .query_row(
                "SELECT model_id, model_name, file_path, size_bytes, download_status,
             download_progress, accuracy_score, performance_rating, supported_languages,
             created_at, last_verified, metadata
             FROM local_whisper_models WHERE model_id = ?1",
                params![model_id],
                |row| {
                    let download_status_str: String = row.get(4)?;
                    let download_progress: f64 = row.get(5)?;

                    let download_status = match download_status_str.as_str() {
                        "NotDownloaded" => super::models::DownloadStatus::NotDownloaded,
                        "Downloading" => super::models::DownloadStatus::Downloading {
                            progress: download_progress,
                        },
                        "Downloaded" => super::models::DownloadStatus::Downloaded,
                        "Corrupted" => super::models::DownloadStatus::Corrupted,
                        "UpdateAvailable" => super::models::DownloadStatus::UpdateAvailable,
                        _ => super::models::DownloadStatus::NotDownloaded,
                    };

                    let file_path_str: Option<String> = row.get(2)?;
                    let file_path = file_path_str.map(|p| std::path::PathBuf::from(p));

                    let supported_languages_str: String = row.get(8)?;
                    let supported_languages: Vec<String> =
                        serde_json::from_str(&supported_languages_str)
                            .unwrap_or_else(|_| vec!["en".to_string()]);

                    let created_at_str: String = row.get(9)?;
                    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .unwrap_or_else(|_| chrono::Utc::now().into())
                        .with_timezone(&chrono::Utc);

                    let last_verified_str: Option<String> = row.get(10)?;
                    let last_verified = last_verified_str.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .ok()
                    });

                    let metadata_str: String = row.get(11)?;
                    let metadata = serde_json::from_str(&metadata_str)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

                    Ok(super::models::LocalModelInfo {
                        model_id: row.get(0)?,
                        model_name: row.get(1)?,
                        file_path,
                        size_bytes: row.get(3)?,
                        download_status,
                        accuracy_score: row.get(6)?,
                        performance_rating: row.get(7)?,
                        supported_languages,
                        created_at,
                        last_verified,
                        metadata,
                    })
                },
            )
            .optional()
            .map_err(|e| AppError::DatabaseError(format!("查询本地模型失败: {}", e)))?;

        Ok(result)
    }

    /// Insert a new local model record
    pub async fn insert_local_model(&self, model: &super::models::LocalModelInfo) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let download_status_str = match &model.download_status {
            super::models::DownloadStatus::NotDownloaded => "NotDownloaded",
            super::models::DownloadStatus::Downloading { .. } => "Downloading",
            super::models::DownloadStatus::Downloaded => "Downloaded",
            super::models::DownloadStatus::Corrupted => "Corrupted",
            super::models::DownloadStatus::UpdateAvailable => "UpdateAvailable",
        };

        let download_progress = match &model.download_status {
            super::models::DownloadStatus::Downloading { progress } => *progress,
            _ => 0.0,
        };

        let file_path_str = model
            .file_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let supported_languages_str = serde_json::to_string(&model.supported_languages).unwrap();
        let created_at_str = model.created_at.to_rfc3339();
        let last_verified_str = model.last_verified.map(|dt| dt.to_rfc3339());
        let metadata_str = model.metadata.to_string();

        conn.execute(
            "INSERT OR REPLACE INTO local_whisper_models
             (model_id, model_name, file_path, size_bytes, download_status, download_progress,
              accuracy_score, performance_rating, supported_languages, created_at, last_verified, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                model.model_id,
                model.model_name,
                file_path_str,
                model.size_bytes,
                download_status_str,
                download_progress,
                model.accuracy_score,
                model.performance_rating,
                supported_languages_str,
                created_at_str,
                last_verified_str,
                metadata_str
            ],
        ).map_err(|e| AppError::DatabaseError(format!("插入本地模型失败: {}", e)))?;

        Ok(())
    }

    /// Update model download status
    pub async fn update_model_status(
        &self,
        model_id: &str,
        status: super::models::DownloadStatus,
    ) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let status_str = match &status {
            super::models::DownloadStatus::NotDownloaded => "NotDownloaded",
            super::models::DownloadStatus::Downloading { .. } => "Downloading",
            super::models::DownloadStatus::Downloaded => "Downloaded",
            super::models::DownloadStatus::Corrupted => "Corrupted",
            super::models::DownloadStatus::UpdateAvailable => "UpdateAvailable",
        };

        let progress = match &status {
            super::models::DownloadStatus::Downloading { progress } => *progress,
            _ => 0.0,
        };

        conn.execute(
            "UPDATE local_whisper_models SET download_status = ?1, download_progress = ?2 WHERE model_id = ?3",
            params![status_str, progress, model_id],
        ).map_err(|e| AppError::DatabaseError(format!("更新模型状态失败: {}", e)))?;

        Ok(())
    }

    /// Update model file path
    pub async fn update_model_file_path(
        &self,
        model_id: &str,
        file_path: Option<std::path::PathBuf>,
    ) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let file_path_str = file_path.map(|p| p.to_string_lossy().to_string());

        conn.execute(
            "UPDATE local_whisper_models SET file_path = ?1 WHERE model_id = ?2",
            params![file_path_str, model_id],
        )
        .map_err(|e| AppError::DatabaseError(format!("更新模型文件路径失败: {}", e)))?;

        Ok(())
    }

    /// Update model size
    pub async fn update_model_size(&self, model_id: &str, size_bytes: u64) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        conn.execute(
            "UPDATE local_whisper_models SET size_bytes = ?1 WHERE model_id = ?2",
            params![size_bytes, model_id],
        )
        .map_err(|e| AppError::DatabaseError(format!("更新模型大小失败: {}", e)))?;

        Ok(())
    }

    /// Update model last verified timestamp
    pub async fn update_model_last_verified(
        &self,
        model_id: &str,
        last_verified: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AppResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接锁: {}", e)))?;

        let last_verified_str = last_verified.map(|dt| dt.to_rfc3339());

        conn.execute(
            "UPDATE local_whisper_models SET last_verified = ?1 WHERE model_id = ?2",
            params![last_verified_str, model_id],
        )
        .map_err(|e| AppError::DatabaseError(format!("更新模型验证时间失败: {}", e)))?;

        Ok(())
    }
}
