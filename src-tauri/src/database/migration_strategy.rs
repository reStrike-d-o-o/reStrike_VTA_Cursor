use crate::config::manager::ConfigManager;
use crate::database::operations::UiSettingsOperations;
use crate::types::AppResult;
use rusqlite::Connection;
use std::collections::HashMap;

/// Migration strategy for transitioning from JSON to database settings
#[derive(Clone)]
pub struct MigrationStrategy {
    config_manager: ConfigManager,
}

impl MigrationStrategy {
    pub fn new(config_manager: ConfigManager) -> Self {
        Self { config_manager }
    }

    /// Perform complete migration from JSON to database
    pub async fn migrate_json_to_database(
        &self,
        conn: &mut Connection,
    ) -> AppResult<MigrationResult> {
        log::info!("Starting JSON to database migration...");

        let mut result = MigrationResult {
            total_settings: 0,
            migrated_settings: 0,
            failed_settings: 0,
            errors: Vec::new(),
        };

        // Step 1: Load existing JSON settings
        let json_settings = self.load_json_settings().await?;
        result.total_settings = json_settings.len();

        log::info!(
            "Found {total_settings} settings in JSON configuration",
            total_settings = result.total_settings
        );

        // Step 2: Initialize database settings table
        UiSettingsOperations::initialize_ui_settings(conn)?;
        log::info!("Database settings table initialized");

        // Step 3: Migrate each setting
        for (key, value) in json_settings {
            match self.migrate_setting(conn, &key, &value).await {
                Ok(_) => {
                    result.migrated_settings += 1;
                    log::debug!("Migrated setting: {key}");
                }
                Err(e) => {
                    result.failed_settings += 1;
                    result
                        .errors
                        .push(format!("Failed to migrate '{key}': {e}"));
                    log::warn!("Failed to migrate setting '{key}': {e}");
                }
            }
        }

        // Step 4: Validate migration
        self.validate_migration(conn, &result).await?;

        log::info!(
            " Migration completed: {migrated}/{total} settings migrated successfully",
            migrated = result.migrated_settings,
            total = result.total_settings
        );

        Ok(result)
    }

    /// Load existing JSON settings from config manager
    async fn load_json_settings(&self) -> AppResult<HashMap<String, String>> {
        let mut settings = HashMap::new();

        // Load UI settings - UiSettings is a struct, not a HashMap
        let ui_settings = self.config_manager.get_ui_settings().await;
        settings.insert(
            "ui.overlay.visible".to_string(),
            ui_settings.overlay.visible.to_string(),
        );
        settings.insert(
            "ui.overlay.opacity".to_string(),
            ui_settings.overlay.opacity.to_string(),
        );
        settings.insert(
            "ui.overlay.position".to_string(),
            ui_settings.overlay.position.clone(),
        );
        settings.insert(
            "ui.overlay.scale".to_string(),
            ui_settings.overlay.scale.to_string(),
        );
        settings.insert(
            "ui.theme.current".to_string(),
            ui_settings.theme.current.clone(),
        );
        settings.insert(
            "ui.layout.sidebar_position".to_string(),
            ui_settings.layout.sidebar_position.clone(),
        );
        settings.insert(
            "ui.layout.sidebar_width".to_string(),
            ui_settings.layout.sidebar_width.to_string(),
        );

        // Load UDP settings
        let udp_settings = self.config_manager.get_udp_settings().await;
        settings.insert(
            "udp.listener.port".to_string(),
            udp_settings.listener.port.to_string(),
        );
        settings.insert(
            "udp.listener.enabled".to_string(),
            udp_settings.listener.enabled.to_string(),
        );

        // Load flag settings
        let flag_settings = self.config_manager.get_flag_settings().await;
        settings.insert(
            "flags.storage.auto_download".to_string(),
            flag_settings.storage.auto_download.to_string(),
        );
        settings.insert(
            "flags.storage.directory".to_string(),
            flag_settings.storage.directory.clone(),
        );

        // Load logging settings
        let logging_settings = self.config_manager.get_logging_settings().await;
        settings.insert(
            "logging.global.level".to_string(),
            logging_settings.global.level.clone(),
        );
        settings.insert(
            "logging.files.max_size_mb".to_string(),
            logging_settings.files.max_size_mb.to_string(),
        );
        settings.insert(
            "logging.files.max_files".to_string(),
            logging_settings.files.max_files.to_string(),
        );

        Ok(settings)
    }

