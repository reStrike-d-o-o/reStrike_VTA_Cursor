use crate::plugins::{ObsPlugin, ObsConnectionConfig, ObsWebSocketVersion, ObsStatusInfo};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;
use crate::types::{AppError, AppResult};

// Global OBS plugin instance
pub type ObsPluginState = Arc<Mutex<Option<ObsPlugin>>>;

// Tauri command request/response structures
#[derive(Debug, Serialize, Deserialize)]
pub struct AddConnectionRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connection_name: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

// OBS Status Response for Status Bar
#[derive(Debug, Serialize, Deserialize)]
pub struct ObsStatusResponse {
    pub success: bool,
    pub data: Option<ObsStatusInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size: u64,
    pub modified: String, // ISO 8601
    pub subsystem: String, // "pss", "obs", "udp", or "other"
}

// Initialize OBS plugin
pub fn init_obs_plugin() -> ObsPluginState {
    let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
    let plugin = ObsPlugin::new(event_tx);
    Arc::new(Mutex::new(Some(plugin)))
}

// Note: Tauri commands are commented out since we removed the tauri dependency
// These can be re-enabled when we add Tauri back for the GUI application

/*
// Tauri command: Add OBS connection
#[tauri::command]
pub async fn obs_add_connection(
    request: AddConnectionRequest,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Remove OBS connection
#[tauri::command]
pub async fn obs_remove_connection(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Connect to OBS
#[tauri::command]
pub async fn obs_connect(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Get connection status
#[tauri::command]
pub async fn obs_get_connection_status(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Get all connection names
#[tauri::command]
pub async fn obs_get_connection_names(
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Get current scene
#[tauri::command]
pub async fn obs_get_current_scene(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Set current scene
#[tauri::command]
pub async fn obs_set_current_scene(
    connection_name: String,
    scene_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Start recording
#[tauri::command]
pub async fn obs_start_recording(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Stop recording
#[tauri::command]
pub async fn obs_stop_recording(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Get recording status
#[tauri::command]
pub async fn obs_get_recording_status(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Start replay buffer
#[tauri::command]
pub async fn obs_start_replay_buffer(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Stop replay buffer
#[tauri::command]
pub async fn obs_stop_replay_buffer(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Save replay buffer
#[tauri::command]
pub async fn obs_save_replay_buffer(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Get replay buffer status
#[tauri::command]
pub async fn obs_get_replay_buffer_status(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Tauri command: Get all scenes
#[tauri::command]
pub async fn obs_get_scenes(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}

// Register all OBS commands with Tauri
pub fn register_obs_commands(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Implementation commented out
    todo!("Re-enable when Tauri is added back")
}
*/

// Direct API functions for use without Tauri
// Remove or comment out the old non-Tauri add_obs_connection function to avoid duplicate definition.

#[tauri::command]
pub async fn add_obs_connection(
    request: AddConnectionRequest,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<String, String> {
    let config = ObsConnectionConfig {
        name: request.name,
        host: request.host,
        port: request.port,
        password: request.password,
        protocol_version: ObsWebSocketVersion::V5,
        enabled: request.enabled,
    };
    let mut plugin = plugin_state.lock().unwrap();
    if let Some(plugin) = plugin.as_mut() {
        plugin.add_connection(config).await.map_err(|e| e.to_string())?;
        Ok("Connection added".to_string())
    } else {
        Err("OBS plugin not initialized".to_string())
    }
}

#[tauri::command]
pub async fn remove_obs_connection(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<String, String> {
    let mut plugin = plugin_state.lock().unwrap();
    if let Some(plugin) = plugin.as_mut() {
        plugin.remove_connection(&connection_name).map_err(|e| e.to_string())?;
        Ok("Connection removed".to_string())
    } else {
        Err("OBS plugin not initialized".to_string())
    }
}

// Get comprehensive OBS status for status bar
pub async fn get_obs_status(plugin: &ObsPlugin) -> Result<ObsStatusResponse, String> {
    match plugin.get_obs_status().await {
        Ok(status_info) => Ok(ObsStatusResponse {
            success: true,
            data: Some(status_info),
            error: None,
        }),
        Err(e) => Ok(ObsStatusResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

// Get recording status from specific connection
pub async fn get_obs_recording_status(plugin: &ObsPlugin, connection_name: &str) -> Result<ObsResponse, String> {
    match plugin.get_recording_status(connection_name).await {
        Ok(is_recording) => Ok(ObsResponse {
            success: true,
            data: Some(serde_json::json!({ "is_recording": is_recording })),
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

// Get streaming status from specific connection
pub async fn get_obs_streaming_status(plugin: &ObsPlugin, connection_name: &str) -> Result<ObsResponse, String> {
    match plugin.get_streaming_status(connection_name).await {
        Ok(is_streaming) => Ok(ObsResponse {
            success: true,
            data: Some(serde_json::json!({ "is_streaming": is_streaming })),
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

// Get CPU usage from specific connection
pub async fn get_obs_cpu_usage(plugin: &ObsPlugin, connection_name: &str) -> Result<ObsResponse, String> {
    match plugin.get_obs_cpu_usage(connection_name).await {
        Ok(cpu_usage) => Ok(ObsResponse {
            success: true,
            data: Some(serde_json::json!({ "cpu_usage": cpu_usage })),
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub fn set_logging_enabled(subsystem: String, enabled: bool) -> Result<(), String> {
    // TODO: Implement runtime logging toggle for each subsystem
    // Example: update a global state or config, or enable/disable log output
    println!("[TAURI] Set logging for {} to {}", subsystem, enabled);
    Ok(())
}

#[tauri::command]
pub fn list_log_files(subsystem: Option<String>) -> Result<Vec<LogFileInfo>, String> {
    use std::fs;
    use std::path::Path;
    use chrono::prelude::*;
    let log_dir = Path::new("log");
    let mut files = Vec::new();
    if log_dir.exists() {
        for entry in fs::read_dir(log_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().map(|ext| ext == "txt").unwrap_or(false) {
                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                let modified = metadata.modified().ok()
                    .and_then(|mtime| DateTime::<Local>::from(mtime).to_rfc3339().into())
                    .unwrap_or_else(|| "".to_string());
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let size = metadata.len();
                let subsystem_guess = if name.contains("pss") {
                    "pss"
                } else if name.contains("obs") {
                    "obs"
                } else if name.contains("udp") {
                    "udp"
                } else {
                    "other"
                };
                if subsystem.as_ref().map(|s| s == subsystem_guess).unwrap_or(true) {
                    files.push(LogFileInfo {
                        name,
                        size,
                        modified,
                        subsystem: subsystem_guess.to_string(),
                    });
                }
            }
        }
    }
    Ok(files)
}

#[tauri::command]
pub fn download_log_file(filename: String) -> Result<Vec<u8>, String> {
    use std::fs;
    use std::path::Path;
    let log_path = Path::new("log").join(&filename);
    if !log_path.exists() {
        return Err("File not found".to_string());
    }
    fs::read(log_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_live_data_streaming(subsystem: String, enabled: bool) -> Result<(), String> {
    // TODO: Implement live data streaming toggle for each subsystem
    // This should start/stop emitting events to the frontend
    println!("[TAURI] Set live data streaming for {} to {}", subsystem, enabled);
    Ok(())
} 