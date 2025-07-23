//! Database module for reStrike VTA
//! 
//! This module provides SQLite database functionality for storing:
//! - PSS events and match data
//! - Application configuration and settings
//! - OBS connection configurations
//! - Flag management data
//! - User preferences and session data

pub mod error;
pub mod connection;
pub mod models;
pub mod migrations;
pub mod operations;

pub use error::DatabaseError;
pub use connection::{DatabaseConnection, DatabaseStatistics};
pub use models::{
    PssEvent, ObsConnection, AppConfig, FlagMapping, SchemaVersion,
    SettingsCategory, SettingsKey, SettingsValue, SettingsHistory,
};
pub use operations::{
    PssEventOperations, ObsConnectionOperations, AppConfigOperations, FlagMappingOperations,
    SettingsOperations, SettingsStatistics, DatabaseMaintenanceOperations,
    PssEventStatistics, DatabaseIntegrityReport, DatabaseSizeInfo, DatabasePerformanceStats,
};

/// Database result type
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Current database schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 2;

/// Database file name
pub const DATABASE_FILE: &str = "restrike_vta.db"; 