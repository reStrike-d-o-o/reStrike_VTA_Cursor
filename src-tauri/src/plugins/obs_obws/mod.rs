//! OBS WebSocket integration using the obws crate
//! 
//! This module provides a native Rust implementation for OBS WebSocket integration
//! using the obws crate, which offers type-safe API access to OBS Studio.

pub mod client;
pub mod manager;
pub mod types;
pub mod operations;
pub mod test_implementation;

use crate::types::AppResult;
use manager::ObsManager;

// Re-export main types for easier access
pub use client::ObsClient;
pub use manager::ObsManager;
pub use types::*;

/// Global OBS manager instance
static mut OBS_MANAGER: Option<ObsManager> = None;

/// Initialize the OBS WebSocket plugin
pub fn init() -> AppResult<()> {
    unsafe {
        OBS_MANAGER = Some(ObsManager::new());
    }
    log::info!("✅ OBS WebSocket plugin initialized");
    Ok(())
}

/// Shutdown the OBS WebSocket plugin
pub async fn shutdown() -> AppResult<()> {
    unsafe {
        if let Some(manager) = &OBS_MANAGER {
            manager.shutdown().await?;
        }
        OBS_MANAGER = None;
    }
    log::info!("✅ OBS WebSocket plugin shutdown");
    Ok(())
}

/// Get a reference to the OBS manager
pub fn get_manager() -> Option<&'static ObsManager> {
    unsafe {
        OBS_MANAGER.as_ref()
    }
}
