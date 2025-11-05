use crate::config::AppConfig;
use crate::database::DatabaseConnection;
use crate::types::AppResult;
use chrono::Utc;
use log::{info, warn};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

const CONFIG_PRIMARY_KEY: &str = "app.config";
const CONFIG_PRIMARY_DESCRIPTION: &str = "Primary application configuration (JSON)";
const CONFIG_PRIMARY_CATEGORY: &str = "app";
const CONFIG_BACKUP_KEY: &str = "app.config.backup";
const CONFIG_BACKUP_DESCRIPTION: &str = "Backup application configuration (JSON)";
const CONFIG_BACKUP_CATEGORY: &str = "app_backup";

/// Configuration manager for handling application settings
#[derive(Clone)]
pub struct ConfigManager {
    /// Current configuration
    config: Arc<RwLock<AppConfig>>,
    /// Database connection backing configuration persistence
    database: Arc<DatabaseConnection>,
    /// Legacy JSON configuration path (retained for import/export/migration)
    legacy_config_path: PathBuf,
    /// Legacy JSON backup path (retained for import/export/migration)
    legacy_backup_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub async fn new(config_dir: &Path) -> AppResult<Self> {
        // Ensure config directory exists for legacy import/export compatibility
        fs::create_dir_all(config_dir)?;

        let legacy_config_path = config_dir.join("app_config.json");
        let legacy_backup_path = config_dir.join("app_config.backup.json");
        let database = Arc::new(DatabaseConnection::new()?);

        // Load configuration from database or migrate legacy JSON file.
        let initial_config = {
            let mut conn = database.get_connection().await?;
            Self::ensure_schema(&conn)?;

            if let Some(config) = Self::load_from_database(&conn)? {
                config
            } else if legacy_config_path.exists() {
                let migrated = Self::load_from_file(&legacy_config_path)?;
                Self::persist_to_database(&mut conn, &migrated)?;
                Self::archive_legacy_file(&legacy_config_path);
                if legacy_backup_path.exists() {
                    Self::archive_legacy_file(&legacy_backup_path);
                }
                migrated
            } else {
                let default_config = AppConfig::default();
                Self::persist_to_database(&mut conn, &default_config)?;
                default_config
            }
        };

        Ok(Self {
            config: Arc::new(RwLock::new(initial_config)),
            database,
            legacy_config_path,
            legacy_backup_path,
        })
    }

