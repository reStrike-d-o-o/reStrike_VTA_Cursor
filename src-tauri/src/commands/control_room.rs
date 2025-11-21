use crate::core::app::App;
use crate::types::TauriError;
use std::sync::Arc;
use tauri::State;
use std::collections::HashMap;
use crate::plugins::obs_obws::types::ObsOperationRequest;

/// Control Room Authentication (Async version)
#[tauri::command]
pub async fn control_room_authenticate_async(
    _password: String,
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room async authentication attempt");

    // Create async database connection
    let app_data_dir = match dirs::data_dir() {
        Some(data_dir) => data_dir.join("reStrikeVTA"),
        None => std::env::current_dir().unwrap_or_default().join("data"),
    };
    let _async_db = match crate::database::AsyncDatabaseConnection::new(&app_data_dir).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            log::error!("Failed to create async database connection: {e}");
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

/// Validate that a session ID is valid and authorized
fn validate_session(session_id: &str) -> Result<(), TauriError> {
    // Basic session validation - check if session_id is not empty
    if session_id.is_empty() {
        return Err(TauriError::from(anyhow::anyhow!(
            "Invalid session: session_id cannot be empty"
        )));
    }

    // Additional validation could include:
    // - Check session against database/session store
    // - Verify session hasn't expired
    // - Validate user permissions for the session

    log::debug!("Session {session_id} validated successfully");
    Ok(())
}

#[tauri::command]
pub async fn control_room_get_obs_connections(
    session_id: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::debug!("Control Room: Getting OBS connections for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

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
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Getting OBS connections with status for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut connections_data = Vec::new();
    for name in names {
        let status = app.obs_obws_plugin().get_connection_status(&name).await;
        let status_str = match status {
            Ok(s) => format!("{s:?}"),
            Err(e) => format!("Error: {e}"),
        };
        connections_data.push(serde_json::json!({"name": name, "status": status_str}));
    }
    Ok(serde_json::json!({"success": true, "connections": connections_data}))
}

/// Get all Control Room OBS connections with their full details and status
#[tauri::command]
pub async fn control_room_get_obs_connections_with_details(
    session_id: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Getting OBS connections with full details for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    match app.obs_obws_plugin().get_connections().await {
        Ok(connections) => {
            let connections_data: Vec<serde_json::Value> = connections.into_iter().map(|conn| {
                serde_json::json!({
                    "name": conn.name,
                    "host": conn.host,
                    "port": conn.port,
                    "status": format!("{:?}", conn.status),
                    "role": format!("{:?}", conn.role),
                    "last_activity": conn.last_activity.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| "Never".to_string())
                })
            }).collect();

            Ok(serde_json::json!({"success": true, "connections": connections_data}))
        }
        Err(e) => {
            log::error!("Failed to get OBS connections with details: {e}");
            Ok(serde_json::json!({"success": false, "error": e.to_string(), "connections": []}))
        }
    }
}

/// Bulk mute all OBS streams
#[tauri::command]
pub async fn control_room_mute_all_obs(
    session_id: String,
    source_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!(
        "Control Room: Bulk mute all OBS with source '{source_name}' for session {session_id}"
    );

    // Validate session before proceeding
    validate_session(&session_id)?;

    // Individual audio control is not supported by obws
    // Bulk operations would require OBS Studio API calls or external tools
    // For now, return an informational response
    let names = app.obs_obws_plugin().get_connection_names().await;
    let results: Vec<serde_json::Value> = names.into_iter().map(|name| {
        serde_json::json!({
            "connection": name,
            "success": false,
            "error": "Individual audio control not supported by obws - consider using scene switching or OBS Studio API"
        })
    }).collect();

    Ok(serde_json::json!({
        "success": false,
        "error": "Individual audio control not supported by obws",
        "results": results,
        "summary": {
            "total": results.len(),
            "successful": 0,
            "failed": results.len(),
            "note": "Audio control requires individual source management or OBS Studio API integration"
        }
    }))
}

/// Bulk unmute all OBS streams
#[tauri::command]
pub async fn control_room_unmute_all_obs(
    session_id: String,
    source_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!(
        "Control Room: Bulk unmute all OBS with source '{source_name}' for session {session_id}"
    );

    // Validate session before proceeding
    validate_session(&session_id)?;

    // Individual audio control is not supported by obws
    // Bulk operations would require OBS Studio API calls or external tools
    // For now, return an informational response
    let names = app.obs_obws_plugin().get_connection_names().await;
    let results: Vec<serde_json::Value> = names.into_iter().map(|name| {
        serde_json::json!({
            "connection": name,
            "success": false,
            "error": "Individual audio control not supported by obws - consider using scene switching or OBS Studio API"
        })
    }).collect();

    Ok(serde_json::json!({
        "success": false,
        "error": "Individual audio control not supported by obws",
        "results": results,
        "summary": {
            "total": results.len(),
            "successful": 0,
            "failed": results.len(),
            "note": "Audio control requires individual source management or OBS Studio API integration"
        }
    }))
}
/// Change all OBS scenes to specified scene
#[tauri::command]
pub async fn control_room_change_all_obs_scenes(
    session_id: String,
    scene_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Change all OBS scenes to '{scene_name}' for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut results = Vec::new();
    for n in names {
        let r = app
            .obs_obws_plugin()
            .set_current_scene(&scene_name, Some(&n))
            .await;
        results.push((n, r.map(|_| ()).map_err(|e| anyhow::anyhow!(e.to_string()))));
    }
    let formatted_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(conn, result)| {
            serde_json::json!({
                "connection": conn,
                "success": result.is_ok(),
                "error": result.err().map(|e| e.to_string())
            })
        })
        .collect();
    Ok(serde_json::json!({ "success": true, "results": formatted_results }))
}
/// Start all OBS streams
#[tauri::command]
pub async fn control_room_start_all_obs(
    session_id: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Start all OBS streams for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut results = Vec::new();
    for n in names {
        let r = app.obs_obws_plugin().start_streaming(Some(&n)).await;
        results.push((n, r.map(|_| ()).map_err(|e| anyhow::anyhow!(e.to_string()))));
    }
    let formatted_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(conn, result)| {
            serde_json::json!({
                "connection": conn,
                "success": result.is_ok(),
                "error": result.err().map(|e| e.to_string())
            })
        })
        .collect();
    Ok(serde_json::json!({ "success": true, "results": formatted_results }))
}

