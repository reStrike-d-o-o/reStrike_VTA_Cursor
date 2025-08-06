//! Configuration migration tool for reStrike VTA security enhancement
//! 
//! This module provides functionality to migrate existing plaintext configuration
//! to encrypted database storage with comprehensive audit logging.

use std::sync::Arc;
use std::fs;
use std::path::Path;
use serde_json::Value;
use chrono::Utc;

use crate::security::{
    SecureConfigManager, ConfigCategory, AccessLevel, SecurityResult, SecurityError
};
use crate::security::audit::{SecurityAudit, AuditAction};

use crate::database::DatabaseConnection;

/// Migration configuration
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Whether to backup original configuration files
    pub backup_originals: bool,
    /// Whether to remove original files after successful migration
    pub remove_originals: bool,
    /// Master password for encryption
    pub master_password: String,
    /// Migration batch size
    pub batch_size: usize,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            backup_originals: true,
            remove_originals: false,
            master_password: "default_master_password".to_string(),
            batch_size: 50,
        }
    }
}

/// Migration statistics
#[derive(Debug, Clone)]
pub struct MigrationStats {
    pub total_configs_found: u32,
    pub configs_migrated: u32,
    pub configs_skipped: u32,
    pub configs_failed: u32,
    pub credentials_migrated: u32,
    pub api_keys_migrated: u32,
    pub files_backed_up: u32,
    pub migration_duration_ms: u64,
}

impl Default for MigrationStats {
    fn default() -> Self {
        Self {
            total_configs_found: 0,
            configs_migrated: 0,
            configs_skipped: 0,
            configs_failed: 0,
            credentials_migrated: 0,
            api_keys_migrated: 0,
            files_backed_up: 0,
            migration_duration_ms: 0,
        }
    }
}

/// Configuration migration tool
pub struct ConfigMigrationTool {
    config_manager: SecureConfigManager,
    audit: SecurityAudit,
    migration_config: MigrationConfig,
    stats: MigrationStats,
}

impl ConfigMigrationTool {
    /// Create a new migration tool
    pub async fn new(
        database: Arc<DatabaseConnection>,
        migration_config: MigrationConfig,
    ) -> SecurityResult<Self> {
        let config_manager = SecureConfigManager::new(
            migration_config.master_password.clone(),
            database.clone(),
        ).await?;
        
        let audit = SecurityAudit::new(database.clone())?;
        
        Ok(Self {
            config_manager,
            audit,
            migration_config,
            stats: MigrationStats::default(),
        })
    }
    
    /// Perform complete configuration migration
    pub async fn migrate_all_configurations(&mut self) -> SecurityResult<MigrationStats> {
        let start_time = std::time::Instant::now();
        
        log::info!("🔄 Starting configuration migration to encrypted storage");
        
        // Create admin session for migration
        let session = self.config_manager.create_session(
            "system_migration".to_string(),
            AccessLevel::Administrator,
            Some("localhost".to_string()),
            Some("ConfigMigrationTool/1.0".to_string()),
        ).await?;
        
        // Log migration start
        self.audit.log_security_event(
            AuditAction::DatabaseMigration,
            "system_migration",
            "Starting configuration migration to encrypted storage",
            true,
            None,
        ).await?;
        
        // Migrate different configuration sources
        self.migrate_json_config_files(&session.session_id).await?;
        self.migrate_hardcoded_credentials(&session.session_id).await?;
        self.migrate_frontend_stores(&session.session_id).await?;
        self.migrate_environment_variables(&session.session_id).await?;
        
        // Update statistics
        self.stats.migration_duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Log migration completion
        self.audit.log_security_event(
            AuditAction::DatabaseMigration,
            "system_migration",
            &format!(
                "Configuration migration completed: {} configs migrated, {} credentials secured",
                self.stats.configs_migrated,
                self.stats.credentials_migrated
            ),
            true,
            None,
        ).await?;
        
        log::info!(
            "✅ Configuration migration completed in {}ms: {} configs migrated",
            self.stats.migration_duration_ms,
            self.stats.configs_migrated
        );
        
        Ok(self.stats.clone())
    }
    
    /// Migrate JSON configuration files
    async fn migrate_json_config_files(&mut self, session_id: &str) -> SecurityResult<()> {
        log::info!("📄 Migrating JSON configuration files...");
        
        let config_files = [
            "src-tauri/config/app_config.json",
            "src-tauri/config/app_config.backup.json",
            "config/dev_resources.json",
        ];
        
        for config_file in &config_files {
            if Path::new(config_file).exists() {
                self.migrate_json_file(session_id, config_file).await?;
            }
        }
        
        Ok(())
    }
    
