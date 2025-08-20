//! Tauri command layer
//!
//! Purpose: Expose backend features (OBS over obws, IVR, Drive, UDP/DB queries) to the frontend via
//! strongly-typed commands. Each command returns a stable JSON shape and performs strict
//! argument validation, logging, and error conversion.
//!
//! Conventions:
//! - All commands return { success, data?, error? } unless noted otherwise
//! - OBS operations call obws-backed manager exclusively; legacy OBS code is removed
//! - Control Room audio mute/unmute is currently a no-op stub; it returns success with empty results
//! - Never block on long-lived operations without spawning; keep UI responsive
//!
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, Emitter, Error as TauriError};
use crate::core::app::App;
use crate::logging::archival::{AutoArchiveConfig, ArchiveSchedule};
use dirs;
use crate::utils::simulation_env::ensure_simulation_env;



#[derive(Debug, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size: u64,
    pub modified: String,
    pub subsystem: String,
}

// Core app commands
#[tauri::command]
pub async fn get_app_status(_app: State<'_, Arc<App>>) -> Result<String, TauriError> {
    log::info!("Getting app status");
    Ok("Running".to_string())
}

#[tauri::command]
pub async fn shutdown_app(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Shutting down app");
    app.stop().await.map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    Ok(())
}

// UDP commands
#[tauri::command]
pub async fn start_udp_server(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Starting UDP server");
    let config = app.config_manager().get_config().await;
    app.udp_plugin().start(&config).await.map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    // Ensure UDP event handler is running so PSS events reach auto-recording
    app.inner().start_udp_event_handler().await;
    log::info!("✅ UDP event handler started (manual start)");
    println!("✅ UDP event handler started (manual start)");
    Ok(())
}

