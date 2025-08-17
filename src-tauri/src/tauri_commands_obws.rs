//! Tauri commands for OBS WebSocket integration using obws crate

use crate::core::app::App;
// AppError and AppResult are used in the ObsManager implementation

use crate::plugins::obs_obws::ObsConnectionConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, Error as TauriError, Emitter};

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
pub async fn ivr_get_replay_settings(app: State<'_, Arc<App>>) -> Result<ObsObwsConnectionResponse, TauriError> {
    let conn = app.database_plugin().get_connection().await?;
    use crate::database::operations::UiSettingsOperations as UIOps;
    let mpv_path = UIOps::get_ui_setting(&*conn, "ivr.replay.mpv_path").ok().flatten();
    let seconds_from_end = UIOps::get_ui_setting(&*conn, "ivr.replay.seconds_from_end").ok().flatten().and_then(|s| s.parse::<u32>().ok()).unwrap_or(10);
    let max_wait_ms = UIOps::get_ui_setting(&*conn, "ivr.replay.max_wait_ms").ok().flatten().and_then(|s| s.parse::<u32>().ok()).unwrap_or(500);
    let auto_on_challenge = UIOps::get_ui_setting(&*conn, "ivr.replay.auto_on_challenge").ok().flatten().map(|s| s == "true").unwrap_or(false);
    Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!({
      "mpv_path": mpv_path,
      "seconds_from_end": seconds_from_end,
      "max_wait_ms": max_wait_ms,
      "auto_on_challenge": auto_on_challenge
    })), error: None })
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
    let _ = UIOps::ensure_key(&*conn, "ivr.replay.mpv_path", "MPV Path", "string", None);
    let _ = UIOps::ensure_key(&*conn, "ivr.replay.seconds_from_end", "IVR Seconds From End", "integer", Some("10"));
    let _ = UIOps::ensure_key(&*conn, "ivr.replay.max_wait_ms", "IVR Max Wait (ms)", "integer", Some("500"));
    let _ = UIOps::ensure_key(&*conn, "ivr.replay.auto_on_challenge", "IVR Auto on Challenge", "boolean", Some("false"));

    let secs = seconds_from_end.min(20);
    let wait = max_wait_ms.clamp(50, 500);
    if let Some(path) = mpv_path {
        let _ = UIOps::set_ui_setting(&mut *conn, "ivr.replay.mpv_path", &path, "user", Some("update mpv path"));
    }
    let _ = UIOps::set_ui_setting(&mut *conn, "ivr.replay.seconds_from_end", &secs.to_string(), "user", Some("update ivr seconds"));
    let _ = UIOps::set_ui_setting(&mut *conn, "ivr.replay.max_wait_ms", &wait.to_string(), "user", Some("update ivr wait"));
    let _ = UIOps::set_ui_setting(&mut *conn, "ivr.replay.auto_on_challenge", if auto_on_challenge {"true"} else {"false"}, "user", Some("update ivr auto"));
    Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!({"message":"IVR replay settings saved"})), error: None })
}

#[tauri::command]
pub async fn ivr_round_replay_now(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    let conn_name = connection_name.unwrap_or_else(|| "OBS_REC".to_string());
    match app.replay_round_now(Some(&conn_name)).await {
        Ok(()) => Ok(ObsObwsConnectionResponse{ success:true, data: Some(serde_json::json!({"launched":true})), error: None }),
        Err(e) => Ok(ObsObwsConnectionResponse{ success:false, data: None, error: Some(e.to_string()) })
    }
}

/// Open recorded video at the exact time of the specified event
#[tauri::command]
pub async fn ivr_open_event_video(
    event_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    match app.open_event_video(event_id).await {
        Ok(()) => Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!({"opened": true})), error: None }),
        Err(e) => Ok(ObsObwsConnectionResponse{ success: false, data: None, error: Some(e.to_string()) })
    }
}

// ============================================================================
// IVR Match History - Read APIs
// ============================================================================

