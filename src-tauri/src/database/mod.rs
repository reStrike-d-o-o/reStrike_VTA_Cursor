//! Database module for reStrike VTA
//! 
//! This module provides SQLite database functionality for storing:
//! - PSS events and match data
//! - Application configuration and settings
//! - OBS connection configurations
//! - Flag management data
//! - User preferences and session data

pub mod connection;
pub mod migrations;
pub mod models;
pub mod operations;
pub mod error;

pub use connection::DatabaseConnection;
pub use error::DatabaseError;
pub use models::*;
pub use operations::*;

/// Database result type
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Current database schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Database file name
pub const DATABASE_FILE: &str = "restrike_vta.db"; 