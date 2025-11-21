use crate::core::app::App;
use crate::types::TauriError;
use std::sync::Arc;
use tauri::State;

// ============================================================================
// Store / Event Commands
// ============================================================================

#[tauri::command]
pub async fn save_event(
    _event: serde_json::Value,
    _app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    // TODO: Implement actual event saving logic
    Ok(())
}

#[tauri::command]
pub async fn get_events(_app: State<'_, Arc<App>>) -> Result<Vec<serde_json::Value>, TauriError> {
    // TODO: Implement actual event retrieval logic
    Ok(vec![])
}

#[tauri::command]
pub async fn clear_events(_app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    // TODO: Implement actual event clearing logic
    Ok(())
}
