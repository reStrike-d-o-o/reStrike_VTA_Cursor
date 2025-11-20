//! Tauri commands for OBS WebSocket integration using obws crate

use crate::core::app::App;
use crate::plugins::obs_obws::ObsConnectionConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Error as TauriError, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsObwsConnectionRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsObwsConnectionResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

// ============================================================================
// IVR Replay Settings and Actions
// ============================================================================

#[tauri::command]
pub async fn ivr_get_replay_settings(
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    let settings = match app.database_plugin().get_all_ui_settings().await {
        Ok(map) => map,
        Err(err) => {
            log::warn!(
                "ivr_get_replay_settings: failed to load UI settings; returning defaults: {err}"
            );
            std::collections::HashMap::<String, String>::new()
        }
    };
    let mpv_path = settings.get("ivr.replay.mpv_path").cloned();
    let seconds_from_end = settings
        .get("ivr.replay.seconds_from_end")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(10);
    let max_wait_ms = settings
        .get("ivr.replay.max_wait_ms")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(500);
    let auto_on_challenge = settings
        .get("ivr.replay.auto_on_challenge")
        .map(|s| s == "true")
        .unwrap_or(false);
    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
          "mpv_path": mpv_path,
          "seconds_from_end": seconds_from_end,
          "max_wait_ms": max_wait_ms,
          "auto_on_challenge": auto_on_challenge
        })),
        error: None,
    })
}

#[tauri::command]
pub async fn ivr_save_replay_settings(
    mpv_path: Option<String>,
    seconds_from_end: u32,
    max_wait_ms: u32,
    auto_on_challenge: bool,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    use crate::database::seaorm_ops::ui_settings as sea_ui_settings;

    let sea = app.database_plugin().seaorm();
    let ensure_specs = [
        ("ivr.replay.mpv_path", "MPV Path", "string", None),
        (
            "ivr.replay.seconds_from_end",
            "IVR Seconds From End",
            "integer",
            Some("10"),
        ),
        (
            "ivr.replay.max_wait_ms",
            "IVR Max Wait (ms)",
            "integer",
            Some("500"),
        ),
        (
            "ivr.replay.auto_on_challenge",
            "IVR Auto on Challenge",
            "boolean",
            Some("false"),
        ),
    ];

    for (key, display, data_type, default) in ensure_specs {
        if let Err(err) = sea_ui_settings::ensure_key(&sea, key, display, data_type, default).await
        {
            log::warn!("Failed to ensure UI setting '{key}': {err}");
        }
    }

    let secs = seconds_from_end.min(20);
    let wait = max_wait_ms.clamp(50, 500);

    if let Some(path) = mpv_path {
        if let Err(err) = app
            .database_plugin()
            .set_ui_setting(
                "ivr.replay.mpv_path",
                &path,
                "user",
                Some("update mpv path"),
            )
            .await
        {
            log::warn!("Failed to persist ivr.replay.mpv_path: {err}");
        }
    }

    if let Err(err) = app
        .database_plugin()
        .set_ui_setting(
            "ivr.replay.seconds_from_end",
            &secs.to_string(),
            "user",
            Some("update ivr seconds"),
        )
        .await
    {
        log::warn!("Failed to persist ivr.replay.seconds_from_end: {err}");
    }

    if let Err(err) = app
        .database_plugin()
        .set_ui_setting(
            "ivr.replay.max_wait_ms",
            &wait.to_string(),
            "user",
            Some("update ivr wait"),
        )
        .await
    {
        log::warn!("Failed to persist ivr.replay.max_wait_ms: {err}");
    }

    if let Err(err) = app
        .database_plugin()
        .set_ui_setting(
            "ivr.replay.auto_on_challenge",
            if auto_on_challenge { "true" } else { "false" },
            "user",
            Some("update ivr auto"),
        )
        .await
    {
        log::warn!("Failed to persist ivr.replay.auto_on_challenge: {err}");
    }
    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({"message":"IVR replay settings saved"})),
        error: None,
    })
}

#[tauri::command]
pub async fn ivr_round_replay_now(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    let conn_name = connection_name.unwrap_or_else(|| "OBS_REC".to_string());
    match app.replay_round_now(Some(&conn_name)).await {
        Ok(()) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({"launched":true})),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Open recorded video at the exact time of the specified event
#[tauri::command]
pub async fn ivr_open_event_video(
    event_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    match app.open_event_video(event_id).await {
        Ok(()) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({"opened": true})),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Validate mpv.exe path exists and is a file
#[tauri::command]
pub async fn ivr_validate_mpv_path(
    mpv_path: String,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    use std::path::Path;
    let p = Path::new(&mpv_path);
    if p.exists() && p.is_file() {
        Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({"valid": true})),
            error: None,
        })
    } else {
        Ok(ObsObwsConnectionResponse {
            success: false,
            data: Some(serde_json::json!({"valid": false})),
            error: Some("Path does not exist or is not a file".to_string()),
        })
    }
}

