//! OBS WebSocket integration using the obws crate
//! 
//! This module provides a native Rust implementation for OBS WebSocket integration
//! using the obws crate, which offers type-safe API access to OBS Studio.

pub mod client;
pub mod manager;
pub mod types;
pub mod operations;
pub mod test_implementation;
pub mod path_generator;
pub mod recording_events;

use crate::types::AppResult;
use std::sync::{Arc, Mutex, Once};

// Re-export main types for easier access
pub use client::ObsClient;
pub use manager::ObsManager;  // Re-export ObsManager for external use
pub use types::*;
pub use path_generator::{ObsPathGenerator, PathGeneratorConfig, GeneratedPath};
pub use recording_events::{
    ObsRecordingEventHandler, RecordingSession, RecordingState, 
    AutomaticRecordingConfig, RecordingEvent
};

/// Global OBS manager instance using thread-safe singleton pattern
static INSTANCE: Once = Once::new();
static mut MANAGER: Option<Arc<Mutex<ObsManager>>> = None;

/// Initialize the OBS WebSocket plugin
pub fn init() -> AppResult<()> {
    INSTANCE.call_once(|| {
        unsafe {
            MANAGER = Some(Arc::new(Mutex::new(ObsManager::new())));
        }
    });
    log::info!("✅ OBS WebSocket plugin initialized");
    Ok(())
}

/// Shutdown the OBS WebSocket plugin
pub async fn shutdown() -> AppResult<()> {
    if let Some(manager) = unsafe { MANAGER.as_ref() } {
        if let Ok(manager) = manager.lock() {
            manager.shutdown().await?;
        }
    }
    unsafe {
        MANAGER = None;
    }
    log::info!("✅ OBS WebSocket plugin shutdown");
    Ok(())
}

/// Get a reference to the OBS manager
pub fn get_manager() -> Option<Arc<Mutex<ObsManager>>> {
    unsafe {
        MANAGER.as_ref().map(Arc::clone)
    }
}
