// 异步数据库管理器 - 连接池版本（性能优化）
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Result as SqliteResult, params, OptionalExtension};
use std::path::Path;
use tokio::task;
use crate::errors::{AppError, AppResult};
use crate::types::TranscriptionEntry;
use super::models::{ModelUsageStats, AppSetting, Tag, SearchResult, DatabaseStats, SearchFilter};

pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type SqliteConnection = PooledConnection<SqliteConnectionManager>;

#[derive(Debug, Clone)]
pub struct AsyncDatabaseManager {
    pool: SqlitePool,
}

impl AsyncDatabaseManager {
    pub fn new(db_path: &Path) -> AppResult<Self> {
        let manager = SqliteConnectionManager::file(db_path)
            .with_flags(rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE)
            .with_init(|conn| {
                // 启用WAL模式提高并发性能
                conn.execute("PRAGMA journal_mode=WAL", [])?;
                // 启用外键约束
                conn.execute("PRAGMA foreign_keys=ON", [])?;
                // 优化性能设置
                conn.execute("PRAGMA synchronous=NORMAL", [])?;
                conn.execute("PRAGMA cache_size=10000", [])?;
                conn.execute("PRAGMA temp_store=MEMORY", [])?;
                Ok(())
            });

        let pool = Pool::builder()
            .min_idle(Some(1))
            .max_size(10) // 最大10个连接
            .build(manager)
            .map_err(|e| AppError::DatabaseError(format!("连接池创建失败: {}", e)))?;

        let pool_clone = pool.clone();
        let db_manager = AsyncDatabaseManager { pool };
        
        // 在单独的任务中初始化数据库
        tokio::spawn(async move {
            if let Err(e) = Self::init_database_async(pool_clone).await {
                eprintln!("数据库初始化失败: {}", e);
            }
        });

        Ok(db_manager)
    }

    async fn init_database_async(pool: SqlitePool) -> AppResult<()> {
        let pool_clone = pool.clone();
        task::spawn_blocking(move || {
            let conn = pool_clone.get()
                .map_err(|e| AppError::DatabaseError(format!("无法获取数据库连接: {}", e)))?;
            
            Self::create_tables_sync(&conn)?;
            Self::create_indexes_sync(&conn)?;
            
            println!("✅ 异步数据库初始化完成");
            Ok::<(), AppError>(())
        }).await
        .map_err(|e| AppError::DatabaseError(format!("初始化任务失败: {}", e)))?
    }

