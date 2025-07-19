use crate::config::AppConfig;
use crate::types::AppResult;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

/// Configuration manager for handling application settings
pub struct ConfigManager {
    /// Current configuration
    config: Arc<RwLock<AppConfig>>,
    /// Configuration file path
    config_path: PathBuf,
    /// Backup configuration file path
    backup_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_dir: &Path) -> AppResult<Self> {
        // Ensure config directory exists
        fs::create_dir_all(config_dir)?;
        
        let config_path = config_dir.join("app_config.json");
        let backup_path = config_dir.join("app_config.backup.json");
        
        // Try to load existing configuration or create default
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            AppConfig::default()
        };
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            backup_path,
        })
    }
    
    /// Load configuration from file
    fn load_config(config_path: &Path) -> AppResult<AppConfig> {
        let content = fs::read_to_string(config_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to file
    async fn save_config(&self, config: &AppConfig) -> AppResult<()> {
        // Create backup of current config if it exists
        if self.config_path.exists() {
            fs::copy(&self.config_path, &self.backup_path)?;
        }
        
        // Update last save timestamp
        let mut config_to_save = config.clone();
        config_to_save.app.last_save = Utc::now().to_rfc3339();
        
        // Serialize and save configuration
        let content = serde_json::to_string_pretty(&config_to_save)?;
        fs::write(&self.config_path, content)?;
        
        Ok(())
    }
    
    /// Get current configuration (read-only)
    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }
    
    /// Update configuration
    pub async fn update_config(&self, new_config: AppConfig) -> AppResult<()> {
        // Save to file
        self.save_config(&new_config).await?;
        
        // Update in memory
        *self.config.write().await = new_config;
        
        Ok(())
    }
    
    /// Update specific configuration section
    pub async fn update_section<F, T>(&self, section_updater: F) -> AppResult<()>
    where
        F: FnOnce(&mut AppConfig) -> &mut T,
    {
        let mut config = self.config.write().await;
        section_updater(&mut config);
        
        // Save to file
        self.save_config(&config).await?;
        
        Ok(())
    }
    
    /// Get OBS connections configuration
    pub async fn get_obs_connections(&self) -> Vec<crate::config::ObsConnectionConfig> {
        self.config.read().await.obs.connections.clone()
    }
    
    /// Update OBS connections
    pub async fn update_obs_connections(&self, _connections: Vec<crate::config::ObsConnectionConfig>) -> AppResult<()> {
        self.update_section(|config| &mut config.obs.connections).await?;
        Ok(())
    }
    
    /// Get UDP settings
    pub async fn get_udp_settings(&self) -> crate::config::UdpSettings {
        self.config.read().await.udp.clone()
    }
    
    /// Update UDP settings
    pub async fn update_udp_settings(&self, _udp_settings: crate::config::UdpSettings) -> AppResult<()> {
        self.update_section(|config| &mut config.udp).await?;
        Ok(())
    }
    
    /// Get logging settings
    pub async fn get_logging_settings(&self) -> crate::config::LoggingSettings {
        self.config.read().await.logging.clone()
    }
    
    /// Update logging settings
    pub async fn update_logging_settings(&self, _logging_settings: crate::config::LoggingSettings) -> AppResult<()> {
        self.update_section(|config| &mut config.logging).await?;
        Ok(())
    }
    
    /// Get UI settings
    pub async fn get_ui_settings(&self) -> crate::config::UiSettings {
        self.config.read().await.ui.clone()
    }
    
    /// Update UI settings
    pub async fn update_ui_settings(&self, _ui_settings: crate::config::UiSettings) -> AppResult<()> {
        self.update_section(|config| &mut config.ui).await?;
        Ok(())
    }
    
    /// Get video settings
    pub async fn get_video_settings(&self) -> crate::config::VideoSettings {
        self.config.read().await.video.clone()
    }
    
    /// Update video settings
    pub async fn update_video_settings(&self, _video_settings: crate::config::VideoSettings) -> AppResult<()> {
        self.update_section(|config| &mut config.video).await?;
        Ok(())
    }
    
    /// Get license settings
    pub async fn get_license_settings(&self) -> crate::config::LicenseSettings {
        self.config.read().await.license.clone()
    }
    
    /// Update license settings
    pub async fn update_license_settings(&self, _license_settings: crate::config::LicenseSettings) -> AppResult<()> {
        self.update_section(|config| &mut config.license).await?;
        Ok(())
    }
    
    /// Get flag settings
    pub async fn get_flag_settings(&self) -> crate::config::FlagSettings {
        self.config.read().await.flags.clone()
    }
    
    /// Update flag settings
    pub async fn update_flag_settings(&self, _flag_settings: crate::config::FlagSettings) -> AppResult<()> {
        self.update_section(|config| &mut config.flags).await?;
        Ok(())
    }
    
    /// Get advanced settings
    pub async fn get_advanced_settings(&self) -> crate::config::AdvancedSettings {
        self.config.read().await.advanced.clone()
    }
    
    /// Update advanced settings
    pub async fn update_advanced_settings(&self, _advanced_settings: crate::config::AdvancedSettings) -> AppResult<()> {
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
        &self.config_path
    }
    
    /// Get backup file path
    pub fn get_backup_path(&self) -> &Path {
        &self.backup_path
    }
    
    /// Restore configuration from backup
    pub async fn restore_from_backup(&self) -> AppResult<()> {
        if !self.backup_path.exists() {
            return Err(crate::types::AppError::ConfigError("No backup file found".to_string()));
        }
        
        let config = Self::load_config(&self.backup_path)?;
        self.update_config(config).await
    }
    
    /// Check if configuration file exists
    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }
    
    /// Check if backup file exists
    pub fn backup_exists(&self) -> bool {
        self.backup_path.exists()
    }
    
    /// Get configuration statistics
    pub async fn get_config_stats(&self) -> AppResult<ConfigStats> {
        let config = self.config.read().await;
        
        let stats = ConfigStats {
            config_file_size: if self.config_path.exists() {
                fs::metadata(&self.config_path)?.len()
            } else {
                0
            },
            backup_file_size: if self.backup_path.exists() {
                fs::metadata(&self.backup_path)?.len()
            } else {
                0
            },
            obs_connections_count: config.obs.connections.len(),
            udp_enabled: config.udp.listener.enabled,
            logging_enabled: config.logging.global.file_enabled,
            last_save: config.app.last_save.clone(),
            version: config.app.version.clone(),
        };
        
        Ok(stats)
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
        let manager = ConfigManager::new(temp_dir.path()).unwrap();
        
        assert!(manager.config_exists());
        assert_eq!(manager.get_config_path().file_name().unwrap(), "app_config.json");
    }
    
    #[tokio::test]
    async fn test_config_persistence() {
        let temp_dir = tempdir().unwrap();
        let manager = ConfigManager::new(temp_dir.path()).unwrap();
        
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
        let manager = ConfigManager::new(temp_dir.path()).unwrap();
        
        // Initial save should create backup
        let config = manager.get_config().await;
        let mut new_config = config.clone();
        new_config.ui.overlay.opacity = 0.3;
        
        manager.update_config(new_config).await.unwrap();
        
        // Should have backup file
        assert!(manager.backup_exists());
    }
} 