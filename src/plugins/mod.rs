// Plugin modules
pub mod plugin_license;
pub mod plugin_obs;
pub mod plugin_playback;
pub mod plugin_store;
pub mod plugin_udp;

// Re-export key OBS types for easier access
pub use plugin_obs::{ObsPlugin, ObsConnectionConfig, ObsWebSocketVersion, ObsStatusInfo, ObsEvent}; 