//! Database module for reStrike VTA
//! 
//! This module provides SQLite database functionality for storing:
//! - PSS events and match data
//! - Application configuration and settings
//! - OBS connection configurations
//! - Flag management data
//! - User preferences and session data

pub mod connection;
pub mod maintenance;
pub mod migrations;
pub mod operations;
pub mod migration_strategy;
pub mod models;

pub use connection::DatabaseConnection;
pub use maintenance::{DatabaseMaintenance, MaintenanceConfig, MaintenanceStatistics, MaintenanceResult, MaintenanceNeeded, DatabaseInfo};
pub use operations::UiSettingsOperations;
pub use migration_strategy::{MigrationStrategy, MigrationResult, HybridSettingsProvider};

/// Database error type
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Initialization error: {0}")]
    Initialization(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Transaction error: {0}")]
    Transaction(String),
    #[error("Schema version error: {0}")]
    SchemaVersion(String),
}

/// Schema version information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchemaVersion {
    pub version: u32,
    pub applied_at: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

impl SchemaVersion {
    pub fn new(version: u32, description: String) -> Self {
        Self {
            version,
            applied_at: chrono::Utc::now(),
            description,
        }
    }

    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            version: row.get(1)?,
            applied_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                .with_timezone(&chrono::Utc),
            description: row.get(3)?,
        })
    }
}

/// Database result type
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Current schema version - increment when adding new migrations
pub const CURRENT_SCHEMA_VERSION: u32 = 10;

/// Database file name
pub const DATABASE_FILE: &str = "restrike_vta.db"; 