    /// Load configuration from file (legacy path).
    fn load_from_file(config_path: &Path) -> AppResult<AppConfig> {
        let content = fs::read_to_string(config_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Ensure the backing database schema exists.
    fn ensure_schema(conn: &Connection) -> AppResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_config (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to ensure app_config table: {}",
                e
            ))
        })?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_app_config_category ON app_config(category)",
            [],
        )
        .map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to ensure app_config index: {}",
                e
            ))
        })?;

        Ok(())
    }

    /// Load configuration payload from database.
    fn load_from_database(conn: &Connection) -> AppResult<Option<AppConfig>> {
        Self::load_config_by_key(conn, CONFIG_PRIMARY_KEY)
    }

    fn load_backup_from_database(conn: &Connection) -> AppResult<Option<AppConfig>> {
        Self::load_config_by_key(conn, CONFIG_BACKUP_KEY)
    }

    fn load_config_by_key(conn: &Connection, key: &str) -> AppResult<Option<AppConfig>> {
        let mut stmt = conn
            .prepare("SELECT value FROM app_config WHERE key = ?1 LIMIT 1")
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!(
                    "Failed to prepare config load statement: {}",
                    e
                ))
            })?;

        let value: Option<String> = stmt
            .query_row(params![key], |row| row.get(0))
            .optional()
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!(
                    "Failed to load configuration from database: {}",
                    e
                ))
            })?;

        match value {
            Some(json) => {
                let config: AppConfig = serde_json::from_str(&json)?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    /// Persist configuration snapshot to database (also updating backup copy).
    fn persist_to_database(conn: &mut Connection, config: &AppConfig) -> AppResult<()> {
        let serialized = serde_json::to_string_pretty(config)?;

        let transaction = conn.transaction().map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to open transaction for config save: {}",
                e
            ))
        })?;

        let existing: Option<String> = transaction
            .prepare("SELECT value FROM app_config WHERE key = ?1 LIMIT 1")
            .and_then(|mut stmt| {
                stmt.query_row(params![CONFIG_PRIMARY_KEY], |row| row.get(0))
                    .optional()
            })
            .map_err(|e| {
                crate::types::AppError::ConfigError(format!(
                    "Failed to read existing configuration: {}",
                    e
                ))
            })?;

        if let Some(previous_json) = existing {
            Self::upsert_config_row(
                &transaction,
                CONFIG_BACKUP_KEY,
                CONFIG_BACKUP_CATEGORY,
                CONFIG_BACKUP_DESCRIPTION,
                &previous_json,
            )?;
        }

        Self::upsert_config_row(
            &transaction,
            CONFIG_PRIMARY_KEY,
            CONFIG_PRIMARY_CATEGORY,
            CONFIG_PRIMARY_DESCRIPTION,
            &serialized,
        )?;

        transaction.commit().map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to commit configuration transaction: {}",
                e
            ))
        })?;

        Ok(())
    }

    fn upsert_config_row(
        tx: &Transaction,
        key: &str,
        category: &str,
        description: &str,
        value: &str,
    ) -> AppResult<()> {
        tx.execute(
            "INSERT INTO app_config (key, value, category, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                category = excluded.category,
                description = excluded.description,
                updated_at = CURRENT_TIMESTAMP",
            params![key, value, category, description],
        )
        .map_err(|e| {
            crate::types::AppError::ConfigError(format!(
                "Failed to upsert configuration row '{}': {}",
                key, e
            ))
        })?;
        Ok(())
    }

    fn archive_legacy_file(path: &Path) {
        if !path.exists() {
            return;
        }

        if let Some(stem) = path.file_stem() {
            let mut archived = path.with_file_name(stem);
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default();
            let new_extension = if extension.is_empty() {
                "legacy"
            } else {
                &format!("{}.legacy", extension)
            };
            archived.set_extension(new_extension);

            if let Err(err) = fs::rename(path, &archived) {
                warn!(
                    "Failed to archive legacy config file {:?}: {}",
                    path,
                    err
                );
            } else {
                info!(
                    "Archived legacy configuration file {:?} -> {:?}",
                    path,
                    archived
                );
            }
        }
    }

    /// Persist configuration with retry logic and database-backed storage.
    async fn save_config(&self, config: &AppConfig) -> AppResult<AppConfig> {
        const MAX_RETRIES: u32 = 3;
        let mut retry_count = 0;

        while retry_count < MAX_RETRIES {
            match self.try_save_config(config).await {
                Ok(saved) => return Ok(saved),
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= MAX_RETRIES {
                        return Err(e);
                    }
                    // Wait before retrying (exponential backoff)
                    let delay = std::time::Duration::from_millis(100 * 2_u64.pow(retry_count - 1));
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(crate::types::AppError::ConfigError(
            "Failed to save config after maximum retries".to_string(),
        ))
    }

    /// Try to persist configuration to database.
    async fn try_save_config(&self, config: &AppConfig) -> AppResult<AppConfig> {
        let mut config_to_save = config.clone();
        config_to_save.app.last_save = Utc::now().to_rfc3339();

        {
            let mut conn = self.database.get_connection().await?;
            Self::persist_to_database(&mut conn, &config_to_save)?;
        }

        Ok(config_to_save)
    }

    /// Get current configuration (read-only)
    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, new_config: AppConfig) -> AppResult<()> {
        let saved = self.save_config(&new_config).await?;
        *self.config.write().await = saved;
        Ok(())
    }

    async fn apply_update<F>(&self, updater: F) -> AppResult<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut working_copy = {
            let guard = self.config.read().await;
            guard.clone()
        };

        updater(&mut working_copy);

        let persisted = self.save_config(&working_copy).await?;
        *self.config.write().await = persisted;
        Ok(())
    }

    /// Update specific configuration section
    pub async fn update_section<F, T>(&self, section_updater: F) -> AppResult<()>
    where
        F: FnOnce(&mut AppConfig) -> &mut T,
    {
        self.apply_update(|config| {
            let _ = section_updater(config);
        })
        .await
    }

    /// Get OBS connections configuration
    pub async fn get_obs_connections(&self) -> Vec<crate::config::ObsConnectionConfig> {
        self.config.read().await.obs.connections.clone()
    }

    /// Update OBS connections
    pub async fn update_obs_connections(
        &self,
        _connections: Vec<crate::config::ObsConnectionConfig>,
    ) -> AppResult<()> {
        self.update_section(|config| &mut config.obs.connections)
            .await?;
        Ok(())
    }

    /// Get UDP settings
    pub async fn get_udp_settings(&self) -> crate::config::UdpSettings {
        self.config.read().await.udp.clone()
    }

    /// Update UDP settings
    pub async fn update_udp_settings(
        &self,
        _udp_settings: crate::config::UdpSettings,
    ) -> AppResult<()> {
        self.update_section(|config| &mut config.udp).await?;
        Ok(())
    }

    /// Update UDP settings from JSON
    pub async fn update_udp_settings_from_json(
        &self,
        settings: serde_json::Value,
    ) -> AppResult<()> {
        self.apply_update(|config| {
            if let Some(udp_data) = settings.get("udp") {
                if let Some(listener_data) = udp_data.get("listener") {
                    if let Some(port) = listener_data.get("port").and_then(|p| p.as_u64()) {
                        config.udp.listener.port = port as u16;
                    }
                    if let Some(bind_address) = listener_data
                        .get("bind_address")
                        .and_then(|value| value.as_str())
                    {
                        config.udp.listener.bind_address = bind_address.to_string();
                    }
                    if let Some(enabled) = listener_data
                        .get("enabled")
                        .and_then(|value| value.as_bool())
                    {
                        config.udp.listener.enabled = enabled;
                    }

                    if let Some(network_data) = listener_data.get("network_interface") {
                        if let Some(auto_detect) = network_data
                            .get("auto_detect")
                            .and_then(|value| value.as_bool())
                        {
                            config.udp.listener.network_interface.auto_detect = auto_detect;
                        }
                        if let Some(preferred_type) = network_data
                            .get("preferred_type")
                            .and_then(|value| value.as_str())
                        {
                            config.udp.listener.network_interface.preferred_type =
                                preferred_type.to_string();
                        }
                        if let Some(fallback_to_localhost) = network_data
                            .get("fallback_to_localhost")
                            .and_then(|value| value.as_bool())
                        {
                            config.udp.listener.network_interface.fallback_to_localhost =
                                fallback_to_localhost;
                        }
                        if let Some(selected_interface) = network_data
                            .get("selected_interface")
                            .and_then(|value| value.as_str())
                        {
                            config.udp.listener.network_interface.selected_interface =
                                Some(selected_interface.to_string());
                        }
                    }
                }
            }
        })
        .await
    }

    /// Get logging settings
    pub async fn get_logging_settings(&self) -> crate::config::LoggingSettings {
        self.config.read().await.logging.clone()
    }

    /// Update logging settings
    pub async fn update_logging_settings(
        &self,
        _logging_settings: crate::config::LoggingSettings,
    ) -> AppResult<()> {
        self.update_section(|config| &mut config.logging).await?;
        Ok(())
    }

    /// Get UI settings
    pub async fn get_ui_settings(&self) -> crate::config::UiSettings {
        self.config.read().await.ui.clone()
    }

    /// Update UI settings
    pub async fn update_ui_settings(
        &self,
        _ui_settings: crate::config::UiSettings,
    ) -> AppResult<()> {
        self.update_section(|config| &mut config.ui).await?;
        Ok(())
    }

    /// Get video settings
    pub async fn get_video_settings(&self) -> crate::config::VideoSettings {
        self.config.read().await.video.clone()
    }

    /// Update video settings
    pub async fn update_video_settings(
        &self,
        _video_settings: crate::config::VideoSettings,
    ) -> AppResult<()> {
        self.update_section(|config| &mut config.video).await?;
        Ok(())
    }

    /// Get license settings
    pub async fn get_license_settings(&self) -> crate::config::LicenseSettings {
        self.config.read().await.license.clone()
    }

    /// Update license settings
    pub async fn update_license_settings(
        &self,
        _license_settings: crate::config::LicenseSettings,
    ) -> AppResult<()> {
        self.update_section(|config| &mut config.license).await?;
        Ok(())
    }

    /// Get flag settings
    pub async fn get_flag_settings(&self) -> crate::config::FlagSettings {
        self.config.read().await.flags.clone()
    }

    /// Update flag settings
    pub async fn update_flag_settings(
        &self,
        _flag_settings: crate::config::FlagSettings,
    ) -> AppResult<()> {
        self.update_section(|config| &mut config.flags).await?;
        Ok(())
    }

    /// Get advanced settings
    pub async fn get_advanced_settings(&self) -> crate::config::AdvancedSettings {
        self.config.read().await.advanced.clone()
    }

    /// Update advanced settings
    pub async fn update_advanced_settings(
        &self,
        _advanced_settings: crate::config::AdvancedSettings,
    ) -> AppResult<()> {
        self.update_section(|config| &mut config.advanced).await?;
        Ok(())
    }

    /// Reset configuration to defaults
    pub async fn reset_to_defaults(&self) -> AppResult<()> {
        let default_config = AppConfig::default();
        self.update_config(default_config).await
    }

    /// Export configuration to file
    pub async fn export_config(&self, export_path: &Path) -> AppResult<()> {
        let config = self.config.read().await;
        let content = serde_json::to_string_pretty(&*config)?;
        fs::write(export_path, content)?;
        Ok(())
    }

    /// Import configuration from file
    pub async fn import_config(&self, import_path: &Path) -> AppResult<()> {
        let content = fs::read_to_string(import_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        self.update_config(config).await
    }

    /// Get configuration file path
    pub fn get_config_path(&self) -> &Path {
        &self.legacy_config_path
    }

    /// Get backup file path
    pub fn get_backup_path(&self) -> &Path {
        &self.legacy_backup_path
    }

    /// Restore configuration from backup
    pub async fn restore_from_backup(&self) -> AppResult<()> {
        let backup_config = {
            let conn = self.database.get_connection().await?;
            Self::load_backup_from_database(&conn)?
        };

        let config = backup_config.ok_or_else(|| {
            crate::types::AppError::ConfigError(
                "No backup configuration stored in database".to_string(),
            )
        })?;

        let saved = self.save_config(&config).await?;
        *self.config.write().await = saved;
        Ok(())
    }

    /// Check if configuration file exists
    pub async fn config_exists(&self) -> bool {
        match self.database.get_connection().await {
            Ok(conn) => matches!(Self::load_from_database(&conn), Ok(Some(_))),
            Err(_) => false,
        }
    }

    /// Check if backup file exists
    pub async fn backup_exists(&self) -> bool {
        match self.database.get_connection().await {
            Ok(conn) => matches!(Self::load_backup_from_database(&conn), Ok(Some(_))),
            Err(_) => false,
        }
    }

    /// Get configuration statistics
    pub async fn get_config_stats(&self) -> AppResult<ConfigStats> {
        let (config_size, obs_connections_count, udp_enabled, logging_enabled, last_save, version) = {
            let guard = self.config.read().await;
            let serialized = serde_json::to_string(&*guard)?;
            (
                serialized.len() as u64,
                guard.obs.connections.len(),
                guard.udp.listener.enabled,
                guard.logging.global.file_enabled,
                guard.app.last_save.clone(),
                guard.app.version.clone(),
            )
        };

        let backup_size = {
            let conn = self.database.get_connection().await?;
            match Self::load_backup_from_database(&conn)? {
                Some(backup) => serde_json::to_string(&backup)?.len() as u64,
                None => 0,
            }
        };

        Ok(ConfigStats {
            config_file_size: config_size,
            backup_file_size: backup_size,
            obs_connections_count,
            udp_enabled,
            logging_enabled,
            last_save,
            version,
        })
    }
}

