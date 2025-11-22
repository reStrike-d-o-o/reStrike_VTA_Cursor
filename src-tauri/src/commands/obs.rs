//! Tauri commands for OBS WebSocket integration using obws crate

use crate::core::app::App;
use std::sync::Arc;
use tauri::{Error as TauriError, State};
pub use crate::tauri_commands_obws::{
    obs_obws_connect, obs_obws_disconnect,
    obs_obws_get_connection_status, obs_obws_get_connections, obs_obws_get_current_scene,
    obs_obws_get_recording_path_settings, obs_obws_get_replay_buffer_status,
    obs_obws_get_status, obs_obws_get_streaming_status, obs_obws_get_version,
    obs_obws_remove_connection, obs_obws_save_replay_buffer, obs_obws_set_current_scene,
    obs_obws_set_recording_filename, obs_obws_set_recording_path,
    obs_obws_set_source_visibility, obs_obws_start_recording, obs_obws_start_replay_buffer,
    obs_obws_start_streaming, obs_obws_stop_recording, obs_obws_stop_replay_buffer,
    obs_obws_stop_streaming, obs_obws_create_test_folders, obs_obws_manual_start_recording,
    obs_obws_manual_stop_recording, obs_obws_generate_recording_path,
    obs_obws_setup_status_listener, obs_obws_save_full_config,
    obs_obws_get_filename_formatting, ivr_match_history_snapshot, ivr_round_replay_now,
    ivr_save_replay_settings, ivr_open_event_video, ivr_validate_mpv_path,
    obs_obws_get_record_directory, ivr_open_video_file, obs_obws_add_connection,
};

use crate::tauri_commands_obws::{
    obs_obws_add_connection as impl_obs_obws_add_connection,
    obs_obws_update_connection as impl_obs_obws_update_connection,
    ObsObwsConnectionRequest, ObsObwsConnectionResponse,
};



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



// ============================================================================
// OBS Command Wrappers (Frontend Compatibility)
// ============================================================================

#[tauri::command(rename_all = "snake_case", rename = "obs_connect")]
pub async fn obs_connect_impl(
    url: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS connect called with URL: {url}");
    crate::utils::ensure_license_ok(&app).await?;
    // Parse basic info
    let host = url
        .replace("ws://", "")
        .replace("wss://", "")
        .split(':')
        .next()
        .unwrap_or("localhost")
        .to_string();
    // Create connection via obws, name default to OBS_REC if not provided in UI
    let req = ObsObwsConnectionRequest {
        name: "OBS_REC".to_string(),
        host,
        port: 4455,
        password: None,
        enabled: true,
    };
    let _ = obs_obws_add_connection(req, app.clone()).await;
    let _ = obs_obws_connect("OBS_REC".to_string(), app.clone()).await;
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
    log::info!("OBS add connection called: {name}@{host}:{port}");
    crate::utils::ensure_license_ok(&app).await?;
    // Delegate to obws add_connection
    let req = ObsObwsConnectionRequest {
        name: name.clone(),
        host: host.clone(),
        port,
        password: password.clone(),
        enabled,
    };
    let res = impl_obs_obws_add_connection(req, app.clone()).await;
    res?;
    // Persist to config as before
    let config_conn = crate::config::types::ObsConnectionConfig {
        name: name.clone(),
        host: host.clone(),
        port,
        password: password.clone(),
        protocol_version: "v5".to_string(),
        enabled,
        timeout_seconds: 30,
        auto_reconnect: true,
        max_reconnect_attempts: 5,
    };
    let mut connections = app.config_manager().get_obs_connections().await;
    connections.retain(|c| c.name != config_conn.name);
    connections.push(config_conn);
    let _ = app
        .config_manager()
        .update_obs_connections(connections)
        .await;
    Ok(serde_json::json!({ "success": true, "message": "OBS connection added successfully" }))
}