/// List tournaments and their days (flattened as day entries)
#[tauri::command]
pub async fn ivr_list_tournament_days(app: State<'_, Arc<App>>) -> Result<ObsObwsConnectionResponse, TauriError> {
    use crate::database::operations::TournamentOperations as TOps;
    use crate::database::models::Tournament;
    use chrono::Utc;

    // Ensure at least a default tournament/day exists
    let mut conn = app.database_plugin().get_connection().await?;
    let mut tournaments = TOps::get_tournaments(&*conn).unwrap_or_default();
    if tournaments.is_empty() {
        // Create default Tournament 1 with Day 1 and mark day active
        let default_t = Tournament::new("Tournament 1".to_string(), 1, "".to_string(), "".to_string(), None);
        let tid = TOps::create_tournament(&mut *conn, &default_t)
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        TOps::create_tournament_days(&mut *conn, tid, Utc::now(), 1)
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        // Make day 1 active
        if let Ok(days) = TOps::get_tournament_days(&*conn, tid) {
            if let Some(d) = days.first().and_then(|d| d.id) {
                let _ = TOps::start_tournament_day(&mut *conn, d);
            }
        }
        tournaments = TOps::get_tournaments(&*conn).unwrap_or_default();
    }
    let mut days: Vec<serde_json::Value> = Vec::new();
    for t in tournaments {
        let t_id = t.id.unwrap_or_default();
        let t_name = t.name.clone();
        let dlist = TOps::get_tournament_days(&*conn, t_id).unwrap_or_default();
        for d in dlist {
            days.push(serde_json::json!({
                "tournament_id": t_id,
                "tournament_name": t_name,
                "day_id": d.id.unwrap_or_default(),
                "day_number": d.day_number,
                "date": d.date.to_rfc3339(),
                "status": d.status,
            }));
        }
    }
    Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!(days)), error: None })
}

/// List matches for a given tournament day (based on recorded_videos linkage)
#[tauri::command]
pub async fn ivr_list_matches_for_day(day_id: i64, app: State<'_, Arc<App>>) -> Result<ObsObwsConnectionResponse, TauriError> {
    let conn = app.database_plugin().get_connection().await?;
    // Primary: matches with recordings for the given day
    let mut stmt = conn.prepare(
        "SELECT m.id, m.match_id, m.match_number, m.category, m.created_at, m.updated_at
         FROM pss_matches m
         JOIN recorded_videos rv ON rv.match_id = m.id
         WHERE rv.tournament_day_id = ?
         GROUP BY m.id
         ORDER BY m.created_at DESC"
    ).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    let mut rows = stmt.query_map(rusqlite::params![day_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "match_id": row.get::<_, String>(1)?,
            "match_number": row.get::<_, Option<String>>(2)?,
            "category": row.get::<_, Option<String>>(3)?,
            "created_at": row.get::<_, String>(4)?,
            "updated_at": row.get::<_, String>(5)?,
        }))
    }).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    // Fallback A: matches linked by tournament_day_id even if no recordings
    if rows.is_empty() {
        let mut stmt2 = conn.prepare(
            "SELECT id, match_id, match_number, category, created_at, updated_at
             FROM pss_matches WHERE tournament_day_id = ? ORDER BY created_at DESC LIMIT 100"
        ).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        rows = stmt2.query_map(rusqlite::params![day_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "match_id": row.get::<_, String>(1)?,
                "match_number": row.get::<_, Option<String>>(2)?,
                "category": row.get::<_, Option<String>>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "updated_at": row.get::<_, String>(5)?,
            }))
        }).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    }

    // Fallback B: latest matches overall
    if rows.is_empty() {
        let mut stmt3 = conn.prepare(
            "SELECT id, match_id, match_number, category, created_at, updated_at
             FROM pss_matches ORDER BY created_at DESC LIMIT 100"
        ).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        rows = stmt3.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "match_id": row.get::<_, String>(1)?,
                "match_number": row.get::<_, Option<String>>(2)?,
                "category": row.get::<_, Option<String>>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "updated_at": row.get::<_, String>(5)?,
            }))
        }).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    }
    Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!(rows)), error: None })
}

/// List recorded videos for a tournament day, optionally filtered by match_id (DB id)
#[tauri::command]
pub async fn ivr_list_recorded_videos(
    tournament_day_id: i64,
    match_id: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    let conn = app.database_plugin().get_connection().await?;
    let (query, params): (&str, Vec<rusqlite::types::Value>) = if let Some(mid) = match_id {
        (
            "SELECT id, match_id, event_id, tournament_id, tournament_day_id, video_type, file_path, record_directory, start_time, duration_seconds, created_at
             FROM recorded_videos WHERE tournament_day_id = ? AND match_id = ? ORDER BY start_time DESC",
            vec![rusqlite::types::Value::from(tournament_day_id), rusqlite::types::Value::from(mid)]
        )
    } else {
        (
            "SELECT id, match_id, event_id, tournament_id, tournament_day_id, video_type, file_path, record_directory, start_time, duration_seconds, created_at
             FROM recorded_videos WHERE tournament_day_id = ? ORDER BY start_time DESC",
            vec![rusqlite::types::Value::from(tournament_day_id)]
        )
    };
    let mut stmt = conn.prepare(query).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "match_id": row.get::<_, i64>(1)?,
            "event_id": row.get::<_, Option<i64>>(2)?,
            "tournament_id": row.get::<_, Option<i64>>(3)?,
            "tournament_day_id": row.get::<_, Option<i64>>(4)?,
            "video_type": row.get::<_, String>(5)?,
            "file_path": row.get::<_, Option<String>>(6)?,
            "record_directory": row.get::<_, Option<String>>(7)?,
            "start_time": row.get::<_, String>(8)?,
            "duration_seconds": row.get::<_, Option<i32>>(9)?,
            "created_at": row.get::<_, String>(10)?,
        }))
    }).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!(rows)), error: None })
}

