use crate::core::app::App;
use std::sync::Arc;
use tauri::State;
use crate::types::TauriError;

// ============================================================================
// Overlay/WebSocket Commands
// ============================================================================

/// Get WebSocket server status
#[tauri::command]
pub async fn websocket_get_status(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting WebSocket server status");

    let websocket_plugin = app.websocket_plugin().lock().await;
    let client_count = websocket_plugin.get_client_count();

    Ok(serde_json::json!({
        "connected_clients": client_count,
        "status": "running"
    }))
}