/// Update an existing OBS connection using obws
/// Update an existing OBS connection using obws
#[tauri::command(rename = "obs_obws_update_connection")]
pub async fn obs_update_connection_wrapper(
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

    let req = ObsObwsConnectionRequest {
        name: connection.name.clone(),
        host: connection.host.clone(),
        port: connection.port,
        password: connection.password.clone(),
        enabled: connection.enabled,
    };

    // Call the obws command
    let res = impl_obs_obws_update_connection(old_name.clone(), req, app.clone()).await;
    res?;

    // Update config manager
    let config_conn = crate::config::types::ObsConnectionConfig {
        name: connection.name.clone(),
        host: connection.host.clone(),
        port: connection.port,
        password: connection.password.clone(),
        protocol_version: "v5".to_string(),
        enabled: connection.enabled,
        timeout_seconds: 30,
        auto_reconnect: true,
        max_reconnect_attempts: 5,
    };

    let mut connections = app.config_manager().get_obs_connections().await;
    connections.retain(|c| c.name != old_name);
    connections.push(config_conn);
    let _ = app
        .config_manager()
        .update_obs_connections(connections)
        .await;

    Ok(ObsObwsConnectionResponse {
        success: true,
        data: Some(serde_json::json!({
            "message": "OBS connection updated successfully"
        })),
        error: None,
    })
}

#[tauri::command]
pub async fn obs_connect_to_connection(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS connect to connection called: {connection_name}");
    crate::utils::ensure_license_ok(&app).await?;
    let res = obs_obws_connect(connection_name, app.clone()).await;
    match res {
        Ok(_) => Ok(serde_json::json!({ "success": true })),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn obs_get_connection_status(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get connection status called: {connection_name}");
    let res = obs_obws_get_connection_status(connection_name, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_get_connections(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get connections called");
    let res = obs_obws_get_connections(app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_disconnect(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS disconnect called for connection: '{connection_name}'");
    if connection_name.is_empty() {
        return Err(TauriError::from(anyhow::anyhow!(
            "Connection name cannot be empty"
        )));
    }
    let res = obs_obws_disconnect(connection_name, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_remove_connection(
    connection_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS remove connection called for connection: {connection_name}");
    let _ = obs_obws_remove_connection(connection_name.clone(), app.clone()).await?;
    // Remove from configuration manager
    let mut connections = app.config_manager().get_obs_connections().await;
    connections.retain(|c| c.name != connection_name);
    let _ = app
        .config_manager()
        .update_obs_connections(connections)
        .await;
    Ok(serde_json::json!({ "success": true, "message": "OBS connection removed" }))
}

#[tauri::command]
pub async fn obs_get_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get status");
    let res = obs_obws_get_status(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_start_recording(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS start recording called");
    crate::utils::ensure_license_ok(&app).await?;
    let res = obs_obws_start_recording(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_stop_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS stop recording called");
    crate::utils::ensure_license_ok(&app).await?;
    let res = obs_obws_stop_recording(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_start_streaming(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS start streaming called");
    crate::utils::ensure_license_ok(&app).await?;
    let res = obs_obws_start_streaming(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_stop_streaming(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS stop streaming called");
    crate::utils::ensure_license_ok(&app).await?;
    let res = obs_obws_stop_streaming(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_get_streaming_status(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get streaming status called");
    let res = obs_obws_get_streaming_status(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_get_current_scene(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get current scene");
    let res = obs_obws_get_current_scene(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_set_current_scene(
    scene_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS set current scene: {scene_name}");
    let res = obs_obws_set_current_scene(scene_name, None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_get_obs_version(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get version called");
    let res = obs_obws_get_version(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_get_replay_buffer_status(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS get replay buffer status called");
    let res = obs_obws_get_replay_buffer_status(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_start_replay_buffer(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS start replay buffer called");
    crate::utils::ensure_license_ok(&app).await?;
    let res = obs_obws_start_replay_buffer(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}

#[tauri::command]
pub async fn obs_stop_replay_buffer(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("OBS stop replay buffer called");
    crate::utils::ensure_license_ok(&app).await?;
    let res = obs_obws_stop_replay_buffer(None, app.clone()).await?;
    Ok(serde_json::json!({ "success": res.success, "data": res.data, "error": res.error }))
}



// ============================================================================
// OBS Profile Read-backs
// ============================================================================







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
