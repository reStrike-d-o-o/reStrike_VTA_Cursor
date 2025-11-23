use crate::core::app::App;
use crate::logging::archival::{AutoArchiveConfig, ArchiveSchedule};
use crate::types::TauriError;
use std::sync::Arc;
use tauri::State;
use serde_json;

#[tauri::command]
pub async fn create_complete_log_archive(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Creating complete log archive");

    let log_manager = app.log_manager().lock().await;
    match log_manager.create_complete_archive() {
        Ok(archive_info) => Ok(serde_json::json!({
            "success": true,
            "data": {
                "name": archive_info.name,
                "size": archive_info.size,
                "created": archive_info.created,
                "path": archive_info.file_path.to_string_lossy()
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to create archive: {}", e)
        })),
    }
}


#[tauri::command]
pub async fn create_and_upload_log_archive(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("=== CREATE_AND_UPLOAD_LOG_ARCHIVE COMMAND START ===");

    // Add comprehensive error logging to app.log
    let log_error_to_file = |error_msg: &str| {
        let error_log = format!(
            "[{}] LogArchiveManager Upload Error:\nError: {}\nCommand: create_and_upload_log_archive\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            error_msg
        );

        if let Err(write_err) = std::fs::write("logs/app.log", error_log) {
            log::error!("Failed to write error log: {write_err}");
        }
    };

    log::info!("Creating and uploading log archive to Google Drive");

    let log_manager = app.log_manager().lock().await;
    match log_manager.create_and_upload_archive().await {
        Ok(message) => {
            log::info!("=== CREATE_AND_UPLOAD_LOG_ARCHIVE COMMAND SUCCESS ===");
            log::info!("Upload successful: {message}");
            Ok(serde_json::json!({
                "success": true,
                "message": message
            }))
        }
        Err(e) => {
            let error_msg = format!("Failed to create and upload archive: {e}");
            log::error!("=== CREATE_AND_UPLOAD_LOG_ARCHIVE COMMAND ERROR ===");
            log::error!("{error_msg}");
            log_error_to_file(&error_msg);

            Ok(serde_json::json!({
                "success": false,
                "error": error_msg
            }))
        }
    }
}

#[tauri::command]
pub async fn create_upload_and_cleanup_log_archive(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("=== CREATE_UPLOAD_AND_CLEANUP_LOG_ARCHIVE COMMAND START ===");

    // Add comprehensive error logging to app.log
    let log_error_to_file = |error_msg: &str| {
        let error_log = format!(
            "[{}] LogArchiveManager Upload Error:\nError: {}\nCommand: create_upload_and_cleanup_log_archive\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            error_msg
        );

        if let Err(write_err) = std::fs::write("logs/app.log", error_log) {
            log::error!("Failed to write error log: {write_err}");
        }
    };

    log::info!("Creating, uploading, and cleaning up log archive");

    let log_manager = app.log_manager().lock().await;
    match log_manager.create_upload_and_cleanup_archive().await {
        Ok(message) => {
            log::info!("=== CREATE_UPLOAD_AND_CLEANUP_LOG_ARCHIVE COMMAND SUCCESS ===");
            log::info!("Upload and cleanup successful: {message}");
            Ok(serde_json::json!({
                "success": true,
                "message": message
            }))
        }
        Err(e) => {
            let error_msg = format!("Failed to create, upload and cleanup archive: {e}");
            log::error!("=== CREATE_UPLOAD_AND_CLEANUP_LOG_ARCHIVE COMMAND ERROR ===");
            log::error!("{error_msg}");
            log_error_to_file(&error_msg);

            Ok(serde_json::json!({
                "success": false,
                "error": error_msg
            }))
        }
    }
}

#[tauri::command]
pub async fn get_auto_archive_config(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting auto-archive configuration");

    // For now, return a default config. In a real implementation,
    // you would load this from a configuration file or database
    let default_config = AutoArchiveConfig {
        enabled: false,
        schedule: ArchiveSchedule::Monthly,
        upload_to_drive: false,
        delete_after_upload: false,
        last_archive_time: None,
    };

    Ok(serde_json::json!({
        "success": true,
        "data": default_config
    }))
}

#[tauri::command]
pub async fn set_auto_archive_config(
    config: AutoArchiveConfig,
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!(
        "Setting auto-archive configuration: enabled={}, schedule={:?}",
        config.enabled,
        config.schedule
    );

    // In a real implementation, you would save this to a configuration file or database
    // For now, we'll just return success

    Ok(serde_json::json!({
        "success": true,
        "message": "Auto-archive configuration updated successfully"
    }))
}

#[tauri::command]
pub async fn check_auto_archive_status(
    config: AutoArchiveConfig,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Checking auto-archive status");

    let log_manager = app.log_manager().lock().await;
    let should_archive = log_manager.should_auto_archive(&config);
    let next_archive_time = log_manager.get_next_archive_time(&config);

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "should_archive": should_archive,
            "next_archive_time": next_archive_time,
            "schedule": config.schedule.to_string(),
            "enabled": config.enabled
        }
    }))
}

#[tauri::command]
pub async fn perform_auto_archive(
    mut config: AutoArchiveConfig,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Performing auto-archive");

    let log_manager = app.log_manager().lock().await;
    match log_manager.perform_auto_archive(&mut config).await {
        Ok(message) => Ok(serde_json::json!({
            "success": true,
            "message": message,
            "updated_config": config
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Auto-archive failed: {}", e)
        })),
    }
}

#[tauri::command]
pub async fn delete_log_archive(
    archive_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Deleting log archive: {archive_name}");

    let log_manager = app.log_manager().lock().await;
    match log_manager.delete_archive(&archive_name) {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Archive '{}' deleted successfully", archive_name)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to delete archive: {}", e)
        })),
    }
}

#[tauri::command]
pub async fn list_log_files(
    subsystem: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Listing log files (subsystem filter: {:?})", subsystem);

    let log_manager = app.log_manager().lock().await;
    match log_manager.list_log_files(subsystem.as_deref()) {
        Ok(files) => Ok(serde_json::json!({
            "success": true,
            "data": files
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to list log files: {}", e)
        })),
    }
}

#[tauri::command]
pub async fn download_log_file(
    filename: String,
    app: State<'_, Arc<App>>,
) -> Result<Vec<u8>, TauriError> {
    log::info!("Downloading log file: {}", filename);

    let log_manager = app.log_manager().lock().await;
    match log_manager.read_log_file(&filename) {
        Ok(content) => Ok(content),
        Err(e) => {
            log::error!("Failed to read log file {}: {}", filename, e);
            Err(TauriError::Io(e))
        }
    }
}