/// Snapshot of recent matches with recorded videos for IVR history panel
#[tauri::command]
pub async fn ivr_match_history_snapshot(
    limit: Option<u32>,
    date: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    use crate::database::seaorm_ops::pss_catalog;

    let sea = app.database_plugin().seaorm();
    let limit = limit.unwrap_or(40).min(200) as i64;
    let selected_date = date
        .as_ref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive())
        .format("%Y-%m-%d")
        .to_string();

    let history_entries = pss_catalog::get_match_history_with_videos(&sea, &selected_date, limit)
        .await
        .map_err(|e| {
            TauriError::from(anyhow::anyhow!(format!(
                "Failed to load match history: {e}"
            )))
        })?;

    let mut matches_json = Vec::with_capacity(history_entries.len());
    for entry in history_entries {
        let match_db_id = entry.match_row.id;
        let athletes_raw = match app
            .database_plugin()
            .get_pss_match_athletes(match_db_id)
            .await
        {
            Ok(list) => list,
            Err(e) => {
                log::warn!("Failed to load match athletes for {match_db_id}: {e}");
                Vec::new()
            }
        };

        let athletes = athletes_raw
            .into_iter()
            .map(|(match_athlete, athlete)| {
                serde_json::json!({
                    "id": athlete.id,
                    "name": athlete.long_name,
                    "short_name": athlete.short_name,
                    "country_code": athlete.country_code,
                    "position": match_athlete.athlete_position,
                })
            })
            .collect::<Vec<_>>();

        let videos = entry
            .videos
            .into_iter()
            .map(|video| {
                serde_json::json!({
                    "id": video.id,
                    "video_type": video.video_type,
                    "file_path": video.file_path,
                    "record_directory": video.record_directory,
                    "start_time": video.start_time,
                    "duration_seconds": video.duration_seconds,
                    "created_at": video.created_at,
                })
            })
            .collect::<Vec<_>>();

        matches_json.push(serde_json::json!({
            "match_db_id": match_db_id,
            "match_id": entry.match_row.match_code,
            "match_number": entry.match_row.match_number,
            "category": entry.match_row.category,
            "weight": entry.match_row.weight_class,
            "division": entry.match_row.division,
            "created_at": entry.match_row.created_at,
            "athletes": athletes,
            "videos": videos,
        }));
    }

    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({ "matches": matches_json })),
        error: None,
    })
}

