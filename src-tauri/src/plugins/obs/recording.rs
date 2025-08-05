// OBS Recording Plugin
// Handles recording operations (start, stop, replay buffer)
// Extracted from the original plugin_obs.rs

use crate::types::AppResult;
use super::types::*;
use super::core::ObsCorePlugin;
use std::sync::Arc;

/// Recording plugin for OBS operations
pub struct ObsRecordingPlugin {
    context: ObsPluginContext,
    core_plugin: Arc<ObsCorePlugin>,
}

impl ObsRecordingPlugin {
    /// Create a new OBS Recording Plugin
    pub fn new(context: ObsPluginContext, core_plugin: Arc<ObsCorePlugin>) -> Self {
        Self { 
            context,
            core_plugin,
        }
    }

    /// Start recording
    pub async fn start_recording(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "StartRecord", None).await?;
        Ok(())
    }

    /// Stop recording
    pub async fn stop_recording(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "StopRecord", None).await?;
        Ok(())
    }

    /// Start replay buffer
    pub async fn start_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "StartReplayBuffer", None).await?;
        Ok(())
    }

    /// Stop replay buffer
    pub async fn stop_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "StopReplayBuffer", None).await?;
        Ok(())
    }

    /// Save replay buffer
    pub async fn save_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "SaveReplayBuffer", None).await?;
        Ok(())
    }

    /// Get recording status
    pub async fn get_recording_status(&self, connection_name: &str) -> AppResult<bool> {
        let response = self.send_recording_request(connection_name, "GetRecordStatus", None).await?;
        
        if let Some(output_active) = response["outputActive"].as_bool() {
            Ok(output_active)
        } else {
            Ok(false)
        }
    }

    /// Get replay buffer status
    pub async fn get_replay_buffer_status(&self, connection_name: &str) -> AppResult<bool> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferStatus", None).await?;
        
        if let Some(output_active) = response["outputActive"].as_bool() {
            Ok(output_active)
        } else {
            Ok(false)
        }
    }

    /// Send a recording request to OBS
    async fn send_recording_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        // Delegate to core plugin for actual WebSocket communication
        let data = request_data.unwrap_or_else(|| serde_json::json!({}));
        self.core_plugin.send_request(connection_name, request_type, Some(data)).await
    }

    /// Handle recording state change events
    pub async fn handle_recording_state_change(&self, connection_name: &str, is_recording: bool) {
        log::info!("[OBS_RECORDING] Recording state changed for '{}': {}", connection_name, is_recording);
        
        // Emit recording state change event
        let event = ObsEvent::RecordingStateChanged {
            connection_name: connection_name.to_string(),
            is_recording,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_RECORDING] Failed to emit recording state change event: {}", e);
        }
    }

    /// Handle replay buffer state change events
    pub async fn handle_replay_buffer_state_change(&self, connection_name: &str, is_active: bool) {
        log::info!("[OBS_RECORDING] Replay buffer state changed for '{}': {}", connection_name, is_active);
        
        // Emit replay buffer state change event
        let event = ObsEvent::ReplayBufferStateChanged {
            connection_name: connection_name.to_string(),
            is_active,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_RECORDING] Failed to emit replay buffer state change event: {}", e);
        }
    }
}

// Implement ObsPlugin trait for the recording plugin
impl ObsPlugin for ObsRecordingPlugin {
    fn name(&self) -> &str {
        "obs_recording"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Recording Plugin");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Recording Plugin");
        Ok(())
    }
} 