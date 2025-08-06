//! Security audit logging module for reStrike VTA
//! 
//! Provides comprehensive audit logging for all security-related operations
//! including configuration access, authentication events, and system changes.

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::security::SecurityResult;
use crate::database::DatabaseConnection;

/// Audit action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    // Authentication actions
    SessionCreate,
    SessionDestroy,
    AuthenticationSuccess,
    AuthenticationFailure,
    
    // Configuration actions
    ConfigRead,
    ConfigCreate,
    ConfigUpdate,
    ConfigDelete,
    
    // System actions
    EncryptionKeyRotation,
    DatabaseMigration,
    SecuritySettingsChange,
    
    // Access control actions
    AccessGranted,
    AccessDenied,
    PrivilegeEscalation,
    
    // Security events
    SuspiciousActivity,
    SecurityViolation,
    IntrusionAttempt,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SessionCreate => "session_create",
            Self::SessionDestroy => "session_destroy",
            Self::AuthenticationSuccess => "auth_success",
            Self::AuthenticationFailure => "auth_failure",
            Self::ConfigRead => "config_read",
            Self::ConfigCreate => "config_create",
            Self::ConfigUpdate => "config_update",
            Self::ConfigDelete => "config_delete",
            Self::EncryptionKeyRotation => "key_rotation",
            Self::DatabaseMigration => "db_migration",
            Self::SecuritySettingsChange => "security_change",
            Self::AccessGranted => "access_granted",
            Self::AccessDenied => "access_denied",
            Self::PrivilegeEscalation => "privilege_escalation",
            Self::SuspiciousActivity => "suspicious_activity",
            Self::SecurityViolation => "security_violation",
            Self::IntrusionAttempt => "intrusion_attempt",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "session_create" => Some(Self::SessionCreate),
            "session_destroy" => Some(Self::SessionDestroy),
            "auth_success" => Some(Self::AuthenticationSuccess),
            "auth_failure" => Some(Self::AuthenticationFailure),
            "config_read" => Some(Self::ConfigRead),
            "config_create" => Some(Self::ConfigCreate),
            "config_update" => Some(Self::ConfigUpdate),
            "config_delete" => Some(Self::ConfigDelete),
            "key_rotation" => Some(Self::EncryptionKeyRotation),
            "db_migration" => Some(Self::DatabaseMigration),
            "security_change" => Some(Self::SecuritySettingsChange),
            "access_granted" => Some(Self::AccessGranted),
            "access_denied" => Some(Self::AccessDenied),
            "privilege_escalation" => Some(Self::PrivilegeEscalation),
            "suspicious_activity" => Some(Self::SuspiciousActivity),
            "security_violation" => Some(Self::SecurityViolation),
            "intrusion_attempt" => Some(Self::IntrusionAttempt),
            _ => None,
        }
    }
    
    pub fn severity_level(&self) -> SeverityLevel {
        match self {
            Self::SessionCreate | Self::SessionDestroy | Self::AuthenticationSuccess | Self::ConfigRead => SeverityLevel::Info,
            Self::ConfigCreate | Self::ConfigUpdate | Self::ConfigDelete | Self::AccessGranted => SeverityLevel::Low,
            Self::EncryptionKeyRotation | Self::DatabaseMigration | Self::SecuritySettingsChange | Self::AccessDenied => SeverityLevel::Medium,
            Self::AuthenticationFailure | Self::PrivilegeEscalation | Self::SuspiciousActivity => SeverityLevel::High,
            Self::SecurityViolation | Self::IntrusionAttempt => SeverityLevel::Critical,
        }
    }
}

/// Audit severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SeverityLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SeverityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "info" => Some(Self::Info),
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Option<i64>,
    pub config_key: Option<String>,
    pub action: AuditAction,
    pub user_context: String,
    pub source_ip: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub severity: SeverityLevel,
    pub session_id: Option<String>,
}

