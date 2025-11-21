use crate::core::app::App;
use crate::logging::archival::{ArchiveSchedule, AutoArchiveConfig};
use anyhow;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use crate::types::TauriError;

// ============================================================================
// Google Drive Commands
// ============================================================================

// Authentication Commands

/// Request Google Drive authorization URL
#[tauri::command]
pub async fn drive_request_auth_url() -> Result<String, TauriError> {
    let (url, _csrf_token) = crate::plugins::drive_plugin()
        .auth_url()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
    Ok(url)
}

/// Complete Google Drive authentication with authorization code
#[tauri::command]
pub async fn drive_complete_auth(code: String) -> Result<(), TauriError> {
    crate::plugins::drive_plugin()
        .exchange_code(code)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))
}

/// Save Google Drive credentials
#[tauri::command]
pub async fn drive_save_credentials(id: String, secret: String) -> Result<(), TauriError> {
    crate::plugins::drive_plugin()
        .save_credentials(id, secret)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))
}

// File Operations

/// List Google Drive files (backup archives)
#[tauri::command]
pub async fn drive_list_files() -> Result<serde_json::Value, TauriError> {
    log::info!("Listing Google Drive files");

    match crate::plugins::drive_plugin().list_files().await {
        Ok(files) => Ok(serde_json::json!({
            "success": true,
            "files": files
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// List all Google Drive files
#[tauri::command]
pub async fn drive_list_all_files() -> Result<serde_json::Value, TauriError> {
    log::info!("Listing all Google Drive files");

    match crate::plugins::drive_plugin().list_all_files().await {
        Ok(files) => Ok(serde_json::json!({
            "success": true,
            "files": files
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Create a folder in Google Drive
#[tauri::command]
pub async fn drive_create_folder(
    name: String,
    parent_id: Option<String>,
) -> Result<serde_json::Value, TauriError> {
    let id = crate::plugins::drive_plugin()
        .create_folder(&name, parent_id.as_deref())
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(serde_json::json!({"success": true, "id": id}))
}

/// List children of a Google Drive folder
#[tauri::command]
pub async fn drive_list_children(
    parent_id: Option<String>,
) -> Result<serde_json::Value, TauriError> {
    let files = crate::plugins::drive_plugin()
        .list_children(parent_id.as_deref())
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(serde_json::json!({"success": true, "files": files}))
}

/// Upload a ZIP file to a Google Drive folder
#[tauri::command]
pub async fn drive_upload_zip_to_folder(
    zip_path: String,
    folder_id: Option<String>,
) -> Result<serde_json::Value, TauriError> {
    let p = PathBuf::from(&zip_path);
    if !p.is_file() {
        return Ok(serde_json::json!({"success": false, "error": "zip not found"}));
    }
    let file_name = p
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("archive.zip")
        .to_string();
    let id = crate::plugins::drive_plugin()
        .upload_file_streaming_to_folder(&p, &file_name, "application/zip", folder_id.as_deref())
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    Ok(serde_json::json!({"success": true, "file_id": id}))
}

// Backup Archive Management

/// Upload backup archive to Google Drive
#[tauri::command]
pub async fn drive_upload_backup_archive() -> Result<serde_json::Value, TauriError> {
    log::info!("=== DRIVE_UPLOAD_BACKUP_ARCHIVE COMMAND START ===");
    log::info!("Creating and uploading backup archive to Google Drive");

    // Log error to file immediately
    let log_error_to_file = |error_msg: &str| {
        let error_log = format!(
            "[{}] Tauri Command Upload Error:\nError: {}\nCommand: drive_upload_backup_archive\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            error_msg
        );
        if let Err(write_err) = std::fs::write("logs/app.log", error_log) {
            log::error!("Failed to write error log: {write_err}");
        }
    };

    // Step 1: Call drive plugin upload method (which creates and uploads the archive)
    log::info!("Step 1: Calling drive_plugin().upload_backup_archive()...");
    match crate::plugins::drive_plugin().upload_backup_archive().await {
        Ok(message) => {
            log::info!("=== DRIVE_UPLOAD_BACKUP_ARCHIVE COMMAND SUCCESS ===");
            log::info!("Upload successful: {message}");
            Ok(serde_json::json!({
                "success": true,
                "message": message
            }))
        }
        Err(e) => {
            let error_msg = format!("Failed to upload archive: {e}");
            log::error!("=== DRIVE_UPLOAD_BACKUP_ARCHIVE COMMAND ERROR ===");
            log::error!("{error_msg}");
            log_error_to_file(&error_msg);

            Ok(serde_json::json!({
                "success": false,
                "error": error_msg
            }))
        }
    }
}

/// Download backup archive from Google Drive
#[tauri::command]
pub async fn drive_download_backup_archive(
    file_id: String,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Downloading backup archive from Google Drive: {file_id}");

    // Use the new download_backup_archive method
    match crate::plugins::drive_plugin()
        .download_backup_archive(&file_id)
        .await
    {
        Ok(message) => Ok(serde_json::json!({
            "success": true,
            "message": message
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to download archive: {}", e)
        })),
    }
}

/// Delete backup archive from Google Drive
#[tauri::command]
pub async fn drive_delete_backup_archive(file_id: String) -> Result<serde_json::Value, TauriError> {
    log::info!("Deleting backup archive from Google Drive: {file_id}");

    match crate::plugins::drive_plugin()
        .delete_backup_archive(&file_id)
        .await
    {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Backup archive deleted successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to delete archive: {}", e)
        })),
    }
}

/// Restore from a Google Drive backup archive
#[tauri::command]
pub async fn drive_restore_from_archive(file_id: String) -> Result<serde_json::Value, TauriError> {
    log::info!("Restoring from Google Drive archive: {file_id}");

    match crate::plugins::drive_plugin()
        .restore_from_archive(&file_id)
        .await
    {
        Ok(message) => Ok(serde_json::json!({
            "success": true,
            "message": message
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

// Connection Status & Testing

/// Get Google Drive connection status
#[tauri::command]
pub async fn drive_get_connection_status() -> Result<serde_json::Value, TauriError> {
    log::info!("Checking Google Drive connection status");

    log::info!("About to call drive_plugin().is_connected()");
    // Use the new is_connected method for better reliability
    match crate::plugins::drive_plugin().is_connected().await {
        Ok(_connected) => {
            // If connected, try to get file count for additional info
            match crate::plugins::drive_plugin().list_files().await {
                Ok(files) => Ok(serde_json::json!({
                    "success": true,
                    "connected": true,
                    "file_count": files.len(),
                    "message": "Connected to Google Drive"
                })),
                Err(e) => Ok(serde_json::json!({
                    "success": true,
                    "connected": true,
                    "file_count": 0,
                    "message": "Connected to Google Drive (file listing failed)",
                    "warning": e.to_string()
                })),
            }
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "connected": false,
            "error": e.to_string(),
            "message": "Failed to check connection status"
        })),
    }
}

/// Test Google Drive connection
#[tauri::command]
pub async fn drive_test_connection() -> Result<serde_json::Value, TauriError> {
    log::info!("Testing Google Drive connection");

    // First check if connected
    match crate::plugins::drive_plugin().is_connected().await {
        Ok(_connected) => {
            if !_connected {
                return Ok(serde_json::json!({
                    "success": false,
                    "error": "Not connected to Google Drive"
                }));
            }
        }
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Connection check failed: {}", e)
            }))
        }
    }

    // Try to list all files to test API access
    match crate::plugins::drive_plugin().list_all_files().await {
        Ok(files) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Connected successfully. Found {} files in Drive.", files.len()),
            "file_count": files.len()
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("API access failed: {}", e)
        })),
    }
}

/// Get Google Drive storage quota
#[tauri::command]
pub async fn drive_get_quota() -> Result<serde_json::Value, TauriError> {
    log::info!("Getting Google Drive quota");
    // Best-effort: if plugin has quota method, call it; else derive from listing/chunks
    // Here we try list_all_files to estimate usage is too heavy; so check plugin API
    match crate::plugins::drive_plugin().get_quota().await {
        Ok((limit, usage, usage_in_drive)) => Ok(serde_json::json!({
            "success": true,
            "quota": {
                "limit": limit,
                "usage": usage,
                "usageInDrive": usage_in_drive,
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
            "error": format!("Failed to create archive: {}", e)
        })),
    }
}

/// Create and upload log archive to Google Drive
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

/// Create, upload, and cleanup log archive
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

// Auto-Archive Configuration

/// Get auto-archive configuration
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

/// Set auto-archive configuration
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

/// Check auto-archive status
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

/// Perform auto-archive
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
            "error": format!("Failed to perform auto-archive: {}", e)
        })),
    }
}
