use std::sync::Arc;
use crate::database::{
    DatabaseConnection,
    PssEventOperations, ObsConnectionOperations, AppConfigOperations, FlagMappingOperations,
    DatabaseMaintenanceOperations, SettingsOperations,
    PssEvent, ObsConnection, AppConfig, FlagMapping,
    PssEventStatistics, DatabaseIntegrityReport, DatabaseSizeInfo, DatabasePerformanceStats,
};
use crate::database::migrations::MigrationManager;
use crate::types::AppResult;
use std::path::PathBuf;

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
    
    // ===== BACKUP AND RESTORE OPERATIONS =====
    
    /// Create a backup of the database
    pub fn create_backup(&self, backup_name: Option<&str>) -> AppResult<PathBuf> {
        self.connection.create_backup(backup_name)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to create backup: {}", e)))
    }
    
    /// Restore database from backup
    pub fn restore_from_backup(&self, backup_path: &PathBuf) -> AppResult<()> {
        self.connection.restore_from_backup(backup_path)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to restore from backup: {}", e)))
    }
    
    /// List available backups
    pub fn list_backups(&self) -> AppResult<Vec<PathBuf>> {
        self.connection.list_backups()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to list backups: {}", e)))
    }
    
    /// Clean up old backups
    pub fn cleanup_old_backups(&self, keep_count: usize) -> AppResult<usize> {
        self.connection.cleanup_old_backups(keep_count)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to cleanup backups: {}", e)))
    }
    
    /// Create compressed backup
    pub fn create_compressed_backup(&self, backup_path: &PathBuf) -> AppResult<()> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        DatabaseMaintenanceOperations::create_compressed_backup(&conn, backup_path)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to create compressed backup: {}", e)))
    }
    
    /// Verify backup integrity
    pub fn verify_backup(&self, backup_path: &PathBuf) -> AppResult<bool> {
        DatabaseMaintenanceOperations::verify_backup(backup_path)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to verify backup: {}", e)))
    }
    
    // ===== MAINTENANCE OPERATIONS =====
    
    /// Optimize the database
    pub fn optimize_database(&self) -> AppResult<()> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        DatabaseMaintenanceOperations::optimize(&mut conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to optimize database: {}", e)))
    }
    
    /// Check database integrity
    pub fn check_integrity(&self) -> AppResult<DatabaseIntegrityReport> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        DatabaseMaintenanceOperations::check_integrity(&conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to check integrity: {}", e)))
    }
    
    /// Get database size information
    pub fn get_size_info(&self) -> AppResult<DatabaseSizeInfo> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        DatabaseMaintenanceOperations::get_size_info(&conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get size info: {}", e)))
    }
    
    /// Get database performance statistics
    pub fn get_performance_stats(&self) -> AppResult<DatabasePerformanceStats> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        DatabaseMaintenanceOperations::get_performance_stats(&conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get performance stats: {}", e)))
    }
    
    /// Export database to SQL file
    pub fn export_to_sql(&self, output_path: &PathBuf) -> AppResult<()> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        DatabaseMaintenanceOperations::export_to_sql(&conn, output_path)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to export to SQL: {}", e)))
    }
    
    /// Import database from SQL file
    pub fn import_from_sql(&self, sql_file_path: &PathBuf) -> AppResult<()> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        DatabaseMaintenanceOperations::import_from_sql(&mut conn, sql_file_path)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to import from SQL: {}", e)))
    }
    
    // ===== PSS EVENTS OPERATIONS =====
    
    pub fn insert_pss_event(&self, event: &PssEvent) -> AppResult<i64> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        PssEventOperations::insert(&mut conn, event)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to insert PSS event: {}", e)))
    }
    
    pub fn get_pss_event(&self, id: i64) -> AppResult<Option<PssEvent>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        PssEventOperations::get_by_id(&conn, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS event: {}", e)))
    }
    
    pub fn get_pss_events_by_match(&self, match_id: &str) -> AppResult<Vec<PssEvent>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        PssEventOperations::get_by_match_id(&conn, match_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS events by match: {}", e)))
    }
    
    pub fn get_recent_pss_events(&self, limit: i64) -> AppResult<Vec<PssEvent>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        PssEventOperations::get_recent(&conn, limit)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get recent PSS events: {}", e)))
    }
    
    pub fn delete_pss_event(&self, id: i64) -> AppResult<bool> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        PssEventOperations::delete_by_id(&mut conn, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to delete PSS event: {}", e)))
    }
    
    pub fn clear_pss_events(&self) -> AppResult<u64> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        PssEventOperations::clear_all(&mut conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to clear PSS events: {}", e)))
    }
    
    pub fn get_pss_event_statistics(&self) -> AppResult<PssEventStatistics> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        PssEventOperations::get_statistics(&conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS event statistics: {}", e)))
    }
    
    // ===== OBS CONNECTIONS OPERATIONS =====
    
    pub fn insert_obs_connection(&self, connection: &ObsConnection) -> AppResult<i64> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        ObsConnectionOperations::insert(&mut conn, connection)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to insert OBS connection: {}", e)))
    }
    
    pub fn get_obs_connection(&self, id: i64) -> AppResult<Option<ObsConnection>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        ObsConnectionOperations::get_by_id(&conn, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get OBS connection: {}", e)))
    }
    
    pub fn get_obs_connection_by_name(&self, name: &str) -> AppResult<Option<ObsConnection>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        ObsConnectionOperations::get_by_name(&conn, name)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get OBS connection by name: {}", e)))
    }
    
    pub fn get_all_obs_connections(&self) -> AppResult<Vec<ObsConnection>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        ObsConnectionOperations::get_all(&conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get all OBS connections: {}", e)))
    }
    
    pub fn update_obs_connection(&self, connection: &ObsConnection) -> AppResult<bool> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        ObsConnectionOperations::update(&mut conn, connection)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to update OBS connection: {}", e)))
    }
    
    pub fn delete_obs_connection(&self, id: i64) -> AppResult<bool> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        ObsConnectionOperations::delete_by_id(&mut conn, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to delete OBS connection: {}", e)))
    }
    
    pub fn set_active_obs_connection(&self, id: i64) -> AppResult<bool> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        ObsConnectionOperations::set_active(&mut conn, id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to set active OBS connection: {}", e)))
    }
    
    // ===== APP CONFIG OPERATIONS =====
    
    pub fn upsert_app_config(&self, config: &AppConfig) -> AppResult<i64> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        AppConfigOperations::upsert(&mut conn, config)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to upsert app config: {}", e)))
    }
    
    pub fn get_app_config(&self, key: &str) -> AppResult<Option<AppConfig>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        AppConfigOperations::get_by_key(&conn, key)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get app config: {}", e)))
    }
    
    pub fn get_app_configs_by_category(&self, category: &str) -> AppResult<Vec<AppConfig>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        AppConfigOperations::get_by_category(&conn, category)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get app configs by category: {}", e)))
    }
    
    pub fn get_all_app_configs(&self) -> AppResult<Vec<AppConfig>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        AppConfigOperations::get_all(&conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get all app configs: {}", e)))
    }
    
    pub fn delete_app_config(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        AppConfigOperations::delete_by_key(&mut conn, key)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to delete app config: {}", e)))
    }
    
    // ===== FLAG MAPPING OPERATIONS =====
    
    pub fn upsert_flag_mapping(&self, mapping: &FlagMapping) -> AppResult<i64> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        FlagMappingOperations::upsert(&mut conn, mapping)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to upsert flag mapping: {}", e)))
    }
    
    pub fn get_flag_mapping_by_pss_code(&self, pss_code: &str) -> AppResult<Option<FlagMapping>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        FlagMappingOperations::get_by_pss_code(&conn, pss_code)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get flag mapping by PSS code: {}", e)))
    }
    
    pub fn get_flag_mapping_by_ioc_code(&self, ioc_code: &str) -> AppResult<Option<FlagMapping>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        FlagMappingOperations::get_by_ioc_code(&conn, ioc_code)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get flag mapping by IOC code: {}", e)))
    }
    
    pub fn get_all_flag_mappings(&self) -> AppResult<Vec<FlagMapping>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        FlagMappingOperations::get_all(&conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get all flag mappings: {}", e)))
    }
    
    pub fn get_custom_flag_mappings(&self) -> AppResult<Vec<FlagMapping>> {
        let conn = self.connection.get_connection()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        FlagMappingOperations::get_custom(&conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get custom flag mappings: {}", e)))
    }
    
    pub fn delete_flag_mapping(&self, pss_code: &str) -> AppResult<bool> {
        let mut conn = self.connection.get_connection_mut()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        FlagMappingOperations::delete_by_pss_code(&mut conn, pss_code)
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