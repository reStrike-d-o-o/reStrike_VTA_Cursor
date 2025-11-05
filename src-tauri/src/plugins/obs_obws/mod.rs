//! OBS WebSocket integration using the obws crate
//!
//! This module provides a native Rust implementation for OBS WebSocket integration
//! using the obws crate, which offers type-safe API access to OBS Studio.

pub mod client;
pub mod manager;
pub mod operations;
pub mod path_generator;
pub mod recording_events;
pub mod test_implementation;
pub mod types;

use crate::types::AppResult;
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

// Re-export main types for easier access
pub use client::ObsClient;
pub use manager::ObsManager; // Re-export ObsManager for external use
pub use path_generator::{GeneratedPath, ObsPathGenerator, PathGeneratorConfig};
pub use recording_events::{
    AutomaticRecordingConfig, ObsRecordingEventHandler, RecordingEvent, RecordingSession,
    RecordingState,
};
pub use types::*;

/// Global OBS manager instance using thread-safe singleton pattern without unsafe
static MANAGER: OnceLock<Arc<Mutex<ObsManager>>> = OnceLock::new();

/// Initialize the OBS WebSocket plugin
pub fn init() -> AppResult<()> {
    MANAGER.get_or_init(|| Arc::new(Mutex::new(ObsManager::new())));
    log::info!("OBS WebSocket plugin initialized");
    Ok(())
}

/// Shutdown the OBS WebSocket plugin
#[allow(clippy::await_holding_lock)]
pub async fn shutdown() -> AppResult<()> {
    if let Some(manager) = MANAGER.get() {
        if let Ok(manager) = manager.lock() {
            manager.shutdown().await?;
        }
    }
    log::info!("OBS WebSocket plugin shutdown");
    Ok(())
}

/// Get a reference to the OBS manager
pub fn get_manager() -> Option<Arc<Mutex<ObsManager>>> {
    MANAGER.get().map(Arc::clone)
}
