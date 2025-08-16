// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use re_strike_vta::core::app::App;
use re_strike_vta::tauri_commands;
#[cfg(feature = "obs-obws")]
use re_strike_vta::tauri_commands_obws;
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
    let _app_clone = app.clone();
    // Optional: legacy status poller removed to avoid mixing APIs
    
    // Create Tauri app builder
    tauri::Builder::default()
        .manage(app)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
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
            tauri_commands::obs_get_obs_version,
            tauri_commands::obs_get_profiles,
            tauri_commands::obs_get_current_profile,
            tauri_commands::obs_set_current_profile,
            tauri_commands::obs_get_studio_mode,
            tauri_commands::obs_set_studio_mode,
            tauri_commands::obs_get_sources,
            tauri_commands::obs_set_source_visibility,
            tauri_commands::obs_get_recording_settings,
            tauri_commands::obs_set_recording_settings,
            tauri_commands::obs_get_streaming_settings,
            tauri_commands::obs_set_streaming_settings,
            tauri_commands::obs_get_streaming_accounts,
            tauri_commands::obs_get_streaming_channels,
            tauri_commands::obs_set_streaming_account,
            tauri_commands::obs_get_streaming_events,
            
            // Control Room Commands - Using new async implementation
            tauri_commands::control_room_authenticate_async,
                    tauri_commands::control_room_get_obs_connections,
                tauri_commands::control_room_get_obs_connections_with_status,
                tauri_commands::control_room_get_obs_connections_with_details,
        tauri_commands::control_room_add_obs_connection,
        tauri_commands::control_room_connect_obs,
        tauri_commands::control_room_disconnect_obs,
        tauri_commands::control_room_remove_obs_connection,
        tauri_commands::control_room_get_obs_connection,
        tauri_commands::control_room_update_obs_connection,
        tauri_commands::control_room_connect_all_obs,
        tauri_commands::control_room_disconnect_all_obs,
            tauri_commands::control_room_change_password,
            tauri_commands::control_room_get_audit_log,
            tauri_commands::control_room_get_session_info,
            tauri_commands::control_room_refresh_session,
            tauri_commands::control_room_logout,
            // tauri_commands::control_room_mute_all_obs,
            // tauri_commands::control_room_unmute_all_obs,
            // tauri_commands::control_room_change_all_obs_scenes,
            // tauri_commands::control_room_start_all_obs,
            // tauri_commands::control_room_stop_all_obs,
            tauri_commands::control_room_get_audio_sources,
            tauri_commands::control_room_get_scenes,
            
            // YouTube Streaming Management Commands
            tauri_commands::obs_get_youtube_accounts,
            tauri_commands::obs_get_youtube_channels,
            tauri_commands::obs_get_youtube_stream_key,
            tauri_commands::obs_set_youtube_streaming_config,
            tauri_commands::obs_get_youtube_categories,
            tauri_commands::obs_get_youtube_privacy_options,
            tauri_commands::obs_get_youtube_latency_options,
            tauri_commands::obs_get_youtube_server_urls,
            tauri_commands::obs_regenerate_youtube_stream_key,
            tauri_commands::obs_get_youtube_streaming_analytics,
            tauri_commands::obs_get_youtube_streaming_schedule,
            tauri_commands::obs_create_youtube_streaming_schedule,
            
            // Other Streaming Destinations Commands
            tauri_commands::obs_get_available_streaming_services,
            tauri_commands::obs_get_twitch_config,
            tauri_commands::obs_get_facebook_config,
            tauri_commands::obs_get_custom_rtmp_config,
            tauri_commands::obs_set_custom_rtmp_config,
            tauri_commands::obs_get_streaming_auth_status,
            tauri_commands::obs_authenticate_streaming_service,
            tauri_commands::obs_refresh_streaming_auth,
            
            tauri_commands::obs_get_performance_metrics,
        tauri_commands::obs_start_monitoring,
        tauri_commands::obs_stop_monitoring,
            tauri_commands::obs_add_event_filter,
            tauri_commands::obs_get_event_filters,
            tauri_commands::obs_add_event_route,
            tauri_commands::obs_get_event_routes,
            tauri_commands::obs_command,
            
            // OBS obws commands - New obws-based implementation
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_add_connection,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_update_connection,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_connect,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_disconnect,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_connection_status,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_connections,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_remove_connection,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_status,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_start_recording,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_stop_recording,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_recording_status,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_start_streaming,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_stop_streaming,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_streaming_status,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_current_scene,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_set_current_scene,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_scenes,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_version,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_stats,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_test_connection,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_setup_status_listener,
            
            // OBS obws Replay Buffer Commands
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_start_replay_buffer,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_stop_replay_buffer,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_save_replay_buffer,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_replay_buffer_status,
            
            // OBS obws Path Configuration Commands
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_recording_path_settings,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_set_recording_path,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_replay_buffer_path_settings,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_set_replay_buffer_path,
            
            // OBS obws Recording Configuration Commands
            // unified config
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_save_full_config,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_full_config,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_create_recording_session,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_update_recording_session_status,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_generate_recording_path,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_windows_videos_folder,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_test_path_generation,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_create_test_folders,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_send_config_to_obs,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_test_recording,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_apply_path_decision,
            // obsolete per-user request removed: automatic/recording split
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_get_current_recording_session,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_clear_recording_session,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_manual_start_recording,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_manual_stop_recording,
            tauri_commands_obws::ivr_get_replay_settings,
            tauri_commands_obws::ivr_save_replay_settings,
            tauri_commands_obws::ivr_round_replay_now,
            tauri_commands_obws::ivr_list_tournament_days,
            tauri_commands_obws::ivr_list_matches_for_day,
            tauri_commands_obws::ivr_list_recorded_videos,
            tauri_commands_obws::ivr_open_video_path,
            tauri_commands_obws::ivr_open_event_video,
            tauri_commands_obws::ivr_delete_recorded_videos,
            tauri_commands_obws::ivr_upload_recorded_videos,
            tauri_commands_obws::ivr_import_recorded_videos,
            tauri_commands_obws::ivr_validate_mpv_path,
            tauri_commands_obws::obs_obws_get_record_directory,
            tauri_commands_obws::obs_obws_get_filename_formatting,
            tauri_commands::control_room_get_obs_connection,
            tauri_commands::control_room_update_obs_connection,
            tauri_commands::control_room_connect_all_obs,
            tauri_commands::control_room_disconnect_all_obs,
            tauri_commands::control_room_change_password,
            tauri_commands::control_room_get_audit_log,
            tauri_commands::control_room_get_session_info,
            tauri_commands::control_room_refresh_session,
            tauri_commands::control_room_logout,
            // tauri_commands::control_room_mute_all_obs,
            // tauri_commands::control_room_unmute_all_obs,
            // tauri_commands::control_room_change_all_obs_scenes,
            // tauri_commands::control_room_start_all_obs,
            // tauri_commands::control_room_stop_all_obs,
            tauri_commands::control_room_get_audio_sources,
            tauri_commands::control_room_get_scenes,
            
            // YouTube Streaming Management Commands
            tauri_commands::obs_get_youtube_accounts,
            tauri_commands::obs_get_youtube_channels,
            tauri_commands::obs_get_youtube_stream_key,
            tauri_commands::obs_set_youtube_streaming_config,
            tauri_commands::obs_get_youtube_categories,
            tauri_commands::obs_get_youtube_privacy_options,
            tauri_commands::obs_get_youtube_latency_options,
            tauri_commands::obs_get_youtube_server_urls,
            tauri_commands::obs_regenerate_youtube_stream_key,
            tauri_commands::obs_get_youtube_streaming_analytics,
            tauri_commands::obs_get_youtube_streaming_schedule,
            tauri_commands::obs_create_youtube_streaming_schedule,
            
            // Other Streaming Destinations Commands
            tauri_commands::obs_get_available_streaming_services,
            tauri_commands::obs_get_twitch_config,
            tauri_commands::obs_get_facebook_config,
            tauri_commands::obs_get_custom_rtmp_config,
            tauri_commands::obs_set_custom_rtmp_config,
            tauri_commands::obs_get_streaming_auth_status,
            tauri_commands::obs_authenticate_streaming_service,
            tauri_commands::obs_refresh_streaming_auth,
            
            tauri_commands::obs_get_performance_metrics,
        tauri_commands::obs_start_monitoring,
        tauri_commands::obs_stop_monitoring,
            tauri_commands::obs_add_event_filter,
            tauri_commands::obs_get_event_filters,
            tauri_commands::obs_add_event_route,
            tauri_commands::obs_get_event_routes,
            tauri_commands::obs_command,
            tauri_commands::obs_connect_to_connection,
            tauri_commands::obs_get_connection_status,
            tauri_commands::obs_get_connections,
            tauri_commands::obs_emit_event,
            tauri_commands::obs_setup_status_listener,
            tauri_commands::cpu_setup_stats_listener,
            
            // Recording Path and Filename Commands
            tauri_commands::obs_get_recording_path_settings,
            tauri_commands::obs_set_recording_path,
            tauri_commands::obs_set_recording_filename,
            tauri_commands::obs_set_recording_format,
            tauri_commands::obs_get_available_recording_formats,
            tauri_commands::obs_get_filename_format_variables,
            tauri_commands::obs_get_default_recording_settings,
            
            // Replay Buffer Settings Commands
            tauri_commands::obs_get_replay_buffer_settings,
            tauri_commands::obs_set_replay_buffer_settings,
            tauri_commands::obs_set_replay_buffer_duration,
            tauri_commands::obs_set_replay_buffer_path,
            tauri_commands::obs_set_replay_buffer_filename,
            tauri_commands::obs_get_default_replay_buffer_settings,
            
            // Advanced Replay Buffer Commands
            tauri_commands::obs_get_detailed_replay_buffer_status,
            tauri_commands::obs_get_replay_buffer_duration,
            tauri_commands::obs_set_replay_buffer_duration_advanced,
            tauri_commands::obs_get_replay_buffer_path_advanced,
            tauri_commands::obs_set_replay_buffer_path_advanced,
            tauri_commands::obs_get_replay_buffer_filename_advanced,
            tauri_commands::obs_set_replay_buffer_filename_advanced,
            tauri_commands::obs_get_replay_buffer_format,
            tauri_commands::obs_set_replay_buffer_format,
            tauri_commands::obs_get_replay_buffer_quality,
            tauri_commands::obs_set_replay_buffer_quality,
            tauri_commands::obs_get_replay_buffer_bitrate,
            tauri_commands::obs_set_replay_buffer_bitrate,
            tauri_commands::obs_get_replay_buffer_keyframe_interval,
            tauri_commands::obs_set_replay_buffer_keyframe_interval,
            tauri_commands::obs_get_replay_buffer_rate_control,
            tauri_commands::obs_set_replay_buffer_rate_control,
            tauri_commands::obs_get_replay_buffer_preset,
            tauri_commands::obs_set_replay_buffer_preset,
            tauri_commands::obs_get_replay_buffer_profile,
            tauri_commands::obs_set_replay_buffer_profile,
            tauri_commands::obs_get_replay_buffer_tune,
            tauri_commands::obs_set_replay_buffer_tune,
            tauri_commands::obs_get_all_replay_buffer_settings,
            tauri_commands::obs_set_all_replay_buffer_settings,
            
            // Replay Buffer Options Commands
            tauri_commands::obs_get_available_replay_buffer_formats,
            tauri_commands::obs_get_available_replay_buffer_qualities,
            tauri_commands::obs_get_available_replay_buffer_rate_controls,
            tauri_commands::obs_get_available_replay_buffer_presets,
            tauri_commands::obs_get_available_replay_buffer_profiles,
            tauri_commands::obs_get_available_replay_buffer_tunes,
            
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
            tauri_commands::pss_get_events_for_match,
            tauri_commands::pss_list_recent_matches,
            tauri_commands::pss_get_current_match,
            tauri_commands::pss_get_match_details,
            tauri_commands::pss_clear_all_data,
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
            tauri_commands::db_run_migrations,
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
            tauri_commands::drive_get_quota,
            
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
            re_strike_vta::tauri_commands_triggers::triggers_recent_logs,
            re_strike_vta::tauri_commands_triggers::triggers_preview_evaluate,
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
            // OBS Commands (duplicates removed - already registered above)
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
                tauri_commands::simulation_run_self_test,
                tauri_commands::simulation_get_self_test_report,
                tauri_commands::simulation_get_self_test_categories,
                tauri_commands::simulation_run_selective_self_test,
            // YouTube API commands
            tauri_commands::youtube_get_auth_url,
            tauri_commands::youtube_authenticate,
            tauri_commands::youtube_create_playlist,
            tauri_commands::youtube_get_playlists,
            tauri_commands::youtube_add_video_to_playlist,
            tauri_commands::youtube_get_playlist_videos,
            tauri_commands::youtube_update_playlist,
            tauri_commands::youtube_delete_playlist,
            tauri_commands::youtube_create_scheduled_stream,
            tauri_commands::youtube_get_live_streams,
            tauri_commands::youtube_get_scheduled_streams,
            tauri_commands::youtube_get_completed_streams,
            tauri_commands::youtube_end_stream,
            tauri_commands::youtube_get_channel_info,
            tauri_commands::youtube_get_video_analytics,
            tauri_commands::youtube_initialize,
            // Security commands - TODO: Fix thread safety issues with SQLite in async contexts
            // re_strike_vta::tauri_commands_security::security_migrate_configurations,
            // re_strike_vta::tauri_commands_security::security_verify_migration,
            // re_strike_vta::tauri_commands_security::security_create_session,
            // re_strike_vta::tauri_commands_security::security_get_config,
            // re_strike_vta::tauri_commands_security::security_set_config,
            // re_strike_vta::tauri_commands_security::security_delete_config,
            // re_strike_vta::tauri_commands_security::security_list_config_keys,
            // re_strike_vta::tauri_commands_security::security_invalidate_session,
            // re_strike_vta::tauri_commands_security::security_get_audit_history,
            // re_strike_vta::tauri_commands_security::security_clear_cache,
            // re_strike_vta::tauri_commands_security::security_get_cache_stats,
            // re_strike_vta::tauri_commands_security::security_test_system,
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


