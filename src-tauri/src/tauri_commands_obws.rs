//! Tauri commands for OBS WebSocket integration using obws crate

use crate::core::app::App;
// AppError and AppResult are used in the ObsManager implementation

use crate::plugins::obs_obws::ObsConnectionConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Error as TauriError, State};

// progress emit helpers

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
    let conn = app.database_plugin().get_connection().await?;
    use crate::database::operations::UiSettingsOperations as UIOps;
    let mpv_path = UIOps::get_ui_setting(&conn, "ivr.replay.mpv_path")
        .ok()
        .flatten();
    let seconds_from_end = UIOps::get_ui_setting(&conn, "ivr.replay.seconds_from_end")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(10);
    let max_wait_ms = UIOps::get_ui_setting(&conn, "ivr.replay.max_wait_ms")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(500);
    let auto_on_challenge = UIOps::get_ui_setting(&conn, "ivr.replay.auto_on_challenge")
        .ok()
        .flatten()
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
    let mut conn = app.database_plugin().get_connection().await?;
    use crate::database::operations::UiSettingsOperations as UIOps;
    // Ensure keys exist before setting values so updates don't fail silently
    let _ = UIOps::ensure_key(&conn, "ivr.replay.mpv_path", "MPV Path", "string", None);
    let _ = UIOps::ensure_key(
        &conn,
        "ivr.replay.seconds_from_end",
        "IVR Seconds From End",
        "integer",
        Some("10"),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "ivr.replay.max_wait_ms",
        "IVR Max Wait (ms)",
        "integer",
        Some("500"),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "ivr.replay.auto_on_challenge",
        "IVR Auto on Challenge",
        "boolean",
        Some("false"),
    );

    let secs = seconds_from_end.min(20);
    let wait = max_wait_ms.clamp(50, 500);
    if let Some(path) = mpv_path {
        let _ = UIOps::set_ui_setting(
            &mut conn,
            "ivr.replay.mpv_path",
            &path,
            "user",
            Some("update mpv path"),
        );
    }
    let _ = UIOps::set_ui_setting(
        &mut conn,
        "ivr.replay.seconds_from_end",
        &secs.to_string(),
        "user",
        Some("update ivr seconds"),
    );
    let _ = UIOps::set_ui_setting(
        &mut conn,
        "ivr.replay.max_wait_ms",
        &wait.to_string(),
        "user",
        Some("update ivr wait"),
    );
    let _ = UIOps::set_ui_setting(
        &mut conn,
        "ivr.replay.auto_on_challenge",
        if auto_on_challenge { "true" } else { "false" },
        "user",
        Some("update ivr auto"),
    );
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

    match app
        .obs_obws_plugin()
        .get_streaming_status(connection_name.as_deref())
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

