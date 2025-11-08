// Plugins module - Central registry for all application plugins
// This module manages the lifecycle and coordination of all plugins

pub mod plugin_cpu_monitor;
pub mod plugin_database;
pub mod plugin_drive;
pub mod plugin_license;
pub mod plugin_ovr;
pub mod plugin_playback;
pub mod plugin_protocol_manager;
pub mod plugin_store;
pub mod plugin_tournament;
pub mod plugin_triggers;
pub mod plugin_udp;
pub mod plugin_websocket;

pub mod advanced_analytics;
pub mod event_cache;
pub mod event_stream;
pub mod load_balancer;
pub mod performance_monitor;

#[cfg(feature = "obs-obws")]
pub mod obs_obws;
#[cfg(feature = "youtube")]
pub mod youtube_api;

pub use plugin_cpu_monitor::{CpuMonitorConfig, CpuMonitorPlugin};
pub use plugin_database::DatabasePlugin;
pub use plugin_drive::DrivePlugin;
pub use plugin_license::LicensePlugin;
pub use plugin_ovr as OvrPlugin;
pub use plugin_playback::PlaybackPlugin;
pub use plugin_protocol_manager::ProtocolManager;
pub use plugin_store::StorePlugin;
pub use plugin_tournament::TournamentPlugin;
pub use plugin_triggers::TriggerPlugin;
pub use plugin_udp::UdpPlugin;
pub use plugin_websocket::WebSocketPlugin;

pub use advanced_analytics::{
    AdvancedAnalytics, AnalyticsConfig, AnalyticsSnapshot, AthleteAnalytics, AthletePerformance,
    CachePerformance, DatabasePerformance, EventProcessingPerformance, MatchAnalytics,
    MatchPerformance, MatchPerformancePoint, NetworkPerformance, PerformanceAnalytics,
    PerformancePoint, SystemPerformance, TournamentAnalytics,
};
pub use event_cache::{
    AthleteStatistics, CacheConfig, CacheStatistics, EventCache, MatchStatistics,
    TournamentStatistics,
};
pub use event_stream::{
    EventStreamConfig, EventStreamProcessor, RealTimeAnalytics, StreamStatistics,
};
pub use load_balancer::{
    DistributorStatistics, EventDistributor, LoadBalancer, LoadBalancerConfig,
    LoadDistributionStrategy, ServerHealth, ServerStatistics, UdpServerInstance,
};
pub use performance_monitor::{
    MemoryUsage, MemoryUsageStats, PerformanceMetrics, PerformanceMonitor,
    ProcessingPerformanceStats, ProcessingStats,
};

#[cfg(feature = "youtube")]
pub use youtube_api::{
    YouTubeApiClient, YouTubeApiConfig, YouTubeApiPlugin, YouTubePlaylist, YouTubeStream,
    YouTubeVideo,
};

pub use plugin_drive::drive_plugin;

/// Initialize all plugins
pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Initializing all plugins...");

    plugin_udp::init()?;
    plugin_store::init()?;
    plugin_playback::init()?;
    plugin_tournament::init()?;
    plugin_triggers::init()?;
    plugin_websocket::init()?;
    plugin_database::init()?;
    plugin_drive::init()?;
    plugin_license::init()?;
    plugin_cpu_monitor::init()?;
    plugin_protocol_manager::init()?;

    #[cfg(feature = "obs-obws")]
    obs_obws::init()?;

    log::info!("All plugins initialized successfully");
    Ok(())
}

/// Shutdown all plugins
pub async fn shutdown() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Shutting down all plugins...");

    #[cfg(feature = "obs-obws")]
    obs_obws::shutdown().await?;

    log::info!("All plugins shut down successfully");
    Ok(())
}