/// Open a recorded video path directly with optional positive offset seconds
#[tauri::command]
pub async fn ivr_open_video_path(
    file_path: String,
    offset_seconds: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    let off = offset_seconds.unwrap_or(0);
    match app.open_video_at(file_path, off).await {
        Ok(()) => Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!({"opened": true})), error: None }),
        Err(e) => Ok(ObsObwsConnectionResponse{ success: false, data: None, error: Some(e.to_string()) })
    }
}

/// Open a recorded video by its DB id and compute a precise offset from event timing when available
#[tauri::command]
pub async fn ivr_open_recorded_video(
    recorded_video_id: i64,
    event_id: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    let conn = app.database_plugin().get_connection().await?;
    // Fetch recorded video info
    let (file_path_opt, _record_dir_opt, start_time_str, match_id_db, stored_event_id_opt): (Option<String>, Option<String>, String, i64, Option<i64>) = conn.query_row(
        "SELECT file_path, record_directory, start_time, match_id, event_id FROM recorded_videos WHERE id = ?",
        rusqlite::params![recorded_video_id],
        |r| Ok((r.get(0).ok(), r.get(1).ok(), r.get::<_, String>(2)?, r.get::<_, i64>(3)?, r.get(4).ok())),
    ).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    let path_to_open = if let Some(fp) = file_path_opt { fp } else {
        return Ok(ObsObwsConnectionResponse{ success: false, data: None, error: Some("Cannot open: file_path not set".to_string()) });
    };
    let start_time = chrono::DateTime::parse_from_rfc3339(&start_time_str)
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?
        .with_timezone(&chrono::Utc);
    // Determine event to use for offset
    let chosen_event_id = event_id.or(stored_event_id_opt);
    let event_time_opt: Option<chrono::DateTime<chrono::Utc>> = if let Some(eid) = chosen_event_id {
        conn.query_row(
            "SELECT timestamp FROM pss_events_v2 WHERE id = ?",
            rusqlite::params![eid],
            |r| r.get::<_, String>(0)
        ).ok().and_then(|s: String| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc)))
    } else {
        // Fallback: first event after start_time for this match
        conn.query_row(
            "SELECT timestamp FROM pss_events_v2 WHERE match_id = ? AND timestamp >= ? ORDER BY timestamp ASC LIMIT 1",
            rusqlite::params![match_id_db, start_time.to_rfc3339()],
            |r| r.get::<_, String>(0)
        ).ok().and_then(|s: String| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc)))
    };
    let offset_seconds: i64 = event_time_opt.map(|et| (et - start_time).num_seconds().max(0)).unwrap_or(0);
    // Open the video at computed offset
    match app.open_video_at(path_to_open, offset_seconds).await {
        Ok(()) => Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!({"opened": true, "offset_seconds": offset_seconds})), error: None }),
        Err(e) => Ok(ObsObwsConnectionResponse{ success: false, data: None, error: Some(e.to_string()) })
    }
}

/// Validate mpv.exe path exists and is a file
#[tauri::command]
pub async fn ivr_validate_mpv_path(mpv_path: String) -> Result<ObsObwsConnectionResponse, TauriError> {
    use std::path::Path;
    let p = Path::new(&mpv_path);
    if p.exists() && p.is_file() {
        Ok(ObsObwsConnectionResponse { success: true, data: Some(serde_json::json!({"valid": true})), error: None })
    } else {
        Ok(ObsObwsConnectionResponse { success: false, data: Some(serde_json::json!({"valid": false})), error: Some("Path does not exist or is not a file".to_string()) })
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
    let res = app.obs_obws_plugin().get_record_directory(connection_name.as_deref()).await;
    match res {
        Ok(dir) => Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!({"directory": dir})), error: None }),
        Err(e) => Ok(ObsObwsConnectionResponse{ success: false, data: None, error: Some(e.to_string()) })
    }
}

#[tauri::command]
pub async fn obs_obws_get_filename_formatting(
    connection_name: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    let res = app.obs_obws_plugin().get_filename_formatting(connection_name.as_deref()).await;
    match res {
        Ok(fmt) => Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!({"formatting": fmt})), error: None }),
        Err(e) => Ok(ObsObwsConnectionResponse{ success: false, data: None, error: Some(e.to_string()) })
    }
}

