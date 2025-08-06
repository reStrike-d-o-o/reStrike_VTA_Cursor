//! Security-related Tauri commands for reStrike VTA
//! 
//! Provides secure configuration management, migration tools, and security operations
//! accessible from the frontend with proper authentication and audit logging.

use std::sync::Arc;
use tauri::State;
use serde::{Serialize, Deserialize};

use crate::core::app::App;
use crate::security::{
    SecurityError, ConfigMigrationTool, MigrationConfig, MigrationStats,
    SecureConfigManager, ConfigCategory, AccessLevel
};

/// Tauri error wrapper for security operations
#[derive(Debug, Serialize)]
pub struct TauriSecurityError {
    message: String,
    error_type: String,
}

impl From<SecurityError> for TauriSecurityError {
    fn from(error: SecurityError) -> Self {
        Self {
            message: error.to_string(),
            error_type: match error {
                SecurityError::Authentication(_) => "authentication",
                SecurityError::Encryption(_) => "encryption",
                SecurityError::Decryption(_) => "decryption",
                SecurityError::InvalidInput(_) => "invalid_input",
                SecurityError::Database(_) => "database",
                SecurityError::DatabaseConnection(_) => "database_connection",
                SecurityError::Serialization(_) => "serialization",
                SecurityError::KeyDerivation(_) => "key_derivation",
                SecurityError::RandomGeneration(_) => "random_generation",
                SecurityError::KeyNotFound(_) => "key_not_found",
            }.to_string(),
        }
    }
}

/// Migration request parameters
#[derive(Debug, Deserialize)]
pub struct MigrationRequest {
    pub master_password: String,
    pub backup_originals: Option<bool>,
    pub remove_originals: Option<bool>,
}

/// Security session request
#[derive(Debug, Deserialize)]
pub struct SessionRequest {
    pub user_context: String,
    pub access_level: String, // "read_only", "configuration", "administrator"
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// Configuration request
#[derive(Debug, Deserialize)]
pub struct ConfigRequest {
    pub session_id: String,
    pub key: String,
    pub value: Option<String>,
    pub category: String,
    pub description: Option<String>,
}

/// Configuration response
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub key: String,
    pub value: Option<String>,
    pub category: String,
    pub last_accessed: Option<String>,
    pub access_count: u64,
}

/// Security session response
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub user_context: String,
    pub access_level: String,
    pub created_at: String,
    pub expires_at: String,
    pub is_active: bool,
}

