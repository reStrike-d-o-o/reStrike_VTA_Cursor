// OBS Plugin Manager
// Manages all modular OBS plugins and provides unified API
// Extracted from the original plugin_obs.rs

use crate::types::AppResult;
use super::types::*;
use super::core::ObsCorePlugin;
use super::recording::ObsRecordingPlugin;
use super::streaming::ObsStreamingPlugin;
use super::scenes::ObsScenesPlugin;
use super::settings::ObsSettingsPlugin;
use super::events::ObsEventsPlugin;
use super::status::ObsStatusPlugin;
use std::sync::Arc;

/// Manager for all OBS plugins
#[derive(Clone)]
pub struct ObsPluginManager {
    context: ObsPluginContext,
    core_plugin: Arc<ObsCorePlugin>,
    recording_plugin: Arc<ObsRecordingPlugin>,
    streaming_plugin: Arc<ObsStreamingPlugin>,
    scenes_plugin: Arc<ObsScenesPlugin>,
    settings_plugin: Arc<ObsSettingsPlugin>,
    events_plugin: Arc<ObsEventsPlugin>,
    status_plugin: Arc<ObsStatusPlugin>,
}

impl ObsPluginManager {
    /// Create a new OBS Plugin Manager
    pub fn new() -> AppResult<Self> {
        log::info!("🔧 Creating OBS Plugin Manager...");
        
        // Create shared context
        let mut context = ObsPluginContext::new()?;
        
        // Create core plugin first (others depend on it)
        let core_plugin = Arc::new(ObsCorePlugin::new(context.clone()));
        
        // Set the core plugin in the context for other plugins to use
        context.core_plugin = Some(core_plugin.clone());
        
        // Create other plugins with dependencies
        let recording_plugin = Arc::new(ObsRecordingPlugin::new(context.clone(), core_plugin.clone()));
        let streaming_plugin = Arc::new(ObsStreamingPlugin::new(context.clone(), core_plugin.clone()));
        let scenes_plugin = Arc::new(ObsScenesPlugin::new(context.clone()));
        let settings_plugin = Arc::new(ObsSettingsPlugin::new(context.clone()));
        let events_plugin = Arc::new(ObsEventsPlugin::new(context.clone()));
        let status_plugin = Arc::new(ObsStatusPlugin::new(
            context.clone(), 
            recording_plugin.clone(), 
            streaming_plugin.clone()
        ));

        Ok(Self {
            context,
            core_plugin,
            recording_plugin,
            streaming_plugin,
            scenes_plugin,
            settings_plugin,
            events_plugin,
            status_plugin,
        })
    }

    /// Initialize all plugins
    pub async fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Plugin Manager...");
        
        // Initialize all plugins
        self.core_plugin.init()?;
        self.recording_plugin.init()?;
        self.streaming_plugin.init()?;
        self.scenes_plugin.init()?;
        self.settings_plugin.init()?;
        self.events_plugin.init()?;
        self.status_plugin.init()?;
        
