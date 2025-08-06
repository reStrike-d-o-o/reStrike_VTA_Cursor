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
pub mod control_room;

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
pub use control_room::ControlRoomManager;

// Global instance of the OBS Plugin Manager
use std::sync::OnceLock;
static OBS_PLUGIN_MANAGER: OnceLock<ObsPluginManager> = OnceLock::new();

// Initialize the modular OBS plugin system
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Initializing modular OBS plugin system...");
    
    // Create the plugin manager
    let manager = ObsPluginManager::new()?;
    
    // Store the manager globally
    if let Err(_) = OBS_PLUGIN_MANAGER.set(manager) {
        log::warn!("OBS Plugin Manager already initialized");
        return Ok(());
    }
    
    // Initialize the manager (this will initialize all plugins)
    if let Some(manager) = OBS_PLUGIN_MANAGER.get() {
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            if let Err(e) = manager_clone.init().await {
                log::error!("Failed to initialize OBS Plugin Manager: {}", e);
            }
        });
    }
    
    log::info!("✅ Modular OBS plugin system initialized");
    Ok(())
}

// Shutdown the modular OBS plugin system
pub async fn shutdown() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Shutting down modular OBS plugin system...");
    
    if let Some(manager) = OBS_PLUGIN_MANAGER.get() {
        manager.shutdown().await?;
    }
    
    log::info!("✅ Modular OBS plugin system shut down");
    Ok(())
}

// Get a reference to the OBS Plugin Manager
pub fn get_obs_plugin_manager() -> Option<&'static ObsPluginManager> {
    OBS_PLUGIN_MANAGER.get()
} 