/// Get current scene using obws
#[tauri::command]
pub async fn obs_obws_get_current_scene(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get current scene called");

    match app
        .obs_obws_plugin()
        .get_current_scene(connection_name.as_deref())
        .await
    {
        Ok(scene_name) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "scene_name": scene_name
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

/// Set current scene using obws
#[tauri::command]
pub async fn obs_obws_set_current_scene(
    scene_name: String,
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws set current scene called: {scene_name}");

    match app
        .obs_obws_plugin()
        .set_current_scene(&scene_name, connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Scene changed successfully"
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

/// Get scenes using obws
#[tauri::command]
pub async fn obs_obws_get_scenes(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get scenes called");

    match app
        .obs_obws_plugin()
        .get_scenes(connection_name.as_deref())
        .await
    {
        Ok(scenes) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "scenes": scenes
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

/// Get OBS version using obws
#[tauri::command]
pub async fn obs_obws_get_version(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get version called");

    match app
        .obs_obws_plugin()
        .get_version(connection_name.as_deref())
        .await
    {
        Ok(version) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "version": version
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

/// Get OBS stats using obws
#[tauri::command]
pub async fn obs_obws_get_stats(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get stats called");

    match app
        .obs_obws_plugin()
        .get_stats(connection_name.as_deref())
        .await
    {
        Ok(stats) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "stats": stats
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

/// Test obws connection
#[tauri::command]
pub async fn obs_obws_test_connection(
    _app: State<'_, Arc<App>>, // Prefix with _ since we don't use it yet
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws test connection called");

    // Test the obws implementation
    match crate::plugins::obs_obws::test_implementation::test_obs_obws_plugin().await {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "OBS obws plugin test completed successfully"
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

/// Set up status listener using obws
#[tauri::command]
pub async fn obs_obws_setup_status_listener(
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws setup status listener called");

    let app_arc = app.inner().clone();
    app_arc
        .ensure_obs_health_task()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
    app_arc
        .ensure_pss_stats_task()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;

    match app.obs_obws_plugin().setup_status_listener().await {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Status listener set up successfully"
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

// ============================================================================
// Replay Buffer Commands
// ============================================================================

/// Start replay buffer using obws
#[tauri::command]
pub async fn obs_obws_start_replay_buffer(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws start replay buffer called");

    match app
        .obs_obws_plugin()
        .start_replay_buffer(connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Replay buffer started successfully"
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

/// Stop replay buffer using obws
#[tauri::command]
pub async fn obs_obws_stop_replay_buffer(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws stop replay buffer called");

    match app
        .obs_obws_plugin()
        .stop_replay_buffer(connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Replay buffer stopped successfully"
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

/// Save replay buffer using obws
#[tauri::command]
pub async fn obs_obws_save_replay_buffer(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws save replay buffer called");

    match app
        .obs_obws_plugin()
        .save_replay_buffer(connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Replay buffer saved successfully"
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

/// Get replay buffer status using obws
#[tauri::command]
pub async fn obs_obws_get_replay_buffer_status(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get replay buffer status called");

    match app
        .obs_obws_plugin()
        .get_replay_buffer_status(connection_name.as_deref())
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

// ============================================================================
// Path Configuration Commands
// ============================================================================

/// Get recording path settings using obws
#[tauri::command]
pub async fn obs_obws_get_recording_path_settings(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get recording path settings called");

    let directory_result = app
        .obs_obws_plugin()
        .get_record_directory(connection_name.as_deref())
        .await;
    let formatting_result = app
        .obs_obws_plugin()
        .get_filename_formatting(connection_name.as_deref())
        .await;

    match (directory_result, formatting_result) {
        (Ok(dir), Ok(pattern)) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "recording_path": dir,
                "filename_pattern": pattern
            })),
            error: None,
        }),
        (dir_res, fmt_res) => {
            let err_msg = format!(
                "Failed to get recording settings: dir={:?}, pattern={:?}",
                dir_res.err().map(|e| e.to_string()),
                fmt_res.err().map(|e| e.to_string())
            );
            Ok(ObsObwsConnectionResponse {
                success: false,
                data: None,
                error: Some(err_msg),
            })
        }
    }
}

/// Set recording path using obws
#[tauri::command]
pub async fn obs_obws_set_recording_path(
    path: String,
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws set recording path called: {path}");
    match app
        .obs_obws_plugin()
        .set_record_directory(&path, connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({ "recording_path": path })),
            error: None,
        }),
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Set recording filename formatting using obws
#[tauri::command]
pub async fn obs_obws_set_recording_filename(
    pattern: String,
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws set recording filename called: {pattern}");
    match app
        .obs_obws_plugin()
        .set_filename_formatting(&pattern, connection_name.as_deref())
        .await
    {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "filename_pattern": pattern,
                "message": "Filename formatting set successfully"
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

// ============================================================================
// Scene Items (Sources) Commands
// ============================================================================

/// List sources (scene items) for a scene using obws. Placeholder implementation until client support is added.
#[tauri::command]
pub async fn obs_obws_get_sources(
    scene_name: String,
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get sources called for scene: {scene_name}");
    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "scene_name": scene_name,
            "sources": []
        })),
        error: None,
    })
}

/// Set source visibility in a scene using obws. Placeholder implementation until client support is added.
#[tauri::command]
pub async fn obs_obws_set_source_visibility(
    scene_name: String,
    source_name: String,
    visible: bool,
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!(
        "OBS obws set source visibility called: scene={scene_name}, source={source_name}, visible={visible}"
    );
    Ok(ObsObwsConnectionResponse {
        success: false,
        data: None,
        error: Some("Source visibility not implemented in obws integration yet".to_string()),
    })
}

/// Get replay buffer path settings using obws
#[tauri::command]
pub async fn obs_obws_get_replay_buffer_path_settings(
    _connection_name: Option<String>,
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get replay buffer path settings called");

    // For now, return a placeholder since obws doesn't have direct path configuration
    // This will be implemented using custom requests in the future
    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "replay_buffer_path": "C:/Users/Damjan/Videos/ReplayBuffer",
            "replay_buffer_format": "mp4",
            "message": "Replay buffer path settings retrieved (placeholder)"
        })),
        error: None,
    })
}

/// Set replay buffer path using obws
#[tauri::command]
pub async fn obs_obws_set_replay_buffer_path(
    path: String,
    _connection_name: Option<String>,
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws set replay buffer path called: {path}");

    // For now, return a placeholder since obws doesn't have direct path configuration
    // This will be implemented using custom requests in the future
    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "replay_buffer_path": path,
            "message": "Replay buffer path set successfully (placeholder)"
        })),
        error: None,
    })
}

// ============================================================================
// Recording Configuration Commands
// ============================================================================

// (obsolete) obs_obws_get_recording_config removed in favor of unified load

// (obsolete) obs_obws_save_recording_config removed in favor of unified save

/// Create a new recording session
#[tauri::command]
pub async fn obs_obws_create_recording_session(
    session: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws create recording session called");

    // Parse the session from JSON
    match serde_json::from_value::<crate::database::models::ObsRecordingSession>(session) {
        Ok(recording_session) => {
            let mut conn = app.database_plugin().get_connection().await?;
            match crate::database::operations::ObsRecordingOperations::create_recording_session(
                &mut conn,
                &recording_session,
            ) {
                Ok(session_id) => Ok(ObsObwsConnectionResponse {
                    success: true,
                    data: Some(serde_json::json!({
                        "session_id": session_id,
                        "message": "Recording session created successfully"
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
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(format!("Invalid session format: {e}")),
        }),
    }
}

/// Update recording session status
#[tauri::command]
pub async fn obs_obws_update_recording_session_status(
    session_id: i64,
    status: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws update recording session status called: {session_id} -> {status}");

    let mut conn = app.database_plugin().get_connection().await?;
    match crate::database::operations::ObsRecordingOperations::update_recording_session_status(
        &mut conn, session_id, &status, None,
    ) {
        Ok(_) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Recording session status updated successfully"
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

/// Generate recording path for a match
#[tauri::command]
pub async fn obs_obws_generate_recording_path(
    match_id: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws generate recording path called: match_id={match_id}");

    // Get database connection
    let conn = app.database_plugin().get_connection().await.map_err(|e| {
        TauriError::from(std::io::Error::other(format!(
            "Database connection error: {e}"
        )))
    })?;

    // Get active tournament and tournament day
    let tournament = crate::database::operations::TournamentOperations::get_active_tournament(
        &conn,
    )
    .map_err(|e| {
        TauriError::from(std::io::Error::other(format!(
            "Failed to get active tournament: {e}"
        )))
    })?;

    let tournament_day = if let Some(ref tournament) = tournament {
        crate::database::operations::TournamentOperations::get_active_tournament_day(
            &conn,
            tournament.id.unwrap(),
        )
        .map_err(|e| {
            TauriError::from(std::io::Error::other(format!(
                "Failed to get active tournament day: {e}"
            )))
        })?
    } else {
        None
    };

    drop(conn);

    // Get match details
    let matches = app
        .database_plugin()
        .get_pss_matches(Some(100))
        .await
        .map_err(|e| {
            TauriError::from(std::io::Error::other(format!("Failed to get matches: {e}")))
        })?;

    let match_info = matches
        .into_iter()
        .find(|m| m.match_id == match_id)
        .ok_or_else(|| {
            TauriError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Match not found: {match_id}"),
            ))
        })?;

    // Get match athletes
    let match_athletes = app
        .database_plugin()
        .get_pss_match_athletes(match_info.id.unwrap())
        .await
        .map_err(|e| {
            TauriError::from(std::io::Error::other(format!(
                "Failed to get match athletes: {e}"
            )))
        })?;

    // Extract player information
    let mut player1_name = None;
    let mut player1_flag = None;
    let mut player2_name = None;
    let mut player2_flag = None;

    for (match_athlete, athlete) in match_athletes {
        match match_athlete.athlete_position {
            1 => {
                player1_name = Some(athlete.short_name);
                player1_flag = athlete.country_code;
            }
            2 => {
                player2_name = Some(athlete.short_name);
                player2_flag = athlete.country_code;
            }
            _ => {}
        }
    }

    // Load recording config to supply folder_pattern
    let config_for_pattern = {
        let conn = app.database_plugin().get_connection().await;
        if let Ok(conn) = conn {
            crate::database::operations::ObsRecordingOperations::get_recording_config(
                &conn, "OBS_REC",
            )
            .ok()
            .flatten()
        } else {
            None
        }
    };

    let path_generator = if let Some(cfg) = config_for_pattern {
        let gen_cfg = crate::plugins::obs_obws::PathGeneratorConfig {
            videos_root: std::path::PathBuf::from(cfg.recording_root_path),
            default_format: cfg.recording_format,
            include_minutes_seconds: true,
            folder_pattern: Some(cfg.folder_pattern),
        };
        crate::plugins::obs_obws::ObsPathGenerator::new(Some(gen_cfg))
    } else {
        crate::plugins::obs_obws::ObsPathGenerator::new(None)
    };

    match path_generator.generate_recording_path(
        &match_id,
        tournament.map(|t| t.name),
        tournament_day.map(|td| format!("Day {}", td.day_number)),
        match_info.match_number,
        player1_name.clone(),
        player1_flag.clone(),
        player2_name.clone(),
        player2_flag.clone(),
    ) {
        Ok(generated_path) => {
            if let Err(e) = path_generator.ensure_directory_exists(&generated_path.directory) {
                return Ok(ObsObwsConnectionResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to create directory: {e}")),
                });
            }

            Ok(ObsObwsConnectionResponse {
                success: true,
                data: Some(serde_json::json!({
                    "full_path": generated_path.full_path.to_string_lossy(),
                    "directory": generated_path.directory.to_string_lossy(),
                    "filename": generated_path.filename,
                    "tournament_name": generated_path.tournament_name,
                    "tournament_day": generated_path.tournament_day,
                    "match_number": generated_path.match_number,
                    "player1_name": player1_name,
                    "player1_flag": player1_flag,
                    "player2_name": player2_name,
                    "player2_flag": player2_flag,
                })),
                error: None,
            })
        }
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get Windows Videos folder path
#[tauri::command]
pub async fn obs_obws_get_windows_videos_folder(
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get Windows Videos folder called");

    let videos_path = crate::plugins::obs_obws::PathGeneratorConfig::detect_windows_videos_folder();

    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "videos_path": videos_path.to_string_lossy(),
            "exists": videos_path.exists(),
        })),
        error: None,
    })
}

/// Test path generation with sample data
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn obs_obws_test_path_generation(
    match_id: String,
    tournament_name: Option<String>,
    tournament_day: Option<String>,
    match_number: Option<String>,
    player1_name: Option<String>,
    player1_flag: Option<String>,
    player2_name: Option<String>,
    player2_flag: Option<String>,
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws test path generation called");

    let config = crate::plugins::obs_obws::PathGeneratorConfig {
        videos_root: crate::plugins::obs_obws::PathGeneratorConfig::detect_windows_videos_folder(),
        default_format: "mp4".to_string(),
        include_minutes_seconds: true,
        folder_pattern: Some("{tournament}/{tournamentDay}".to_string()),
    };

    let path_generator = crate::plugins::obs_obws::ObsPathGenerator::new(Some(config));

    // Create a test directory path
    let directory =
        path_generator.generate_directory_path(&tournament_name, &tournament_day, &match_number);

    // Create test match info
    let test_match_info = crate::plugins::obs_obws::path_generator::MatchInfo {
        match_id: match_id.clone(),
        match_number: match_number.clone(),
        player1_name,
        player1_flag,
        player2_name,
        player2_flag,
    };

    // Generate test filename
    let filename =
        path_generator.generate_filename(&test_match_info, &tournament_name, &tournament_day);
    let full_path = directory.join(&filename);

    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "full_path": full_path.to_string_lossy(),
            "directory": directory.to_string_lossy(),
            "filename": filename,
            "tournament_name": tournament_name,
            "tournament_day": tournament_day,
            "match_number": match_number,
        })),
        error: None,
    })
}

