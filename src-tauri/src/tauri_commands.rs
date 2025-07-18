use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use crate::core::app::App;



#[derive(Debug, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size: u64,
    pub modified: String,
    pub subsystem: String,
}

// Core app commands
#[tauri::command]
pub async fn get_app_status(_app: State<'_, Arc<App>>) -> Result<String, String> {
    log::info!("Getting app status");
    Ok("Running".to_string())
}

#[tauri::command]
pub async fn shutdown_app(app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("Shutting down app");
    app.stop().await.map_err(|e| e.to_string())?;
    Ok(())
}

// UDP commands
#[tauri::command]
pub async fn start_udp_server(app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("Starting UDP server");
    app.udp_plugin().start().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn stop_udp_server(app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("Stopping UDP server");
    app.udp_plugin().stop().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_udp_status(app: State<'_, Arc<App>>) -> Result<String, String> {
    log::info!("Getting UDP status");
    let status = app.udp_plugin().get_status();
    Ok(format!("{:?}", status))
}

// OBS commands - Fixed names to match frontend expectations
#[tauri::command]
pub async fn obs_connect(url: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("OBS connect called with URL: {}", url);
    
    // Parse the URL to extract connection details
    let config = crate::plugins::plugin_obs::ObsConnectionConfig {
        name: "default".to_string(),
        host: url.replace("ws://", "").replace("wss://", "").split(':').next().unwrap_or("localhost").to_string(),
        port: 4455, // Default OBS port
        password: None,
        protocol_version: crate::plugins::plugin_obs::ObsWebSocketVersion::V5,
        enabled: true,
    };
    
    match app.obs_plugin().add_connection(config).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "OBS connection initiated"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn obs_add_connection(
    name: String,
    host: String,
    port: u16,
    password: Option<String>,
    protocol_version: String,
    enabled: bool,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("OBS add connection called: {}@{}:{}", name, host, port);
    
    let version = match protocol_version.as_str() {
        "v5" => crate::plugins::plugin_obs::ObsWebSocketVersion::V5,
        _ => crate::plugins::plugin_obs::ObsWebSocketVersion::V5, // Default to v5
    };
    
    let config = crate::plugins::plugin_obs::ObsConnectionConfig {
        name,
        host,
        port,
        password,
        protocol_version: version,
        enabled,
    };
    
    match app.obs_plugin().add_connection(config).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "OBS connection added successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn obs_connect_to_connection(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("OBS connect to connection called: {}", connection_name);
    
    match app.obs_plugin().connect_obs(&connection_name).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("OBS connection '{}' initiated", connection_name)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn obs_get_connection_status(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("OBS get connection status called: {}", connection_name);
    
    match app.obs_plugin().get_connection_status(&connection_name).await {
        Some(status) => {
            let status_str = match status {
                crate::plugins::plugin_obs::ObsConnectionStatus::Disconnected => "Disconnected",
                crate::plugins::plugin_obs::ObsConnectionStatus::Connecting => "Connecting",
                crate::plugins::plugin_obs::ObsConnectionStatus::Connected => "Connected",
                crate::plugins::plugin_obs::ObsConnectionStatus::Authenticating => "Authenticating",
                crate::plugins::plugin_obs::ObsConnectionStatus::Authenticated => "Authenticated",
                crate::plugins::plugin_obs::ObsConnectionStatus::Error => "Error",
            };
            
            Ok(serde_json::json!({
                "success": true,
                "status": status_str
            }))
        },
        None => Ok(serde_json::json!({
            "success": false,
            "error": "Connection not found"
        }))
    }
}

#[tauri::command]
pub async fn obs_get_connections(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("OBS get connections called");
    
    let connections = app.obs_plugin().get_connection_names().await;
    let mut connection_details = Vec::new();
    
    for name in connections {
        if let Some(status) = app.obs_plugin().get_connection_status(&name).await {
            let status_str = match status {
                crate::plugins::plugin_obs::ObsConnectionStatus::Disconnected => "Disconnected",
                crate::plugins::plugin_obs::ObsConnectionStatus::Connecting => "Connecting",
                crate::plugins::plugin_obs::ObsConnectionStatus::Connected => "Connected",
                crate::plugins::plugin_obs::ObsConnectionStatus::Authenticating => "Authenticating",
                crate::plugins::plugin_obs::ObsConnectionStatus::Authenticated => "Authenticated",
                crate::plugins::plugin_obs::ObsConnectionStatus::Error => "Error",
            };
            
            connection_details.push(serde_json::json!({
                "name": name,
                "status": status_str
            }));
        }
    }
    
    Ok(serde_json::json!({
        "success": true,
        "connections": connection_details
    }))
}

#[tauri::command]
pub async fn obs_disconnect(connection_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("OBS disconnect called for connection: {}", connection_name);
    app.obs_plugin().remove_connection(&connection_name).await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": true,
        "message": "OBS disconnection initiated"
    }))
}

#[tauri::command]
pub async fn obs_get_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("OBS get status called");
    match app.obs_plugin().get_obs_status().await {
        Ok(status) => Ok(serde_json::json!({
            "success": true,
            "status": status
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn obs_start_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("OBS start recording called");
    // Get the first available connection
    let connections = app.obs_plugin().get_connection_names().await;
    if let Some(connection_name) = connections.first() {
        match app.obs_plugin().start_recording(connection_name).await {
            Ok(_) => Ok(serde_json::json!({
                "success": true,
                "message": "OBS recording started"
            })),
            Err(e) => Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    } else {
        Ok(serde_json::json!({
            "success": false,
            "error": "No OBS connections available"
        }))
    }
}

#[tauri::command]
pub async fn obs_stop_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("OBS stop recording called");
    // Get the first available connection
    let connections = app.obs_plugin().get_connection_names().await;
    if let Some(connection_name) = connections.first() {
        match app.obs_plugin().stop_recording(connection_name).await {
            Ok(_) => Ok(serde_json::json!({
                "success": true,
                "message": "OBS recording stopped"
            })),
            Err(e) => Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    } else {
        Ok(serde_json::json!({
            "success": false,
            "error": "No OBS connections available"
        }))
    }
}

#[tauri::command]
pub async fn obs_command(_action: String, _params: serde_json::Value, _app: State<'_, Arc<App>>) -> Result<(), String> {
    Ok(())
}

// Video commands
#[tauri::command]
pub async fn video_play(path: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Video play called with path: {}", path);
    
    // Create a video clip from the path
    let clip = crate::plugins::plugin_playback::VideoClip {
        id: uuid::Uuid::new_v4().to_string(),
        name: std::path::Path::new(&path).file_name().unwrap_or_default().to_string_lossy().to_string(),
        path: path.clone(),
        duration: 0.0,
        timestamp: std::time::SystemTime::now(),
        tags: vec![],
        metadata: crate::plugins::plugin_playback::VideoMetadata {
            width: 0,
            height: 0,
            fps: 0.0,
            codec: "unknown".to_string(),
            bitrate: 0,
            file_size: 0,
        },
    };
    
    match app.playback_plugin().play_clip(clip) {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Video playback initiated"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn video_stop(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Video stop called");
    match app.playback_plugin().stop() {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Video playback stopped"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn video_get_info(path: String, _app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Video get info called for path: {}", path);
    match crate::plugins::plugin_playback::VideoUtils::get_video_info(&path) {
        Ok(info) => Ok(serde_json::json!({
            "success": true,
            "duration": 0,
            "format": "unknown",
            "metadata": info
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn extract_clip(_connection: String, _app: State<'_, Arc<App>>) -> Result<(), String> {
    Ok(())
}

// Store commands
#[tauri::command]
pub async fn save_event(_event: serde_json::Value, _app: State<'_, Arc<App>>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn get_events(_app: State<'_, Arc<App>>) -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn clear_events(_app: State<'_, Arc<App>>) -> Result<(), String> {
    Ok(())
}

// License commands
#[tauri::command]
pub async fn activate_license(_key: String, _app: State<'_, Arc<App>>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn validate_license(_app: State<'_, Arc<App>>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn get_license_status(_app: State<'_, Arc<App>>) -> Result<String, String> {
    Ok("Valid".to_string())
}

// Settings commands
#[tauri::command]
pub async fn get_settings(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn update_settings(_settings: serde_json::Value, _app: State<'_, Arc<App>>) -> Result<(), String> {
    Ok(())
}

// Flag commands
#[tauri::command]
pub async fn get_flag_url(_ioc_code: String, _app: State<'_, Arc<App>>) -> Result<String, String> {
    Ok("".to_string())
}

#[tauri::command]
pub async fn download_flags(_app: State<'_, Arc<App>>) -> Result<(), String> {
    Ok(())
}

// PSS commands
#[tauri::command]
pub async fn pss_start_listener(port: u16, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("PSS start listener called on port: {}", port);
    match app.udp_plugin().start() {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "PSS listener started"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn pss_stop_listener(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("PSS stop listener called");
    match app.udp_plugin().stop() {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "PSS listener stopped"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn pss_get_events(_app: State<'_, Arc<App>>) -> Result<Vec<serde_json::Value>, String> {
    log::info!("PSS get events called");
    // TODO: Implement actual PSS events retrieval from UDP plugin
    Ok(vec![])
}

// System commands
#[tauri::command]
pub async fn system_get_info(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("System get info called");
    Ok(serde_json::json!({
        "success": true,
        "platform": "windows",
        "version": "1.0.0"
    }))
}

#[tauri::command]
pub async fn system_open_file_dialog(_app: State<'_, Arc<App>>) -> Result<Vec<String>, String> {
    log::info!("System open file dialog called");
    // TODO: Implement file dialog using Tauri's dialog plugin
    Ok(vec![])
}

// Diagnostics & Logs commands
#[tauri::command]
pub async fn set_logging_enabled(
    subsystem: String,
    enabled: bool,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("set_logging_enabled: {} -> {}", subsystem, enabled);
    
    // Prevent disabling the "app" subsystem as it's needed for system logging
    if subsystem == "app" && !enabled {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Cannot disable 'app' subsystem logging as it's required for system operations"
        }));
    }
    
    // Update the log manager
    app.log_manager().set_subsystem_enabled(&subsystem, enabled);
    
    // Log the change to the system log (app) instead of the subsystem log
    if let Err(e) = app.log_manager().log("app", "INFO", &format!("Logging {} for subsystem: {}", if enabled { "enabled" } else { "disabled" }, subsystem)) {
        log::error!("Failed to log logging state change: {}", e);
    }
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Logging {} for {}", if enabled { "enabled" } else { "disabled" }, subsystem)
    }))
}

#[tauri::command]
pub async fn list_log_files(
    subsystem: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("Listing log files for subsystem: {:?}", subsystem);
    
    match app.log_manager().list_log_files(subsystem.as_deref()) {
        Ok(files) => Ok(serde_json::json!({
            "success": true,
            "data": files
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to list log files: {}", e)
        }))
    }
}

#[tauri::command]
pub async fn download_log_file(
    filename: String,
    app: State<'_, Arc<App>>,
) -> Result<Vec<u8>, String> {
    log::info!("Downloading log file: {}", filename);
    
    match app.log_manager().read_log_file(&filename) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(format!("Failed to read log file: {}", e))
    }
}

#[tauri::command]
pub async fn list_archives(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Listing archives");
    
    match app.log_manager().list_archives() {
        Ok(archives) => Ok(serde_json::json!({
            "success": true,
            "data": archives
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to list archives: {}", e)
        }))
    }
}

#[tauri::command]
pub async fn extract_archive(
    archive_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("Extracting archive: {}", archive_name);
    
    match app.log_manager().extract_archive(&archive_name) {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Archive {} extracted successfully", archive_name)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to extract archive: {}", e)
        }))
    }
}

#[tauri::command]
pub async fn download_archive(
    archive_name: String,
    app: State<'_, Arc<App>>,
) -> Result<Vec<u8>, String> {
    log::info!("Downloading archive: {}", archive_name);
    
    match app.log_manager().download_archive(&archive_name) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(format!("Failed to read archive: {}", e))
    }
}

#[tauri::command]
pub async fn set_live_data_streaming(
    subsystem: String,
    enabled: bool,
) -> Result<serde_json::Value, String> {
    log::info!("Setting live data streaming for {}: {}", subsystem, enabled);
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Live data streaming {} for {}", if enabled { "enabled" } else { "disabled" }, subsystem)
    }))
}

// Legacy commands for backward compatibility
#[tauri::command]
pub async fn enable_logging(
    subsystem: String,
    app: State<'_, Arc<App>>,
) -> Result<(), String> {
    set_logging_enabled(subsystem, true, app).await?;
    Ok(())
}

#[tauri::command]
pub async fn disable_logging(
    subsystem: String,
    app: State<'_, Arc<App>>,
) -> Result<(), String> {
    set_logging_enabled(subsystem, false, app).await?;
    Ok(())
}

#[tauri::command]
pub async fn start_live_data(subsystem: String) -> Result<(), String> {
    set_live_data_streaming(subsystem, true).await?;
    Ok(())
}

#[tauri::command]
pub async fn stop_live_data(subsystem: String) -> Result<(), String> {
    set_live_data_streaming(subsystem, false).await?;
    Ok(())
} 