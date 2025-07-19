// Plugin modules
pub mod plugin_license;
pub mod plugin_obs;
pub mod plugin_playback;
pub mod plugin_store;
pub mod plugin_udp;
pub mod plugin_cpu_monitor;

// Re-export key plugin types for easier access
pub use plugin_obs::{ObsPlugin, ObsConnectionConfig, ObsWebSocketVersion, ObsStatusInfo, ObsEvent};
pub use plugin_playback::PlaybackPlugin;
pub use plugin_udp::UdpPlugin;
pub use plugin_store::StorePlugin;
pub use plugin_license::LicensePlugin;
pub use plugin_cpu_monitor::{CpuMonitorPlugin, CpuMonitorConfig, CpuProcessData, SystemCpuData};

/// Initialize all plugins
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing plugins...");
    
    // Initialize each plugin
    plugin_obs::init()?;
    plugin_playback::init()?;
    plugin_udp::init()?;
    plugin_store::init()?;
    plugin_license::init()?;
    plugin_cpu_monitor::init()?;
    
    println!("✅ All plugins initialized successfully");
    Ok(())
} 