/// Test recording functionality
#[tauri::command]
pub async fn obs_obws_test_recording(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws test recording called for connection: {connection_name}");

    // Use the obws plugin for recording controls to avoid legacy API
    if let Err(e) = app
        .obs_obws_plugin()
        .start_recording(Some(&connection_name))
        .await
    {
        return Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to start recording: {e}")),
        });
    }

    if let Err(e) = app
        .obs_obws_plugin()
        .start_replay_buffer(Some(&connection_name))
        .await
    {
        return Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to start replay buffer: {e}")),
        });
    }

    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "message": "Recording test successful! Recording and replay buffer started."
        })),
        error: None,
    })
}

/// Apply path decision from UI (tournament/day overrides)
#[tauri::command]
pub async fn obs_obws_apply_path_decision(
    tournament_name: String,
    tournament_day: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws apply path decision called: {tournament_name} / {tournament_day}");

    let handler = app.recording_event_handler();
    // Ensure the chosen tournament/day exist in DB and set them active so indexing will capture non-null IDs
    {
        use crate::database::models::Tournament;
        use crate::database::operations::TournamentOperations as TOps;
        let mut conn = app.database_plugin().get_connection().await?;
        // Find or create tournament by name
        let tid_opt: Option<i64> = conn
            .query_row(
                "SELECT id FROM tournaments WHERE name = ?",
                rusqlite::params![&tournament_name],
                |r| r.get::<_, i64>(0),
            )
            .ok();
        let tid: i64 = if let Some(id) = tid_opt {
            id
        } else {
            // Create a minimal tournament record
            let t = Tournament::new(
                tournament_name.clone(),
                30,
                "".to_string(),
                "".to_string(),
                None,
            );
            TOps::create_tournament(&mut conn, &t)
                .map_err(|e| TauriError::from(anyhow::anyhow!(format!("create_tournament: {e}"))))?
        };
        // Parse day number from string like "Day 3"
        let day_num: i32 = tournament_day
            .split_whitespace()
            .last()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);
        // Ensure the day exists up to day_num
        let existing: i64 = conn
            .query_row(
                "SELECT COUNT(1) FROM tournament_days WHERE tournament_id = ?",
                rusqlite::params![tid],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if existing < day_num as i64 {
            // Create missing days starting today; create_tournament_days appends days sequentially
            let start_dt = chrono::Utc::now();
            // Create enough days to reach day_num
            TOps::create_tournament_days(&mut conn, tid, start_dt, day_num).map_err(|e| {
                TauriError::from(anyhow::anyhow!(format!("create_tournament_days: {e}")))
            })?;
        }
        // Lookup the selected day id
        let day_id: i64 = conn
            .query_row(
                "SELECT id FROM tournament_days WHERE tournament_id = ? AND day_number = ?",
                rusqlite::params![tid, day_num],
                |r| r.get::<_, i64>(0),
            )
            .map_err(|e| TauriError::from(anyhow::anyhow!(format!("resolve day_id: {e}"))))?;
        // Start the selected day (mark active)
        TOps::start_tournament_day(&mut conn, day_id)
            .map_err(|e| TauriError::from(anyhow::anyhow!(format!("start_tournament_day: {e}"))))?;
        // Update UDP/tournament context so subsequent events carry these IDs
        app.udp_plugin()
            .set_tournament_context(Some(tid))
            .await
            .map_err(|e| {
                TauriError::from(anyhow::anyhow!(format!("set_tournament_context: {e}")))
            })?;
    }

    match handler
        .regenerate_path_with_overrides(tournament_name, tournament_day)
        .await
    {
        Ok(()) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": "Recording path decision applied"
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

/// Create test folders in Windows (actually creates the directory structure)
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn obs_obws_create_test_folders(
    match_id: String,
    tournament_name: Option<String>,
    tournament_day: Option<String>,
    match_number: Option<String>,
    player1_name: Option<String>,
    player1_flag: Option<String>,
    player2_name: Option<String>,
    player2_flag: Option<String>,
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws create test folders called");

    let config = crate::plugins::obs_obws::PathGeneratorConfig {
        videos_root: crate::plugins::obs_obws::PathGeneratorConfig::detect_windows_videos_folder(),
        default_format: "mp4".to_string(),
        include_minutes_seconds: true,
        folder_pattern: Some("{tournament}/{tournamentDay}".to_string()),
    };

    let path_generator = crate::plugins::obs_obws::ObsPathGenerator::new(Some(config));

    // Create a test directory path
    let directory =
        path_generator.generate_directory_path(&tournament_name, &tournament_day, &match_number);

    // Actually create the directory
    match path_generator.ensure_directory_exists(&directory) {
        Ok(_) => {
            // Create test match info
            let test_match_info = crate::plugins::obs_obws::path_generator::MatchInfo {
                match_id: match_id.clone(),
                match_number: match_number.clone(),
                player1_name,
                player1_flag,
                player2_name,
                player2_flag,
            };

            // Generate test filename
            let filename = path_generator.generate_filename(
                &test_match_info,
                &tournament_name,
                &tournament_day,
            );
            let full_path = directory.join(&filename);

            Ok(ObsObwsConnectionResponse {
                success: true,
                data: Some(serde_json::json!({
                    "full_path": full_path.to_string_lossy(),
                    "directory": directory.to_string_lossy(),
                    "filename": filename,
                    "tournament_name": tournament_name,
                    "tournament_day": tournament_day,
                    "match_number": match_number,
                    "message": "Folders created successfully!"
                })),
                error: None,
            })
        }
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to create folders: {e}")),
        }),
    }
}

