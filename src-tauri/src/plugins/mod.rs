// Plugin modules
pub mod plugin_udp;
pub mod plugin_websocket;
pub mod plugin_database;
pub mod plugin_drive;
pub mod plugin_obs;
pub mod plugin_playback;
pub mod plugin_store;
pub mod plugin_license;
pub mod plugin_cpu_monitor;
pub mod plugin_protocol_manager;
pub mod plugin_tournament;
pub mod plugin_triggers;
pub mod performance_monitor;
pub mod event_cache;
pub mod event_stream;
pub mod load_balancer;
pub mod advanced_analytics;
pub mod obs; // Add modular OBS plugins

pub use plugin_udp::UdpPlugin;
pub use plugin_websocket::WebSocketPlugin;
pub use plugin_database::DatabasePlugin;
pub use plugin_drive::drive_plugin;
pub use plugin_obs::{ObsPlugin, ObsConnectionConfig, ObsWebSocketVersion, ObsStatusInfo, ObsEvent, ObsConnectionStatus};
pub use plugin_playback::PlaybackPlugin;
pub use plugin_store::StorePlugin;
pub use plugin_license::LicensePlugin;
pub use plugin_cpu_monitor::{CpuMonitorPlugin, CpuMonitorConfig, CpuProcessData, SystemCpuData};
pub use plugin_protocol_manager::{ProtocolManager, ProtocolFile, ProtocolVersion, StreamDefinition};
pub use plugin_tournament::{TournamentPlugin, LocationVerification, CreateTournamentRequest, UpdateTournamentRequest};
pub use plugin_triggers::{TriggerPlugin, PssEventType, TriggerType, TriggerExecutionResult};
pub use performance_monitor::{PerformanceMonitor, MemoryTracker, ProcessingStats, EventRateTracker, PerformanceMetrics, MemoryUsageStats, ProcessingPerformanceStats};
pub use event_cache::{EventCache, CacheConfig, CacheStatistics, AthleteStatistics, TournamentStatistics, MatchStatistics};
pub use event_stream::{EventStreamProcessor, EventStreamConfig, StreamStatistics, RealTimeAnalytics, EventStreamSubscriber};
pub use load_balancer::{EventDistributor, LoadBalancer, LoadBalancerConfig, LoadDistributionStrategy, ServerHealth, ServerStatistics, DistributorStatistics, UdpServerInstance};
pub use advanced_analytics::{AdvancedAnalytics, AnalyticsConfig, TournamentAnalytics, PerformanceAnalytics, AthleteAnalytics, MatchAnalytics, AnalyticsSnapshot, AthletePerformance, SystemPerformance, EventProcessingPerformance, DatabasePerformance, CachePerformance, NetworkPerformance, MatchPerformance, PerformancePoint, MatchPerformancePoint};
// Re-export modular OBS plugins
pub use obs::{ObsPluginManager, ObsCorePlugin, ObsRecordingPlugin, ObsStreamingPlugin, ObsScenesPlugin, ObsSettingsPlugin, ObsEventsPlugin, ObsStatusPlugin};

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
    plugin_websocket::init()?;  // WebSocket server for HTML overlays
    plugin_tournament::init()?; // Tournament management system
    plugin_triggers::init()?;   // Trigger system for PSS events
    obs::init()?; // Initialize modular OBS plugin system
    
    println!("✅ All plugins initialized successfully");
    Ok(())
} 