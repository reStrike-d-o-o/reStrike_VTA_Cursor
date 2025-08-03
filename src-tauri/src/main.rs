// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use re_strike_vta::core::app::App;
use re_strike_vta::tauri_commands;
use re_strike_vta::types::AppResult;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Set up global panic hook to log all panics to app.log
    std::panic::set_hook(Box::new(|panic_info| {
        let panic_msg = format!(
            "[{}] PANIC: {:?}\nBacktrace: {:?}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            panic_info,
            std::backtrace::Backtrace::capture()
        );
        
        // Log to app.log
        if let Err(write_err) = std::fs::write("logs/app.log", &panic_msg) {
            eprintln!("Failed to write panic log: {}", write_err);
        }
        
        // Also log to stderr
        eprintln!("{}", panic_msg);
    }));

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
        .manage(app)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Core app commands
            tauri_commands::get_app_status,
            tauri_commands::shutdown_app,
            
            // UDP commands
            tauri_commands::start_udp_server,
            tauri_commands::stop_udp_server,
            tauri_commands::get_udp_status,
            tauri_commands::update_udp_settings,
            
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
            
            // WebSocket commands for HTML overlays
            tauri_commands::websocket_get_status,
            tauri_commands::websocket_broadcast_pss_event,
            tauri_commands::store_pss_event,
            
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
                    tauri_commands::get_network_interfaces,
        tauri_commands::get_best_network_interface,
        tauri_commands::get_best_ip_address_for_interface,
            
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
            
            // New Log Archive & Google Drive commands
            tauri_commands::create_complete_log_archive,
            tauri_commands::create_and_upload_log_archive,
            tauri_commands::create_upload_and_cleanup_log_archive,
            tauri_commands::get_auto_archive_config,
            tauri_commands::set_auto_archive_config,
            tauri_commands::check_auto_archive_status,
            tauri_commands::perform_auto_archive,
            tauri_commands::delete_log_archive,
            
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
            tauri_commands::restore_from_backup,
            tauri_commands::get_migration_status,
            tauri_commands::enable_database_mode,
            tauri_commands::list_backup_files,
            tauri_commands::get_database_preview,
            tauri_commands::get_database_tables,
            tauri_commands::get_table_data,
            tauri_commands::get_flag_mappings_data,
            
            // Tournament Management commands
            tauri_commands::tournament_create,
            tauri_commands::tournament_get_all,
            tauri_commands::tournament_get,
            tauri_commands::tournament_update,
            tauri_commands::tournament_delete,
            tauri_commands::tournament_get_days,
            tauri_commands::tournament_start_day,
            tauri_commands::tournament_end_day,
            tauri_commands::tournament_get_active,
            tauri_commands::tournament_get_active_day,
            tauri_commands::tournament_update_logo,
            tauri_commands::tournament_verify_location,
            tauri_commands::get_tournament_statistics,
        tauri_commands::database_run_vacuum,
        tauri_commands::database_run_integrity_check,
        tauri_commands::database_run_analyze,
        tauri_commands::database_run_optimize,
        tauri_commands::database_run_full_maintenance,
        tauri_commands::database_get_info,
                    tauri_commands::database_get_maintenance_status,
            
            // Event Status Analysis commands
            tauri_commands::get_comprehensive_event_statistics,
            tauri_commands::get_events_by_status,
            tauri_commands::get_unknown_events,
            
            // Tournament Context Management commands
                    tauri_commands::set_udp_tournament_context,
        tauri_commands::get_udp_tournament_context,
        tauri_commands::clear_udp_tournament_context,
                    tauri_commands::get_udp_performance_metrics,
            tauri_commands::get_udp_memory_usage,
            
            // Phase 2 Optimization - Data Archival commands
            tauri_commands::archive_old_events,
            tauri_commands::get_archive_statistics,
            tauri_commands::restore_from_archive,
            tauri_commands::cleanup_old_archive_data,
            tauri_commands::optimize_archive_tables,
            tauri_commands::get_database_pool_stats,
            tauri_commands::cleanup_database_pool,
            
            // Flag management commands
            tauri_commands::scan_and_populate_flags,
            tauri_commands::get_flags_data,
            tauri_commands::clear_flags_table,
            
            // Google Drive commands
            tauri_commands::drive_request_auth_url,
            tauri_commands::drive_complete_auth,
            tauri_commands::drive_save_credentials,
            tauri_commands::drive_list_files,
            tauri_commands::drive_list_all_files,
            tauri_commands::drive_test_connection,
            tauri_commands::drive_upload_backup_archive,
            tauri_commands::drive_download_backup_archive,
            tauri_commands::drive_restore_from_archive,
            tauri_commands::drive_delete_backup_archive,
            tauri_commands::drive_get_connection_status,
            
            // Phase 3: Advanced Caching Commands
            tauri_commands::get_cache_statistics,
            tauri_commands::clear_cache,
            tauri_commands::invalidate_tournament_cache,
            tauri_commands::invalidate_match_cache,
            
            // Phase 3: Event Stream Commands
            tauri_commands::get_stream_statistics,
            tauri_commands::send_event_to_stream,
            
            // Trigger system commands
            re_strike_vta::tauri_commands_triggers::triggers_list_pss_events,
            re_strike_vta::tauri_commands_triggers::triggers_list_obs_scenes,
            re_strike_vta::tauri_commands_triggers::triggers_list_active_overlays,
            re_strike_vta::tauri_commands_triggers::triggers_get,
            re_strike_vta::tauri_commands_triggers::triggers_save,
            re_strike_vta::tauri_commands_overlays::overlays_sync_templates,
            re_strike_vta::tauri_commands_overlays::overlays_populate_from_files,
            re_strike_vta::tauri_commands_obs_connections::obs_connections_get_all,
            re_strike_vta::tauri_commands_obs_connections::obs_connections_get_active,
            re_strike_vta::tauri_commands_obs_connections::obs_connections_save,
            re_strike_vta::tauri_commands_obs_connections::obs_connections_update_status,
            re_strike_vta::tauri_commands_obs_connections::obs_connections_delete,
            re_strike_vta::tauri_commands_obs_connections::obs_connections_clear_all,
            re_strike_vta::tauri_commands_obs_connections::obs_connections_sync_from_config,
            // Manual Mode Commands
            re_strike_vta::tauri_commands_manual_mode::manual_create_match,
            re_strike_vta::tauri_commands_manual_mode::manual_restore_data,
            re_strike_vta::tauri_commands_manual_mode::manual_get_statistics,
            // OBS Commands
            tauri_commands::obs_connect,
            tauri_commands::obs_add_connection,
            tauri_commands::obs_connect_to_connection,
            tauri_commands::obs_get_connection_status,
            tauri_commands::obs_get_connections,
            tauri_commands::obs_disconnect,
            tauri_commands::obs_remove_connection,
            tauri_commands::obs_get_status,
            tauri_commands::obs_start_recording,
            tauri_commands::obs_stop_recording,
            tauri_commands::obs_list_scenes,
            // Phase 3: Load Balancer Commands
            tauri_commands::get_distributor_statistics,
            tauri_commands::get_server_statistics,
            tauri_commands::add_server,
            tauri_commands::remove_server,
            
            // Phase 3: Advanced Analytics Commands
            tauri_commands::get_tournament_analytics,
            tauri_commands::get_performance_analytics,
            tauri_commands::get_athlete_analytics,
            tauri_commands::get_match_analytics,
            tauri_commands::get_analytics_history,
            
                            // Simulation commands
                tauri_commands::simulation_start,
                tauri_commands::simulation_stop,
                tauri_commands::simulation_get_status,
                tauri_commands::simulation_send_event,
                tauri_commands::simulation_get_scenarios,
                tauri_commands::simulation_run_automated,
                tauri_commands::simulation_get_detailed_status,
        ])
        .setup(|app| {
            log::info!("Tauri application setup complete");
            
            // Set the global app handle for frontend event emission
            re_strike_vta::core::app::App::set_global_app_handle(app.handle().clone());
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    Ok(())
}


