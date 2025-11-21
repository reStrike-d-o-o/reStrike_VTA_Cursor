use crate::core::app::App;
use anyhow;
use std::sync::Arc;
use tauri::State;
use crate::types::TauriError;

// ============================================================================
// Settings Commands
// ============================================================================

/// Get application settings
#[tauri::command]
pub async fn get_settings(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting application settings");

    let config = app.config_manager().get_config().await;
    let config_json = serde_json::to_value(config)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to serialize config: {e}")))?;
    Ok(config_json)
}

/// Update application settings
#[tauri::command]
pub async fn update_settings(
    settings: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    log::info!("Updating application settings");

    let config: crate::config::AppConfig = serde_json::from_value(settings)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to deserialize settings: {e}")))?;

    app.config_manager()
        .update_config(config)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to update settings: {e}")))
}

/// Get configuration statistics
#[tauri::command]
pub async fn get_config_stats(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting configuration statistics");

    match app.config_manager().get_config_stats().await {
        Ok(stats) => {
            let stats_json = serde_json::to_value(stats)
                .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to serialize stats: {e}")))?;
            Ok(stats_json)
        }
        Err(e) => Err(TauriError::from(anyhow::anyhow!(
            "Failed to get config stats: {e}"
        ))),
    }
}

/// Reset settings to defaults
#[tauri::command]
pub async fn reset_settings(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Resetting settings to defaults");

    app.config_manager()
        .reset_to_defaults()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to reset settings: {e}")))
}

/// Export settings to a file
#[tauri::command]
pub async fn export_settings(
    export_path: String,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    log::info!("Exporting settings to: {export_path}");

    let path = std::path::Path::new(&export_path);
    app.config_manager()
        .export_config(path)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to export settings: {e}")))
}

/// Import settings from a file
#[tauri::command]
pub async fn import_settings(
    import_path: String,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    log::info!("Importing settings from: {import_path}");

    let path = std::path::Path::new(&import_path);
    app.config_manager()
        .import_config(path)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to import settings: {e}")))
}

/// Restore settings from backup
#[tauri::command]
pub async fn restore_settings_backup(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Restoring settings from backup");

    app.config_manager()
        .restore_from_backup()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to restore settings backup: {e}")))
}