/// Audit entry response
#[derive(Debug, Serialize)]
pub struct AuditResponse {
    pub id: Option<i64>,
    pub config_key: Option<String>,
    pub action: String,
    pub user_context: String,
    pub timestamp: String,
    pub details: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Start configuration migration
#[tauri::command]
pub async fn security_migrate_configurations(
    request: MigrationRequest,
    app: State<'_, Arc<App>>,
) -> Result<MigrationStats, TauriSecurityError> {
    log::info!("🔄 Starting security configuration migration");
    
    // Get database connection from app
    let database = app.database_plugin().get_database_connection();
    
    // Create migration configuration
    let migration_config = MigrationConfig {
        backup_originals: request.backup_originals.unwrap_or(true),
        remove_originals: request.remove_originals.unwrap_or(false),
        master_password: request.master_password,
        batch_size: 50,
    };
    
    // Create migration tool
    let mut migration_tool = ConfigMigrationTool::new(database, migration_config)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Perform migration
    let stats = migration_tool.migrate_all_configurations()
        .await
        .map_err(TauriSecurityError::from)?;
    
    log::info!("✅ Security configuration migration completed successfully");
    Ok(stats)
}

/// Verify migration completeness
#[tauri::command]
pub async fn security_verify_migration(
    session_id: String,
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<bool, TauriSecurityError> {
    log::info!("🔍 Verifying security configuration migration");
    
    let database = app.database_plugin().get_database_connection();
    
    // Create secure config manager
    let _config_manager = SecureConfigManager::new(master_password, database.clone())
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Create migration tool for verification
    let migration_config = MigrationConfig::default();
    let migration_tool = ConfigMigrationTool::new(database, migration_config)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Verify migration
    let is_verified = migration_tool.verify_migration(&session_id)
        .await
        .map_err(TauriSecurityError::from)?;
    
    log::info!("📊 Migration verification result: {}", is_verified);
    Ok(is_verified)
}

/// Create a new security session
#[tauri::command]
pub async fn security_create_session(
    request: SessionRequest,
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<SessionResponse, TauriSecurityError> {
    log::info!("🔐 Creating security session for user: {}", request.user_context);
    
    let database = app.database_plugin().get_database_connection();
    
    // Parse access level
    let access_level = match request.access_level.as_str() {
        "read_only" => AccessLevel::ReadOnly,
        "configuration" => AccessLevel::Configuration,
        "administrator" => AccessLevel::Administrator,
        _ => return Err(TauriSecurityError {
            message: "Invalid access level".to_string(),
            error_type: "invalid_input".to_string(),
        }),
    };
    
    // Create secure config manager
    let config_manager = SecureConfigManager::new(master_password, database)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Create session
    let session = config_manager.create_session(
        request.user_context,
        access_level,
        request.source_ip,
        request.user_agent,
    ).await.map_err(TauriSecurityError::from)?;
    
    let response = SessionResponse {
        session_id: session.session_id,
        user_context: session.user_context,
        access_level: session.access_level.as_str().to_string(),
        created_at: session.created_at.to_rfc3339(),
        expires_at: session.expires_at.to_rfc3339(),
        is_active: session.is_active,
    };
    
    log::info!("✅ Security session created successfully");
    Ok(response)
}

/// Get encrypted configuration value
#[tauri::command]
pub async fn security_get_config(
    config_request: ConfigRequest,
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<Option<ConfigResponse>, TauriSecurityError> {
    let database = app.database_plugin().get_database_connection();
    
    // Create secure config manager
    let config_manager = SecureConfigManager::new(master_password, database)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Get configuration value
    let value = config_manager.get_config(&config_request.session_id, &config_request.key)
        .await
        .map_err(TauriSecurityError::from)?;
    
    match value {
        Some(val) => Ok(Some(ConfigResponse {
            key: config_request.key,
            value: Some(val),
            category: config_request.category,
            last_accessed: Some(chrono::Utc::now().to_rfc3339()),
            access_count: 1, // This would come from the database in a real implementation
        })),
        None => Ok(None),
    }
}

/// Set encrypted configuration value
#[tauri::command]
pub async fn security_set_config(
    config_request: ConfigRequest,
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<bool, TauriSecurityError> {
    let database = app.database_plugin().get_database_connection();
    
    // Parse category
    let category = ConfigCategory::from_str(&config_request.category)
        .ok_or_else(|| TauriSecurityError {
            message: "Invalid configuration category".to_string(),
            error_type: "invalid_input".to_string(),
        })?;
    
    // Create secure config manager
    let config_manager = SecureConfigManager::new(master_password, database)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Set configuration value
    if let Some(value) = config_request.value {
        config_manager.set_config(
            &config_request.session_id,
            &config_request.key,
            &value,
            category,
            config_request.description.as_deref(),
        ).await.map_err(TauriSecurityError::from)?;
        
        Ok(true)
    } else {
        Err(TauriSecurityError {
            message: "Configuration value is required".to_string(),
            error_type: "invalid_input".to_string(),
        })
    }
}

/// Delete encrypted configuration value
#[tauri::command]
pub async fn security_delete_config(
    session_id: String,
    config_key: String,
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<bool, TauriSecurityError> {
    let database = app.database_plugin().get_database_connection();
    
    // Create secure config manager
    let config_manager = SecureConfigManager::new(master_password, database)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Delete configuration value
    let deleted = config_manager.delete_config(&session_id, &config_key)
        .await
        .map_err(TauriSecurityError::from)?;
    
    Ok(deleted)
}

/// List configuration keys by category
#[tauri::command]
pub async fn security_list_config_keys(
    session_id: String,
    category: Option<String>,
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<Vec<String>, TauriSecurityError> {
    let database = app.database_plugin().get_database_connection();
    
    // Parse category if provided
    let config_category = if let Some(cat_str) = category {
        Some(ConfigCategory::from_str(&cat_str)
            .ok_or_else(|| TauriSecurityError {
                message: "Invalid configuration category".to_string(),
                error_type: "invalid_input".to_string(),
            })?)
    } else {
        None
    };
    
    // Create secure config manager
    let config_manager = SecureConfigManager::new(master_password, database)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // List configuration keys
    let keys = config_manager.list_config_keys(&session_id, config_category)
        .await
        .map_err(TauriSecurityError::from)?;
    
    Ok(keys)
}

/// Invalidate a security session
#[tauri::command]
pub async fn security_invalidate_session(
    session_id: String,
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<bool, TauriSecurityError> {
    let database = app.database_plugin().get_database_connection();
    
    // Create secure config manager
    let config_manager = SecureConfigManager::new(master_password, database)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Invalidate session
    config_manager.invalidate_session(&session_id)
        .await
        .map_err(TauriSecurityError::from)?;
    
    Ok(true)
}

/// Get security audit history
#[tauri::command]
pub async fn security_get_audit_history(
    config_key: Option<String>,
    user_context: Option<String>,
    limit: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<Vec<AuditResponse>, TauriSecurityError> {
    let database = app.database_plugin().get_database_connection();
    
    // Create security audit
    let audit = crate::security::SecurityAudit::new(database)
        .map_err(TauriSecurityError::from)?;
    
    // Get audit history
    let entries = if let Some(key) = config_key {
        audit.get_config_audit_history(&key, limit)
            .await
            .map_err(TauriSecurityError::from)?
    } else if let Some(user) = user_context {
        audit.get_user_audit_history(&user, limit)
            .await
            .map_err(TauriSecurityError::from)?
    } else {
        // Get recent security events
        audit.get_security_events(24) // Last 24 hours
            .await
            .map_err(TauriSecurityError::from)?
    };
    
    // Convert to response format
    let responses: Vec<AuditResponse> = entries.into_iter().map(|entry| {
        AuditResponse {
            id: entry.id,
            config_key: entry.config_key,
            action: entry.action.as_str().to_string(),
            user_context: entry.user_context,
            timestamp: entry.timestamp.to_rfc3339(),
            details: entry.details,
            success: entry.success,
            error_message: entry.error_message,
        }
    }).collect();
    
    Ok(responses)
}

/// Clear configuration cache
#[tauri::command]
pub async fn security_clear_cache(
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<bool, TauriSecurityError> {
    let database = app.database_plugin().get_database_connection();
    
    // Create secure config manager
    let config_manager = SecureConfigManager::new(master_password, database)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Clear cache
    config_manager.clear_cache().await;
    
    log::info!("🧹 Security configuration cache cleared");
    Ok(true)
}

/// Get cache statistics
#[tauri::command]
pub async fn security_get_cache_stats(
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<(usize, usize), TauriSecurityError> {
    let database = app.database_plugin().get_database_connection();
    
    // Create secure config manager
    let config_manager = SecureConfigManager::new(master_password, database)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Get cache statistics
    let stats = config_manager.get_cache_stats().await;
    
    Ok(stats)
}

/// Test security system
#[tauri::command]
pub async fn security_test_system(
    master_password: String,
    app: State<'_, Arc<App>>,
) -> Result<bool, TauriSecurityError> {
    log::info!("🧪 Testing security system...");
    
    let database = app.database_plugin().get_database_connection();
    
    // Create secure config manager
    let config_manager = SecureConfigManager::new(master_password.clone(), database)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Create test session
    let session = config_manager.create_session(
        "test_user".to_string(),
        AccessLevel::Administrator,
        Some("127.0.0.1".to_string()),
        Some("SecurityTest/1.0".to_string()),
    ).await.map_err(TauriSecurityError::from)?;
    
    // Test encryption/decryption
    let test_key = "security.test.value";
    let test_value = "test_secret_data_123";
    
    // Set test configuration
    config_manager.set_config(
        &session.session_id,
        test_key,
        test_value,
        ConfigCategory::SystemConfig,
        Some("Security system test"),
    ).await.map_err(TauriSecurityError::from)?;
    
    // Get test configuration
    let retrieved_value = config_manager.get_config(&session.session_id, test_key)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Verify encryption/decryption worked
    let test_passed = retrieved_value == Some(test_value.to_string());
    
    // Clean up test data
    config_manager.delete_config(&session.session_id, test_key)
        .await
        .map_err(TauriSecurityError::from)?;
    
    // Invalidate test session
    config_manager.invalidate_session(&session.session_id)
        .await
        .map_err(TauriSecurityError::from)?;
    
    log::info!("🧪 Security system test result: {}", if test_passed { "PASSED" } else { "FAILED" });
    Ok(test_passed)
}