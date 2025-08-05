// OBS Recording Plugin
// Handles recording start/stop, replay buffer, and recording status
// Extracted from the original plugin_obs.rs

use crate::types::{AppError, AppResult};
use super::types::*;

/// OBS Recording Plugin for recording management
pub struct ObsRecordingPlugin {
    context: ObsPluginContext,
}

impl ObsRecordingPlugin {
    /// Create a new OBS Recording Plugin
    pub fn new(context: ObsPluginContext) -> Self {
        Self { context }
    }

    /// Start recording
    pub async fn start_recording(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_RECORDING] start_recording called for '{}'", connection_name);
        
        // Use the core plugin's send_request method
        let response = self.send_recording_request(connection_name, "StartRecord", None).await?;
        
        log::info!("[OBS_RECORDING] Recording started for '{}'", connection_name);
        Ok(())
    }

    /// Stop recording
    pub async fn stop_recording(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_RECORDING] stop_recording called for '{}'", connection_name);
        
        let response = self.send_recording_request(connection_name, "StopRecord", None).await?;
        
        log::info!("[OBS_RECORDING] Recording stopped for '{}'", connection_name);
        Ok(())
    }

    /// Start replay buffer
    pub async fn start_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_RECORDING] start_replay_buffer called for '{}'", connection_name);
        
        let response = self.send_recording_request(connection_name, "StartReplayBuffer", None).await?;
        
        log::info!("[OBS_RECORDING] Replay buffer started for '{}'", connection_name);
        Ok(())
    }

    /// Stop replay buffer
    pub async fn stop_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_RECORDING] stop_replay_buffer called for '{}'", connection_name);
        
        let response = self.send_recording_request(connection_name, "StopReplayBuffer", None).await?;
        
        log::info!("[OBS_RECORDING] Replay buffer stopped for '{}'", connection_name);
        Ok(())
    }

    /// Save replay buffer
    pub async fn save_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_RECORDING] save_replay_buffer called for '{}'", connection_name);
        
        let response = self.send_recording_request(connection_name, "SaveReplayBuffer", None).await?;
        
        log::info!("[OBS_RECORDING] Replay buffer saved for '{}'", connection_name);
        Ok(())
    }

    /// Get recording status
    pub async fn get_recording_status(&self, connection_name: &str) -> AppResult<bool> {
        log::debug!("[OBS_RECORDING] get_recording_status called for '{}'", connection_name);
        
        let response = self.send_recording_request(connection_name, "GetRecordStatus", None).await?;
        
        // Parse the response to get recording status
        if let Some(output_path) = response.get("outputPath") {
            let is_recording = !output_path.is_null() && output_path.as_str().unwrap_or("").len() > 0;
            log::debug!("[OBS_RECORDING] Recording status for '{}': {}", connection_name, is_recording);
            Ok(is_recording)
        } else {
            log::warn!("[OBS_RECORDING] Unexpected response format for recording status");
            Ok(false)
        }
    }

    /// Get replay buffer status
    pub async fn get_replay_buffer_status(&self, connection_name: &str) -> AppResult<bool> {
        log::debug!("[OBS_RECORDING] get_replay_buffer_status called for '{}'", connection_name);
        
        let response = self.send_recording_request(connection_name, "GetReplayBufferStatus", None).await?;
        
        // Parse the response to get replay buffer status
        if let Some(output_path) = response.get("outputPath") {
            let is_active = !output_path.is_null() && output_path.as_str().unwrap_or("").len() > 0;
            log::debug!("[OBS_RECORDING] Replay buffer status for '{}': {}", connection_name, is_active);
            Ok(is_active)
        } else {
            log::warn!("[OBS_RECORDING] Unexpected response format for replay buffer status");
            Ok(false)
        }
    }

    /// Send a recording-related request to OBS
    async fn send_recording_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        // This will be implemented when we integrate with the core plugin
        // For now, this is a placeholder that will be replaced with actual implementation
        log::debug!("[OBS_RECORDING] Sending request '{}' to '{}'", request_type, connection_name);
        
        // TODO: Integrate with core plugin's send_request method
        // This is a placeholder response
        Ok(serde_json::json!({
            "outputPath": "",
            "outputTimecode": "",
            "recordingTime": 0
        }))
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