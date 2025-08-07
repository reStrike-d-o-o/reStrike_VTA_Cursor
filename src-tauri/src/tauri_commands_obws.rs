//! Tauri commands for OBS WebSocket integration using obws crate

use crate::core::app::App;
// AppError and AppResult are used in the ObsManager implementation
use crate::plugins::obs_obws::manager::ObsManager;
use crate::plugins::obs_obws::ObsConnectionConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, Error as TauriError};

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

/// Connect to an OBS instance using obws
#[tauri::command]
pub async fn obs_obws_connect(
    connectionName: String,
    app: State<'_, Arc<App>>,
) -> Result<ObsObwsConnectionResponse, TauriError> {
    log::info!("OBS obws connect called: {}", connectionName);
    
    match app.obs_obws_plugin().connect(&connectionName).await {
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
