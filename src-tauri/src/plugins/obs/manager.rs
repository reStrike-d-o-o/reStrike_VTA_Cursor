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
use super::control_room_async::AsyncControlRoomManager;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    control_room_manager: Arc<Mutex<Option<AsyncControlRoomManager>>>,
}

impl ObsPluginManager {
    /// Create a new OBS Plugin Manager
    pub fn new() -> AppResult<Self> {
        log::info!("🔧 Creating OBS Plugin Manager...");
        
        // Create shared context
        let mut context = ObsPluginContext::new()?;
        
        // Create core plugin first (others depend on it)
        let mut core_plugin = ObsCorePlugin::new(context.clone());
        
        // Create events plugin first so we can set it in core plugin
        let events_plugin = Arc::new(ObsEventsPlugin::new(context.clone()));
        
        // Set the events plugin in the core plugin for event processing
        core_plugin.set_events_plugin(events_plugin.clone());
        
        // Wrap core plugin in Arc
        let core_plugin = Arc::new(core_plugin);
        
        // Set the core plugin in the context for other plugins to use
        context.core_plugin = Some(core_plugin.clone());
        
        // Create other plugins with dependencies
        let recording_plugin = Arc::new(ObsRecordingPlugin::new(context.clone(), core_plugin.clone()));
        let streaming_plugin = Arc::new(ObsStreamingPlugin::new(context.clone(), core_plugin.clone()));
        let scenes_plugin = Arc::new(ObsScenesPlugin::new(context.clone()));
        let settings_plugin = Arc::new(ObsSettingsPlugin::new(context.clone()));
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
            control_room_manager: Arc::new(Mutex::new(None)),
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

    // Control Room methods - uses dedicated Control Room Manager
    /// Initialize Control Room with authentication
    pub async fn control_room_initialize(&self, master_password: String, database: Arc<crate::database::AsyncDatabaseConnection>) -> AppResult<()> {
        let control_room = AsyncControlRoomManager::new(master_password, database, self.core_plugin.clone()).await?;
        let mut manager = self.control_room_manager.lock().await;
        *manager = Some(control_room);
        log::info!("[OBS_MANAGER] Control Room initialized successfully");
        Ok(())
    }
    
    /// Check if Control Room is initialized
    pub async fn control_room_is_initialized(&self) -> bool {
        let manager = self.control_room_manager.lock().await;
        manager.is_some()
    }
    
    /// Add Control Room STR connection
    pub async fn control_room_add_connection(&self, config: super::control_room_async::ControlRoomConnection) -> AppResult<()> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.add_connection(config).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }
    
