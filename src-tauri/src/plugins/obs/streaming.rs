// OBS Streaming Plugin
// Handles streaming operations (start, stop, status)
// Extracted from the original plugin_obs.rs

use crate::types::AppResult;
use super::types::*;
use super::core::ObsCorePlugin;
use std::sync::Arc;

/// Streaming plugin for OBS operations
pub struct ObsStreamingPlugin {
    context: ObsPluginContext,
    core_plugin: Arc<ObsCorePlugin>,
}

impl ObsStreamingPlugin {
    /// Create a new OBS Streaming Plugin
    pub fn new(context: ObsPluginContext, core_plugin: Arc<ObsCorePlugin>) -> Self {
        Self { 
            context,
            core_plugin,
        }
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
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        // Delegate to core plugin for actual WebSocket communication
        let data = request_data.unwrap_or_else(|| serde_json::json!({}));
        self.core_plugin.send_request(connection_name, request_type, Some(data)).await
    }

    /// Mute audio source
    pub async fn mute_audio_source(&self, connection_name: &str, source_name: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "inputName": source_name,
            "inputMuted": true
        });
        let _response = self.send_streaming_request(connection_name, "SetInputMute", Some(request_data)).await?;
        log::info!("[OBS_STREAMING] Muted audio source '{}' for '{}'", source_name, connection_name);
        Ok(())
    }
    
    /// Unmute audio source
    pub async fn unmute_audio_source(&self, connection_name: &str, source_name: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "inputName": source_name,
            "inputMuted": false
        });
        let _response = self.send_streaming_request(connection_name, "SetInputMute", Some(request_data)).await?;
        log::info!("[OBS_STREAMING] Unmuted audio source '{}' for '{}'", source_name, connection_name);
        Ok(())
    }
    
    /// Get audio mute status
    pub async fn get_audio_mute_status(&self, connection_name: &str, source_name: &str) -> AppResult<bool> {
        let request_data = serde_json::json!({
            "inputName": source_name
        });
        let response = self.send_streaming_request(connection_name, "GetInputMute", Some(request_data)).await?;
        
        let is_muted = response["inputMuted"].as_bool().unwrap_or(false);
        log::debug!("[OBS_STREAMING] Audio source '{}' mute status for '{}': {}", source_name, connection_name, is_muted);
        Ok(is_muted)
    }
    
    /// Get audio sources list
    pub async fn get_audio_sources(&self, connection_name: &str) -> AppResult<Vec<String>> {
        let response = self.send_streaming_request(connection_name, "GetInputList", None).await?;
        
        if let Some(inputs) = response["inputs"].as_array() {
            let audio_sources: Vec<String> = inputs.iter()
                .filter_map(|input| {
                    if let (Some(input_kind), Some(input_name)) = (
                        input["inputKind"].as_str(),
                        input["inputName"].as_str()
                    ) {
                        // Check if it's an audio input
                        if input_kind.contains("audio") || input_kind.contains("wasapi") || input_kind.contains("pulse") {
                            Some(input_name.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            log::debug!("[OBS_STREAMING] Found {} audio sources for '{}'", audio_sources.len(), connection_name);
            Ok(audio_sources)
        } else {
            log::warn!("[OBS_STREAMING] No inputs found in response for '{}'", connection_name);
            Ok(Vec::new())
        }
    }

    /// Handle streaming state change events
    pub async fn handle_streaming_state_change(&self, connection_name: &str, is_streaming: bool) {
        log::info!("[OBS_STREAMING] Streaming state changed for '{}': {}", connection_name, is_streaming);
        
        // Emit streaming state change event
        let event = ObsEvent::StreamingStateChanged {
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