use std::sync::Arc;
use crate::database::{
    DatabaseConnection,
    UiSettingsOperations,
};
use crate::database::migrations::MigrationManager;
use crate::types::AppResult;

/// Database plugin for managing SQLite database operations
pub struct DatabasePlugin {
    connection: Arc<DatabaseConnection>,
    migration_manager: MigrationManager,
}

impl DatabasePlugin {
    /// Create a new database plugin
    pub fn new() -> AppResult<Self> {
        log::info!("🔧 Initializing database plugin...");
        
        let connection = Arc::new(DatabaseConnection::new()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Database initialization failed: {}", e)))?);
        
        let migration_manager = MigrationManager::new();
        
        // Run migrations
        {
            let conn = connection.get_connection()
                .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
            
            migration_manager.migrate(&conn)
                .map_err(|e| crate::types::AppError::ConfigError(format!("Database migration failed: {}", e)))?;
        }
        
        log::info!("✅ Database plugin initialized successfully");
        
        Ok(Self {
            connection,
            migration_manager,
        })
    }
    
    /// Get database connection
    pub fn get_connection(&self) -> &Arc<DatabaseConnection> {
        &self.connection
    }
    
    /// Check if database is accessible
    pub fn is_accessible(&self) -> bool {
        self.connection.is_accessible()
    }
    
    /// Get database file size
    pub fn get_file_size(&self) -> AppResult<u64> {
        self.connection.get_file_size()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database file size: {}", e)))
    }
    
    /// Initialize UI settings in the database
    pub fn initialize_ui_settings(&self) -> AppResult<()> {
        let mut conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        UiSettingsOperations::initialize_ui_settings(&mut *conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize UI settings: {}", e)))
    }
    
    /// Get a UI setting value
    pub fn get_ui_setting(&self, key_name: &str) -> AppResult<Option<String>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        UiSettingsOperations::get_ui_setting(&*conn, key_name)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get UI setting: {}", e)))
    }
    
    /// Set a UI setting value
    pub fn set_ui_setting(&self, key_name: &str, value: &str, changed_by: &str, change_reason: Option<&str>) -> AppResult<()> {
        let mut conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        UiSettingsOperations::set_ui_setting(&mut *conn, key_name, value, changed_by, change_reason)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to set UI setting: {}", e)))
    }
    
    /// Get all UI settings
    pub fn get_all_ui_settings(&self) -> AppResult<Vec<(String, String)>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        UiSettingsOperations::get_all_ui_settings(&*conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get all UI settings: {}", e)))
    }
}

/// Database statistics for UI settings
pub struct DatabaseStatistics {
    pub ui_settings_count: i64,
    pub file_size: u64,
}

/// Initialize the database plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Initializing database plugin...");
    Ok(())
} 