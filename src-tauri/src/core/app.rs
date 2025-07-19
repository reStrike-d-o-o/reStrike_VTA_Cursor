//! Main application class and lifecycle management

use crate::types::{AppResult, AppState, AppView};
use crate::plugins::{ObsPlugin, PlaybackPlugin, UdpPlugin, StorePlugin, LicensePlugin, CpuMonitorPlugin};
use crate::logging::LogManager;
use crate::config::ConfigManager;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// Main application class that orchestrates all systems
pub struct App {
    state: Arc<RwLock<AppState>>,
    config_manager: ConfigManager,
    obs_plugin: ObsPlugin,
    playback_plugin: PlaybackPlugin,
    udp_plugin: UdpPlugin,
    store_plugin: StorePlugin,
    license_plugin: LicensePlugin,
    cpu_monitor_plugin: CpuMonitorPlugin,
    log_manager: Arc<LogManager>,
}

impl App {
    /// Create a new application instance
    pub async fn new() -> AppResult<Self> {
        println!("🚀 Creating new application instance...");
        
        let state = Arc::new(RwLock::new(AppState::default()));
        
        // Initialize configuration manager
        let config_dir = PathBuf::from("config");
        let config_manager = ConfigManager::new(&config_dir)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize config manager: {}", e)))?;
        
        // Create event channels for plugins
        let (obs_event_tx, _obs_event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (playback_event_tx, _playback_event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (udp_event_tx, _udp_event_rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Initialize logging manager with external log directory to prevent rebuild loops
        let mut log_config = crate::logging::LogConfig::default();
        // Use a directory outside the project to prevent Tauri file watching from triggering rebuilds
        log_config.log_dir = "logs".to_string();
        log_config.archive_dir = "logs/archives".to_string();
        let log_manager = Arc::new(LogManager::new(log_config)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize logging: {}", e)))?);
        
        // Initialize plugins
        let obs_plugin = ObsPlugin::new(obs_event_tx);
        let playback_plugin = PlaybackPlugin::new(crate::plugins::plugin_playback::PlaybackConfig::default(), playback_event_tx);
        let udp_plugin = UdpPlugin::new(crate::plugins::plugin_udp::UdpServerConfig::default(), udp_event_tx);
        let store_plugin = StorePlugin::new();
        let license_plugin = LicensePlugin::new();
        let cpu_monitor_plugin = CpuMonitorPlugin::new(crate::plugins::CpuMonitorConfig::default());
        
        // Load OBS connections from config manager
        let config_connections = config_manager.get_obs_connections().await;
        if let Err(e) = obs_plugin.load_connections_from_config(config_connections).await {
            println!("⚠️ Warning: Failed to load OBS connections from config: {}", e);
        }
        
        Ok(Self {
            state,
            config_manager,
            obs_plugin,
            playback_plugin,
            udp_plugin,
            store_plugin,
            license_plugin,
            cpu_monitor_plugin,
            log_manager,
        })
    }
    
    /// Initialize the application
    pub async fn init(&self) -> AppResult<()> {
        println!("🔧 Initializing application...");
        
        // Initialize all subsystems
        // Note: Plugins are already initialized in the plugins::init() function
        // called from lib.rs init()
        
        println!("✅ Application initialized successfully");
        Ok(())
    }
    
    /// Start the application
    pub async fn start(&self) -> AppResult<()> {
        println!("▶️ Starting application...");
        
        // Start all subsystems
        // Note: Plugins are started on-demand when needed
        
        println!("✅ Application started successfully");
        Ok(())
    }
    
    /// Stop the application
    pub async fn stop(&self) -> AppResult<()> {
        println!("⏹️ Stopping application...");
        
        // Stop all subsystems
        self.udp_plugin.stop()?;
        
        println!("✅ Application stopped successfully");
        Ok(())
    }
    
    /// Get application state
    pub async fn get_state(&self) -> AppState {
        self.state.read().await.clone()
    }
    
    /// Update current view
    pub async fn set_view(&self, view: AppView) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.current_view = view;
        Ok(())
    }
    
    /// Get OBS plugin reference
    pub fn obs_plugin(&self) -> &ObsPlugin {
        &self.obs_plugin
    }
    
    /// Get playback plugin reference
    pub fn playback_plugin(&self) -> &PlaybackPlugin {
        &self.playback_plugin
    }
    
    /// Get UDP plugin reference
    pub fn udp_plugin(&self) -> &UdpPlugin {
        &self.udp_plugin
    }
    
    /// Get store plugin reference
    pub fn store_plugin(&self) -> &StorePlugin {
        &self.store_plugin
    }
    
    /// Get license plugin reference
    pub fn license_plugin(&self) -> &LicensePlugin {
        &self.license_plugin
    }
    
    /// Get CPU monitor plugin reference
    pub fn cpu_monitor_plugin(&self) -> &CpuMonitorPlugin {
        &self.cpu_monitor_plugin
    }
    
    /// Get log manager reference
    pub fn log_manager(&self) -> &LogManager {
        &self.log_manager
    }
    
    /// Get configuration manager reference
    pub fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            obs_connections: Vec::new(),
            active_obs_connection: None,
            obs_status: None,
            overlay_settings: crate::types::OverlaySettings::default(),
            video_clips: Vec::new(),
            current_clip: None,
            is_playing: false,
            current_view: AppView::SidebarTest,
            is_loading: false,
            error: None,
        }
    }
}

impl Default for crate::types::OverlaySettings {
    fn default() -> Self {
        Self {
            opacity: 0.9,
            position: crate::types::OverlayPosition::BottomRight,
            scale: 1.0,
            visible: true,
            theme: crate::types::OverlayTheme::Dark,
        }
    }
} 