/// Send recording configuration to OBS
#[tauri::command]
pub async fn obs_obws_send_config_to_obs(
    connection_name: String,
    recording_path: String,
    filename_template: String,
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws send config to OBS called for connection: {connection_name}");
    // Route through obws manager to set recording directory and filename formatting
    let result = async {
        let manager = _app.obs_obws_plugin();
        manager
            .set_record_directory(&recording_path, Some(&connection_name))
            .await?;
        manager
            .set_filename_formatting(&filename_template, Some(&connection_name))
            .await?;
        crate::types::AppResult::<()>::Ok(())
    }
    .await;

    match result {
        Ok(()) => Ok(ObsObwsConnectionResponse {
            success: true,
            data: Some(serde_json::json!({
                "message": format!("Configuration applied to OBS connection '{}' successfully", connection_name),
                "recording_path": recording_path,
                "filename_template": filename_template
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

// (obsolete) obs_obws_get_automatic_recording_config removed in favor of unified load

// (obsolete) obs_obws_update_automatic_recording_config removed in favor of unified save

/// Get current recording session
#[tauri::command]
pub async fn obs_obws_get_current_recording_session(
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get current recording session called");

    // Get the recording event handler from the app state
    let recording_handler = app.recording_event_handler();

    let session = recording_handler.get_current_session();

    match session {
        Some(session) => {
            let state_str = match session.state {
                crate::plugins::obs_obws::RecordingState::Idle => "idle",
                crate::plugins::obs_obws::RecordingState::Preparing => "preparing",
                crate::plugins::obs_obws::RecordingState::Recording => "recording",
                crate::plugins::obs_obws::RecordingState::Stopping => "stopping",
                crate::plugins::obs_obws::RecordingState::Error(ref _msg) => "error",
            };

            Ok(ObsObwsConnectionResponse {
                success: true,
                data: Some(serde_json::json!({
                    "id": session.id,
                    "match_id": session.match_id,
                    "tournament_name": session.tournament_name,
                    "tournament_day": session.tournament_day,
                    "match_number": session.match_number,
                    "player1_name": session.player1_name,
                    "player1_flag": session.player1_flag,
                    "player2_name": session.player2_name,
                    "player2_flag": session.player2_flag,
                    "recording_path": session.recording_path,
                    "recording_filename": session.recording_filename,
                    "state": state_str,
                    "start_time": session.start_time.map(|t| t.timestamp()),
                    "end_time": session.end_time.map(|t| t.timestamp()),
                    "obs_connection_name": session.obs_connection_name,
                    "created_at": session.created_at.timestamp(),
                    "updated_at": session.updated_at.timestamp(),
                })),
                error: None,
            })
        }
        None => Ok(ObsObwsConnectionResponse {
            success: true,
            data: None,
            error: None,
        }),
    }
}

/// Clear current recording session
#[tauri::command]
pub async fn obs_obws_clear_recording_session(
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws clear recording session called");

    // Get the recording event handler from the app state
    let recording_handler = app.recording_event_handler();

    recording_handler.clear_session().map_err(|e| {
        TauriError::from(std::io::Error::other(format!(
            "Failed to clear session: {e}"
        )))
    })?;

    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "message": "Recording session cleared successfully"
        })),
        error: None,
    })
}

/// Manually start recording for a match
#[tauri::command]
pub async fn obs_obws_manual_start_recording(
    match_id: String,
    obs_connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!(
        "OBS obws manual start recording called: match_id={match_id}, connection={obs_connection_name}"
    );

    // Get the recording event handler from the app state
    let recording_handler = app.recording_event_handler();

    // Create a manual recording session
    let session = crate::plugins::obs_obws::RecordingSession {
        id: None,
        db_session_id: None,
        match_id: match_id.clone(),
        match_db_id: None,
        tournament_name: None,
        tournament_day: None,
        tournament_id: None,
        tournament_day_id: None,
        match_number: None,
        player1_name: None,
        player1_flag: None,
        player2_name: None,
        player2_flag: None,
        recording_path: None,
        recording_filename: None,
        state: crate::plugins::obs_obws::RecordingState::Preparing,
        start_time: None,
        end_time: None,
        obs_connection_name: Some(obs_connection_name.clone()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Update current session
    {
        let mut session_guard = recording_handler.current_session.lock().unwrap();
        *session_guard = Some(session);
    }

    // Generate recording path
    if let Err(e) = recording_handler.generate_recording_path(&match_id).await {
        log::error!("Failed to generate recording path: {e}");
        recording_handler
            .update_session_state(crate::plugins::obs_obws::RecordingState::Error(
                e.to_string(),
            ))
            .await
            .map_err(|e| {
                TauriError::from(std::io::Error::other(format!(
                    "Failed to update session state: {e}"
                )))
            })?;

        return Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to generate recording path: {e}")),
        });
    }

    // Start recording
    recording_handler
        .update_session_state(crate::plugins::obs_obws::RecordingState::Recording)
        .await
        .map_err(|e| {
            TauriError::from(std::io::Error::other(format!(
                "Failed to update session state: {e}"
            )))
        })?;

    // Start recording immediately via obws (authoritative) to avoid depending on event consumers
    if let Err(e) = app
        .obs_obws_plugin()
        .start_recording(Some(&obs_connection_name))
        .await
    {
        log::error!("Failed to start recording via obws: {e}");
        return Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        });
    }

    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "message": "Manual recording started successfully"
        })),
        error: None,
    })
}