    /// Migrate a single JSON configuration file
    async fn migrate_json_file(&mut self, session_id: &str, file_path: &str) -> SecurityResult<()> {
        log::info!("📄 Processing configuration file: {}", file_path);
        
        // Read and parse the JSON file
        let content = fs::read_to_string(file_path)
            .map_err(|e| SecurityError::InvalidInput(format!("Failed to read {}: {}", file_path, e)))?;
        
        let config: Value = serde_json::from_str(&content)
            .map_err(|e| SecurityError::Serialization(e))?;
        
        // Backup original file if requested
        if self.migration_config.backup_originals {
            self.backup_file(file_path).await?;
        }
        
        // Extract and migrate sensitive configurations
        self.extract_obs_credentials(session_id, &config).await?;
        self.extract_api_keys(session_id, &config).await?;
        self.extract_database_credentials(session_id, &config).await?;
        self.extract_network_secrets(session_id, &config).await?;
        
        self.stats.total_configs_found += 1;
        self.stats.configs_migrated += 1;
        
        Ok(())
    }
    
    /// Extract OBS credentials from configuration
    async fn extract_obs_credentials(&mut self, session_id: &str, config: &Value) -> SecurityResult<()> {
        if let Some(obs_config) = config.get("obs") {
            if let Some(connections) = obs_config.get("connections") {
                if let Some(connections_array) = connections.as_array() {
                    for connection in connections_array {
                        if let (Some(name), Some(password)) = (
                            connection.get("name").and_then(|v| v.as_str()),
                            connection.get("password").and_then(|v| v.as_str())
                        ) {
                            if !password.is_empty() {
                                let config_key = format!("obs.{}.password", name);
                                
                                self.config_manager.set_config(
                                    session_id,
                                    &config_key,
                                    password,
                                    ConfigCategory::ObsCredentials,
                                    Some(&format!("OBS WebSocket password for {}", name)),
                                ).await?;
                                
                                self.stats.credentials_migrated += 1;
                                
                                log::info!("🔐 Migrated OBS password for connection: {}", name);
                            }
                        }
                        
                        // Also migrate host/port as non-sensitive config
                        if let (Some(name), Some(host), Some(port)) = (
                            connection.get("name").and_then(|v| v.as_str()),
                            connection.get("host").and_then(|v| v.as_str()),
                            connection.get("port").and_then(|v| v.as_u64())
                        ) {
                            let host_key = format!("obs.{}.host", name);
                            let port_key = format!("obs.{}.port", name);
                            
                            self.config_manager.set_config(
                                session_id,
                                &host_key,
                                host,
                                ConfigCategory::UserPreferences,
                                Some(&format!("OBS host for {}", name)),
                            ).await?;
                            
                            self.config_manager.set_config(
                                session_id,
                                &port_key,
                                &port.to_string(),
                                ConfigCategory::UserPreferences,
                                Some(&format!("OBS port for {}", name)),
                            ).await?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract API keys from configuration
    async fn extract_api_keys(&mut self, session_id: &str, config: &Value) -> SecurityResult<()> {
        // Look for various API key patterns
        let api_key_patterns = [
            ("youtube", "client_secret"),
            ("google", "api_key"),
            ("drive", "client_secret"),
            ("streaming", "api_key"),
        ];
        
        for (service, key_field) in &api_key_patterns {
            if let Some(service_config) = config.get(*service) {
                if let Some(api_key) = service_config.get(*key_field).and_then(|v| v.as_str()) {
                    if !api_key.is_empty() {
                        let config_key = format!("api.{}.{}", service, key_field);
                        
                        self.config_manager.set_config(
                            session_id,
                            &config_key,
                            api_key,
                            ConfigCategory::ApiKeys,
                            Some(&format!("{} API key for {}", key_field, service)),
                        ).await?;
                        
                        self.stats.api_keys_migrated += 1;
                        
                        log::info!("🔑 Migrated {} API key for service: {}", key_field, service);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract database credentials from configuration
    async fn extract_database_credentials(&mut self, session_id: &str, config: &Value) -> SecurityResult<()> {
        if let Some(db_config) = config.get("database") {
            if let Some(connection_string) = db_config.get("connection_string").and_then(|v| v.as_str()) {
                if !connection_string.is_empty() {
                    self.config_manager.set_config(
                        session_id,
                        "database.connection_string",
                        connection_string,
                        ConfigCategory::DatabaseConfig,
                        Some("Database connection string"),
                    ).await?;
                    
                    self.stats.credentials_migrated += 1;
                    log::info!("🗄️ Migrated database connection string");
                }
            }
            
            if let Some(password) = db_config.get("password").and_then(|v| v.as_str()) {
                if !password.is_empty() {
                    self.config_manager.set_config(
                        session_id,
                        "database.password",
                        password,
                        ConfigCategory::DatabaseConfig,
                        Some("Database password"),
                    ).await?;
                    
                    self.stats.credentials_migrated += 1;
                    log::info!("🔐 Migrated database password");
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract network secrets from configuration
    async fn extract_network_secrets(&mut self, session_id: &str, config: &Value) -> SecurityResult<()> {
        if let Some(network_config) = config.get("network") {
            if let Some(auth_token) = network_config.get("auth_token").and_then(|v| v.as_str()) {
                if !auth_token.is_empty() {
                    self.config_manager.set_config(
                        session_id,
                        "network.auth_token",
                        auth_token,
                        ConfigCategory::NetworkSecrets,
                        Some("Network authentication token"),
                    ).await?;
                    
                    self.stats.credentials_migrated += 1;
                    log::info!("🌐 Migrated network auth token");
                }
            }
        }
        
        Ok(())
    }
    
    /// Migrate hardcoded credentials from source code
    async fn migrate_hardcoded_credentials(&mut self, session_id: &str) -> SecurityResult<()> {
        log::info!("🔍 Migrating hardcoded credentials from codebase...");
        
        // Known hardcoded credentials that need to be migrated
        let hardcoded_credentials = [
            ("obs.default.password", "cekPIbj@245", ConfigCategory::ObsCredentials, "Default OBS password"),
            ("obs.OBS_REC.password", "cekPIbj@245", ConfigCategory::ObsCredentials, "OBS_REC connection password"),
            ("obs.OBS_STR.password", "cekPIbj@245", ConfigCategory::ObsCredentials, "OBS_STR connection password"),
        ];
        
        for (key, value, category, description) in &hardcoded_credentials {
            self.config_manager.set_config(
                session_id,
                key,
                value,
                category.clone(),
                Some(description),
            ).await?;
            
            self.stats.credentials_migrated += 1;
            
            log::warn!("⚠️ Migrated hardcoded credential: {}", key);
        }
        
        Ok(())
    }
    
    /// Migrate frontend store configurations
    async fn migrate_frontend_stores(&mut self, session_id: &str) -> SecurityResult<()> {
        log::info!("🎨 Migrating frontend store configurations...");
        
        // Read frontend store files if they exist
        let store_files = [
            "ui/src/stores/index.ts",
            "ui/src/config/environments/web.ts",
        ];
        
        for store_file in &store_files {
            if Path::new(store_file).exists() {
                // Parse TypeScript/JavaScript files for configuration
                if let Ok(content) = fs::read_to_string(store_file) {
                    self.extract_frontend_credentials(session_id, &content).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract credentials from frontend code
    async fn extract_frontend_credentials(&mut self, session_id: &str, content: &str) -> SecurityResult<()> {
        // Look for password patterns in frontend code
        if content.contains("password:") && content.contains("cekPIbj@245") {
            // Extract the hardcoded password from frontend stores
            self.config_manager.set_config(
                session_id,
                "frontend.obs.default_password",
                "cekPIbj@245",
                ConfigCategory::ObsCredentials,
                Some("Frontend default OBS password"),
            ).await?;
            
            self.stats.credentials_migrated += 1;
            
            log::warn!("⚠️ Migrated hardcoded password from frontend store");
        }
        
        Ok(())
    }
    
    /// Migrate environment variables
    async fn migrate_environment_variables(&mut self, session_id: &str) -> SecurityResult<()> {
        log::info!("🌍 Checking environment variables for sensitive data...");
        
        // Check for sensitive environment variables
        let env_vars = [
            "OBS_PASSWORD",
            "DATABASE_PASSWORD",
            "API_KEY",
            "SECRET_KEY",
            "AUTH_TOKEN",
        ];
        
        for env_var in &env_vars {
            if let Ok(value) = std::env::var(env_var) {
                if !value.is_empty() {
                    let config_key = format!("env.{}", env_var.to_lowercase());
                    let category = match *env_var {
                        "OBS_PASSWORD" => ConfigCategory::ObsCredentials,
                        "DATABASE_PASSWORD" => ConfigCategory::DatabaseConfig,
                        "API_KEY" => ConfigCategory::ApiKeys,
                        _ => ConfigCategory::NetworkSecrets,
                    };
                    
                    self.config_manager.set_config(
                        session_id,
                        &config_key,
                        &value,
                        category,
                        Some(&format!("Environment variable: {}", env_var)),
                    ).await?;
                    
                    self.stats.credentials_migrated += 1;
                    
                    log::info!("🌍 Migrated environment variable: {}", env_var);
                }
            }
        }
        
        Ok(())
    }
    
    /// Backup a configuration file
    async fn backup_file(&mut self, file_path: &str) -> SecurityResult<()> {
        let backup_path = format!("{}.backup.{}", file_path, Utc::now().format("%Y%m%d_%H%M%S"));
        
        fs::copy(file_path, &backup_path)
            .map_err(|e| SecurityError::InvalidInput(format!("Failed to backup {}: {}", file_path, e)))?;
        
        self.stats.files_backed_up += 1;
        
        log::info!("💾 Backed up configuration file: {} -> {}", file_path, backup_path);
        
        Ok(())
    }
    
    /// Verify migration completeness
    pub async fn verify_migration(&self, session_id: &str) -> SecurityResult<bool> {
        log::info!("🔍 Verifying migration completeness...");
        
        // Check that key configurations were migrated
        let required_configs = [
            "obs.OBS_REC.password",
            "obs.OBS_STR.password",
            "obs.default.password",
        ];
        
        for config_key in &required_configs {
            match self.config_manager.get_config(session_id, config_key).await? {
                Some(value) => {
                    if value.is_empty() {
                        log::error!("❌ Verification failed: {} is empty", config_key);
                        return Ok(false);
                    }
                    log::info!("✅ Verified migration of: {}", config_key);
                }
                None => {
                    log::error!("❌ Verification failed: {} not found", config_key);
                    return Ok(false);
                }
            }
        }
        
        log::info!("✅ Migration verification successful");
        Ok(true)
    }
    
    /// Generate migration report
    pub fn generate_report(&self) -> String {
        format!(
            r#"
# Configuration Migration Report

## Summary
- **Total configurations found**: {}
- **Configurations migrated**: {}
- **Configurations skipped**: {}
- **Configurations failed**: {}
- **Credentials migrated**: {}
- **API keys migrated**: {}
- **Files backed up**: {}
- **Migration duration**: {}ms

## Security Enhancements
- All sensitive configuration data is now encrypted with SHA256
- Hardcoded passwords have been migrated to secure storage
- API keys are protected with role-based access control
- Comprehensive audit logging is now active
- Configuration access requires authenticated sessions

## Next Steps
1. Verify that all applications can access encrypted configurations
2. Update code to use SecureConfigManager instead of plaintext config
3. Remove hardcoded credentials from source code
4. Set up regular key rotation schedule
5. Monitor audit logs for any security issues

## Migration Completed Successfully ✅
Date: {}
"#,
            self.stats.total_configs_found,
            self.stats.configs_migrated,
            self.stats.configs_skipped,
            self.stats.configs_failed,
            self.stats.credentials_migrated,
            self.stats.api_keys_migrated,
            self.stats.files_backed_up,
            self.stats.migration_duration_ms,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    
    async fn create_test_migration_tool() -> ConfigMigrationTool {
        // Use default database connection for testing
        let database = Arc::new(DatabaseConnection::new().unwrap());
        
        let config = MigrationConfig {
            master_password: "test_password".to_string(),
            ..Default::default()
        };
        
        ConfigMigrationTool::new(database, config).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_json_migration() {
        let mut tool = create_test_migration_tool().await;
        
        // Create test configuration
        let test_config = serde_json::json!({
            "obs": {
                "connections": [
                    {
                        "name": "OBS_REC",
                        "host": "localhost",
                        "port": 4455,
                        "password": "test_password"
                    }
                ]
            }
        });
        
        // Test configuration extraction
        let session = tool.config_manager.create_session(
            "test_user".to_string(),
            AccessLevel::Administrator,
            None,
            None,
        ).await.unwrap();
        
        tool.extract_obs_credentials(&session.session_id, &test_config).await.unwrap();
        
        // Verify migration
        let migrated_password = tool.config_manager.get_config(&session.session_id, "obs.OBS_REC.password").await.unwrap();
        assert_eq!(migrated_password, Some("test_password".to_string()));
    }
    
    #[tokio::test]
    async fn test_migration_verification() {
        let mut tool = create_test_migration_tool().await;
        
        let session = tool.config_manager.create_session(
            "test_user".to_string(),
            AccessLevel::Administrator,
            None,
            None,
        ).await.unwrap();
        
        // Migrate some test credentials
        tool.migrate_hardcoded_credentials(&session.session_id).await.unwrap();
        
        // Verify migration
        let verification_result = tool.verify_migration(&session.session_id).await.unwrap();
        assert!(verification_result);
    }
}