// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use re_strike_vta::core::app::App;
use re_strike_vta::tauri_commands;
use re_strike_vta::types::AppResult;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging
    env_logger::init();
    
    log::info!("Starting reStrike VTA Tauri Application");
    
    // Create the main application instance
    let app: Arc<App> = Arc::new(App::new().await?);
    
    // Initialize the application
    app.init().await?;
    
    // Initialize plugins
    re_strike_vta::plugins::init().await?;
    
    // Start the application (this will auto-start UDP if configured)
    app.start().await?;
    
    // Start OBS event listener to forward events to frontend
    let app_clone = app.clone();
    tokio::spawn(async move {
        // For now, we'll use a simpler approach - periodic status checks
        // In a full implementation, we'd listen to the OBS event channel
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // Check OBS connection status and emit updates
            if let Ok(status) = app_clone.obs_plugin().get_obs_status().await {
                // Log status for debugging
                log::debug!("OBS Status: recording={}, streaming={}, cpu={}%", 
                    status.is_recording, status.is_streaming, status.cpu_usage);
            }
        }
    });
    
    // Create Tauri app builder
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(app)
        .invoke_handler(tauri::generate_handler![
            // Core commands
            tauri_commands::get_app_status,
            tauri_commands::shutdown_app,
            
            // UDP commands
            tauri_commands::start_udp_server,
        tauri_commands::get_network_interfaces,
        tauri_commands::get_best_network_interface,
            tauri_commands::stop_udp_server,
            tauri_commands::get_udp_status,
            
            // OBS commands - Fixed names
            tauri_commands::obs_connect,
            tauri_commands::obs_disconnect,
            tauri_commands::obs_remove_connection,
            tauri_commands::obs_get_status,
            tauri_commands::obs_start_recording,
            tauri_commands::obs_stop_recording,
            tauri_commands::obs_command,
            tauri_commands::obs_add_connection,
            tauri_commands::obs_connect_to_connection,
            tauri_commands::obs_get_connection_status,
            tauri_commands::obs_get_connections,
            tauri_commands::obs_emit_event,
            tauri_commands::obs_setup_status_listener,
            tauri_commands::cpu_setup_stats_listener,
            
            // Video commands - Fixed names
            tauri_commands::video_play,
            tauri_commands::video_stop,
            tauri_commands::video_get_info,
            tauri_commands::extract_clip,
            
            // PSS commands
            tauri_commands::pss_start_listener,
            tauri_commands::pss_stop_listener,
            tauri_commands::pss_get_events,
            tauri_commands::pss_emit_event,
            tauri_commands::pss_emit_pending_events,
            tauri_commands::pss_setup_event_listener,
            
            // System commands
            tauri_commands::system_get_info,
            tauri_commands::system_open_file_dialog,
            
            // Store commands
            tauri_commands::save_event,
            tauri_commands::get_events,
            tauri_commands::clear_events,
            
            // License commands
            tauri_commands::activate_license,
            tauri_commands::validate_license,
            tauri_commands::get_license_status,
            
            // Settings commands
            tauri_commands::get_settings,
            tauri_commands::update_settings,
            tauri_commands::get_config_stats,
            tauri_commands::reset_settings,
            tauri_commands::export_settings,
            tauri_commands::import_settings,
            tauri_commands::restore_settings_backup,
            tauri_commands::restore_backup_with_dialog,
            
            // Flag commands
            tauri_commands::get_flag_url,
            tauri_commands::download_flags,
            
            // Diagnostics & Logs commands - Fixed names
            tauri_commands::list_log_files,
            tauri_commands::download_log_file,
            tauri_commands::list_archives,
            tauri_commands::extract_archive,
            tauri_commands::download_archive,
            tauri_commands::set_live_data_streaming,
            
            // Legacy commands for backward compatibility
            tauri_commands::start_live_data,
            tauri_commands::stop_live_data,
            tauri_commands::get_live_data,
            tauri_commands::obs_get_debug_info,
            tauri_commands::obs_toggle_full_events,
            tauri_commands::obs_get_full_events_setting,
            tauri_commands::obs_emit_event_to_frontend,
            tauri_commands::obs_get_recent_events,
            
            // CPU Monitoring commands
            tauri_commands::cpu_get_process_data,
            tauri_commands::cpu_get_system_data,
            tauri_commands::cpu_get_obs_usage,
            tauri_commands::cpu_update_config,
            tauri_commands::cpu_test_plugin,
            tauri_commands::cpu_enable_monitoring,
            tauri_commands::cpu_disable_monitoring,
            tauri_commands::cpu_get_monitoring_status,
            
            // Protocol Management commands
            tauri_commands::protocol_get_versions,
            tauri_commands::protocol_set_active_version,
            tauri_commands::protocol_upload_file,
            tauri_commands::protocol_delete_version,
            tauri_commands::protocol_export_file,
            tauri_commands::protocol_get_current,
            
            // Window Management commands
            tauri_commands::set_window_fullscreen,
            tauri_commands::set_window_compact,
            tauri_commands::set_window_custom_size,
            tauri_commands::set_window_position,
            tauri_commands::set_window_startup_position,
            tauri_commands::save_window_settings,
            tauri_commands::load_window_settings,
            tauri_commands::get_screen_size,
            
            // Database commands
            tauri_commands::initialize_ui_settings_database,
            tauri_commands::db_initialize_ui_settings,
            tauri_commands::db_get_ui_setting,
            tauri_commands::db_set_ui_setting,
            tauri_commands::db_get_all_ui_settings,
            tauri_commands::db_get_database_info,
            tauri_commands::migrate_json_to_database,
            tauri_commands::create_json_backup,
            tauri_commands::restore_from_json_backup,
            tauri_commands::get_migration_status,
            tauri_commands::enable_database_mode,
            
            // Google Drive commands
            tauri_commands::drive_request_auth_url,
            tauri_commands::drive_complete_auth,
            tauri_commands::drive_save_credentials,
        ])
        .setup(|_app| {
            log::info!("Tauri application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    Ok(())
}


