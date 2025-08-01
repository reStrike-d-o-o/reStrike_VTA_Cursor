use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, Emitter, Error as TauriError};
use crate::core::app::App;
use crate::logging::archival::{AutoArchiveConfig, ArchiveSchedule};
use dirs;



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
    }
    
    Ok(())
}

// OBS commands - Fixed names to match frontend expectations
#[tauri::command]
pub async fn obs_connect(url: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
) -> Result<serde_json::Value, TauriError> {
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
) -> Result<serde_json::Value, TauriError> {
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
) -> Result<serde_json::Value, TauriError> {
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
pub async fn obs_get_connections(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
pub async fn obs_disconnect(connection_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS disconnect called for connection: {}", connection_name);
    app.obs_plugin().disconnect_obs(&connection_name).await.map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    Ok(serde_json::json!({
        "success": true,
        "message": "OBS disconnection initiated"
    }))
}

#[tauri::command]
pub async fn obs_remove_connection(connection_name: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS remove connection called for connection: {}", connection_name);
    
    // Remove from OBS plugin
    app.obs_plugin().remove_connection(&connection_name).await.map_err(|e| TauriError::from(anyhow::anyhow!("{}", e)))?;
    
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
pub async fn obs_get_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
pub async fn obs_start_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
pub async fn obs_stop_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
        match event {
            crate::plugins::plugin_udp::PssEvent::Points { athlete, point_type } => {
                serde_json::json!({
                    "type": "points",
                    "athlete": athlete,
                    "point_type": point_type,
                    "description": format!("Athlete {} scored {} points", athlete, point_type)
                })
            }
            crate::plugins::plugin_udp::PssEvent::HitLevel { athlete, level } => {
                serde_json::json!({
                    "type": "hit_level",
                    "athlete": athlete,
                    "level": level,
                    "description": format!("Athlete {} hit level {}", athlete, level)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                serde_json::json!({
                    "type": "warnings",
                    "athlete1_warnings": athlete1_warnings,
                    "athlete2_warnings": athlete2_warnings,
                    "description": format!("Warnings - Athlete1: {}, Athlete2: {}", athlete1_warnings, athlete2_warnings)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Clock { time, action } => {
                serde_json::json!({
                    "type": "clock",
                    "time": time,
                    "action": action,
                    "description": format!("Clock: {} {:?}", time, action.as_ref().unwrap_or(&String::new()))
                })
            }
            crate::plugins::plugin_udp::PssEvent::Round { current_round } => {
                serde_json::json!({
                    "type": "round",
                    "current_round": current_round,
                    "description": format!("Round {}", current_round)
                })
            }
            crate::plugins::plugin_udp::PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                serde_json::json!({
                    "type": "winner_rounds",
                    "round1_winner": round1_winner,
                    "round2_winner": round2_winner,
                    "round3_winner": round3_winner,
                    "description": format!("Winner Rounds - R1: {}, R2: {}, R3: {}", round1_winner, round2_winner, round3_winner)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => {
                serde_json::json!({
                    "type": "scores",
                    "athlete1_r1": athlete1_r1,
                    "athlete2_r1": athlete2_r1,
                    "athlete1_r2": athlete1_r2,
                    "athlete2_r2": athlete2_r2,
                    "athlete1_r3": athlete1_r3,
                    "athlete2_r3": athlete2_r3,
                    "description": format!("Scores - A1: R1={}, R2={}, R3={} | A2: R1={}, R2={}, R3={}", 
                        athlete1_r1, athlete1_r2, athlete1_r3, athlete2_r1, athlete2_r2, athlete2_r3)
                })
            }
            crate::plugins::plugin_udp::PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                serde_json::json!({
                    "type": "current_scores",
                    "athlete1_score": athlete1_score,
                    "athlete2_score": athlete2_score,
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
        }
    }).collect();
    
    Ok(event_json)
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
            let log_manager = app.log_manager().clone();
            
            // Spawn a background task to monitor OBS events
            tokio::spawn(async move {
                loop {
                    // Simulate OBS events for now
                    // In a real implementation, this would listen to actual OBS WebSocket events
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    
                    let event_data = format!("[{}] OBS Event: Scene changed to 'Main Scene'", chrono::Utc::now().format("%H:%M:%S"));
                    
                    // Log to OBS subsystem file
                    {
                        let log_manager_guard = log_manager.lock().await;
                        if let Err(e) = log_manager_guard.log(&subsystem_clone, "INFO", &event_data) {
                            log::error!("Failed to log OBS event: {}", e);
                        }
                    }
                    
                    if let Err(e) = app_handle_clone.emit("live_data", serde_json::json!({
                        "subsystem": subsystem_clone,
                        "data": event_data,
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
            let log_manager = app.log_manager().clone();
            
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    
                    let event_data = format!("[{}] PSS Event: Match data received", chrono::Utc::now().format("%H:%M:%S"));
                    
                    // Log to PSS subsystem file
                    {
                        let log_manager_guard = log_manager.lock().await;
                        if let Err(e) = log_manager_guard.log(&subsystem_clone, "INFO", &event_data) {
                            log::error!("Failed to log PSS event: {}", e);
                        }
                    }
                    
                    if let Err(e) = app_handle_clone.emit("live_data", serde_json::json!({
                        "subsystem": subsystem_clone,
                        "data": event_data,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })) {
                        log::error!("Failed to emit PSS live data event: {}", e);
                        break;
                    }
                }
            });
        }
        
        // For UDP subsystem we rely on real-time push from core::App handle_udp_events; no simulated loop here.
        
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
pub async fn get_live_data(subsystem: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
pub async fn obs_toggle_full_events(enabled: bool, app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Toggling OBS full events display: {}", enabled);
    
    app.obs_plugin().toggle_full_events(enabled).await;
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Full OBS events display {}", if enabled { "enabled" } else { "disabled" })
    }))
}

#[tauri::command]
pub async fn obs_get_full_events_setting(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting OBS full events setting");
    
    let enabled = app.obs_plugin().get_full_events_setting().await;
    
    Ok(serde_json::json!({
        "success": true,
        "enabled": enabled
    }))
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
pub async fn obs_get_recent_events(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
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
pub async fn cpu_get_process_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    // println!("ðŸš¨ [CPU_CMD] ===== CPU GET PROCESS DATA CALLED =====");
    log::info!("[CPU_CMD] ===== CPU GET PROCESS DATA CALLED =====");
    
    // println!("ðŸš¨ [CPU_CMD] Triggering immediate data collection...");
    log::info!("[CPU_CMD] Triggering immediate data collection...");
    
    match app.cpu_monitor_plugin().update_cpu_data().await {
        Ok(_) => {
            // println!("ðŸš¨ [CPU_CMD] Data collection successful");
            log::info!("[CPU_CMD] Data collection successful");
        },
        Err(e) => {
            // println!("ðŸš¨ [CPU_CMD] Failed to update CPU data: {}", e);
            log::error!("[CPU_CMD] Failed to update CPU data: {}", e);
        },
    }
    
    let process_data = app.cpu_monitor_plugin().get_process_cpu_data().await;
    
    // println!("ðŸš¨ [CPU_CMD] Process data count: {}", process_data.len());
    log::info!("[CPU_CMD] Process data count: {}", process_data.len());
    
    // Log first few processes for debugging
    for (i, process) in process_data.iter().take(3).enumerate() {
        // println!("ðŸš¨ [CPU_CMD] Process {}: {} - CPU: {:.1}%, Memory: {:.1}MB", 
        //     i, process.process_name, process.cpu_percent, process.memory_mb);
        log::debug!("[CPU_CMD] Process {}: {} - CPU: {:.1}%, Memory: {:.1}MB", 
            i, process.process_name, process.cpu_percent, process.memory_mb);
    }
    
    // println!("ðŸš¨ [CPU_CMD] Returning result with {} processes", process_data.len());
    log::info!("[CPU_CMD] Returning result with {} processes", process_data.len());
    
    Ok(serde_json::json!({
        "success": true,
        "processes": process_data
    }))
}

#[tauri::command]
pub async fn cpu_get_system_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    // println!("ðŸš¨ [CPU_CMD] ===== CPU GET SYSTEM DATA CALLED =====");
    log::info!("[CPU_CMD] ===== CPU GET SYSTEM DATA CALLED =====");
    
    // Trigger immediate data collection
    // println!("ðŸš¨ [CPU_CMD] Triggering immediate data collection...");
    log::info!("[CPU_CMD] Triggering immediate data collection...");
    match app.cpu_monitor_plugin().update_cpu_data().await {
        Ok(_) => {
            // println!("ðŸš¨ [CPU_CMD] Data collection successful");
            log::info!("[CPU_CMD] Data collection successful");
        },
        Err(e) => {
            // println!("ðŸš¨ [CPU_CMD] Failed to update CPU data: {}", e);
            log::error!("[CPU_CMD] Failed to update CPU data: {}", e);
        },
    }
    
    let system_data = app.cpu_monitor_plugin().get_system_cpu_data().await;
    // println!("ðŸš¨ [CPU_CMD] System data available: {}", system_data.is_some());
    log::info!("[CPU_CMD] System data available: {}", system_data.is_some());
    
    let result = serde_json::json!({
        "success": true,
        "system": system_data
    });
    
    // println!("ðŸš¨ [CPU_CMD] Returning system data");
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
    log::info!("ðŸ§ª Emitting PSS event via hybrid approach: {:?}", event_data);
    
    // HYBRID APPROACH: Real-time emission to both systems
    // 1. Emit to Tauri frontend (React components) - Real-time
    if let Err(e) = window.emit("pss_event", event_data.clone()) {
        log::error!("âŒ Failed to emit PSS event to Tauri frontend: {}", e);
        return Err(TauriError::from(anyhow::anyhow!("{}", e)));
    }
    
    // 2. Broadcast to WebSocket overlays (HTML overlays) - Real-time
    crate::core::app::App::emit_pss_event(event_data);
    
    log::info!("âœ… Successfully emitted PSS event via hybrid approach");
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
        let event_json = match event {
            crate::plugins::plugin_udp::PssEvent::Points { athlete, point_type } => {
                serde_json::json!({
                    "type": "points",
                    "athlete": athlete,
                    "point_type": point_type,
                    "description": format!("Athlete {} scored {} points", athlete, point_type)
                })
            }
            crate::plugins::plugin_udp::PssEvent::HitLevel { athlete, level } => {
                serde_json::json!({
                    "type": "hit_level",
                    "athlete": athlete,
                    "level": level,
                    "description": format!("Athlete {} hit level {}", athlete, level)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                serde_json::json!({
                    "type": "warnings",
                    "athlete1_warnings": athlete1_warnings,
                    "athlete2_warnings": athlete2_warnings,
                    "description": format!("Warnings - Athlete1: {}, Athlete2: {}", athlete1_warnings, athlete2_warnings)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Clock { time, action } => {
                serde_json::json!({
                    "type": "clock",
                    "time": time,
                    "action": action,
                    "description": format!("Clock: {} {:?}", time, action.as_ref().unwrap_or(&String::new()))
                })
            }
            crate::plugins::plugin_udp::PssEvent::Round { current_round } => {
                serde_json::json!({
                    "type": "round",
                    "current_round": current_round,
                    "description": format!("Round {}", current_round)
                })
            }
            crate::plugins::plugin_udp::PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                serde_json::json!({
                    "type": "winner_rounds",
                    "round1_winner": round1_winner,
                    "round2_winner": round2_winner,
                    "round3_winner": round3_winner,
                    "description": format!("Winner Rounds - R1: {}, R2: {}, R3: {}", round1_winner, round2_winner, round3_winner)
                })
            }
            crate::plugins::plugin_udp::PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => {
                serde_json::json!({
                    "type": "scores",
                    "athlete1_r1": athlete1_r1,
                    "athlete2_r1": athlete2_r1,
                    "athlete1_r2": athlete1_r2,
                    "athlete2_r2": athlete2_r2,
                    "athlete1_r3": athlete1_r3,
                    "athlete2_r3": athlete2_r3,
                    "description": format!("Scores - A1: R1={}, R2={}, R3={} | A2: R1={}, R2={}, R3={}", 
                        athlete1_r1, athlete1_r2, athlete1_r3, athlete2_r1, athlete2_r2, athlete2_r3)
                })
            }
            crate::plugins::plugin_udp::PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                serde_json::json!({
                    "type": "current_scores",
                    "athlete1_score": athlete1_score,
                    "athlete2_score": athlete2_score,
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
    log::info!("âœ… PSS event listener setup complete (using original mechanism)");
    
    Ok(())
} 

#[tauri::command]
pub async fn obs_setup_status_listener(window: tauri::Window, app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Setting up OBS status listener for frontend");

    let window_clone = window.clone();
    let app_arc = app.inner().clone();
    // Spawn background task (using cloned Arc<App>)
    tokio::spawn(async move {
        let mut last_payload = serde_json::Value::Null;
        loop {
            // Fetch current status
            let status_result = app_arc.obs_plugin().get_obs_status().await;
            if let Ok(status) = status_result {
                let payload = serde_json::json!({
                    "is_recording": status.is_recording,
                    "is_streaming": status.is_streaming,
                    "cpu_usage": status.cpu_usage,
                    "recording_connection": status.recording_connection,
                    "streaming_connection": status.streaming_connection,
                });
                // Emit only if changed
                if payload != last_payload {
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
        Ok(connected) => {
            if connected {
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
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "connected": false,
                    "message": "Not connected to Google Drive"
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
        Ok(connected) => {
            if !connected {
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
    
    // Path to the flags directory (relative to project root)
    let flags_dir = std::path::Path::new("../ui/public/assets/flags");
    
    if !flags_dir.exists() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Flags directory does not exist: ../ui/public/assets/flags"
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
        
        // Only process PNG files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("png") {
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
    let status = websocket_plugin.get_status().await;
    
    Ok(status)
}

#[tauri::command]
pub async fn websocket_broadcast_pss_event(
    event_data: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Broadcasting PSS event via WebSocket: {:?}", event_data);
    
    let websocket_plugin = app.websocket_plugin().lock().await;
    match websocket_plugin.broadcast_pss_event(event_data).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "PSS event broadcasted successfully"
        })),
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


