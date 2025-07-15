//! reStrike VTA - Windows Desktop Application Library
//!
//! This library provides the core functionality for the taekwondo referee application
//! including OBS integration, video playback, and PSS protocol handling.

pub mod commands;
pub mod core;
// pub mod obs;
// pub mod pss;
pub mod types;
// pub mod utils;
// pub mod video;
pub mod plugins;

// Re-export commonly used items
pub use core::app::App;
pub use obs::manager::ObsManager;
pub use video::player::VideoPlayer;
pub use pss::protocol::PssProtocol;
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
    // obs::init()?;
    // video::init()?;
    // pss::init()?;

    println!("✅ {} library initialized successfully", APP_NAME);
    Ok(())
}
