use crate::types::AppResult;
use crate::database::{
    DatabaseConnection,
    UiSettingsOperations,
    MigrationStrategy,
    MigrationResult,
    HybridSettingsProvider,
};
use crate::config::manager::ConfigManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::Path;

/// Database plugin for managing UI settings and migrations
pub struct DatabasePlugin {
    connection: Arc<DatabaseConnection>,
    migration_strategy: MigrationStrategy,
    hybrid_provider: Arc<Mutex<HybridSettingsProvider>>,
}

impl DatabasePlugin {
    pub fn new() -> AppResult<Self> {
        let connection = Arc::new(DatabaseConnection::new()?);
        
        // Initialize config manager with default config directory
        let config_dir = Path::new("config");
        let config_manager = ConfigManager::new(config_dir)?;
        
        let migration_strategy = MigrationStrategy::new(config_manager.clone());
        let hybrid_provider = Arc::new(Mutex::new(HybridSettingsProvider::new(config_manager.clone())));

        Ok(Self {
            connection,
            migration_strategy,
            hybrid_provider,
        })
    }

    /// Initialize UI settings in database
    pub async fn initialize_ui_settings(&self) -> AppResult<()> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        UiSettingsOperations::initialize_ui_settings(&mut *conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize UI settings: {}", e)))
    }

    /// Get UI setting from database
    pub async fn get_ui_setting(&self, key: &str) -> AppResult<Option<String>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        UiSettingsOperations::get_ui_setting(&*conn, key)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get UI setting: {}", e)))
    }

    /// Set UI setting in database
    pub async fn set_ui_setting(&self, key: &str, value: &str, changed_by: &str, change_reason: Option<&str>) -> AppResult<()> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        UiSettingsOperations::set_ui_setting(&mut *conn, key, value, changed_by, change_reason)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to set UI setting: {}", e)))
    }

    /// Get all UI settings from database
    pub async fn get_all_ui_settings(&self) -> AppResult<std::collections::HashMap<String, String>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        let settings_vec = UiSettingsOperations::get_all_ui_settings(&*conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get all UI settings: {}", e)))?;
        
        // Convert Vec<(String, String)> to HashMap<String, String>
        let settings_map: std::collections::HashMap<String, String> = settings_vec.into_iter().collect();
        Ok(settings_map)
    }

    /// Check if database is accessible
    pub async fn is_accessible(&self) -> bool {
        self.connection.is_accessible().await
    }

    /// Get database file size
    pub fn get_file_size(&self) -> AppResult<u64> {
        self.connection.get_file_size()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database file size: {}", e)))
    }

    /// Migrate JSON settings to database
    pub async fn migrate_json_to_database(&self) -> AppResult<MigrationResult> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        self.migration_strategy.migrate_json_to_database(&mut *conn).await
    }

    /// Create JSON backup
    pub async fn create_json_backup(&self) -> AppResult<String> {
        self.migration_strategy.create_json_backup().await
    }

    /// Restore from JSON backup
    pub async fn restore_from_json_backup(&self, backup_path: &str) -> AppResult<()> {
        self.migration_strategy.restore_from_json_backup(backup_path).await
    }

    /// Get migration status
    pub async fn get_migration_status(&self) -> AppResult<MigrationStatus> {
        let settings_count = self.get_all_ui_settings().await.map(|s| s.len()).unwrap_or(0);
        
        // For now, use default values since we don't have a simple get_setting method
        // These could be stored in the database or config file in the future
        let database_enabled = true; // Default to enabled
        let json_fallback_enabled = true; // Default to enabled
        let migration_completed = settings_count > 0; // Assume completed if we have settings
        let last_migration = Some(chrono::Utc::now().to_rfc3339()); // Use current time
        
        Ok(MigrationStatus {
            database_enabled,
            json_fallback_enabled,
            migration_completed,
            last_migration,
            settings_count,
        })
    }

    /// Set database mode
    pub async fn set_database_mode(&self, enabled: bool) -> AppResult<()> {
        let mut provider = self.hybrid_provider.lock().await;
        provider.set_database_mode(enabled);
        Ok(())
    }

    /// Get setting with fallback
    pub async fn get_setting_with_fallback(&self, key: &str) -> AppResult<Option<String>> {
        let provider = self.hybrid_provider.lock().await;
        provider.get_setting(key).await
    }

    /// Set setting (database only)
    pub async fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        let provider = self.hybrid_provider.lock().await;
        provider.set_setting(key, value).await
    }

    /// Get database connection for direct access
    pub async fn get_connection(&self) -> AppResult<tokio::sync::MutexGuard<'_, rusqlite::Connection>> {
        self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))
    }
}

/// Migration status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MigrationStatus {
    pub database_enabled: bool,
    pub json_fallback_enabled: bool,
    pub migration_completed: bool,
    pub last_migration: Option<String>,
    pub settings_count: usize,
}

/// Database statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseStatistics {
    pub ui_settings_count: i64,
    pub file_size: u64,
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Initializing database plugin");
    Ok(())
}