/// Configuration statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigStats {
    pub config_file_size: u64,
    pub backup_file_size: u64,
    pub obs_connections_count: usize,
    pub udp_enabled: bool,
    pub logging_enabled: bool,
    pub last_save: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = ConfigManager::new(temp_dir.path()).await.unwrap();

        assert!(manager.config_exists().await);
        assert_eq!(
            manager.get_config_path().file_name().unwrap(),
            "app_config.json"
        );
    }

    #[tokio::test]
    async fn test_config_persistence() {
        let temp_dir = tempdir().unwrap();
        let manager = ConfigManager::new(temp_dir.path()).await.unwrap();

        // Get initial config
        let initial_config = manager.get_config().await;

        // Update a setting
        let mut new_config = initial_config.clone();
        new_config.ui.overlay.opacity = 0.5;

        manager.update_config(new_config).await.unwrap();

        // Verify the change was persisted
        let updated_config = manager.get_config().await;
        assert_eq!(updated_config.ui.overlay.opacity, 0.5);
    }

    #[tokio::test]
    async fn test_config_backup() {
        let temp_dir = tempdir().unwrap();
        let manager = ConfigManager::new(temp_dir.path()).await.unwrap();

        // Initial save should create backup
        let config = manager.get_config().await;
        let mut new_config = config.clone();
        new_config.ui.overlay.opacity = 0.3;

        manager.update_config(new_config).await.unwrap();

        // Should have backup file
        assert!(manager.backup_exists().await);
    }
}
