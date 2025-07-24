use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, Emitter};
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
    let config = app.config_manager().get_config().await;
    app.udp_plugin().start(&config).await.map_err(|e| e.to_string())?;
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
    let status_str = match status {
        crate::plugins::plugin_udp::UdpServerStatus::Stopped => "Stopped",
        crate::plugins::plugin_udp::UdpServerStatus::Starting => "Starting",
        crate::plugins::plugin_udp::UdpServerStatus::Running => "Running",
        crate::plugins::plugin_udp::UdpServerStatus::Error(e) => return Err(e),
    };
    Ok(status_str.to_string())
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
            let config = app.config_manager().get_config().await;
        match app.udp_plugin().start(&config).await {
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
pub async fn pss_get_events(app: State<'_, Arc<App>>) -> Result<Vec<serde_json::Value>, String> {
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
                    "description": format!("Clock: {} {:?}", time, action.unwrap_or_default())
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
    
    // For Tauri v2, we need to use the dialog plugin
    // The dialog plugin is configured in tauri.conf.json with "open": true
    // We'll use a simple approach that works for now
    let default_backup = "config/app_config.backup.json";
    
    // Check if the default backup exists
    if std::path::Path::new(default_backup).exists() {
        log::info!("Using default backup file: {}", default_backup);
        Ok(vec![default_backup.to_string()])
    } else {
        log::info!("No default backup found, returning empty selection");
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn restore_backup_with_dialog(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Restore backup with dialog called");
    
    // For Tauri v2, we need to implement the native file dialog
    // Since the dialog plugin is configured, we can use it
    // For now, let's use a simple approach that works
    let backup_path = "config/app_config.backup.json";
    
    // Check if the backup file exists
    let path = std::path::Path::new(backup_path);
    if !path.exists() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "No backup file found. Please ensure app_config.backup.json exists in the config directory."
        }));
    }
    
    // Restore from the backup file
    match app.config_manager().import_config(path).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Successfully restored from backup: {}", backup_path)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to restore from backup: {}", e)
        }))
    }
}

// Diagnostics & Logs commands

#[tauri::command]
pub async fn list_log_files(
    subsystem: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
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
) -> Result<Vec<u8>, String> {
    log::info!("Downloading log file: {}", filename);
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.read_log_file(&filename) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(format!("Failed to read log file: {}", e))
    }
}