    fn create_tables_sync(conn: &SqliteConnection) -> AppResult<()> {
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
                metadata TEXT -- JSON字符串，存储额外信息
            )", []
        ).map_err(|e| AppError::DatabaseError(format!("创建表失败: {}", e)))?;

        // 创建模型使用统计表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS model_usage_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                model_name TEXT NOT NULL,
                usage_count INTEGER DEFAULT 1,
                total_duration REAL DEFAULT 0.0,
                average_confidence REAL DEFAULT 0.0,
                last_used DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(model_name)
            )", []
        ).map_err(|e| AppError::DatabaseError(format!("创建统计表失败: {}", e)))?;

        // 创建应用设置表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                setting_type TEXT NOT NULL DEFAULT 'string',
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )", []
        ).map_err(|e| AppError::DatabaseError(format!("创建设置表失败: {}", e)))?;

        // 创建标签表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                color TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )", []
        ).map_err(|e| AppError::DatabaseError(format!("创建标签表失败: {}", e)))?;

        Ok(())
    }

    fn create_indexes_sync(conn: &SqliteConnection) -> AppResult<()> {
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_timestamp ON transcriptions(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_model ON transcriptions(model)", 
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_confidence ON transcriptions(confidence)",
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_created_at ON transcriptions(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_model_stats_last_used ON model_usage_stats(last_used)",
        ];

        for index_sql in indexes {
            conn.execute(index_sql, [])
                .map_err(|e| AppError::DatabaseError(format!("创建索引失败: {}", e)))?;
        }

        Ok(())
    }

    // 异步方法：插入转录记录
    pub async fn insert_transcription(&self, entry: &TranscriptionEntry) -> AppResult<()> {
        let pool = self.pool.clone();
        let entry = entry.clone();
        
        task::spawn_blocking(move || {
            let conn = pool.get()
                .map_err(|e| AppError::DatabaseError(format!("获取连接失败: {}", e)))?;

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
            ).map_err(|e| AppError::DatabaseError(format!("插入转录记录失败: {}", e)))?;

            Ok::<(), AppError>(())
        }).await
        .map_err(|e| AppError::DatabaseError(format!("异步任务失败: {}", e)))?
    }

    // 异步方法：获取转录历史
    pub async fn get_transcriptions(&self, limit: Option<u32>, offset: Option<u32>) -> AppResult<Vec<TranscriptionEntry>> {
        let pool = self.pool.clone();
        
        task::spawn_blocking(move || {
            let conn = pool.get()
                .map_err(|e| AppError::DatabaseError(format!("获取连接失败: {}", e)))?;

            let limit = limit.unwrap_or(50);
            let offset = offset.unwrap_or(0);

            let mut stmt = conn.prepare(
                "SELECT id, text, timestamp, duration, model, confidence, 
                        audio_file_path, created_at, updated_at, tags, metadata
                 FROM transcriptions 
                 ORDER BY timestamp DESC 
                 LIMIT ?1 OFFSET ?2"
            ).map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

            let entries = stmt.query_map(params![limit, offset], |row| {
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
            }).map_err(|e| AppError::DatabaseError(format!("查询失败: {}", e)))?;

            let mut results = Vec::new();
            for entry in entries {
                results.push(entry.map_err(|e| AppError::DatabaseError(format!("解析记录失败: {}", e)))?);
            }

            Ok(results)
        }).await
        .map_err(|e| AppError::DatabaseError(format!("异步任务失败: {}", e)))?
    }

    // 异步方法：搜索转录记录
    pub async fn search_transcriptions(&self, filter: &SearchFilter) -> AppResult<SearchResult> {
        let pool = self.pool.clone();
        let filter = filter.clone();
        
        task::spawn_blocking(move || {
            let conn = pool.get()
                .map_err(|e| AppError::DatabaseError(format!("获取连接失败: {}", e)))?;

            let mut query = String::from(
                "SELECT id, text, timestamp, model, confidence, 
                        audio_file_path, created_at
                 FROM transcriptions WHERE 1=1"
            );
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            // TODO: 实现文本搜索功能 - 当前SearchFilter没有query字段

            if let Some(model) = &filter.model {
                query.push_str(" AND model = ?");
                params.push(Box::new(model.clone()));
            }

            if let Some(min_confidence) = filter.min_confidence {
                query.push_str(" AND confidence >= ?");
                params.push(Box::new(min_confidence));
            }

            query.push_str(" ORDER BY timestamp DESC");

            // 默认限制结果数量
            query.push_str(" LIMIT 100");

            let mut stmt = conn.prepare(&query)
                .map_err(|e| AppError::DatabaseError(format!("准备搜索查询失败: {}", e)))?;

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let results = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(crate::types::TranscriptionEntry {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                    duration: row.get("duration")?,
                    model: row.get(3)?,
                    confidence: row.get(4)?,
                    audio_file_path: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get("updated_at")?,
                    tags: row.get("tags")?,
                    metadata: row.get("metadata")?,
                })
            }).map_err(|e| AppError::DatabaseError(format!("执行搜索失败: {}", e)))?;

            let mut entries = Vec::new();
            for result in results {
                entries.push(result.map_err(|e| AppError::DatabaseError(format!("解析搜索结果失败: {}", e)))?);
            }

            let total_count = entries.len();
            Ok(SearchResult {
                entries,
                total_count,
                has_more: false, // TODO: 实现分页逻辑
            })
        }).await
        .map_err(|e| AppError::DatabaseError(format!("异步搜索任务失败: {}", e)))?
    }

    // 异步方法：获取数据库统计信息
    pub async fn get_database_stats(&self) -> AppResult<DatabaseStats> {
        let pool = self.pool.clone();
        
        task::spawn_blocking(move || {
            let conn = pool.get()
                .map_err(|e| AppError::DatabaseError(format!("获取连接失败: {}", e)))?;

            let total_transcriptions: u64 = conn.query_row(
                "SELECT COUNT(*) FROM transcriptions", 
                [], 
                |row| row.get(0)
            ).unwrap_or(0);

            let total_duration: f64 = conn.query_row(
                "SELECT COALESCE(SUM(duration), 0.0) FROM transcriptions", 
                [], 
                |row| row.get(0)
            ).unwrap_or(0.0);

            let average_confidence: f64 = conn.query_row(
                "SELECT COALESCE(AVG(confidence), 0.0) FROM transcriptions", 
                [], 
                |row| row.get(0)
            ).unwrap_or(0.0);

            let unique_models: u64 = conn.query_row(
                "SELECT COUNT(DISTINCT model) FROM transcriptions", 
                [], 
                |row| row.get(0)
            ).unwrap_or(0);

            Ok(DatabaseStats {
                total_transcriptions: total_transcriptions as i64,
                total_duration,
                most_used_model: None, // TODO: 实现最常用模型查询
                average_confidence,
                database_size_mb: 0.0, // TODO: 需要额外计算
            })
        }).await
        .map_err(|e| AppError::DatabaseError(format!("统计任务失败: {}", e)))?
    }

    // 批量操作：清理旧记录
    pub async fn cleanup_old_records(&self, days_to_keep: u32) -> AppResult<u64> {
        let pool = self.pool.clone();
        
        task::spawn_blocking(move || {
            let conn = pool.get()
                .map_err(|e| AppError::DatabaseError(format!("获取连接失败: {}", e)))?;

            let cutoff_timestamp = chrono::Utc::now().timestamp() - (days_to_keep as i64 * 24 * 60 * 60);
            
            let deleted_count = conn.execute(
                "DELETE FROM transcriptions WHERE timestamp < ?1",
                params![cutoff_timestamp]
            ).map_err(|e| AppError::DatabaseError(format!("清理记录失败: {}", e)))?;

            // 清理孤立的统计数据
            conn.execute(
                "DELETE FROM model_usage_stats WHERE model_name NOT IN (SELECT DISTINCT model FROM transcriptions)",
                []
            ).map_err(|e| AppError::DatabaseError(format!("清理统计数据失败: {}", e)))?;

            // 执行VACUUM优化存储空间
            conn.execute("VACUUM", [])
                .map_err(|e| AppError::DatabaseError(format!("数据库优化失败: {}", e)))?;

            Ok(deleted_count as u64)
        }).await
        .map_err(|e| AppError::DatabaseError(format!("清理任务失败: {}", e)))?
    }

    // 获取连接池统计信息
    pub fn get_pool_stats(&self) -> (u32, u32) {
        let state = self.pool.state();
        (state.connections, state.idle_connections)
    }
}