/// Manually stop recording
#[tauri::command]
pub async fn obs_obws_manual_stop_recording(
    obs_connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws manual stop recording called: connection={obs_connection_name}");

    // Get the recording event handler from the app state
    let recording_handler = app.recording_event_handler();

    // Update session state to stopping
    recording_handler
        .update_session_state(crate::plugins::obs_obws::RecordingState::Stopping)
        .await
        .map_err(|e| {
            TauriError::from(std::io::Error::other(format!(
                "Failed to update session state: {e}"
            )))
        })?;

    // Stop recording immediately via obws
    if let Err(e) = app
        .obs_obws_plugin()
        .stop_recording(Some(&obs_connection_name))
        .await
    {
        log::error!("Failed to stop recording via obws: {e}");
        return Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        });
    }

    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "message": "Manual recording stopped successfully"
        })),
        error: None,
    })
}

// ============================================================================
// Unified save for Recording + Automatic config
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct FullObsConfigPayload {
    // Recording config
    connection_name: String,
    recording_path: String,
    recording_format: String,
    filename_template: String,
    #[serde(default = "default_folder_pattern")]
    folder_pattern: String,
    // Automatic config
    enabled: bool,
    stop_delay_seconds: u32,
    include_replay_buffer: bool,
    #[serde(default)]
    replay_buffer_duration: Option<u32>,
    #[serde(default = "default_true")]
    auto_stop_on_match_end: bool,
    #[serde(default = "default_true")]
    auto_stop_on_winner: bool,
    // Right column (auto-start) flags
    #[serde(default = "default_true")]
    auto_start_recording_on_match_begin: bool,
    #[serde(default = "default_true")]
    auto_start_replay_on_match_begin: bool,
}