#[tauri::command]
pub async fn list_archives(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
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
) -> Result<serde_json::Value, String> {
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
) -> Result<Vec<u8>, String> {
    log::info!("Downloading archive: {}", archive_name);
    
    let log_manager = app.log_manager().lock().await;
    match log_manager.download_archive(&archive_name) {
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
pub async fn cpu_get_system_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
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

// Test command to verify CPU monitor plugin works
#[tauri::command]
pub async fn cpu_test_plugin(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    // println!("🚨 [CPU_TEST] ===== CPU TEST PLUGIN CALLED =====");
    log::info!("[CPU_TEST] ===== CPU TEST PLUGIN CALLED =====");
    
    // println!("🚨 [CPU_TEST] CPU monitor plugin accessed successfully");
    log::info!("[CPU_TEST] CPU monitor plugin accessed successfully");
    
    // Trigger immediate data collection
    // println!("🚨 [CPU_TEST] Triggering immediate data collection...");
    log::info!("[CPU_TEST] Triggering immediate data collection...");
    match app.cpu_monitor_plugin().update_cpu_data().await {
        Ok(_) => {
            // println!("🚨 [CPU_TEST] Data collection successful");
            log::info!("[CPU_TEST] Data collection successful");
        },
        Err(e) => {
            // println!("🚨 [CPU_TEST] Data collection failed: {}", e);
            log::error!("[CPU_TEST] Data collection failed: {}", e);
        },
    }
    
    let process_data = app.cpu_monitor_plugin().get_process_cpu_data().await;
    // println!("🚨 [CPU_TEST] Process data count: {}", process_data.len());
    log::info!("[CPU_TEST] Process data count: {}", process_data.len());
    
    let system_data = app.cpu_monitor_plugin().get_system_cpu_data().await;
    // println!("🚨 [CPU_TEST] System data available: {}", system_data.is_some());
    log::info!("[CPU_TEST] System data available: {}", system_data.is_some());
    
    // Log first few processes for debugging
    for (i, process) in process_data.iter().take(3).enumerate() {
        // println!("🚨 [CPU_TEST] Process {}: {} - CPU: {:.1}%, Memory: {:.1}MB", 
        //     i, process.process_name, process.cpu_percent, process.memory_mb);
        log::debug!("[CPU_TEST] Process {}: {} - CPU: {:.1}%, Memory: {:.1}MB", 
            i, process.process_name, process.cpu_percent, process.memory_mb);
    }
    
    let result = serde_json::json!({
        "success": true,
        "process_count": process_data.len(),
        "system_available": system_data.is_some(),
        "test_completed": true
    });
    
    // println!("🚨 [CPU_TEST] Test completed successfully");
    log::info!("[CPU_TEST] Test completed successfully");
    Ok(result)
} 

#[tauri::command]
pub async fn cpu_enable_monitoring(app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("[CPU_CMD] ===== ENABLE CPU MONITORING CALLED =====");
    
    match app.cpu_monitor_plugin().enable_monitoring().await {
        Ok(_) => {
            log::info!("[CPU_CMD] CPU monitoring enabled successfully");
            Ok(())
        },
        Err(e) => {
            log::error!("[CPU_CMD] Failed to enable CPU monitoring: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn cpu_disable_monitoring(app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("[CPU_CMD] ===== DISABLE CPU MONITORING CALLED =====");
    
    match app.cpu_monitor_plugin().disable_monitoring().await {
        Ok(_) => {
            log::info!("[CPU_CMD] CPU monitoring disabled successfully");
            Ok(())
        },
        Err(e) => {
            log::error!("[CPU_CMD] Failed to disable CPU monitoring: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn cpu_get_monitoring_status(app: State<'_, Arc<App>>) -> Result<bool, String> {
    log::info!("[CPU_CMD] ===== GET CPU MONITORING STATUS CALLED =====");
    
    match app.cpu_monitor_plugin().is_monitoring_enabled().await {
        Ok(enabled) => {
            log::info!("[CPU_CMD] CPU monitoring status: {}", enabled);
            Ok(enabled)
        },
        Err(e) => {
            log::error!("[CPU_CMD] Failed to get CPU monitoring status: {}", e);
            Err(e.to_string())
        }
    }
}

// Protocol Management Commands
#[tauri::command]
pub async fn protocol_get_versions(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    let log_manager = app.log_manager().lock().await;
    if let Err(e) = log_manager.log("pss", "INFO", "Getting protocol versions") {
        log::error!("Failed to log protocol get versions: {}", e);
    }
    
    let versions = app.protocol_manager().get_versions().await.map_err(|e| e.to_string())?;
    let current_protocol = app.protocol_manager().get_current_protocol().await.map_err(|e| e.to_string())?;
    
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
) -> Result<serde_json::Value, String> {
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
) -> Result<serde_json::Value, String> {
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
) -> Result<serde_json::Value, String> {
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
) -> Result<Vec<u8>, String> {
    let log_manager = app.log_manager().lock().await;
    if let Err(e) = log_manager.log("pss", "INFO", &format!("Exporting protocol file: {}", version)) {
        log::error!("Failed to log protocol export: {}", e);
    }
    
    app.protocol_manager().export_protocol_file(&version).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn protocol_get_current(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
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
pub async fn get_network_interfaces() -> Result<serde_json::Value, String> {
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
pub async fn get_best_network_interface() -> Result<serde_json::Value, String> {
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

// PSS Event Emission Command
#[tauri::command]
pub async fn pss_emit_event(event_data: serde_json::Value, window: tauri::Window) -> Result<(), String> {
    log::info!("Emitting PSS event to frontend: {:?}", event_data);
    if let Err(e) = window.emit("pss_event", event_data) {
        log::error!("Failed to emit PSS event: {}", e);
        return Err(e.to_string());
    }
    Ok(())
}

// Get and emit PSS events to frontend
#[tauri::command]
pub async fn pss_emit_pending_events(window: tauri::Window, app: State<'_, Arc<App>>) -> Result<(), String> {
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
                    "description": format!("Clock: {} {:?}", time, action.unwrap_or_default())
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
            return Err(e.to_string());
        }
    }
    
    Ok(())
} 

// Set up PSS event listener that emits events to frontend
#[tauri::command]
pub async fn pss_setup_event_listener(window: tauri::Window) -> Result<(), String> {
    log::info!("Setting up PSS event listener for frontend");
    
    // Get a receiver for PSS events
    if let Some(mut receiver) = crate::core::app::App::subscribe_to_pss_events() {
        // Spawn a task to listen for events and emit them to the frontend
        tokio::spawn(async move {
            while let Ok(event_data) = receiver.recv().await {
                log::info!("📡 Emitting PSS event to frontend: {:?}", event_data);
                if let Err(e) = window.emit("pss_event", event_data) {
                    log::error!("Failed to emit PSS event to frontend: {}", e);
                }
            }
        });
        log::info!("✅ PSS event listener setup complete");
    } else {
        log::warn!("⚠️ PSS event broadcaster not initialized");
        return Err("PSS event broadcaster not initialized".to_string());
    }
    
    Ok(())
} 

#[tauri::command]
pub async fn obs_setup_status_listener(window: tauri::Window, app: State<'_, Arc<App>>) -> Result<(), String> {
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
pub async fn cpu_setup_stats_listener(window: tauri::Window, app: State<'_, Arc<App>>) -> Result<(), String> {
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
pub async fn set_window_fullscreen(window: tauri::Window) -> Result<(), String> {
    log::info!("Setting window to fullscreen");
    window.set_fullscreen(true).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_window_compact(width: Option<f64>, height: Option<f64>, window: tauri::Window) -> Result<(), String> {
    let default_width = 350.0;
    let default_height = 1080.0;
    
    log::info!("Setting window to compact mode: {}x{}", width.unwrap_or(default_width), height.unwrap_or(default_height));
    window.set_fullscreen(false).map_err(|e| e.to_string())?;
    window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(
        width.unwrap_or(default_width), 
        height.unwrap_or(default_height)
    ))).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_window_custom_size(width: f64, height: f64, window: tauri::Window) -> Result<(), String> {
    log::info!("Setting window to custom size: {}x{}", width, height);
    window.set_fullscreen(false).map_err(|e| e.to_string())?;
    window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_window_position(x: f64, y: f64, window: tauri::Window) -> Result<(), String> {
    log::info!("Setting window position: x={}, y={}", x, y);
    window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_window_startup_position(window: tauri::Window) -> Result<(), String> {
    log::info!("Setting window to startup position: x=1, y=1");
    
    // Set window to compact mode (350x1080)
    window.set_fullscreen(false).map_err(|e| e.to_string())?;
    window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(350.0, 1080.0)))
        .map_err(|e| e.to_string())?;
    
    // Set position to x=1, y=1
    window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(1.0, 1.0)))
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn save_window_settings(settings: serde_json::Value, _app: State<'_, Arc<App>>) -> Result<(), String> {
    log::info!("Saving window settings: {:?}", settings);
    
    // For now, just log the settings - we'll implement proper persistence later
    log::info!("Window settings would be saved: {:?}", settings);
    
    Ok(())
}

#[tauri::command]
pub async fn load_window_settings(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
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
pub async fn get_screen_size() -> Result<serde_json::Value, String> {
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
pub async fn initialize_ui_settings_database() -> Result<serde_json::Value, String> {
    log::info!("Initializing UI settings database");
    
    match crate::ui_settings::UiSettingsManager::initialize() {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "UI settings database initialized successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e
        }))
    }
}

// Database Plugin Commands
#[tauri::command]
pub async fn db_initialize_ui_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
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
pub async fn db_get_ui_setting(key: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
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
) -> Result<serde_json::Value, String> {
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
pub async fn db_get_all_ui_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
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
pub async fn db_get_database_info(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Getting database information");
    
    let is_accessible = app.database_plugin().is_accessible().await;
    let file_size = app.database_plugin().get_file_size();
    
    let file_size_value = match file_size {
        Ok(size) => serde_json::Value::Number(serde_json::Number::from(size)),
        Err(_) => serde_json::Value::Null,
    };
    
    Ok(serde_json::json!({
        "success": true,
        "is_accessible": is_accessible,
        "file_size": file_size_value
    }))
}

// Database Migration Commands
#[tauri::command]
pub async fn migrate_json_to_database(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
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
pub async fn create_json_backup(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
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
) -> Result<serde_json::Value, String> {
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
pub async fn get_migration_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Getting migration status");
    
    match app.database_plugin().get_migration_status().await {
        Ok(status) => Ok(serde_json::json!({
            "success": true,
            "status": {
                "database_enabled": status.database_enabled,
                "json_fallback_enabled": status.json_fallback_enabled,
                "migration_completed": status.migration_completed,
                "last_migration": status.last_migration,
                "settings_count": status.settings_count
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

#[tauri::command]
pub async fn enable_database_mode(
    app: State<'_, Arc<App>>,
    enabled: bool
) -> Result<serde_json::Value, String> {
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

// Google Drive commands
#[tauri::command]
pub async fn drive_request_auth_url() -> Result<String, String> {
    let (url, _csrf_token) = crate::plugins::drive_plugin()
        .auth_url()
        .await
        .map_err(|e| e.to_string())?;
    Ok(url)
}

#[tauri::command]
pub async fn drive_complete_auth(code: String) -> Result<(), String> {
    crate::plugins::drive_plugin()
        .exchange_code(code)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn drive_save_credentials(id: String, secret: String) -> Result<(), String> {
    crate::plugins::drive_plugin()
        .save_credentials(id, secret)
        .await
        .map_err(|e| e.to_string())
}