impl AuditEntry {
    pub fn new(
        action: AuditAction,
        user_context: String,
        details: Option<String>,
        success: bool,
        error_message: Option<String>,
    ) -> Self {
        let severity = action.severity_level();
        
        Self {
            id: None,
            config_key: None,
            action,
            user_context,
            source_ip: None,
            timestamp: Utc::now(),
            details,
            success,
            error_message,
            severity,
            session_id: None,
        }
    }
    
    pub fn with_config_key(mut self, config_key: String) -> Self {
        self.config_key = Some(config_key);
        self
    }
    
    pub fn with_source_ip(mut self, source_ip: String) -> Self {
        self.source_ip = Some(source_ip);
        self
    }
    
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Security audit logger
pub struct SecurityAudit {
    database: Arc<DatabaseConnection>,
}

impl SecurityAudit {
    /// Create a new security audit logger
    pub fn new(database: Arc<DatabaseConnection>) -> SecurityResult<Self> {
        Ok(Self { database })
    }
    
    /// Log a security event
    pub async fn log_security_event(
        &self,
        action: AuditAction,
        user_context: &str,
        details: &str,
        success: bool,
        error_message: Option<&str>,
    ) -> SecurityResult<i64> {
        let entry = AuditEntry::new(
            action,
            user_context.to_string(),
            Some(details.to_string()),
            success,
            error_message.map(|s| s.to_string()),
        );
        
        self.log_entry(&entry).await
    }
    
    /// Log a configuration access event
    pub async fn log_config_access(
        &self,
        config_key: &str,
        action: AuditAction,
        user_context: &str,
        details: &str,
        success: bool,
        error_message: Option<&str>,
    ) -> SecurityResult<i64> {
        let entry = AuditEntry::new(
            action,
            user_context.to_string(),
            Some(details.to_string()),
            success,
            error_message.map(|s| s.to_string()),
        ).with_config_key(config_key.to_string());
        
        self.log_entry(&entry).await
    }
    
    /// Log an audit entry
    pub async fn log_entry(&self, entry: &AuditEntry) -> SecurityResult<i64> {
        let conn = self.database.get_connection().await?;
        
        let _row_id = conn.execute(
            "INSERT INTO config_audit 
            (config_key, action, user_context, source_ip, timestamp, details, success, error_message)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                entry.config_key,
                entry.action.as_str(),
                entry.user_context,
                entry.source_ip,
                entry.timestamp.to_rfc3339(),
                entry.details,
                entry.success,
                entry.error_message,
            ],
        )?;
        
        // Log high-severity events to application log as well
        if entry.severity >= SeverityLevel::High {
            log::warn!(
                "Security event: {} by {} - {} (success: {})",
                entry.action.as_str(),
                entry.user_context,
                entry.details.as_deref().unwrap_or("No details"),
                entry.success
            );
            
            if let Some(error) = &entry.error_message {
                log::error!("Security error: {}", error);
            }
        }
        
        // Log critical events with additional details
        if entry.severity == SeverityLevel::Critical {
            log::error!(
                "CRITICAL SECURITY EVENT: {} by {} from IP {} - {}",
                entry.action.as_str(),
                entry.user_context,
                entry.source_ip.as_deref().unwrap_or("unknown"),
                entry.details.as_deref().unwrap_or("No details")
            );
        }
        
        Ok(conn.last_insert_rowid())
    }
    
    /// Get audit entries for a specific configuration key
    pub async fn get_config_audit_history(
        &self,
        config_key: &str,
        limit: Option<i64>,
    ) -> SecurityResult<Vec<AuditEntry>> {
        let conn = self.database.get_connection().await?;
        
        let query = match limit {
            Some(_) => "SELECT id, config_key, action, user_context, source_ip, timestamp, details, success, error_message 
                       FROM config_audit WHERE config_key = ? ORDER BY timestamp DESC LIMIT ?",
            None => "SELECT id, config_key, action, user_context, source_ip, timestamp, details, success, error_message 
                    FROM config_audit WHERE config_key = ? ORDER BY timestamp DESC",
        };
        
        let mut stmt = conn.prepare(query)?;
        let rows = match limit {
            Some(l) => stmt.query_map(params![config_key, l], Self::audit_entry_from_row)?,
            None => stmt.query_map(params![config_key], Self::audit_entry_from_row)?,
        };
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        
        Ok(entries)
    }
    