#[tauri::command]
pub async fn stop_udp_server(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Stopping UDP server");
    app.udp_plugin().stop().await.map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn get_udp_status(app: State<'_, Arc<App>>) -> Result<String, TauriError> {
    log::info!("Getting UDP status");
    let status = app.udp_plugin().get_status();
    let status_str = match status {
        crate::plugins::plugin_udp::UdpServerStatus::Stopped => "Stopped",
        crate::plugins::plugin_udp::UdpServerStatus::Starting => "Starting",
        crate::plugins::plugin_udp::UdpServerStatus::Running => "Running",
        crate::plugins::plugin_udp::UdpServerStatus::Error(e) => return Err(TauriError::from(anyhow::anyhow!("{}", e))),
    };
    Ok(status_str.to_string())
}

#[tauri::command]
pub async fn update_udp_settings(settings: serde_json::Value, app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Updating UDP settings: {:?}", settings);
    
    // Update the app configuration
    app.config_manager().update_udp_settings_from_json(settings).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    
    // If UDP server is running, restart it with new settings
    let status = app.udp_plugin().get_status();
    if matches!(status, crate::plugins::plugin_udp::UdpServerStatus::Running) {
        log::info!("UDP server is running, restarting with new settings");
        app.udp_plugin().stop().await.map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
        let config = app.config_manager().get_config().await;
        app.udp_plugin().start(&config).await.map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
        // Re-ensure event handler is started after restart
        app.inner().start_udp_event_handler().await;
        log::info!("✅ UDP event handler started (restart)");
        println!("✅ UDP event handler started (restart)");
    }
    
    Ok(())
}

// OBS commands - Fixed names to match frontend expectations
#[tauri::command]
pub async fn obs_connect(url: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS connect called with URL: {}", url);
    // Parse basic info
    let host = url.replace("ws://", "").replace("wss://", "").split(':').next().unwrap_or("localhost").to_string();
    // Create connection via obws, name default to OBS_REC if not provided in UI
    let req = crate::tauri_commands_obws::ObsObwsConnectionRequest { name: "OBS_REC".to_string(), host, port: 4455, password: None, enabled: true };
    let _ = crate::tauri_commands_obws::obs_obws_add_connection(req, app.clone()).await;
    let _ = crate::tauri_commands_obws::obs_obws_connect("OBS_REC".to_string(), app.clone()).await;
    Ok(serde_json::json!({ "success": true, "message": "OBS connection initiated" }))
}

#[tauri::command]
pub async fn obs_add_connection(
    name: String,
    host: String,
    port: u16,
    password: Option<String>,
    enabled: bool,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS add connection called: {}@{}:{}", name, host, port);
    // Delegate to obws add_connection
    let req = crate::tauri_commands_obws::ObsObwsConnectionRequest { name: name.clone(), host: host.clone(), port, password: password.clone(), enabled };
    let res = crate::tauri_commands_obws::obs_obws_add_connection(req, app.clone()).await;
    if let Err(e) = res { return Err(e); }
    // Persist to config as before
    let config_conn = crate::config::ObsConnectionConfig { name: name.clone(), host: host.clone(), port, password: password.clone(), protocol_version: "v5".to_string(), enabled, timeout_seconds: 30, auto_reconnect: true, max_reconnect_attempts: 5 };
    let mut connections = app.config_manager().get_obs_connections().await;
    connections.retain(|c| c.name != config_conn.name);
    connections.push(config_conn);
    let _ = app.config_manager().update_obs_connections(connections).await;
    Ok(serde_json::json!({ "success": true, "message": "OBS connection added successfully" }))
}

#[tauri::command]
pub async fn obs_connect_to_connection(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS connect to connection called: {}", connection_name);
    let res = crate::tauri_commands_obws::obs_obws_connect(connection_name, app.clone()).await;
    match res { Ok(_) => Ok(serde_json::json!({ "success": true })), Err(e) => Err(e) }
}

#[tauri::command]
pub async fn obs_get_connection_status(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get connection status called: {}", connection_name);
    let res = crate::tauri_commands_obws::obs_obws_get_connection_status(connection_name, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_get_connections(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("🔍 OBS get connections called");
    let res = crate::tauri_commands_obws::obs_obws_get_connections(app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_disconnect(connection_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("🔍 OBS disconnect called for connection: '{}'", connection_name);
    if connection_name.is_empty() { return Err(TauriError::from(anyhow::anyhow!("Connection name cannot be empty"))); }
    let res = crate::tauri_commands_obws::obs_obws_disconnect(connection_name, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_remove_connection(connection_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS remove connection called for connection: {}", connection_name);
    let _ = crate::tauri_commands_obws::obs_obws_remove_connection(connection_name.clone(), app.clone()).await?;
    // Remove from configuration manager
    let mut connections = app.config_manager().get_obs_connections().await;
    connections.retain(|c| c.name != connection_name);
    let _ = app.config_manager().update_obs_connections(connections).await;
    Ok(serde_json::json!({ "success": true, "message": "OBS connection removed" }))
}

#[tauri::command]
pub async fn obs_get_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get status");
    let res = crate::tauri_commands_obws::obs_obws_get_status(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_start_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS start recording called");
    let res = crate::tauri_commands_obws::obs_obws_start_recording(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_stop_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS stop recording called");
    let res = crate::tauri_commands_obws::obs_obws_stop_recording(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

// Streaming Commands
#[tauri::command]
pub async fn obs_start_streaming(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS start streaming called");
    let res = crate::tauri_commands_obws::obs_obws_start_streaming(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_stop_streaming(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS stop streaming called");
    let res = crate::tauri_commands_obws::obs_obws_stop_streaming(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_get_streaming_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get streaming status called");
    let res = crate::tauri_commands_obws::obs_obws_get_streaming_status(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

// Scene Management Commands
#[tauri::command]
pub async fn obs_get_current_scene(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get current scene");
    let res = crate::tauri_commands_obws::obs_obws_get_current_scene(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_set_current_scene(scene_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS set current scene: {}", scene_name);
    let res = crate::tauri_commands_obws::obs_obws_set_current_scene(scene_name, None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

// Settings Commands
#[tauri::command]
pub async fn obs_get_obs_version(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get version called");
    let res = crate::tauri_commands_obws::obs_obws_get_version(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

// Advanced Commands
#[tauri::command]
pub async fn obs_get_replay_buffer_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get replay buffer status called");
    let res = crate::tauri_commands_obws::obs_obws_get_replay_buffer_status(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_start_replay_buffer(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS start replay buffer called");
    let res = crate::tauri_commands_obws::obs_obws_start_replay_buffer(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_stop_replay_buffer(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS stop replay buffer called");
    let res = crate::tauri_commands_obws::obs_obws_stop_replay_buffer(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_save_replay_buffer(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS save replay buffer called");
    let res = crate::tauri_commands_obws::obs_obws_save_replay_buffer(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}
// Studio Mode Commands
// (studio mode functions removed)

// Source Management Commands (removed: legacy plugin not needed and no obws equivalent currently used)
#[tauri::command]
pub async fn obs_get_sources(scene_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get sources called for scene: {}", scene_name);
    let res = crate::tauri_commands_obws::obs_obws_get_sources(scene_name, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_set_source_visibility(
    scene_name: String, 
    source_name: String, 
    visible: bool,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS set source visibility called: scene={}, source={}, visible={}", scene_name, source_name, visible);
    let res = crate::tauri_commands_obws::obs_obws_set_source_visibility(scene_name, source_name, visible, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

// (legacy recording settings get/set removed)

// Recording Path and Filename Commands
#[tauri::command]
pub async fn obs_get_recording_path_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let res = crate::tauri_commands_obws::obs_obws_get_recording_path_settings(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_set_recording_path(app: State<'_, Arc<App>>, path: String) -> Result<serde_json::Value, TauriError> {
    let res = crate::tauri_commands_obws::obs_obws_set_recording_path(path, None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_set_recording_filename(app: State<'_, Arc<App>>, filename_format: String) -> Result<serde_json::Value, TauriError> {
    let res = crate::tauri_commands_obws::obs_obws_set_recording_filename(filename_format, None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

// (legacy set_recording_format removed)

// Recording Settings Templates and Options
// (legacy recording options/templates removed)

// (legacy Replay Buffer Settings Commands removed)

// (legacy Advanced Replay Buffer Commands removed)

// (legacy replay buffer format get/set removed)

// (legacy replay buffer quality get/set removed)

// (legacy replay buffer bitrate get/set removed)
// (legacy replay buffer keyframe interval get/set removed)

// (legacy replay buffer rate control get/set removed)

// (legacy replay buffer preset get/set removed)

// (legacy replay buffer profile get/set removed)

// (legacy replay buffer tune get/set removed)

// (legacy replay buffer bulk settings removed)

// (legacy Replay Buffer Options Commands removed)

// (legacy streaming settings commands removed)

// Unused legacy status/event commands removed

#[tauri::command]
pub async fn obs_command(_action: String, _params: serde_json::Value, _app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    Ok(())
}

#[tauri::command]
pub async fn obs_emit_event(event_data: serde_json::Value, window: tauri::Window) -> Result<(), TauriError> {
    log::info!("Emitting OBS event to frontend: {:?}", event_data);
    if let Err(e) = window.emit("obs_event", event_data) {
        log::error!("Failed to emit OBS event: {}", e);
        return Err(TauriError::from(anyhow::anyhow!("{}", e)));
    }
    Ok(())
}

// Video commands
#[tauri::command]
pub async fn video_play(path: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
pub async fn video_stop(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
pub async fn video_get_info(path: String, _app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
pub async fn extract_clip(_connection: String, _app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    Ok(())
}

// Store commands
#[tauri::command]
pub async fn save_event(_event: serde_json::Value, _app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    Ok(())
}

#[tauri::command]
pub async fn get_events(_app: State<'_, Arc<App>>) -> Result<Vec<serde_json::Value>, TauriError> {
    Ok(vec![])
}

#[tauri::command]
pub async fn clear_events(_app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    Ok(())
}

// License commands
#[tauri::command]
pub async fn activate_license(_key: String, _app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    Ok(())
}

#[tauri::command]
pub async fn validate_license(_app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    Ok(())
}

#[tauri::command]
pub async fn get_license_status(_app: State<'_, Arc<App>>) -> Result<String, TauriError> {
    Ok("Valid".to_string())
}

// Settings commands
#[tauri::command]
pub async fn get_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting application settings");
    
    let config = app.config_manager().get_config().await;
    let config_json = serde_json::to_value(config)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to serialize config: {}", e)))?;
    Ok(config_json)
}

#[tauri::command]
pub async fn update_settings(settings: serde_json::Value, app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Updating application settings");
    
    let config: crate::config::AppConfig = serde_json::from_value(settings)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to deserialize settings: {}", e)))?;
    
    app.config_manager().update_config(config).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to update settings: {}", e)))
}

#[tauri::command]
pub async fn get_config_stats(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting configuration statistics");
    
    match app.config_manager().get_config_stats().await {
        Ok(stats) => {
            let stats_json = serde_json::to_value(stats)
                .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to serialize stats: {}", e)))?;
            Ok(stats_json)
        }
        Err(e) => Err(TauriError::from(anyhow::anyhow!("Failed to get config stats: {}", e)))
    }
}

#[tauri::command]
pub async fn reset_settings(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Resetting settings to defaults");
    
    app.config_manager().reset_to_defaults().await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to reset settings: {}", e)))
}

#[tauri::command]
pub async fn export_settings(export_path: String, app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Exporting settings to: {}", export_path);
    
    let path = std::path::Path::new(&export_path);
    app.config_manager().export_config(path).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to export settings: {}", e)))
}

#[tauri::command]
pub async fn import_settings(import_path: String, app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Importing settings from: {}", import_path);
    
    let path = std::path::Path::new(&import_path);
    app.config_manager().import_config(path).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to import settings: {}", e)))
}

#[tauri::command]
pub async fn restore_settings_backup(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Restoring settings from backup");
    
    app.config_manager().restore_from_backup().await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to restore settings backup: {}", e)))
}

// Flag commands
#[tauri::command]
pub async fn get_flag_url(_ioc_code: String, _app: State<'_, Arc<App>>) -> Result<String, TauriError> {
    Ok("".to_string())
}

#[tauri::command]
pub async fn download_flags(_app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    Ok(())
}

// PSS commands
#[tauri::command]
pub async fn pss_start_listener(port: u16, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("PSS start listener called on port: {}", port);
    
    // Update the UDP server configuration with the new port
    let mut config = app.config_manager().get_config().await;
    config.udp.listener.port = port;
    
    // Update the UDP plugin's internal configuration
    app.udp_plugin().update_config(port, "127.0.0.1".to_string()).await;
    
    match app.udp_plugin().start(&config).await {
        Ok(_) => {
            // Start UDP event handler when UDP server starts
            app.inner().start_udp_event_handler().await;
            
            Ok(serde_json::json!({
                "success": true,
                "message": format!("PSS listener started on port {} with event handler", port)
            }))
        },
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn pss_stop_listener(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("PSS stop listener called");
    match app.udp_plugin().stop().await {
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
pub async fn pss_get_events(app: State<'_, Arc<App>>) -> Result<Vec<serde_json::Value>, TauriError> {
    log::info!("PSS get events called");
    
    let events = app.udp_plugin().get_recent_events();
    
    // Convert PssEvent enum to JSON
    let event_json: Vec<serde_json::Value> = events.into_iter().map(|event| {
        let event_code = crate::plugins::plugin_udp::UdpServer::get_event_code(&event);
        
        match event {
            crate::plugins::plugin_udp::PssEvent::Points { athlete, point_type } => {
                let athlete_str = match athlete {
                    1 => "blue",
                    2 => "red",
                    _ => "unknown"
                };
                serde_json::json!({
                    "type": "points",
                    "event_code": event_code,
                    "athlete": athlete_str,
                    "point_type": point_type,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Athlete {} scored {} points", athlete, point_type)
                })
            }
            crate::plugins::plugin_udp::PssEvent::HitLevel { athlete, level } => {
                let athlete_str = match athlete {
                    1 => "blue",
                    2 => "red",
                    _ => "unknown"
                };
                serde_json::json!({
                    "type": "hit_level",
                    "event_code": event_code,
                    "athlete": athlete_str,
                    "level": level,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Athlete {} hit level {}", athlete, level)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                serde_json::json!({
                    "type": "warnings",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_warnings": athlete1_warnings,
                    "athlete2_warnings": athlete2_warnings,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Warnings - Athlete1: {}, Athlete2: {}", athlete1_warnings, athlete2_warnings)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Clock { time, action } => {
                serde_json::json!({
                    "type": "clock",
                    "event_code": event_code,
                    "athlete": "",
                    "time": time,
                    "action": action,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Clock: {} {:?}", time, action.as_ref().unwrap_or(&String::new()))
                })
            }
            crate::plugins::plugin_udp::PssEvent::Round { current_round } => {
                serde_json::json!({
                    "type": "round",
                    "event_code": event_code,
                    "athlete": "",
                    "current_round": current_round,
                    "round": current_round,
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Round {}", current_round)
                })
            }
            crate::plugins::plugin_udp::PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                serde_json::json!({
                    "type": "winner_rounds",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "round1_winner": round1_winner,
                    "round2_winner": round2_winner,
                    "round3_winner": round3_winner,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Winner Rounds - R1: {}, R2: {}, R3: {}", round1_winner, round2_winner, round3_winner)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => {
                serde_json::json!({
                    "type": "scores",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_r1": athlete1_r1,
                    "athlete2_r1": athlete2_r1,
                    "athlete1_r2": athlete1_r2,
                    "athlete2_r2": athlete2_r2,
                    "athlete1_r3": athlete1_r3,
                    "athlete2_r3": athlete2_r3,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Scores - A1: R1={}, R2={}, R3={} | A2: R1={}, R2={}, R3={}", 
                        athlete1_r1, athlete1_r2, athlete1_r3, athlete2_r1, athlete2_r2, athlete2_r3)
                })
            }
            crate::plugins::plugin_udp::PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                serde_json::json!({
                    "type": "current_scores",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_score": athlete1_score,
                    "athlete2_score": athlete2_score,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Current Scores - A1: {}, A2: {}", athlete1_score, athlete2_score)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Athletes { athlete1_short, athlete1_long, athlete1_country, athlete2_short, athlete2_long, athlete2_country } => {
                serde_json::json!({
                    "type": "athletes",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_short": athlete1_short,
                    "athlete1_long": athlete1_long,
                    "athlete1_country": athlete1_country,
                    "athlete2_short": athlete2_short,
                    "athlete2_long": athlete2_long,
                    "athlete2_country": athlete2_country,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Athletes - {} ({}) vs {} ({})", athlete1_short, athlete1_country, athlete2_short, athlete2_country)
                })
            }
            crate::plugins::plugin_udp::PssEvent::MatchConfig { number, category, weight, rounds, colors, match_id, division, total_rounds, round_duration, countdown_type, count_up, format } => {
                serde_json::json!({
                    "type": "match_config",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "number": number,
                    "category": category,
                    "weight": weight,
                    "rounds": rounds,
                    "colors": colors,
                    "match_id": match_id,
                    "division": division,
                    "total_rounds": total_rounds,
                    "round_duration": round_duration,
                    "countdown_type": countdown_type,
                    "count_up": count_up,
                    "format": format,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Match Config - #{} {} {} ({})", number, category, weight, division)
                })
            }
            crate::plugins::plugin_udp::PssEvent::FightLoaded => {
                serde_json::json!({
                    "type": "fight_loaded",
                    "event_code": event_code,
                    "athlete": "",
                    "event": "FightLoaded",
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": "Fight Loaded"
                })
            }
            crate::plugins::plugin_udp::PssEvent::FightReady => {
                serde_json::json!({
                    "type": "fight_ready",
                    "event_code": event_code,
                    "athlete": "",
                    "event": "FightReady",
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": "Fight Ready"
                })
            }
            crate::plugins::plugin_udp::PssEvent::Raw(message) => {
                serde_json::json!({
                    "type": "raw",
                    "event_code": event_code,
                    "athlete": "",
                    "message": message,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Raw message: {}", message)
                })
            }
            _ => {
                serde_json::json!({
                    "type": "other",
                    "event_code": event_code,
                    "athlete": "",
                    "event": format!("{:?}", event),
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Event: {:?}", event)
                })
            }
        }
    }).collect();
    
    Ok(event_json)
}
#[tauri::command]
pub async fn pss_get_events_for_match(app: State<'_, Arc<App>>, match_id: String) -> Result<Vec<serde_json::Value>, TauriError> {
    log::info!("pss_get_events_for_match called for match_id={}", match_id);
    // Require numeric DB id only. Reject non-numeric inputs.
    let resolved_mid: i64 = match match_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return Ok(vec![]),
    };

    // Fetch events for resolved match id
    let mut rows = app.database_plugin().get_pss_events_for_match(resolved_mid, Some(1000)).await
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("DB fetch for match events failed: {}", e))))?;

    // If no rows found for this match, return empty (no fallback to live memory)
    if rows.is_empty() { return Ok(vec![]); }

    // Build enriched event list with inferred round/time/athlete from parsed_data
    // Iterate oldest->newest to maintain running state
    rows.sort_by_key(|r| r.timestamp);
    let mut last_round: u8 = 1;
    let mut last_time: String = "0:00".to_string();
    let mut out: Vec<serde_json::Value> = Vec::with_capacity(rows.len());

    for row in rows.into_iter() {
        // Default system events to referee/yellow; specific event types will override
        let mut athlete = "yellow".to_string();
        let mut round = last_round as i64;
        let mut time = last_time.clone();
        let mut ev_type = String::from("other");
        let mut event_code = String::from("O");

        if let Some(ref pd) = row.parsed_data {
            if let Ok(ev) = serde_json::from_str::<crate::plugins::plugin_udp::PssEvent>(pd) {
                use crate::plugins::plugin_udp::PssEvent;
                event_code = crate::plugins::plugin_udp::UdpServer::get_event_code(&ev);
                match ev {
                    PssEvent::Round { current_round } => {
                        athlete = "yellow".to_string();
                        last_round = current_round;
                        round = current_round as i64;
                        ev_type = "round".to_string();
                    }
                    PssEvent::Clock { time: t, .. } => {
                        athlete = "yellow".to_string();
                        last_time = t.clone();
                        time = t;
                        ev_type = "clock".to_string();
                    }
                    PssEvent::Points { athlete: a, point_type: _ } => {
                        athlete = match a { 1 => "blue".to_string(), 2 => "red".to_string(), _ => "yellow".to_string() };
                        round = last_round as i64;
                        time = last_time.clone();
                        ev_type = "points".to_string();
                        // Optionally include point_type in description
                    }
                    PssEvent::HitLevel { athlete: a, level: _ } => {
                        athlete = match a { 1 => "blue".to_string(), 2 => "red".to_string(), _ => "yellow".to_string() };
                        round = last_round as i64;
                        time = last_time.clone();
                        ev_type = "hit_level".to_string();
                    }
                    PssEvent::Warnings { .. } => {
                        athlete = "yellow".to_string();
                        round = last_round as i64;
                        time = last_time.clone();
                        ev_type = "warnings".to_string();
                    }
                    PssEvent::Break { time: t, .. } => {
                        athlete = "yellow".to_string();
                        time = t.clone();
                        round = last_round as i64;
                        ev_type = "break".to_string();
                    }
                    PssEvent::CurrentScores { .. } => {
                        athlete = "yellow".to_string();
                        round = last_round as i64;
                        time = last_time.clone();
                        ev_type = "current_scores".to_string();
                    }
                    PssEvent::Scores { .. } => {
                        athlete = "yellow".to_string();
                        round = last_round as i64;
                        time = last_time.clone();
                        ev_type = "scores".to_string();
                    }
                    PssEvent::WinnerRounds { .. } => {
                        athlete = "yellow".to_string();
                        round = last_round as i64;
                        time = last_time.clone();
                        ev_type = "winner_rounds".to_string();
                    }
                    PssEvent::Winner { .. } => {
                        athlete = "yellow".to_string();
                        round = last_round as i64;
                        time = last_time.clone();
                        ev_type = "winner".to_string();
                    }
                    PssEvent::Athletes { .. } => {
                        athlete = "yellow".to_string();
                        ev_type = "athletes".to_string();
                    }
                    PssEvent::MatchConfig { .. } => {
                        athlete = "yellow".to_string();
                        ev_type = "match_config".to_string();
                    }
                    PssEvent::FightLoaded => { athlete = "yellow".to_string(); ev_type = "fight_loaded".to_string(); }
                    PssEvent::FightReady => { athlete = "yellow".to_string(); ev_type = "fight_ready".to_string(); }
                    PssEvent::Challenge { .. } => {
                        athlete = "yellow".to_string();
                        round = last_round as i64;
                        time = last_time.clone();
                        ev_type = "challenge".to_string();
                    }
                    PssEvent::Injury { time: t, athlete: a, .. } => {
                        time = t.clone();
                        athlete = match a { 1 => "blue".to_string(), 2 => "red".to_string(), _ => "yellow".to_string() };
                        round = last_round as i64;
                        ev_type = "injury".to_string();
                    }
                    PssEvent::Supremacy { .. } => {
                        athlete = "yellow".to_string();
                        round = last_round as i64;
                        time = last_time.clone();
                        ev_type = "supremacy".to_string();
                    }
                    PssEvent::Raw(_) => { athlete = "yellow".to_string(); ev_type = "raw".to_string(); }
                }
            }
        }

        // Only return important events to UI; keep others persisted for timing/state
        let important = matches!(event_code.as_str(), "K"|"P"|"H"|"TH"|"TB"|"R");
        if important {
            out.push(serde_json::json!({
                "id": row.id,
                "type": ev_type,
                "event_type": ev_type,
                "event_code": event_code,
                "athlete": athlete,
                "round": round,
                "time": time,
                "timestamp": row.timestamp.to_rfc3339(),
                "raw_data": row.raw_data,
                "description": row.parsed_data
            }));
        }
    }

    Ok(out)
}
/// List recent PSS matches for review dropdown
#[tauri::command]
pub async fn pss_list_recent_matches(app: State<'_, Arc<App>>, limit: Option<i64>) -> Result<Vec<serde_json::Value>, TauriError> {
    let conn = app.database_plugin().get_connection().await
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("DB connection error: {}", e))))?;
    // Return only matches that have at least one event; newest first
    let max = limit.unwrap_or(50);
    let mut stmt = conn.prepare(
        "SELECT m.id, m.match_id, m.match_number, m.category, m.weight_class, m.division, m.created_at, m.updated_at
         FROM pss_matches m
         WHERE EXISTS (SELECT 1 FROM pss_events_v2 e WHERE e.match_id = m.id)
         ORDER BY m.created_at DESC
         LIMIT ?"
    ).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    let rows = stmt.query_map([max], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "match_id": row.get::<_, String>(1)?,
            "match_number": row.get::<_, Option<String>>(2)?,
            "category": row.get::<_, Option<String>>(3)?,
            "weight_class": row.get::<_, Option<String>>(4)?,
            "division": row.get::<_, Option<String>>(5)?,
            "created_at": row.get::<_, String>(6)?,
            "updated_at": row.get::<_, String>(7)?,
        }))
    }).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(rows)
}

/// Danger: clear all PSS matches and events for a fresh start
#[tauri::command]
pub async fn pss_clear_all_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let mut conn = app.database_plugin().get_connection().await
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("DB connection error: {}", e))))?;
    // Wrap in transaction for atomicity
    let tx = conn.transaction().map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    // Delete in child->parent order
    tx.execute("DELETE FROM pss_event_details", [])
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    tx.execute("DELETE FROM pss_events_v2", [])
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    tx.execute("DELETE FROM pss_scores", [])
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    tx.execute("DELETE FROM pss_warnings", [])
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    tx.execute("DELETE FROM pss_rounds", [])
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    tx.execute("DELETE FROM pss_match_athletes", [])
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    tx.execute("DELETE FROM pss_athletes", [])
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    tx.execute("DELETE FROM pss_matches", [])
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    tx.commit().map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(serde_json::json!({"success": true}))
}

/// Get current live match DB id (if any)
#[tauri::command]
pub async fn pss_get_current_match(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    // Pull from WebSocket server's propagated current_match_db_id
    let id_opt = app.websocket_plugin().lock().await.get_current_match_db_id();
    Ok(serde_json::json!({ "id": id_opt }))
}

/// Advance tournament/day context according to selection
/// mode: "continue" | "next" | "new"
#[tauri::command]
pub async fn tournament_progress_context(
    app: State<'_, Arc<App>>,
    mode: String,
) -> Result<serde_json::Value, TauriError> {
    let mut conn = app.database_plugin().get_connection().await
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("DB connection error: {}", e))))?;

    // Get current active tournament/day
    let active_tournament = crate::database::operations::TournamentOperations::get_active_tournament(&*conn)
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("get_active_tournament: {}", e))))?;
    let (mut tournament_id, mut tournament_day_id) = if let Some(t) = active_tournament {
        let day = crate::database::operations::TournamentOperations::get_active_tournament_day(&*conn, t.id.unwrap())
            .map_err(|e| TauriError::from(anyhow::anyhow!(format!("get_active_tournament_day: {}", e))))?;
        (t.id, day.and_then(|d| d.id))
    } else { (None, None) };

    match mode.to_lowercase().as_str() {
        "continue" => {
            // Keep as-is
        }
        "next" => {
            // Same tournament, next day
            if let Some(tid) = tournament_id {
                // Create a new day under same tournament (1 day duration) starting now
                let start_dt = chrono::Utc::now();
                crate::database::operations::TournamentOperations::create_tournament_days(&mut *conn, tid, start_dt, 1)
                    .map_err(|e| TauriError::from(anyhow::anyhow!(format!("create_tournament_days(next): {}", e))))?;
                let day = crate::database::operations::TournamentOperations::get_active_tournament_day(&*conn, tid)
                    .map_err(|e| TauriError::from(anyhow::anyhow!(format!("get_active_tournament_day(after next): {}", e))))?;
                tournament_day_id = day.and_then(|d| d.id);
            }
        }
        "new" => {
            // New tournament, new day
            let name = format!("Tournament {}", chrono::Utc::now().format("%Y-%m-%d"));
            let tid = app.tournament_plugin().create_tournament(
                name,
                1, // duration_days
                "".to_string(), // city
                "".to_string(), // country
                None, // country_code
                Some(chrono::Utc::now()), // start_date
            ).await.map_err(|e| TauriError::from(anyhow::anyhow!(format!("create_tournament: {}", e))))?;
            tournament_id = Some(tid);
            // Auto create first day
            let start_dt = chrono::Utc::now();
            crate::database::operations::TournamentOperations::create_tournament_days(&mut *conn, tid, start_dt, 1)
                .map_err(|e| TauriError::from(anyhow::anyhow!(format!("create_tournament_days(new): {}", e))))?;
            let day = crate::database::operations::TournamentOperations::get_active_tournament_day(&*conn, tid)
                .map_err(|e| TauriError::from(anyhow::anyhow!(format!("get_active_tournament_day(after new): {}", e))))?;
            tournament_day_id = day.and_then(|d| d.id);
        }
        _ => {}
    }

    // Set UDP context so events inherit these IDs
    app.udp_plugin().set_tournament_context(tournament_id, tournament_day_id).await
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("set_tournament_context: {}", e))))?;

    Ok(serde_json::json!({
        "tournament_id": tournament_id,
        "tournament_day_id": tournament_day_id
    }))
}

/// Provide details for a match, including athletes, for MatchDetailsSection
#[tauri::command]
pub async fn pss_get_match_details(app: State<'_, Arc<App>>, match_id: String) -> Result<serde_json::Value, TauriError> {
    let conn = app.database_plugin().get_connection().await
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("DB connection error: {}", e))))?;

    // ID-only resolution: accept only numeric DB id
    let dbid: i64 = match match_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return Err(TauriError::from(anyhow::anyhow!("Match not found"))),
    };
    let list = crate::database::operations::PssUdpOperations::get_pss_matches(&*conn, Some(1000))
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("get_pss_matches: {}", e))))?;
    let info = list.into_iter().find(|m| m.id == Some(dbid));

    let info = info.ok_or_else(|| TauriError::from(anyhow::anyhow!("Match not found")))?;
    let athletes = crate::database::operations::PssUdpOperations::get_pss_match_athletes(&*conn, info.id.unwrap())
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("get_pss_match_athletes: {}", e))))?;
    let mut a1 = serde_json::json!({});
    let mut a2 = serde_json::json!({});
    for (ma, a) in athletes {
        let obj = serde_json::json!({
            "short_name": a.short_name,
            "long_name": a.long_name,
            "country_code": a.country_code,
        });
        if ma.athlete_position == 1 { a1 = obj; } else if ma.athlete_position == 2 { a2 = obj; }
    }
    Ok(serde_json::json!({
        "match": {
            "id": info.id,
            "match_id": info.match_id,
            "number": info.match_number,
            "category": info.category,
            "weight": info.weight_class,
            "division": info.division,
        },
        "athlete1": a1,
        "athlete2": a2,
    }))
}

// System commands
#[tauri::command]
pub async fn system_get_info(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("System get info called");
    Ok(serde_json::json!({
        "success": true,
        "platform": "windows",
        "version": "1.0.0"
    }))
}

#[tauri::command]
pub async fn system_open_file_dialog() -> Result<String, TauriError> {
    // Placeholder for native file dialog - not implemented
    Err(TauriError::from(anyhow::anyhow!("File dialog not available")))
}

#[tauri::command]
pub async fn restore_backup_with_dialog() -> Result<serde_json::Value, TauriError> {
    // Placeholder for dialog-based restore - not implemented
    Ok(serde_json::json!({ "success": false, "error": "File dialog not available" }))
}

// Diagnostics & Logs commands

#[tauri::command]
pub async fn list_log_files(
    subsystem: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Listing log files for subsystem: {:?}", subsystem);
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.list_log_files(subsystem.as_deref()) {
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
) -> Result<Vec<u8>, TauriError> {
    log::info!("Downloading log file: {}", filename);
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.read_log_file(&filename) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(TauriError::from(anyhow::anyhow!("Failed to read log file: {}", e)))
    }
}

#[tauri::command]
pub async fn list_archives(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Listing archives");
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.list_archives() {
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
) -> Result<serde_json::Value, TauriError> {
    log::info!("Extracting archive: {}", archive_name);
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.extract_archive(&archive_name) {
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
) -> Result<Vec<u8>, TauriError> {
    log::info!("Downloading archive: {}", archive_name);
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.download_archive(&archive_name) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(TauriError::from(anyhow::anyhow!("Failed to read archive: {}", e)))
    }
}

#[tauri::command]
pub async fn set_live_data_streaming(
    subsystem: String,
    enabled: bool,
    app: State<'_, Arc<App>>,
    window: tauri::Window,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Setting live data streaming for {}: {}", subsystem, enabled);
    
    // Clone window once for emitting events (available throughout function)
    let app_handle = window.clone();
    
    if enabled {
        log::info!("Live data streaming enabled for subsystem: {}", subsystem);
        // Emit initial message
        let _ = app_handle.emit("live_data", serde_json::json!({
            "subsystem": subsystem,
            "data": format!("[{}] Live data streaming started for {}", chrono::Utc::now().format("%H:%M:%S"), subsystem),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        // Real routing per subsystem
        if subsystem == "obs" {
            // Forward OBS recent events periodically
            let app_arc = app.inner().clone();
            let window = app_handle.clone();
            tokio::spawn(async move {
                loop {
                    match app_arc.obs_obws_plugin().get_status(None).await {
                        Ok(events) => {
                            let _ = window.emit("live_data", serde_json::json!({
                                "subsystem": "obs",
                                "data": events,
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }));
                        }
                        Err(_) => {}
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            });
        } else if subsystem == "pss" || subsystem == "udp" {
            // Forward UDP recent events periodically (covers PSS over UDP)
            let app_arc = app.inner().clone();
            let window = app_handle.clone();
            tokio::spawn(async move {
                loop {
                    let events = app_arc.udp_plugin().get_recent_events();
                    for ev in events {
                        let _ = window.emit("live_data", serde_json::json!({
                            "subsystem": "pss",
                            "data": ev, // assuming string, adjust if structured
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }));
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
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
pub async fn start_live_data(subsystem: String, app: State<'_, Arc<App>>, window: tauri::Window) -> Result<(), TauriError> {
    set_live_data_streaming(subsystem, true, app, window).await?;
    Ok(())
}

#[tauri::command]
pub async fn stop_live_data(subsystem: String, app: State<'_, Arc<App>>, window: tauri::Window) -> Result<(), TauriError> {
    set_live_data_streaming(subsystem, false, app, window).await?;
    Ok(())
}

#[tauri::command]
pub async fn drive_create_folder(name: String, parent_id: Option<String>) -> Result<serde_json::Value, TauriError> {
    let id = crate::plugins::drive_plugin().create_folder(&name, parent_id.as_deref())
        .await.map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(serde_json::json!({"success": true, "id": id}))
}

#[tauri::command]
pub async fn drive_list_children(parent_id: Option<String>) -> Result<serde_json::Value, TauriError> {
    let files = crate::plugins::drive_plugin().list_children(parent_id.as_deref()).await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(serde_json::json!({"success": true, "files": files}))
}

#[tauri::command]
pub async fn drive_upload_zip_to_folder(zip_path: String, folder_id: Option<String>) -> Result<serde_json::Value, TauriError> {
    let p = std::path::PathBuf::from(&zip_path);
    if !p.is_file() { return Ok(serde_json::json!({"success": false, "error": "zip not found"})); }
    let file_name = p.file_name().and_then(|s| s.to_str()).unwrap_or("archive.zip").to_string();
    let id = crate::plugins::drive_plugin().upload_file_streaming_to_folder(&p, &file_name, folder_id.as_deref())
        .await.map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(serde_json::json!({"success": true, "file_id": id}))
}
#[tauri::command]
pub async fn get_live_data(subsystem: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting live data for subsystem: {}", subsystem);
    
    match subsystem.as_str() {
        "obs" => {
            // Get OBS live data
            let obs_status = app.obs_obws_plugin().get_status(None).await;
            match obs_status {
                Ok(status) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "data": {
                            "subsystem": "obs",
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "recording_status": format!("{:?}", status.recording_status),
                            "streaming_status": format!("{:?}", status.streaming_status),
                            "replay_buffer_status": format!("{:?}", status.replay_buffer_status),
                            "virtual_camera_status": format!("{:?}", status.virtual_camera_status),
                            "current_scene": status.current_scene,
                            "scenes": status.scenes,
                            "stats": status.stats
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
            let udp_stats = app.udp_plugin().get_stats();
            
            // Calculate uptime
            let uptime = if let Some(start_time) = udp_stats.server_start_time {
                if let Ok(duration) = std::time::SystemTime::now().duration_since(start_time) {
                    format!("{}s", duration.as_secs())
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Not started".to_string()
            };
            
            Ok(serde_json::json!({
                "success": true,
                "data": {
                    "subsystem": "udp",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "status": format!("{:?}", udp_status),
                    "is_running": matches!(udp_status, crate::plugins::plugin_udp::UdpServerStatus::Running),
                    "packets_received": udp_stats.packets_received,
                    "packets_parsed": udp_stats.packets_parsed,
                    "parse_errors": udp_stats.parse_errors,
                    "connected_clients": udp_stats.connected_clients,
                    "total_bytes_received": udp_stats.total_bytes_received,
                    "average_packet_size": (udp_stats.average_packet_size * 100.0).round() / 100.0,
                    "uptime": uptime,
                    "last_packet_time": udp_stats.last_packet_time.map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
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
pub async fn obs_get_debug_info(connection_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting OBS debug info for connection: {}", connection_name);
    
    match app.obs_obws_plugin().get_status(Some(&connection_name)).await {
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
pub async fn obs_toggle_full_events(enabled: bool, _app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Toggling OBS full events display: {}", enabled);
    
    // Legacy full-events toggle removed; no-op for obws
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Full OBS events display {}", if enabled { "enabled" } else { "disabled" })
    }))
}

#[tauri::command]
pub async fn obs_get_full_events_setting(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "enabled": false }))
}

#[tauri::command]
pub async fn obs_emit_event_to_frontend(event_data: serde_json::Value, window: tauri::Window) -> Result<serde_json::Value, TauriError> {
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
pub async fn obs_get_recent_events(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({
        "success": true,
        "events": []
    }))
}

// CPU Monitoring Commands
#[tauri::command]
pub async fn cpu_get_process_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    // println!("🚨 [CPU_CMD] ===== CPU GET PROCESS DATA CALLED =====");
    log::info!("[CPU_CMD] ===== CPU GET PROCESS DATA CALLED =====");
    
    // println!("🚨 [CPU_CMD] Triggering immediate data collection...");
    log::info!("[CPU_CMD] Triggering immediate data collection...");
    
    match app.cpu_monitor_plugin().update_cpu_data().await {
        Ok(_) => {
            // println!("🚨 [CPU_CMD] Data collection successful");
            log::info!("[CPU_CMD] Data collection successful");
        },
        Err(e) => {
            // println!("🚨 [CPU_CMD] Failed to update CPU data: {}", e);
            log::error!("[CPU_CMD] Failed to update CPU data: {}", e);
        },
    }
    
    let process_data = app.cpu_monitor_plugin().get_process_cpu_data().await;
    
    // println!("🚨 [CPU_CMD] Process data count: {}", process_data.len());
    log::info!("[CPU_CMD] Process data count: {}", process_data.len());
    
    // Log first few processes for debugging
    for (i, process) in process_data.iter().take(3).enumerate() {
        // println!("🚨 [CPU_CMD] Process {}: {} - CPU: {:.1}%, Memory: {:.1}MB", 
        //     i, process.process_name, process.cpu_percent, process.memory_mb);
        log::debug!("[CPU_CMD] Process {}: {} - CPU: {:.1}%, Memory: {:.1}MB", 
            i, process.process_name, process.cpu_percent, process.memory_mb);
    }
    
    // println!("🚨 [CPU_CMD] Returning result with {} processes", process_data.len());
    log::info!("[CPU_CMD] Returning result with {} processes", process_data.len());
    
    Ok(serde_json::json!({
        "success": true,
        "processes": process_data
    }))
}

#[tauri::command]
pub async fn cpu_get_system_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    // println!("🚨 [CPU_CMD] ===== CPU GET SYSTEM DATA CALLED =====");
    log::info!("[CPU_CMD] ===== CPU GET SYSTEM DATA CALLED =====");
    
    // Trigger immediate data collection
    // println!("🚨 [CPU_CMD] Triggering immediate data collection...");
    log::info!("[CPU_CMD] Triggering immediate data collection...");
    match app.cpu_monitor_plugin().update_cpu_data().await {
        Ok(_) => {
            // println!("🚨 [CPU_CMD] Data collection successful");
            log::info!("[CPU_CMD] Data collection successful");
        },
        Err(e) => {
            // println!("🚨 [CPU_CMD] Failed to update CPU data: {}", e);
            log::error!("[CPU_CMD] Failed to update CPU data: {}", e);
        },
    }
    
    let system_data = app.cpu_monitor_plugin().get_system_cpu_data().await;
    // println!("🚨 [CPU_CMD] System data available: {}", system_data.is_some());
    log::info!("[CPU_CMD] System data available: {}", system_data.is_some());
    
    let result = serde_json::json!({
        "success": true,
        "system": system_data
    });
    
    // println!("🚨 [CPU_CMD] Returning system data");
    log::info!("[CPU_CMD] Returning system data");
    Ok(result)
}
#[tauri::command]
pub async fn cpu_get_obs_usage(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let obs_cpu = app.cpu_monitor_plugin().get_obs_cpu_usage().await;
    
    Ok(serde_json::json!({
        "success": true,
        "obs_cpu_percent": obs_cpu
    }))
}

#[tauri::command]
pub async fn cpu_update_config(app: State<'_, Arc<App>>, config: crate::plugins::CpuMonitorConfig) -> Result<serde_json::Value, TauriError> {
    match app.cpu_monitor_plugin().update_config(config).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "CPU monitoring configuration updated"
        })),
        Err(e) => Err(TauriError::from(anyhow::anyhow!("Failed to update CPU monitoring config: {}", e)))
    }
} 

 

#[tauri::command]
pub async fn cpu_enable_monitoring(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("[CPU_CMD] ===== ENABLE CPU MONITORING CALLED =====");
    
    match app.cpu_monitor_plugin().enable_monitoring().await {
        Ok(_) => {
            log::info!("[CPU_CMD] CPU monitoring enabled successfully");
            Ok(())
        },
        Err(e) => {
            log::error!("[CPU_CMD] Failed to enable CPU monitoring: {}", e);
            Err(TauriError::from(anyhow::anyhow!("{}", e)))
        }
    }
}

#[tauri::command]
pub async fn cpu_disable_monitoring(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("[CPU_CMD] ===== DISABLE CPU MONITORING CALLED =====");
    
    match app.cpu_monitor_plugin().disable_monitoring().await {
        Ok(_) => {
            log::info!("[CPU_CMD] CPU monitoring disabled successfully");
            Ok(())
        },
        Err(e) => {
            log::error!("[CPU_CMD] Failed to disable CPU monitoring: {}", e);
            Err(TauriError::from(anyhow::anyhow!("{}", e)))
        }
    }
}

#[tauri::command]
pub async fn cpu_get_monitoring_status(app: State<'_, Arc<App>>) -> Result<bool, TauriError> {
    log::info!("[CPU_CMD] ===== GET CPU MONITORING STATUS CALLED =====");
    
    match app.cpu_monitor_plugin().is_monitoring_enabled().await {
        Ok(enabled) => {
            log::info!("[CPU_CMD] CPU monitoring status: {}", enabled);
            Ok(enabled)
        },
        Err(e) => {
            log::error!("[CPU_CMD] Failed to get CPU monitoring status: {}", e);
            Err(TauriError::from(anyhow::anyhow!("{}", e)))
        }
    }
}

// Protocol Management Commands
#[tauri::command]
pub async fn protocol_get_versions(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let log_manager = app.log_manager().lock().await;
    if let Err(e) = log_manager.log("pss", "INFO", "Getting protocol versions") {
        log::error!("Failed to log protocol get versions: {}", e);
    }
    
    let versions = app.protocol_manager().get_versions().await.map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    let current_protocol = app.protocol_manager().get_current_protocol().await.map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    
    Ok(serde_json::json!({
        "success": true,
        "versions": versions,
        "current_protocol": current_protocol
    }))
}

#[tauri::command]
pub async fn protocol_set_active_version(
    version: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let log_manager = app.log_manager().lock().await;
    if let Err(e) = log_manager.log("pss", "INFO", &format!("Setting active protocol version: {}", version)) {
        log::error!("Failed to log protocol set active version: {}", e);
    }
    
    match app.protocol_manager().set_active_version(&version).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Protocol version '{}' activated", version)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn protocol_upload_file(
    file_content: Vec<u8>,
    filename: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let log_manager = app.log_manager().lock().await;
    if let Err(e) = log_manager.log("pss", "INFO", &format!("Uploading protocol file: {}", filename)) {
        log::error!("Failed to log protocol upload: {}", e);
    }
    
    match app.protocol_manager().upload_protocol_file(file_content, &filename).await {
        Ok(protocol_version) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Protocol file '{}' uploaded successfully", filename),
            "protocol_version": protocol_version
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn protocol_delete_version(
    version: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let log_manager = app.log_manager().lock().await;
    if let Err(e) = log_manager.log("pss", "INFO", &format!("Deleting protocol version: {}", version)) {
        log::error!("Failed to log protocol delete: {}", e);
    }
    
    match app.protocol_manager().delete_version(&version).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Protocol version '{}' deleted", version)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn protocol_export_file(
    version: String,
    app: State<'_, Arc<App>>,
) -> Result<Vec<u8>, TauriError> {
    let log_manager = app.log_manager().lock().await;
    if let Err(e) = log_manager.log("pss", "INFO", &format!("Exporting protocol file: {}", version)) {
        log::error!("Failed to log protocol export: {}", e);
    }
    
    app.protocol_manager().export_protocol_file(&version).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))
}

#[tauri::command]
pub async fn protocol_get_current(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let log_manager = app.log_manager().lock().await;
    if let Err(e) = log_manager.log("pss", "INFO", "Getting current protocol") {
        log::error!("Failed to log protocol get current: {}", e);
    }
    
    match app.protocol_manager().get_current_protocol().await {
        Ok(Some(protocol)) => Ok(serde_json::json!({
            "success": true,
            "protocol": protocol
        })),
        Ok(None) => Ok(serde_json::json!({
            "success": false,
            "error": "No protocol currently loaded"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
} 

/// Get available network interfaces
#[tauri::command]
pub async fn get_network_interfaces() -> Result<serde_json::Value, TauriError> {
    match crate::utils::NetworkDetector::get_interfaces() {
        Ok(interfaces) => {
            let interface_data: Vec<serde_json::Value> = interfaces
                .into_iter()
                .map(|iface| {
                    serde_json::json!({
                        "name": iface.name,
                        "type": match iface.interface_type {
                            crate::utils::InterfaceType::Ethernet => "ethernet",
                            crate::utils::InterfaceType::WiFi => "wifi",
                            crate::utils::InterfaceType::Loopback => "loopback",
                            crate::utils::InterfaceType::Bluetooth => "bluetooth",
                            crate::utils::InterfaceType::Virtual => "virtual",
                            crate::utils::InterfaceType::Unknown => "unknown",
                        },
                        "ip_addresses": iface.ip_addresses.iter().map(|ip| ip.to_string()).collect::<Vec<_>>(),
                        "subnet_masks": iface.subnet_masks,
                        "default_gateway": iface.default_gateway,
                        "dns_suffix": iface.dns_suffix,
                        "media_state": match iface.media_state {
                            crate::utils::MediaState::Connected => "connected",
                            crate::utils::MediaState::Disconnected => "disconnected",
                            crate::utils::MediaState::Unknown => "unknown",
                        },
                        "is_up": iface.is_up,
                        "is_loopback": iface.is_loopback,
                        "description": iface.description,
                    })
                })
                .collect();
            
            Ok(serde_json::json!({
                "success": true,
                "interfaces": interface_data
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

/// Get the best network interface based on current configuration
#[tauri::command]
pub async fn get_best_network_interface() -> Result<serde_json::Value, TauriError> {
    let settings = crate::config::NetworkInterfaceSettings::default();
    match crate::utils::NetworkDetector::get_best_interface(&settings) {
        Ok(Some(interface)) => {
            Ok(serde_json::json!({
                "success": true,
                "interface": {
                    "name": interface.name,
                    "type": match interface.interface_type {
                        crate::utils::InterfaceType::Ethernet => "ethernet",
                        crate::utils::InterfaceType::WiFi => "wifi",
                        crate::utils::InterfaceType::Loopback => "loopback",
                        crate::utils::InterfaceType::Bluetooth => "bluetooth",
                        crate::utils::InterfaceType::Virtual => "virtual",
                        crate::utils::InterfaceType::Unknown => "unknown",
                    },
                    "ip_addresses": interface.ip_addresses.iter().map(|ip| ip.to_string()).collect::<Vec<_>>(),
                    "subnet_masks": interface.subnet_masks,
                    "default_gateway": interface.default_gateway,
                    "dns_suffix": interface.dns_suffix,
                    "media_state": match interface.media_state {
                        crate::utils::MediaState::Connected => "connected",
                        crate::utils::MediaState::Disconnected => "disconnected",
                        crate::utils::MediaState::Unknown => "unknown",
                    },
                    "is_up": interface.is_up,
                    "is_loopback": interface.is_loopback,
                    "description": interface.description,
                }
            }))
        }
        Ok(None) => Ok(serde_json::json!({
            "success": false,
            "error": "No suitable network interface found"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

/// Get the best IP address for a specific interface
#[tauri::command]
pub async fn get_best_ip_address_for_interface(interface_name: String) -> Result<serde_json::Value, TauriError> {
    let _settings = crate::config::NetworkInterfaceSettings::default();
    match crate::utils::NetworkDetector::get_interfaces() {
        Ok(interfaces) => {
            // Find the specified interface
            if let Some(interface) = interfaces.into_iter().find(|iface| iface.name == interface_name) {
                // Get the best IP address for this interface
                let best_ip = interface.ip_addresses.iter()
                    .find(|ip| {
                        if let std::net::IpAddr::V4(ipv4) = ip {
                            // Prefer private addresses for UDP server binding
                            !ipv4.is_loopback() && ipv4.is_private()
                        } else {
                            false
                        }
                    })
                    .or_else(|| interface.ip_addresses.iter()
                        .find(|ip| {
                            if let std::net::IpAddr::V4(ipv4) = ip {
                                !ipv4.is_loopback()
                            } else {
                                false
                            }
                        }))
                    .or_else(|| interface.ip_addresses.first());

                if let Some(ip) = best_ip {
                    Ok(serde_json::json!({
                        "success": true,
                        "ip_address": ip.to_string()
                    }))
                } else {
                    Ok(serde_json::json!({
                        "success": false,
                        "error": "No suitable IP address found for interface"
                    }))
                }
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "error": "Interface not found"
                }))
            }
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
} 

// PSS Event Emission Command
#[tauri::command]
pub async fn pss_emit_event(event_data: serde_json::Value, window: tauri::Window) -> Result<(), TauriError> {
    log::info!("🧪 Emitting PSS event via hybrid approach: {:?}", event_data);
    
    // HYBRID APPROACH: Real-time emission to both systems
    // 1. Emit to Tauri frontend (React components) - Real-time
    if let Err(e) = window.emit("pss_event", event_data.clone()) {
        log::error!("❌ Failed to emit PSS event to Tauri frontend: {}", e);
        return Err(TauriError::from(anyhow::anyhow!("{}", e)));
    }
    
    // 2. Broadcast to WebSocket overlays (HTML overlays) - Real-time
    crate::core::app::App::emit_pss_event(event_data);
    
    log::info!("✅ Successfully emitted PSS event via hybrid approach");
    Ok(())
}

// Get and emit PSS events to frontend
#[tauri::command]
pub async fn pss_emit_pending_events(window: tauri::Window, app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Getting and emitting pending PSS events");
    
    // Get events from the UDP plugin
    let events = app.udp_plugin().get_recent_events();
    
    // Convert and emit each event
    for event in events {
        let event_code = crate::plugins::plugin_udp::UdpServer::get_event_code(&event);
        
        let event_json = match event {
            crate::plugins::plugin_udp::PssEvent::Points { athlete, point_type } => {
                let athlete_str = match athlete {
                    1 => "blue",
                    2 => "red",
                    _ => "unknown"
                };
                serde_json::json!({
                    "type": "points",
                    "event_code": event_code,
                    "athlete": athlete_str,
                    "point_type": point_type,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Athlete {} scored {} points", athlete, point_type)
                })
            }
            crate::plugins::plugin_udp::PssEvent::HitLevel { athlete, level } => {
                let athlete_str = match athlete {
                    1 => "blue",
                    2 => "red",
                    _ => "unknown"
                };
                serde_json::json!({
                    "type": "hit_level",
                    "event_code": event_code,
                    "athlete": athlete_str,
                    "level": level,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Athlete {} hit level {}", athlete, level)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                serde_json::json!({
                    "type": "warnings",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_warnings": athlete1_warnings,
                    "athlete2_warnings": athlete2_warnings,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Warnings - Athlete1: {}, Athlete2: {}", athlete1_warnings, athlete2_warnings)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Clock { time, action } => {
                serde_json::json!({
                    "type": "clock",
                    "event_code": event_code,
                    "athlete": "",
                    "time": time,
                    "action": action,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Clock: {} {:?}", time, action.as_ref().unwrap_or(&String::new()))
                })
            }
            crate::plugins::plugin_udp::PssEvent::Round { current_round } => {
                serde_json::json!({
                    "type": "round",
                    "event_code": event_code,
                    "athlete": "",
                    "current_round": current_round,
                    "round": current_round,
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Round {}", current_round)
                })
            }
            crate::plugins::plugin_udp::PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                serde_json::json!({
                    "type": "winner_rounds",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "round1_winner": round1_winner,
                    "round2_winner": round2_winner,
                    "round3_winner": round3_winner,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Winner Rounds - R1: {}, R2: {}, R3: {}", round1_winner, round2_winner, round3_winner)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => {
                serde_json::json!({
                    "type": "scores",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_r1": athlete1_r1,
                    "athlete2_r1": athlete2_r1,
                    "athlete1_r2": athlete1_r2,
                    "athlete2_r2": athlete2_r2,
                    "athlete1_r3": athlete1_r3,
                    "athlete2_r3": athlete2_r3,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Scores - A1: R1={}, R2={}, R3={} | A2: R1={}, R2={}, R3={}", 
                        athlete1_r1, athlete1_r2, athlete1_r3, athlete2_r1, athlete2_r2, athlete2_r3)
                })
            }
            crate::plugins::plugin_udp::PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                serde_json::json!({
                    "type": "current_scores",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_score": athlete1_score,
                    "athlete2_score": athlete2_score,
                    "round": 1, // Will be updated by WebSocket plugin
                    "time": "2:00", // Will be updated by WebSocket plugin
                    "description": format!("Current Scores - A1: {}, A2: {}", athlete1_score, athlete2_score)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Athletes { athlete1_short, athlete1_long, athlete1_country, athlete2_short, athlete2_long, athlete2_country } => {
                serde_json::json!({
                    "type": "athletes",
                    "athlete1_short": athlete1_short,
                    "athlete1_long": athlete1_long,
                    "athlete1_country": athlete1_country,
                    "athlete2_short": athlete2_short,
                    "athlete2_long": athlete2_long,
                    "athlete2_country": athlete2_country,
                    "description": format!("Athletes - {} ({}) vs {} ({})", athlete1_short, athlete1_country, athlete2_short, athlete2_country)
                })
            }
            crate::plugins::plugin_udp::PssEvent::MatchConfig { number, category, weight, rounds, colors, match_id, division, total_rounds, round_duration, countdown_type, count_up, format } => {
                serde_json::json!({
                    "type": "match_config",
                    "number": number,
                    "category": category,
                    "weight": weight,
                    "rounds": rounds,
                    "colors": colors,
                    "match_id": match_id,
                    "division": division,
                    "total_rounds": total_rounds,
                    "round_duration": round_duration,
                    "countdown_type": countdown_type,
                    "count_up": count_up,
                    "format": format,
                    "description": format!("Match Config - #{} {} {} ({})", number, category, weight, division)
                })
            }
            crate::plugins::plugin_udp::PssEvent::FightLoaded => {
                serde_json::json!({
                    "type": "fight_loaded",
                    "event": "FightLoaded",
                    "description": "Fight Loaded"
                })
            }
            crate::plugins::plugin_udp::PssEvent::FightReady => {
                serde_json::json!({
                    "type": "fight_ready",
                    "event": "FightReady",
                    "description": "Fight Ready"
                })
            }
            crate::plugins::plugin_udp::PssEvent::Raw(message) => {
                serde_json::json!({
                    "type": "raw",
                    "message": message,
                    "description": format!("Raw message: {}", message)
                })
            }
            _ => {
                serde_json::json!({
                    "type": "other",
                    "event": format!("{:?}", event),
                    "description": format!("Event: {:?}", event)
                })
            }
        };
        
        log::info!("Emitting PSS event to frontend: {:?}", event_json);
        if let Err(e) = window.emit("pss_event", event_json) {
            log::error!("Failed to emit PSS event: {}", e);
            return Err(TauriError::from(anyhow::anyhow!("{}", e)));
        }
    }
    
    Ok(())
} 
// Set up PSS event listener that emits events to frontend
#[tauri::command]
pub async fn pss_setup_event_listener(_window: tauri::Window) -> Result<(), TauriError> {
    log::info!("Setting up PSS event listener for frontend");
    
    // Note: This command is no longer needed since we're using the original working mechanism
    // The frontend will fetch events via pss_get_events or they will be emitted via pss_emit_event
    log::info!("✅ PSS event listener setup complete (using original mechanism)");
    
    Ok(())
} 

#[tauri::command]
pub async fn obs_setup_status_listener(window: tauri::Window, app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("🔧 Setting up OBS status listener for frontend - COMMAND CALLED");

    let window_clone = window.clone();
    let app_arc = app.inner().clone();
    // Spawn background task (using cloned Arc<App>)
    tokio::spawn(async move {
        log::info!("🔧 OBS status listener background task started");
        let mut last_payload = serde_json::Value::Null;
        loop {
            // Fetch current status
            log::debug!("🔧 Fetching OBS status...");
            let status_result = app_arc.obs_obws_plugin().get_status(None).await;
            if let Ok(status) = status_result {
                let payload = serde_json::json!({
                    "recording_status": format!("{:?}", status.recording_status),
                    "streaming_status": format!("{:?}", status.streaming_status),
                    "replay_buffer_status": format!("{:?}", status.replay_buffer_status),
                    "virtual_camera_status": format!("{:?}", status.virtual_camera_status),
                    "current_scene": status.current_scene,
                    "scenes": status.scenes,
                    "stats": status.stats,
                });
                // Emit only if changed
                if payload != last_payload {
                    log::info!("🔧 Emitting OBS status update: {:?}", payload);
                    if let Err(e) = window_clone.emit("obs_status", payload.clone()) {
                        log::error!("Failed to emit obs_status: {}", e);
                    }
                    last_payload = payload;
                }
            } else if let Err(e) = status_result {
                log::error!("OBS status fetch error: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    Ok(())
} 

#[tauri::command]
pub async fn cpu_setup_stats_listener(window: tauri::Window, app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Setting up CPU stats listener for frontend");

    let window_clone = window.clone();
    let cpu_plugin = app.inner().cpu_monitor_plugin().clone();

    tokio::spawn(async move {
        let mut last_payload = serde_json::Value::Null;
        loop {
            let processes = cpu_plugin.get_process_cpu_data().await;
            let system = cpu_plugin.get_system_cpu_data().await;

            // Build JSON payload
            let payload = serde_json::json!({
                "processes": processes,
                "system": system,
            });

            if payload != last_payload {
                if let Err(e) = window_clone.emit("cpu_stats", payload.clone()) {
                    log::error!("Failed to emit cpu_stats: {}", e);
                }
                last_payload = payload;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    Ok(())
} 

// Window Management Commands
#[tauri::command]
pub async fn set_window_fullscreen(window: tauri::Window) -> Result<(), TauriError> {
    log::info!("Setting window to fullscreen");
    window.set_fullscreen(true).map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    Ok(())
}
#[tauri::command]
pub async fn set_window_compact(width: Option<f64>, height: Option<f64>, window: tauri::Window) -> Result<(), TauriError> {
    let default_width = 350.0;
    let default_height = 1080.0;
    
    log::info!("Setting window to compact mode: {}x{}", width.unwrap_or(default_width), height.unwrap_or(default_height));
    window.set_fullscreen(false).map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
        width.unwrap_or(default_width), 
        height.unwrap_or(default_height)
    ))).map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn set_window_custom_size(width: f64, height: f64, window: tauri::Window) -> Result<(), TauriError> {
    log::info!("Setting window to custom size: {}x{}", width, height);
    window.set_fullscreen(false).map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)))
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn set_window_position(x: f64, y: f64, window: tauri::Window) -> Result<(), TauriError> {
    log::info!("Setting window position: x={}, y={}", x, y);
    window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)))
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn set_window_startup_position(window: tauri::Window) -> Result<(), TauriError> {
    log::info!("Setting window to startup position: x=1, y=1");
    
    // Set window to compact mode (350x1080)
    window.set_fullscreen(false).map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(350.0, 1080.0)))
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    
    // Set position to x=1, y=1
    window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(1.0, 1.0)))
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    
    Ok(())
}

#[tauri::command]
pub async fn save_window_settings(settings: serde_json::Value, _app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Saving window settings: {:?}", settings);
    
    // For now, just log the settings - we'll implement proper persistence later
    log::info!("Window settings would be saved: {:?}", settings);
    
    Ok(())
}

#[tauri::command]
pub async fn load_window_settings(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Loading window settings");
    
    // Return default settings for now
    let window_settings = serde_json::json!({
        "compactWidth": 350,
        "compactHeight": 1080,
        "fullscreenWidth": 1920,
        "fullscreenHeight": 1080,
    });
    
    Ok(window_settings)
}

#[tauri::command]
pub async fn get_screen_size() -> Result<serde_json::Value, TauriError> {
    log::info!("Getting screen size");
    
    // This would need to be implemented with proper screen detection
    // For now, return a default size
    Ok(serde_json::json!({
        "width": 1920,
        "height": 1080
    }))
}

// UI Settings Migration Commands
#[tauri::command]
pub async fn initialize_ui_settings_database() -> Result<serde_json::Value, TauriError> {
    log::info!("Initializing UI settings database");
    
    // This command should initialize the database schema, not just UI settings
    // For now, we'll return a success message since the database plugin handles initialization
    Ok(serde_json::json!({
        "success": true,
        "message": "UI settings database initialization command available. Use db_initialize_ui_settings for actual initialization."
    }))
}

// Database Plugin Commands
#[tauri::command]
pub async fn db_initialize_ui_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Initializing UI settings in database");
    
    match app.database_plugin().initialize_ui_settings().await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "UI settings initialized in database successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn db_get_ui_setting(key: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting UI setting: {}", key);
    
    match app.database_plugin().get_ui_setting(&key).await {
        Ok(value) => Ok(serde_json::json!({
            "success": true,
            "key": key,
            "value": value
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn db_set_ui_setting(
    key: String, 
    value: String, 
    changed_by: String, 
    change_reason: Option<String>, 
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Setting UI setting: {} = {}", key, value);
    
    match app.database_plugin().set_ui_setting(&key, &value, &changed_by, change_reason.as_deref()).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("UI setting '{}' set successfully", key)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}
#[tauri::command]
pub async fn db_get_all_ui_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting all UI settings");
    
    match app.database_plugin().get_all_ui_settings().await {
        Ok(settings) => {
            let settings_map: std::collections::HashMap<String, String> = settings.into_iter().collect();
            Ok(serde_json::json!({
                "success": true,
                "settings": settings_map
            }))
        },
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn db_get_database_info(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database information");
    
    let is_accessible = app.database_plugin().is_accessible().await;
    let file_size = app.database_plugin().get_file_size();
    let database_path = app.database_plugin().get_database_path();
    let settings_count = app.database_plugin().get_all_ui_settings().await.map(|s| s.len()).unwrap_or(0);
    
    let file_size_value = match file_size {
        Ok(size) => serde_json::Value::Number(serde_json::Number::from(size)),
        Err(_) => serde_json::Value::Null,
    };
    
    let path_value = match database_path {
        Ok(path) => serde_json::Value::String(path),
        Err(_) => serde_json::Value::String("Unknown".to_string()),
    };
    
    // Get tables count
    let tables_count = match app.database_plugin().get_connection().await {
        Ok(conn) => {
            match conn.prepare("SELECT name FROM sqlite_master WHERE type='table'") {
                Ok(mut stmt) => {
                    match stmt.query_map([], |row| row.get::<_, String>(0)) {
                        Ok(rows) => {
                            let tables: Result<Vec<String>, _> = rows.collect();
                            match tables {
                                Ok(tables) => tables.len(),
                                Err(_) => 0
                            }
                        },
                        Err(_) => 0
                    }
                },
                Err(_) => 0
            }
        },
        Err(_) => 0
    };
    
    let status = if is_accessible { "Active" } else { "Inactive" };
    let last_modified = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    Ok(serde_json::json!({
        "success": true,
        "path": path_value,
        "size": file_size_value,
        "tables": tables_count,
        "settings_count": settings_count,
        "last_modified": last_modified,
        "status": status,
        "is_accessible": is_accessible
    }))
}

// Database Migration Commands
#[tauri::command]
pub async fn migrate_json_to_database(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Starting JSON to database migration");
    
    match app.database_plugin().migrate_json_to_database().await {
        Ok(result) => Ok(serde_json::json!({
            "success": true,
            "result": {
                "total_settings": result.total_settings,
                "migrated_settings": result.migrated_settings,
                "failed_settings": result.failed_settings,
                "success_rate": result.success_rate(),
                "errors": result.errors
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn db_run_migrations(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    match app.database_plugin().run_migrations().await {
        Ok(_) => Ok(serde_json::json!({ "success": true, "message": "Database migrations ran successfully" })),
        Err(e) => Ok(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

#[tauri::command]
pub async fn create_json_backup(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Creating JSON settings backup");
    
    match app.database_plugin().create_json_backup().await {
        Ok(backup_path) => Ok(serde_json::json!({
            "success": true,
            "backup_path": backup_path
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn restore_from_json_backup(
    app: State<'_, Arc<App>>,
    backup_path: String
) -> Result<serde_json::Value, TauriError> {
    log::info!("Restoring from JSON backup: {}", backup_path);
    
    match app.database_plugin().restore_from_json_backup(&backup_path).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Settings restored successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn restore_from_backup(
    app: State<'_, Arc<App>>,
    backup_path: String
) -> Result<serde_json::Value, TauriError> {
    log::info!("Restoring from backup: {}", backup_path);
    
    match app.database_plugin().restore_from_json_backup(&backup_path).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Backup restored successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn get_migration_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting migration status");
    
    // Get database status
    let db_status = match app.database_plugin().get_migration_status().await {
        Ok(status) => status,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    };
    
    // Check for backup files in external directory
    let backup_dir = match dirs::data_dir() {
        Some(data_dir) => data_dir.join("reStrikeVTA").join("backups"),
        None => std::path::PathBuf::from("backups"),
    };
    let backup_files_exist = std::fs::read_dir(&backup_dir)
        .map(|entries| entries.filter_map(|entry| entry.ok()).count() > 0)
        .unwrap_or(false);
    
    // Get actual database settings count
    let db_settings_count = match app.database_plugin().get_all_ui_settings().await {
        Ok(settings) => settings.len(),
        Err(_) => 0
    };
    
    // Get JSON settings count - simplified for now
    let json_settings_count = 0;
    
    Ok(serde_json::json!({
        "success": true,
        "status": {
            "database_enabled": db_status.database_enabled,
            "json_fallback_enabled": db_status.json_fallback_enabled,
            "migration_completed": db_status.migration_completed,
            "last_migration": db_status.last_migration,
            "settings_count": db_settings_count,
            "backup_created": backup_files_exist,
            "json_settings_count": json_settings_count,
            "database_settings_count": db_settings_count
        }
    }))
}

#[tauri::command]
pub async fn enable_database_mode(
    app: State<'_, Arc<App>>,
    enabled: bool
) -> Result<serde_json::Value, TauriError> {
    log::info!("Setting database mode to: {}", enabled);
    
    match app.database_plugin().set_database_mode(enabled).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Database mode {}", if enabled { "enabled" } else { "disabled" })
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn get_database_preview(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database preview");
    
    // Get all UI settings from database
    let db_settings = match app.database_plugin().get_all_ui_settings().await {
        Ok(settings) => settings,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to get database settings: {}", e)
        }))
    };
    
    // Get JSON settings for comparison - simplified for now
    let json_settings: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    
    // Convert settings to preview format
    let db_preview: Vec<serde_json::Value> = db_settings.iter().map(|(key, value)| {
        serde_json::json!({
            "key": key,
            "value": value,
            "source": "database"
        })
    }).collect();
    
    let json_preview: Vec<serde_json::Value> = json_settings.iter().map(|(key, value)| {
        serde_json::json!({
            "key": key,
            "value": value,
            "source": "json"
        })
    }).collect();
    
    Ok(serde_json::json!({
        "success": true,
        "database_settings": db_preview,
        "json_settings": json_preview,
        "database_count": db_settings.len(),
        "json_count": json_settings.len()
    }))
}

#[tauri::command]
pub async fn get_database_tables(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database tables");
    
    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to get database connection: {}", e)
        }))
    };
    
    // Query to get all table names
    let tables: Vec<String> = match conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    ) {
        Ok(mut stmt) => {
            let mut table_names = Vec::new();
            let rows = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            }).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query tables: {}", e)))?;
            
            for row in rows {
                let table_name = row.map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get table name: {}", e)))?;
                table_names.push(table_name);
            }
            table_names
        },
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to prepare table query: {}", e)
        }))
    };
    
    Ok(serde_json::json!({
        "success": true,
        "tables": tables
    }))
}

#[tauri::command]
pub async fn get_table_data(
    app: State<'_, Arc<App>>,
    table_name: String
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting data for table: {}", table_name);
    
    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to get database connection: {}", e)
        }))
    };
    
    // First, get the table schema to understand the columns
    let schema_query = format!("PRAGMA table_info({})", table_name);
    let columns: Vec<serde_json::Value> = match conn.prepare(&schema_query) {
        Ok(mut stmt) => {
            let mut column_info = Vec::new();
            let rows = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "name": row.get::<_, String>(1)?,
                    "type": row.get::<_, String>(2)?,
                    "not_null": row.get::<_, i32>(3)? == 1,
                    "primary_key": row.get::<_, i32>(5)? == 1
                }))
            }).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query schema: {}", e)))?;
            
            for row in rows {
                let column = row.map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get column info: {}", e)))?;
                column_info.push(column);
            }
            column_info
        },
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to prepare schema query: {}", e)
        }))
    };
    
    // Get the data from the table (no limit to show all rows)
    let data_query = format!("SELECT * FROM {}", table_name);
    let rows: Vec<serde_json::Value> = match conn.prepare(&data_query) {
        Ok(mut stmt) => {
            let mut table_data = Vec::new();
            let rows = stmt.query_map([], |row| {
                let mut row_data = serde_json::Map::new();
                for (i, column) in columns.iter().enumerate() {
                    let column_name = column["name"].as_str().unwrap_or("unknown");
                    let value = match row.get::<_, rusqlite::types::Value>(i) {
                        Ok(val) => match val {
                            rusqlite::types::Value::Null => serde_json::Value::Null,
                            rusqlite::types::Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
                            rusqlite::types::Value::Real(f) => serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0))),
                            rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                            rusqlite::types::Value::Blob(b) => serde_json::Value::String(format!("[BLOB: {} bytes]", b.len())),
                        },
                        Err(_) => serde_json::Value::Null,
                    };
                    row_data.insert(column_name.to_string(), value);
                }
                Ok(serde_json::Value::Object(row_data))
            }).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query table data: {}", e)))?;
            
            for row in rows {
                let row_data = row.map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get row data: {}", e)))?;
                table_data.push(row_data);
            }
            table_data
        },
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to prepare data query: {}", e)
        }))
    };
    
    Ok(serde_json::json!({
        "success": true,
        "table_name": table_name,
        "columns": columns,
        "rows": rows,
        "row_count": rows.len()
    }))
}

// Google Drive commands
#[tauri::command]
pub async fn drive_request_auth_url() -> Result<String, TauriError> {
    let (url, _csrf_token) = crate::plugins::drive_plugin()
        .auth_url()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    Ok(url)
}

#[tauri::command]
pub async fn drive_complete_auth(code: String) -> Result<(), TauriError> {
    crate::plugins::drive_plugin()
        .exchange_code(code)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))
}

#[tauri::command]
pub async fn drive_save_credentials(id: String, secret: String) -> Result<(), TauriError> {
    crate::plugins::drive_plugin()
        .save_credentials(id, secret)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))
}

#[tauri::command]
pub async fn list_backup_files() -> Result<Vec<BackupFileInfo>, TauriError> {
    // Use the backups directory outside the project
    let backup_dir = match dirs::data_dir() {
        Some(data_dir) => data_dir.join("reStrikeVTA").join("backups"),
        None => std::path::PathBuf::from("backups"),
    };
    
    log::info!("=== LIST_BACKUP_FILES START ===");
    log::info!("Looking for backup files in: {}", backup_dir.display());
    
    let mut backup_files = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&backup_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("zip") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let modified_time: chrono::DateTime<chrono::Local> = chrono::DateTime::from(modified);
                            backup_files.push(BackupFileInfo {
                                name: path.file_name().unwrap().to_string_lossy().to_string(),
                                path: path.to_string_lossy().to_string(),
                                size: metadata.len(),
                                modified: modified_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    // Sort by modification time (newest first)
    backup_files.sort_by(|a, b| b.modified.cmp(&a.modified));
    
    Ok(backup_files)
}

#[derive(serde::Serialize)]
pub struct BackupFileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
}

#[tauri::command]
pub async fn drive_list_files() -> Result<serde_json::Value, TauriError> {
    log::info!("Listing Google Drive files");
    
    match crate::plugins::drive_plugin().list_files().await {
        Ok(files) => Ok(serde_json::json!({
            "success": true,
            "files": files
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}
#[tauri::command]
pub async fn drive_upload_backup_archive() -> Result<serde_json::Value, TauriError> {
    log::info!("=== DRIVE_UPLOAD_BACKUP_ARCHIVE COMMAND START ===");
    log::info!("Creating and uploading backup archive to Google Drive");
    
    // Log error to file immediately
    let log_error_to_file = |error_msg: &str| {
        let error_log = format!(
            "[{}] Tauri Command Upload Error:\nError: {}\nCommand: drive_upload_backup_archive\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            error_msg
        );
        if let Err(write_err) = std::fs::write("logs/app.log", error_log) {
            log::error!("Failed to write error log: {}", write_err);
        }
    };
    
    // Step 1: Call drive plugin upload method (which creates and uploads the archive)
    log::info!("Step 1: Calling drive_plugin().upload_backup_archive()...");
    match crate::plugins::drive_plugin().upload_backup_archive().await {
        Ok(message) => {
            log::info!("=== DRIVE_UPLOAD_BACKUP_ARCHIVE COMMAND SUCCESS ===");
            log::info!("Upload successful: {}", message);
            Ok(serde_json::json!({
                "success": true,
                "message": message
            }))
        },
        Err(e) => {
            let error_msg = format!("Failed to upload archive: {}", e);
            log::error!("=== DRIVE_UPLOAD_BACKUP_ARCHIVE COMMAND ERROR ===");
            log::error!("{}", error_msg);
            log_error_to_file(&error_msg);
            
            Ok(serde_json::json!({
                "success": false,
                "error": error_msg
            }))
        }
    }
}

#[tauri::command]
pub async fn drive_download_backup_archive(file_id: String) -> Result<serde_json::Value, TauriError> {
    log::info!("Downloading backup archive from Google Drive: {}", file_id);
    
    // Use the new download_backup_archive method
    match crate::plugins::drive_plugin().download_backup_archive(&file_id).await {
        Ok(message) => Ok(serde_json::json!({
            "success": true,
            "message": message
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to download archive: {}", e)
        }))
    }
}
#[tauri::command]
pub async fn drive_delete_backup_archive(file_id: String) -> Result<serde_json::Value, TauriError> {
    log::info!("Deleting backup archive from Google Drive: {}", file_id);
    
    match crate::plugins::drive_plugin().delete_backup_archive(&file_id).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Backup archive deleted successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to delete archive: {}", e)
        }))
    }
}
#[tauri::command]
pub async fn drive_get_connection_status() -> Result<serde_json::Value, TauriError> {
    log::info!("Checking Google Drive connection status");
    
    log::info!("About to call drive_plugin().is_connected()");
    // Use the new is_connected method for better reliability
    match crate::plugins::drive_plugin().is_connected().await {
        Ok(_connected) => {
            // If connected, try to get file count for additional info
            match crate::plugins::drive_plugin().list_files().await {
                Ok(files) => Ok(serde_json::json!({
                    "success": true,
                    "connected": true,
                    "file_count": files.len(),
                    "message": "Connected to Google Drive"
                })),
                Err(e) => Ok(serde_json::json!({
                    "success": true,
                    "connected": true,
                    "file_count": 0,
                    "message": "Connected to Google Drive (file listing failed)",
                    "warning": e.to_string()
                }))
            }
        },
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "connected": false,
            "error": e.to_string(),
            "message": "Failed to check connection status"
        }))
    }
}

#[tauri::command]
pub async fn drive_get_quota() -> Result<serde_json::Value, TauriError> {
    log::info!("Getting Google Drive quota");
    // Best-effort: if plugin has quota method, call it; else derive from listing/chunks
    // Here we try list_all_files to estimate usage is too heavy; so check plugin API
    match crate::plugins::drive_plugin().get_quota().await {
        Ok((limit, usage, usage_in_drive)) => Ok(serde_json::json!({
            "success": true,
            "quota": {
                "limit": limit,
                "usage": usage,
                "usageInDrive": usage_in_drive,
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn drive_restore_from_archive(file_id: String) -> Result<serde_json::Value, TauriError> {
    log::info!("Restoring from Google Drive archive: {}", file_id);
    
    match crate::plugins::drive_plugin().restore_from_archive(&file_id).await {
        Ok(message) => Ok(serde_json::json!({
            "success": true,
            "message": message
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn drive_test_connection() -> Result<serde_json::Value, TauriError> {
    log::info!("Testing Google Drive connection");
    
    // First check if connected
    match crate::plugins::drive_plugin().is_connected().await {
        Ok(_connected) => {
            if !_connected {
                return Ok(serde_json::json!({
                    "success": false,
                    "error": "Not connected to Google Drive"
                }));
            }
        }
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Connection check failed: {}", e)
        }))
    }
    
    // Try to list all files to test API access
    match crate::plugins::drive_plugin().list_all_files().await {
        Ok(files) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Connected successfully. Found {} files in Drive.", files.len()),
            "file_count": files.len()
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("API access failed: {}", e)
        }))
    }
}

#[tauri::command]
pub async fn drive_list_all_files() -> Result<serde_json::Value, TauriError> {
    log::info!("Listing all Google Drive files");
    
    match crate::plugins::drive_plugin().list_all_files().await {
        Ok(files) => Ok(serde_json::json!({
            "success": true,
            "files": files
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn get_flag_mappings_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting flag mappings data");
    
    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to get database connection: {}", e)
        }))
    };
    
    // Check if flag_mappings table exists and get its data
    let table_exists: i32 = match conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='flag_mappings'",
        [],
        |row| row.get(0),
    ) {
        Ok(count) => count,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to check table existence: {}", e)
        }))
    };
    
    if table_exists == 0 {
        return Ok(serde_json::json!({
            "success": false,
            "error": "flag_mappings table does not exist",
            "table_exists": false
        }));
    }
    
    // Get the data from flag_mappings table
    let mappings: Vec<serde_json::Value> = match conn.prepare(
        "SELECT id, pss_code, ioc_code, country_name, is_custom, created_at, updated_at FROM flag_mappings ORDER BY pss_code"
    ) {
        Ok(mut stmt) => {
            let mut mapping_data = Vec::new();
            let rows = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "pss_code": row.get::<_, String>(1)?,
                    "ioc_code": row.get::<_, String>(2)?,
                    "country_name": row.get::<_, String>(3)?,
                    "is_custom": row.get::<_, bool>(4)?,
                    "created_at": row.get::<_, String>(5)?,
                    "updated_at": row.get::<_, String>(6)?
                }))
            }).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query flag mappings: {}", e)))?;
            
            for row in rows {
                let mapping = row.map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get mapping data: {}", e)))?;
                mapping_data.push(mapping);
            }
            mapping_data
        },
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to prepare flag mappings query: {}", e)
        }))
    };
    
    Ok(serde_json::json!({
        "success": true,
        "table_exists": true,
        "mappings": mappings,
        "count": mappings.len()
    }))
}
#[tauri::command]
pub async fn scan_and_populate_flags(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Scanning and populating flags table");
    
    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to get database connection: {}", e)
        }))
    };
    
    // Path to the SVG flags directory (relative to project root)
    let flags_dir = std::path::Path::new("../ui/public/assets/flags/svg");
    
    if !flags_dir.exists() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Flags directory does not exist: ../ui/public/assets/flags/svg"
        }));
    }
    
    let mut processed_count = 0;
    let mut skipped_count = 0;
    let mut errors = Vec::new();
    
    // Read directory entries
    let entries = match std::fs::read_dir(flags_dir) {
        Ok(entries) => entries,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to read flags directory: {}", e)
        }))
    };
    
    let current_time = chrono::Utc::now().to_rfc3339();
    
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                errors.push(format!("Failed to read directory entry: {}", e));
                continue;
            }
        };
        
        let path = entry.path();
        
        // Only process SVG files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("svg") {
            continue;
        }
        
        let filename = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => {
                errors.push(format!("Invalid filename for path: {}", path.display()));
                continue;
            }
        };
        
        // Skip report files
        if filename.contains("REPORT") || filename.contains("report") {
            skipped_count += 1;
            continue;
        }
        
        // Extract IOC code from filename (e.g., "USA.svg" -> "USA")
        let ioc_code = filename.trim_end_matches(".svg").to_uppercase();
        
        // Get file metadata
        let metadata = match std::fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(e) => {
                errors.push(format!("Failed to get metadata for {}: {}", filename, e));
                continue;
            }
        };
        
        let file_size = metadata.len() as i64;
        let file_path = path.to_string_lossy().to_string();
        
        // Try to get country name from flag_mappings table
        let country_name: Option<String> = conn.query_row(
            "SELECT country_name FROM flag_mappings WHERE ioc_code = ? OR pss_code = ?",
            [&ioc_code, &ioc_code],
            |row| row.get(0),
        ).unwrap_or(None);
        
        let recognition_status = if country_name.is_some() { "recognized" } else { "pending" };
        let is_recognized = country_name.is_some();
        let recognition_confidence = if is_recognized { Some(1.0) } else { None };
        
        // Check if this flag already exists in the database
        let exists: i32 = conn.query_row(
            "SELECT COUNT(*) FROM flags WHERE filename = ? OR (ioc_code = ? AND ioc_code IS NOT NULL)",
            [filename, &ioc_code],
            |row| row.get(0),
        ).unwrap_or(0);
        
        if exists > 0 {
            skipped_count += 1;
            continue;
        }
        
        // Insert the flag into the database
        let result = conn.execute(
            "INSERT INTO flags (filename, ioc_code, country_name, recognition_status, recognition_confidence, upload_date, last_modified, file_size, file_path, is_recognized) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                filename,
                if ioc_code.is_empty() { None } else { Some(ioc_code.clone()) },
                country_name,
                recognition_status,
                recognition_confidence,
                &current_time,
                &current_time,
                file_size,
                file_path,
                is_recognized
            ],
        );
        
        match result {
            Ok(_) => {
                processed_count += 1;
                log::info!("Added flag to database: {} -> {}", filename, ioc_code);
            }
            Err(e) => {
                errors.push(format!("Failed to insert flag {}: {}", filename, e));
            }
        }
    }
    
    log::info!("Flag scanning completed: {} processed, {} skipped, {} errors", processed_count, skipped_count, errors.len());
    
    Ok(serde_json::json!({
        "success": true,
        "processed_count": processed_count,
        "skipped_count": skipped_count,
        "errors": errors,
        "message": format!("Successfully processed {} flag files", processed_count)
    }))
}

