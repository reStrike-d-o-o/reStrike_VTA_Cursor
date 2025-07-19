use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, Manager, Emitter};
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
    enabled: bool,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("OBS add connection called: {}@{}:{}", name, host, port);
    
    // Always use v5 protocol
    let version = crate::plugins::plugin_obs::ObsWebSocketVersion::V5;
    
    // Clone values before moving them
    let name_clone = name.clone();
    let host_clone = host.clone();
    let password_clone = password.clone();
    
    let config = crate::plugins::plugin_obs::ObsConnectionConfig {
        name,
        host,
        port,
        password,
        protocol_version: version,
        enabled,
    };
    
    match app.obs_plugin().add_connection(config).await {
        Ok(_) => {
            // Also save to configuration manager
            let config_conn = crate::config::ObsConnectionConfig {
                name: name_clone,
                host: host_clone,
                port,
                password: password_clone,
                protocol_version: "v5".to_string(), // Always v5
                enabled,
                timeout_seconds: 30,
                auto_reconnect: true,
                max_reconnect_attempts: 5,
            };
            
            // Get current connections and add new one
            let mut connections = app.config_manager().get_obs_connections().await;
            // Remove existing connection with same name if it exists
            connections.retain(|c| c.name != config_conn.name);
            connections.push(config_conn);
            
            if let Err(e) = app.config_manager().update_obs_connections(connections).await {
                log::warn!("Failed to save connection to config: {}", e);
            }
            
            Ok(serde_json::json!({
                "success": true,
                "message": "OBS connection added successfully"
            }))
        }
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
                crate::plugins::plugin_obs::ObsConnectionStatus::Error(_) => "Error",
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
    
    let connections = app.config_manager().get_obs_connections().await;
    let mut connection_details = Vec::new();
    
    for conn in connections {
        // Get actual status from OBS plugin if available
        let status_str = if let Some(status) = app.obs_plugin().get_connection_status(&conn.name).await {
            match status {
                crate::plugins::plugin_obs::ObsConnectionStatus::Disconnected => "Disconnected",
                crate::plugins::plugin_obs::ObsConnectionStatus::Connecting => "Connecting",
                crate::plugins::plugin_obs::ObsConnectionStatus::Connected => "Connected",
                crate::plugins::plugin_obs::ObsConnectionStatus::Authenticating => "Authenticating",
                crate::plugins::plugin_obs::ObsConnectionStatus::Authenticated => "Authenticated",
                crate::plugins::plugin_obs::ObsConnectionStatus::Error(_) => "Error",
            }
        } else {
            "Disconnected"
        };
        
        connection_details.push(serde_json::json!({
            "name": conn.name,
            "host": conn.host,
            "port": conn.port,
            "password": conn.password,
            "protocol_version": conn.protocol_version,
            "enabled": conn.enabled,
            "status": status_str
        }));
    }
    
    Ok(serde_json::json!({
        "success": true,
        "connections": connection_details
    }))
}

#[tauri::command]
pub async fn obs_disconnect(connection_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("OBS disconnect called for connection: {}", connection_name);
    app.obs_plugin().disconnect_obs(&connection_name).await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": true,
        "message": "OBS disconnection initiated"
    }))
}

