use crate::config::manager::ConfigManager;
use crate::database::{
    connection::{DatabaseConnection, DatabaseConnectionPool, PooledConnection},
    // models::*,
    // operations::*,
    models::{
        PssAthlete as DbPssAthlete, PssEventType as DbPssEventType, PssMatch as DbPssMatch,
        PssMatchAthlete as DbPssMatchAthlete, UdpClientConnection as DbUdpClientConnection,
        UdpServerConfig as DbUdpServerConfig, UdpServerSession as DbUdpServerSession,
    },
    seaorm::{connect as seaorm_connect, SeaOrmConnection},
    seaorm_ops::{
        network as sea_network, pss as sea_pss, pss_catalog as sea_catalog,
        pss_status as sea_status, ui_settings as sea_settings,
    },
    DatabaseError,
    HybridSettingsProvider,
    MigrationResult,
    MigrationStrategy,
};
use crate::entity::{
    athlete, event, event_type, matches, udp_client_connection, udp_server_config,
    udp_server_session,
};
use crate::types::{AppError, AppResult};
use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

const SCHEMA_UNIFICATION_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/db_migrations/20251105_schema_unification.sql"
));

/// Phase 2 Optimization: Enhanced Database Plugin with Connection Pooling
/// Thread-safe database operations using connection pooling
#[derive(Clone)]
pub struct DatabasePlugin {
    connection_pool: Arc<DatabaseConnectionPool>,
    connection: Arc<DatabaseConnection>,
    migration_strategy: MigrationStrategy,
    hybrid_provider: Arc<Mutex<HybridSettingsProvider>>,
    seaorm_connection: SeaOrmConnection,
}