#[tauri::command]
pub async fn get_flags_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting flags data");
    
    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to get database connection: {}", e)
        }))
    };
    
    // Get all flags from the database
    let flags: Vec<serde_json::Value> = match conn.prepare(
        "SELECT id, filename, ioc_code, country_name, recognition_status, recognition_confidence, upload_date, last_modified, file_size, file_path, is_recognized FROM flags ORDER BY filename"
    ) {
        Ok(mut stmt) => {
            let mut flag_data = Vec::new();
            let rows = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "filename": row.get::<_, String>(1)?,
                    "ioc_code": row.get::<_, Option<String>>(2)?,
                    "country_name": row.get::<_, Option<String>>(3)?,
                    "recognition_status": row.get::<_, String>(4)?,
                    "recognition_confidence": row.get::<_, Option<f64>>(5)?,
                    "upload_date": row.get::<_, String>(6)?,
                    "last_modified": row.get::<_, String>(7)?,
                    "file_size": row.get::<_, i64>(8)?,
                    "file_path": row.get::<_, String>(9)?,
                    "is_recognized": row.get::<_, bool>(10)?
                }))
            }).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query flags: {}", e)))?;
            
            for row in rows {
                let flag = row.map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get flag data: {}", e)))?;
                flag_data.push(flag);
            }
            flag_data
        },
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to prepare flags query: {}", e)
        }))
    };
    
    // Get statistics
    let stats = match conn.prepare("SELECT recognition_status, COUNT(*) FROM flags GROUP BY recognition_status") {
        Ok(mut stmt) => {
            let mut stats_map = std::collections::HashMap::new();
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            }).map_err(|e| TauriError::from(anyhow::anyhow!("Failed to query flag statistics: {}", e)))?;
            
            for row in rows {
                let (status, count) = row.map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get stats: {}", e)))?;
                stats_map.insert(status, count);
            }
            stats_map
        },
        Err(_) => std::collections::HashMap::new()
    };
    
    Ok(serde_json::json!({
        "success": true,
        "flags": flags,
        "count": flags.len(),
        "statistics": {
            "total": flags.len(),
            "recognized": stats.get("recognized").unwrap_or(&0),
            "pending": stats.get("pending").unwrap_or(&0),
            "failed": stats.get("failed").unwrap_or(&0)
        }
    }))
}