/// Open a recorded video file at an optional offset
#[tauri::command]
pub async fn ivr_open_video_file(
    file_path: String,
    offset_seconds: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    match app
        .open_video_at(file_path, offset_seconds.unwrap_or(0))
        .await
    {
        Ok(()) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({"opened": true})),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

// ============================================================================
// OBS Profile Read-backs
// ============================================================================

#[tauri::command]
pub async fn obs_obws_get_record_directory(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    let res = app
        .obs_obws_plugin()
        .get_record_directory(connection_name.as_deref())
        .await;
    match res {
        Ok(dir) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({"directory": dir})),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn obs_obws_get_filename_formatting(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    let res = app
        .obs_obws_plugin()
        .get_filename_formatting(connection_name.as_deref())
        .await;
    match res {
        Ok(fmt) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({"formatting": fmt})),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Add a new OBS connection using obws
#[tauri::command]
pub async fn obs_obws_add_connection(
    connection: ObsObwsConnectionRequest,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!(
        "OBS obws add connection called: {}@{}:{}",
        connection.name,
        connection.host,
        connection.port
    );

    let config = ObsConnectionConfig {
        name: connection.name,
        host: connection.host,
        port: connection.port,
        password: connection.password,
        timeout_seconds: 30,
        role: crate::plugins::obs_obws::ObsConnectionRole::None,
    };

    match app.obs_obws_plugin().add_connection(config).await {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "OBS connection added successfully"
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Update an existing OBS connection using obws
#[tauri::command]
pub async fn obs_obws_update_connection(
    old_name: String,
    connection: ObsObwsConnectionRequest,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!(
        "OBS obws update connection called: {} -> {}@{}:{}",
        old_name,
        connection.name,
        connection.host,
        connection.port
    );

    let config = ObsConnectionConfig {
        name: connection.name,
        host: connection.host,
        port: connection.port,
        password: connection.password,
        timeout_seconds: 30,
        role: crate::plugins::obs_obws::ObsConnectionRole::None,
    };

    match app
        .obs_obws_plugin()
        .update_connection(&old_name, config)
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "OBS connection updated successfully"
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Connect to an OBS instance using obws
#[tauri::command]
pub async fn obs_obws_connect(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws connect called: {connection_name}");

    match app.obs_obws_plugin().connect(&connection_name).await {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Connected to OBS successfully"
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Disconnect from an OBS instance using obws
#[tauri::command]
pub async fn obs_obws_disconnect(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws disconnect called: {connection_name}");

    match app.obs_obws_plugin().disconnect(&connection_name).await {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Disconnected from OBS successfully"
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get connection status using obws
#[tauri::command]
pub async fn obs_obws_get_connection_status(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get connection status called: {connection_name}");

    match app
        .obs_obws_plugin()
        .get_connection_status(&connection_name)
        .await
    {
        Ok(status) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "status": status
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get all connections using obws
#[tauri::command]
pub async fn obs_obws_get_connections(
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get connections called");

    match app.obs_obws_plugin().get_connections().await {
        Ok(connections) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "connections": connections
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Remove a connection using obws
#[tauri::command]
pub async fn obs_obws_remove_connection(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws remove connection called: {connection_name}");

    match app
        .obs_obws_plugin()
        .remove_connection(&connection_name)
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Connection removed successfully"
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get OBS status using obws
#[tauri::command]
pub async fn obs_obws_get_status(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get status called");

    match app
        .obs_obws_plugin()
        .get_status(connection_name.as_deref())
        .await
    {
        Ok(status) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "status": status
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Start recording using obws
#[tauri::command]
pub async fn obs_obws_start_recording(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws start recording called");

    match app
        .obs_obws_plugin()
        .start_recording(connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Recording started successfully"
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Stop recording using obws
#[tauri::command]
pub async fn obs_obws_stop_recording(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws stop recording called");

    match app
        .obs_obws_plugin()
        .stop_recording(connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Recording stopped successfully"
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get recording status using obws
#[tauri::command]
pub async fn obs_obws_get_recording_status(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get recording status called");

    match app
        .obs_obws_plugin()
        .get_recording_status(connection_name.as_deref())
        .await
    {
        Ok(status) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "status": status
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Start streaming using obws
#[tauri::command]
pub async fn obs_obws_start_streaming(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws start streaming called");

    match app
        .obs_obws_plugin()
        .start_streaming(connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Streaming started successfully"
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Stop streaming using obws
#[tauri::command]
pub async fn obs_obws_stop_streaming(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws stop streaming called");

    match app
        .obs_obws_plugin()
        .stop_streaming(connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Streaming stopped successfully"
            })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get streaming status using obws
#[tauri::command]
pub async fn obs_obws_get_streaming_status(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get streaming status called");


// ============================================================================
// OBS Connection Management (Database)
// ============================================================================

use crate::database::models::ObsConnection;
use chrono::Utc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObsConnectionPayload {
    pub id: Option<i64>,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub is_active: bool,
    pub status: String,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn obs_connections_get_all(
    app: State<'_, Arc<App>>,
) -> Result<Vec<ObsConnection>, TauriError> {
    let plugin = app.database_plugin();

    let connections = plugin
        .list_obs_connections()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(connections)
}

#[tauri::command]
pub async fn obs_connections_get_active(
    app: State<'_, Arc<App>>,
) -> Result<Vec<ObsConnection>, TauriError> {
    let plugin = app.database_plugin();

    let connections = plugin
        .list_active_obs_connections()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(connections)
}

#[tauri::command]
pub async fn obs_connections_save(
    app: State<'_, Arc<App>>,
    connection: ObsConnectionPayload,
) -> Result<ObsConnection, TauriError> {
    let plugin = app.database_plugin();

    let obs_connection = ObsConnection {
        id: connection.id,
        name: connection.name,
        host: connection.host,
        port: connection.port,
        password: connection.password,
        is_active: connection.is_active,
        status: connection.status,
        error: connection.error,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let saved = plugin
        .save_obs_connection(&obs_connection)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(saved)
}

#[tauri::command]
pub async fn obs_connections_update_status(
    app: State<'_, Arc<App>>,
    name: String,
    status: String,
    error: Option<String>,
) -> Result<(), TauriError> {
    let plugin = app.database_plugin();

    plugin
        .update_obs_connection_status(&name, &status, error.as_deref())
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(())
}

#[tauri::command]
pub async fn obs_connections_delete(
    app: State<'_, Arc<App>>,
    name: String,
) -> Result<(), TauriError> {
    let plugin = app.database_plugin();

    plugin
        .delete_obs_connection(&name)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(())
}

#[tauri::command]
pub async fn obs_connections_clear_all(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    let plugin = app.database_plugin();

    plugin
        .clear_obs_connections()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    Ok(())
}

#[tauri::command]
pub async fn obs_connections_sync_from_config(
    app: State<'_, Arc<App>>,
) -> Result<Vec<ObsConnection>, TauriError> {
    let plugin = app.database_plugin();

    // Get connections from config manager
    let config_connections = app.config_manager().get_obs_connections().await;

    // Clear existing database connections
    plugin
        .clear_obs_connections()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    // Convert config connections to database connections
    let mut db_connections = Vec::new();
    for config_conn in config_connections {
        let obs_connection = ObsConnection {
            id: None,
            name: config_conn.name,
            host: config_conn.host,
            port: config_conn.port,
            password: config_conn.password,
            is_active: config_conn.enabled,
            status: "disconnected".to_string(),
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let saved = plugin
            .save_obs_connection(&obs_connection)
            .await
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

        db_connections.push(saved);
    }

    Ok(db_connections)
}
