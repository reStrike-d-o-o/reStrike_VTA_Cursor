// OBS Status Plugin
// Handles status aggregation and reporting
// Extracted from the original plugin_obs.rs

use crate::types::AppResult;
use super::types::*;
use super::recording::ObsRecordingPlugin;
use super::streaming::ObsStreamingPlugin;
use std::sync::Arc;

/// Status plugin for OBS operations
pub struct ObsStatusPlugin {
    context: ObsPluginContext,
    recording_plugin: Arc<ObsRecordingPlugin>,
    streaming_plugin: Arc<ObsStreamingPlugin>,
}

impl ObsStatusPlugin {
    /// Create a new OBS Status Plugin
    pub fn new(
        context: ObsPluginContext, 
        recording_plugin: Arc<ObsRecordingPlugin>,
        streaming_plugin: Arc<ObsStreamingPlugin>,
    ) -> Self {
        Self { 
            context,
            recording_plugin,
            streaming_plugin,
        }
    }

    /// Get comprehensive OBS status for all connections
    pub async fn get_obs_status(&self) -> AppResult<ObsStatusInfo> {
        let mut status_info = ObsStatusInfo {
            is_recording: false,
            is_streaming: false,
            cpu_usage: 0.0,
            recording_connection: None,
            streaming_connection: None,
            connections: Vec::new(),
        };

        // Get all active connections
        let connections = self.context.connections.lock().await;
        for (connection_name, connection) in connections.iter() {
            if connection.is_connected {
                // Get recording status for this connection
                match self.recording_plugin.get_recording_status(connection_name).await {
                    Ok(is_recording) => {
                        if is_recording {
                            status_info.is_recording = true;
                        }
                    }
                    Err(e) => {
                        log::warn!("[OBS_STATUS] Failed to get recording status for '{}': {}", connection_name, e);
                    }
                }

                // Get streaming status for this connection
                match self.streaming_plugin.get_streaming_status(connection_name).await {
                    Ok(is_streaming) => {
                        if is_streaming {
                            status_info.is_streaming = true;
                        }
                    }
                    Err(e) => {
                        log::warn!("[OBS_STATUS] Failed to get streaming status for '{}': {}", connection_name, e);
                    }
                }

                // Add connection info
                status_info.connections.push(ObsConnectionInfo {
                    name: connection_name.clone(),
                    is_connected: connection.is_connected,
                    last_heartbeat: connection.last_heartbeat,
                });
            }
        }

        // Get CPU usage (placeholder for now)
        status_info.cpu_usage = self.get_cpu_usage().await;

        Ok(status_info)
    }

    /// Get CPU usage (placeholder implementation)
    async fn get_cpu_usage(&self) -> f64 {
        // This would integrate with system monitoring
        // For now, return a placeholder value
        0.0
    }

    /// Get status for a specific connection
    pub async fn get_connection_status(&self, connection_name: &str) -> AppResult<ObsConnectionStatus> {
        let connections = self.context.connections.lock().await;
        
        if let Some(connection) = connections.get(connection_name) {
            Ok(connection.status.clone())
        } else {
            Err(crate::types::AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }

    /// Handle status update events
    pub async fn handle_status_update(&self, connection_name: &str, status: ObsConnectionStatus) {
        log::info!("[OBS_STATUS] Status update for '{}': {:?}", 
            connection_name, status);
        
        // Emit status update event
        let event = ObsEvent::StatusUpdate {
            connection_name: connection_name.to_string(),
            status,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_STATUS] Failed to emit status update event: {}", e);
        }
    }
}

// Implement ObsPlugin trait for the status plugin
impl ObsPlugin for ObsStatusPlugin {
    fn name(&self) -> &str {
        "obs_status"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Status Plugin");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Status Plugin");
        Ok(())
    }
} 