        log::info!("✅ OBS Plugin Manager initialized successfully");
        Ok(())
    }

    /// Shutdown all plugins
    pub async fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Plugin Manager...");
        
        // Shutdown all plugins in reverse order
        self.status_plugin.shutdown()?;
        self.events_plugin.shutdown()?;
        self.settings_plugin.shutdown()?;
        self.scenes_plugin.shutdown()?;
        self.streaming_plugin.shutdown()?;
        self.recording_plugin.shutdown()?;
        self.core_plugin.shutdown()?;
        
        log::info!("✅ OBS Plugin Manager shut down successfully");
        Ok(())
    }

    // Core plugin methods
    pub async fn connect_obs(&self, connection_name: &str) -> AppResult<()> {
        self.core_plugin.connect_obs(connection_name).await
    }

    pub async fn disconnect_obs(&self, connection_name: &str) -> AppResult<()> {
        self.core_plugin.disconnect_obs(connection_name).await
    }

    pub async fn send_request(&self, connection_name: &str, request_type: &str, request_data: serde_json::Value) -> AppResult<serde_json::Value> {
        self.core_plugin.send_request(connection_name, request_type, Some(request_data)).await
    }

    // Recording plugin methods
    pub async fn start_recording(&self, connection_name: &str) -> AppResult<()> {
        self.recording_plugin.start_recording(connection_name).await
    }

    pub async fn stop_recording(&self, connection_name: &str) -> AppResult<()> {
        self.recording_plugin.stop_recording(connection_name).await
    }

    pub async fn get_recording_status(&self, connection_name: &str) -> AppResult<bool> {
        self.recording_plugin.get_recording_status(connection_name).await
    }

    // Streaming plugin methods
    pub async fn start_streaming(&self, connection_name: &str) -> AppResult<()> {
        self.streaming_plugin.start_streaming(connection_name).await
    }

    pub async fn stop_streaming(&self, connection_name: &str) -> AppResult<()> {
        self.streaming_plugin.stop_streaming(connection_name).await
    }

    pub async fn get_streaming_status(&self, connection_name: &str) -> AppResult<bool> {
        self.streaming_plugin.get_streaming_status(connection_name).await
    }

    // Status plugin methods
    pub async fn get_obs_status(&self) -> AppResult<ObsStatusInfo> {
        self.status_plugin.get_obs_status().await
    }

    pub async fn get_connection_status(&self, connection_name: &str) -> AppResult<ObsConnectionStatus> {
        self.status_plugin.get_connection_status(connection_name).await
    }

    // Events plugin methods
    pub async fn get_recent_events(&self) -> AppResult<Vec<RecentEvent>> {
        // TODO: Implement this when the events plugin is complete
        Ok(Vec::new())
    }

    // Settings plugin methods
    pub async fn get_obs_settings(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        // Get OBS version and current profile
        let version = self.settings_plugin.get_obs_version(connection_name).await?;
        let profile = self.settings_plugin.get_current_profile(connection_name).await?;
        let profiles = self.settings_plugin.get_profiles(connection_name).await?;
        
        Ok(serde_json::json!({
            "version": version,
            "current_profile": profile,
            "profiles": profiles
        }))
    }

    pub async fn set_obs_settings(&self, connection_name: &str, settings: serde_json::Value) -> AppResult<()> {
        // Handle different setting types
        if let Some(profile_name) = settings.get("profile_name").and_then(|v| v.as_str()) {
            self.settings_plugin.set_current_profile(connection_name, profile_name).await?;
        }
        
        Ok(())
    }

    // Settings plugin methods
    pub async fn get_obs_version(&self, connection_name: &str) -> AppResult<String> {
        self.settings_plugin.get_obs_version(connection_name).await
    }

    pub async fn get_current_profile(&self, connection_name: &str) -> AppResult<String> {
        self.settings_plugin.get_current_profile(connection_name).await
    }

    pub async fn set_current_profile(&self, connection_name: &str, profile_name: &str) -> AppResult<()> {
        self.settings_plugin.set_current_profile(connection_name, profile_name).await
    }

    pub async fn get_profiles(&self, connection_name: &str) -> AppResult<Vec<String>> {
        self.settings_plugin.get_profiles(connection_name).await
    }

    // Scenes plugin methods
    pub async fn get_scenes(&self, connection_name: &str) -> AppResult<Vec<String>> {
        self.scenes_plugin.get_scenes(connection_name).await
    }

    pub async fn set_current_scene(&self, connection_name: &str, scene_name: &str) -> AppResult<()> {
        self.scenes_plugin.set_current_scene(connection_name, scene_name).await
    }

    pub async fn get_current_scene(&self, connection_name: &str) -> AppResult<String> {
        self.scenes_plugin.get_current_scene(connection_name).await
    }

    // Additional methods for Tauri commands compatibility
    pub async fn add_connection(&self, config: ObsConnectionConfig) -> AppResult<()> {
        self.core_plugin.add_connection(config).await
    }

    pub async fn get_connection_names(&self) -> Vec<String> {
        self.core_plugin.get_connection_names().await
    }

    /// Load connections from config manager
    pub async fn load_connections_from_config(&self, config_connections: Vec<crate::config::ObsConnectionConfig>) -> AppResult<()> {
        log::info!("[OBS_MANAGER] Loading {} connections from config", config_connections.len());
        self.core_plugin.load_connections_from_config(config_connections).await
    }

    pub async fn remove_connection(&self, connection_name: &str) -> AppResult<()> {
        self.core_plugin.remove_connection(connection_name).await
    }

    pub async fn get_latest_events(&self, _connection_name: &str) -> AppResult<Vec<RecentEvent>> {
        // TODO: Implement latest events
        Ok(Vec::new())
    }

    pub async fn toggle_full_events(&self, enabled: bool) -> AppResult<()> {
        log::info!("[OBS_MANAGER] Toggling full events: {}", enabled);
        
        // Delegate to events plugin to handle full events toggle
        self.events_plugin.set_show_full_events(enabled).await;
        
        // Also update the context setting for persistence
        {
            let mut context = self.context.show_full_events.lock().await;
            *context = enabled;
        }
        
        log::info!("[OBS_MANAGER] Full events toggle completed: {}", enabled);
        Ok(())
    }

    pub async fn get_full_events_setting(&self) -> AppResult<bool> {
        // Get the current setting from context
        let setting = self.context.show_full_events.lock().await;
        Ok(*setting)
    }

    // Context access
    pub fn get_context(&self) -> &ObsPluginContext {
        &self.context
    }

    // Individual plugin access
    pub fn core(&self) -> &Arc<ObsCorePlugin> {
        &self.core_plugin
    }

    pub fn recording(&self) -> &Arc<ObsRecordingPlugin> {
        &self.recording_plugin
    }

    pub fn streaming(&self) -> &Arc<ObsStreamingPlugin> {
        &self.streaming_plugin
    }

    pub fn status(&self) -> &Arc<ObsStatusPlugin> {
        &self.status_plugin
    }

    pub fn events(&self) -> &Arc<ObsEventsPlugin> {
        &self.events_plugin
    }

    pub fn settings(&self) -> &Arc<ObsSettingsPlugin> {
        &self.settings_plugin
    }

    pub fn scenes(&self) -> &Arc<ObsScenesPlugin> {
        &self.scenes_plugin
    }
} 