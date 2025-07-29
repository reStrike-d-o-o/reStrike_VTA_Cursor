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

        let plugin = Self {
            connection,
            migration_strategy,
            hybrid_provider,
        };

        // Run database migrations automatically in a separate task
        let connection_clone = plugin.connection.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_migrations_internal(connection_clone).await {
                log::error!("Failed to run database migrations: {}", e);
            }
        });

        Ok(plugin)
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

    /// Get database file path
    pub fn get_database_path(&self) -> AppResult<String> {
        crate::database::connection::DatabaseConnection::get_database_path()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database path: {}", e)))
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

    /// Get database connection for direct operations
    pub async fn get_connection(&self) -> AppResult<tokio::sync::MutexGuard<'_, rusqlite::Connection>> {
        self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> AppResult<()> {
        Self::run_migrations_internal(self.connection.clone()).await
    }

    // PSS and UDP Subsystem Methods

    /// Get all network interfaces
    pub async fn get_network_interfaces(&self) -> AppResult<Vec<crate::database::models::NetworkInterface>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_network_interfaces(&*conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get network interfaces: {}", e)))
    }

    /// Get recommended network interface
    pub async fn get_recommended_interface(&self) -> AppResult<Option<crate::database::models::NetworkInterface>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_recommended_interface(&*conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get recommended interface: {}", e)))
    }

    /// Add or update network interface
    pub async fn upsert_network_interface(&self, interface: &crate::database::models::NetworkInterface) -> AppResult<i64> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::upsert_network_interface(&mut *conn, interface)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to upsert network interface: {}", e)))
    }

    /// Get all UDP server configurations
    pub async fn get_udp_server_configs(&self) -> AppResult<Vec<crate::database::models::UdpServerConfig>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_udp_server_configs(&*conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get UDP server configs: {}", e)))
    }

    /// Get UDP server configuration by ID
    pub async fn get_udp_server_config(&self, config_id: i64) -> AppResult<Option<crate::database::models::UdpServerConfig>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_udp_server_config(&*conn, config_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get UDP server config: {}", e)))
    }

    /// Add or update UDP server configuration
    pub async fn upsert_udp_server_config(&self, config: &crate::database::models::UdpServerConfig) -> AppResult<i64> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::upsert_udp_server_config(&mut *conn, config)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to upsert UDP server config: {}", e)))
    }

    /// Create new UDP server session
    pub async fn create_udp_server_session(&self, server_config_id: i64) -> AppResult<i64> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::create_udp_server_session(&mut *conn, server_config_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to create UDP server session: {}", e)))
    }

    /// Update UDP server session statistics
    pub async fn update_udp_server_session_stats(
        &self,
        session_id: i64,
        packets_received: i32,
        packets_parsed: i32,
        parse_errors: i32,
        total_bytes_received: i32,
        average_packet_size: f64,
        max_packet_size_seen: i32,
        min_packet_size_seen: i32,
        unique_clients_count: i32,
    ) -> AppResult<()> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::update_udp_server_session_stats(
            &mut *conn,
            session_id,
            packets_received,
            packets_parsed,
            parse_errors,
            total_bytes_received,
            average_packet_size,
            max_packet_size_seen,
            min_packet_size_seen,
            unique_clients_count,
        )
        .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to update UDP server session stats: {}", e)))
    }

    /// End UDP server session
    pub async fn end_udp_server_session(&self, session_id: i64, status: &str, error_message: Option<&str>) -> AppResult<()> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::end_udp_server_session(&mut *conn, session_id, status, error_message)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to end UDP server session: {}", e)))
    }

    /// Get UDP server session by ID
    pub async fn get_udp_server_session(&self, session_id: i64) -> AppResult<Option<crate::database::models::UdpServerSession>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_udp_server_session(&*conn, session_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get UDP server session: {}", e)))
    }

    /// Get recent UDP server sessions
    pub async fn get_recent_udp_server_sessions(&self, limit: i64) -> AppResult<Vec<crate::database::models::UdpServerSession>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_recent_udp_server_sessions(&*conn, limit)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get recent UDP server sessions: {}", e)))
    }

    /// Add or update UDP client connection
    pub async fn upsert_udp_client_connection(&self, client: &crate::database::models::UdpClientConnection) -> AppResult<i64> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::upsert_udp_client_connection(&mut *conn, client)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to upsert UDP client connection: {}", e)))
    }

    /// Get active client connections for a session
    pub async fn get_active_client_connections(&self, session_id: i64) -> AppResult<Vec<crate::database::models::UdpClientConnection>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_active_client_connections(&*conn, session_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get active client connections: {}", e)))
    }

    /// Get all PSS event types
    pub async fn get_pss_event_types(&self) -> AppResult<Vec<crate::database::models::PssEventType>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_pss_event_types(&*conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS event types: {}", e)))
    }

    /// Get PSS event type by code
    pub async fn get_pss_event_type_by_code(&self, event_code: &str) -> AppResult<Option<crate::database::models::PssEventType>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_pss_event_type_by_code(&*conn, event_code)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS event type: {}", e)))
    }

    /// Get or create PSS match
    pub async fn get_or_create_pss_match(&self, match_id: &str) -> AppResult<i64> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_or_create_pss_match(&mut *conn, match_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get or create PSS match: {}", e)))
    }

    /// Update PSS match information
    pub async fn update_pss_match(&self, match_id: i64, match_data: &crate::database::models::PssMatch) -> AppResult<()> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::update_pss_match(&mut *conn, match_id, match_data)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to update PSS match: {}", e)))
    }

    /// Get or create PSS athlete
    pub async fn get_or_create_pss_athlete(&self, athlete_code: &str, short_name: &str) -> AppResult<i64> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_or_create_pss_athlete(&mut *conn, athlete_code, short_name)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get or create PSS athlete: {}", e)))
    }

    /// Update PSS athlete information
    pub async fn update_pss_athlete(&self, athlete_id: i64, athlete_data: &crate::database::models::PssAthlete) -> AppResult<()> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::update_pss_athlete(&mut *conn, athlete_id, athlete_data)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to update PSS athlete: {}", e)))
    }

    /// Store PSS event
    pub async fn store_pss_event(&self, event: &crate::database::models::PssEventV2) -> AppResult<i64> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::store_pss_event(&mut *conn, event)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to store PSS event: {}", e)))
    }

    /// Get PSS events for a session
    pub async fn get_pss_events_for_session(&self, session_id: i64, limit: Option<i64>) -> AppResult<Vec<crate::database::models::PssEventV2>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_pss_events_for_session(&*conn, session_id, limit)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS events for session: {}", e)))
    }

    /// Get PSS events for a match
    pub async fn get_pss_events_for_match(&self, match_id: i64, limit: Option<i64>) -> AppResult<Vec<crate::database::models::PssEventV2>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_pss_events_for_match(&*conn, match_id, limit)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS events for match: {}", e)))
    }

    /// Store PSS event details
    pub async fn store_pss_event_details(&self, event_id: i64, details: &[(String, Option<String>, String)]) -> AppResult<()> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::store_pss_event_details(&mut *conn, event_id, details)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to store PSS event details: {}", e)))
    }

    /// Get PSS event details
    pub async fn get_pss_event_details(&self, event_id: i64) -> AppResult<Vec<crate::database::models::PssEventDetail>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_pss_event_details(&*conn, event_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get PSS event details: {}", e)))
    }

    /// Store PSS score
    pub async fn store_pss_score(&self, score: &crate::database::models::PssScore) -> AppResult<i64> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::store_pss_score(&mut *conn, score)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to store PSS score: {}", e)))
    }

    /// Get current scores for a match
    pub async fn get_current_scores_for_match(&self, match_id: i64) -> AppResult<Vec<crate::database::models::PssScore>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_current_scores_for_match(&*conn, match_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get current scores for match: {}", e)))
    }

    /// Store PSS warning
    pub async fn store_pss_warning(&self, warning: &crate::database::models::PssWarning) -> AppResult<i64> {
        let mut conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::store_pss_warning(&mut *conn, warning)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to store PSS warning: {}", e)))
    }

    /// Get current warnings for a match
    pub async fn get_current_warnings_for_match(&self, match_id: i64) -> AppResult<Vec<crate::database::models::PssWarning>> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_current_warnings_for_match(&*conn, match_id)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get current warnings for match: {}", e)))
    }

    /// Get UDP server statistics
    pub async fn get_udp_server_statistics(&self) -> AppResult<serde_json::Value> {
        let conn = self.connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        crate::database::operations::PssUdpOperations::get_udp_server_statistics(&*conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get UDP server statistics: {}", e)))
    }

    /// Internal method to run database migrations
    async fn run_migrations_internal(connection: Arc<DatabaseConnection>) -> AppResult<()> {
        let mut conn = connection.get_connection().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        // Import the migration manager
        use crate::database::migrations::MigrationManager;
        
        let migration_manager = MigrationManager::new();
        migration_manager.migrate(&mut *conn)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to run database migrations: {}", e)))?;
        
        log::info!("Database migrations completed successfully");
        Ok(())
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