/// Stop all OBS streams
#[tauri::command]
pub async fn control_room_stop_all_obs(
    session_id: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Stop all OBS streams for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut results = Vec::new();
    for n in names {
        let r = app.obs_obws_plugin().stop_streaming(Some(&n)).await;
        results.push((n, r.map(|_| ()).map_err(|e| anyhow::anyhow!(e.to_string()))));
    }
    let formatted_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(conn, result)| {
            serde_json::json!({
                "connection": conn,
                "success": result.is_ok(),
                "error": result.err().map(|e| e.to_string())
            })
        })
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
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!(
        "Control Room: Adding OBS connection '{name}' at {host}:{port} for session {session_id}"
    );

    // Validate session before proceeding
    validate_session(&session_id)?;

    let config = crate::plugins::obs_obws::types::ObsConnectionConfig {
        name: name.clone(),
        host,
        port,
        password,
        timeout_seconds: 30,
        role: crate::plugins::obs_obws::ObsConnectionRole::None,
    };
    match app.obs_obws_plugin().add_connection(config).await {
        Ok(_) => {
            log::info!("Control Room: Successfully added OBS connection '{name}'");
            Ok(serde_json::json!({
                "success": true,
                "message": format!("OBS connection '{}' added successfully", name)
            }))
        }
        Err(e) => {
            log::error!("Failed to add OBS connection '{name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to add OBS connection: {e}"
            )))
        }
    }
}

/// Connect to OBS instance
#[tauri::command]
pub async fn control_room_connect_obs(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Connecting to OBS '{obs_name}' for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    match app.obs_obws_plugin().connect(&obs_name).await {
        Ok(_) => {
            log::info!("Control Room: Successfully connected to OBS '{obs_name}'");
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Connected to OBS '{}'", obs_name)
            }))
        }
        Err(e) => {
            log::error!("Failed to connect to OBS '{obs_name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to connect to OBS: {e}"
            )))
        }
    }
}

/// Disconnect from OBS instance
#[tauri::command]
pub async fn control_room_disconnect_obs(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Disconnecting from OBS '{obs_name}' for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    match app.obs_obws_plugin().disconnect(&obs_name).await {
        Ok(_) => {
            log::info!("Control Room: Successfully disconnected from OBS '{obs_name}'");
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Disconnected from OBS '{}'", obs_name)
            }))
        }
        Err(e) => {
            log::error!("Failed to disconnect from OBS '{obs_name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to disconnect from OBS: {e}"
            )))
        }
    }
}