/// Add a new OBS connection using obws
#[tauri::command]
pub async fn obs_obws_add_connection(
    connection: ObsObwsConnectionRequest,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws add connection called: {}@{}:{}", connection.name, connection.host, connection.port);
    
    let config = ObsConnectionConfig {
        name: connection.name,
        host: connection.host,
        port: connection.port,
        password: connection.password,
        timeout_seconds: 30,
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
    log::info!("OBS obws update connection called: {} -> {}@{}:{}", old_name, connection.name, connection.host, connection.port);
    
    let config = ObsConnectionConfig {
        name: connection.name,
        host: connection.host,
        port: connection.port,
        password: connection.password,
        timeout_seconds: 30,
    };
    
    match app.obs_obws_plugin().update_connection(&old_name, config).await {
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
    log::info!("OBS obws connect called: {}", connection_name);
    
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
    log::info!("OBS obws disconnect called: {}", connection_name);
    
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
    log::info!("OBS obws get connection status called: {}", connection_name);
    
    match app.obs_obws_plugin().get_connection_status(&connection_name).await {
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
    log::info!("OBS obws remove connection called: {}", connection_name);
    
    match app.obs_obws_plugin().remove_connection(&connection_name).await {
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
    
    match app.obs_obws_plugin().get_status(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().start_recording(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().stop_recording(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().get_recording_status(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().start_streaming(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().stop_streaming(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().get_streaming_status(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().get_current_scene(connection_name.as_deref()).await {
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
    log::info!("OBS obws set current scene called: {}", scene_name);
    
    match app.obs_obws_plugin().set_current_scene(&scene_name, connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().get_scenes(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().get_version(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().get_stats(connection_name.as_deref()).await {
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
    _app: State<'_, Arc<App>>,  // Prefix with _ since we don't use it yet
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
    
    match app.obs_obws_plugin().start_replay_buffer(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().stop_replay_buffer(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().save_replay_buffer(connection_name.as_deref()).await {
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
    
    match app.obs_obws_plugin().get_replay_buffer_status(connection_name.as_deref()).await {
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
    _connection_name: Option<String>,
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws get recording path settings called");
    
    // For now, return a placeholder since obws doesn't have direct path configuration
    // This will be implemented using custom requests in the future
    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "recording_path": "C:/Users/Damjan/Videos",
            "recording_format": "mp4",
            "filename_pattern": "{matchNumber}_{player1}_{player2}_{date}",
            "message": "Recording path settings retrieved (placeholder)"
        })),
        error: None,
    })
}

/// Set recording path using obws
#[tauri::command]
pub async fn obs_obws_set_recording_path(
    path: String,
    _connection_name: Option<String>,
    _app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws set recording path called: {}", path);
    
    // For now, return a placeholder since obws doesn't have direct path configuration
    // This will be implemented using custom requests in the future
    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "recording_path": path,
            "message": "Recording path set successfully (placeholder)"
        })),
        error: None,
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
    log::info!("OBS obws set replay buffer path called: {}", path);
    
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
            match crate::database::operations::ObsRecordingOperations::create_recording_session(&mut *conn, &recording_session) {
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
        },
        Err(e) => Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(format!("Invalid session format: {}", e)),
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
    log::info!("OBS obws update recording session status called: {} -> {}", session_id, status);
    
    let mut conn = app.database_plugin().get_connection().await?;
    match crate::database::operations::ObsRecordingOperations::update_recording_session_status(&mut *conn, session_id, &status, None) {
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
    log::info!("OBS obws generate recording path called: match_id={}", match_id);

    // Get database connection
    let conn = app.database_plugin().get_connection().await
        .map_err(|e| TauriError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Database connection error: {}", e))))?;

    // Get active tournament and tournament day
    let tournament = crate::database::operations::TournamentOperations::get_active_tournament(&*conn)
        .map_err(|e| TauriError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get active tournament: {}", e))))?;
    
    let tournament_day = if let Some(ref tournament) = tournament {
        crate::database::operations::TournamentOperations::get_active_tournament_day(&*conn, tournament.id.unwrap())
            .map_err(|e| TauriError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get active tournament day: {}", e))))?
    } else {
        None
    };

    // Get match details
    let matches = crate::database::operations::PssUdpOperations::get_pss_matches(&*conn, Some(100))
        .map_err(|e| TauriError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get matches: {}", e))))?;
    
    let match_info = matches.into_iter()
        .find(|m| m.match_id == match_id)
        .ok_or_else(|| TauriError::from(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Match not found: {}", match_id))))?;

    // Get match athletes
    let match_athletes = crate::database::operations::PssUdpOperations::get_pss_match_athletes(&*conn, match_info.id.unwrap())
        .map_err(|e| TauriError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get match athletes: {}", e))))?;

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
            },
            2 => {
                player2_name = Some(athlete.short_name);
                player2_flag = athlete.country_code;
            },
            _ => {}
        }
    }

    // Load recording config to supply folder_pattern
    let config_for_pattern = {
        let conn = app.database_plugin().get_connection().await;
        if let Ok(conn) = conn {
            crate::database::operations::ObsRecordingOperations::get_recording_config(&*conn, "OBS_REC").ok().flatten()
        } else { None }
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
        player2_flag.clone()
    ) {
        Ok(generated_path) => {
            if let Err(e) = path_generator.ensure_directory_exists(&generated_path.directory) {
                return Ok(ObsObwsConnectionResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to create directory: {}", e)),
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
        },
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
    let directory = path_generator.generate_directory_path(&tournament_name, &tournament_day, &match_number);
    
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
    let filename = path_generator.generate_filename(&test_match_info, &tournament_name, &tournament_day);
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
    log::info!("OBS obws test recording called for connection: {}", connection_name);

    // Use the obws plugin for recording controls to avoid legacy API
    if let Err(e) = app.obs_obws_plugin().start_recording(Some(&connection_name)).await {
        return Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to start recording: {}", e)),
        });
    }

    if let Err(e) = app.obs_obws_plugin().start_replay_buffer(Some(&connection_name)).await {
        return Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to start replay buffer: {}", e)),
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
    log::info!("OBS obws apply path decision called: {} / {}", tournament_name, tournament_day);

    let handler = app.recording_event_handler();
    match handler.regenerate_path_with_overrides(tournament_name, tournament_day).await {
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
    let directory = path_generator.generate_directory_path(&tournament_name, &tournament_day, &match_number);
    
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
            let filename = path_generator.generate_filename(&test_match_info, &tournament_name, &tournament_day);
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
        },
        Err(e) => {
            Ok(ObsObwsConnectionResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to create folders: {}", e)),
            })
        }
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
    log::info!("OBS obws send config to OBS called for connection: {}", connection_name);
    // Route through obws manager to set recording directory and filename formatting
    let result = async {
        let manager = _app.obs_obws_plugin();
        manager.set_record_directory(&recording_path, Some(&connection_name)).await?;
        manager.set_filename_formatting(&filename_template, Some(&connection_name)).await?;
        crate::types::AppResult::<()>::Ok(())
    }.await;

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
        })
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

    recording_handler.clear_session()
        .map_err(|e| TauriError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to clear session: {}", e))))?;

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
    log::info!("OBS obws manual start recording called: match_id={}, connection={}", match_id, obs_connection_name);

    // Get the recording event handler from the app state
    let recording_handler = app.recording_event_handler();

    // Create a manual recording session
    let session = crate::plugins::obs_obws::RecordingSession {
        id: None,
        match_id: match_id.clone(),
        tournament_name: None,
        tournament_day: None,
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
        log::error!("Failed to generate recording path: {}", e);
        recording_handler.update_session_state(crate::plugins::obs_obws::RecordingState::Error(e.to_string())).await
            .map_err(|e| TauriError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to update session state: {}", e))))?;
        
        return Ok(ObsObwsConnectionResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to generate recording path: {}", e)),
        });
    }

    // Start recording
    recording_handler.update_session_state(crate::plugins::obs_obws::RecordingState::Recording).await
        .map_err(|e| TauriError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to update session state: {}", e))))?;

    // Start recording immediately via obws (authoritative) to avoid depending on event consumers
    if let Err(e) = app.obs_obws_plugin().start_recording(Some(&obs_connection_name)).await {
        log::error!("Failed to start recording via obws: {}", e);
        return Ok(ObsObwsConnectionResponse { success: false, data: None, error: Some(e.to_string()) });
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
    log::info!("OBS obws manual stop recording called: connection={}", obs_connection_name);

    // Get the recording event handler from the app state
    let recording_handler = app.recording_event_handler();

    // Update session state to stopping
    recording_handler.update_session_state(crate::plugins::obs_obws::RecordingState::Stopping).await
        .map_err(|e| TauriError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to update session state: {}", e))))?;

    // Stop recording immediately via obws
    if let Err(e) = app.obs_obws_plugin().stop_recording(Some(&obs_connection_name)).await {
        log::error!("Failed to stop recording via obws: {}", e);
        return Ok(ObsObwsConnectionResponse { success: false, data: None, error: Some(e.to_string()) });
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
    #[serde(default)] replay_buffer_duration: Option<u32>,
    #[serde(default = "default_true")] auto_stop_on_match_end: bool,
    #[serde(default = "default_true")] auto_stop_on_winner: bool,
    // Right column (auto-start) flags
    #[serde(default = "default_true")] auto_start_recording_on_match_begin: bool,
    #[serde(default = "default_true")] auto_start_replay_on_match_begin: bool,
}

fn default_folder_pattern() -> String { "{tournament}/{tournamentDay}".to_string() }
fn default_true() -> bool { true }

#[tauri::command]
pub async fn obs_obws_save_full_config(
    payload: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    // Parse payload
    let cfg: FullObsConfigPayload = serde_json::from_value(payload)
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("Invalid full config payload: {}", e))))?;

    println!(
        "💾 obs_obws_save_full_config(conn='{}', path='{}', fmt='{}', tmpl='{}', enabled={}, stop_delay={}, include_rb={}, rb_dur={:?}, stop_on_end={}, stop_on_winner={}, start_rec={}, start_replay={})",
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
    crate::database::operations::ObsRecordingOperations::upsert_recording_config(&mut *conn, &recording_config)
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
    recording_handler.update_config(new_auto_cfg)
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("Failed to update handler config: {}", e))))?;

    use crate::database::operations::UiSettingsOperations as UIOps;
    UIOps::set_ui_setting(&mut *conn, "obs.auto.enabled", if cfg.enabled {"true"} else {"false"}, "user", Some("save full config"))
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(&mut *conn, "obs.auto.connection", &cfg.connection_name, "user", Some("save full config"))
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(&mut *conn, "obs.auto.stop_delay_seconds", &cfg.stop_delay_seconds.to_string(), "user", Some("save full config"))
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(&mut *conn, "obs.auto.include_replay_buffer", if cfg.include_replay_buffer {"true"} else {"false"}, "user", Some("save full config"))
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(&mut *conn, "obs.auto.stop_on_match_end", if cfg.auto_stop_on_match_end {"true"} else {"false"}, "user", Some("save full config"))
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(&mut *conn, "obs.auto.stop_on_winner", if cfg.auto_stop_on_winner {"true"} else {"false"}, "user", Some("save full config"))
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(&mut *conn, "obs.auto.start_recording_on_match_begin", if cfg.auto_start_recording_on_match_begin {"true"} else {"false"}, "user", Some("save full config"))
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    UIOps::set_ui_setting(&mut *conn, "obs.auto.start_replay_on_match_begin", if cfg.auto_start_replay_on_match_begin {"true"} else {"false"}, "user", Some("save full config"))
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    // removed: save replay on match end persistence

    // Also return live connection status for the selected connection so the UI doesn't reset indicator
    let live_status = app.obs_obws_plugin().get_connection_status(&cfg.connection_name).await.ok();
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
    use crate::database::operations::{ObsRecordingOperations as RecOps, UiSettingsOperations as UIOps};

    let conn = app.database_plugin().get_connection().await?;

    // Resolve connection name: param -> UiSettings -> default
    let resolved_conn = connection_name
        .or(UIOps::get_ui_setting(&*conn, "obs.auto.connection").ok().flatten())
        .unwrap_or_else(|| "OBS_REC".to_string());

    // Ensure keys exist so get_ui_setting won't fail on fresh DBs
    let _ = UIOps::ensure_key(&*conn, "obs.auto.enabled", "OBS Auto Enabled", "boolean", Some("false"));
    let _ = UIOps::ensure_key(&*conn, "obs.auto.connection", "OBS Auto Connection", "string", Some(&resolved_conn));
    let _ = UIOps::ensure_key(&*conn, "obs.auto.stop_delay_seconds", "OBS Stop Delay", "integer", Some("30"));
    let _ = UIOps::ensure_key(&*conn, "obs.auto.include_replay_buffer", "Include Replay Buffer", "boolean", Some("true"));
    let _ = UIOps::ensure_key(&*conn, "obs.auto.start_recording_on_match_begin", "Auto-start Recording", "boolean", Some("true"));
    let _ = UIOps::ensure_key(&*conn, "obs.auto.start_replay_on_match_begin", "Auto-start Replay", "boolean", Some("true"));
    let _ = UIOps::ensure_key(&*conn, "obs.auto.save_replay_on_match_end", "Save Replay On End", "boolean", Some("false"));
    let _ = UIOps::ensure_key(&*conn, "obs.auto.stop_on_match_end", "Stop On Match End", "boolean", Some("true"));
    let _ = UIOps::ensure_key(&*conn, "obs.auto.stop_on_winner", "Stop On Winner", "boolean", Some("true"));

    // Recording config
    let rec_cfg = RecOps::get_recording_config(&*conn, &resolved_conn)
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    // Automatic config from UiSettings (fallbacks applied)
    let enabled = UIOps::get_ui_setting(&*conn, "obs.auto.enabled").ok().flatten().map(|v| v=="true").unwrap_or(false);
    let stop_delay_seconds = UIOps::get_ui_setting(&*conn, "obs.auto.stop_delay_seconds").ok().flatten().and_then(|s| s.parse::<u32>().ok()).unwrap_or(30);
    let include_replay_buffer = UIOps::get_ui_setting(&*conn, "obs.auto.include_replay_buffer").ok().flatten().map(|v| v=="true").unwrap_or(true);
    let auto_start_recording_on_match_begin = UIOps::get_ui_setting(&*conn, "obs.auto.start_recording_on_match_begin").ok().flatten().map(|v| v=="true").unwrap_or(true);
    let auto_start_replay_on_match_begin = UIOps::get_ui_setting(&*conn, "obs.auto.start_replay_on_match_begin").ok().flatten().map(|v| v=="true").unwrap_or(true);
    let save_replay_on_match_end = false;
    let auto_stop_on_match_end = UIOps::get_ui_setting(&*conn, "obs.auto.stop_on_match_end").ok().flatten().map(|v| v=="true").unwrap_or(true);
    let auto_stop_on_winner = UIOps::get_ui_setting(&*conn, "obs.auto.stop_on_winner").ok().flatten().map(|v| v=="true").unwrap_or(true);

    println!("📥 obs_obws_get_full_config(conn='{}')", resolved_conn);

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

// ============================================================================
// IVR Match History - Actions (Delete, Upload, Import)
// ============================================================================

/// Delete recorded videos by IDs: removes DB rows and local files
#[tauri::command]
pub async fn ivr_delete_recorded_videos(ids: Vec<i64>, app: State<'_, Arc<App>>) -> Result<ObsObwsConnectionResponse, TauriError> {
    use std::fs;
    let mut deleted: Vec<i64> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let conn = app.database_plugin().get_connection().await?;
    for id in ids.iter() {
        let row = conn.query_row(
            "SELECT file_path, record_directory FROM recorded_videos WHERE id = ?",
            rusqlite::params![id],
            |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<String>>(1)?))
        ).ok();
        if let Some((file_path, _dir)) = row {
            if let Some(fp) = file_path {
                if let Err(e) = fs::remove_file(&fp) { errors.push(format!("{}", e)); }
            }
            let _ = conn.execute("DELETE FROM recorded_videos WHERE id = ?", rusqlite::params![id]);
            deleted.push(*id);
        } else {
            // Row not found; still push as deleted from UI standpoint
            let _ = conn.execute("DELETE FROM recorded_videos WHERE id = ?", rusqlite::params![id]);
            deleted.push(*id);
        }
    }
    Ok(ObsObwsConnectionResponse{ success: errors.is_empty(), data: Some(serde_json::json!({"deleted": deleted})), error: if errors.is_empty(){None}else{Some(errors.join("; "))} })
}

/// Upload selected recorded videos to Google Drive by zipping them first
#[tauri::command]
pub async fn ivr_upload_recorded_videos(ids: Vec<i64>, app: State<'_, Arc<App>>, window: tauri::Window, folder_id: Option<String>) -> Result<ObsObwsConnectionResponse, TauriError> {
    let conn = app.database_plugin().get_connection().await?;
    let mut paths: Vec<String> = Vec::new();
    for id in ids.iter() {
        let row = conn.query_row(
            "SELECT file_path FROM recorded_videos WHERE id = ?",
            rusqlite::params![id],
            |r| r.get::<_, Option<String>>(0)
        ).ok().flatten();
        if let Some(fp) = row { paths.push(fp); }
    }
    if paths.is_empty() {
        return Ok(ObsObwsConnectionResponse{ success: false, data: None, error: Some("No files to upload".to_string()) });
    }
    // Create a zip in temp dir
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let zip_path = std::env::temp_dir().join(format!("ivr_videos_{}.zip", ts));
    {
        use std::io::Write;
        let file = std::fs::File::create(&zip_path).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let total = paths.len() as u64;
        for (idx, p) in paths.iter().enumerate() {
            let name_in_zip = std::path::Path::new(p).file_name().and_then(|s| s.to_str()).unwrap_or("video.mp4");
            zip.start_file(name_in_zip, options).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
            let bytes = std::fs::read(p).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
            zip.write_all(&bytes).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
            let _ = window.emit("ivr_zip_progress", serde_json::json!({
                "phase":"zipping","items_done": idx+1, "items_total": total, "file": name_in_zip
            }));
        }
        zip.finish().map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    }
    let file_name = zip_path.file_name().and_then(|s| s.to_str()).unwrap_or("ivr_videos.zip").to_string();
    let _ = window.emit("ivr_upload_progress", serde_json::json!({"phase":"starting","file": file_name}));
    let file_id = if let Some(fid) = folder_id.as_ref() {
        crate::plugins::drive_plugin().upload_file_streaming_to_folder(&zip_path, &file_name, Some(fid.as_str())).await
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?
    } else {
        crate::plugins::drive_plugin().upload_file_streaming(&zip_path, &file_name).await
            .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?
    };
    let _ = window.emit("ivr_upload_progress", serde_json::json!({"phase":"complete","file": file_name, "file_id": file_id}));
    Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!({"zip_path": zip_path.to_string_lossy().to_string(), "file_id": file_id})), error: None })
}

/// Import recordings from local zip or Drive into a tournament day directory and index them
#[tauri::command]
pub async fn ivr_import_recorded_videos(
    source: String,
    path_or_id: String,
    tournament_day_id: i64,
    match_id: i64,
    app: State<'_, Arc<App>>,
    window: tauri::Window,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    // Resolve videos root and target directory
    let conn = app.database_plugin().get_connection().await?;
    let (tournament_id, day_number): (i64, i32) = conn.query_row(
        "SELECT tournament_id, day_number FROM tournament_days WHERE id = ?",
        rusqlite::params![tournament_day_id],
        |r| Ok((r.get(0)?, r.get(1)?))
    ).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    let tournament_name: String = conn.query_row(
        "SELECT name FROM tournaments WHERE id = ?",
        rusqlite::params![tournament_id],
        |r| r.get(0)
    ).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    let videos_root: std::path::PathBuf = {
        use crate::database::operations::ObsRecordingOperations as RecOps;
        let resolved_conn_name = "OBS_REC".to_string();
        if let Ok(Some(cfg)) = RecOps::get_recording_config(&*conn, &resolved_conn_name) {
            std::path::PathBuf::from(cfg.recording_root_path)
        } else {
            crate::plugins::obs_obws::PathGeneratorConfig::detect_windows_videos_folder()
        }
    };
    let target_dir = videos_root.join(&tournament_name).join(format!("Day {}", day_number));
    std::fs::create_dir_all(&target_dir).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;

    // Download if Drive
    let zip_path = if source.to_lowercase() == "drive" {
        let _ = window.emit("ivr_download_progress", serde_json::json!({"phase":"starting","id": path_or_id}));
        let tmp = std::env::temp_dir().join(format!("ivr_import_{}.zip", chrono::Utc::now().timestamp()));
        match crate::plugins::drive_plugin().download_backup_archive(&path_or_id).await {
            Ok(_) => tmp,
            Err(e) => return Ok(ObsObwsConnectionResponse{ success: false, data: None, error: Some(format!("Failed to download from Drive: {}", e)) })
        }
    } else {
        let p = std::path::PathBuf::from(&path_or_id);
        if !p.is_file() { return Ok(ObsObwsConnectionResponse{ success: false, data: None, error: Some("Zip file not found".to_string()) }); }
        p
    };

    // Extract zip
    let file = std::fs::File::open(&zip_path).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    let mut imported: Vec<String> = Vec::new();
    let total = archive.len();
    for i in 0..total {
        let mut zf = archive.by_index(i).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        if zf.is_dir() { continue; }
        let name = zf.name().to_string();
        let lower = name.to_lowercase();
        if !(lower.ends_with(".mp4") || lower.ends_with(".mkv") || lower.ends_with(".mov") || lower.ends_with(".avi")) { continue; }
        let fname = std::path::Path::new(&name).file_name().and_then(|s| s.to_str()).unwrap_or("video.mp4");
        let out_path = target_dir.join(fname);
        let mut out = std::fs::File::create(&out_path).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        std::io::copy(&mut zf, &mut out).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        imported.push(out_path.to_string_lossy().to_string());
        let _ = window.emit("ivr_extract_progress", serde_json::json!({"phase":"file","done": imported.len(), "total": total, "file": fname}));

        // Metadata
        let file_size = std::fs::metadata(&out_path).ok().map(|m| m.len() as i64);
        let checksum = None::<String>; // optional: compute later

        // Insert into recorded_videos
        let start_time = chrono::Utc::now();
        let record_directory = target_dir.to_string_lossy().to_string();
        let file_path_str = out_path.to_string_lossy().to_string();
        conn.execute(
            "INSERT INTO recorded_videos (match_id, event_id, tournament_id, tournament_day_id, video_type, file_path, record_directory, filename_formatting, start_time, duration_seconds, file_size, checksum, created_at)
             VALUES (?, NULL, ?, ?, 'recording', ?, ?, NULL, ?, NULL, ?, ?, ?)",
            rusqlite::params![ match_id, tournament_id, tournament_day_id, file_path_str, record_directory, start_time.to_rfc3339(), file_size, checksum, chrono::Utc::now().to_rfc3339() ]
        ).map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
        let rvid = conn.last_insert_rowid();

        // Link events within window using nearest event after start_time
        let _ = conn.execute(
            "INSERT OR IGNORE INTO recorded_video_events (recorded_video_id, event_id, offset_ms, created_at)
             SELECT ?, e.id, CAST((julianday(e.timestamp) - julianday(?)) * 86400000 AS INTEGER), ?
             FROM pss_events_v2 e WHERE e.match_id = ? AND e.timestamp >= ? ORDER BY e.timestamp ASC LIMIT 1",
            rusqlite::params![ rvid, start_time.to_rfc3339(), chrono::Utc::now().to_rfc3339(), match_id, start_time.to_rfc3339() ]
        );
    }

    let _ = window.emit("ivr_index_progress", serde_json::json!({"phase":"complete","count": imported.len()}));
    Ok(ObsObwsConnectionResponse{ success: true, data: Some(serde_json::json!({"imported": imported})), error: None })
}
