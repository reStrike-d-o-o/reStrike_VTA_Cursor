//! reStrike VTA - Windows Desktop Application Library
//!
//! This library provides the core functionality for the taekwondo referee application
//! including OBS integration, video playback, PSS protocol handling, and database management.

pub mod commands;
pub mod config;
pub mod core;
pub mod database;
pub mod types;
pub mod plugins;
pub mod tauri_commands;
pub mod tauri_commands_triggers;
pub mod tauri_commands_overlays;
pub mod tauri_commands_obs_connections;
pub mod logging;
pub mod utils;
pub mod ui_settings;

// Re-export commonly used items
pub use core::app::App;
pub use plugins::plugin_obs::ObsPlugin;
pub use plugins::plugin_playback::PlaybackPlugin;
pub use plugins::plugin_udp::UdpPlugin;
pub use types::*;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "reStrike VTA";

/// Initialize the application library
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Initializing {} v{}", APP_NAME, VERSION);

    // Initialize core systems
    core::init()?;

    println!("✅ {} library initialized successfully", APP_NAME);
    Ok(())
}
