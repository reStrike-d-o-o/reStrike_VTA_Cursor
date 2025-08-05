// OBS Scenes Plugin
// Handles scene management, switching, and source manipulation
// Extracted from the original plugin_obs.rs

use crate::types::{AppResult, AppError};
use super::types::*;

/// OBS Scenes Plugin for scene management
pub struct ObsScenesPlugin {
    context: ObsPluginContext,
}

impl ObsScenesPlugin {
    /// Create a new OBS Scenes Plugin
    pub fn new(context: ObsPluginContext) -> Self {
        Self { context }
    }

    /// Get current scene
    pub async fn get_current_scene(&self, connection_name: &str) -> AppResult<String> {
        log::debug!("[OBS_SCENES] get_current_scene called for '{}'", connection_name);
        
        let response = self.send_scene_request(connection_name, "GetCurrentProgramScene", None).await?;
        
        // Parse the response to get current scene name
        if let Some(scene_name) = response.get("currentProgramSceneName") {
            if let Some(name) = scene_name.as_str() {
                log::debug!("[OBS_SCENES] Current scene for '{}': {}", connection_name, name);
                return Ok(name.to_string());
            }
        }
        
        Err(AppError::ConfigError("Failed to get current scene name".to_string()))
    }

    /// Set current scene
    pub async fn set_current_scene(&self, connection_name: &str, scene_name: &str) -> AppResult<()> {
        log::info!("[OBS_SCENES] set_current_scene called for '{}' to '{}'", connection_name, scene_name);
        
        let request_data = serde_json::json!({
            "sceneName": scene_name
        });
        
        let _response = self.send_scene_request(connection_name, "SetCurrentProgramScene", Some(request_data)).await?;
        
        log::info!("[OBS_SCENES] Scene changed for '{}' to '{}'", connection_name, scene_name);
        Ok(())
    }

    /// Get all scenes
    pub async fn get_scenes(&self, connection_name: &str) -> AppResult<Vec<String>> {
        log::debug!("[OBS_SCENES] get_scenes called for '{}'", connection_name);
        
        let response = self.send_scene_request(connection_name, "GetSceneList", None).await?;
        
        // Parse the response to get scene names
        if let Some(scenes) = response.get("scenes") {
            if let Some(scenes_array) = scenes.as_array() {
                let scene_names: Vec<String> = scenes_array
                    .iter()
                    .filter_map(|scene| {
                        scene.get("sceneName")?.as_str().map(|s| s.to_string())
                    })
                    .collect();
                
                log::debug!("[OBS_SCENES] Found {} scenes for '{}'", scene_names.len(), connection_name);
                return Ok(scene_names);
            }
        }
        
        log::warn!("[OBS_SCENES] Failed to parse scenes response");
        Ok(Vec::new())
    }

