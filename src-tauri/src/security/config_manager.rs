//! Secure configuration manager for reStrike VTA
//!
//! Provides centralized, encrypted storage and retrieval of sensitive configuration data
//! with comprehensive audit logging and access control.

use base64::Engine as _;
use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::database::DatabaseConnection;
use crate::security::audit::{AuditAction, SecurityAudit};
use crate::security::encryption::EncryptedData;
use crate::security::{SecureConfig, SecurityError, SecurityResult};

/// Configuration categories for organizing encrypted data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigCategory {
    ObsCredentials,
    ApiKeys,
    DatabaseConfig,
    NetworkSecrets,
    LicenseInfo,
    UserPreferences,
    SystemConfig,
    EncryptionKeys,
    ControlRoom,
}

impl ConfigCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ObsCredentials => "obs_credentials",
            Self::ApiKeys => "api_keys",
            Self::DatabaseConfig => "database_config",
            Self::NetworkSecrets => "network_secrets",
            Self::LicenseInfo => "license_info",
            Self::UserPreferences => "user_preferences",
            Self::SystemConfig => "system_config",
            Self::EncryptionKeys => "encryption_keys",
            Self::ControlRoom => "control_room",
        }
    }

    pub fn from_key(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ObsCredentials => "OBS Credentials",
            Self::ApiKeys => "API Keys",
            Self::DatabaseConfig => "Database Configuration",
            Self::NetworkSecrets => "Network Secrets",
            Self::LicenseInfo => "License Information",
            Self::UserPreferences => "User Preferences",
            Self::SystemConfig => "System Configuration",
            Self::EncryptionKeys => "Encryption Keys",
            Self::ControlRoom => "Control Room",
        }
    }

    pub fn required_access_level(&self) -> AccessLevel {
        match self {
            Self::ObsCredentials => AccessLevel::Configuration,
            Self::ApiKeys => AccessLevel::Administrator,
            Self::DatabaseConfig => AccessLevel::Administrator,
            Self::NetworkSecrets => AccessLevel::Administrator,
            Self::LicenseInfo => AccessLevel::Administrator,
            Self::UserPreferences => AccessLevel::ReadOnly,
            Self::SystemConfig => AccessLevel::Administrator,
            Self::EncryptionKeys => AccessLevel::Administrator,
            Self::ControlRoom => AccessLevel::Administrator,
        }
    }
}

impl std::str::FromStr for ConfigCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "obs_credentials" => Ok(Self::ObsCredentials),
            "api_keys" => Ok(Self::ApiKeys),
            "database_config" => Ok(Self::DatabaseConfig),
            "network_secrets" => Ok(Self::NetworkSecrets),
            "license_info" => Ok(Self::LicenseInfo),
            "user_preferences" => Ok(Self::UserPreferences),
            "system_config" => Ok(Self::SystemConfig),
            "encryption_keys" => Ok(Self::EncryptionKeys),
            "control_room" => Ok(Self::ControlRoom),
            _ => Err(()),
        }
    }
}

/// Access levels for configuration operations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AccessLevel {
    ReadOnly,
    Configuration,
    Administrator,
}

impl AccessLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Configuration => "configuration",
            Self::Administrator => "administrator",
        }
    }

    pub fn from_key(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

impl std::str::FromStr for AccessLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read_only" => Ok(Self::ReadOnly),
            "configuration" => Ok(Self::Configuration),
            "administrator" => Ok(Self::Administrator),
            _ => Err(()),
        }
    }
}

/// Cached configuration entry
#[derive(Debug, Clone)]
struct CachedConfig {
    value: String,
    cached_at: Instant,
    access_count: u64,
}

/// Security session for access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySession {
    pub session_id: String,
    pub user_context: String,
    pub access_level: AccessLevel,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
}

