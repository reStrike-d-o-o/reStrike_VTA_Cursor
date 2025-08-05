// OBS Streaming Plugin
// Handles streaming start/stop and streaming status monitoring
// Extracted from the original plugin_obs.rs

use crate::types::{AppError, AppResult};
use super::types::*;

/// OBS Streaming Plugin for streaming management
pub struct ObsStreamingPlugin {
    context: ObsPluginContext,
}

impl ObsStreamingPlugin {
    /// Create a new OBS Streaming Plugin
    pub fn new(context: ObsPluginContext) -> Self {
        Self { context }
    }

    /// Start streaming
    pub async fn start_streaming(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_STREAMING] start_streaming called for '{}'", connection_name);
        
        let response = self.send_streaming_request(connection_name, "StartStream", None).await?;
        
        log::info!("[OBS_STREAMING] Streaming started for '{}'", connection_name);
        Ok(())
    }

    /// Stop streaming
    pub async fn stop_streaming(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_STREAMING] stop_streaming called for '{}'", connection_name);
        
        let response = self.send_streaming_request(connection_name, "StopStream", None).await?;
        
        log::info!("[OBS_STREAMING] Streaming stopped for '{}'", connection_name);
        Ok(())
    }

    /// Get streaming status
    pub async fn get_streaming_status(&self, connection_name: &str) -> AppResult<bool> {
        log::debug!("[OBS_STREAMING] get_streaming_status called for '{}'", connection_name);
        
        let response = self.send_streaming_request(connection_name, "GetStreamStatus", None).await?;
        
        // Parse the response to get streaming status
        if let Some(output_path) = response.get("outputPath") {
            let is_streaming = !output_path.is_null() && output_path.as_str().unwrap_or("").len() > 0;
            log::debug!("[OBS_STREAMING] Streaming status for '{}': {}", connection_name, is_streaming);
            Ok(is_streaming)
        } else {
            log::warn!("[OBS_STREAMING] Unexpected response format for streaming status");
            Ok(false)
        }
    }

    /// Send a streaming-related request to OBS
    async fn send_streaming_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        // This will be implemented when we integrate with the core plugin
        // For now, this is a placeholder that will be replaced with actual implementation
        log::debug!("[OBS_STREAMING] Sending request '{}' to '{}'", request_type, connection_name);
        
        // TODO: Integrate with core plugin's send_request method
        // This is a placeholder response
        Ok(serde_json::json!({
            "outputPath": "",
            "outputTimecode": "",
            "streamingTime": 0
        }))
    }

    /// Handle streaming state change events
    pub async fn handle_streaming_state_change(&self, connection_name: &str, is_streaming: bool) {
        log::info!("[OBS_STREAMING] Streaming state changed for '{}': {}", connection_name, is_streaming);
        
        // Emit streaming state change event
        let event = ObsEvent::StreamStateChanged {
            connection_name: connection_name.to_string(),
            is_streaming,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_STREAMING] Failed to emit streaming state change event: {}", e);
        }
    }
}

// Implement ObsPlugin trait for the streaming plugin
impl ObsPlugin for ObsStreamingPlugin {
    fn name(&self) -> &str {
        "obs_streaming"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Streaming Plugin");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Streaming Plugin");
        Ok(())
    }
} 