    /// Send a scene-related request to OBS using the core plugin
    async fn send_scene_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_SCENES] send_scene_request: {} for '{}'", request_type, connection_name);
        
        // Use the core plugin's send_request method instead of handling WebSocket directly
        let request_data = request_data.unwrap_or_else(|| serde_json::json!({}));
        
        // Get the core plugin from the context
        if let Some(core_plugin) = self.context.core_plugin.as_ref() {
            core_plugin.send_request(connection_name, request_type, Some(request_data)).await
        } else {
            Err(AppError::ConfigError("Core plugin not available".to_string()))
        }
    }

    /// Get studio mode status
    pub async fn get_studio_mode(&self, connection_name: &str) -> AppResult<bool> {
        log::debug!("[OBS_SCENES] get_studio_mode called for '{}'", connection_name);
        
        let response = self.send_scene_request(connection_name, "GetStudioModeEnabled", None).await?;
        
        if let Some(enabled) = response.get("studioModeEnabled").and_then(|v| v.as_bool()) {
            log::debug!("[OBS_SCENES] Studio mode for '{}': {}", connection_name, enabled);
            return Ok(enabled);
        }
        
        Err(AppError::ConfigError("Failed to get studio mode status".to_string()))
    }

    /// Set studio mode
    pub async fn set_studio_mode(&self, connection_name: &str, enabled: bool) -> AppResult<()> {
        log::info!("[OBS_SCENES] set_studio_mode called for '{}' to '{}'", connection_name, enabled);
        
        let request_data = serde_json::json!({
            "studioModeEnabled": enabled
        });
        
        let _response = self.send_scene_request(connection_name, "SetStudioModeEnabled", Some(request_data)).await?;
        
        log::info!("[OBS_SCENES] Studio mode changed for '{}' to '{}'", connection_name, enabled);
        Ok(())
    }

    /// Get sources in a scene
    pub async fn get_sources(&self, connection_name: &str, scene_name: &str) -> AppResult<Vec<String>> {
        log::debug!("[OBS_SCENES] get_sources called for '{}' in scene '{}'", connection_name, scene_name);
        
        let request_data = serde_json::json!({
            "sceneName": scene_name
        });
        
        let response = self.send_scene_request(connection_name, "GetSceneItemList", Some(request_data)).await?;
        
        if let Some(scene_items) = response.get("sceneItems") {
            if let Some(items_array) = scene_items.as_array() {
                let source_names: Vec<String> = items_array
                    .iter()
                    .filter_map(|item| {
                        item.get("sourceName")?.as_str().map(|s| s.to_string())
                    })
                    .collect();
                
                log::debug!("[OBS_SCENES] Found {} sources in scene '{}' for '{}'", source_names.len(), scene_name, connection_name);
                return Ok(source_names);
            }
        }
        
        log::warn!("[OBS_SCENES] Failed to parse sources response");
        Ok(Vec::new())
    }

    /// Set source visibility
    pub async fn set_source_visibility(&self, connection_name: &str, scene_name: &str, source_name: &str, visible: bool) -> AppResult<()> {
        log::info!("[OBS_SCENES] set_source_visibility called for '{}' in scene '{}', source '{}' to '{}'", 
            connection_name, scene_name, source_name, visible);
        
        let request_data = serde_json::json!({
            "sceneName": scene_name,
            "sourceName": source_name,
            "sceneItemEnabled": visible
        });
        
        let _response = self.send_scene_request(connection_name, "SetSceneItemEnabled", Some(request_data)).await?;
        
        log::info!("[OBS_SCENES] Source visibility changed for '{}' in scene '{}', source '{}' to '{}'", 
            connection_name, scene_name, source_name, visible);
        Ok(())
    }

    /// Get source visibility
    pub async fn get_source_visibility(&self, connection_name: &str, scene_name: &str, source_name: &str) -> AppResult<bool> {
        log::debug!("[OBS_SCENES] get_source_visibility called for '{}' in scene '{}', source '{}'", 
            connection_name, scene_name, source_name);
        
        let request_data = serde_json::json!({
            "sceneName": scene_name,
            "sourceName": source_name
        });
        
        let response = self.send_scene_request(connection_name, "GetSceneItemEnabled", Some(request_data)).await?;
        
        if let Some(enabled) = response.get("sceneItemEnabled").and_then(|v| v.as_bool()) {
            log::debug!("[OBS_SCENES] Source visibility for '{}' in scene '{}', source '{}': {}", 
                connection_name, scene_name, source_name, enabled);
            return Ok(enabled);
        }
        
        Err(AppError::ConfigError("Failed to get source visibility".to_string()))
    }

    /// Handle scene change events
    pub async fn handle_scene_change(&self, connection_name: &str, scene_name: &str) {
        log::info!("[OBS_SCENES] Scene changed for '{}' to '{}'", connection_name, scene_name);
        
        // Emit scene change event
        let event = ObsEvent::SceneChanged {
            connection_name: connection_name.to_string(),
            scene_name: scene_name.to_string(),
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_SCENES] Failed to emit scene change event: {}", e);
        }
    }
}

// Implement ObsPlugin trait for the scenes plugin
impl ObsPlugin for ObsScenesPlugin {
    fn name(&self) -> &str {
        "obs_scenes"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Scenes Plugin");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Scenes Plugin");
        Ok(())
    }
} 