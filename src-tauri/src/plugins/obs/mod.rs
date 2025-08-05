// OBS Plugin Modular Structure
// This module provides a modular approach to OBS WebSocket integration
// Breaking down the monolithic 1366-line plugin_obs.rs into focused modules

pub mod types;
pub mod manager;
pub mod core;
pub mod recording;
pub mod streaming;
pub mod scenes;
pub mod settings;
pub mod events;
pub mod status;

// Re-export main types for easy access
pub use types::*;
pub use manager::ObsPluginManager;
pub use core::ObsCorePlugin;
pub use recording::ObsRecordingPlugin;
pub use streaming::ObsStreamingPlugin;
pub use scenes::ObsScenesPlugin;
pub use settings::ObsSettingsPlugin;
pub use events::ObsEventsPlugin;
pub use status::ObsStatusPlugin;

// Initialize the modular OBS plugin system
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Initializing modular OBS plugin system...");
    Ok(())
} 