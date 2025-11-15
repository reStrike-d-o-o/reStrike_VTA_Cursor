#![allow(clippy::uninlined_format_args)] // TODO: migrate logging calls to captured format args

//! reStrike VTA - Windows Desktop Application Library
//!
//! This library provides the core functionality for the taekwondo referee application
//! including OBS integration, video playback, PSS protocol handling, and database management.

pub mod commands;
pub mod config;
pub mod core;
pub mod database;
pub mod entity;
pub mod importers;
pub mod logging;
pub mod openapi;
pub mod plugins;
pub mod pss;
pub mod security;
pub mod tauri_commands;
pub mod tauri_commands_manual_mode;
pub mod tauri_commands_medal_ceremony;
pub mod tauri_commands_obs_connections;
#[cfg(feature = "obs-obws")]
pub mod tauri_commands_obws;
pub mod tauri_commands_openapi;
pub mod tauri_commands_overlays;
pub mod tauri_commands_security;
pub mod tauri_commands_triggers;
pub mod types;
pub mod ui_settings;
pub mod utils;

// Re-export commonly used items
pub use core::app::App;
// Old ObsPlugin removed - using modular ObsPluginManager
pub use plugins::plugin_playback::PlaybackPlugin;
pub use plugins::plugin_udp::UdpPlugin;
pub use types::*;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "reStrike VTA";

/// Initialize the application library
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Initializing {APP_NAME} v{VERSION}");

    // Initialize core systems
    core::init()?;

    log::info!("{APP_NAME} library initialized successfully");
    Ok(())
}
