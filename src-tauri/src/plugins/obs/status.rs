// OBS Status Plugin
// Handles status aggregation, monitoring, and health checks
// Extracted from the original plugin_obs.rs

use crate::types::{AppError, AppResult};
use super::types::*;

/// OBS Status Plugin for status monitoring
pub struct ObsStatusPlugin {
    context: ObsPluginContext,
}

impl ObsStatusPlugin {
    /// Create a new OBS Status Plugin
    pub fn new(context: ObsPluginContext) -> Self {
        Self { context }
    }

    /// Get OBS CPU usage
    pub async fn get_obs_cpu_usage(&self, connection_name: &str) -> AppResult<f64> {
        log::debug!("[OBS_STATUS] get_obs_cpu_usage called for '{}'", connection_name);
        
        let response = self.send_status_request(connection_name, "GetStats", None).await?;
        
        // Parse the response to get CPU usage
        if let Some(cpu_usage) = response.get("cpuUsage") {
            if let Some(usage) = cpu_usage.as_f64() {
                log::debug!("[OBS_STATUS] CPU usage for '{}': {}%", connection_name, usage);
                return Ok(usage);
            }
        }
        
        log::warn!("[OBS_STATUS] Failed to get CPU usage for '{}'", connection_name);
        Ok(0.0)
    }

    /// Get comprehensive OBS status information
    pub async fn get_obs_status_info(&self, connection_name: &str) -> AppResult<ObsStatusInfo> {
        log::debug!("[OBS_STATUS] get_obs_status_info called for '{}'", connection_name);
        
        // Get recording status
        let is_recording = self.get_recording_status(connection_name).await.unwrap_or(false);
        
        // Get streaming status
        let is_streaming = self.get_streaming_status(connection_name).await.unwrap_or(false);
        
        // Get CPU usage
        let cpu_usage = self.get_obs_cpu_usage(connection_name).await.unwrap_or(0.0);
        
        // Determine which connection is handling recording/streaming
        let recording_connection = if is_recording { Some(connection_name.to_string()) } else { None };
        let streaming_connection = if is_streaming { Some(connection_name.to_string()) } else { None };
        
        let status_info = ObsStatusInfo {
            is_recording,
            is_streaming,
            cpu_usage,
            recording_connection,
            streaming_connection,
        };
        
        log::debug!("[OBS_STATUS] Status info for '{}': recording={}, streaming={}, cpu={}%", 
                   connection_name, is_recording, is_streaming, cpu_usage);
        
        Ok(status_info)
    }

    /// Get recording status (delegates to recording plugin)
    async fn get_recording_status(&self, connection_name: &str) -> AppResult<bool> {
        // TODO: Integrate with recording plugin
        // For now, return a placeholder
        log::debug!("[OBS_STATUS] Getting recording status for '{}'", connection_name);
        Ok(false)
    }

    /// Get streaming status (delegates to streaming plugin)
    async fn get_streaming_status(&self, connection_name: &str) -> AppResult<bool> {
        // TODO: Integrate with streaming plugin
        // For now, return a placeholder
        log::debug!("[OBS_STATUS] Getting streaming status for '{}'", connection_name);
        Ok(false)
    }

    /// Send a status-related request to OBS
    async fn send_status_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        // This will be implemented when we integrate with the core plugin
        // For now, this is a placeholder that will be replaced with actual implementation
        log::debug!("[OBS_STATUS] Sending request '{}' to '{}'", request_type, connection_name);
        
        // TODO: Integrate with core plugin's send_request method
        // This is a placeholder response
        match request_type {
            "GetStats" => Ok(serde_json::json!({
                "cpuUsage": 15.5,
                "memoryUsage": 1024.0,
                "availableDiskSpace": 50000.0,
                "activeFps": 60.0,
                "renderSkippedFrames": 0,
                "renderTotalFrames": 3600,
                "outputSkippedFrames": 0,
                "outputTotalFrames": 3600
            })),
            _ => Ok(serde_json::json!({}))
        }
    }

    /// Perform health check on OBS connection
    pub async fn health_check(&self, connection_name: &str) -> AppResult<bool> {
        log::debug!("[OBS_STATUS] Performing health check for '{}'", connection_name);
        
        // Check if connection exists and is authenticated
        let connections = self.context.connections.lock().await;
        if let Some(connection) = connections.get(connection_name) {
            let is_healthy = connection.status == ObsConnectionStatus::Authenticated;
            log::debug!("[OBS_STATUS] Health check for '{}': {}", connection_name, is_healthy);
            return Ok(is_healthy);
        }
        
        log::warn!("[OBS_STATUS] Connection '{}' not found for health check", connection_name);
        Ok(false)
    }

    /// Get connection health status for all connections
    pub async fn get_all_connection_health(&self) -> AppResult<Vec<(String, bool)>> {
        log::debug!("[OBS_STATUS] Getting health status for all connections");
        
        let connections = self.context.connections.lock().await;
        let health_status: Vec<(String, bool)> = connections
            .iter()
            .map(|(name, connection)| {
                let is_healthy = connection.status == ObsConnectionStatus::Authenticated;
                (name.clone(), is_healthy)
            })
            .collect();
        
        log::debug!("[OBS_STATUS] Health status for {} connections", health_status.len());
        Ok(health_status)
    }

    /// Monitor OBS status periodically
    pub async fn start_status_monitoring(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_STATUS] Starting status monitoring for '{}'", connection_name);
        
        // TODO: Implement periodic status monitoring
        // This would spawn a background task that periodically checks OBS status
        // and emits status update events
        
        log::info!("[OBS_STATUS] Status monitoring started for '{}'", connection_name);
        Ok(())
    }

    /// Stop OBS status monitoring
    pub async fn stop_status_monitoring(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_STATUS] Stopping status monitoring for '{}'", connection_name);
        
        // TODO: Implement stopping status monitoring
        // This would stop the background monitoring task
        
        log::info!("[OBS_STATUS] Status monitoring stopped for '{}'", connection_name);
        Ok(())
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