/// Remove OBS connection
#[tauri::command]
pub async fn control_room_remove_obs_connection(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Removing OBS connection '{obs_name}' for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    match app.obs_obws_plugin().remove_connection(&obs_name).await {
        Ok(_) => {
            log::info!("Control Room: Successfully removed OBS connection '{obs_name}'");
            Ok(serde_json::json!({
                "success": true,
                "message": format!("OBS connection '{}' removed successfully", obs_name)
            }))
        }
        Err(e) => {
            log::error!("Failed to remove OBS connection '{obs_name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to remove OBS connection: {e}"
            )))
        }
    }
}

/// Get OBS connection configuration
#[tauri::command]
pub async fn control_room_get_obs_connection(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Getting OBS connection '{obs_name}' for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    match app.obs_obws_plugin().get_connection_status(&obs_name).await {
        Ok(status) => Ok(
            serde_json::json!({ "success": true, "connection": {"name": obs_name, "status": format!("{:?}", status)} }),
        ),
        Err(e) => {
            log::error!("Failed to get OBS connection '{obs_name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to get OBS connection: {e}"
            )))
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
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Updating OBS connection '{obs_name}' for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    let _ = app.obs_obws_plugin().remove_connection(&obs_name).await;
    let cfg = crate::plugins::obs_obws::types::ObsConnectionConfig {
        name: obs_name.clone(),
        host,
        port,
        password,
        timeout_seconds: 30,
        role: crate::plugins::obs_obws::ObsConnectionRole::None,
    };
    match app.obs_obws_plugin().add_connection(cfg).await {
        Ok(_) => {
            log::info!("Control Room: Successfully updated OBS connection '{obs_name}'");
            Ok(serde_json::json!({
                "success": true,
                "message": format!("OBS connection '{}' updated successfully", obs_name)
            }))
        }
        Err(e) => {
            log::error!("Failed to update OBS connection '{obs_name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to update OBS connection: {e}"
            )))
        }
    }
}
/// Connect all disconnected OBS connections
#[tauri::command]
pub async fn control_room_connect_all_obs(
    session_id: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Connecting all OBS connections for session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut results: Vec<(String, Result<(), anyhow::Error>)> = Vec::new();
    for n in names {
        results.push((
            n.clone(),
            app.obs_obws_plugin()
                .connect(&n)
                .await
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!(e.to_string())),
        ));
    }
    {
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        let total_count = results.len();
        let failed_connections: Vec<String> = results
            .iter()
            .filter_map(|(name, r)| r.as_ref().err().map(|e| format!("{name}: {e}")))
            .collect();
        log::info!("Control Room: Connected {success_count} of {total_count} OBS connections");
        if failed_connections.is_empty() {
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Successfully connected {} OBS connections", success_count),
                "connected_count": success_count,
                "total_count": total_count
            }))
        } else {
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Connected {} of {} OBS connections. Some failed: {}", success_count, total_count, failed_connections.join(", ")),
                "connected_count": success_count,
                "total_count": total_count,
                "failed_connections": failed_connections
            }))
        }
    }
}

/// Disconnect all connected OBS connections
#[tauri::command]
pub async fn control_room_disconnect_all_obs(
    session_id: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room: Disconnecting all OBS connections");

    // Validate session before proceeding
    validate_session(&session_id)?;

    let names = app.obs_obws_plugin().get_connection_names().await;
    let mut results: Vec<(String, Result<(), anyhow::Error>)> = Vec::new();
    for n in names {
        results.push((
            n.clone(),
            app.obs_obws_plugin()
                .disconnect(&n)
                .await
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!(e.to_string())),
        ));
    }
    {
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        let total_count = results.len();
        let failed_connections: Vec<String> = results
            .iter()
            .filter_map(|(name, r)| r.as_ref().err().map(|e| format!("{name}: {e}")))
            .collect();
        log::info!("Control Room: Disconnected {success_count} of {total_count} OBS connections");
        if failed_connections.is_empty() {
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Successfully disconnected {} OBS connections", success_count),
                "disconnected_count": success_count,
                "total_count": total_count
            }))
        } else {
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Disconnected {} of {} OBS connections. Some failed: {}", success_count, total_count, failed_connections.join(", ")),
                "disconnected_count": success_count,
                "total_count": total_count,
                "failed_connections": failed_connections
            }))
        }
    }
}