impl DatabasePlugin {
    /// Create a new database plugin with connection pooling
    pub async fn new() -> AppResult<Self> {
        // Phase 2: Initialize connection pool with 10 connections for high-volume operations
        let connection_pool = Arc::new(DatabaseConnectionPool::new(10));

        // Initialize config manager with default config directory
        let config_dir = Path::new("config");
        let config_manager = ConfigManager::new(config_dir).await?;

        // Create a database connection that uses the pool
        let connection = Arc::new(DatabaseConnection::new_from_pool(connection_pool.clone()));

        // Initialise SeaORM connection against the same SQLite database.
        let db_path = DatabaseConnection::get_database_path().map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to resolve database path: {e}"))
        })?;
        let seaorm_connection = seaorm_connect(&db_path).await.map_err(|e| {
            crate::types::AppError::ConfigError(format!("SeaORM connection failed: {e}"))
        })?;

        let migration_strategy = MigrationStrategy::new(config_manager.clone());
        let hybrid_provider = Arc::new(Mutex::new(HybridSettingsProvider::new(
            config_manager.clone(),
            seaorm_connection.clone(),
        )));

        let plugin = Self {
            connection_pool,
            connection,
            migration_strategy,
            hybrid_provider,
            seaorm_connection,
        };

        // Run database migrations using the connection pool
        if let Err(e) = Self::run_migrations_with_pool(plugin.connection_pool.clone()).await {
            log::error!("Failed to run database migrations: {e}");
            return Err(crate::types::AppError::ConfigError(format!(
                "Database migration failed: {e}"
            )));
        }

        if let Err(err) = plugin.ensure_canonical_pss_schema().await {
            log::error!("Failed to ensure canonical SeaORM schema: {err}");
            return Err(err);
        }

        Ok(plugin)
    }

    /// Run database migrations using the connection pool
    async fn run_migrations_with_pool(
        connection_pool: Arc<DatabaseConnectionPool>,
    ) -> AppResult<()> {
        let conn = connection_pool.get_connection().map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to get database connection for migrations: {e}"
            ))
        })?;

        // Run migrations using the pooled connection
        Self::run_migrations_internal_with_pooled(&conn).await?;

        Ok(())
    }

    /// Get a pooled connection for high-performance operations
    pub fn get_pooled_connection(&self) -> Result<PooledConnection, DatabaseError> {
        self.connection_pool
            .get_connection()
            .map_err(|e| DatabaseError::Connection(e.to_string()))
    }

    /// Get pool statistics for monitoring
    pub fn get_pool_stats(&self) -> crate::database::connection::PoolStats {
        self.connection_pool.get_pool_stats()
    }

    /// Clean up old connections in the pool
    pub fn cleanup_pool(&self) {
        self.connection_pool.cleanup_old_connections();
    }

    /// Initialize UI settings in database
    pub async fn initialize_ui_settings(&self) -> AppResult<()> {
        sea_settings::initialize_ui_settings(&self.seaorm_connection)
            .await
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!(
                    "Failed to initialize UI settings: {e}"
                ))
            })
    }

    /// Get UI setting from database
    pub async fn get_ui_setting(&self, key: &str) -> AppResult<Option<String>> {
        sea_settings::get_ui_setting(&self.seaorm_connection, key)
            .await
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!("Failed to get UI setting: {e}"))
            })
    }

    /// Set UI setting in database
    pub async fn set_ui_setting(
        &self,
        key: &str,
        value: &str,
        changed_by: &str,
        change_reason: Option<&str>,
    ) -> AppResult<()> {
        sea_settings::set_ui_setting(
            &self.seaorm_connection,
            key,
            value,
            changed_by,
            change_reason,
        )
        .await
        .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to set UI setting: {e}")))
    }

    /// Get all UI settings from database
    pub async fn get_all_ui_settings(
        &self,
    ) -> AppResult<std::collections::HashMap<String, String>> {
        let settings_vec = sea_settings::get_all_ui_settings(&self.seaorm_connection)
            .await
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!("Failed to get all UI settings: {e}"))
            })?;

        // Convert Vec<(String, String)> to HashMap<String, String>
        let settings_map: std::collections::HashMap<String, String> =
            settings_vec.into_iter().collect();
        Ok(settings_map)
    }

    /// Check if database is accessible
    pub async fn is_accessible(&self) -> bool {
        self.get_connection().await.is_ok()
    }

    /// Get database connection for other plugins
    /// Returns the thread-safe database connection that uses connection pooling
    pub fn get_database_connection(&self) -> Arc<DatabaseConnection> {
        self.connection.clone()
    }

    pub fn seaorm(&self) -> SeaOrmConnection {
        self.seaorm_connection.clone()
    }

    /// Get database file size
    pub fn get_file_size(&self) -> AppResult<u64> {
        // Use the connection pool to get database statistics
        self.connection.get_file_size().map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to get database file size: {e}"))
        })
    }

    /// Get database file path
    pub fn get_database_path(&self) -> AppResult<String> {
        crate::database::connection::DatabaseConnection::get_database_path()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!("Failed to get database path: {e}"))
            })
    }

    /// Migrate JSON settings to database
    pub async fn migrate_json_to_database(&self) -> AppResult<MigrationResult> {
        self.migration_strategy
            .migrate_json_to_database(&self.seaorm_connection)
            .await
    }

    /// Get migration status
    pub async fn get_migration_status(&self) -> AppResult<MigrationStatus> {
        let settings_count = self
            .get_all_ui_settings()
            .await
            .map(|s| s.len())
            .unwrap_or(0);

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
    pub async fn get_connection(
        &self,
    ) -> AppResult<tokio::sync::MutexGuard<'_, rusqlite::Connection>> {
        self.connection.get_connection().await.map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> AppResult<()> {
        Self::run_migrations_internal(self.connection.clone()).await
    }

    // PSS and UDP Subsystem Methods

    /// Get all network interfaces
    pub async fn get_network_interfaces(
        &self,
    ) -> AppResult<Vec<crate::database::models::NetworkInterface>> {
        sea_network::get_network_interfaces(&self.seaorm_connection)
            .await
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!(
                    "Failed to get network interfaces: {e}"
                ))
            })
    }

    /// Get recommended network interface
    pub async fn get_recommended_interface(
        &self,
    ) -> AppResult<Option<crate::database::models::NetworkInterface>> {
        sea_network::get_recommended_interface(&self.seaorm_connection)
            .await
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!(
                    "Failed to get recommended interface: {e}"
                ))
            })
    }

    /// Add or update network interface
    pub async fn upsert_network_interface(
        &self,
        interface: &crate::database::models::NetworkInterface,
    ) -> AppResult<i64> {
        sea_network::upsert_network_interface(&self.seaorm_connection, interface)
            .await
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!(
                    "Failed to upsert network interface: {e}"
                ))
            })
    }

    /// Get all UDP server configurations
    pub async fn get_udp_server_configs(&self) -> AppResult<Vec<DbUdpServerConfig>> {
        let sea = self.seaorm_connection.clone();
        let records = udp_server_config::Entity::find()
            .order_by_asc(udp_server_config::Column::Name)
            .all(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get UDP server configs: {e}")))?;

        let configs = records
            .into_iter()
            .map(map_udp_server_config_model)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(configs)
    }

    /// Get UDP server configuration by ID
    pub async fn get_udp_server_config(
        &self,
        config_id: i64,
    ) -> AppResult<Option<DbUdpServerConfig>> {
        let sea = self.seaorm_connection.clone();
        let record = udp_server_config::Entity::find_by_id(config_id as i32)
            .one(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get UDP server config: {e}")))?;

        record.map(map_udp_server_config_model).transpose()
    }

    /// Add or update UDP server configuration
    pub async fn upsert_udp_server_config(&self, config: &DbUdpServerConfig) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        if let Some(id) = config.id {
            let active = udp_server_config::ActiveModel {
                id: Set(id as i32),
                name: Set(config.name.clone()),
                port: Set(i32::from(config.port)),
                bind_address: Set(config.bind_address.clone()),
                network_interface_id: Set(config.network_interface_id.map(|v| v as i32)),
                enabled: Set(config.enabled),
                auto_start: Set(config.auto_start),
                max_packet_size: Set(config.max_packet_size),
                buffer_size: Set(config.buffer_size),
                timeout_ms: Set(config.timeout_ms),
                created_at: Set(config.created_at.naive_utc()),
                updated_at: Set(Utc::now().naive_utc()),
            };

            active.update(&sea).await.map_err(|e| {
                AppError::ConfigError(format!("Failed to update UDP server config: {e}"))
            })?;
            Ok(id)
        } else {
            let now = Utc::now();
            let active = udp_server_config::ActiveModel {
                name: Set(config.name.clone()),
                port: Set(i32::from(config.port)),
                bind_address: Set(config.bind_address.clone()),
                network_interface_id: Set(config.network_interface_id.map(|v| v as i32)),
                enabled: Set(config.enabled),
                auto_start: Set(config.auto_start),
                max_packet_size: Set(config.max_packet_size),
                buffer_size: Set(config.buffer_size),
                timeout_ms: Set(config.timeout_ms),
                created_at: Set(now.naive_utc()),
                updated_at: Set(now.naive_utc()),
                ..Default::default()
            };

            let inserted = active.insert(&sea).await.map_err(|e| {
                AppError::ConfigError(format!("Failed to insert UDP server config: {e}"))
            })?;
            Ok(inserted.id as i64)
        }
    }

    /// Create new UDP server session
    pub async fn create_udp_server_session(&self, server_config_id: i64) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        let now = Utc::now();
        let active = udp_server_session::ActiveModel {
            server_config_id: Set(server_config_id as i32),
            start_time: Set(now.to_rfc3339()),
            end_time: Set(None),
            status: Set("running".to_string()),
            packets_received: Set(0),
            packets_parsed: Set(0),
            parse_errors: Set(0),
            total_bytes_received: Set(0),
            average_packet_size: Set(0.0),
            max_packet_size_seen: Set(0),
            min_packet_size_seen: Set(0),
            unique_clients_count: Set(0),
            error_message: Set(None),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
            ..Default::default()
        };

        let model = active.insert(&sea).await.map_err(|e| {
            AppError::ConfigError(format!("Failed to create UDP server session: {e}"))
        })?;

        Ok(model.id as i64)
    }

    /// Update UDP server session statistics
    #[allow(clippy::too_many_arguments)]
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
        let sea = self.seaorm_connection.clone();
        let now = Utc::now();
        udp_server_session::ActiveModel {
            id: Set(session_id as i32),
            packets_received: Set(packets_received),
            packets_parsed: Set(packets_parsed),
            parse_errors: Set(parse_errors),
            total_bytes_received: Set(i64::from(total_bytes_received)),
            average_packet_size: Set(average_packet_size),
            max_packet_size_seen: Set(max_packet_size_seen),
            min_packet_size_seen: Set(min_packet_size_seen),
            unique_clients_count: Set(unique_clients_count),
            updated_at: Set(now.naive_utc()),
            ..Default::default()
        }
        .update(&sea)
        .await
        .map_err(|e| AppError::ConfigError(format!("Failed to update UDP session stats: {e}")))?;

        Ok(())
    }

    /// End UDP server session
    pub async fn end_udp_server_session(
        &self,
        session_id: i64,
        status: &str,
        error_message: Option<&str>,
    ) -> AppResult<()> {
        let sea = self.seaorm_connection.clone();
        let now = Utc::now();
        udp_server_session::ActiveModel {
            id: Set(session_id as i32),
            end_time: Set(Some(now.to_rfc3339())),
            status: Set(status.to_string()),
            error_message: Set(error_message.map(|s| s.to_string())),
            updated_at: Set(now.naive_utc()),
            ..Default::default()
        }
        .update(&sea)
        .await
        .map_err(|e| AppError::ConfigError(format!("Failed to end UDP server session: {e}")))?;

        Ok(())
    }

    /// Get UDP server session by ID
    pub async fn get_udp_server_session(
        &self,
        session_id: i64,
    ) -> AppResult<Option<DbUdpServerSession>> {
        let sea = self.seaorm_connection.clone();
        let record = udp_server_session::Entity::find_by_id(session_id as i32)
            .one(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get UDP server session: {e}")))?;

        record.map(map_udp_server_session_model).transpose()
    }

    /// Get recent UDP server sessions
    pub async fn get_recent_udp_server_sessions(
        &self,
        limit: i64,
    ) -> AppResult<Vec<DbUdpServerSession>> {
        let sea = self.seaorm_connection.clone();
        let records = udp_server_session::Entity::find()
            .order_by_desc(udp_server_session::Column::StartTime)
            .limit(limit as u64)
            .all(&sea)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to get recent UDP server sessions: {e}"))
            })?;

        let sessions = records
            .into_iter()
            .map(map_udp_server_session_model)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Add or update UDP client connection
    pub async fn upsert_udp_client_connection(
        &self,
        client: &DbUdpClientConnection,
    ) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        if let Some(id) = client.id {
            udp_client_connection::ActiveModel {
                id: Set(id as i32),
                last_seen: Set(client.last_seen.to_rfc3339()),
                packets_received: Set(client.packets_received),
                total_bytes_received: Set(i64::from(client.total_bytes_received)),
                is_active: Set(client.is_active),
                ..Default::default()
            }
            .update(&sea)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to update UDP client connection: {e}"))
            })?;
            Ok(id)
        } else {
            let now = Utc::now();
            let active = udp_client_connection::ActiveModel {
                server_session_id: Set(client.session_id as i32),
                client_address: Set(client.client_address.clone()),
                client_port: Set(i32::from(client.client_port)),
                first_seen: Set(client.first_seen.to_rfc3339()),
                last_seen: Set(client.last_seen.to_rfc3339()),
                packets_received: Set(client.packets_received),
                total_bytes_received: Set(i64::from(client.total_bytes_received)),
                is_active: Set(client.is_active),
                created_at: Set(now.naive_utc()),
                ..Default::default()
            };

            let inserted = active
                .insert(&sea)
                .await
                .map_err(|e| AppError::ConfigError(format!("Failed to insert UDP client: {e}")))?;
            Ok(inserted.id as i64)
        }
    }

    /// Get active client connections for a session
    pub async fn get_active_client_connections(
        &self,
        session_id: i64,
    ) -> AppResult<Vec<DbUdpClientConnection>> {
        let sea = self.seaorm_connection.clone();
        let records = udp_client_connection::Entity::find()
            .filter(udp_client_connection::Column::ServerSessionId.eq(session_id as i32))
            .filter(udp_client_connection::Column::IsActive.eq(true))
            .order_by_desc(udp_client_connection::Column::LastSeen)
            .all(&sea)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to get active client connections: {e}"))
            })?;

        let clients = records
            .into_iter()
            .map(map_udp_client_connection_model)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(clients)
    }

    /// Get all PSS event types
    pub async fn get_pss_event_types(&self) -> AppResult<Vec<DbPssEventType>> {
        let sea = self.seaorm_connection.clone();
        let records = event_type::Entity::find()
            .order_by_asc(event_type::Column::Code)
            .all(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get PSS event types: {e}")))?;

        let mut items = Vec::with_capacity(records.len());
        for record in records {
            items.push(map_event_type_model(record)?);
        }
        Ok(items)
    }

    /// Get PSS event type by code
    pub async fn get_pss_event_type_by_code(
        &self,
        event_code: &str,
    ) -> AppResult<Option<DbPssEventType>> {
        let sea = self.seaorm_connection.clone();
        let record = event_type::Entity::find()
            .filter(event_type::Column::Code.eq(event_code))
            .one(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get PSS event type: {e}")))?;

        record.map(map_event_type_model).transpose()
    }

    /// Upsert PSS event type
    pub async fn upsert_pss_event_type(&self, event_type: &DbPssEventType) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        let created_at = event_type.created_at;
        if let Some(id) = event_type.id {
            event_type::ActiveModel {
                id: Set(id as i32),
                code: Set(event_type.event_code.clone()),
                name: Set(event_type.event_name.clone()),
                description: Set(event_type.description.clone()),
                category: Set(event_type.category.clone()),
                is_active: Set(event_type.is_active),
                created_at: Set(created_at.naive_utc()),
            }
            .update(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to update PSS event type: {e}")))?;
            Ok(id)
        } else {
            let inserted = event_type::ActiveModel {
                code: Set(event_type.event_code.clone()),
                name: Set(event_type.event_name.clone()),
                description: Set(event_type.description.clone()),
                category: Set(event_type.category.clone()),
                is_active: Set(event_type.is_active),
                created_at: Set(created_at.naive_utc()),
                ..Default::default()
            }
            .insert(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to insert PSS event type: {e}")))?;

            Ok(inserted.id as i64)
        }
    }

    /// Get or create PSS match
    pub async fn get_or_create_pss_match(&self, match_id: &str) -> AppResult<i64> {
        sea_catalog::get_or_create_match(&self.seaorm_connection, match_id)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get/create match: {e}")))
    }

    /// Update PSS match information
    pub async fn update_pss_match(
        &self,
        match_id: i64,
        match_data: &crate::database::models::PssMatch,
    ) -> AppResult<()> {
        sea_catalog::update_match(&self.seaorm_connection, match_id, match_data)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to update PSS match: {e}")))
    }

    /// Fetch match metadata by database id
    pub async fn get_pss_match_by_id(&self, match_id: i64) -> AppResult<Option<DbPssMatch>> {
        sea_catalog::get_match_by_id(&self.seaorm_connection, match_id)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to load match {match_id}: {e}")))
    }

    /// Fetch match metadata by match_code
    pub async fn get_pss_match_by_match_id(
        &self,
        match_code: &str,
    ) -> AppResult<Option<DbPssMatch>> {
        sea_catalog::get_match_by_code(&self.seaorm_connection, match_code)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to load match {match_code}: {e}")))
    }

    /// List recent matches
    pub async fn get_pss_matches(&self, limit: Option<i64>) -> AppResult<Vec<DbPssMatch>> {
        sea_catalog::get_matches(&self.seaorm_connection, limit)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to list PSS matches: {e}")))
    }

    /// Insert a new match row
    pub async fn insert_pss_match(&self, match_data: &DbPssMatch) -> AppResult<i64> {
        sea_catalog::insert_match(&self.seaorm_connection, match_data)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to insert PSS match: {e}")))
    }

    /// Rename match identifier
    pub async fn rename_pss_match_id(&self, match_db_id: i64, new_match_id: &str) -> AppResult<()> {
        sea_catalog::rename_match_id(&self.seaorm_connection, match_db_id, new_match_id)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to rename match {match_db_id}: {e}"))
            })
    }

    /// Attach tournament context
    pub async fn set_pss_match_tournament_context(
        &self,
        match_db_id: i64,
        tournament_id: Option<i64>,
    ) -> AppResult<()> {
        sea_catalog::set_match_tournament_context(
            &self.seaorm_connection,
            match_db_id,
            tournament_id,
        )
        .await
        .map_err(|e| AppError::ConfigError(format!("Failed to set match tournament context: {e}")))
    }

    /// Reassign events from one match to another
    pub async fn reassign_events_between_matches(
        &self,
        from_match_id: i64,
        to_match_id: i64,
    ) -> AppResult<usize> {
        sea_catalog::reassign_events_between_matches(
            &self.seaorm_connection,
            from_match_id,
            to_match_id,
        )
        .await
        .map_err(|e| AppError::ConfigError(format!("Failed to reassign events: {e}")))
    }

    /// Fetch match participants and associated athletes
    pub async fn get_pss_match_athletes(
        &self,
        match_id: i64,
    ) -> AppResult<Vec<(DbPssMatchAthlete, DbPssAthlete)>> {
        sea_catalog::get_match_athletes(&self.seaorm_connection, match_id)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get match athletes: {e}")))
    }

    /// Link an athlete to a match
    pub async fn insert_pss_match_athlete(
        &self,
        match_athlete: &DbPssMatchAthlete,
    ) -> AppResult<i64> {
        sea_catalog::insert_match_athlete(&self.seaorm_connection, match_athlete)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to insert match athlete: {e}")))
    }

    /// Get or create PSS athlete
    pub async fn get_or_create_pss_athlete(
        &self,
        athlete_code: &str,
        short_name: &str,
    ) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        if let Some(existing) = athlete::Entity::find()
            .filter(athlete::Column::PssCode.eq(Some(athlete_code.to_string())))
            .one(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to lookup athlete: {e}")))?
        {
            return Ok(existing.id as i64);
        }

        let now = Utc::now().naive_utc();
        let active = athlete::ActiveModel {
            uuid: Set(Uuid::new_v4().to_string()),
            pss_code: Set(Some(athlete_code.to_string())),
            short_name: Set(Some(short_name.to_string())),
            display_name: Set(Some(short_name.to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let inserted = active
            .insert(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to create athlete: {e}")))?;

        Ok(inserted.id as i64)
    }

    /// Update PSS athlete information
    pub async fn update_pss_athlete(
        &self,
        athlete_id: i64,
        athlete_data: &crate::database::models::PssAthlete,
    ) -> AppResult<()> {
        let sea = self.seaorm_connection.clone();
        let existing = athlete::Entity::find_by_id(athlete_id as i32)
            .one(&sea)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to load athlete {athlete_id}: {e}"))
            })?
            .ok_or_else(|| AppError::ConfigError(format!("Athlete {athlete_id} not found")))?;

        let mut active: athlete::ActiveModel = existing.into();
        active.pss_code = Set(Some(athlete_data.athlete_code.clone()));
        active.short_name = Set(Some(athlete_data.short_name.clone()));
        active.display_name = Set(athlete_data
            .long_name
            .clone()
            .or_else(|| Some(athlete_data.short_name.clone())));
        active.country_code = Set(athlete_data.country_code.clone());
        active.flag_id = Set(athlete_data.flag_id.map(|v| v as i32));
        active.updated_at = Set(athlete_data.updated_at.naive_utc());
        active.created_at = Set(athlete_data.created_at.naive_utc());

        active
            .update(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to update athlete: {e}")))?;

        Ok(())
    }

    /// Store PSS event
    pub async fn store_pss_event(
        &self,
        event: &crate::database::models::PssEventV2,
    ) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        sea_pss::insert_event(&sea, event)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to store PSS event: {e}")))
    }

    /// Get PSS events for a session
    pub async fn get_pss_events_for_session(
        &self,
        session_id: i64,
        limit: Option<i64>,
    ) -> AppResult<Vec<crate::database::models::PssEventV2>> {
        let sea = self.seaorm_connection.clone();
        sea_pss::get_events_for_session(&sea, session_id, limit)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to get PSS events for session: {e}"))
            })
    }

    /// Get PSS events for a match
    pub async fn get_pss_events_for_match(
        &self,
        match_id: i64,
        limit: Option<i64>,
    ) -> AppResult<Vec<crate::database::models::PssEventV2>> {
        let sea = self.seaorm_connection.clone();
        sea_pss::get_events_for_match(&sea, match_id, limit)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get PSS events for match: {e}")))
    }

    /// Store PSS event details
    pub async fn store_pss_event_details(
        &self,
        event_id: i64,
        details: &[(String, Option<String>, String)],
    ) -> AppResult<()> {
        let sea = self.seaorm_connection.clone();
        sea_pss::insert_event_details(&sea, event_id, details)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to store PSS event details: {e}")))
    }

    /// Get PSS event details
    pub async fn get_pss_event_details(
        &self,
        event_id: i64,
    ) -> AppResult<Vec<crate::database::models::PssEventDetail>> {
        let sea = self.seaorm_connection.clone();
        sea_pss::get_event_details(&sea, event_id)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get PSS event details: {e}")))
    }

    /// Store PSS score
    pub async fn store_pss_score(
        &self,
        score: &crate::database::models::PssScore,
    ) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        sea_pss::insert_score(&sea, score)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to store PSS score: {e}")))
    }

    /// Get current scores for a match
    pub async fn get_current_scores_for_match(
        &self,
        match_id: i64,
    ) -> AppResult<Vec<crate::database::models::PssScore>> {
        let sea = self.seaorm_connection.clone();
        sea_pss::get_current_scores_for_match(&sea, match_id)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to get current scores for match: {e}"))
            })
    }

    /// Store PSS warning
    pub async fn store_pss_warning(
        &self,
        warning: &crate::database::models::PssWarning,
    ) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        sea_pss::insert_warning(&sea, warning)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to store PSS warning: {e}")))
    }

    /// Get current warnings for a match
    pub async fn get_current_warnings_for_match(
        &self,
        match_id: i64,
    ) -> AppResult<Vec<crate::database::models::PssWarning>> {
        let sea = self.seaorm_connection.clone();
        sea_pss::get_current_warnings_for_match(&sea, match_id)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to get current warnings for match: {e}"))
            })
    }

    /// Get UDP server statistics
    pub async fn get_udp_server_statistics(&self) -> AppResult<serde_json::Value> {
        let sea = self.seaorm_connection.clone();
        let total_sessions = udp_server_session::Entity::find()
            .count(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to count UDP sessions: {e}")))?;
        let active_sessions = udp_server_session::Entity::find()
            .filter(udp_server_session::Column::Status.eq("running"))
            .count(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to count active sessions: {e}")))?;
        let total_events = event::Entity::find()
            .count(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to count events: {e}")))?;
        let total_matches = matches::Entity::find()
            .count(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to count matches: {e}")))?;
        let threshold = Utc::now() - ChronoDuration::hours(24);
        let recent_events = event::Entity::find()
            .filter(event::Column::CreatedAt.gt(threshold))
            .count(&sea)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to count recent events: {e}")))?;

        Ok(serde_json::json!({
            "total_sessions": total_sessions,
            "active_sessions": active_sessions,
            "total_events": total_events,
            "total_matches": total_matches,
            "recent_events_24h": recent_events
        }))
    }

    // PSS Event Status Operations

    /// Store a PSS event with status mark
    pub async fn store_pss_event_with_status(
        &self,
        event: &crate::database::models::PssEventV2,
    ) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        sea_status::store_event_with_status(&sea, event)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to store PSS event with status: {e}"))
            })
    }

    /// Update event recognition status and record history
    pub async fn update_event_recognition_status(
        &self,
        event_id: i64,
        new_status: &str,
        changed_by: &str,
        change_reason: Option<&str>,
    ) -> AppResult<()> {
        let sea = self.seaorm_connection.clone();
        sea_status::update_event_recognition_status(
            &sea,
            event_id,
            new_status,
            changed_by,
            change_reason,
        )
        .await
        .map_err(|e| {
            AppError::ConfigError(format!("Failed to update event recognition status: {e}"))
        })
    }

    /// Store unknown event
    pub async fn store_unknown_event(
        &self,
        unknown_event: &crate::database::models::PssUnknownEvent,
    ) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        sea_status::store_unknown_event(&sea, unknown_event)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to store unknown event: {e}")))
    }

    /// Get validation rules for an event type
    pub async fn get_validation_rules(
        &self,
        event_code: &str,
        protocol_version: &str,
    ) -> AppResult<Vec<crate::database::models::PssEventValidationRule>> {
        let sea = self.seaorm_connection.clone();
        sea_status::get_validation_rules(&sea, event_code, protocol_version)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get validation rules: {e}")))
    }

    /// Store validation result
    pub async fn store_validation_result(
        &self,
        validation_result: &crate::database::models::PssEventValidationResult,
    ) -> AppResult<i64> {
        let sea = self.seaorm_connection.clone();
        sea_status::store_validation_result(&sea, validation_result)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to store validation result: {e}")))
    }

    /// Update event statistics
    pub async fn update_event_statistics(
        &self,
        session_id: i64,
        event_type_id: Option<i64>,
        recognition_status: &str,
        processing_time_ms: Option<i32>,
    ) -> AppResult<()> {
        let sea = self.seaorm_connection.clone();
        sea_status::update_event_statistics(
            &sea,
            session_id,
            event_type_id,
            recognition_status,
            processing_time_ms,
        )
        .await
        .map_err(|e| AppError::ConfigError(format!("Failed to update event statistics: {e}")))
    }

    /// Get event statistics for a session
    pub async fn get_session_statistics(
        &self,
        session_id: i64,
    ) -> AppResult<Vec<crate::database::models::PssEventStatistics>> {
        let sea = self.seaorm_connection.clone();
        sea_status::get_session_statistics(&sea, session_id)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get session statistics: {e}")))
    }

    /// Get unknown events for analysis
    pub async fn get_unknown_events(
        &self,
        session_id: Option<i64>,
        limit: Option<i64>,
    ) -> AppResult<Vec<crate::database::models::PssUnknownEvent>> {
        let sea = self.seaorm_connection.clone();
        sea_status::get_unknown_events(&sea, session_id, limit)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get unknown events: {e}")))
    }

    /// Get recognition history for an event
    pub async fn get_event_recognition_history(
        &self,
        event_id: i64,
    ) -> AppResult<Vec<crate::database::models::PssEventRecognitionHistory>> {
        let sea = self.seaorm_connection.clone();
        sea_status::get_event_recognition_history(&sea, event_id)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to get event recognition history: {e}"))
            })
    }

    /// Get events by recognition status
    pub async fn get_events_by_status(
        &self,
        session_id: i64,
        recognition_status: &str,
        limit: Option<i64>,
    ) -> AppResult<Vec<crate::database::models::PssEventV2>> {
        let sea = self.seaorm_connection.clone();
        sea_status::get_events_by_status(&sea, session_id, recognition_status, limit)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get events by status: {e}")))
    }

    /// Get comprehensive event statistics with status breakdown
    pub async fn get_comprehensive_event_statistics(
        &self,
        session_id: i64,
    ) -> AppResult<serde_json::Value> {
        let sea = self.seaorm_connection.clone();
        sea_status::get_comprehensive_event_statistics(&sea, session_id)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to get comprehensive event statistics: {e}"))
            })
    }

    // Phase 2: Data Archival Operations

    /// Archive events older than specified days
    pub async fn archive_old_events(&self, days_old: i64) -> AppResult<usize> {
        let mut conn = self.connection.get_connection().await.map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;
        crate::database::operations::DataArchivalOperations::archive_old_events(&mut conn, days_old)
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!("Failed to archive old events: {e}"))
            })
    }

    /// Get archive statistics
    pub async fn get_archive_statistics(
        &self,
    ) -> AppResult<crate::database::operations::ArchiveStatistics> {
        let conn = self.connection.get_connection().await.map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;
        crate::database::operations::DataArchivalOperations::get_archive_statistics(&conn).map_err(
            |e| {
                crate::types::AppError::ConfigError(format!(
                    "Failed to get archive statistics: {e}"
                ))
            },
        )
    }

    /// Restore events from archive
    pub async fn restore_from_archive(&self, start_date: &str, end_date: &str) -> AppResult<usize> {
        let mut conn = self.connection.get_connection().await.map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;
        crate::database::operations::DataArchivalOperations::restore_from_archive(
            &mut conn, start_date, end_date,
        )
        .map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to restore from archive: {e}"))
        })
    }

    /// Clean up old archive data
    pub async fn cleanup_old_archive_data(&self, days_old: i64) -> AppResult<usize> {
        let mut conn = self.connection.get_connection().await.map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;
        crate::database::operations::DataArchivalOperations::cleanup_old_archive_data(
            &mut conn, days_old,
        )
        .map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to cleanup archive data: {e}"))
        })
    }

    /// Optimize archive tables
    pub async fn optimize_archive_tables(&self) -> AppResult<()> {
        let mut conn = self.connection.get_connection().await.map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;
        crate::database::operations::DataArchivalOperations::optimize_archive_tables(&mut conn)
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!(
                    "Failed to optimize archive tables: {e}"
                ))
            })
    }

    /// Get OBS recording operations
    pub fn obs_recording_operations(&self) -> &crate::database::operations::ObsRecordingOperations {
        &crate::database::operations::ObsRecordingOperations
    }

    async fn ensure_canonical_pss_schema(&self) -> AppResult<()> {
        let conn = self.connection.get_connection().await.map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        let has_match = table_exists(&conn, "match").map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to inspect schema for canonical match table: {e}"
            ))
        })?;
        let has_event = table_exists(&conn, "event").map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to inspect schema for canonical event table: {e}"
            ))
        })?;

        if has_match && has_event {
            return Ok(());
        }

        let legacy_matches = table_exists(&conn, "pss_matches").map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to inspect legacy match schema: {e}"
            ))
        })?;
        let legacy_events = table_exists(&conn, "pss_events").map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to inspect legacy event schema: {e}"
            ))
        })?;

        if !(legacy_matches && legacy_events) {
            log::warn!(
                "Legacy PSS tables not found; skipping canonical schema migration (matches={legacy_matches}, events={legacy_events})"
            );
            return Ok(());
        }

        log::info!("Migrating legacy PSS data into canonical SeaORM schema");
        conn.execute_batch(SCHEMA_UNIFICATION_SQL).map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to execute schema unification migration: {e}"
            ))
        })?;
        log::info!("Canonical SeaORM schema migration completed successfully");

        Ok(())
    }

    /// Internal method to run database migrations
    async fn run_migrations_internal(connection: Arc<DatabaseConnection>) -> AppResult<()> {
        let conn = connection.get_connection().await.map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        // Import the migration manager
        use crate::database::migrations::MigrationManager;

        let migration_manager = MigrationManager::new();
        migration_manager.migrate(&conn).map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to run database migrations: {e}"))
        })?;

        log::info!("Database migrations completed successfully");
        Ok(())
    }

    /// Internal method to run database migrations using a pooled connection
    async fn run_migrations_internal_with_pooled(conn: &PooledConnection) -> AppResult<()> {
        // Import the migration manager
        use crate::database::migrations::MigrationManager;

        let migration_manager = MigrationManager::new();
        migration_manager.migrate(conn).map_err(|e| {
            crate::types::AppError::ConfigError(format!("Failed to run database migrations: {e}"))
        })?;

        log::info!("Database migrations completed successfully");
        Ok(())
    }
}

