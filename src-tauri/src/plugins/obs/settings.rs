// OBS Settings Plugin
// Handles OBS Studio settings, profile management, and output settings
// Extracted from the original plugin_obs.rs

use crate::types::{AppError, AppResult};
use super::types::*;

/// OBS Settings Plugin for settings management
pub struct ObsSettingsPlugin {
    context: ObsPluginContext,
}

impl ObsSettingsPlugin {
    /// Create a new OBS Settings Plugin
    pub fn new(context: ObsPluginContext) -> Self {
        Self { context }
    }

    /// Get OBS Studio version
    pub async fn get_obs_version(&self, connection_name: &str) -> AppResult<String> {
        log::debug!("[OBS_SETTINGS] get_obs_version called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetVersion", None).await?;
        
        // Parse the response to get OBS version
        if let Some(version) = response.get("obsVersion") {
            if let Some(ver) = version.as_str() {
                log::debug!("[OBS_SETTINGS] OBS version for '{}': {}", connection_name, ver);
                return Ok(ver.to_string());
            }
        }
        
        Err(AppError::ConfigError("Failed to get OBS version".to_string()))
    }

    /// Get current profile
    pub async fn get_current_profile(&self, connection_name: &str) -> AppResult<String> {
        log::debug!("[OBS_SETTINGS] get_current_profile called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetCurrentProfile", None).await?;
        
        // Parse the response to get current profile name
        if let Some(profile_name) = response.get("profileName") {
            if let Some(name) = profile_name.as_str() {
                log::debug!("[OBS_SETTINGS] Current profile for '{}': {}", connection_name, name);
                return Ok(name.to_string());
            }
        }
        
        Err(AppError::ConfigError("Failed to get current profile".to_string()))
    }

    /// Set current profile
    pub async fn set_current_profile(&self, connection_name: &str, profile_name: &str) -> AppResult<()> {
        log::info!("[OBS_SETTINGS] set_current_profile called for '{}' to '{}'", connection_name, profile_name);
        
        let request_data = serde_json::json!({
            "profileName": profile_name
        });
        
        let response = self.send_settings_request(connection_name, "SetCurrentProfile", Some(request_data)).await?;
        
        log::info!("[OBS_SETTINGS] Profile changed for '{}' to '{}'", connection_name, profile_name);
        Ok(())
    }

    /// Get all profiles
    pub async fn get_profiles(&self, connection_name: &str) -> AppResult<Vec<String>> {
        log::debug!("[OBS_SETTINGS] get_profiles called for '{}'", connection_name);
        
        let response = self.send_settings_request(connection_name, "GetProfileList", None).await?;
        
        // Parse the response to get profile names
        if let Some(profiles) = response.get("profiles") {
            if let Some(profiles_array) = profiles.as_array() {
                let profile_names: Vec<String> = profiles_array
                    .iter()
                    .filter_map(|profile| {
                        profile.get("profileName")?.as_str().map(|s| s.to_string())
                    })
                    .collect();
                
                log::debug!("[OBS_SETTINGS] Found {} profiles for '{}'", profile_names.len(), connection_name);
                return Ok(profile_names);
            }
        }
        
        log::warn!("[OBS_SETTINGS] Failed to parse profiles response");
        Ok(Vec::new())
    }

    /// Send a settings-related request to OBS
    async fn send_settings_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        // This will be implemented when we integrate with the core plugin
        // For now, this is a placeholder that will be replaced with actual implementation
        log::debug!("[OBS_SETTINGS] Sending request '{}' to '{}'", request_type, connection_name);
        
        // TODO: Integrate with core plugin's send_request method
        // This is a placeholder response
        match request_type {
            "GetVersion" => Ok(serde_json::json!({
                "obsVersion": "30.0.0",
                "obsWebSocketVersion": "5.0.0"
            })),
            "GetCurrentProfile" => Ok(serde_json::json!({
                "profileName": "Default"
            })),
            "GetProfileList" => Ok(serde_json::json!({
                "profiles": [
                    {"profileName": "Default"},
                    {"profileName": "Profile 2"},
                    {"profileName": "Profile 3"}
                ]
            })),
            _ => Ok(serde_json::json!({}))
        }
    }
}

// Implement ObsPlugin trait for the settings plugin
impl ObsPlugin for ObsSettingsPlugin {
    fn name(&self) -> &str {
        "obs_settings"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Settings Plugin");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Settings Plugin");
        Ok(())
    }
} 