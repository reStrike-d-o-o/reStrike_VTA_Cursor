// Plugin modules
pub mod plugin_database;  // Re-enabled for Phase 2
pub mod plugin_drive;     // Google Drive integration
pub mod plugin_license;
pub mod plugin_obs;
pub mod plugin_playback;
pub mod plugin_store;
pub mod plugin_udp;
pub mod plugin_cpu_monitor;
pub mod plugin_protocol_manager;

// Re-export key plugin types for easier access
pub use plugin_database::{DatabasePlugin, DatabaseStatistics};  // Re-enabled for Phase 2
pub use plugin_drive::drive_plugin;  // Google Drive integration
pub use plugin_obs::{ObsPlugin, ObsConnectionConfig, ObsWebSocketVersion, ObsStatusInfo, ObsEvent};
pub use plugin_playback::PlaybackPlugin;
pub use plugin_udp::UdpPlugin;
pub use plugin_store::StorePlugin;
pub use plugin_license::LicensePlugin;
pub use plugin_cpu_monitor::{CpuMonitorPlugin, CpuMonitorConfig, CpuProcessData, SystemCpuData};
pub use plugin_protocol_manager::{ProtocolManager, ProtocolFile, ProtocolVersion, StreamDefinition};

/// Initialize all plugins
pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing plugins...");
    
    // Initialize each plugin
    plugin_database::init()?;
    plugin_drive::init()?;  // Google Drive integration
    plugin_obs::init()?;
    plugin_playback::init()?;
    plugin_udp::init()?;
    plugin_store::init()?;
    plugin_license::init()?;
    plugin_cpu_monitor::init()?;
    plugin_protocol_manager::init()?;
    
    println!("✅ All plugins initialized successfully");
    Ok(())
} 