//! reStrike VTA - Windows Desktop Application Library
//!
//! This library provides the core functionality for the taekwondo referee application
//! including OBS integration, video playback, and PSS protocol handling.

pub mod commands;
pub mod core;
pub mod types;
pub mod plugins;
pub mod tauri_commands;
pub mod logging;

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
    plugins::init()?;

    println!("✅ {} library initialized successfully", APP_NAME);
    Ok(())
}