    /// Get audit entries by user context
    pub async fn get_user_audit_history(
        &self,
        user_context: &str,
        limit: Option<i64>,
    ) -> SecurityResult<Vec<AuditEntry>> {
        let conn = self.database.get_connection().await?;
        
        let query = match limit {
            Some(_) => "SELECT id, config_key, action, user_context, source_ip, timestamp, details, success, error_message 
                       FROM config_audit WHERE user_context = ? ORDER BY timestamp DESC LIMIT ?",
            None => "SELECT id, config_key, action, user_context, source_ip, timestamp, details, success, error_message 
                    FROM config_audit WHERE user_context = ? ORDER BY timestamp DESC",
        };
        
        let mut stmt = conn.prepare(query)?;
        let rows = match limit {
            Some(l) => stmt.query_map(params![user_context, l], Self::audit_entry_from_row)?,
            None => stmt.query_map(params![user_context], Self::audit_entry_from_row)?,
        };
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        
        Ok(entries)
    }
    
    /// Get audit entries by action type
    pub async fn get_action_audit_history(
        &self,
        action: AuditAction,
        limit: Option<i64>,
    ) -> SecurityResult<Vec<AuditEntry>> {
        let conn = self.database.get_connection().await?;
        
        let query = match limit {
            Some(_) => "SELECT id, config_key, action, user_context, source_ip, timestamp, details, success, error_message 
                       FROM config_audit WHERE action = ? ORDER BY timestamp DESC LIMIT ?",
            None => "SELECT id, config_key, action, user_context, source_ip, timestamp, details, success, error_message 
                    FROM config_audit WHERE action = ? ORDER BY timestamp DESC",
        };
        
        let mut stmt = conn.prepare(query)?;
        let rows = match limit {
            Some(l) => stmt.query_map(params![action.as_str(), l], Self::audit_entry_from_row)?,
            None => stmt.query_map(params![action.as_str()], Self::audit_entry_from_row)?,
        };
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        
        Ok(entries)
    }
    
    /// Get recent security events (high and critical severity)
    pub async fn get_security_events(&self, hours: i64) -> SecurityResult<Vec<AuditEntry>> {
        let conn = self.database.get_connection().await?;
        let since = Utc::now() - chrono::Duration::hours(hours);
        
        let mut stmt = conn.prepare(
            "SELECT id, config_key, action, user_context, source_ip, timestamp, details, success, error_message 
             FROM config_audit 
             WHERE timestamp >= ? AND action IN ('auth_failure', 'privilege_escalation', 'suspicious_activity', 'security_violation', 'intrusion_attempt')
             ORDER BY timestamp DESC"
        )?;
        
        let rows = stmt.query_map(params![since.to_rfc3339()], Self::audit_entry_from_row)?;
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        
        Ok(entries)
    }
    
    /// Get audit statistics
    pub async fn get_audit_statistics(&self, hours: Option<i64>) -> SecurityResult<AuditStatistics> {
        let conn = self.database.get_connection().await?;
        
        let (where_clause, params) = match hours {
            Some(h) => {
                let since = Utc::now() - chrono::Duration::hours(h);
                ("WHERE timestamp >= ?", vec![since.to_rfc3339()])
            }
            None => ("", vec![]),
        };
        
        // Total events
        let total_events: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM config_audit {}", where_clause),
            rusqlite::params_from_iter(&params),
            |row| row.get(0),
        )?;
        
