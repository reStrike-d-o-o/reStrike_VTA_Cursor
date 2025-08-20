//! OBS WebSocket integration using the obws crate
//!
//! Native Rust implementation with type-safe API access to OBS Studio.
//! Multi-connection manager, clients, operations, path generation, and
//! automatic recording/IVR indexing live here.

pub mod client;
pub mod manager;
pub mod types;
pub mod operations;
pub mod test_implementation;
pub mod path_generator;
pub mod recording_events;

use crate::types::AppResult;
use std::sync::{Arc, Mutex};
use std::sync::OnceLock;

// Re-export main types for easier access
pub use client::ObsClient;
pub use manager::ObsManager;  // Re-export ObsManager for external use
pub use types::*;
pub use path_generator::{ObsPathGenerator, PathGeneratorConfig, GeneratedPath};
pub use recording_events::{
    ObsRecordingEventHandler, RecordingSession, RecordingState, 
    AutomaticRecordingConfig, RecordingEvent
};

/// Global OBS manager instance using thread-safe singleton pattern without unsafe
static MANAGER: OnceLock<Arc<Mutex<ObsManager>>> = OnceLock::new();

/// Initialize the OBS WebSocket plugin
pub fn init() -> AppResult<()> {
    MANAGER.get_or_init(|| Arc::new(Mutex::new(ObsManager::new())));
    log::info!("✅ OBS WebSocket plugin initialized");
    Ok(())
}

/// Shutdown the OBS WebSocket plugin
pub async fn shutdown() -> AppResult<()> {
    if let Some(manager) = MANAGER.get() {
        if let Ok(manager) = manager.lock() {
            manager.shutdown().await?;
        }
    }
    log::info!("✅ OBS WebSocket plugin shutdown");
    Ok(())
}

/// Get a reference to the OBS manager
pub fn get_manager() -> Option<Arc<Mutex<ObsManager>>> {
    MANAGER.get().map(Arc::clone)
}