fn default_folder_pattern() -> String {
    "{tournament}/{tournamentDay}".to_string()
}
fn default_true() -> bool {
    true
}

#[tauri::command]
pub async fn obs_obws_save_full_config(
    payload: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    // Parse payload
    let cfg: FullObsConfigPayload = serde_json::from_value(payload).map_err(|e| {
        TauriError::from(anyhow::anyhow!(format!("Invalid full config payload: {e}")))
    })?;

    println!(
        " obs_obws_save_full_config(conn='{}', path='{}', fmt='{}', tmpl='{}', enabled={}, stop_delay={}, include_rb={}, rb_dur={:?}, stop_on_end={}, stop_on_winner={}, start_rec={}, start_replay={})",
        cfg.connection_name,
        cfg.recording_path,
        cfg.recording_format,
        cfg.filename_template,
        cfg.enabled,
        cfg.stop_delay_seconds,
        cfg.include_replay_buffer,
        cfg.replay_buffer_duration,
        cfg.auto_stop_on_match_end,
        cfg.auto_stop_on_winner,
        cfg.auto_start_recording_on_match_begin,
        cfg.auto_start_replay_on_match_begin,
    );

    // 1) Save recording config
    let recording_config = crate::database::models::ObsRecordingConfig {
        id: None,
        obs_connection_name: cfg.connection_name.clone(),
        recording_root_path: cfg.recording_path.clone(),
        recording_format: cfg.recording_format.clone(),
        replay_buffer_enabled: cfg.include_replay_buffer,
        replay_buffer_duration: Some(cfg.replay_buffer_duration.unwrap_or(30) as i32),
        auto_start_recording: cfg.auto_start_recording_on_match_begin,
        auto_start_replay_buffer: cfg.auto_start_replay_on_match_begin,
        filename_template: cfg.filename_template.clone(),
        folder_pattern: cfg.folder_pattern.clone(),
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let mut conn = app.database_plugin().get_connection().await?;
    crate::database::operations::ObsRecordingOperations::upsert_recording_config(
        &mut conn,
        &recording_config,
    )
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    // 2) Update in-memory handler and persist UiSettings
    let recording_handler = app.recording_event_handler();
    let new_auto_cfg = crate::plugins::obs_obws::AutomaticRecordingConfig {
        enabled: cfg.enabled,
        obs_connection_name: Some(cfg.connection_name.clone()),
        auto_stop_on_match_end: cfg.auto_stop_on_match_end,
        auto_stop_on_winner: cfg.auto_stop_on_winner,
        stop_delay_seconds: cfg.stop_delay_seconds,
        include_replay_buffer: cfg.include_replay_buffer,
        auto_start_recording_on_match_begin: cfg.auto_start_recording_on_match_begin,
        auto_start_replay_on_match_begin: cfg.auto_start_replay_on_match_begin,
    };
    recording_handler.update_config(new_auto_cfg).map_err(|e| {
        TauriError::from(anyhow::anyhow!(format!(
            "Failed to update handler config: {e}"
        )))
    })?;

    use crate::database::operations::UiSettingsOperations as UIOps;
    UIOps::set_ui_setting(
        &mut conn,
        "obs.auto.enabled",
        if cfg.enabled { "true" } else { "false" },
        "user",
        Some("save full config"),
    )
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(
        &mut conn,
        "obs.auto.connection",
        &cfg.connection_name,
        "user",
        Some("save full config"),
    )
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(
        &mut conn,
        "obs.auto.stop_delay_seconds",
        &cfg.stop_delay_seconds.to_string(),
        "user",
        Some("save full config"),
    )
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(
        &mut conn,
        "obs.auto.include_replay_buffer",
        if cfg.include_replay_buffer {
            "true"
        } else {
            "false"
        },
        "user",
        Some("save full config"),
    )
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(
        &mut conn,
        "obs.auto.stop_on_match_end",
        if cfg.auto_stop_on_match_end {
            "true"
        } else {
            "false"
        },
        "user",
        Some("save full config"),
    )
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(
        &mut conn,
        "obs.auto.stop_on_winner",
        if cfg.auto_stop_on_winner {
            "true"
        } else {
            "false"
        },
        "user",
        Some("save full config"),
    )
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(
        &mut conn,
        "obs.auto.start_recording_on_match_begin",
        if cfg.auto_start_recording_on_match_begin {
            "true"
        } else {
            "false"
        },
        "user",
        Some("save full config"),
    )
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(
        &mut conn,
        "obs.auto.start_replay_on_match_begin",
        if cfg.auto_start_replay_on_match_begin {
            "true"
        } else {
            "false"
        },
        "user",
        Some("save full config"),
    )
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    // removed: save replay on match end persistence

    // Also return live connection status for the selected connection so the UI doesn't reset indicator
    let live_status = app
        .obs_obws_plugin()
        .get_connection_status(&cfg.connection_name)
        .await
        .ok();
    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "message": "Recording and automatic configuration saved successfully",
            "connection": cfg.connection_name,
            "status": live_status
        })),
        error: None,
    })
}