fn map_udp_server_config_model(model: udp_server_config::Model) -> AppResult<DbUdpServerConfig> {
    let udp_server_config::Model {
        id,
        name,
        port,
        bind_address,
        network_interface_id,
        enabled,
        auto_start,
        max_packet_size,
        buffer_size,
        timeout_ms,
        created_at,
        updated_at,
    } = model;

    let port_u16 = u16::try_from(port)
        .map_err(|e| AppError::ConfigError(format!("Invalid UDP port value {port}: {e}")))?;

    Ok(DbUdpServerConfig {
        id: Some(id as i64),
        name,
        port: port_u16,
        bind_address,
        network_interface_id: network_interface_id.map(|v| v as i64),
        enabled,
        auto_start,
        max_packet_size,
        buffer_size,
        timeout_ms,
        created_at: Utc.from_utc_datetime(&created_at),
        updated_at: Utc.from_utc_datetime(&updated_at),
    })
}

fn map_udp_server_session_model(model: udp_server_session::Model) -> AppResult<DbUdpServerSession> {
    let udp_server_session::Model {
        id,
        server_config_id,
        start_time,
        end_time,
        status,
        packets_received,
        packets_parsed,
        parse_errors,
        total_bytes_received,
        average_packet_size,
        max_packet_size_seen,
        min_packet_size_seen,
        unique_clients_count,
        error_message,
        created_at: _,
        updated_at: _,
    } = model;

    let start = parse_rfc3339(&start_time, "start_time")?;
    let end = match end_time {
        Some(value) => Some(parse_rfc3339(&value, "end_time")?),
        None => None,
    };
    let total_bytes = convert_i64_to_i32(total_bytes_received, "total_bytes_received")?;

    Ok(DbUdpServerSession {
        id: Some(id as i64),
        server_config_id: server_config_id as i64,
        start_time: start,
        end_time: end,
        status,
        packets_received,
        packets_parsed,
        parse_errors,
        total_bytes_received: total_bytes,
        average_packet_size,
        max_packet_size_seen,
        min_packet_size_seen,
        unique_clients_count,
        error_message,
    })
}