        // Failed events
        let failed_events: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM config_audit {} {}", 
                    where_clause, 
                    if where_clause.is_empty() { "WHERE" } else { "AND" }),
            rusqlite::params_from_iter(params.iter().chain(std::iter::once(&"0".to_string()))),
            |row| row.get(0),
        )?;
        
        // Unique users
        let unique_users: i64 = conn.query_row(
            &format!("SELECT COUNT(DISTINCT user_context) FROM config_audit {}", where_clause),
            rusqlite::params_from_iter(&params),
            |row| row.get(0),
        )?;
        
        // Most active user
        let most_active_user = conn.query_row(
            &format!("SELECT user_context, COUNT(*) as count FROM config_audit {} GROUP BY user_context ORDER BY count DESC LIMIT 1", where_clause),
            rusqlite::params_from_iter(&params),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        ).ok();
        
        Ok(AuditStatistics {
            total_events: total_events as u64,
            failed_events: failed_events as u64,
            unique_users: unique_users as u64,
            most_active_user: most_active_user.map(|(user, count)| (user, count as u64)),
        })
    }
    
    /// Clean up old audit entries
    pub async fn cleanup_old_entries(&self, retention_days: u32) -> SecurityResult<u64> {
        let conn = self.database.get_connection().await?;
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        
        let deleted = conn.execute(
            "DELETE FROM config_audit WHERE timestamp < ?",
            params![cutoff.to_rfc3339()],
        )?;
        
        log::info!("Cleaned up {} old audit entries older than {} days", deleted, retention_days);
        Ok(deleted as u64)
    }
    
    /// Helper function to create AuditEntry from database row
    fn audit_entry_from_row(row: &rusqlite::Row) -> rusqlite::Result<AuditEntry> {
        let action_str: String = row.get(2)?;
        let action = AuditAction::from_str(&action_str)
            .ok_or_else(|| rusqlite::Error::InvalidColumnType(2, "action".to_string(), rusqlite::types::Type::Text))?;
        
        let timestamp_str: String = row.get(5)?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(5, "timestamp".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);
        
        Ok(AuditEntry {
            id: Some(row.get(0)?),
            config_key: row.get(1)?,
            action,
            user_context: row.get(3)?,
            source_ip: row.get(4)?,
            timestamp,
            details: row.get(6)?,
            success: row.get(7)?,
            error_message: row.get(8)?,
            severity: action.severity_level(),
            session_id: None, // Not stored in current schema
        })
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_events: u64,
    pub failed_events: u64,
    pub unique_users: u64,
    pub most_active_user: Option<(String, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::database::DatabaseConnection;
    
    async fn create_test_audit() -> SecurityAudit {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database = Arc::new(DatabaseConnection::new(db_path.to_str().unwrap()).await.unwrap());
        
        SecurityAudit::new(database).unwrap()
    }
    
    #[tokio::test]
    async fn test_audit_logging() {
        let audit = create_test_audit().await;
        
        let entry_id = audit.log_security_event(
            AuditAction::AuthenticationSuccess,
            "test_user",
            "User logged in successfully",
            true,
            None,
        ).await.unwrap();
        
        assert!(entry_id > 0);
    }
    
    #[tokio::test]
    async fn test_config_audit() {
        let audit = create_test_audit().await;
        
        audit.log_config_access(
            "obs.password",
            AuditAction::ConfigUpdate,
            "test_user",
            "Updated OBS password",
            true,
            None,
        ).await.unwrap();
        
        let history = audit.get_config_audit_history("obs.password", Some(10)).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].action, AuditAction::ConfigUpdate);
    }
    
    #[tokio::test]
    async fn test_audit_statistics() {
        let audit = create_test_audit().await;
        
        // Add some test entries
        for i in 0..5 {
            audit.log_security_event(
                AuditAction::ConfigRead,
                &format!("user_{}", i % 2),
                "Test event",
                i < 4, // One failure
                if i >= 4 { Some("Test error") } else { None },
            ).await.unwrap();
        }
        
        let stats = audit.get_audit_statistics(Some(24)).await.unwrap();
        assert_eq!(stats.total_events, 5);
        assert_eq!(stats.failed_events, 1);
        assert_eq!(stats.unique_users, 2);
    }
}