// ============================================================================
// Unified load for Recording + Automatic config
// ============================================================================

#[tauri::command]
pub async fn obs_obws_get_full_config(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    use crate::database::operations::{
        ObsRecordingOperations as RecOps, UiSettingsOperations as UIOps,
    };

    let conn = app.database_plugin().get_connection().await?;

    // Resolve connection name: param -> UiSettings -> default
    let resolved_conn = connection_name
        .or(UIOps::get_ui_setting(&conn, "obs.auto.connection")
            .ok()
            .flatten())
        .unwrap_or_else(|| "OBS_REC".to_string());

    // Ensure keys exist so get_ui_setting won't fail on fresh DBs
    let _ = UIOps::ensure_key(
        &conn,
        "obs.auto.enabled",
        "OBS Auto Enabled",
        "boolean",
        Some("false"),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "obs.auto.connection",
        "OBS Auto Connection",
        "string",
        Some(&resolved_conn),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "obs.auto.stop_delay_seconds",
        "OBS Stop Delay",
        "integer",
        Some("30"),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "obs.auto.include_replay_buffer",
        "Include Replay Buffer",
        "boolean",
        Some("true"),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "obs.auto.start_recording_on_match_begin",
        "Auto-start Recording",
        "boolean",
        Some("true"),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "obs.auto.start_replay_on_match_begin",
        "Auto-start Replay",
        "boolean",
        Some("true"),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "obs.auto.save_replay_on_match_end",
        "Save Replay On End",
        "boolean",
        Some("false"),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "obs.auto.stop_on_match_end",
        "Stop On Match End",
        "boolean",
        Some("true"),
    );
    let _ = UIOps::ensure_key(
        &conn,
        "obs.auto.stop_on_winner",
        "Stop On Winner",
        "boolean",
        Some("true"),
    );

    // Recording config
    let rec_cfg = RecOps::get_recording_config(&conn, &resolved_conn)
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    // Automatic config from UiSettings (fallbacks applied)
    let enabled = UIOps::get_ui_setting(&conn, "obs.auto.enabled")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false);
    let stop_delay_seconds = UIOps::get_ui_setting(&conn, "obs.auto.stop_delay_seconds")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(30);
    let include_replay_buffer = UIOps::get_ui_setting(&conn, "obs.auto.include_replay_buffer")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(true);
    let auto_start_recording_on_match_begin =
        UIOps::get_ui_setting(&conn, "obs.auto.start_recording_on_match_begin")
            .ok()
            .flatten()
            .map(|v| v == "true")
            .unwrap_or(true);
    let auto_start_replay_on_match_begin =
        UIOps::get_ui_setting(&conn, "obs.auto.start_replay_on_match_begin")
            .ok()
            .flatten()
            .map(|v| v == "true")
            .unwrap_or(true);
    let save_replay_on_match_end = false;
    let auto_stop_on_match_end = UIOps::get_ui_setting(&conn, "obs.auto.stop_on_match_end")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(true);
    let auto_stop_on_winner = UIOps::get_ui_setting(&conn, "obs.auto.stop_on_winner")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(true);

    println!(" obs_obws_get_full_config(conn='{resolved_conn}')");

    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "connection_name": resolved_conn,
            "recording_config": rec_cfg,
            "automatic_config": {
                "enabled": enabled,
                "obs_connection_name": resolved_conn,
                "stop_delay_seconds": stop_delay_seconds,
                "include_replay_buffer": include_replay_buffer,
                "auto_stop_on_match_end": auto_stop_on_match_end,
                "auto_stop_on_winner": auto_stop_on_winner,
                "auto_start_recording_on_match_begin": auto_start_recording_on_match_begin,
                "auto_start_replay_on_match_begin": auto_start_replay_on_match_begin,
                "save_replay_on_match_end": save_replay_on_match_end
            }
        })),
        error: None,
    })
}

// =============================================================================
// OBS CONNECTION ROLES - Get/Set connection roles (recording, streaming, none)
// =============================================================================

