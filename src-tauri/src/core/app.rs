//! Main application class and lifecycle management

use crate::types::{AppResult, AppState, AppView};
use crate::plugins::{ObsPlugin, PlaybackPlugin, UdpPlugin, StorePlugin, LicensePlugin, CpuMonitorPlugin, ProtocolManager, DatabasePlugin};
use crate::logging::LogManager;
use crate::config::ConfigManager;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::{RwLock, Mutex, broadcast};

// Global PSS event broadcaster for real-time event emission
static PSS_EVENT_BROADCASTER: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

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
    protocol_manager: ProtocolManager,
    database_plugin: DatabasePlugin,
    log_manager: Arc<Mutex<LogManager>>,
}

impl App {
    /// Create a new application instance
    pub async fn new() -> AppResult<Self> {
        log::info!("🚀 Creating new application instance...");
        
        // Initialize global PSS event broadcaster
        PSS_EVENT_BROADCASTER.get_or_init(|| broadcast::channel(100).0);
        
        let state = Arc::new(RwLock::new(AppState::default()));
        
        // Initialize configuration manager
        let config_dir = PathBuf::from("config");
        let config_manager = ConfigManager::new(&config_dir)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize config manager: {}", e)))?;
        log::info!("✅ Configuration manager initialized");
        
        // Create event channels for plugins
        let (obs_event_tx, _obs_event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (playback_event_tx, _playback_event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (udp_event_tx, udp_event_rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Initialize logging manager with external log directory to prevent rebuild loops
        let mut log_config = crate::logging::LogConfig::default();
        // Use a directory outside the project to prevent Tauri file watching from triggering rebuilds
        log_config.log_dir = "logs".to_string();
        log_config.archive_dir = "logs/archives".to_string();
        let log_manager = Arc::new(Mutex::new(LogManager::new(log_config)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize logging: {}", e)))?));
        
        // Initialize plugins
        let obs_plugin = ObsPlugin::new(obs_event_tx, log_manager.clone());
        log::info!("✅ OBS plugin initialized");
        
        let playback_plugin = PlaybackPlugin::new(crate::plugins::plugin_playback::PlaybackConfig::default(), playback_event_tx);
        log::info!("✅ Playback plugin initialized");
        
        let store_plugin = StorePlugin::new();
        log::info!("✅ Store plugin initialized");
        
        let license_plugin = LicensePlugin::new();
        log::info!("✅ License plugin initialized");
        
        let cpu_monitor_plugin = CpuMonitorPlugin::new(crate::plugins::CpuMonitorConfig::default());
        log::info!("✅ CPU monitor plugin initialized");
        
        let protocol_manager = ProtocolManager::new()?;
        if let Err(e) = protocol_manager.init().await {
            log::warn!("⚠️ Warning: Failed to initialize protocol manager: {}", e);
        }
        log::info!("✅ Protocol manager plugin initialized");
        
        let protocol_manager_arc = Arc::new(protocol_manager.clone());
        let udp_plugin = UdpPlugin::new(crate::plugins::plugin_udp::UdpServerConfig::default(), udp_event_tx, protocol_manager_arc);
        log::info!("✅ UDP plugin initialized");
        
        // Initialize database plugin
        let database_plugin = DatabasePlugin::new()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize database plugin: {}", e)))?;
        log::info!("✅ Database plugin initialized");
        
        // Start UDP event handler
        let log_manager_clone = log_manager.clone();
        tokio::spawn(async move {
            Self::handle_udp_events(udp_event_rx, log_manager_clone).await;
        });
        
        // Load OBS connections from config manager
        let config_connections = config_manager.get_obs_connections().await;
        if let Err(e) = obs_plugin.load_connections_from_config(config_connections).await {
            log::warn!("⚠️ Warning: Failed to load OBS connections from config: {}", e);
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
            protocol_manager,
            database_plugin,
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
        
        // Check if UDP should auto-start
        let config = self.config_manager.get_config().await;
        if config.app.startup.auto_start_udp {
            println!("🎯 Auto-starting UDP server...");
            if let Err(e) = self.udp_plugin().start(&config).await {
                println!("⚠️ Failed to auto-start UDP server: {}", e);
            } else {
                println!("✅ UDP server auto-started successfully");
            }
        }
        
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
    
    /// Get protocol manager reference
    pub fn protocol_manager(&self) -> &ProtocolManager {
        &self.protocol_manager
    }
    
    /// Get database plugin reference
    pub fn database_plugin(&self) -> &DatabasePlugin {
        &self.database_plugin
    }
    
    /// Get log manager reference
    pub fn log_manager(&self) -> &Arc<Mutex<LogManager>> {
        &self.log_manager
    }
    
    /// Get configuration manager reference
    pub fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }

    /// Emit a PSS event to all listeners
    pub fn emit_pss_event(event_json: serde_json::Value) {
        if let Some(broadcaster) = PSS_EVENT_BROADCASTER.get() {
            if let Err(e) = broadcaster.send(event_json) {
                log::warn!("⚠️ Failed to broadcast PSS event: {}", e);
            }
        }
    }

    /// Get a receiver for PSS events
    pub fn subscribe_to_pss_events() -> Option<broadcast::Receiver<serde_json::Value>> {
        PSS_EVENT_BROADCASTER.get().map(|broadcaster| broadcaster.subscribe())
    }
    
    /// Handle UDP events
    async fn handle_udp_events(
        mut event_rx: tokio::sync::mpsc::UnboundedReceiver<crate::plugins::plugin_udp::PssEvent>,
        log_manager: Arc<Mutex<LogManager>>,
    ) {
        log::info!("🎯 UDP event handler started");
        
        while let Some(event) = event_rx.recv().await {
            // Convert PSS event to JSON for frontend emission
            let event_json = match &event {
                crate::plugins::plugin_udp::PssEvent::Points { athlete, point_type } => {
                    serde_json::json!({
                        "type": "points",
                        "athlete": athlete,
                        "point_type": point_type,
                        "description": format!("Athlete {} scored {} points", athlete, point_type)
                    })
                }
                crate::plugins::plugin_udp::PssEvent::HitLevel { athlete, level } => {
                    serde_json::json!({
                        "type": "hit_level",
                        "athlete": athlete,
                        "level": level,
                        "description": format!("Athlete {} hit level {}", athlete, level)
                    })
                }
                crate::plugins::plugin_udp::PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                    serde_json::json!({
                        "type": "warnings",
                        "athlete1_warnings": athlete1_warnings,
                        "athlete2_warnings": athlete2_warnings,
                        "description": format!("Warnings - Athlete1: {}, Athlete2: {}", athlete1_warnings, athlete2_warnings)
                    })
                }
                crate::plugins::plugin_udp::PssEvent::Clock { time, action } => {
                    serde_json::json!({
                        "type": "clock",
                        "time": time,
                        "action": action,
                        "description": format!("Clock: {} {:?}", time, action.clone().unwrap_or_default())
                    })
                }
                crate::plugins::plugin_udp::PssEvent::Round { current_round } => {
                    serde_json::json!({
                        "type": "round",
                        "current_round": current_round,
                        "description": format!("Round {}", current_round)
                    })
                }
                crate::plugins::plugin_udp::PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                    serde_json::json!({
                        "type": "winner_rounds",
                        "round1_winner": round1_winner,
                        "round2_winner": round2_winner,
                        "round3_winner": round3_winner,
                        "description": format!("Winner Rounds - R1: {}, R2: {}, R3: {}", round1_winner, round2_winner, round3_winner)
                    })
                }
                crate::plugins::plugin_udp::PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => {
                    serde_json::json!({
                        "type": "scores",
                        "athlete1_r1": athlete1_r1,
                        "athlete2_r1": athlete2_r1,
                        "athlete1_r2": athlete1_r2,
                        "athlete2_r2": athlete2_r2,
                        "athlete1_r3": athlete1_r3,
                        "athlete2_r3": athlete2_r3,
                        "description": format!("Scores - A1: R1={}, R2={}, R3={} | A2: R1={}, R2={}, R3={}", 
                            athlete1_r1, athlete1_r2, athlete1_r3, athlete2_r1, athlete2_r2, athlete2_r3)
                    })
                }
                crate::plugins::plugin_udp::PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                    serde_json::json!({
                        "type": "current_scores",
                        "athlete1_score": athlete1_score,
                        "athlete2_score": athlete2_score,
                        "description": format!("Current Scores - A1: {}, A2: {}", athlete1_score, athlete2_score)
                    })
                }
                crate::plugins::plugin_udp::PssEvent::Athletes { athlete1_short, athlete1_long, athlete1_country, athlete2_short, athlete2_long, athlete2_country } => {
                    serde_json::json!({
                        "type": "athletes",
                        "athlete1_short": athlete1_short,
                        "athlete1_long": athlete1_long,
                        "athlete1_country": athlete1_country,
                        "athlete2_short": athlete2_short,
                        "athlete2_long": athlete2_long,
                        "athlete2_country": athlete2_country,
                        "description": format!("Athletes - {} ({}) vs {} ({})", athlete1_short, athlete1_country, athlete2_short, athlete2_country)
                    })
                }
                crate::plugins::plugin_udp::PssEvent::MatchConfig { number, category, weight, rounds, colors, match_id, division, total_rounds, round_duration, countdown_type, count_up, format } => {
                    serde_json::json!({
                        "type": "match_config",
                        "number": number,
                        "category": category,
                        "weight": weight,
                        "rounds": rounds,
                        "colors": colors,
                        "match_id": match_id,
                        "division": division,
                        "total_rounds": total_rounds,
                        "round_duration": round_duration,
                        "countdown_type": countdown_type,
                        "count_up": count_up,
                        "format": format,
                        "description": format!("Match Config - #{} {} {} ({})", number, category, weight, division)
                    })
                }
                crate::plugins::plugin_udp::PssEvent::FightLoaded => {
                    serde_json::json!({
                        "type": "fight_loaded",
                        "event": "FightLoaded",
                        "description": "Fight Loaded"
                    })
                }
                crate::plugins::plugin_udp::PssEvent::FightReady => {
                    serde_json::json!({
                        "type": "fight_ready",
                        "event": "FightReady",
                        "description": "Fight Ready"
                    })
                }
                crate::plugins::plugin_udp::PssEvent::Raw(message) => {
                    serde_json::json!({
                        "type": "raw",
                        "message": message,
                        "description": format!("Raw message: {}", message)
                    })
                }
                _ => {
                    serde_json::json!({
                        "type": "other",
                        "event": format!("{:?}", event),
                        "description": format!("Event: {:?}", event)
                    })
                }
            };

            // Emit event to frontend listeners
            Self::emit_pss_event(event_json.clone());

            // Process different event types for logging
            match event {
                crate::plugins::plugin_udp::PssEvent::Raw(message) => {
                    // Log raw UDP datagram content to both subsystems
                    let raw_str = message.clone();
                    // Store full datagram in udp log for easier debugging
                    if let Err(e) = log_manager.lock().await.log("udp", "INFO", &raw_str) {
                        log::error!("Failed to log raw UDP message: {}", e);
                    }

                    // Also keep the previously‐existing PSS raw log for protocol analysis
                    let pss_str = format!("📡 Raw PSS message: {}", message);
                    if let Err(e) = log_manager.lock().await.log("pss", "INFO", &pss_str) {
                        log::error!("Failed to log PSS raw message: {}", e);
                    }
                    log::debug!("🎯 Raw UDP datagram: {}", message);
                }
                _ => {
                    // Log parsed events to UDP subsystem (they are UDP server events)
                    let event_str = match event {
                        crate::plugins::plugin_udp::PssEvent::Points { athlete, point_type } => {
                            format!("🥊 UDP-EVENT: Athlete {} scored {} points", athlete, point_type)
                        }
                        crate::plugins::plugin_udp::PssEvent::HitLevel { athlete, level } => {
                            format!("💥 UDP-EVENT: Athlete {} hit level {}", athlete, level)
                        }
                        crate::plugins::plugin_udp::PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                            format!("⚠️ UDP-EVENT: Warnings - Athlete1: {}, Athlete2: {}", athlete1_warnings, athlete2_warnings)
                        }
                        crate::plugins::plugin_udp::PssEvent::Clock { time, action } => {
                            format!("⏰ UDP-EVENT: Clock {} {:?}", time, action.unwrap_or_default())
                        }
                        crate::plugins::plugin_udp::PssEvent::Break { time, action } => {
                            format!("⏸️ UDP-EVENT: Break {} {:?}", time, action.unwrap_or_default())
                        }
                        crate::plugins::plugin_udp::PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                            format!("🏆 UDP-EVENT: WinnerRounds - R1:{}, R2:{}, R3:{}", round1_winner, round2_winner, round3_winner)
                        }
                        crate::plugins::plugin_udp::PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => {
                            format!("📊 UDP-EVENT: Scores - A1(R1:{},R2:{},R3:{}), A2(R1:{},R2:{},R3:{})", 
                                athlete1_r1, athlete1_r2, athlete1_r3, athlete2_r1, athlete2_r2, athlete2_r3)
                        }
                        crate::plugins::plugin_udp::PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                            format!("🎯 UDP-EVENT: Current Scores - A1:{}, A2:{}", athlete1_score, athlete2_score)
                        }
                        crate::plugins::plugin_udp::PssEvent::Round { current_round } => {
                            format!("🔄 UDP-EVENT: Round {}", current_round)
                        }
                        crate::plugins::plugin_udp::PssEvent::FightLoaded => {
                            "🎬 UDP-EVENT: Fight Loaded".to_string()
                        }
                        crate::plugins::plugin_udp::PssEvent::FightReady => {
                            "✅ UDP-EVENT: Fight Ready".to_string()
                        }
                        _ => {
                            format!("📋 UDP-EVENT: {:?}", event)
                        }
                    };
                    
                    if let Err(e) = log_manager.lock().await.log("udp", "INFO", &event_str) {
                        log::error!("Failed to log UDP event: {}", e);
                    }

                    // Also log parsed representation to PSS subsystem for easier protocol debugging
                    let pss_event_str = format!("PSS Event: {}", event_str);
                    if let Err(e) = log_manager.lock().await.log("pss", "INFO", &pss_event_str) {
                        log::error!("Failed to log PSS parsed event: {}", e);
                    }
                    
                    log::info!("🎯 UDP event: {}", event_str);
                }
            }
        }
        
        log::info!("🎯 UDP event handler stopped");
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