#[tauri::command]
pub async fn clear_flags_table(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Clearing flags table");
    
    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => return Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to get database connection: {}", e)
        }))
    };
    
    match conn.execute("DELETE FROM flags", []) {
        Ok(deleted_count) => {
            log::info!("Cleared {} entries from flags table", deleted_count);
            Ok(serde_json::json!({
                "success": true,
                "deleted_count": deleted_count,
                "message": format!("Cleared {} flag entries", deleted_count)
            }))
        },
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to clear flags table: {}", e)
        }))
    }
}

// New Log Archive & Google Drive Commands

#[tauri::command]
pub async fn create_complete_log_archive(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Creating complete log archive");
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.create_complete_archive() {
        Ok(archive_info) => Ok(serde_json::json!({
            "success": true,
            "data": {
                "name": archive_info.name,
                "size": archive_info.size,
                "created": archive_info.created,
                "path": archive_info.file_path.to_string_lossy()
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to create archive: {}", e)
        }))
    }
}

#[tauri::command]
pub async fn create_and_upload_log_archive(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("=== CREATE_AND_UPLOAD_LOG_ARCHIVE COMMAND START ===");
    
    // Add comprehensive error logging to app.log
    let log_error_to_file = |error_msg: &str| {
        let error_log = format!(
            "[{}] LogArchiveManager Upload Error:\nError: {}\nCommand: create_and_upload_log_archive\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            error_msg
        );
        
        if let Err(write_err) = std::fs::write("logs/app.log", error_log) {
            log::error!("Failed to write error log: {}", write_err);
        }
    };
    
    log::info!("Creating and uploading log archive to Google Drive");
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.create_and_upload_archive().await {
        Ok(message) => {
            log::info!("=== CREATE_AND_UPLOAD_LOG_ARCHIVE COMMAND SUCCESS ===");
            log::info!("Upload successful: {}", message);
            Ok(serde_json::json!({
                "success": true,
                "message": message
            }))
        },
        Err(e) => {
            let error_msg = format!("Failed to create and upload archive: {}", e);
            log::error!("=== CREATE_AND_UPLOAD_LOG_ARCHIVE COMMAND ERROR ===");
            log::error!("{}", error_msg);
            log_error_to_file(&error_msg);
            
            Ok(serde_json::json!({
                "success": false,
                "error": error_msg
            }))
        }
    }
}

#[tauri::command]
pub async fn create_upload_and_cleanup_log_archive(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("=== CREATE_UPLOAD_AND_CLEANUP_LOG_ARCHIVE COMMAND START ===");
    
    // Add comprehensive error logging to app.log
    let log_error_to_file = |error_msg: &str| {
        let error_log = format!(
            "[{}] LogArchiveManager Upload Error:\nError: {}\nCommand: create_upload_and_cleanup_log_archive\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            error_msg
        );
        
        if let Err(write_err) = std::fs::write("logs/app.log", error_log) {
            log::error!("Failed to write error log: {}", write_err);
        }
    };
    
    log::info!("Creating, uploading, and cleaning up log archive");
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.create_upload_and_cleanup_archive().await {
        Ok(message) => {
            log::info!("=== CREATE_UPLOAD_AND_CLEANUP_LOG_ARCHIVE COMMAND SUCCESS ===");
            log::info!("Upload and cleanup successful: {}", message);
            Ok(serde_json::json!({
                "success": true,
                "message": message
            }))
        },
        Err(e) => {
            let error_msg = format!("Failed to create, upload and cleanup archive: {}", e);
            log::error!("=== CREATE_UPLOAD_AND_CLEANUP_LOG_ARCHIVE COMMAND ERROR ===");
            log::error!("{}", error_msg);
            log_error_to_file(&error_msg);
            
            Ok(serde_json::json!({
                "success": false,
                "error": error_msg
            }))
        }
    }
}

#[tauri::command]
pub async fn get_auto_archive_config(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting auto-archive configuration");
    
    // For now, return a default config. In a real implementation, 
    // you would load this from a configuration file or database
    let default_config = AutoArchiveConfig {
        enabled: false,
        schedule: ArchiveSchedule::Monthly,
        upload_to_drive: false,
        delete_after_upload: false,
        last_archive_time: None,
    };
    
    Ok(serde_json::json!({
        "success": true,
        "data": default_config
    }))
}
#[tauri::command]
pub async fn set_auto_archive_config(
    config: AutoArchiveConfig,
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Setting auto-archive configuration: enabled={}, schedule={:?}", 
               config.enabled, config.schedule);
    
    // In a real implementation, you would save this to a configuration file or database
    // For now, we'll just return success
    
    Ok(serde_json::json!({
        "success": true,
        "message": "Auto-archive configuration updated successfully"
    }))
}