#[tauri::command]
pub async fn obs_remove_connection(connection_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("OBS remove connection called for connection: {}", connection_name);
    
    // Remove from OBS plugin
    app.obs_plugin().remove_connection(&connection_name).await.map_err(|e| e.to_string())?;
    
    // Also remove from configuration manager
    let mut connections = app.config_manager().get_obs_connections().await;
    connections.retain(|c| c.name != connection_name);
    
    if let Err(e) = app.config_manager().update_obs_connections(connections).await {
        log::warn!("Failed to remove connection from config: {}", e);
    }
    
    Ok(serde_json::json!({
        "success": true,
        "message": "OBS connection removed"
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

#[tauri::command]
pub async fn obs_emit_event(event_data: serde_json::Value, window: tauri::Window) -> Result<(), String> {
    log::info!("Emitting OBS event to frontend: {:?}", event_data);
    if let Err(e) = window.emit("obs_event", event_data) {
        log::error!("Failed to emit OBS event: {}", e);
        return Err(e.to_string());
    }
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
pub async fn get_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Getting application settings");
    
    let config = app.config_manager().get_config().await;
    let config_json = serde_json::to_value(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    Ok(config_json)
}

#[tauri::command]
pub async fn update_settings(settings: serde_json::Value, app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("Updating application settings");
    
    let config: crate::config::AppConfig = serde_json::from_value(settings)
        .map_err(|e| format!("Failed to deserialize settings: {}", e))?;
    
    app.config_manager().update_config(config).await
        .map_err(|e| format!("Failed to update settings: {}", e))
}

#[tauri::command]
pub async fn get_config_stats(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Getting configuration statistics");
    
    match app.config_manager().get_config_stats().await {
        Ok(stats) => {
            let stats_json = serde_json::to_value(stats)
                .map_err(|e| format!("Failed to serialize stats: {}", e))?;
            Ok(stats_json)
        }
        Err(e) => Err(format!("Failed to get config stats: {}", e))
    }
}

#[tauri::command]
pub async fn reset_settings(app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("Resetting settings to defaults");
    
    app.config_manager().reset_to_defaults().await
        .map_err(|e| format!("Failed to reset settings: {}", e))
}

#[tauri::command]
pub async fn export_settings(export_path: String, app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("Exporting settings to: {}", export_path);
    
    let path = std::path::Path::new(&export_path);
    app.config_manager().export_config(path).await
        .map_err(|e| format!("Failed to export settings: {}", e))
}

#[tauri::command]
pub async fn import_settings(import_path: String, app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("Importing settings from: {}", import_path);
    
    let path = std::path::Path::new(&import_path);
    app.config_manager().import_config(path).await
        .map_err(|e| format!("Failed to import settings: {}", e))
}

#[tauri::command]
pub async fn restore_settings_backup(app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("Restoring settings from backup");
    
    app.config_manager().restore_from_backup().await
        .map_err(|e| format!("Failed to restore settings backup: {}", e))
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
    app: State<'_, Arc<App>>,
    window: tauri::Window,
) -> Result<serde_json::Value, String> {
    log::info!("Setting live data streaming for {}: {}", subsystem, enabled);
    
    // Get the app handle for emitting events
    let app_handle = window.app_handle();
    
    if enabled {
        log::info!("Live data streaming enabled for subsystem: {}", subsystem);
        
        // Start streaming by emitting a test event
        // In a real implementation, this would start a background task that continuously emits events
        if let Err(e) = app_handle.emit("live_data", serde_json::json!({
            "subsystem": subsystem,
            "data": format!("[{}] Live data streaming started for {}", chrono::Utc::now().format("%H:%M:%S"), subsystem),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })) {
            log::error!("Failed to emit live data event: {}", e);
        }
        
        // For OBS subsystem, we can start monitoring OBS events
        if subsystem == "obs" {
            // Start monitoring OBS events and forward them to frontend
            let app_handle_clone = app_handle.clone();
            let subsystem_clone = subsystem.clone();
            
            // Spawn a background task to monitor OBS events
            tokio::spawn(async move {
                loop {
                    // Simulate OBS events for now
                    // In a real implementation, this would listen to actual OBS WebSocket events
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    
                    if let Err(e) = app_handle_clone.emit("live_data", serde_json::json!({
                        "subsystem": subsystem_clone,
                        "data": format!("[{}] OBS Event: Scene changed to 'Main Scene'", chrono::Utc::now().format("%H:%M:%S")),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })) {
                        log::error!("Failed to emit OBS live data event: {}", e);
                        break;
                    }
                }
            });
        }
        
        // For PSS subsystem, we can start monitoring PSS events
        if subsystem == "pss" {
            let app_handle_clone = app_handle.clone();
            let subsystem_clone = subsystem.clone();
            
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    
                    if let Err(e) = app_handle_clone.emit("live_data", serde_json::json!({
                        "subsystem": subsystem_clone,
                        "data": format!("[{}] PSS Event: Match data received", chrono::Utc::now().format("%H:%M:%S")),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })) {
                        log::error!("Failed to emit PSS live data event: {}", e);
                        break;
                    }
                }
            });
        }
        
        // For UDP subsystem, we can start monitoring UDP events
        if subsystem == "udp" {
            let app_handle_clone = app_handle.clone();
            let subsystem_clone = subsystem.clone();
            
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    
                    if let Err(e) = app_handle_clone.emit("live_data", serde_json::json!({
                        "subsystem": subsystem_clone,
                        "data": format!("[{}] UDP Event: Datagram received", chrono::Utc::now().format("%H:%M:%S")),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })) {
                        log::error!("Failed to emit UDP live data event: {}", e);
                        break;
                    }
                }
            });
        }
        
    } else {
        log::info!("Live data streaming disabled for subsystem: {}", subsystem);
        
        // Emit a final event to indicate streaming stopped
        if let Err(e) = app_handle.emit("live_data", serde_json::json!({
            "subsystem": subsystem,
            "data": format!("[{}] Live data streaming stopped for {}", chrono::Utc::now().format("%H:%M:%S"), subsystem),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })) {
            log::error!("Failed to emit live data stop event: {}", e);
        }
    }
    
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
pub async fn start_live_data(subsystem: String, app: State<'_, Arc<App>>, window: tauri::Window) -> Result<(), String> {
    set_live_data_streaming(subsystem, true, app, window).await?;
    Ok(())
}

