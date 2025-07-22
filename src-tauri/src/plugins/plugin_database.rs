use std::sync::Arc;
use crate::database::{
    DatabaseConnection,
    PssEventOperations, ObsConnectionOperations, AppConfigOperations, FlagMappingOperations,
    PssEvent, ObsConnection, AppConfig, FlagMapping,
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
    
    /// Get database statistics
    pub fn get_statistics(&self) -> AppResult<DatabaseStatistics> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        let pss_events_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pss_events",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        let obs_connections_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM obs_connections",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        let app_configs_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM app_config",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        let flag_mappings_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM flag_mappings",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        Ok(DatabaseStatistics {
            pss_events_count,
            obs_connections_count,
            app_configs_count,
            flag_mappings_count,
            file_size: self.get_file_size()?,
        })
    }
    
    // PSS Events operations
    pub fn insert_pss_event(&self, event: &PssEvent) -> AppResult<i64> {
        PssEventOperations::insert(&self.connection, event)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to insert PSS event: {}", e)))
    }
    
    pub fn get_pss_event(&self, id: i64) -> AppResult<Option<PssEvent>> {
        PssEventOperations::get_by_id(&self.connection, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS event: {}", e)))
    }
    
    pub fn get_pss_events_by_match(&self, match_id: &str) -> AppResult<Vec<PssEvent>> {
        PssEventOperations::get_by_match_id(&self.connection, match_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS events by match: {}", e)))
    }
    
    pub fn get_recent_pss_events(&self, limit: i64) -> AppResult<Vec<PssEvent>> {
        PssEventOperations::get_recent(&self.connection, limit)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get recent PSS events: {}", e)))
    }
    
    pub fn delete_pss_event(&self, id: i64) -> AppResult<bool> {
        PssEventOperations::delete_by_id(&self.connection, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to delete PSS event: {}", e)))
    }
    
    pub fn clear_pss_events(&self) -> AppResult<u64> {
        PssEventOperations::clear_all(&self.connection)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to clear PSS events: {}", e)))
    }
    
    // OBS Connections operations
    pub fn insert_obs_connection(&self, connection: &ObsConnection) -> AppResult<i64> {
        ObsConnectionOperations::insert(&self.connection, connection)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to insert OBS connection: {}", e)))
    }
    
    pub fn get_obs_connection(&self, id: i64) -> AppResult<Option<ObsConnection>> {
        ObsConnectionOperations::get_by_id(&self.connection, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get OBS connection: {}", e)))
    }
    
    pub fn get_obs_connection_by_name(&self, name: &str) -> AppResult<Option<ObsConnection>> {
        ObsConnectionOperations::get_by_name(&self.connection, name)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get OBS connection by name: {}", e)))
    }
    
    pub fn get_all_obs_connections(&self) -> AppResult<Vec<ObsConnection>> {
        ObsConnectionOperations::get_all(&self.connection)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get all OBS connections: {}", e)))
    }
    
    pub fn update_obs_connection(&self, connection: &ObsConnection) -> AppResult<bool> {
        ObsConnectionOperations::update(&self.connection, connection)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to update OBS connection: {}", e)))
    }
    
    pub fn delete_obs_connection(&self, id: i64) -> AppResult<bool> {
        ObsConnectionOperations::delete_by_id(&self.connection, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to delete OBS connection: {}", e)))
    }
    
    pub fn set_active_obs_connection(&self, id: i64) -> AppResult<bool> {
        ObsConnectionOperations::set_active(&self.connection, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to set active OBS connection: {}", e)))
    }
    
    // App Config operations
    pub fn upsert_app_config(&self, config: &AppConfig) -> AppResult<i64> {
        AppConfigOperations::upsert(&self.connection, config)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to upsert app config: {}", e)))
    }
    
    pub fn get_app_config(&self, key: &str) -> AppResult<Option<AppConfig>> {
        AppConfigOperations::get_by_key(&self.connection, key)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get app config: {}", e)))
    }
    
    pub fn get_app_configs_by_category(&self, category: &str) -> AppResult<Vec<AppConfig>> {
        AppConfigOperations::get_by_category(&self.connection, category)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get app configs by category: {}", e)))
    }
    
    pub fn get_all_app_configs(&self) -> AppResult<Vec<AppConfig>> {
        AppConfigOperations::get_all(&self.connection)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get all app configs: {}", e)))
    }
    
    pub fn delete_app_config(&self, key: &str) -> AppResult<bool> {
        AppConfigOperations::delete_by_key(&self.connection, key)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to delete app config: {}", e)))
    }
    
    // Flag Mapping operations
    pub fn upsert_flag_mapping(&self, mapping: &FlagMapping) -> AppResult<i64> {
        FlagMappingOperations::upsert(&self.connection, mapping)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to upsert flag mapping: {}", e)))
    }
    
    pub fn get_flag_mapping_by_pss_code(&self, pss_code: &str) -> AppResult<Option<FlagMapping>> {
        FlagMappingOperations::get_by_pss_code(&self.connection, pss_code)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get flag mapping by PSS code: {}", e)))
    }
    
    pub fn get_flag_mapping_by_ioc_code(&self, ioc_code: &str) -> AppResult<Option<FlagMapping>> {
        FlagMappingOperations::get_by_ioc_code(&self.connection, ioc_code)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get flag mapping by IOC code: {}", e)))
    }
    
    pub fn get_all_flag_mappings(&self) -> AppResult<Vec<FlagMapping>> {
        FlagMappingOperations::get_all(&self.connection)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get all flag mappings: {}", e)))
    }
    
    pub fn get_custom_flag_mappings(&self) -> AppResult<Vec<FlagMapping>> {
        FlagMappingOperations::get_custom(&self.connection)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get custom flag mappings: {}", e)))
    }
    
    pub fn delete_flag_mapping(&self, pss_code: &str) -> AppResult<bool> {
        FlagMappingOperations::delete_by_pss_code(&self.connection, pss_code)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to delete flag mapping: {}", e)))
    }
}

/// Database statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseStatistics {
    pub pss_events_count: i64,
    pub obs_connections_count: i64,
    pub app_configs_count: i64,
    pub flag_mappings_count: i64,
    pub file_size: u64,
}

/// Initialize the database plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing database plugin...");
    // Create a temporary instance to test initialization
    let _plugin = DatabasePlugin::new()
        .map_err(|e| format!("Database plugin initialization failed: {}", e))?;
    println!("✅ Database plugin initialized successfully");
    Ok(())
} 