    /// Remove Control Room STR connection
    pub async fn control_room_remove_connection(&self, name: &str) -> AppResult<()> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.remove_connection(name).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }

    /// Get Control Room OBS connection configuration
    pub async fn control_room_get_connection(&self, name: &str) -> AppResult<super::control_room_async::ControlRoomConnection> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.get_connection(name).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }

    /// Update Control Room OBS connection configuration
    pub async fn control_room_update_connection(&self, name: &str, config: super::control_room_async::ControlRoomConnection) -> AppResult<()> {
        let mut manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_mut() {
            control_room.update_connection(name, config).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }

    /// Connect all disconnected Control Room OBS connections
    pub async fn control_room_connect_all_obs(&self) -> AppResult<Vec<(String, AppResult<()>)>> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.connect_all_obs().await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }

    /// Disconnect all connected Control Room OBS connections
    pub async fn control_room_disconnect_all_obs(&self) -> AppResult<Vec<(String, AppResult<()>)>> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.disconnect_all_obs().await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }
    
    /// Get all Control Room STR connections
    pub async fn control_room_get_obs_connections(&self) -> AppResult<Vec<String>> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            Ok(control_room.get_connection_names().await)
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }

    /// Get all Control Room OBS connections with their status
    pub async fn control_room_get_obs_connections_with_status(&self) -> AppResult<Vec<(String, super::control_room_async::ControlRoomStatus)>> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.get_all_connections().await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }

    /// Get all Control Room OBS connections with their full details and status
    pub async fn control_room_get_obs_connections_with_details(&self) -> AppResult<Vec<(String, super::control_room_async::ControlRoomConnection, super::control_room_async::ControlRoomStatus)>> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.get_all_connections_with_details().await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }
    
    /// Control Room: Get OBS connection name for a Control Room connection
    pub async fn control_room_get_obs_connection_name(&self, obs_name: &str) -> AppResult<String> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.get_obs_connection_name(obs_name).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }
    
    /// Connect to Control Room OBS instance
    pub async fn control_room_connect_obs(&self, name: &str) -> AppResult<()> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.connect_obs(name).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }
    
    /// Disconnect from Control Room OBS instance  
    pub async fn control_room_disconnect_obs(&self, name: &str) -> AppResult<()> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.disconnect_obs(name).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }
    
    /// Control Room: Mute audio for STR connection
    pub async fn control_room_mute_audio(&self, str_name: &str, source_name: &str) -> AppResult<()> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            let obs_name = control_room.get_obs_connection_name(str_name).await?;
            drop(manager); // Release lock before calling streaming plugin
            self.streaming_plugin.mute_audio_source(&obs_name, source_name).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }
    
    /// Control Room: Unmute audio for STR connection
    pub async fn control_room_unmute_audio(&self, str_name: &str, source_name: &str) -> AppResult<()> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            let obs_name = control_room.get_obs_connection_name(str_name).await?;
            drop(manager); // Release lock before calling streaming plugin
            self.streaming_plugin.unmute_audio_source(&obs_name, source_name).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }
    
    /// Control Room: Get audio sources for STR connection
    pub async fn control_room_get_audio_sources(&self, str_name: &str) -> AppResult<Vec<String>> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            let obs_name = control_room.get_obs_connection_name(str_name).await?;
            drop(manager); // Release lock before calling streaming plugin
            self.streaming_plugin.get_audio_sources(&obs_name).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }
    
    /// Control Room: Bulk mute all OBS connections
    pub async fn control_room_mute_all_obs(&self, source_name: &str) -> AppResult<Vec<(String, AppResult<()>)>> {
        let obs_connections = self.control_room_get_obs_connections().await?;
        let mut results = Vec::new();
        
        for obs_name in obs_connections {
            let result = self.control_room_mute_audio(&obs_name, source_name).await;
            results.push((obs_name, result));
        }
        
        log::info!("[OBS_MANAGER] Control Room: Bulk mute operation completed for {} connections", results.len());
        Ok(results)
    }
    
    /// Control Room: Bulk unmute all OBS connections
    pub async fn control_room_unmute_all_obs(&self, source_name: &str) -> AppResult<Vec<(String, AppResult<()>)>> {
        let obs_connections = self.control_room_get_obs_connections().await?;
        let mut results = Vec::new();
        
        for obs_name in obs_connections {
            let result = self.control_room_unmute_audio(&obs_name, source_name).await;
            results.push((obs_name, result));
        }
        
        log::info!("[OBS_MANAGER] Control Room: Bulk unmute operation completed for {} connections", results.len());
        Ok(results)
    }
    
    /// Control Room: Bulk scene change for all OBS connections
    pub async fn control_room_change_all_obs_scenes(&self, scene_name: &str) -> AppResult<Vec<(String, AppResult<()>)>> {
        let obs_connections = self.control_room_get_obs_connections().await?;
        let mut results = Vec::new();
        
        for obs_name in obs_connections {
            let manager = self.control_room_manager.lock().await;
            if let Some(control_room) = manager.as_ref() {
                if let Ok(obs_connection_name) = control_room.get_obs_connection_name(&obs_name).await {
                    drop(manager); // Release lock before calling scenes plugin
                    let result = self.set_current_scene(&obs_connection_name, scene_name).await;
                    results.push((obs_name, result));
                } else {
                    results.push((obs_name.clone(), Err(crate::types::AppError::ConfigError(format!("OBS '{}' is not connected", obs_name)))));
                }
            } else {
                results.push((obs_name.clone(), Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))));
            }
        }
        
        log::info!("[OBS_MANAGER] Control Room: Bulk scene change to '{}' completed for {} connections", scene_name, results.len());
        Ok(results)
    }
    
    /// Control Room: Bulk start streaming for all OBS connections
    pub async fn control_room_start_all_obs(&self) -> AppResult<Vec<(String, AppResult<()>)>> {
        let obs_connections = self.control_room_get_obs_connections().await?;
        let mut results = Vec::new();
        
        for obs_name in obs_connections {
            let manager = self.control_room_manager.lock().await;
            if let Some(control_room) = manager.as_ref() {
                if let Ok(obs_connection_name) = control_room.get_obs_connection_name(&obs_name).await {
                    drop(manager); // Release lock before calling streaming plugin
                    let result = self.start_streaming(&obs_connection_name).await;
                    results.push((obs_name, result));
                } else {
                    results.push((obs_name.clone(), Err(crate::types::AppError::ConfigError(format!("OBS '{}' is not connected", obs_name)))));
                }
            } else {
                results.push((obs_name.clone(), Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))));
            }
        }
        
        log::info!("[OBS_MANAGER] Control Room: Bulk start streaming completed for {} connections", results.len());
        Ok(results)
    }
    
    /// Control Room: Bulk stop streaming for all OBS connections
    pub async fn control_room_stop_all_obs(&self) -> AppResult<Vec<(String, AppResult<()>)>> {
        let obs_connections = self.control_room_get_obs_connections().await?;
        let mut results = Vec::new();
        
        for obs_name in obs_connections {
            let manager = self.control_room_manager.lock().await;
            if let Some(control_room) = manager.as_ref() {
                if let Ok(obs_connection_name) = control_room.get_obs_connection_name(&obs_name).await {
                    drop(manager); // Release lock before calling streaming plugin
                    let result = self.stop_streaming(&obs_connection_name).await;
                    results.push((obs_name, result));
                } else {
                    results.push((obs_name.clone(), Err(crate::types::AppError::ConfigError(format!("OBS '{}' is not connected", obs_name)))));
                }
            } else {
                results.push((obs_name.clone(), Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))));
            }
        }
        
        log::info!("[OBS_MANAGER] Control Room: Bulk stop streaming completed for {} connections", results.len());
        Ok(results)
    }

    /// Control Room: Change master password
    pub async fn control_room_change_password(&self, _session_id: &str, current_password: &str, new_password: &str) -> AppResult<()> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.change_master_password(current_password, new_password).await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }

    /// Control Room: Get audit log
    pub async fn control_room_get_audit_log(&self, _session_id: &str) -> AppResult<Vec<serde_json::Value>> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.get_audit_log().await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }

    /// Control Room: Get session info
    pub async fn control_room_get_session_info(&self) -> AppResult<serde_json::Value> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            Ok(control_room.get_session_info().await)
        } else {
            Ok(serde_json::json!({
                "authenticated": false,
                "error": "Control Room not initialized"
            }))
        }
    }

    /// Control Room: Refresh session
    pub async fn control_room_refresh_session(&self) -> AppResult<()> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.refresh_session().await
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
    }

    /// Control Room: Logout
    pub async fn control_room_logout(&self) -> AppResult<()> {
        let manager = self.control_room_manager.lock().await;
        if let Some(control_room) = manager.as_ref() {
            control_room.logout().await;
            Ok(())
        } else {
            Err(crate::types::AppError::ConfigError("Control Room not initialized".to_string()))
        }
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