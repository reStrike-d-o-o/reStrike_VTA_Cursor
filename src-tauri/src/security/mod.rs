//! Security module for reStrike VTA
//! 
//! This module provides comprehensive security functionality including:
//! - SHA256-based encryption for sensitive configuration data
//! - Secure credential storage and retrieval
//! - Audit logging for security operations
//! - Key management and rotation capabilities

pub mod encryption;
pub mod config_manager;
pub mod audit;
pub mod key_manager;
pub mod migration;

pub use encryption::{SecureConfig, SecurityError};
pub use config_manager::{SecureConfigManager, ConfigCategory, AccessLevel};
pub use audit::{SecurityAudit, AuditAction, AuditEntry};
pub use key_manager::{KeyManager, KeyRotationConfig};
pub use migration::{ConfigMigrationTool, MigrationConfig, MigrationStats};

/// Security result type
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Security configuration constants
pub mod constants {
    /// PBKDF2 iteration count for key derivation
    pub const PBKDF2_ITERATIONS: u32 = 100_000;
    
    /// Salt length in bytes
    pub const SALT_LENGTH: usize = 32;
    
    /// Key length in bytes
    pub const KEY_LENGTH: usize = 32;
    
    /// Session timeout in minutes
    pub const SESSION_TIMEOUT_MINUTES: u64 = 30;
    
    /// Maximum failed authentication attempts
    pub const MAX_AUTH_ATTEMPTS: u32 = 5;
    
    /// Audit log retention days
    pub const AUDIT_RETENTION_DAYS: u32 = 90;
}