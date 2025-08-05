// OBS Streaming Plugin
// Handles streaming operations (start, stop, status)
// Extracted from the original plugin_obs.rs

use crate::types::AppResult;
use super::types::*;

/// Streaming plugin for OBS operations
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
        let _response = self.send_streaming_request(connection_name, "StartStream", None).await?;
        Ok(())
    }

    /// Stop streaming
    pub async fn stop_streaming(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_streaming_request(connection_name, "StopStream", None).await?;
        Ok(())
    }

    /// Get streaming status
    pub async fn get_streaming_status(&self, connection_name: &str) -> AppResult<bool> {
        let response = self.send_streaming_request(connection_name, "GetStreamStatus", None).await?;
        
        if let Some(output_active) = response["outputActive"].as_bool() {
            Ok(output_active)
        } else {
            Ok(false)
        }
    }

    /// Send a streaming request to OBS
    async fn send_streaming_request(
        &self,
        connection_name: &str,
        request_type: &str,
        _request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        // This would delegate to the core plugin's send_request method
        // For now, return a placeholder response
        Ok(serde_json::json!({
            "outputActive": false,
            "outputTimecode": "00:00:00.000"
        }))
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