// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use re_strike_vta::core::app::App;
use re_strike_vta::tauri_commands::{self, LoggingStateType};
use re_strike_vta::types::AppResult;
use std::sync::Arc;
use tauri::Manager;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging
    env_logger::init();
    
    log::info!("Starting reStrike VTA Tauri Application");
    
    // Create the main application instance
    let app = Arc::new(App::new().await?);
    
    // Start the application
    app.start().await?;
    
    // Create Tauri app builder
    tauri::Builder::default()
        .manage(app)
        .manage(Arc::new(std::sync::Mutex::new(tauri_commands::LoggingState::default())))
        .invoke_handler(tauri::generate_handler![
            // Core commands
            tauri_commands::get_app_status,
            tauri_commands::shutdown_app,
            
            // UDP commands
            tauri_commands::start_udp_server,
            tauri_commands::stop_udp_server,
            tauri_commands::get_udp_status,
            
            // OBS commands
            tauri_commands::connect_obs,
            tauri_commands::disconnect_obs,
            tauri_commands::get_obs_status,
            tauri_commands::obs_command,
            
            // Video commands
            tauri_commands::play_video,
            tauri_commands::extract_clip,
            
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
            
            // Flag commands
            tauri_commands::get_flag_url,
            tauri_commands::download_flags,
            
            // Diagnostics & Logs commands
            tauri_commands::enable_logging,
            tauri_commands::disable_logging,
            tauri_commands::list_log_files,
            tauri_commands::download_log_file,
            tauri_commands::start_live_data,
            tauri_commands::stop_live_data,
        ])
        .setup(|app| {
            log::info!("Tauri application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    Ok(())
}