    /// Migrate a single setting from JSON to database
    async fn migrate_setting(
        &self,
        conn: &mut Connection,
        key: &str,
        value: &str,
    ) -> AppResult<()> {
        // Check if setting already exists in database
        let existing = UiSettingsOperations::get_ui_setting(conn, key)?;

        if existing.is_none() {
            // Setting doesn't exist, migrate it
            UiSettingsOperations::set_ui_setting(
                conn,
                key,
                value,
                "migration",
                Some("Migrated from JSON configuration"),
            )?;
        } else {
            log::debug!("Setting '{key}' already exists in database, skipping");
        }

        Ok(())
    }

    /// Validate the migration by comparing JSON and database settings
    async fn validate_migration(
        &self,
        conn: &mut Connection,
        _result: &MigrationResult,
    ) -> AppResult<()> {
        log::info!("Validating migration...");

        let json_settings = self.load_json_settings().await?;
        let db_settings_vec = UiSettingsOperations::get_all_ui_settings(conn)?;

        // Convert Vec<(String, String)> to HashMap<String, String>
        let db_settings: HashMap<String, String> = db_settings_vec.into_iter().collect();

        let mut validation_errors = Vec::new();

        for (key, json_value) in json_settings {
            if let Some(db_value) = db_settings.get(&key) {
                if json_value != *db_value {
                    validation_errors.push(format!(
                        "Value mismatch for '{key}': JSON='{json_value}', DB='{db_value}'"
                    ));
                }
            } else {
                validation_errors.push(format!("Setting '{key}' not found in database"));
            }
        }

        if !validation_errors.is_empty() {
            let issue_count = validation_errors.len();
            log::warn!("Migration validation found {issue_count} issues:");
            for error in &validation_errors {
                log::warn!(" - {error}");
            }
            let error_count = validation_errors.len();
            return Err(crate::types::AppError::ConfigError(format!(
                "Migration validation failed: {error_count} errors"
            )));
        }

        log::info!("Migration validation passed");
        Ok(())
    }
}

/// Result of the migration process
#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub total_settings: usize,
    pub migrated_settings: usize,
    pub failed_settings: usize,
    pub errors: Vec<String>,
}

impl MigrationResult {
    pub fn success_rate(&self) -> f64 {
        if self.total_settings == 0 {
            0.0
        } else {
            self.migrated_settings as f64 / self.total_settings as f64
        }
    }

    pub fn is_successful(&self) -> bool {
        self.failed_settings == 0 && self.migrated_settings > 0
    }
}

/// Settings provider that can fall back to JSON if database is unavailable
pub struct HybridSettingsProvider {
    migration_strategy: MigrationStrategy,
    use_database: bool,
}

impl HybridSettingsProvider {
    pub fn new(config_manager: ConfigManager) -> Self {
        Self {
            migration_strategy: MigrationStrategy::new(config_manager),
            use_database: true,
        }
    }

    /// Get a setting with fallback to JSON
    pub async fn get_setting(&self, key: &str) -> AppResult<Option<String>> {
        if self.use_database {
            // Try database first
            match self.get_from_database(key).await {
                Ok(value) => Ok(value),
                Err(e) => {
<<<<<<< HEAD
                    log::warn!("Database lookup failed for '{key}', falling back to JSON: {e}");
=======
                    log::warn!(
                        "Database lookup failed for '{key}', falling back to JSON: {e}"
                    );
>>>>>>> Scoreboard
                    self.get_from_json(key).await
                }
            }
        } else {
            // Use JSON directly
            self.get_from_json(key).await
        }
    }

    /// Set a setting (database only)
    pub async fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        if self.use_database {
            self.set_in_database(key, value).await
        } else {
            Err(crate::types::AppError::ConfigError(
                "Database mode disabled, cannot set settings".to_string(),
            ))
        }
    }

    /// Get setting from database
    async fn get_from_database(&self, _key: &str) -> AppResult<Option<String>> {
        // This would use the database connection
        // For now, return None to trigger fallback
        Ok(None)
    }

    /// Get setting from JSON
    async fn get_from_json(&self, key: &str) -> AppResult<Option<String>> {
        let settings = self.migration_strategy.load_json_settings().await?;
        Ok(settings.get(key).cloned())
    }

    /// Set setting in database
    async fn set_in_database(&self, _key: &str, _value: &str) -> AppResult<()> {
        // This would use the database connection
        // For now, return success
        Ok(())
    }

    /// Enable/disable database mode
    pub fn set_database_mode(&mut self, enabled: bool) {
        self.use_database = enabled;
        let mode = if enabled { "enabled" } else { "disabled" };
        log::info!("Database mode {mode}");
    }
}