/// Get the role of an OBS connection
#[tauri::command]
pub async fn obs_obws_get_connection_role(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting OBS connection role for: {connection_name}");

    #[cfg(feature = "obs-obws")]
    {
        match app.obs_obws_plugin().get_connection(&connection_name).await {
            Ok(client_arc) => {
                let client = client_arc.lock().await;
                let config = client.get_config().clone();
                Ok(serde_json::json!({
                    "success": true,
                    "connection_name": connection_name,
                    "role": config.role
                }))
            }
            Err(e) => {
                log::error!("Failed to get connection role: {e}");
                Ok(serde_json::json!({
                    "success": false,
                    "connection_name": connection_name,
                    "error": e.to_string()
                }))
            }
        }
    }

    #[cfg(not(feature = "obs-obws"))]
    {
        Ok(serde_json::json!({
            "success": false,
            "error": "OBS obws feature not enabled"
        }))
    }
}

/// Set the role of an OBS connection
#[tauri::command]
pub async fn obs_obws_set_connection_role(
    connection_name: String,
    role: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Setting OBS connection role for {connection_name} to {role}");

    #[cfg(feature = "obs-obws")]
    {
        use crate::plugins::obs_obws::ObsConnectionRole;

        // Parse the role string
        let new_role = match role.as_str() {
            "recording" => ObsConnectionRole::Recording,
            "streaming" => ObsConnectionRole::Streaming,
            "none" => ObsConnectionRole::None,
            _ => {
                return Ok(serde_json::json!({
                    "success": false,
                    "connection_name": connection_name,
                    "error": format!("Invalid role: {}. Valid roles are: recording, streaming, none", role)
                }));
            }
        };

        match app.obs_obws_plugin().get_connection(&connection_name).await {
            Ok(client_arc) => {
                let client = client_arc.lock().await;
                // Get current config and update role
                let mut config = client.get_config().clone();
                let old_role = config.role.clone();
                config.role = new_role.clone();

                // Update the connection
                match app
                    .obs_obws_plugin()
                    .update_connection(&connection_name, config)
                    .await
                {
                    Ok(_) => {
                        log::info!(
                            "Successfully updated connection role from {old_role:?} to {new_role:?}"
                        );
                        Ok(serde_json::json!({
                            "success": true,
                            "connection_name": connection_name,
                            "old_role": old_role,
                            "new_role": new_role
                        }))
                    }
                    Err(e) => {
                        log::error!("Failed to update connection role: {e}");
                        Ok(serde_json::json!({
                            "success": false,
                            "connection_name": connection_name,
                            "error": e.to_string()
                        }))
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get connection for role update: {e}");
                Ok(serde_json::json!({
                    "success": false,
                    "connection_name": connection_name,
                    "error": e.to_string()
                }))
            }
        }
    }

    #[cfg(not(feature = "obs-obws"))]
    {
        Ok(serde_json::json!({
            "success": false,
            "error": "OBS obws feature not enabled"
        }))
    }
}

/// Start monitoring an OBS connection for events
#[tauri::command]
pub async fn obs_obws_start_monitoring(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Starting OBS monitoring for: {connection_name}");

    #[cfg(feature = "obs-obws")]
    {
        match app.obs_obws_plugin().get_connection(&connection_name).await {
            Ok(client_arc) => {
                let mut client = client_arc.lock().await;

                // Check if already connected
                if client.get_connection_status()
                    == crate::plugins::obs_obws::types::ObsConnectionStatus::Disconnected
                {
                    // Connect first if not connected
                    if let Err(e) = client.connect().await {
                        return Ok(serde_json::json!({
                            "success": false,
                            "connection_name": connection_name,
                            "error": format!("Failed to connect: {}", e)
                        }));
                    }
                }

                // Start monitoring (subscribe to events)
                match client.start_monitoring().await {
                    Ok(_) => {
                        log::info!("Started monitoring for connection: {connection_name}");
                        Ok(serde_json::json!({
                            "success": true,
                            "connection_name": connection_name,
                            "message": "Monitoring started successfully"
                        }))
                    }
                    Err(e) => {
                        log::error!("Failed to start monitoring: {e}");
                        Ok(serde_json::json!({
                            "success": false,
                            "connection_name": connection_name,
                            "error": e.to_string()
                        }))
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get connection for monitoring: {e}");
                Ok(serde_json::json!({
                    "success": false,
                    "connection_name": connection_name,
                    "error": e.to_string()
                }))
            }
        }
    }

    #[cfg(not(feature = "obs-obws"))]
    {
        Ok(serde_json::json!({
            "success": false,
            "error": "OBS obws feature not enabled"
        }))
    }
}

/// Stop monitoring an OBS connection
#[tauri::command]
pub async fn obs_obws_stop_monitoring(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Stopping OBS monitoring for: {connection_name}");

    #[cfg(feature = "obs-obws")]
    {
        match app.obs_obws_plugin().get_connection(&connection_name).await {
            Ok(client_arc) => {
                let mut client = client_arc.lock().await;

                match client.stop_monitoring().await {
                    Ok(_) => {
                        log::info!("Stopped monitoring for connection: {connection_name}");
                        Ok(serde_json::json!({
                            "success": true,
                            "connection_name": connection_name,
                            "message": "Monitoring stopped successfully"
                        }))
                    }
                    Err(e) => {
                        log::error!("Failed to stop monitoring: {e}");
                        Ok(serde_json::json!({
                            "success": false,
                            "connection_name": connection_name,
                            "error": e.to_string()
                        }))
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get connection for stopping monitoring: {e}");
                Ok(serde_json::json!({
                    "success": false,
                    "connection_name": connection_name,
                    "error": e.to_string()
                }))
            }
        }
    }

    #[cfg(not(feature = "obs-obws"))]
    {
        Ok(serde_json::json!({
            "success": false,
            "error": "OBS obws feature not enabled"
        }))
    }
}
