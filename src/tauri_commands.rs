use crate::plugin_obs::{ObsPlugin, ObsConnectionConfig, ObsWebSocketVersion, ObsEvent};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tauri::State;

// Global OBS plugin instance
pub type ObsPluginState = Arc<Mutex<Option<ObsPlugin>>>;

// Tauri command request/response structures
#[derive(Debug, Serialize, Deserialize)]
pub struct AddConnectionRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub protocol_version: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connection_name: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

// Initialize OBS plugin
pub fn init_obs_plugin() -> ObsPluginState {
    let (event_tx, _event_rx) = mpsc::unbounded_channel();
    let plugin = ObsPlugin::new(event_tx);
    Arc::new(Mutex::new(Some(plugin)))
}

// Tauri command: Add OBS connection
#[tauri::command]
pub async fn obs_add_connection(
    request: AddConnectionRequest,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let mut plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_mut()
        .ok_or("OBS plugin not initialized")?;

    // Convert protocol version string to enum
    let protocol_version = match request.protocol_version.as_str() {
        "v4" => ObsWebSocketVersion::V4,
        "v5" => ObsWebSocketVersion::V5,
        _ => return Err("Invalid protocol version. Must be 'v4' or 'v5'".to_string()),
    };

    // Create connection config
    let config = ObsConnectionConfig {
        name: request.name,
        host: request.host,
        port: request.port,
        password: request.password,
        protocol_version,
        enabled: request.enabled,
    };

    // Add connection
    match plugin.add_connection(config).await {
        Ok(_) => Ok(ObsResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Remove OBS connection
#[tauri::command]
pub async fn obs_remove_connection(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let mut plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_mut()
        .ok_or("OBS plugin not initialized")?;

    match plugin.remove_connection(&connection_name) {
        Ok(_) => Ok(ObsResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Connect to OBS
#[tauri::command]
pub async fn obs_connect(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let mut plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_mut()
        .ok_or("OBS plugin not initialized")?;

    match plugin.connect_obs(&connection_name).await {
        Ok(_) => Ok(ObsResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Get connection status
#[tauri::command]
pub async fn obs_get_connection_status(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    let status = plugin.get_connection_status(&connection_name);
    
    let status_data = status.map(|s| {
        serde_json::json!({
            "connection_name": connection_name,
            "status": format!("{:?}", s),
        })
    });

    Ok(ObsResponse {
        success: true,
        data: status_data,
        error: None,
    })
}

// Tauri command: Get all connection names
#[tauri::command]
pub async fn obs_get_connection_names(
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    let names = plugin.get_connection_names();
    
    Ok(ObsResponse {
        success: true,
        data: Some(serde_json::json!(names)),
        error: None,
    })
}

// Tauri command: Get current scene
#[tauri::command]
pub async fn obs_get_current_scene(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.get_current_scene(&connection_name).await {
        Ok(scene_name) => Ok(ObsResponse {
            success: true,
            data: Some(serde_json::json!({ "scene_name": scene_name })),
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Set current scene
#[tauri::command]
pub async fn obs_set_current_scene(
    connection_name: String,
    scene_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.set_current_scene(&connection_name, &scene_name).await {
        Ok(_) => Ok(ObsResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Start recording
#[tauri::command]
pub async fn obs_start_recording(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.start_recording(&connection_name).await {
        Ok(_) => Ok(ObsResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Stop recording
#[tauri::command]
pub async fn obs_stop_recording(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.stop_recording(&connection_name).await {
        Ok(_) => Ok(ObsResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Get recording status
#[tauri::command]
pub async fn obs_get_recording_status(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.get_recording_status(&connection_name).await {
        Ok(is_recording) => Ok(ObsResponse {
            success: true,
            data: Some(serde_json::json!({ "is_recording": is_recording })),
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Start replay buffer
#[tauri::command]
pub async fn obs_start_replay_buffer(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.start_replay_buffer(&connection_name).await {
        Ok(_) => Ok(ObsResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Stop replay buffer
#[tauri::command]
pub async fn obs_stop_replay_buffer(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.stop_replay_buffer(&connection_name).await {
        Ok(_) => Ok(ObsResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Save replay buffer
#[tauri::command]
pub async fn obs_save_replay_buffer(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.save_replay_buffer(&connection_name).await {
        Ok(_) => Ok(ObsResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Get replay buffer status
#[tauri::command]
pub async fn obs_get_replay_buffer_status(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.get_replay_buffer_status(&connection_name).await {
        Ok(is_active) => Ok(ObsResponse {
            success: true,
            data: Some(serde_json::json!({ "is_active": is_active })),
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Tauri command: Get all scenes
#[tauri::command]
pub async fn obs_get_scenes(
    connection_name: String,
    plugin_state: State<'_, ObsPluginState>,
) -> Result<ObsResponse, String> {
    let plugin_guard = plugin_state.lock().unwrap();
    let plugin = plugin_guard.as_ref()
        .ok_or("OBS plugin not initialized")?;

    match plugin.get_scenes(&connection_name).await {
        Ok(scenes) => Ok(ObsResponse {
            success: true,
            data: Some(serde_json::json!({ "scenes": scenes })),
            error: None,
        }),
        Err(e) => Ok(ObsResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

// Register all OBS commands with Tauri
pub fn register_obs_commands(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.invoke_handler(tauri::generate_handler![
        obs_add_connection,
        obs_remove_connection,
        obs_connect,
        obs_get_connection_status,
        obs_get_connection_names,
        obs_get_current_scene,
        obs_set_current_scene,
        obs_start_recording,
        obs_stop_recording,
        obs_get_recording_status,
        obs_start_replay_buffer,
        obs_stop_replay_buffer,
        obs_save_replay_buffer,
        obs_get_replay_buffer_status,
        obs_get_scenes,
    ]);
    
    Ok(())
} 