impl SecuritySession {
    pub fn new(user_context: String, access_level: AccessLevel) -> Self {
        let now = Utc::now();
        let session_id = uuid::Uuid::new_v4().to_string();
        let expires_at = now
            + chrono::Duration::minutes(crate::security::constants::SESSION_TIMEOUT_MINUTES as i64);

        Self {
            session_id,
            user_context,
            access_level,
            created_at: now,
            last_accessed: now,
            expires_at,
            is_active: true,
            source_ip: None,
            user_agent: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn can_access(&self, required_level: &AccessLevel) -> bool {
        self.is_active && !self.is_expired() && &self.access_level >= required_level
    }
}

/// Secure configuration manager
pub struct SecureConfigManager {
    encryption: SecureConfig,
    database: Arc<DatabaseConnection>,
    audit: SecurityAudit,
    cache: Arc<Mutex<HashMap<String, CachedConfig>>>,
    sessions: Arc<Mutex<HashMap<String, SecuritySession>>>,
    cache_ttl: Duration,
}

impl SecureConfigManager {
    pub(crate) async fn ensure_tables(database: &Arc<DatabaseConnection>) -> SecurityResult<()> {
        let conn = database.get_connection().await?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS secure_config (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                config_key TEXT NOT NULL UNIQUE,
                encrypted_value BLOB NOT NULL,
                category TEXT NOT NULL,
                is_sensitive BOOLEAN NOT NULL DEFAULT 1,
                salt BLOB NOT NULL,
                algorithm TEXT NOT NULL DEFAULT 'AES-256-GCM',
                kdf_params TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_accessed TEXT,
                access_count INTEGER DEFAULT 0,
                description TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_secure_config_key ON secure_config(config_key)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_secure_config_category ON secure_config(category)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_secure_config_sensitive ON secure_config(is_sensitive)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS security_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL UNIQUE,
                user_context TEXT NOT NULL,
                access_level TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_accessed TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                source_ip TEXT,
                user_agent TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_security_sessions_id ON security_sessions(session_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_security_sessions_expires ON security_sessions(expires_at)",
            [],
        )?;

        Ok(())
    }

    /// Create a new secure configuration manager
    pub async fn new(
        master_password: String,
        database: Arc<DatabaseConnection>,
    ) -> SecurityResult<Self> {
        Self::ensure_tables(&database).await?;

        let encryption = SecureConfig::new(master_password)?;
        let audit = SecurityAudit::new(database.clone()).await?;

        Ok(Self {
            encryption,
            database,
            audit,
            cache: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl: Duration::from_secs(15 * 60), // 15-minute cache TTL
        })
    }

    /// Create a new security session
    pub async fn create_session(
        &self,
        user_context: String,
        access_level: AccessLevel,
        source_ip: Option<String>,
        user_agent: Option<String>,
    ) -> SecurityResult<SecuritySession> {
        let mut session = SecuritySession::new(user_context.clone(), access_level.clone());
        session.source_ip = source_ip;
        session.user_agent = user_agent;

        // Store session in database - Scope the connection to ensure it's dropped before await
        {
            let conn = self.database.get_connection().await?;
            conn.execute(
                "INSERT INTO security_sessions
                (session_id, user_context, access_level, created_at, last_accessed, expires_at, is_active, source_ip, user_agent)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    session.session_id,
                    session.user_context,
                    session.access_level.as_str(),
                    session.created_at.to_rfc3339(),
                    session.last_accessed.to_rfc3339(),
                    session.expires_at.to_rfc3339(),
                    session.is_active,
                    session.source_ip,
                    session.user_agent,
                ],
            )?;
        }

        // Store in memory cache
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session.session_id.clone(), session.clone());

        self.audit
            .log_security_event(
                AuditAction::SessionCreate,
                &user_context,
                &format!("Created {} session", access_level.as_str()),
                true,
                None,
            )
            .await?;

        Ok(session)
    }

    /// Validate and get session
    pub async fn get_session(&self, session_id: &str) -> SecurityResult<Option<SecuritySession>> {
        // Check memory cache first
        {
            let mut sessions = self.sessions.lock().await;
            if let Some(session) = sessions.get(session_id) {
                if !session.is_expired() {
                    return Ok(Some(session.clone()));
                } else {
                    // Remove expired session from cache
                    sessions.remove(session_id);
                }
            }
        }

        // Check database - Scope connection and statement
        let session_result = {
            let conn = self.database.get_connection().await?;
            let mut stmt = conn.prepare(
                "SELECT session_id, user_context, access_level, created_at, last_accessed, expires_at, is_active, source_ip, user_agent
                 FROM security_sessions WHERE session_id = ? AND is_active = 1"
            )?;

            stmt.query_row(params![session_id], |row| {
                let access_level_str: String = row.get(2)?;
                let access_level = access_level_str.parse::<AccessLevel>().map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        2,
                        "access_level".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;

                Ok(SecuritySession {
                    session_id: row.get(0)?,
                    user_context: row.get(1)?,
                    access_level,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                3,
                                "created_at".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?
                        .with_timezone(&Utc),
                    last_accessed: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                4,
                                "last_accessed".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?
                        .with_timezone(&Utc),
                    expires_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                5,
                                "expires_at".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?
                        .with_timezone(&Utc),
                    is_active: row.get(6)?,
                    source_ip: row.get(7)?,
                    user_agent: row.get(8)?,
                })
            })
        };

        match session_result {
            Ok(session) => {
                if session.is_expired() {
                    // Mark as inactive
                    self.invalidate_session(session_id).await?;
                    Ok(None)
                } else {
                    // Update cache
                    let mut sessions = self.sessions.lock().await;
                    sessions.insert(session.session_id.clone(), session.clone());
                    Ok(Some(session))
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(SecurityError::Database(e)),
        }
    }

    /// Invalidate a session
    pub async fn invalidate_session(&self, session_id: &str) -> SecurityResult<()> {
        // Remove from memory cache
        {
            let mut sessions = self.sessions.lock().await;
            sessions.remove(session_id);
        }

        // Mark as inactive in database
        {
            let conn = self.database.get_connection().await?;
            conn.execute(
                "UPDATE security_sessions SET is_active = 0 WHERE session_id = ?",
                params![session_id],
            )?;
        }

        Ok(())
    }

    /// Set encrypted configuration value
    pub async fn set_config(
        &self,
        session_id: &str,
        key: &str,
        value: &str,
        category: ConfigCategory,
        description: Option<&str>,
    ) -> SecurityResult<()> {
        // Validate session and access
        let session = self.get_session(session_id).await?.ok_or_else(|| {
            SecurityError::Authentication("Invalid or expired session".to_string())
        })?;

        if !session.can_access(&category.required_access_level()) {
            return Err(SecurityError::Authentication(
                "Insufficient access level".to_string(),
            ));
        }

        // Encrypt the value
        let encrypted_data = self.encryption.encrypt_value(value)?;
        let encrypted_json = serde_json::to_string(&encrypted_data)?;
        let kdf_params_json = serde_json::to_string(&encrypted_data.kdf_params)?;

        // Store in database - Scope connection
        {
            let conn = self.database.get_connection().await?;
            let now = Utc::now().to_rfc3339();

            conn.execute(
                "INSERT OR REPLACE INTO secure_config
                (config_key, encrypted_value, category, is_sensitive, salt, algorithm, kdf_params, created_at, updated_at, description)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    key,
                    encrypted_json.as_bytes(),
                    category.as_str(),
                    true, // All values are sensitive by default
                    base64::engine::general_purpose::STANDARD.decode(&encrypted_data.salt)
                        .map_err(|e| SecurityError::Decryption(format!("Failed to decode salt: {e}")))?,
                    encrypted_data.algorithm,
                    kdf_params_json,
                    now,
                    now,
                    description,
                ],
            )?;
        }

        // Update cache
        {
            let mut cache = self.cache.lock().await;
            cache.insert(
                key.to_string(),
                CachedConfig {
                    value: value.to_string(),
                    cached_at: Instant::now(),
                    access_count: 0,
                },
            );
        }

        // Log audit event
        self.audit
            .log_config_access(
                key,
                AuditAction::ConfigUpdate,
                &session.user_context,
                &format!("Updated {} configuration", category.as_str()),
                true,
                None,
            )
            .await?;

        Ok(())
    }

    /// Get encrypted configuration value
    pub async fn get_config(&self, session_id: &str, key: &str) -> SecurityResult<Option<String>> {
        // Validate session
        let session = self.get_session(session_id).await?.ok_or_else(|| {
            SecurityError::Authentication("Invalid or expired session".to_string())
        })?;

        // Check cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(cached) = cache.get_mut(key) {
                if cached.cached_at.elapsed() < self.cache_ttl {
                    cached.access_count += 1;

                    // Log access
                    self.audit
                        .log_config_access(
                            key,
                            AuditAction::ConfigRead,
                            &session.user_context,
                            "Retrieved from cache",
                            true,
                            None,
                        )
                        .await?;

                    return Ok(Some(cached.value.clone()));
                } else {
                    // Remove expired cache entry
                    cache.remove(key);
                }
            }
        }

        // Get from database - Scope connection and statement
        let result = {
            let conn = self.database.get_connection().await?;
            let mut stmt = conn.prepare(
                "SELECT encrypted_value, category, salt, algorithm, kdf_params, access_count
                 FROM secure_config WHERE config_key = ?",
            )?;

            stmt.query_row(params![key], |row| {
                let encrypted_value_bytes: Vec<u8> = row.get(0)?;
                let category_str: String = row.get(1)?;
                let _salt: Vec<u8> = row.get(2)?;
                let _algorithm: String = row.get(3)?;
                let _kdf_params: String = row.get(4)?;
                let access_count: i64 = row.get(5)?;

                Ok((encrypted_value_bytes, category_str, access_count))
            })
        };

        match result {
            Ok((encrypted_value_bytes, category_str, access_count)) => {
                let encrypted_json = String::from_utf8(encrypted_value_bytes).map_err(|e| {
                    SecurityError::Decryption(format!("Invalid UTF-8 in encrypted data: {e}"))
                })?;

                let encrypted_data: EncryptedData = serde_json::from_str(&encrypted_json)?;

                // Check access level for category
                if let Ok(category) = category_str.parse::<ConfigCategory>() {
                    if !session.can_access(&category.required_access_level()) {
                        return Err(SecurityError::Authentication(
                            "Insufficient access level".to_string(),
                        ));
                    }
                }

                // Decrypt the value
                let decrypted_value = self.encryption.decrypt_value(&encrypted_data)?;

                // Update access count and last accessed time - Scope connection
                {
                    let conn = self.database.get_connection().await?;
                    conn.execute(
                        "UPDATE secure_config SET access_count = ?, last_accessed = ? WHERE config_key = ?",
                        params![access_count + 1, Utc::now().to_rfc3339(), key],
                    )?;
                }

                // Update cache
                {
                    let mut cache = self.cache.lock().await;
                    cache.insert(
                        key.to_string(),
                        CachedConfig {
                            value: decrypted_value.clone(),
                            cached_at: Instant::now(),
                            access_count: (access_count + 1) as u64,
                        },
                    );
                }

                // Log audit event
                self.audit
                    .log_config_access(
                        key,
                        AuditAction::ConfigRead,
                        &session.user_context,
                        "Retrieved from database",
                        true,
                        None,
                    )
                    .await?;

                Ok(Some(decrypted_value))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(SecurityError::Database(e)),
        }
    }

    /// Delete configuration value
    pub async fn delete_config(&self, session_id: &str, key: &str) -> SecurityResult<bool> {
        // Validate session
        let session = self.get_session(session_id).await?.ok_or_else(|| {
            SecurityError::Authentication("Invalid or expired session".to_string())
        })?;

        // Get category and delete - Scope connection
        let delete_result = {
            let conn = self.database.get_connection().await?;
            let category_result: Result<String, _> = conn.query_row(
                "SELECT category FROM secure_config WHERE config_key = ?",
                params![key],
                |row| row.get(0),
            );

            match category_result {
                Ok(category_str) => {
                    if let Ok(category) = category_str.parse::<ConfigCategory>() {
                        if !session.can_access(&category.required_access_level()) {
                            return Err(SecurityError::Authentication(
                                "Insufficient access level".to_string(),
                            ));
                        }
                    }

                    // Delete from database
                    let changes = conn.execute(
                        "DELETE FROM secure_config WHERE config_key = ?",
                        params![key],
                    )?;
                    Ok(Some(changes > 0))
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(SecurityError::Database(e)),
            }
        };

        match delete_result {
            Ok(Some(deleted)) => {
                if deleted {
                    // Remove from cache
                    {
                        let mut cache = self.cache.lock().await;
                        cache.remove(key);
                    }

                    // Log audit event
                    self.audit
                        .log_config_access(
                            key,
                            AuditAction::ConfigDelete,
                            &session.user_context,
                            "Configuration deleted",
                            true,
                            None,
                        )
                        .await?;
                }
                Ok(deleted)
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// List configuration keys by category
    pub async fn list_config_keys(
        &self,
        session_id: &str,
        category: Option<ConfigCategory>,
    ) -> SecurityResult<Vec<String>> {
        // Validate session
        let session = self.get_session(session_id).await?.ok_or_else(|| {
            SecurityError::Authentication("Invalid or expired session".to_string())
        })?;

        // Scope connection
        let keys = {
            let conn = self.database.get_connection().await?;
            let (query, params): (&str, Vec<String>) = match category {
                Some(cat) => {
                    // Check access level
                    if !session.can_access(&cat.required_access_level()) {
                        return Err(SecurityError::Authentication(
                            "Insufficient access level".to_string(),
                        ));
                    }
                    (
                        "SELECT config_key FROM secure_config WHERE category = ?",
                        vec![cat.as_str().to_string()],
                    )
                }
                None => ("SELECT config_key FROM secure_config", vec![]),
            };

            let mut stmt = conn.prepare(query)?;
            let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
                row.get::<_, String>(0)
            })?;

            let mut keys = Vec::new();
            for row in rows {
                keys.push(row?);
            }
            keys
        };

        Ok(keys)
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().await;
        let total_entries = cache.len();
        let expired_entries = cache
            .values()
            .filter(|entry| entry.cached_at.elapsed() > self.cache_ttl)
            .count();
        (total_entries, expired_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_manager() -> SecureConfigManager {
        // Use default database connection for testing
        let database = Arc::new(DatabaseConnection::new_in_memory().unwrap());

        SecureConfigManager::new("test_password".to_string(), database)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_session_management() {
        let manager = create_test_manager().await;

        let session = manager
            .create_session(
                "test_user".to_string(),
                AccessLevel::Configuration,
                Some("127.0.0.1".to_string()),
                Some("test_agent".to_string()),
            )
            .await
            .unwrap();

        assert!(session.can_access(&AccessLevel::ReadOnly));
        assert!(session.can_access(&AccessLevel::Configuration));
        assert!(!session.can_access(&AccessLevel::Administrator));

        let retrieved = manager.get_session(&session.session_id).await.unwrap();
        assert!(retrieved.is_some());

        manager
            .invalidate_session(&session.session_id)
            .await
            .unwrap();
        let after_invalidation = manager.get_session(&session.session_id).await.unwrap();
        assert!(after_invalidation.is_none());
    }

    #[tokio::test]
    async fn test_config_storage() {
        let manager = create_test_manager().await;

        let session = manager
            .create_session(
                "test_user".to_string(),
                AccessLevel::Administrator,
                None,
                None,
            )
            .await
            .unwrap();

        // Set configuration
        manager
            .set_config(
                &session.session_id,
                "obs.password",
                "secret_password",
                ConfigCategory::ObsCredentials,
                Some("OBS WebSocket password"),
            )
            .await
            .unwrap();

        // Get configuration
        let retrieved = manager
            .get_config(&session.session_id, "obs.password")
            .await
            .unwrap();
        assert_eq!(retrieved, Some("secret_password".to_string()));

        // Delete configuration
        let deleted = manager
            .delete_config(&session.session_id, "obs.password")
            .await
            .unwrap();
        assert!(deleted);

        // Verify deletion
        let after_delete = manager
            .get_config(&session.session_id, "obs.password")
            .await
            .unwrap();
        assert!(after_delete.is_none());
    }
}
