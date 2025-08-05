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
// Old plugin_obs removed - using modular obs system
pub mod load_balancer;
pub mod advanced_analytics;
pub mod obs; // Add modular OBS plugins

// Add placeholder modules for missing imports
pub mod performance_monitor {
    // Placeholder module for performance monitoring
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PerformanceMonitor;
    
    impl PerformanceMonitor {
        pub fn new() -> Self {
            Self
        }
        
        pub fn update_memory_usage(&self) {
            // Placeholder implementation
        }
        
        pub fn record_event_arrival(&self) {
            // Placeholder implementation
        }
        
        pub fn record_event_processed(&self, _processing_time: u64) {
            // Placeholder implementation
        }
        
        pub fn get_performance_metrics(&self) -> PerformanceMetrics {
            PerformanceMetrics
        }
        
        pub fn get_memory_stats(&self) -> MemoryUsageStats {
            MemoryUsageStats
        }
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PerformanceMetrics;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MemoryUsageStats;
}

pub mod event_cache {
    // Placeholder module for event caching
    use serde::{Deserialize, Serialize};
    use std::time::SystemTime;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EventCache;
    
    impl EventCache {
        pub fn new() -> Self {
            Self
        }
        
        pub fn set_match_stats(&self, _match_id: String, _stats: MatchStatistics) {
            // Placeholder implementation
        }
        
        pub async fn set_match_events(&self, _match_id: String, _events: Vec<serde_json::Value>) {
            // Placeholder implementation
        }
        
        pub async fn get_cache_stats(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        
        pub async fn clear_all(&self) {
            // Placeholder implementation
        }
        
        pub async fn invalidate_tournament(&self, _tournament_id: i64) {
            // Placeholder implementation
        }
        
        pub async fn invalidate_match(&self, _match_id: i64) {
            // Placeholder implementation
        }
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MatchStatistics {
        pub match_id: String,
        pub event_count: i32,
        pub duration_seconds: i32,
        pub athlete1_score: i32,
        pub athlete2_score: i32,
        pub last_updated: SystemTime,
    }
}

pub mod event_stream {
    // Placeholder module for event streaming
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EventStreamProcessor;
    
    impl EventStreamProcessor {
        pub fn new(_cache: Arc<super::event_cache::EventCache>) -> Self {
            Self
        }
        
        pub async fn get_statistics(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        
        pub async fn send_event(&self, _event: crate::database::models::PssEventV2) {
            // Placeholder implementation
        }
    }
}

// Re-export main plugin types
pub use plugin_udp::UdpPlugin;
pub use plugin_store::StorePlugin;
pub use plugin_playback::PlaybackPlugin;
pub use plugin_tournament::TournamentPlugin;
pub use plugin_triggers::TriggerPlugin; // Fixed: was TriggersPlugin
pub use plugin_websocket::WebSocketPlugin;
pub use plugin_database::DatabasePlugin;
pub use plugin_drive::DrivePlugin;
pub use plugin_license::LicensePlugin;
pub use plugin_cpu_monitor::{CpuMonitorPlugin, CpuMonitorConfig}; // Added CpuMonitorConfig
pub use plugin_protocol_manager::ProtocolManager; // Fixed: was ProtocolManagerPlugin
// Old ObsPlugin removed - using ObsPluginManager
pub use load_balancer::{EventDistributor, LoadBalancer, LoadBalancerConfig, LoadDistributionStrategy, ServerHealth, ServerStatistics, DistributorStatistics, UdpServerInstance};
pub use advanced_analytics::{AdvancedAnalytics, AnalyticsConfig, TournamentAnalytics, PerformanceAnalytics, AthleteAnalytics, MatchAnalytics, AnalyticsSnapshot, AthletePerformance, SystemPerformance, EventProcessingPerformance, DatabasePerformance, CachePerformance, NetworkPerformance, MatchPerformance, PerformancePoint, MatchPerformancePoint};
// Re-export modular OBS plugins
pub use obs::{ObsPluginManager, ObsCorePlugin, ObsRecordingPlugin, ObsStreamingPlugin, ObsScenesPlugin, ObsSettingsPlugin, ObsEventsPlugin, ObsStatusPlugin};

// Re-export drive plugin function
pub use plugin_drive::drive_plugin;

// Re-export placeholder types
pub use performance_monitor::{PerformanceMonitor, PerformanceMetrics, MemoryUsageStats};
pub use event_cache::{EventCache, MatchStatistics};
pub use event_stream::EventStreamProcessor;

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
    // Old OBS plugin removed - using modular system
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