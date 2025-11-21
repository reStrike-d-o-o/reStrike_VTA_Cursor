// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use env_logger::{fmt::Color, WriteStyle};
use log::{Level, LevelFilter};
use re_strike_vta::core::app::App;
// use re_strike_vta::tauri_commands; // Removed - moved to commands module
use re_strike_vta::commands;
#[cfg(feature = "obs-obws")]
use re_strike_vta::tauri_commands_obws;
use re_strike_vta::types::{AppError, AppResult};
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::oneshot;

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
            eprintln!("Failed to write panic log: {write_err}");
        }

        // Also log to stderr
        eprintln!("{panic_msg}");
    }));

    // Initialize logging
    let mut logger_builder = env_logger::Builder::from_env(env_logger::Env::default());
    logger_builder.write_style(WriteStyle::Always);
    if std::env::var("RUST_LOG").is_err() {
        logger_builder.filter_level(LevelFilter::Info);
    }
    logger_builder.format(|buf, record| {
        use std::io::Write;

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level_code = re_strike_vta::logging::level_code(record.level());
        let prefix_len = format!("[{timestamp}] [{level_code}] - ").len();

        let message = format!("{}", record.args());
        let sanitized = re_strike_vta::logging::sanitize_message(&message);
        let formatted_message =
            re_strike_vta::logging::align_multiline(prefix_len, sanitized.as_ref());

        let mut level_style = buf.style();
        match record.level() {
            Level::Error => {
                level_style.set_color(Color::Red);
                level_style.set_intense(true);
            }
            Level::Warn => {
                level_style.set_color(Color::Yellow);
                level_style.set_intense(true);
            }
            Level::Info => {
                level_style.set_color(Color::Green);
                level_style.set_intense(true);
            }
            Level::Debug => {
                level_style.set_color(Color::Cyan);
            }
            Level::Trace => {
                level_style.set_color(Color::Magenta);
            }
        }
        match record.level() {
            Level::Warn | Level::Error => {
                let mut message_style = buf.style();
                if record.level() == Level::Error {
                    message_style.set_color(Color::White);
                    message_style.set_intense(true);
                    message_style.set_bg(Color::Red);
                } else {
                    message_style.set_color(Color::Yellow);
                    message_style.set_intense(true);
                }
                writeln!(
                    buf,
                    "[{}] [{}] - {}",
                    timestamp,
                    level_style.value(level_code),
                    message_style.value(formatted_message)
                )
            }
            _ => writeln!(
                buf,
                "[{}] [{}] - {}",
                timestamp,
                level_style.value(level_code),
                formatted_message
            ),
        }
    });
    logger_builder.init();

    log::error!("Demo error log to verify styling");

    log::info!("Starting reStrike VTA Tauri Application");

    // Create the main application instance
    let app: Arc<App> = Arc::new(App::new().await?);

    // Initialize the application
    app.init().await?;

    // Initialize plugins
    re_strike_vta::plugins::init().await?;

    // Validate license before starting services
    {
        let status = app.license_plugin().validate(app.config_manager()).await?;
        match status.state {
            re_strike_vta::plugins::plugin_license::LicenseState::Valid
            | re_strike_vta::plugins::plugin_license::LicenseState::Trial => {
                log::info!(
                    "License OK: {:?}, days remaining: {:?}",
                    status.state,
                    status.days_remaining
                );
            }
            _ => {
                log::error!(
                    "License check failed: {:?} ({:?})",
                    status.state,
                    status.reason
                );
                // Proceed to build UI so user can activate, but avoid auto-start services
            }
        }
    }

    // Start the application (this will auto-start UDP if configured)
    app.start().await?;

    // Start OBS event listener to forward events to frontend
    let _app_clone = app.clone();
    // Optional: legacy status poller removed to avoid mixing APIs

    // Create Tauri app builder
    let managed_app = app.clone();
    let shutdown_app = app.clone();

    let tauri_result = tauri::Builder::default()
        .manage(managed_app)
        .on_window_event({
            let shutdown_app = shutdown_app.clone();
            move |window, event| {
                // Only intercept close for the main application window.
                // Overlay and auxiliary windows should be allowed to close normally.
                if window.label() != "main" {
                    return;
                }

                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();

                    let app = shutdown_app.clone();
                    if !app.begin_shutdown() {
                        log::info!("Shutdown already in progress; ignoring duplicate close request");
                        return;
                    }

                    let window = window.clone();
                    tauri::async_runtime::spawn(async move {
                        let context = app.shutdown_context().await;
                        if context.requires_confirmation() {
                            let mut warnings: Vec<String> = Vec::new();

                            if context.recording_active {
                                let state = context
                                    .recording_state
                                    .clone()
                                    .unwrap_or_else(|| "Recording in progress".to_string());
                                warnings.push(format!("• OBS recording is still running ({state})."));
                            }

                            if context.match_in_progress {
                                let match_label = context
                                    .match_number
                                    .clone()
                                    .unwrap_or_else(|| "Current match".to_string());
                                if let Some(description) = context.match_description.as_ref() {
                                    warnings.push(format!(
                                        "• Match {match_label} is still active ({description})."
                                    ));
                                } else {
                                    warnings.push(format!("• Match {match_label} is still active."));
                                }
                            }

                            if context.websocket_clients > 0 {
                                warnings.push(format!(
                                    "• {} overlay client(s) are still connected.",
                                    context.websocket_clients
                                ));
                            }

                            if warnings.is_empty() {
                                warnings.push("• Active tasks detected.".to_string());
                            }

                            let message = format!(
                                "The application detected ongoing tasks:\n{}\n\nShutting down now may interrupt recordings or live data feeds. Do you want to exit anyway?",
                                warnings.join("\n")
                            );
                            let (tx, rx) = oneshot::channel();
                            window
                                .dialog()
                                .message(message)
                                .title("Confirm Shutdown")
                                .kind(MessageDialogKind::Warning)
                                .buttons(MessageDialogButtons::OkCancelCustom(
                                    "Shut Down".to_string(),
                                    "Cancel".to_string(),
                                ))
                                .show(move |confirmed| {
                                    let _ = tx.send(confirmed);
                                });

                            let confirmed = rx.await.unwrap_or(false);
                            if !confirmed {
                                app.cancel_shutdown();
                                return;
                            }
                        }

                        if let Err(err) = app.stop().await {
                            log::error!("Failed to stop application cleanly: {err}");
                            app.cancel_shutdown();
                            return;
                        }

                        if let Err(err) = window.close() {
                            log::warn!("Failed to close main window gracefully: {err}");
                        }
                        window.app_handle().exit(0);
                    });
                }
            }
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Core app commands
            // tauri_commands::get_app_status,
            // tauri_commands::shutdown_app,
            // UDP commands
            // tauri_commands::start_udp_server,
            // tauri_commands::stop_udp_server,
            // tauri_commands::get_udp_status,
            // tauri_commands::update_udp_settings,
            // OpenAPI management
            re_strike_vta::tauri_commands_openapi::openapi_get_state,
            re_strike_vta::tauri_commands_openapi::openapi_save_schema,
            re_strike_vta::tauri_commands_openapi::openapi_validate_schema,
            re_strike_vta::tauri_commands_openapi::openapi_upload_schema,
            re_strike_vta::tauri_commands_openapi::openapi_export_schema,
            commands::medal_ceremony::medal_ceremony_list_divisions,
            commands::medal_ceremony::medal_ceremony_list_athletes,
            commands::medal_ceremony::medal_ceremony_list,
            commands::medal_ceremony::medal_ceremony_get,
            commands::medal_ceremony::medal_ceremony_save,
            commands::medal_ceremony::medal_ceremony_delete,
            commands::medal_ceremony::medal_ceremony_prepare,
            commands::medal_ceremony::medal_ceremony_mark_division_played,
            commands::medal_ceremony::medal_ceremony_reset_playback,
            commands::medal_ceremony::medal_ceremony_set_show_external,
            commands::medal_ceremony::medal_ceremony_list_flag_assets,
            commands::medal_ceremony::medal_ceremony_save_flag_asset,
            commands::medal_ceremony::medal_ceremony_delete_flag_asset,
            commands::medal_ceremony::medal_ceremony_list_anthems,
            commands::medal_ceremony::medal_ceremony_save_anthem,
            commands::medal_ceremony::medal_ceremony_delete_anthem,
            // OBS commands - Fixed names
            // tauri_commands::obs_connect,
            // tauri_commands::obs_disconnect,
            // tauri_commands::obs_remove_connection,
            // tauri_commands::obs_get_obs_version,
            // Control Room Commands - Using new async implementation
            // tauri_commands::control_room_authenticate_async,
            // tauri_commands::control_room_get_obs_connections,
            // tauri_commands::control_room_get_obs_connections_with_status,
            // tauri_commands::control_room_get_obs_connections_with_details,
            // tauri_commands::control_room_add_obs_connection,
            // tauri_commands::control_room_connect_obs,
            // tauri_commands::control_room_disconnect_obs,
            // tauri_commands::control_room_remove_obs_connection,
            // tauri_commands::control_room_get_obs_connection,
            // tauri_commands::control_room_update_obs_connection,
            // tauri_commands::control_room_connect_all_obs,
            // tauri_commands::control_room_disconnect_all_obs,
            // tauri_commands::control_room_change_password,
            // tauri_commands::control_room_get_audit_log,
            // tauri_commands::control_room_get_session_info,
            // tauri_commands::control_room_refresh_session,
            // tauri_commands::control_room_logout,
            // tauri_commands::control_room_mute_all_obs,
            // tauri_commands::control_room_unmute_all_obs,
            // tauri_commands::control_room_change_all_obs_scenes,
            // tauri_commands::control_room_start_all_obs,
            // tauri_commands::control_room_stop_all_obs,
            // tauri_commands::control_room_get_audio_sources,
            // tauri_commands::control_room_get_scenes,
            // tauri_commands::control_room_execute_custom_operation,
            // tauri_commands::control_room_execute_raw_request,
            // YouTube Streaming Management Commands (feature-gated)
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_accounts,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_channels,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_stream_key,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_set_youtube_streaming_config,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_categories,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_privacy_options,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_latency_options,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_server_urls,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_regenerate_youtube_stream_key,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_streaming_analytics,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_streaming_schedule,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_create_youtube_streaming_schedule,
            // YouTube API commands (feature-gated)
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_get_auth_url,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_authenticate,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_create_playlist,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_get_playlists,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_add_video_to_playlist,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_get_playlist_videos,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_update_playlist,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_delete_playlist,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_create_scheduled_stream,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_get_live_streams,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_get_scheduled_streams,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_get_completed_streams,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_end_stream,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_get_channel_info,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_get_video_analytics,
            #[cfg(feature = "youtube")]
            // tauri_commands::youtube_initialize,
            // Other Streaming Destinations Commands (removed)
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
            tauri_commands_obws::obs_obws_get_connection_role,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_set_connection_role,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_start_monitoring,
            #[cfg(feature = "obs-obws")]
            tauri_commands_obws::obs_obws_stop_monitoring,
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
            // tauri_commands_obws::ivr_get_replay_settings, // Now in commands::obs::ivr_get_replay_settings
            tauri_commands_obws::ivr_save_replay_settings,
            tauri_commands_obws::ivr_round_replay_now,
            tauri_commands_obws::ivr_open_event_video,
            tauri_commands_obws::ivr_match_history_snapshot,
            tauri_commands_obws::ivr_open_video_file,
            tauri_commands_obws::ivr_validate_mpv_path,
            tauri_commands_obws::obs_obws_get_record_directory,
            tauri_commands_obws::obs_obws_get_filename_formatting,
            // tauri_commands::control_room_get_obs_connection,
            // tauri_commands::control_room_update_obs_connection,
            // tauri_commands::control_room_connect_all_obs,
            // tauri_commands::control_room_disconnect_all_obs,
            // tauri_commands::control_room_change_password,
            // tauri_commands::control_room_get_audit_log,
            // tauri_commands::control_room_get_session_info,
            // tauri_commands::control_room_refresh_session,
            // tauri_commands::control_room_logout,
            // tauri_commands::control_room_mute_all_obs,
            // tauri_commands::control_room_unmute_all_obs,
            // tauri_commands::control_room_change_all_obs_scenes,
            // tauri_commands::control_room_start_all_obs,
            // tauri_commands::control_room_stop_all_obs,
            // tauri_commands::control_room_get_audio_sources,
            // tauri_commands::control_room_get_scenes,
            // tauri_commands::control_room_execute_custom_operation,
            // tauri_commands::control_room_execute_raw_request,
            // YouTube Streaming Management Commands
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_accounts,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_channels,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_stream_key,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_set_youtube_streaming_config,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_categories,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_privacy_options,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_latency_options,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_server_urls,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_regenerate_youtube_stream_key,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_streaming_analytics,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_get_youtube_streaming_schedule,
            #[cfg(feature = "youtube")]
            // tauri_commands::obs_create_youtube_streaming_schedule,
            // Other Streaming Destinations Commands (removed)
            // tauri_commands::obs_command,
            // tauri_commands::obs_connect_to_connection,
            // tauri_commands::obs_get_connection_status,
            // tauri_commands::obs_get_connections,
            // tauri_commands::obs_emit_event,
            // tauri_commands::obs_setup_status_listener,
            // Recording Path and Filename Commands (obws-backed where applicable)
            // tauri_commands::obs_get_recording_path_settings,
            // tauri_commands::obs_set_recording_path,
            // tauri_commands::obs_set_recording_filename,
            // Replay Buffer Settings Commands (legacy removed)

            // Advanced Replay Buffer Commands (legacy removed)

            // Replay Buffer Options Commands (legacy removed)

            // WebSocket commands for HTML overlays
            // tauri_commands::websocket_get_status,
            // tauri_commands::websocket_broadcast_pss_event,
            // tauri_commands::store_pss_event_cmd,
            // Video commands - Fixed names
            // tauri_commands::video_play,
            // tauri_commands::video_stop,
            // tauri_commands::video_get_info,
            // tauri_commands::extract_clip,
            // PSS commands
            // tauri_commands::pss_start_listener,
            // tauri_commands::pss_stop_listener,
            // tauri_commands::pss_get_events,
            // tauri_commands::pss_get_events_for_match,
            // tauri_commands::pss_list_recent_matches,
            // tauri_commands::pss_get_current_match,
            // tauri_commands::pss_get_match_details,
            // tauri_commands::pss_clear_all_data,
            // tauri_commands::pss_emit_event,
            // tauri_commands::pss_emit_pending_events,
            // tauri_commands::pss_setup_event_listener,
            // System commands
            // tauri_commands::system_get_info,
            // tauri_commands::get_network_interfaces,
            // tauri_commands::get_best_network_interface,
            // tauri_commands::get_best_ip_address_for_interface,
            // Store commands
            // tauri_commands::save_event,
            // tauri_commands::get_events,
            // tauri_commands::clear_events,
            // License commands
            // tauri_commands::activate_license,
            // tauri_commands::validate_license,
            // tauri_commands::get_license_status,
            // tauri_commands::get_machine_identity,
            // Settings commands
            // tauri_commands::get_settings,
            // tauri_commands::update_settings,
            // tauri_commands::get_config_stats,
            // tauri_commands::reset_settings,
            // tauri_commands::export_settings,
            // tauri_commands::import_settings,
            // tauri_commands::restore_settings_backup,
            // Flag commands
            // tauri_commands::get_flag_url,
            // tauri_commands::download_flags,
            // Diagnostics & Logs commands - Fixed names
            // tauri_commands::list_log_files,
            // tauri_commands::download_log_file,
            // tauri_commands::list_archives,
            // tauri_commands::extract_archive,
            // tauri_commands::download_archive,
            // tauri_commands::set_live_data_streaming,
            // New Log Archive & Google Drive commands
            commands::logging::create_complete_log_archive,
            commands::logging::create_and_upload_log_archive,
            commands::logging::create_upload_and_cleanup_log_archive,
            commands::logging::get_auto_archive_config,
            commands::logging::set_auto_archive_config,
            commands::logging::check_auto_archive_status,
            commands::logging::perform_auto_archive,
            commands::logging::delete_log_archive,
            // Legacy commands for backward compatibility
            // tauri_commands::start_live_data,
            // tauri_commands::stop_live_data,
            // tauri_commands::get_live_data,
            // tauri_commands::obs_get_debug_info,
            // tauri_commands::obs_toggle_full_events,
            // tauri_commands::obs_get_full_events_setting,
            // tauri_commands::obs_emit_event_to_frontend,
            // tauri_commands::obs_get_recent_events,
            // CPU Monitoring commands
            // tauri_commands::cpu_get_process_data,
            // tauri_commands::cpu_get_system_data,
            // tauri_commands::cpu_get_obs_usage,
            // tauri_commands::cpu_update_config,
            // tauri_commands::cpu_enable_monitoring,
            // tauri_commands::cpu_disable_monitoring,
            // tauri_commands::cpu_get_monitoring_status,
            // Protocol Management commands
            // tauri_commands::protocol_get_versions,
            // tauri_commands::protocol_set_active_version,
            // tauri_commands::protocol_upload_file,
            // tauri_commands::protocol_delete_version,
            // tauri_commands::protocol_export_file,
            // tauri_commands::protocol_get_current,
            // Window Management commands
            // tauri_commands::set_window_fullscreen,
            // tauri_commands::set_window_compact,
            // tauri_commands::set_window_custom_size,
            // tauri_commands::set_window_position,
            // tauri_commands::set_window_startup_position,
            // tauri_commands::save_window_settings,
            // tauri_commands::load_window_settings,
            // tauri_commands::get_screen_size,
            // Database commands
            // tauri_commands::initialize_ui_settings_database,
            // tauri_commands::db_initialize_ui_settings,
            // tauri_commands::db_get_ui_setting,
            // tauri_commands::db_set_ui_setting,
            // tauri_commands::db_get_all_ui_settings,
            // tauri_commands::db_get_database_info,
            // tauri_commands::migrate_json_to_database,
            // tauri_commands::db_run_migrations,
            // tauri_commands::get_migration_status,
            // tauri_commands::enable_database_mode,
            // tauri_commands::get_database_preview,
            // tauri_commands::get_database_tables,
            // tauri_commands::get_table_data,
            commands::resources::get_flag_mappings_data,
            // Tournament Management commands
            commands::tournament::tournament_create,
            commands::tournament::tournament_import_from_directory,
            commands::tournament::tournament_get_all,
            commands::tournament::tournament_get,
            commands::tournament::tournament_update,
            commands::tournament::tournament_delete,
            commands::tournament::tournament_get_days,
            commands::tournament::tournament_start_day,
            commands::tournament::tournament_end_day,
            commands::tournament::tournament_get_active,
            commands::tournament::tournament_get_active_day,
            commands::tournament::tournament_update_logo,
            commands::tournament::tournament_verify_location,
            commands::tournament::get_tournament_statistics,
            // OVR commands
            commands::ovr::ovr_get_providers,
            commands::ovr::ovr_upsert_provider,
            commands::ovr::ovr_remove_provider,
            commands::ovr::ovr_list_tournaments,
            commands::ovr::ovr_get_categories,
            commands::ovr::ovr_promote_tournament,
            commands::ovr::ovr_refresh_all,
            commands::ovr::ovr_refresh_provider,
            commands::ovr::ovr_clear_all_tournaments,
            commands::ovr::ovr_start_refresh_all,
            commands::ovr::ovr_start_refresh_provider,
            commands::ovr::ovr_get_refresh_status,
            commands::ovr::ovr_cancel_refresh,
            // Control Room Commands
            commands::control_room::control_room_authenticate_async,
            commands::control_room::control_room_get_obs_connections,
            commands::control_room::control_room_get_obs_connections_with_status,
            commands::control_room::control_room_get_obs_connections_with_details,
            commands::control_room::control_room_mute_all_obs,
            commands::control_room::control_room_unmute_all_obs,
            commands::control_room::control_room_change_all_obs_scenes,
            commands::control_room::control_room_start_all_obs,
            commands::control_room::control_room_stop_all_obs,
            commands::control_room::control_room_add_obs_connection,
            commands::control_room::control_room_connect_obs,
            commands::control_room::control_room_disconnect_obs,
            commands::control_room::control_room_remove_obs_connection,
            commands::control_room::control_room_get_obs_connection,
            commands::control_room::control_room_update_obs_connection,
            commands::control_room::control_room_connect_all_obs,
            commands::control_room::control_room_disconnect_all_obs,
            commands::control_room::control_room_get_audio_sources,
            commands::control_room::control_room_execute_custom_operation,
            commands::control_room::control_room_execute_raw_request,
            commands::control_room::control_room_get_scenes,
            commands::control_room::control_room_change_password,
            commands::control_room::control_room_get_audit_log,
            commands::control_room::control_room_get_session_info,
            commands::control_room::control_room_refresh_session,
            commands::control_room::control_room_logout,
            // PSS Commands
            commands::pss::store_pss_event_cmd,
            // DB backup commands (reuse existing helpers)
            // tauri_commands::db_create_sqlite_backup,
            // tauri_commands::db_list_sqlite_backups,
            // tauri_commands::db_restore_sqlite_backup,
            commands::database::database_run_vacuum,
            commands::database::database_run_integrity_check,
            commands::database::database_run_analyze,
            commands::database::database_run_optimize,
            commands::database::database_run_full_maintenance,
            // // tauri_commands::database_get_info, // Removed as missing
            commands::database::database_get_maintenance_status,
            // Event Status Analysis commands
            commands::database::get_comprehensive_event_statistics,
            commands::database::get_events_by_status,
            commands::database::get_unknown_events,
            // Tournament Context Management commands
            commands::udp::set_udp_tournament_context,
            commands::udp::get_udp_tournament_context,
            commands::udp::clear_udp_tournament_context,
            commands::udp::get_udp_performance_metrics,
            commands::udp::get_udp_memory_usage,
            // Phase 2 Optimization - Data Archival commands
            // tauri_commands::archive_old_events,
            // tauri_commands::get_archive_statistics,
            // tauri_commands::restore_from_archive,
            // tauri_commands::cleanup_old_archive_data,
            // tauri_commands::optimize_archive_tables,
            // tauri_commands::get_database_pool_stats,
            // tauri_commands::cleanup_database_pool,
            // Flag management commands
            commands::resources::scan_and_populate_flags,
            commands::resources::get_flags_data,
            commands::resources::clear_flags_table,
            // Google Drive commands
            // tauri_commands::drive_request_auth_url,
            // tauri_commands::drive_complete_auth,
            // tauri_commands::drive_save_credentials,
            // tauri_commands::drive_list_files,
            // tauri_commands::drive_list_all_files,
            // tauri_commands::drive_test_connection,
            // tauri_commands::drive_upload_backup_archive,
            // tauri_commands::drive_download_backup_archive,
            // tauri_commands::drive_restore_from_archive,
            // tauri_commands::drive_delete_backup_archive,
            // tauri_commands::drive_get_connection_status,
            // tauri_commands::drive_get_quota,
            // tauri_commands::drive_create_folder,
            // tauri_commands::drive_list_children,
            // tauri_commands::drive_upload_zip_to_folder,
            // Phase 3: Advanced Caching Commands
            // Phase 3: Cache Management Commands - TODO: Implement these
            // // tauri_commands::get_cache_statistics,
            // // tauri_commands::clear_cache,
            // // tauri_commands::invalidate_tournament_cache,
            // // tauri_commands::invalidate_match_cache,
            // Phase 3: Event Stream Commands - TODO: Implement these
            // // tauri_commands::get_stream_statistics,
            // // tauri_commands::send_event_to_stream,
            // Trigger system commands
            commands::triggers::triggers_list_pss_events,
            commands::triggers::triggers_list_obs_scenes,
            commands::triggers::triggers_list_active_overlays,
            commands::triggers::triggers_get,
            commands::triggers::triggers_save,
            commands::triggers::triggers_recent_logs,
            commands::triggers::triggers_preview_evaluate,
            re_strike_vta::tauri_commands_overlays::overlays_sync_templates,
            re_strike_vta::tauri_commands_overlays::overlays_populate_from_files,
            re_strike_vta::tauri_commands_overlays::get_overlay_routing_config,
            re_strike_vta::tauri_commands_overlays::set_overlay_routing_config,
            re_strike_vta::tauri_commands_overlays::close_overlay_window,
            // obs_connections commands - DISABLED - these don't exist in tauri_commands_obs_connections
            // re_strike_vta::tauri_commands_obs_connections::obs_connections_get_all,
            // re_strike_vta::tauri_commands_obs_connections::obs_connections_get_active,
            // re_strike_vta::tauri_commands_obs_connections::obs_connections_save,
            // re_strike_vta::tauri_commands_obs_connections::obs_connections_update_status,
            // re_strike_vta::tauri_commands_obs_connections::obs_connections_delete,
            // re_strike_vta::tauri_commands_obs_connections::obs_connections_clear_all,
            // re_strike_vta::tauri_commands_obs_connections::obs_connections_sync_from_config,
            // Manual Mode Commands
            commands::manual_mode::manual_create_match,
            commands::manual_mode::manual_restore_data,
            commands::manual_mode::manual_get_statistics,
            // OBS Commands
            tauri_commands_obws::obs_obws_get_connections,
            tauri_commands_obws::obs_obws_remove_connection,
            tauri_commands_obws::obs_obws_get_status,
            tauri_commands_obws::obs_obws_start_recording,
            tauri_commands_obws::obs_obws_stop_recording,
            tauri_commands_obws::obs_obws_get_recording_status,
            tauri_commands_obws::obs_obws_start_streaming,
            tauri_commands_obws::obs_obws_stop_streaming,
            tauri_commands_obws::obs_obws_get_streaming_status,
            tauri_commands_obws::obs_obws_get_current_scene,
            tauri_commands_obws::obs_obws_set_current_scene,
            tauri_commands_obws::obs_obws_get_version,
            tauri_commands_obws::obs_obws_save_replay_buffer,
            tauri_commands_obws::obs_obws_get_replay_buffer_status,
            tauri_commands_obws::obs_obws_get_recording_path_settings,
            tauri_commands_obws::obs_obws_set_recording_path,
            tauri_commands_obws::obs_obws_set_recording_filename,
            tauri_commands_obws::obs_obws_set_source_visibility,
            tauri_commands_obws::obs_obws_start_replay_buffer,
            tauri_commands_obws::obs_obws_stop_replay_buffer,
            tauri_commands_obws::obs_obws_create_test_folders,
            tauri_commands_obws::obs_obws_manual_start_recording,
            tauri_commands_obws::obs_obws_manual_stop_recording,
            tauri_commands_obws::obs_obws_generate_recording_path,
            tauri_commands_obws::obs_obws_setup_status_listener,
            tauri_commands_obws::obs_obws_save_full_config,
            tauri_commands_obws::obs_obws_get_filename_formatting,
            tauri_commands_obws::ivr_match_history_snapshot,
            tauri_commands_obws::ivr_round_replay_now,
            tauri_commands_obws::ivr_save_replay_settings,
            // Commands::obs re-exports these from tauri_commands_obws, but pub use doesn't work for tauri commands
            tauri_commands_obws::ivr_open_event_video,
            tauri_commands_obws::ivr_validate_mpv_path,
            tauri_commands_obws::ivr_open_video_file,
            tauri_commands_obws::obs_obws_add_connection,
            commands::obs::obs_update_connection_wrapper,
            commands::obs::obs_connect_impl,
            commands::obs::ivr_get_replay_settings,
            // commands::obs::obs_list_scenes, // Need to check if implemented
            // Store Commands
            commands::store::save_event,
            commands::store::get_events,
            commands::store::clear_events,
            // Phase 3: Load Balancer Commands - TODO: Implement these
            // // tauri_commands::get_distributor_statistics,
            // // tauri_commands::get_server_statistics,
            // // tauri_commands::add_server,
            // // tauri_commands::remove_server,
            // Phase 3: Advanced Analytics Commands - TODO: Implement these
            // // tauri_commands::get_tournament_analytics,
            // // tauri_commands::get_performance_analytics,
            // // tauri_commands::get_athlete_analytics,
            // // tauri_commands::get_match_analytics,
            // // tauri_commands::get_analytics_history,
            // Simulation commands - TODO: Implement these
            // // tauri_commands::simulation_start,
            // // tauri_commands::simulation_stop,
            // // tauri_commands::simulation_get_status,
            // // tauri_commands::simulation_send_event,
            // // tauri_commands::simulation_get_scenarios,
            // tauri_commands::simulation_run_automated,
            // tauri_commands::simulation_get_detailed_status,
            // tauri_commands::simulation_run_self_test,
            // tauri_commands::simulation_get_self_test_report,
            // tauri_commands::simulation_get_self_test_categories,
            // tauri_commands::simulation_run_selective_self_test,
            // YouTube API commands
            // tauri_commands::youtube_get_auth_url,
            // tauri_commands::youtube_authenticate,
            // tauri_commands::youtube_create_playlist,
            // tauri_commands::youtube_get_playlists,
            // tauri_commands::youtube_add_video_to_playlist,
            // tauri_commands::youtube_get_playlist_videos,
            // tauri_commands::youtube_update_playlist,
            // tauri_commands::youtube_delete_playlist,
            // tauri_commands::youtube_create_scheduled_stream,
            // tauri_commands::youtube_get_live_streams,
            // tauri_commands::youtube_get_scheduled_streams,
            // tauri_commands::youtube_get_completed_streams,
            // tauri_commands::youtube_end_stream,
            // tauri_commands::youtube_get_channel_info,
            // tauri_commands::youtube_get_video_analytics,
            // tauri_commands::youtube_initialize,
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
        .run(tauri::generate_context!());

    if let Err(err) = tauri_result {
        log::error!("Tauri runtime exited with an error: {err}");
        return Err(AppError::ConfigError(format!(
            "Tauri runtime failed: {err}"
        )));
    }

    Ok(())
}
