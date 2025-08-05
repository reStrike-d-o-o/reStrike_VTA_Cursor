// OBS Status Plugin
// Handles status aggregation and reporting
// Extracted from the original plugin_obs.rs

use crate::types::{AppResult, AppError};
use super::types::*;
use super::recording::ObsRecordingPlugin;
use super::streaming::ObsStreamingPlugin;
use std::sync::Arc;
use std::time::{Instant, Duration};

/// Status plugin for OBS operations
pub struct ObsStatusPlugin {
    context: ObsPluginContext,
    recording_plugin: Arc<ObsRecordingPlugin>,
    streaming_plugin: Arc<ObsStreamingPlugin>,
    last_cpu_check: Arc<tokio::sync::Mutex<Instant>>,
    cached_cpu_usage: Arc<tokio::sync::Mutex<f64>>,
    monitoring_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
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
            last_cpu_check: Arc::new(tokio::sync::Mutex::new(Instant::now())),
            cached_cpu_usage: Arc::new(tokio::sync::Mutex::new(0.0)),
            monitoring_task: Arc::new(tokio::sync::Mutex::new(None)),
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
                            status_info.recording_connection = Some(connection_name.clone());
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
                            status_info.streaming_connection = Some(connection_name.clone());
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

        // Get CPU usage with caching
        status_info.cpu_usage = self.get_cpu_usage().await;