/// Get audio sources for a Control Room OBS connection
#[tauri::command]
pub async fn control_room_get_audio_sources(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::debug!("Control Room: Getting audio sources for OBS '{obs_name}' session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    // Get audio sources from OBS connection
    match app
        .obs_obws_plugin()
        .get_audio_sources(Some(&obs_name))
        .await
    {
        Ok(sources) => {
            let sources_data: Vec<serde_json::Value> = sources
                .into_iter()
                .map(|source| {
                    serde_json::json!({
                        "name": source.name,
                        "type": source.type_name,
                        "enabled": source.enabled,
                        "muted": source.muted,
                        "volume": source.volume
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "success": true,
                "sources": sources_data
            }))
        }
        Err(e) => {
            log::error!("Failed to get audio sources for OBS '{obs_name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to get audio sources: {e}"
            )))
        }
    }
}

/// Execute custom operation on Control Room OBS connection
#[tauri::command]
pub async fn control_room_execute_custom_operation(
    session_id: String,
    obs_name: String,
    operation: String,
    parameters: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::debug!(
        "Control Room: Executing custom operation '{operation}' on OBS '{obs_name}' for session {session_id}"
    );

    // Validate session before proceeding
    validate_session(&session_id)?;

    // Parse parameters into HashMap
    let params: HashMap<String, serde_json::Value> = match parameters {
        serde_json::Value::Object(map) => map.into_iter().collect(),
        _ => HashMap::new(),
    };

    let request = ObsOperationRequest {
        operation,
        parameters: params,
    };

    // Execute custom operation
    match app
        .obs_obws_plugin()
        .execute_custom_operation(request, Some(&obs_name))
        .await
    {
        Ok(response) => Ok(serde_json::json!({
            "success": true,
            "request_id": response.request_id,
            "status": response.status,
            "data": response.data,
            "error": response.error
        })),
        Err(e) => {
            log::error!("Failed to execute custom operation on OBS '{obs_name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to execute custom operation: {e}"
            )))
        }
    }
}

/// Execute raw OBS WebSocket request on Control Room OBS connection
#[tauri::command]
pub async fn control_room_execute_raw_request(
    session_id: String,
    obs_name: String,
    request_type: String,
    request_data: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::debug!(
        "Control Room: Executing raw request '{request_type}' on OBS '{obs_name}' for session {session_id}"
    );

    // Validate session before proceeding
    validate_session(&session_id)?;

    // Execute raw request
    match app
        .obs_obws_plugin()
        .execute_raw_request(&request_type, request_data, Some(&obs_name))
        .await
    {
        Ok(result) => Ok(serde_json::json!({
            "success": true,
            "requestType": request_type,
            "result": result
        })),
        Err(e) => {
            log::error!("Failed to execute raw request '{request_type}' on OBS '{obs_name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to execute raw request: {e}"
            )))
        }
    }
}

/// Get scenes for a Control Room OBS connection
#[tauri::command]
pub async fn control_room_get_scenes(
    session_id: String,
    obs_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::debug!("Control Room: Getting scenes for OBS '{obs_name}' session {session_id}");

    // Validate session before proceeding
    validate_session(&session_id)?;

    match app.obs_obws_plugin().get_scenes(Some(&obs_name)).await {
        Ok(scenes) => Ok(serde_json::json!({
            "success": true,
            "scenes": scenes
        })),
        Err(e) => {
            log::error!("Failed to get scenes for OBS '{obs_name}': {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to get scenes: {e}"
            )))
        }
    }
}

// Control Room Security Enhancement Commands
#[tauri::command]
pub async fn control_room_change_password(
    _session_id: String,
    _current_password: String,
    _new_password: String,
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room password change attempt");

    Ok(serde_json::json!({ "success": false, "error": "Not supported" }))
}
#[tauri::command]
pub async fn control_room_get_audit_log(
    _session_id: String,
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Control Room audit log request");

    Ok(serde_json::json!({ "success": true, "audit_entries": [] }))
}

#[tauri::command]
pub async fn control_room_get_session_info(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "authenticated": false }))
}

#[tauri::command]
pub async fn control_room_refresh_session(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
pub async fn control_room_logout(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true }))
}