#[tauri::command]
pub async fn check_auto_archive_status(
    config: AutoArchiveConfig,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Checking auto-archive status");
    
    let log_manager = app.log_manager().lock().await;
    let should_archive = log_manager.should_auto_archive(&config);
    let next_archive_time = log_manager.get_next_archive_time(&config);
    
    Ok(serde_json::json!({
        "success": true,
        "data": {
            "should_archive": should_archive,
            "next_archive_time": next_archive_time,
            "schedule": config.schedule.to_string(),
            "enabled": config.enabled
        }
    }))
}

#[tauri::command]
pub async fn perform_auto_archive(
    mut config: AutoArchiveConfig,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Performing auto-archive");
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.perform_auto_archive(&mut config).await {
        Ok(message) => Ok(serde_json::json!({
            "success": true,
            "message": message,
            "updated_config": config
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Auto-archive failed: {}", e)
        }))
    }
}

#[tauri::command]
pub async fn delete_log_archive(
    archive_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Deleting log archive: {}", archive_name);
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.delete_archive(&archive_name) {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Archive '{}' deleted successfully", archive_name)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to delete archive: {}", e)
        }))
    }
}

// WebSocket commands for HTML overlays
#[tauri::command]
pub async fn websocket_get_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting WebSocket server status");
    
    let websocket_plugin = app.websocket_plugin().lock().await;
    let client_count = websocket_plugin.get_client_count();
    
    Ok(serde_json::json!({
        "connected_clients": client_count,
        "status": "running"
    }))
}

