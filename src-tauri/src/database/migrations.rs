// 数据库迁移管理 - 支持本地Whisper模型

use crate::errors::{AppError, AppResult};
use rusqlite::Connection;

pub struct DatabaseMigration;

impl DatabaseMigration {
    pub fn run_migrations(conn: &Connection) -> AppResult<()> {
        println!("🔄 Running database migrations...");

        // Check current schema version
        let version = Self::get_schema_version(conn)?;
        println!("📊 Current schema version: {}", version);

        // Run migrations in order
        if version < 1 {
            Self::migrate_to_v1(conn)?;
        }

        if version < 2 {
            Self::migrate_to_v2_local_models(conn)?;
        }

        println!("✅ Database migrations completed successfully");
        Ok(())
    }

    fn get_schema_version(conn: &Connection) -> AppResult<i32> {
        // Create schema_version table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            )",
            [],
        )
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to create schema_version table: {}", e))
        })?;

        // Get current version, default to 0 if no version exists
        let version: Result<i32, rusqlite::Error> = conn.query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        );

        Ok(version.unwrap_or(0))
    }

    fn set_schema_version(conn: &Connection, version: i32) -> AppResult<()> {
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [version],
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to set schema version: {}", e)))?;

        Ok(())
    }

    fn migrate_to_v1(conn: &Connection) -> AppResult<()> {
        println!("📈 Migrating to schema version 1...");

        // This would contain any existing table migrations
        // For now, we assume the basic tables already exist

        Self::set_schema_version(conn, 1)?;
        println!("✅ Migration to v1 completed");
        Ok(())
    }

    /// Story 1.4: Add local Whisper model management tables
    fn migrate_to_v2_local_models(conn: &Connection) -> AppResult<()> {
        println!("📈 Migrating to schema version 2 (Local Whisper Models)...");

        // Create local_whisper_models table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS local_whisper_models (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                model_id TEXT UNIQUE NOT NULL,
                model_name TEXT NOT NULL,
                file_path TEXT,
                size_bytes INTEGER NOT NULL DEFAULT 0,
                download_status TEXT NOT NULL DEFAULT 'NotDownloaded',
                download_progress REAL DEFAULT 0.0,
                accuracy_score REAL,
                performance_rating REAL,
                supported_languages TEXT NOT NULL DEFAULT '[]',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_verified TIMESTAMP,
                metadata TEXT DEFAULT '{}'
            )",
            [],
        )
        .map_err(|e| {
            AppError::DatabaseError(format!(
                "Failed to create local_whisper_models table: {}",
                e
            ))
        })?;

        // Create model_performance_metrics table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS model_performance_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                model_id TEXT NOT NULL,
                average_processing_time_ms INTEGER NOT NULL DEFAULT 0,
                gpu_acceleration_used BOOLEAN NOT NULL DEFAULT 0,
                memory_usage_mb INTEGER NOT NULL DEFAULT 0,
                cpu_usage_percent REAL NOT NULL DEFAULT 0.0,
                accuracy_samples TEXT DEFAULT '[]',
                last_benchmark TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (model_id) REFERENCES local_whisper_models(model_id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| {
            AppError::DatabaseError(format!(
                "Failed to create model_performance_metrics table: {}",
                e
            ))
        })?;

        // Create indexes for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_local_models_model_id ON local_whisper_models(model_id)",
            [],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to create model_id index: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_local_models_status ON local_whisper_models(download_status)",
            [],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to create download_status index: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_performance_model_id ON model_performance_metrics(model_id)",
            [],
        ).map_err(|e| AppError::DatabaseError(format!("Failed to create performance model_id index: {}", e)))?;

        // Insert default model configurations
        Self::insert_default_models(conn)?;

        Self::set_schema_version(conn, 2)?;
        println!("✅ Migration to v2 completed - Local model tables created");
        Ok(())
    }

    fn insert_default_models(conn: &Connection) -> AppResult<()> {
        println!("📦 Inserting default Whisper model configurations...");

        let models = [
            (
                "whisper-tiny",
                "whisper-tiny",
                39_000_000,
                "Fastest model, lower accuracy",
            ),
            (
                "whisper-base",
                "whisper-base",
                142_000_000,
                "Good balance of speed and accuracy",
            ),
            (
                "whisper-small",
                "whisper-small",
                244_000_000,
                "Better accuracy, moderate speed",
            ),
            (
                "whisper-medium",
                "whisper-medium",
                769_000_000,
                "High accuracy, slower processing",
            ),
            (
                "whisper-large",
                "whisper-large",
                1_550_000_000,
                "Highest accuracy, slowest processing",
            ),
        ];

        for (model_id, model_name, size_bytes, description) in models.iter() {
            let supported_languages =
                serde_json::to_string(&vec!["en", "zh", "es", "fr", "de"]).unwrap();
            let metadata = serde_json::json!({
                "description": description,
                "source": "huggingface",
                "architecture": "whisper",
                "recommended_use": match *model_id {
                    "whisper-tiny" => "Real-time transcription, testing",
                    "whisper-base" => "General purpose transcription",
                    "whisper-small" => "High quality transcription",
                    "whisper-medium" => "Professional transcription, accuracy critical",
                    "whisper-large" => "Maximum accuracy needs, offline professional use",
                    _ => "General use"
                }
            });

            conn.execute(
                "INSERT OR IGNORE INTO local_whisper_models
                 (model_id, model_name, size_bytes, supported_languages, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                [
                    model_id,
                    model_name,
                    size_bytes.to_string().as_str(),
                    &supported_languages,
                    &metadata.to_string(),
                ],
            )
            .map_err(|e| {
                AppError::DatabaseError(format!(
                    "Failed to insert default model {}: {}",
                    model_id, e
                ))
            })?;
        }

        println!("✅ Default model configurations inserted");
        Ok(())
    }
}
