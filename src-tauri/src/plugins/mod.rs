// Plugins module - Central registry for all application plugins
// This module manages the lifecycle and coordination of all plugins

pub mod plugin_udp;
pub mod plugin_store;
pub mod plugin_playback;
pub mod plugin_tournament;
pub mod plugin_triggers;
pub mod plugin_websocket;
pub mod plugin_database;
pub mod plugin_drive;
pub mod plugin_license;
pub mod plugin_cpu_monitor;
pub mod plugin_protocol_manager;
pub mod plugin_obs; // Keep old plugin for now during transition
pub mod load_balancer;
pub mod advanced_analytics;
pub mod obs; // Add modular OBS plugins

// Re-export main plugin types
pub use plugin_udp::UdpPlugin;
pub use plugin_store::StorePlugin;
pub use plugin_playback::PlaybackPlugin;
pub use plugin_tournament::TournamentPlugin;
pub use plugin_triggers::TriggersPlugin;
pub use plugin_websocket::WebSocketPlugin;
pub use plugin_database::DatabasePlugin;
pub use plugin_drive::DrivePlugin;
pub use plugin_license::LicensePlugin;
pub use plugin_cpu_monitor::CpuMonitorPlugin;
pub use plugin_protocol_manager::ProtocolManagerPlugin;
pub use plugin_obs::ObsPlugin; // Keep old plugin for now
pub use load_balancer::{EventDistributor, LoadBalancer, LoadBalancerConfig, LoadDistributionStrategy, ServerHealth, ServerStatistics, DistributorStatistics, UdpServerInstance};
pub use advanced_analytics::{AdvancedAnalytics, AnalyticsConfig, TournamentAnalytics, PerformanceAnalytics, AthleteAnalytics, MatchAnalytics, AnalyticsSnapshot, AthletePerformance, SystemPerformance, EventProcessingPerformance, DatabasePerformance, CachePerformance, NetworkPerformance, MatchPerformance, PerformancePoint, MatchPerformancePoint};
// Re-export modular OBS plugins
pub use obs::{ObsPluginManager, ObsCorePlugin, ObsRecordingPlugin, ObsStreamingPlugin, ObsScenesPlugin, ObsSettingsPlugin, ObsEventsPlugin, ObsStatusPlugin};

/// Initialize all plugins
pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Initializing all plugins...");
    
    // Initialize core plugins
    plugin_udp::init()?;           // UDP PSS event handling
    plugin_store::init()?;         // Data storage and caching
    plugin_playback::init()?;      // Video playback and replay
    plugin_tournament::init()?;    // Tournament management system
    plugin_triggers::init()?;      // Trigger system for PSS events
    plugin_websocket::init()?;     // WebSocket server for frontend
    plugin_database::init()?;      // Database operations
    plugin_drive::init()?;         // Drive integration
    plugin_license::init()?;       // License management
    plugin_cpu_monitor::init()?;   // CPU monitoring
    plugin_protocol_manager::init()?; // Protocol management
    plugin_obs::init()?;           // Initialize old OBS plugin (for transition)
    obs::init()?;                  // Initialize modular OBS plugin system
    
    println!("✅ All plugins initialized successfully");
    Ok(())
}

/// Shutdown all plugins
pub async fn shutdown() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Shutting down all plugins...");
    
    // Shutdown plugins in reverse order
    obs::shutdown().await?;        // Shutdown modular OBS plugin system
    // Note: Individual plugin shutdown methods may not exist yet
    // plugin_obs::shutdown()?;       // Shutdown old OBS plugin
    // plugin_protocol_manager::shutdown()?;
    // plugin_cpu_monitor::shutdown()?;
    // plugin_license::shutdown()?;
    // plugin_drive::shutdown()?;
    // plugin_database::shutdown()?;
    // plugin_websocket::shutdown()?;
    // plugin_triggers::shutdown()?;
    // plugin_tournament::shutdown()?;
    // plugin_playback::shutdown()?;
    // plugin_store::shutdown()?;
    // plugin_udp::shutdown()?;
    
    println!("✅ All plugins shut down successfully");
    Ok(())
} 