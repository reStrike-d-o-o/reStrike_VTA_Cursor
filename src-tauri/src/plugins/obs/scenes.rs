// OBS Scenes Plugin
// Handles scene management, switching, and source manipulation
// Extracted from the original plugin_obs.rs

use crate::types::{AppError, AppResult};
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
        
        let response = self.send_scene_request(connection_name, "SetCurrentProgramScene", Some(request_data)).await?;
        
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

    /// Send a scene-related request to OBS
    async fn send_scene_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        // This will be implemented when we integrate with the core plugin
        // For now, this is a placeholder that will be replaced with actual implementation
        log::debug!("[OBS_SCENES] Sending request '{}' to '{}'", request_type, connection_name);
        
        // TODO: Integrate with core plugin's send_request method
        // This is a placeholder response
        match request_type {
            "GetCurrentProgramScene" => Ok(serde_json::json!({
                "currentProgramSceneName": "Default Scene"
            })),
            "GetSceneList" => Ok(serde_json::json!({
                "scenes": [
                    {"sceneName": "Default Scene"},
                    {"sceneName": "Scene 2"},
                    {"sceneName": "Scene 3"}
                ]
            })),
            _ => Ok(serde_json::json!({}))
        }
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