fn map_udp_client_connection_model(
    model: udp_client_connection::Model,
) -> AppResult<DbUdpClientConnection> {
    let udp_client_connection::Model {
        id,
        server_session_id,
        client_address,
        client_port,
        first_seen,
        last_seen,
        packets_received,
        total_bytes_received,
        is_active,
        created_at: _,
    } = model;

    let port_u16 = u16::try_from(client_port).map_err(|e| {
        AppError::ConfigError(format!("Invalid UDP client port {client_port}: {e}"))
    })?;
    let first_seen_dt = parse_rfc3339(&first_seen, "first_seen")?;
    let last_seen_dt = parse_rfc3339(&last_seen, "last_seen")?;
    let total_bytes = convert_i64_to_i32(total_bytes_received, "total_bytes_received")?;

    Ok(DbUdpClientConnection {
        id: Some(id as i64),
        session_id: server_session_id as i64,
        client_address,
        client_port: port_u16,
        first_seen: first_seen_dt,
        last_seen: last_seen_dt,
        packets_received,
        total_bytes_received: total_bytes,
        is_active,
    })
}

fn parse_rfc3339(value: &str, field: &str) -> AppResult<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| AppError::ConfigError(format!("Invalid {field} value '{value}': {e}")))
}

fn convert_i64_to_i32(value: i64, field: &str) -> AppResult<i32> {
    i32::try_from(value)
        .map_err(|e| AppError::ConfigError(format!("{field} out of range ({value}): {e}")))
}

fn map_event_type_model(model: event_type::Model) -> AppResult<DbPssEventType> {
    Ok(DbPssEventType {
        id: Some(model.id as i64),
        event_code: model.code,
        event_name: model.name,
        description: model.description,
        category: model.category,
        is_active: model.is_active,
        created_at: Utc.from_utc_datetime(&model.created_at),
    })
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

fn table_exists(conn: &rusqlite::Connection, table: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT 1 FROM sqlite_master WHERE type IN ('table', 'view') AND name = ?1 LIMIT 1",
    )?;
    stmt.exists([table])
}