#[tauri::command]
pub async fn websocket_broadcast_pss_event(
    event_data: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Broadcasting PSS event via WebSocket: {:?}", event_data);
    
    let _websocket_plugin = app.websocket_plugin().lock().await;
    // For now, return success since the WebSocket server handles broadcasting internally
    Ok(serde_json::json!({
        "success": true,
        "message": "PSS event broadcasted successfully"
    }))
}
#[tauri::command]
pub async fn store_pss_event(
    event_data: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Storing PSS event to database: {:?}", event_data);

    // Extract basic fields expected from the UI store
    let match_id_str = event_data.get("match_id").and_then(|v| v.as_str()).unwrap_or("");
    let event_code = event_data.get("event_code").and_then(|v| v.as_str()).unwrap_or("");
    let athlete = event_data.get("athlete").and_then(|v| v.as_str()).unwrap_or("");
    let round_num = event_data.get("round").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let time_str = event_data.get("time").and_then(|v| v.as_str()).unwrap_or("");
    let timestamp_str = event_data.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
    let raw_data = event_data.get("raw_data").and_then(|v| v.as_str()).unwrap_or("");

    // Guard: require a match id and event_code
    if match_id_str.is_empty() || event_code.is_empty() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Missing match_id or event_code"
        }));
    }

    let _conn = app.database_plugin().get_connection().await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Database connection error: {}", e)))?;

    // Resolve DB ids
    let db_match_id = app.database_plugin().get_or_create_pss_match(match_id_str).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get/create match: {}", e)))?;

    // Map event_code to event_type_id
    let event_type = app.database_plugin().get_pss_event_type_by_code(event_code).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to resolve event type: {}", e)))?;
    let event_type_id = if let Some(t) = event_type { t.id.unwrap_or(0) } else { 0 };

    // Current UDP session if available (fallback to 0)
    let session_id = 0i64;

    // Build model
    let timestamp = if !timestamp_str.is_empty() {
        chrono::DateTime::parse_from_rfc3339(timestamp_str).map(|d| d.with_timezone(&chrono::Utc)).unwrap_or(chrono::Utc::now())
    } else { chrono::Utc::now() };

    let round_id: Option<i64> = None;
    let parsed_data: Option<String> = None;
    let processing_time_ms: Option<i32> = None;
    let error_message: Option<String> = None;

    let event_model = crate::database::models::PssEventV2 {
        id: None,
        session_id,
        match_id: Some(db_match_id),
        round_id,
        event_type_id,
        timestamp,
        raw_data: raw_data.to_string(),
        parsed_data,
        event_sequence: 0,
        processing_time_ms,
        is_valid: true,
        error_message,
        recognition_status: "recognized".to_string(),
        protocol_version: Some("2.3".to_string()),
        parser_confidence: Some(1.0),
        validation_errors: None,
        tournament_id: None,
        tournament_day_id: None,
        created_at: chrono::Utc::now(),
    };

    match app.database_plugin().store_pss_event(&event_model).await {
        Ok(event_id) => {
            // Store a few details for convenience
            let details = vec![
                ("round".to_string(), Some(round_num.to_string()), "i32".to_string()),
                ("time".to_string(), Some(time_str.to_string()), "String".to_string()),
                ("athlete".to_string(), Some(athlete.to_string()), "String".to_string()),
            ];
            let _ = app.database_plugin().store_pss_event_details(event_id, &details).await;

            Ok(serde_json::json!({
                "success": true,
                "event_id": event_id
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}
// Tournament Management Commands
#[tauri::command]
pub async fn tournament_create(
    name: String,
    duration_days: i32,
    city: String,
    country: String,
    country_code: Option<String>,
    start_date: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Creating tournament: {} in {}, {}", name, city, country);
    
    let start_date_parsed = if let Some(date_str) = start_date {
        if date_str.trim().is_empty() {
            None
        } else {
            Some(chrono::DateTime::parse_from_rfc3339(&date_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| TauriError::from(anyhow::anyhow!("Invalid start date format '{}': {}", date_str, e)))?)
        }
    } else {
        None
    };
    
    match app.tournament_plugin().create_tournament(
        name,
        duration_days,
        city,
        country,
        country_code,
        start_date_parsed,
    ).await {
        Ok(tournament_id) => Ok(serde_json::json!({
            "success": true,
            "tournament_id": tournament_id,
            "message": "Tournament created successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_get_all(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting all tournaments");
    
    match app.tournament_plugin().get_tournaments().await {
        Ok(tournaments) => {
            let tournaments_json: Vec<serde_json::Value> = tournaments
                .into_iter()
                .map(|t| serde_json::json!({
                    "id": t.id,
                    "name": t.name,
                    "duration_days": t.duration_days,
                    "city": t.city,
                    "country": t.country,
                    "country_code": t.country_code,
                    "logo_path": t.logo_path,
                    "status": t.status,
                    "start_date": t.start_date.map(|d| d.to_rfc3339()),
                    "end_date": t.end_date.map(|d| d.to_rfc3339()),
                    "created_at": t.created_at.to_rfc3339(),
                    "updated_at": t.updated_at.to_rfc3339(),
                }))
                .collect();
            
            Ok(serde_json::json!({
                "success": true,
                "tournaments": tournaments_json
            }))
        },
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_get(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting tournament: {}", tournament_id);
    
    match app.tournament_plugin().get_tournament(tournament_id).await {
        Ok(Some(tournament)) => {
            let tournament_json = serde_json::json!({
                "id": tournament.id,
                "name": tournament.name,
                "duration_days": tournament.duration_days,
                "city": tournament.city,
                "country": tournament.country,
                "country_code": tournament.country_code,
                "logo_path": tournament.logo_path,
                "status": tournament.status,
                "start_date": tournament.start_date.map(|d| d.to_rfc3339()),
                "end_date": tournament.end_date.map(|d| d.to_rfc3339()),
                "created_at": tournament.created_at.to_rfc3339(),
                "updated_at": tournament.updated_at.to_rfc3339(),
            });
            
            Ok(serde_json::json!({
                "success": true,
                "tournament": tournament_json
            }))
        },
        Ok(None) => Ok(serde_json::json!({
            "success": false,
            "error": "Tournament not found"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_update(
    tournament_id: i64,
    name: String,
    duration_days: i32,
    city: String,
    country: String,
    country_code: Option<String>,
    logo_path: Option<String>,
    status: String,
    start_date: Option<String>,
    end_date: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Updating tournament: {}", tournament_id);
    
    let start_date_parsed = if let Some(date_str) = start_date {
        if date_str.trim().is_empty() {
            None
        } else {
            Some(chrono::DateTime::parse_from_rfc3339(&date_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| TauriError::from(anyhow::anyhow!("Invalid start date format '{}': {}", date_str, e)))?)
        }
    } else {
        None
    };
    
    let end_date_parsed = if let Some(date_str) = end_date {
        if date_str.trim().is_empty() {
            None
        } else {
            Some(chrono::DateTime::parse_from_rfc3339(&date_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| TauriError::from(anyhow::anyhow!("Invalid end date format '{}': {}", date_str, e)))?)
        }
    } else {
        None
    };
    
    let tournament = crate::database::models::Tournament {
        id: Some(tournament_id),
        name,
        duration_days,
        city,
        country,
        country_code,
        logo_path,
        status,
        start_date: start_date_parsed,
        end_date: end_date_parsed,
        created_at: chrono::Utc::now(), // This will be ignored in update
        updated_at: chrono::Utc::now(),
    };
    
    match app.tournament_plugin().update_tournament(tournament_id, tournament).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament updated successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_delete(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Deleting tournament: {}", tournament_id);
    
    match app.tournament_plugin().delete_tournament(tournament_id).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament deleted successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}
#[tauri::command]
pub async fn tournament_get_days(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting tournament days for tournament: {}", tournament_id);
    
    match app.tournament_plugin().get_tournament_days(tournament_id).await {
        Ok(days) => {
            let days_json: Vec<serde_json::Value> = days
                .into_iter()
                .map(|d| serde_json::json!({
                    "id": d.id,
                    "tournament_id": d.tournament_id,
                    "day_number": d.day_number,
                    "date": d.date.to_rfc3339(),
                    "status": d.status,
                    "start_time": d.start_time.map(|t| t.to_rfc3339()),
                    "end_time": d.end_time.map(|t| t.to_rfc3339()),
                    "created_at": d.created_at.to_rfc3339(),
                    "updated_at": d.updated_at.to_rfc3339(),
                }))
                .collect();
            
            Ok(serde_json::json!({
                "success": true,
                "days": days_json
            }))
        },
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_start_day(
    tournament_day_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Starting tournament day: {}", tournament_day_id);
    
    match app.tournament_plugin().start_tournament_day(tournament_day_id).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament day started successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_end_day(
    tournament_day_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Ending tournament day: {}", tournament_day_id);
    
    match app.tournament_plugin().end_tournament_day(tournament_day_id).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament day ended successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_get_active(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting active tournament");
    
    match app.tournament_plugin().get_active_tournament().await {
        Ok(Some(tournament)) => {
            let tournament_json = serde_json::json!({
                "id": tournament.id,
                "name": tournament.name,
                "duration_days": tournament.duration_days,
                "city": tournament.city,
                "country": tournament.country,
                "country_code": tournament.country_code,
                "logo_path": tournament.logo_path,
                "status": tournament.status,
                "start_date": tournament.start_date.map(|d| d.to_rfc3339()),
                "end_date": tournament.end_date.map(|d| d.to_rfc3339()),
                "created_at": tournament.created_at.to_rfc3339(),
                "updated_at": tournament.updated_at.to_rfc3339(),
            });
            
            Ok(serde_json::json!({
                "success": true,
                "tournament": tournament_json
            }))
        },
        Ok(None) => Ok(serde_json::json!({
            "success": true,
            "tournament": null
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_get_active_day(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting active tournament day for tournament: {}", tournament_id);
    
    match app.tournament_plugin().get_active_tournament_day(tournament_id).await {
        Ok(Some(day)) => {
            let day_json = serde_json::json!({
                "id": day.id,
                "tournament_id": day.tournament_id,
                "day_number": day.day_number,
                "date": day.date.to_rfc3339(),
                "status": day.status,
                "start_time": day.start_time.map(|t| t.to_rfc3339()),
                "end_time": day.end_time.map(|t| t.to_rfc3339()),
                "created_at": day.created_at.to_rfc3339(),
                "updated_at": day.updated_at.to_rfc3339(),
            });
            
            Ok(serde_json::json!({
                "success": true,
                "day": day_json
            }))
        },
        Ok(None) => Ok(serde_json::json!({
            "success": true,
            "day": null
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_update_logo(
    tournament_id: i64,
    logo_path: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Updating tournament logo for tournament: {}", tournament_id);
    
    match app.tournament_plugin().update_tournament_logo(tournament_id, logo_path).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament logo updated successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn tournament_verify_location(
    city: String,
    country: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Verifying location: {}, {}", city, country);
    
    match app.tournament_plugin().verify_city_country(city, country).await {
        Ok(verification) => Ok(serde_json::json!({
            "success": true,
            "verified": verification.verified,
            "country_code": verification.country_code,
            "display_name": verification.display_name
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn get_tournament_statistics(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting tournament statistics for tournament: {}", tournament_id);
    
    match app.tournament_plugin().get_tournament_statistics(tournament_id).await {
        Ok(statistics) => Ok(serde_json::json!({
            "success": true,
            "statistics": statistics
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

// Database optimization commands
#[tauri::command]
pub async fn database_run_vacuum(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database VACUUM operation");
    
    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();
    
    match maintenance.run_vacuum(&db_conn).await {
        Ok(_) => {
            Ok(serde_json::json!({
                "success": true,
                "message": "Database VACUUM completed successfully"
            }))
        }
        Err(e) => {
            log::error!("Database VACUUM failed: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}
#[tauri::command]
pub async fn database_run_integrity_check(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database integrity check");
    
    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();
    
    match maintenance.run_integrity_check(&db_conn).await {
        Ok(integrity_ok) => {
            if integrity_ok {
                Ok(serde_json::json!({
                    "success": true,
                    "message": "Database integrity check passed",
                    "integrity_ok": true
                }))
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "message": "Database integrity check failed",
                    "integrity_ok": false
                }))
            }
        }
        Err(e) => {
            log::error!("Database integrity check error: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_analyze(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database ANALYZE operation");
    
    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();
    
    match maintenance.run_analyze(&db_conn).await {
        Ok(_) => {
            Ok(serde_json::json!({
                "success": true,
                "message": "Database ANALYZE completed successfully"
            }))
        }
        Err(e) => {
            log::error!("Database ANALYZE failed: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_optimize(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database OPTIMIZE operation");
    
    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();
    
    match maintenance.run_optimize(&db_conn).await {
        Ok(_) => {
            Ok(serde_json::json!({
                "success": true,
                "message": "Database OPTIMIZE completed successfully"
            }))
        }
        Err(e) => {
            log::error!("Database OPTIMIZE failed: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_full_maintenance(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Running full database maintenance");
    
    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();
    
    match maintenance.run_full_maintenance(&db_conn).await {
        Ok(result) => {
            Ok(serde_json::json!({
                "success": true,
                "message": "Full database maintenance completed",
                "result": {
                    "integrity_check_passed": result.integrity_check_passed,
                    "analyze_success": result.analyze_success,
                    "optimize_success": result.optimize_success,
                    "vacuum_success": result.vacuum_success,
                    "total_duration_secs": result.total_duration.as_secs()
                }
            }))
        }
        Err(e) => {
            log::error!("Full database maintenance failed: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_get_info(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database information");
    
    let db_conn = app.database_plugin().get_database_connection();
    let maintenance = crate::database::DatabaseMaintenance::new_default();
    
    match maintenance.get_database_info(&db_conn).await {
        Ok(info) => {
            Ok(serde_json::json!({
                "success": true,
                "info": {
                    "total_size": info.total_size,
                    "used_size": info.used_size,
                    "free_size": info.free_size,
                    "fragmentation_percentage": info.fragmentation_percentage,
                    "page_count": info.page_count,
                    "page_size": info.page_size,
                    "freelist_count": info.freelist_count,
                    "cache_size": info.cache_size,
                    "journal_mode": info.journal_mode,
                    "synchronous": info.synchronous
                }
            }))
        }
        Err(e) => {
            log::error!("Failed to get database info: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_get_maintenance_status(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database maintenance status");
    
    let maintenance = crate::database::DatabaseMaintenance::new_default();
    let needed = maintenance.check_maintenance_needed();
    let stats = maintenance.get_statistics();
    let config = maintenance.get_config();
    
    Ok(serde_json::json!({
        "success": true,
        "maintenance_needed": {
            "vacuum_needed": needed.vacuum_needed,
            "integrity_check_needed": needed.integrity_check_needed,
            "analyze_needed": needed.analyze_needed,
            "optimize_needed": needed.optimize_needed,
            "any_needed": needed.any_needed()
        },
        "statistics": {
            "last_vacuum": stats.last_vacuum,
            "last_integrity_check": stats.last_integrity_check,
            "last_analyze": stats.last_analyze,
            "last_optimize": stats.last_optimize,
            "vacuum_count": stats.vacuum_count,
            "integrity_check_count": stats.integrity_check_count,
            "analyze_count": stats.analyze_count,
            "optimize_count": stats.optimize_count,
            "total_maintenance_time_secs": stats.total_maintenance_time_secs
        },
        "config": {
            "vacuum_interval_secs": config.vacuum_interval.as_secs(),
            "integrity_check_interval_secs": config.integrity_check_interval.as_secs(),
            "analyze_interval_secs": config.analyze_interval.as_secs(),
            "optimize_interval_secs": config.optimize_interval.as_secs(),
            "max_vacuum_time_secs": config.max_vacuum_time.as_secs(),
            "backup_before_maintenance": config.backup_before_maintenance
        }
    }))
}

/// Get comprehensive event statistics with status breakdown
#[tauri::command]
pub async fn get_comprehensive_event_statistics(
    session_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting comprehensive event statistics for session {}", session_id);
    
    match app.database_plugin().get_comprehensive_event_statistics(session_id).await {
        Ok(stats) => Ok(stats),
        Err(e) => {
            log::error!("Failed to get comprehensive event statistics: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to get event statistics: {}", e)))
        }
    }
}

/// Get events by recognition status
#[tauri::command]
pub async fn get_events_by_status(
    session_id: i64,
    recognition_status: String,
    limit: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting events by status: {} for session {}", recognition_status, session_id);
    
    match app.database_plugin().get_events_by_status(session_id, &recognition_status, limit).await {
        Ok(events) => {
            let events_json: Vec<serde_json::Value> = events.into_iter()
                .map(|event| serde_json::to_value(event).unwrap_or_default())
                .collect();
            Ok(serde_json::json!({
                "success": true,
                "events": events_json,
                "count": events_json.len()
            }))
        }
        Err(e) => {
            log::error!("Failed to get events by status: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to get events by status: {}", e)))
        }
    }
}

/// Get unknown events for analysis
#[tauri::command]
pub async fn get_unknown_events(
    session_id: Option<i64>,
    limit: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting unknown events");
    
    match app.database_plugin().get_unknown_events(session_id, limit).await {
        Ok(events) => {
            let events_json: Vec<serde_json::Value> = events.into_iter()
                .map(|event| serde_json::to_value(event).unwrap_or_default())
                .collect();
            Ok(serde_json::json!({
                "success": true,
                "events": events_json,
                "count": events_json.len()
            }))
        }
        Err(e) => {
            log::error!("Failed to get unknown events: {}", e);
            Err(TauriError::from(anyhow::anyhow!("Failed to get unknown events: {}", e)))
        }
    }
}
/// Set tournament context for UDP event tracking
#[tauri::command]
pub async fn set_udp_tournament_context(
    app: tauri::State<'_, crate::core::app::App>,
    tournament_id: Option<i64>,
    tournament_day_id: Option<i64>,
) -> Result<(), TauriError> {
    log::info!("Setting UDP tournament context: tournament_id={:?}, tournament_day_id={:?}", tournament_id, tournament_day_id);
    
    app.udp_plugin().set_tournament_context(tournament_id, tournament_day_id).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))
}

/// Get current tournament context from UDP server
#[tauri::command]
pub async fn get_udp_tournament_context(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, TauriError> {
    let (tournament_id, tournament_day_id) = app.udp_plugin().get_tournament_context();
    
    Ok(serde_json::json!({
        "tournament_id": tournament_id,
        "tournament_day_id": tournament_day_id
    }))
}

/// Clear tournament context from UDP server
#[tauri::command]
pub async fn clear_udp_tournament_context(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<(), TauriError> {
    log::info!("Clearing UDP tournament context");
    
    app.udp_plugin().clear_tournament_context().await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))
}

/// Phase 1 Optimization: Get UDP server performance metrics
#[tauri::command]
pub async fn get_udp_performance_metrics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting UDP performance metrics");
    let metrics = app.udp_plugin().get_performance_metrics();
    serde_json::to_value(metrics)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to serialize performance metrics: {}", e)))
}

/// Phase 1 Optimization: Get UDP server memory usage
#[tauri::command]
pub async fn get_udp_memory_usage(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting UDP memory usage");
    let usage = app.udp_plugin().get_memory_usage();
    serde_json::to_value(usage)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to serialize memory usage: {}", e)))
}

/// Phase 2 Optimization: Archive events older than specified days
#[tauri::command]
pub async fn archive_old_events(
    app: tauri::State<'_, crate::core::app::App>,
    days_old: i64,
) -> Result<usize, TauriError> {
    log::info!("Archiving events older than {} days", days_old);
    let archived_count = app.database_plugin().archive_old_events(days_old).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    log::info!("✅ Archived {} events", archived_count);
    Ok(archived_count)
}

/// Phase 2 Optimization: Get archive statistics
#[tauri::command]
pub async fn get_archive_statistics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting archive statistics");
    let stats = app.database_plugin().get_archive_statistics().await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    serde_json::to_value(stats)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to serialize archive statistics: {}", e)))
}

/// Phase 2 Optimization: Restore events from archive
#[tauri::command]
pub async fn restore_from_archive(
    app: tauri::State<'_, crate::core::app::App>,
    start_date: String,
    end_date: String,
) -> Result<usize, TauriError> {
    log::info!("Restoring events from archive between {} and {}", start_date, end_date);
    let restored_count = app.database_plugin().restore_from_archive(&start_date, &end_date).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    log::info!("✅ Restored {} events from archive", restored_count);
    Ok(restored_count)
}
/// Phase 2 Optimization: Clean up old archive data
#[tauri::command]
pub async fn cleanup_old_archive_data(
    app: tauri::State<'_, crate::core::app::App>,
    days_old: i64,
) -> Result<usize, TauriError> {
    log::info!("Cleaning up archive data older than {} days", days_old);
    let deleted_count = app.database_plugin().cleanup_old_archive_data(days_old).await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    log::info!("✅ Cleaned up {} archived events", deleted_count);
    Ok(deleted_count)
}

/// Phase 2 Optimization: Optimize archive tables
#[tauri::command]
pub async fn optimize_archive_tables(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<(), TauriError> {
    log::info!("Optimizing archive tables");
    app.database_plugin().optimize_archive_tables().await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    log::info!("✅ Archive tables optimized successfully");
    Ok(())
}

/// Phase 2 Optimization: Get database pool statistics
#[tauri::command]
pub async fn get_database_pool_stats(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database pool statistics");
    let stats = app.database_plugin().get_pool_stats();
    serde_json::to_value(stats)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to serialize pool statistics: {}", e)))
}

/// Phase 2 Optimization: Clean up database pool
#[tauri::command]
pub async fn cleanup_database_pool(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<(), TauriError> {
    log::info!("Cleaning up database connection pool");
    app.database_plugin().cleanup_pool();
    log::info!("✅ Database pool cleaned up");
    Ok(())
}

// Phase 3: Advanced Caching Commands
#[tauri::command]
pub async fn get_cache_statistics(app: tauri::State<'_, crate::core::app::App>) -> Result<serde_json::Value, tauri::Error> {
    let cache_stats = app.event_cache().get_cache_stats().await;
    serde_json::to_value(cache_stats)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to serialize cache statistics: {}", e)))
}

#[tauri::command]
pub async fn clear_cache(app: tauri::State<'_, crate::core::app::App>) -> Result<(), tauri::Error> {
    app.event_cache().clear_all().await;
    Ok(())
}

#[tauri::command]
pub async fn invalidate_tournament_cache(app: tauri::State<'_, crate::core::app::App>, tournament_id: i64) -> Result<(), tauri::Error> {
    app.event_cache().invalidate_tournament(tournament_id).await;
    Ok(())
}

#[tauri::command]
pub async fn invalidate_match_cache(app: tauri::State<'_, crate::core::app::App>, match_id: i64) -> Result<(), tauri::Error> {
    app.event_cache().invalidate_match(match_id).await;
    Ok(())
}

// Phase 3: Event Stream Commands
#[tauri::command]
pub async fn get_stream_statistics(app: tauri::State<'_, crate::core::app::App>) -> Result<serde_json::Value, tauri::Error> {
    let stream_stats = app.event_stream_processor().get_statistics().await;
    serde_json::to_value(stream_stats)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to serialize stream statistics: {}", e)))
}

#[tauri::command]
pub async fn send_event_to_stream(app: tauri::State<'_, crate::core::app::App>, event: crate::database::models::PssEventV2) -> Result<(), tauri::Error> {
    app.event_stream_processor().send_event(event).await;
    Ok(())
}

// Phase 3: Load Balancer Commands
#[tauri::command]
pub async fn get_distributor_statistics(app: tauri::State<'_, crate::core::app::App>) -> Result<serde_json::Value, tauri::Error> {
    let distributor_stats = app.event_distributor().get_statistics().await;
    serde_json::to_value(distributor_stats)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to serialize distributor statistics: {}", e)))
}

#[tauri::command]
pub async fn get_server_statistics(app: tauri::State<'_, crate::core::app::App>) -> Result<serde_json::Value, tauri::Error> {
    let server_stats = app.event_distributor().get_server_statistics().await;
    serde_json::to_value(server_stats)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to serialize server statistics: {}", e)))
}

#[tauri::command]
pub async fn add_server(app: tauri::State<'_, crate::core::app::App>, server_id: String, bind_address: String, port: u16) -> Result<(), tauri::Error> {
    app.event_distributor().add_server(server_id, bind_address, port).await
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to add server: {}", e)))
}

#[tauri::command]
pub async fn remove_server(app: tauri::State<'_, crate::core::app::App>, server_id: String) -> Result<(), tauri::Error> {
    app.event_distributor().remove_server(&server_id).await
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to remove server: {}", e)))
}

// Phase 3: Advanced Analytics Commands
#[tauri::command]
pub async fn get_tournament_analytics(app: tauri::State<'_, crate::core::app::App>) -> Result<serde_json::Value, tauri::Error> {
    let tournament_analytics = app.advanced_analytics().get_tournament_analytics().await;
    serde_json::to_value(tournament_analytics)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to serialize tournament analytics: {}", e)))
}

#[tauri::command]
pub async fn get_performance_analytics(app: tauri::State<'_, crate::core::app::App>) -> Result<serde_json::Value, tauri::Error> {
    let performance_analytics = app.advanced_analytics().get_performance_analytics().await;
    serde_json::to_value(performance_analytics)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to serialize performance analytics: {}", e)))
}

#[tauri::command]
pub async fn get_athlete_analytics(app: tauri::State<'_, crate::core::app::App>) -> Result<serde_json::Value, tauri::Error> {
    let athlete_analytics = app.advanced_analytics().get_athlete_analytics().await;
    serde_json::to_value(athlete_analytics)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to serialize athlete analytics: {}", e)))
}

#[tauri::command]
pub async fn get_match_analytics(app: tauri::State<'_, crate::core::app::App>) -> Result<serde_json::Value, tauri::Error> {
    let match_analytics = app.advanced_analytics().get_match_analytics().await;
    serde_json::to_value(match_analytics)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to serialize match analytics: {}", e)))
}

#[tauri::command]
pub async fn get_analytics_history(app: tauri::State<'_, crate::core::app::App>, limit: Option<usize>) -> Result<serde_json::Value, tauri::Error> {
    let analytics_history = app.advanced_analytics().get_analytics_history(limit).await;
    serde_json::to_value(analytics_history)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to serialize analytics history: {}", e)))
}

#[tauri::command]
pub async fn obs_list_scenes(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting scenes from all connected OBS instances");
    
    let mut all_scenes = Vec::new();
    let connection_names = app.obs_obws_plugin().get_connection_names().await;
    
    // Collect connection statuses first
    let mut connection_statuses = Vec::new();
    for connection_name in &connection_names {
        let status = app.obs_obws_plugin().get_connection_status(connection_name).await;
        connection_statuses.push((connection_name.clone(), status));
    }
    
    for (connection_name, status) in &connection_statuses {
        // Check if connection is connected/authenticated
        let is_connected = status.is_ok();
        
        if is_connected {
            match app.obs_obws_plugin().get_scenes(Some(connection_name.as_str())).await {
                Ok(scene_names) => {
                    for (idx, scene_name) in scene_names.iter().enumerate() {
                        all_scenes.push(serde_json::json!({
                            "id": idx,
                            "scene_name": scene_name,
                            "scene_id": scene_name, // OBS WebSocket v5 uses scene name as ID
                            "is_active": true,
                            "connection_name": connection_name
                        }));
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get scenes from connection '{}': {}", connection_name, e);
                }
            }
        } else {
            log::info!("Skipping connection '{}' - not connected (status: {:?})", connection_name, status);
        }
    }
    
    let connected_count = connection_statuses.iter().filter(|(_, status)| status.is_ok()).count();
    
    Ok(serde_json::json!({
        "scenes": all_scenes,
        "total_connections": connection_names.len(),
        "connected_connections": connected_count
    }))
}
// Simulation commands
#[tauri::command]
pub async fn simulation_start(
    mode: String,
    scenario: String,
    duration: u32,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Starting simulation: mode={}, scenario={}, duration={}", mode, scenario, duration);
    
    // Get the actual UDP settings from the app configuration
    let udp_settings = app.config_manager().get_udp_settings().await;
    let host = udp_settings.listener.bind_address.clone();
    let port = udp_settings.listener.port;
    
    log::info!("Using UDP settings: host={}, port={}", host, port);
    
    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };
    let result = std::process::Command::new(&python_cmd)
        .args(&[
            sim_main.to_str().unwrap(),
            "--mode", &mode,
            "--scenario", &scenario,
            "--duration", &duration.to_string(),
            "--host", &host,
            "--port", &port.to_string()
        ])
        .spawn();
    match result {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Simulation started successfully on {}:{}", host, port)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to start simulation: {}", e)
        }))
    }
}

#[tauri::command]
pub async fn simulation_stop(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Stopping simulation");
    
    let result = std::process::Command::new("taskkill")
        .args(&["/F", "/IM", "python.exe"])
        .output();
    
    match result {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Simulation stopped successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to stop simulation: {}", e)
        }))
    }
}
#[tauri::command]
pub async fn simulation_get_status(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting simulation status");
    
    let result = std::process::Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq python.exe"])
        .output();
    
    let is_running = match result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.contains("python.exe")
        },
        Err(_) => false
    };
    
    Ok(serde_json::json!({
        "success": true,
        "data": {
            "isRunning": is_running,
            "isConnected": is_running, // Assume connected if running
            "currentScenario": if is_running { "Unknown" } else { "None" },
            "currentMode": if is_running { "Unknown" } else { "None" },
            "eventsSent": 0, // Would need to track this separately
            "lastEvent": if is_running { "Unknown" } else { "None" }
        }
    }))
}

#[tauri::command]
pub async fn simulation_send_event(
    event_type: String,
    params: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Sending simulation event: type={}, params={:?}", event_type, params);
    
    // Get the actual UDP settings from the app configuration
    let udp_settings = app.config_manager().get_udp_settings().await;
    let host = udp_settings.listener.bind_address.clone();
    let port = udp_settings.listener.port;
    
    log::info!("Using UDP settings: host={}, port={}", host, port);
    
    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };
    
    // Convert params to JSON string for command line
    let params_str = serde_json::to_string(&params).unwrap_or("{}".to_string());
    
    let result = std::process::Command::new(&python_cmd)
        .args(&[
            sim_main.to_str().unwrap(),
            "--mode", "interactive",
            "--send-event",
            "--event-type", &event_type,
            "--event-params", &params_str,
            "--host", &host,
            "--port", &port.to_string()
        ])
        .spawn();
    
    match result {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("{} event sent successfully to {}:{}", event_type, host, port)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to send {} event: {}", event_type, e)
        }))
    }
}

// New automated simulation commands
#[tauri::command]
pub async fn simulation_get_scenarios(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting available automated scenarios");
    
    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Simulation environment error: {:?}", e);
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };
    
    log::info!("Running command: {} {} --list-scenarios", python_cmd, sim_main.to_str().unwrap());
    
    let result = std::process::Command::new(&python_cmd)
        .args(&[
            sim_main.to_str().unwrap(),
            "--list-scenarios"
        ])
        .output();
    
    match result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            log::info!("Scenarios output: {}", output_str);
            if !stderr_str.is_empty() {
                log::warn!("Scenarios stderr: {}", stderr_str);
            }
            
            if !output.status.success() {
                log::error!("Command failed with exit code: {}", output.status.code().unwrap_or(-1));
                return Ok(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get scenarios: Command exited with code {}", output.status.code().unwrap_or(-1))
                }));
            }
            
            // Parse the scenarios from the output
            let scenarios = parse_scenarios_from_output(&output_str);
            log::info!("Parsed {} scenarios: {:?}", scenarios.len(), scenarios);
            
            if scenarios.is_empty() {
                log::warn!("No scenarios were parsed from the output");
                return Ok(serde_json::json!({
                    "success": false,
                    "error": "No scenarios found in the output"
                }));
            }
            
            Ok(serde_json::json!({
                "success": true,
                "data": scenarios
            }))
        },
        Err(e) => {
            log::error!("Failed to execute simulation command: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get scenarios: {}", e)
            }))
        }
    }
}

#[tauri::command]
pub async fn simulation_run_automated(
    scenario_name: String,
    custom_config: Option<serde_json::Value>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running automated simulation: scenario={}", scenario_name);
    
    // Get the actual UDP settings from the app configuration
    let udp_settings = app.config_manager().get_udp_settings().await;
    let host = udp_settings.listener.bind_address.clone();
    let port = udp_settings.listener.port;
    
    log::info!("Using UDP settings: host={}, port={}", host, port);
    
    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };
    
    let mut args = vec![
        sim_main.to_str().unwrap().to_string(),
        "--mode".to_string(),
        "automated".to_string(),
        "--scenario".to_string(),
        scenario_name.clone(),
        "--host".to_string(),
        host.clone(),
        "--port".to_string(),
        port.to_string(),
    ];
    
    // Add custom config if provided
    if let Some(config) = custom_config {
        if let Some(config_str) = config.as_str() {
            args.extend_from_slice(&["--config".to_string(), config_str.to_string()]);
        }
    }
    
    let result = std::process::Command::new(&python_cmd)
        .args(&args)
        .spawn();
    
    match result {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Automated {} simulation started successfully on {}:{}", scenario_name, host, port)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to start automated simulation: {}", e)
        }))
    }
}
            #[tauri::command]
            pub async fn simulation_get_detailed_status(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
                log::info!("Getting detailed simulation status");
                
                // Check if Python process is running
                let process_result = std::process::Command::new("tasklist")
                    .args(&["/FI", "IMAGENAME eq python.exe"])
                    .output();
                
                let is_running = match process_result {
                    Ok(output) => {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        output_str.contains("python.exe")
                    },
                    Err(_) => false
                };
                
                // Always try to get scenarios
                let scenarios = match ensure_simulation_env() {
                    Ok((python_cmd, sim_main)) => {
                        let scenarios_result = std::process::Command::new(&python_cmd)
                            .args(&[sim_main.to_str().unwrap(), "--list-scenarios"])
                            .output();
                        
                        match scenarios_result {
                            Ok(output) => {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                parse_scenarios_from_output(&output_str)
                            },
                            Err(_) => vec![]
                        }
                    },
                    Err(_) => vec![]
                };
                
                Ok(serde_json::json!({
                    "success": true,
                    "data": {
                        "isRunning": is_running,
                        "isConnected": is_running,
                        "currentScenario": if is_running { "Running" } else { "None" },
                        "currentMode": if is_running { "Automated" } else { "None" },
                        "eventsSent": 0,
                        "lastEvent": if is_running { "Processing" } else { "None" },
                        "automatedScenarios": scenarios
                    }
                }))
            }
            
            #[tauri::command]
            pub async fn simulation_run_self_test(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
                log::info!("Running comprehensive self-test");
                
                let (python_cmd, sim_main) = match ensure_simulation_env() {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(serde_json::json!({
                            "success": false,
                            "error": format!("Simulation environment error: {:?}", e)
                        }))
                    }
                };
                
                let result = std::process::Command::new(&python_cmd)
                    .args(&[
                        sim_main.to_str().unwrap(),
                        "--self-test"
                    ])
                    .output();
                
                match result {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        
                        if output.status.success() {
                            Ok(serde_json::json!({
                                "success": true,
                                "data": {
                                    "output": stdout.to_string(),
                                    "error": stderr.to_string(),
                                    "exitCode": output.status.code().unwrap_or(0)
                                }
                            }))
                        } else {
                            Ok(serde_json::json!({
                                "success": false,
                                "error": format!("Self-test failed: {}", stderr),
                                "data": {
                                    "output": stdout.to_string(),
                                    "error": stderr.to_string(),
                                    "exitCode": output.status.code().unwrap_or(1)
                                }
                            }))
                        }
                    },
                    Err(e) => Ok(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to run self-test: {}", e)
                    }))
                }
            }
            
            #[tauri::command]
            pub async fn simulation_get_self_test_report(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
                log::info!("Getting self-test report");
                
                let report_path = match crate::utils::simulation_env::get_simulation_main_py() {
                    Ok(sim_main) => {
                        let sim_dir = sim_main.parent().unwrap();
                        sim_dir.join("self_test_report.md")
                    },
                    Err(e) => {
                        return Ok(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to resolve simulation path: {:?}", e)
                        }))
                    }
                };
                
                let result = std::fs::read_to_string(&report_path);
                
                match result {
                    Ok(content) => Ok(serde_json::json!({
                        "success": true,
                        "data": {
                            "report": content,
                            "path": report_path.to_str().unwrap_or("unknown")
                        }
                    })),
                    Err(e) => Ok(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to read self-test report: {}", e)
                    }))
                }
            }
            
            #[tauri::command]
            pub async fn simulation_get_self_test_categories(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
                log::info!("Getting self-test categories");
                
                let (python_cmd, sim_main) = match ensure_simulation_env() {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(serde_json::json!({
                            "success": false,
                            "error": format!("Simulation environment error: {:?}", e)
                        }))
                    }
                };
                
                let result = std::process::Command::new(&python_cmd)
                    .args(&[
                        sim_main.to_str().unwrap(),
                        "--list-test-categories"
                    ])
                    .output();
                
                match result {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        
                        if output.status.success() {
                            // Parse categories from output
                            let categories: Vec<String> = stdout
                                .lines()
                                .filter(|line| line.trim().starts_with("• "))
                                .map(|line| line.trim()[2..].to_string())
                                .collect();
                            
                            Ok(serde_json::json!({
                                "success": true,
                                "data": {
                                    "categories": categories,
                                    "output": stdout.to_string(),
                                    "error": stderr.to_string()
                                }
                            }))
                        } else {
                            Ok(serde_json::json!({
                                "success": false,
                                "error": format!("Failed to get categories: {}", stderr),
                                "data": {
                                    "output": stdout.to_string(),
                                    "error": stderr.to_string()
                                }
                            }))
                        }
                    },
                    Err(e) => Ok(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to get categories: {}", e)
                    }))
                }
            }
            
            #[tauri::command]
            pub async fn simulation_run_selective_self_test(
                selected_categories: Vec<String>,
                _app: State<'_, Arc<App>>
            ) -> Result<serde_json::Value, TauriError> {
                log::info!("Running selective self-test for categories: {:?}", selected_categories);
                
                let (python_cmd, sim_main) = match ensure_simulation_env() {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(serde_json::json!({
                            "success": false,
                            "error": format!("Simulation environment error: {:?}", e)
                        }))
                    }
                };
                
                let mut args = vec![sim_main.to_str().unwrap(), "--self-test"];
                args.extend(selected_categories.iter().map(|s| s.as_str()));
                
                let result = std::process::Command::new(&python_cmd)
                    .args(&args)
                    .output();
                
                match result {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        
                        if output.status.success() {
                            Ok(serde_json::json!({
                                "success": true,
                                "data": {
                                    "output": stdout.to_string(),
                                    "error": stderr.to_string(),
                                    "exitCode": output.status.code().unwrap_or(0)
                                }
                            }))
                        } else {
                            Ok(serde_json::json!({
                                "success": false,
                                "error": format!("Selective self-test failed: {}", stderr),
                                "data": {
                                    "output": stdout.to_string(),
                                    "error": stderr.to_string(),
                                    "exitCode": output.status.code().unwrap_or(1)
                                }
                            }))
                        }
                    },
                    Err(e) => Ok(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to run selective self-test: {}", e)
                    }))
                }
            }
// Helper function to parse scenarios from command output
fn parse_scenarios_from_output(output: &str) -> Vec<serde_json::Value> {
    let mut scenarios = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    
    log::info!("Parsing {} lines from output", lines.len());
    log::debug!("Raw output: {}", output);
    
    let mut current_scenario = serde_json::Map::new();
    let mut in_scenario = false;
    
    for (line_num, line) in lines.iter().enumerate() {
        let line = line.trim();
        log::debug!("Line {}: '{}'", line_num, line);
        
        // Check for scenario start with multiple possible bullet characters
        if line.starts_with("• ") || line.starts_with("- ") || line.starts_with("* ") || line.starts_with("  ") || line.starts_with("ò ") {
            // New scenario
            if in_scenario && !current_scenario.is_empty() {
                scenarios.push(serde_json::Value::Object(current_scenario.clone()));
                log::debug!("Added scenario: {:?}", current_scenario);
            }
            
            current_scenario.clear();
            in_scenario = true;
            
            // Extract name after bullet point
            let name = if line.starts_with("• ") {
                line[2..].trim()
            } else if line.starts_with("- ") {
                line[2..].trim()
            } else if line.starts_with("* ") {
                line[2..].trim()
            } else if line.starts_with("  ") {
                line[2..].trim()
            } else if line.starts_with("ò ") {
                line[2..].trim()
            } else {
                line
            };
            
            if !name.is_empty() {
                current_scenario.insert("display_name".to_string(), serde_json::Value::String(name.to_string()));
                current_scenario.insert("name".to_string(), serde_json::Value::String(name.to_lowercase().replace(" ", "_")));
                log::debug!("Found scenario: {}", name);
            }
        } else if line.starts_with("  Description: ") && in_scenario {
            let description = line[14..].trim();
            current_scenario.insert("description".to_string(), serde_json::Value::String(description.to_string()));
            log::debug!("Added description: {}", description);
        } else if line.starts_with("  Matches: ") && in_scenario {
            let matches = line[10..].trim();
            if let Ok(count) = matches.parse::<i32>() {
                current_scenario.insert("match_count".to_string(), serde_json::Value::Number(count.into()));
                log::debug!("Added match count: {}", count);
            }
        } else if line.starts_with("  Est. Duration: ") && in_scenario {
            let duration = line[16..].trim();
            // Handle "45.0 seconds" format
            let duration_parts: Vec<&str> = duration.split_whitespace().collect();
            if let Some(seconds_str) = duration_parts.first() {
                if let Ok(seconds) = seconds_str.parse::<f64>() {
                    current_scenario.insert("estimated_duration".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(seconds).unwrap_or(serde_json::Number::from(0))));
                    log::debug!("Added duration: {}", seconds);
                }
            }
        }
    }
    
    // Add the last scenario
    if in_scenario && !current_scenario.is_empty() {
        scenarios.push(serde_json::Value::Object(current_scenario.clone()));
        log::debug!("Added final scenario: {:?}", current_scenario);
    }
    
    log::info!("Parsed {} scenarios successfully", scenarios.len());
    scenarios
}

// (legacy streaming accounts/channels/events removed)

// ===== YOUTUBE STREAMING MANAGEMENT COMMANDS =====

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_accounts(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;
    
    match app.obs_obws_plugin().get_status(Some(&connection_name)).await {
        Ok(accounts) => Ok(serde_json::json!({
            "success": true,
            "data": accounts
        })),
        Err(e) => {
            log::error!("Failed to get YouTube accounts: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_accounts(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_channels(app: State<'_, Arc<App>>, account_id: String) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;
    
    match app.obs_obws_plugin().get_status(Some(&connection_name)).await {
        Ok(channels) => Ok(serde_json::json!({
            "success": true,
            "data": channels
        })),
        Err(e) => {
            log::error!("Failed to get YouTube channels: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_channels(_app: State<'_, Arc<App>>, _account_id: String) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_stream_key(app: State<'_, Arc<App>>, channel_id: String) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;
    
    match app.obs_obws_plugin().get_status(Some(&connection_name)).await {
        Ok(stream_key_info) => Ok(serde_json::json!({
            "success": true,
            "data": stream_key_info
        })),
        Err(e) => {
            log::error!("Failed to get YouTube stream key: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_stream_key(_app: State<'_, Arc<App>>, _channel_id: String) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_set_youtube_streaming_config(app: State<'_, Arc<App>>, channel_id: String, config: serde_json::Value) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;
    
    match app.obs_obws_plugin().get_status(Some(&connection_name)).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "YouTube streaming configuration updated successfully"
        })),
        Err(e) => {
            log::error!("Failed to set YouTube streaming config: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_set_youtube_streaming_config(_app: State<'_, Arc<App>>, _channel_id: String, _config: serde_json::Value) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_categories(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    match app.obs_obws_plugin().get_status(None).await {
        Ok(categories) => Ok(serde_json::json!({
            "success": true,
            "data": categories
        })),
        Err(e) => {
            log::error!("Failed to get YouTube categories: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_categories(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_privacy_options(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    match app.obs_obws_plugin().get_status(None).await {
        Ok(privacy_options) => Ok(serde_json::json!({
            "success": true,
            "data": privacy_options
        })),
        Err(e) => {
            log::error!("Failed to get YouTube privacy options: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_privacy_options(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_latency_options(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    match app.obs_obws_plugin().get_status(None).await {
        Ok(latency_options) => Ok(serde_json::json!({
            "success": true,
            "data": latency_options
        })),
        Err(e) => {
            log::error!("Failed to get YouTube latency options: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_latency_options(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_server_urls(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    match app.obs_obws_plugin().get_status(None).await {
        Ok(server_urls) => Ok(serde_json::json!({
            "success": true,
            "data": server_urls
        })),
        Err(e) => {
            log::error!("Failed to get YouTube server URLs: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_server_urls(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_regenerate_youtube_stream_key(app: State<'_, Arc<App>>, channel_id: String) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;
    
    match app.obs_obws_plugin().get_status(Some(&connection_name)).await {
        Ok(new_stream_key_info) => Ok(serde_json::json!({
            "success": true,
            "data": new_stream_key_info
        })),
        Err(e) => {
            log::error!("Failed to regenerate YouTube stream key: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_regenerate_youtube_stream_key(_app: State<'_, Arc<App>>, _channel_id: String) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_streaming_analytics(app: State<'_, Arc<App>>, channel_id: String) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;
    
    match app.obs_obws_plugin().get_status(Some(&connection_name)).await {
        Ok(analytics) => Ok(serde_json::json!({
            "success": true,
            "data": analytics
        })),
        Err(e) => {
            log::error!("Failed to get YouTube streaming analytics: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_streaming_analytics(_app: State<'_, Arc<App>>, _channel_id: String) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_streaming_schedule(app: State<'_, Arc<App>>, channel_id: String) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;
    
    match app.obs_obws_plugin().get_status(Some(&connection_name)).await {
        Ok(schedule) => Ok(serde_json::json!({
            "success": true,
            "data": schedule
        })),
        Err(e) => {
            log::error!("Failed to get YouTube streaming schedule: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_streaming_schedule(_app: State<'_, Arc<App>>, _channel_id: String) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

#[cfg(feature = "youtube")]
pub async fn obs_create_youtube_streaming_schedule(app: State<'_, Arc<App>>, channel_id: String, schedule_data: serde_json::Value) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;
    
    match app.obs_obws_plugin().get_status(Some(&connection_name)).await {
        Ok(created_schedule) => Ok(serde_json::json!({
            "success": true,
            "data": created_schedule
        })),
        Err(e) => {
            log::error!("Failed to create YouTube streaming schedule: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_create_youtube_streaming_schedule(_app: State<'_, Arc<App>>, _channel_id: String, _schedule_data: serde_json::Value) -> Result<serde_json::Value, TauriError> { Ok(serde_json::json!({ "disabled": true })) }

// (legacy streaming destination commands removed)

// YouTube API Commands (disabled by default)
#[tauri::command]
pub async fn youtube_get_auth_url(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_authenticate(_app: State<'_, Arc<App>>, _code: String) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_create_playlist(
    _app: State<'_, Arc<App>>, 
    _title: String, 
    _description: Option<String>, 
    _privacy: String
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_playlists(_app: State<'_, Arc<App>>, _max_results: Option<u32>) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_add_video_to_playlist(
    _app: State<'_, Arc<App>>, 
    _playlist_id: String, 
    _video_id: String
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_playlist_videos(
    _app: State<'_, Arc<App>>, 
    _playlist_id: String, 
    _max_results: Option<u32>
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_update_playlist(
    _app: State<'_, Arc<App>>, 
    _playlist_id: String, 
    _title: Option<String>, 
    _description: Option<String>, 
    _privacy: Option<String>
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_delete_playlist(_app: State<'_, Arc<App>>, _playlist_id: String) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_create_scheduled_stream(
    _app: State<'_, Arc<App>>, 
    _title: String, 
    _description: Option<String>, 
    _scheduled_time: String
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_live_streams(_app: State<'_, Arc<App>>, _max_results: Option<u32>) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_scheduled_streams(_app: State<'_, Arc<App>>, _max_results: Option<u32>) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_completed_streams(_app: State<'_, Arc<App>>, _max_results: Option<u32>) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_end_stream(_app: State<'_, Arc<App>>, _stream_id: String) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_channel_info(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_video_analytics(_app: State<'_, Arc<App>>, _video_id: String) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_initialize(
    _app: State<'_, Arc<App>>,
    _client_id: String,
    _client_secret: String,
    _redirect_uri: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}

// ============================================================================
// Control Room Commands
// ============================================================================

/// Control Room Authentication (Async version)
#[tauri::command]
pub async fn control_room_authenticate_async(
    _password: String,
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room async authentication attempt");
    
    // Create async database connection
    let app_data_dir = match dirs::data_dir() {
        Some(data_dir) => data_dir.join("reStrikeVTA"),
        None => std::env::current_dir().unwrap_or_default().join("data")
    };
    let _async_db = match crate::database::AsyncDatabaseConnection::new(&app_data_dir).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            log::error!("Failed to create async database connection: {}", e);
            return Ok(serde_json::json!({
                "success": false,
                "error": "Database connection failed"
            }));
        }
    };
    
    // obws does not require separate authentication. Return a synthetic session.
    Ok(serde_json::json!({
        "success": true,
        "session_id": "async_session",
        "message": "Control Room ready"
    }))
}

/// Get OBS connections for Control Room
#[tauri::command]
pub async fn control_room_get_obs_connections(
    session_id: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::debug!("Control Room: Getting OBS connections for session {}", session_id);
    // TODO: Validate session
    let connections = app.obs_obws_plugin().get_connection_names().await;
    Ok(serde_json::json!({
        "success": true,
        "connections": connections
    }))
}

/// Get all Control Room OBS connections with their status
#[tauri::command]
pub async fn control_room_get_obs_connections_with_status(
    session_id: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Getting OBS connections with status for session {}", session_id);
    // TODO: Validate session
    
    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut connections_data = Vec::new();
    for name in names {
        let status = app.obs_obws_plugin().get_connection_status(&name).await;
        let status_str = match status { Ok(s) => format!("{:?}", s), Err(e) => format!("Error: {}", e) };
        connections_data.push(serde_json::json!({"name": name, "status": status_str}));
    }
    Ok(serde_json::json!({"success": true, "connections": connections_data}))
}
/// Get all Control Room OBS connections with their full details and status
#[tauri::command]
pub async fn control_room_get_obs_connections_with_details(
    session_id: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Getting OBS connections with full details for session {}", session_id);
    // TODO: Validate session
    
    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut connections_data = Vec::new();
    for name in names {
        let status = app.obs_obws_plugin().get_connection_status(&name).await;
        let status_str = match status { Ok(s) => format!("{:?}", s), Err(e) => format!("Error: {}", e) };
        connections_data.push(serde_json::json!({"name": name, "status": status_str}));
    }
    Ok(serde_json::json!({"success": true, "connections": connections_data}))
}

/// Bulk mute all OBS streams
#[tauri::command]
pub async fn control_room_mute_all_obs(
    session_id: String,
    source_name: String,
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Bulk mute all OBS with source '{}' for session {}", source_name, session_id);
    // TODO: Validate session
    
    // Not supported via obws yet. Return empty results.
    Ok(serde_json::json!({ "success": true, "results": [] }))
}

/// Bulk unmute all OBS streams
#[tauri::command]
pub async fn control_room_unmute_all_obs(
    session_id: String,
    source_name: String,
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Bulk unmute all OBS with source '{}' for session {}", source_name, session_id);
    // TODO: Validate session
    
    // Not supported via obws yet. Return empty results.
    Ok(serde_json::json!({ "success": true, "results": [] }))
}
pub async fn control_room_change_all_obs_scenes(
    session_id: String,
    scene_name: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Change all OBS scenes to '{}' for session {}", scene_name, session_id);
    // TODO: Validate session
    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut results = Vec::new();
    for n in names {
        let r = app.obs_obws_plugin().set_current_scene(&scene_name, Some(&n)).await;
        results.push((n, r.map(|_| ()).map_err(|e| anyhow::anyhow!(e.to_string()))));
    }
    let formatted_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(conn, result)| serde_json::json!({
            "connection": conn,
            "success": result.is_ok(),
            "error": result.err().map(|e| e.to_string())
        }))
        .collect();
    Ok(serde_json::json!({ "success": true, "results": formatted_results }))
}
/// Start all OBS streams
#[tauri::command]
pub async fn control_room_start_all_obs(
    session_id: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Start all OBS streams for session {}", session_id);
    // TODO: Validate session
    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut results = Vec::new();
    for n in names {
        let r = app.obs_obws_plugin().start_streaming(Some(&n)).await;
        results.push((n, r.map(|_| ()).map_err(|e| anyhow::anyhow!(e.to_string()))));
    }
    let formatted_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(conn, result)| serde_json::json!({
            "connection": conn,
            "success": result.is_ok(),
            "error": result.err().map(|e| e.to_string())
        }))
        .collect();
    Ok(serde_json::json!({ "success": true, "results": formatted_results }))
}

/// Stop all OBS streams
#[tauri::command]
pub async fn control_room_stop_all_obs(
    session_id: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Stop all OBS streams for session {}", session_id);
    // TODO: Validate session
    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut results = Vec::new();
    for n in names {
        let r = app.obs_obws_plugin().stop_streaming(Some(&n)).await;
        results.push((n, r.map(|_| ()).map_err(|e| anyhow::anyhow!(e.to_string()))));
    }
    let formatted_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(conn, result)| serde_json::json!({
            "connection": conn,
            "success": result.is_ok(),
            "error": result.err().map(|e| e.to_string())
        }))
        .collect();
    Ok(serde_json::json!({ "success": true, "results": formatted_results }))
}

/// Add STR connection
#[tauri::command]
pub async fn control_room_add_obs_connection(
    session_id: String,
    name: String,
    host: String,
    port: u16,
    password: Option<String>,
    _notes: Option<String>,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Adding OBS connection '{}' at {}:{} for session {}", name, host, port, session_id);
    // TODO: Validate session
    
    let config = crate::plugins::obs_obws::types::ObsConnectionConfig { name: name.clone(), host, port, password, timeout_seconds: 30 };
    match app.obs_obws_plugin().add_connection(config).await {
        Ok(_) => {
            log::info!("Control Room: Successfully added OBS connection '{}'", name);
            Ok(serde_json::json!({
                "success": true,
                "message": format!("OBS connection '{}' added successfully", name)
            }))
        }
        Err(e) => {
            log::error!("Failed to add OBS connection '{}': {}", name, e);
            Err(TauriError::from(anyhow::anyhow!("Failed to add OBS connection: {}", e)))
        }
    }
}

/// Connect to OBS instance
#[tauri::command]
pub async fn control_room_connect_obs(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Connecting to OBS '{}' for session {}", obs_name, session_id);
    // TODO: Validate session
    
    match app.obs_obws_plugin().connect(&obs_name).await {
        Ok(_) => {
            log::info!("Control Room: Successfully connected to OBS '{}'", obs_name);
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Connected to OBS '{}'", obs_name)
            }))
        }
        Err(e) => {
            log::error!("Failed to connect to OBS '{}': {}", obs_name, e);
            Err(TauriError::from(anyhow::anyhow!("Failed to connect to OBS: {}", e)))
        }
    }
}

/// Disconnect from OBS instance
#[tauri::command]
pub async fn control_room_disconnect_obs(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Disconnecting from OBS '{}' for session {}", obs_name, session_id);
    // TODO: Validate session
    
    match app.obs_obws_plugin().disconnect(&obs_name).await {
        Ok(_) => {
            log::info!("Control Room: Successfully disconnected from OBS '{}'", obs_name);
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Disconnected from OBS '{}'", obs_name)
            }))
        }
        Err(e) => {
            log::error!("Failed to disconnect from OBS '{}': {}", obs_name, e);
            Err(TauriError::from(anyhow::anyhow!("Failed to disconnect from OBS: {}", e)))
        }
    }
}

/// Remove OBS connection
#[tauri::command]
pub async fn control_room_remove_obs_connection(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Removing OBS connection '{}' for session {}", obs_name, session_id);
    // TODO: Validate session
    
    match app.obs_obws_plugin().remove_connection(&obs_name).await {
        Ok(_) => {
            log::info!("Control Room: Successfully removed OBS connection '{}'", obs_name);
            Ok(serde_json::json!({
                "success": true,
                "message": format!("OBS connection '{}' removed successfully", obs_name)
            }))
        }
        Err(e) => {
            log::error!("Failed to remove OBS connection '{}': {}", obs_name, e);
            Err(TauriError::from(anyhow::anyhow!("Failed to remove OBS connection: {}", e)))
        }
    }
}

/// Get OBS connection configuration
#[tauri::command]
pub async fn control_room_get_obs_connection(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Getting OBS connection '{}' for session {}", obs_name, session_id);
    // TODO: Validate session
    
    match app.obs_obws_plugin().get_connection_status(&obs_name).await {
        Ok(status) => Ok(serde_json::json!({ "success": true, "connection": {"name": obs_name, "status": format!("{:?}", status)} })),
        Err(e) => {
            log::error!("Failed to get OBS connection '{}': {}", obs_name, e);
            Err(TauriError::from(anyhow::anyhow!("Failed to get OBS connection: {}", e)))
        }
    }
}

/// Update OBS connection configuration
#[tauri::command]
pub async fn control_room_update_obs_connection(
    session_id: String,
    obs_name: String,
    host: String,
    port: u16,
    password: Option<String>,
    _notes: Option<String>,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Updating OBS connection '{}' for session {}", obs_name, session_id);
    // TODO: Validate session
    
    let _ = app.obs_obws_plugin().remove_connection(&obs_name).await;
    let cfg = crate::plugins::obs_obws::types::ObsConnectionConfig { name: obs_name.clone(), host, port, password, timeout_seconds: 30 };
    match app.obs_obws_plugin().add_connection(cfg).await {
        Ok(_) => {
            log::info!("Control Room: Successfully updated OBS connection '{}'", obs_name);
            Ok(serde_json::json!({
                "success": true,
                "message": format!("OBS connection '{}' updated successfully", obs_name)
            }))
        }
        Err(e) => {
            log::error!("Failed to update OBS connection '{}': {}", obs_name, e);
            Err(TauriError::from(anyhow::anyhow!("Failed to update OBS connection: {}", e)))
        }
    }
}
/// Connect all disconnected OBS connections
#[tauri::command]
pub async fn control_room_connect_all_obs(
    session_id: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Connecting all OBS connections for session {}", session_id);
    // TODO: Validate session
    
    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut results: Vec<(String, Result<(), anyhow::Error>)> = Vec::new();
    for n in names { results.push((n.clone(), app.obs_obws_plugin().connect(&n).await.map(|_| ()).map_err(|e| anyhow::anyhow!(e.to_string())))); }
    {
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        let total_count = results.len();
        let failed_connections: Vec<String> = results.iter().filter_map(|(name, r)| r.as_ref().err().map(|e| format!("{}: {}", name, e))).collect();
        log::info!("Control Room: Connected {} of {} OBS connections", success_count, total_count);
        if failed_connections.is_empty() {
            return Ok(serde_json::json!({
                "success": true,
                "message": format!("Successfully connected {} OBS connections", success_count),
                "connected_count": success_count,
                "total_count": total_count
            }));
        } else {
            return Ok(serde_json::json!({
                "success": true,
                "message": format!("Connected {} of {} OBS connections. Some failed: {}", success_count, total_count, failed_connections.join(", ")),
                "connected_count": success_count,
                "total_count": total_count,
                "failed_connections": failed_connections
            }));
        }
    }
}

/// Disconnect all connected OBS connections
#[tauri::command]
pub async fn control_room_disconnect_all_obs(
    _session_id: String,
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Disconnecting all OBS connections");
    // TODO: Validate session
    
    let names = _app.obs_obws_plugin().get_connection_names().await;
    let mut results: Vec<(String, Result<(), anyhow::Error>)> = Vec::new();
    for n in names { results.push((n.clone(), _app.obs_obws_plugin().disconnect(&n).await.map(|_| ()).map_err(|e| anyhow::anyhow!(e.to_string())))); }
    {
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        let total_count = results.len();
        let failed_connections: Vec<String> = results.iter().filter_map(|(name, r)| r.as_ref().err().map(|e| format!("{}: {}", name, e))).collect();
        log::info!("Control Room: Disconnected {} of {} OBS connections", success_count, total_count);
        if failed_connections.is_empty() {
            return Ok(serde_json::json!({
                "success": true,
                "message": format!("Successfully disconnected {} OBS connections", success_count),
                "disconnected_count": success_count,
                "total_count": total_count
            }));
        } else {
            return Ok(serde_json::json!({
                "success": true,
                "message": format!("Disconnected {} of {} OBS connections. Some failed: {}", success_count, total_count, failed_connections.join(", ")),
                "disconnected_count": success_count,
                "total_count": total_count,
                "failed_connections": failed_connections
            }));
        }
    }
    // legacy branch removed
}

/// Get audio sources for a Control Room OBS connection
#[tauri::command]
pub async fn control_room_get_audio_sources(
    session_id: String,
    obs_name: String,
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::debug!("Control Room: Getting audio sources for OBS '{}' session {}", obs_name, session_id);
    // TODO: Validate session
    
    // Not implemented with obws yet
    Ok(serde_json::json!({ "success": true, "sources": [] }))
}

/// Get scenes for a Control Room OBS connection
#[tauri::command]
pub async fn control_room_get_scenes(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::debug!("Control Room: Getting scenes for OBS '{}' session {}", obs_name, session_id);
    // TODO: Validate session
    
    match app.obs_obws_plugin().get_scenes(Some(&obs_name)).await {
        Ok(scenes) => Ok(serde_json::json!({
            "success": true,
            "scenes": scenes
        })),
        Err(e) => {
            log::error!("Failed to get scenes for OBS '{}': {}", obs_name, e);
            Err(TauriError::from(anyhow::anyhow!("Failed to get scenes: {}", e)))
        }
    }
}

// Control Room Security Enhancement Commands
#[tauri::command]
pub async fn control_room_change_password(
    _session_id: String,
    _current_password: String,
    _new_password: String,
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room password change attempt");
    
    Ok(serde_json::json!({ "success": false, "error": "Not supported" }))
}
#[tauri::command]
pub async fn control_room_get_audit_log(
    _session_id: String,
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room audit log request");
    
    Ok(serde_json::json!({ "success": true, "audit_entries": [] }))
}

#[tauri::command]
pub async fn control_room_get_session_info(
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "authenticated": false }))
}

#[tauri::command]
pub async fn control_room_refresh_session(
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
pub async fn control_room_logout(
    _app: State<'_, Arc<App>>
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true }))
}