        Ok(status_info)
    }

    /// Get CPU usage with caching to avoid excessive system calls
    async fn get_cpu_usage(&self) -> f64 {
        let mut last_check = self.last_cpu_check.lock().await;
        let mut cached_usage = self.cached_cpu_usage.lock().await;
        
        // Only update CPU usage every 2 seconds to avoid excessive system calls
        if last_check.elapsed() > Duration::from_secs(2) {
            *cached_usage = self.get_real_cpu_usage().await;
            *last_check = Instant::now();
        }
        
        *cached_usage
    }

    /// Get real CPU usage from system
    async fn get_real_cpu_usage(&self) -> f64 {
        // Try to get CPU usage from system monitoring
        // This is a simplified implementation - in a real system, you'd want to use
        // a proper system monitoring library like sysinfo
        
        #[cfg(target_os = "windows")]
        {
            // Windows-specific CPU monitoring
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["cpu", "get", "loadpercentage", "/format:value"])
                .output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("LoadPercentage=") {
                            if let Ok(usage) = line.split('=').nth(1).unwrap_or("0").parse::<f64>() {
                                return usage / 100.0; // Convert percentage to decimal
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Unix-like systems
            if let Ok(contents) = tokio::fs::read_to_string("/proc/loadavg").await {
                if let Some(first) = contents.split_whitespace().next() {
                    if let Ok(load) = first.parse::<f64>() {
                        // Convert load average to approximate CPU usage
                        // This is a rough approximation
                        return (load / num_cpus::get() as f64).min(1.0);
                    }
                }
            }
        }
        
        // Fallback: return a reasonable default
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

    /// Get memory usage for OBS processes
    pub async fn get_memory_usage(&self) -> AppResult<f64> {
        // Try to find OBS processes and get their memory usage
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("tasklist")
                .args(&["/FI", "IMAGENAME eq obs64.exe", "/FO", "CSV"])
                .output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines().skip(1) { // Skip header
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 5 {
                            if let Ok(memory_kb) = parts[4].trim_matches('"').replace(",", "").parse::<u64>() {
                                return Ok(memory_kb as f64 / 1024.0 / 1024.0); // Convert to GB
                            }
                        }
                    }
                }
            }
        }
        
        Ok(0.0)
    }

    /// Get FPS for OBS (if available)
    pub async fn get_fps(&self) -> AppResult<f64> {
        // Try to get FPS from OBS WebSocket API
        if let Some(core_plugin) = &self.context.core_plugin {
            match core_plugin.send_request("OBS_REC", "GetStats", None).await {
                Ok(stats) => {
                    if let Some(fps) = stats.get("fps").and_then(|v| v.as_f64()) {
                        return Ok(fps);
                    }
                }
                Err(e) => {
                    log::debug!("[OBS_STATUS] Failed to get FPS from OBS: {}", e);
                }
            }
        }
        
        // Fallback: try to get from system monitoring
        #[cfg(target_os = "windows")]
        {
            // Try to get FPS from OBS process
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["process", "where", "name='obs64.exe'", "get", "ProcessId", "/format:value"])
                .output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("ProcessId=") {
                            if let Ok(_pid) = line.split('=').nth(1).unwrap_or("0").parse::<u32>() {
                                // For now, return a reasonable default based on typical OBS settings
                                return Ok(30.0);
                            }
                        }
                    }
                }
            }
        }
        
        // Default fallback
        Ok(30.0)
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

    /// Get detailed system information
    pub async fn get_system_info(&self) -> AppResult<SystemInfo> {
        let cpu_usage = self.get_cpu_usage().await;
        let memory_usage = self.get_memory_usage().await?;
        let fps = self.get_fps().await?;
        
        Ok(SystemInfo {
            cpu_usage,
            memory_usage,
            fps,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get detailed performance metrics
    pub async fn get_performance_metrics(&self) -> AppResult<serde_json::Value> {
        let cpu_usage = self.get_cpu_usage().await;
        let memory_usage = self.get_memory_usage().await?;
        let fps = self.get_fps().await?;
        let disk_usage = self.get_disk_usage().await?;
        let network_stats = self.get_network_stats().await?;
        
        Ok(serde_json::json!({
            "cpu": {
                "usage_percent": cpu_usage * 100.0,
                "cores": num_cpus::get(),
                "frequency_mhz": self.get_cpu_frequency().await?
            },
            "memory": {
                "usage_gb": memory_usage,
                "total_gb": self.get_total_memory().await?,
                "available_gb": self.get_available_memory().await?
            },
            "disk": {
                "usage_percent": disk_usage,
                "free_gb": self.get_free_disk_space().await?
            },
            "network": network_stats,
            "obs": {
                "fps": fps,
                "dropped_frames": self.get_dropped_frames().await?,
                "lagged_frames": self.get_lagged_frames().await?
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Get disk usage percentage
    async fn get_disk_usage(&self) -> AppResult<f64> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["logicaldisk", "where", "DeviceID='C:'", "get", "Size,FreeSpace", "/format:value"])
                .output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let mut total = 0u64;
                    let mut free = 0u64;
                    
                    for line in output_str.lines() {
                        if line.starts_with("Size=") {
                            if let Ok(size) = line.split('=').nth(1).unwrap_or("0").parse::<u64>() {
                                total = size;
                            }
                        } else if line.starts_with("FreeSpace=") {
                            if let Ok(free_space) = line.split('=').nth(1).unwrap_or("0").parse::<u64>() {
                                free = free_space;
                            }
                        }
                    }
                    
                    if total > 0 {
                        return Ok((total - free) as f64 / total as f64);
                    }
                }
            }
        }
        
        Ok(0.0)
    }

    /// Get network statistics
    async fn get_network_stats(&self) -> AppResult<serde_json::Value> {
        // This would require more complex network monitoring
        // For now, return placeholder data
        Ok(serde_json::json!({
            "bytes_sent": 0,
            "bytes_received": 0,
            "packets_sent": 0,
            "packets_received": 0,
            "connection_count": 0
        }))
    }

    /// Get CPU frequency
    async fn get_cpu_frequency(&self) -> AppResult<f64> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["cpu", "get", "MaxClockSpeed", "/format:value"])
                .output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("MaxClockSpeed=") {
                            if let Ok(freq) = line.split('=').nth(1).unwrap_or("0").parse::<f64>() {
                                return Ok(freq);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(0.0)
    }

    /// Get total memory
    async fn get_total_memory(&self) -> AppResult<f64> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["computersystem", "get", "TotalPhysicalMemory", "/format:value"])
                .output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("TotalPhysicalMemory=") {
                            if let Ok(memory) = line.split('=').nth(1).unwrap_or("0").parse::<u64>() {
                                return Ok(memory as f64 / 1024.0 / 1024.0 / 1024.0); // Convert to GB
                            }
                        }
                    }
                }
            }
        }
        
        Ok(0.0)
    }

    /// Get available memory
    async fn get_available_memory(&self) -> AppResult<f64> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["os", "get", "FreePhysicalMemory", "/format:value"])
                .output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("FreePhysicalMemory=") {
                            if let Ok(memory) = line.split('=').nth(1).unwrap_or("0").parse::<u64>() {
                                return Ok(memory as f64 / 1024.0 / 1024.0); // Convert to GB
                            }
                        }
                    }
                }
            }
        }
        
        Ok(0.0)
    }

    /// Get free disk space
    async fn get_free_disk_space(&self) -> AppResult<f64> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["logicaldisk", "where", "DeviceID='C:'", "get", "FreeSpace", "/format:value"])
                .output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("FreeSpace=") {
                            if let Ok(free_space) = line.split('=').nth(1).unwrap_or("0").parse::<u64>() {
                                return Ok(free_space as f64 / 1024.0 / 1024.0 / 1024.0); // Convert to GB
                            }
                        }
                    }
                }
            }
        }
        
        Ok(0.0)
    }

    /// Get dropped frames from OBS
    async fn get_dropped_frames(&self) -> AppResult<u64> {
        // Try to get dropped frames from OBS WebSocket API
        if let Some(core_plugin) = &self.context.core_plugin {
            match core_plugin.send_request("OBS_REC", "GetStats", None).await {
                Ok(stats) => {
                    if let Some(dropped) = stats.get("droppedFrames").and_then(|v| v.as_u64()) {
                        return Ok(dropped);
                    }
                }
                Err(e) => {
                    log::debug!("[OBS_STATUS] Failed to get dropped frames from OBS: {}", e);
                }
            }
        }
        
        // Fallback: return 0
        Ok(0)
    }

    /// Get lagged frames from OBS
    async fn get_lagged_frames(&self) -> AppResult<u64> {
        // Try to get lagged frames from OBS WebSocket API
        if let Some(core_plugin) = &self.context.core_plugin {
            match core_plugin.send_request("OBS_REC", "GetStats", None).await {
                Ok(stats) => {
                    if let Some(lagged) = stats.get("laggedFrames").and_then(|v| v.as_u64()) {
                        return Ok(lagged);
                    }
                }
                Err(e) => {
                    log::debug!("[OBS_STATUS] Failed to get lagged frames from OBS: {}", e);
                }
            }
        }
        
        // Fallback: return 0
        Ok(0)
    }

    /// Start real-time monitoring
    pub async fn start_monitoring(&self) -> AppResult<()> {
        let mut task_guard = self.monitoring_task.lock().await;
        
        if task_guard.is_some() {
            return Err(AppError::ConfigError("Monitoring is already running".to_string()));
        }

        let context = self.context.clone();
        let recording_plugin = self.recording_plugin.clone();
        let streaming_plugin = self.streaming_plugin.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5)); // Update every 5 seconds
            
            loop {
                interval.tick().await;
                
                // Get current status
                let status = Self::get_monitoring_status(&context, &recording_plugin, &streaming_plugin).await;
                
                // Emit status update event
                if let Ok(_status_info) = status {
                    let event = ObsEvent::StatusUpdate {
                        connection_name: "system".to_string(),
                        status: ObsConnectionStatus::Connected,
                    };
                    
                    if let Err(e) = context.event_tx.send(event) {
                        log::error!("[OBS_STATUS] Failed to emit monitoring event: {}", e);
                    }
                }
            }
        });
        
        *task_guard = Some(handle);
        log::info!("[OBS_STATUS] Started real-time monitoring");
        Ok(())
    }

    /// Stop real-time monitoring
    pub async fn stop_monitoring(&self) -> AppResult<()> {
        let mut task_guard = self.monitoring_task.lock().await;
        
        if let Some(handle) = task_guard.take() {
            handle.abort();
            log::info!("[OBS_STATUS] Stopped real-time monitoring");
        }
        
        Ok(())
    }

    /// Get monitoring status (static method for use in monitoring task)
    async fn get_monitoring_status(
        context: &ObsPluginContext,
        recording_plugin: &Arc<ObsRecordingPlugin>,
        streaming_plugin: &Arc<ObsStreamingPlugin>,
    ) -> AppResult<ObsStatusInfo> {
        let mut status_info = ObsStatusInfo {
            is_recording: false,
            is_streaming: false,
            cpu_usage: 0.0,
            recording_connection: None,
            streaming_connection: None,
            connections: Vec::new(),
        };

        // Get all active connections
        let connections = context.connections.lock().await;
        for (connection_name, connection) in connections.iter() {
            if connection.is_connected {
                // Get recording status for this connection
                match recording_plugin.get_recording_status(connection_name).await {
                    Ok(is_recording) => {
                        if is_recording {
                            status_info.is_recording = true;
                            status_info.recording_connection = Some(connection_name.clone());
                        }
                    }
                    Err(e) => {
                        log::warn!("[OBS_STATUS] Failed to get recording status for '{}': {}", connection_name, e);
                    }
                }

                // Get streaming status for this connection
                match streaming_plugin.get_streaming_status(connection_name).await {
                    Ok(is_streaming) => {
                        if is_streaming {
                            status_info.is_streaming = true;
                            status_info.streaming_connection = Some(connection_name.clone());
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

        // Get CPU usage (simplified for monitoring)
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["cpu", "get", "loadpercentage", "/format:value"])
                .output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("LoadPercentage=") {
                            if let Ok(usage) = line.split('=').nth(1).unwrap_or("0").parse::<f64>() {
                                status_info.cpu_usage = usage / 100.0;
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(status_info)
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
        
        // Stop monitoring if running
        let task_guard = self.monitoring_task.blocking_lock();
        if let Some(handle) = task_guard.as_ref() {
            handle.abort();
        }
        
        Ok(())
    }
} 