#[tauri::command]
pub async fn stop_live_data(subsystem: String, app: State<'_, Arc<App>>, window: tauri::Window) -> Result<(), String> {
    set_live_data_streaming(subsystem, false, app, window).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_live_data(subsystem: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Getting live data for subsystem: {}", subsystem);
    
    match subsystem.as_str() {
        "obs" => {
            // Get OBS live data
            let obs_status = app.obs_plugin().get_obs_status().await;
            match obs_status {
                Ok(status) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "data": {
                            "subsystem": "obs",
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "is_recording": status.is_recording,
                            "is_streaming": status.is_streaming,
                            "cpu_usage": status.cpu_usage,
                            "recording_connection": status.recording_connection,
                            "streaming_connection": status.streaming_connection
                        }
                    }))
                }
                Err(e) => {
                    Ok(serde_json::json!({
                        "success": false,
                        "error": e.to_string()
                    }))
                }
            }
        }
        "pss" => {
            // Get PSS live data from UDP plugin (PSS events come through UDP)
            let udp_stats = app.udp_plugin().get_stats();
            Ok(serde_json::json!({
                "success": true,
                "data": {
                    "subsystem": "pss",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "packets_received": udp_stats.packets_received,
                    "packets_parsed": udp_stats.packets_parsed,
                    "parse_errors": udp_stats.parse_errors,
                    "connected_clients": udp_stats.connected_clients,
                    "last_packet_time": udp_stats.last_packet_time.map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                }
            }))
        }
        "udp" => {
            // Get UDP live data
            let udp_status = app.udp_plugin().get_status();
            Ok(serde_json::json!({
                "success": true,
                "data": {
                    "subsystem": "udp",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "status": format!("{:?}", udp_status),
                    "is_running": matches!(udp_status, crate::plugins::plugin_udp::UdpServerStatus::Running)
                }
            }))
        }
        _ => {
            Ok(serde_json::json!({
                "success": false,
                "error": format!("Unknown subsystem: {}", subsystem)
            }))
        }
    }
}

#[tauri::command]
pub async fn obs_get_debug_info(connection_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Getting OBS debug info for connection: {}", connection_name);
    
    match app.obs_plugin().get_latest_events(&connection_name).await {
        Ok(debug_info) => Ok(serde_json::json!({
            "success": true,
            "data": debug_info
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn obs_toggle_full_events(enabled: bool, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Toggling OBS full events display: {}", enabled);
    
    app.obs_plugin().toggle_full_events(enabled).await;
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Full OBS events display {}", if enabled { "enabled" } else { "disabled" })
    }))
}

#[tauri::command]
pub async fn obs_get_full_events_setting(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Getting OBS full events setting");
    
    let enabled = app.obs_plugin().get_full_events_setting().await;
    
    Ok(serde_json::json!({
        "success": true,
        "enabled": enabled
    }))
}

#[tauri::command]
pub async fn obs_emit_event_to_frontend(event_data: serde_json::Value, window: tauri::Window) -> Result<serde_json::Value, String> {
    log::info!("Emitting OBS event to frontend: {:?}", event_data);
    
    match window.emit("obs_event", event_data) {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Event emitted successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn obs_get_recent_events(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    let events = app.obs_plugin().get_recent_events().await;
    
    // Convert RecentEvent structs to JSON
    let event_json: Vec<serde_json::Value> = events.into_iter().map(|event| {
        serde_json::json!({
            "connection_name": event.connection_name,
            "event_type": event.event_type,
            "data": event.data,
            "timestamp": event.timestamp.to_rfc3339()
        })
    }).collect();
    
    Ok(serde_json::json!({
        "success": true,
        "events": event_json
    }))
}

// CPU Monitoring Commands
#[tauri::command]
pub async fn cpu_get_process_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("[CPU_CMD] Getting process data...");
    
    let process_data = app.cpu_monitor_plugin().get_process_cpu_data().await;
    log::info!("[CPU_CMD] Process data count: {}", process_data.len());
    
    Ok(serde_json::json!({
        "success": true,
        "processes": process_data
    }))
}

#[tauri::command]
pub async fn cpu_get_system_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("[CPU_CMD] Getting system data...");
    
    let system_data = app.cpu_monitor_plugin().get_system_cpu_data().await;
    log::info!("[CPU_CMD] System data available: {}", system_data.is_some());
    
    Ok(serde_json::json!({
        "success": true,
        "system": system_data
    }))
}

#[tauri::command]
pub async fn cpu_get_obs_usage(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    let obs_cpu = app.cpu_monitor_plugin().get_obs_cpu_usage().await;
    
    Ok(serde_json::json!({
        "success": true,
        "obs_cpu_percent": obs_cpu
    }))
}

#[tauri::command]
pub async fn cpu_update_config(app: State<'_, Arc<App>>, config: crate::plugins::CpuMonitorConfig) -> Result<serde_json::Value, String> {
    match app.cpu_monitor_plugin().update_config(config).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "CPU monitoring configuration updated"
        })),
        Err(e) => Err(format!("Failed to update CPU monitoring config: {}", e))
    }
} 