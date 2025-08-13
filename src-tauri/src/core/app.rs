//! Main application class and lifecycle management

use crate::types::{AppResult, AppState, AppView};
use crate::plugins::{PlaybackPlugin, UdpPlugin, StorePlugin, LicensePlugin, CpuMonitorPlugin, ProtocolManager, DatabasePlugin, WebSocketPlugin, TournamentPlugin, EventCache, EventStreamProcessor, EventDistributor, AdvancedAnalytics, YouTubeApiPlugin};
use crate::plugins::obs::ObsPluginManager; // Use new modular OBS plugin system
#[cfg(feature = "obs-obws")]
use crate::plugins::obs_obws::manager::ObsManager as ObsObwsManager; // Use new obws-based OBS manager
#[cfg(feature = "obs-obws")]
use crate::plugins::obs_obws::ObsRecordingEventHandler; // Use new recording event handler
use crate::logging::LogManager;
use crate::config::ConfigManager;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::{RwLock, Mutex, broadcast};
use tauri::{Emitter, Manager};

// Global PSS event broadcaster for real-time event emission to WebSocket overlays
static PSS_EVENT_BROADCASTER: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

// Global Tauri app handle for real-time event emission to frontend
static TAURI_APP_HANDLE: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

/// Main application class that orchestrates all systems
pub struct App {
    state: Arc<RwLock<AppState>>,
    config_manager: ConfigManager,
    obs_plugin_manager: Arc<ObsPluginManager>, // Use new modular OBS plugin manager
    #[cfg(feature = "obs-obws")]
    obs_obws_manager: Arc<ObsObwsManager>, // Use new obws-based OBS manager
    #[cfg(feature = "obs-obws")]
    recording_event_handler: Arc<ObsRecordingEventHandler>, // Recording event handler for PSS integration
    youtube_api_plugin: Arc<Mutex<YouTubeApiPlugin>>, // YouTube API integration
    playback_plugin: PlaybackPlugin,
    udp_plugin: UdpPlugin,
    store_plugin: StorePlugin,
    license_plugin: LicensePlugin,
    cpu_monitor_plugin: CpuMonitorPlugin,
    protocol_manager: ProtocolManager,
    database_plugin: DatabasePlugin,
    websocket_plugin: Arc<Mutex<WebSocketPlugin>>,
    tournament_plugin: TournamentPlugin,
    log_manager: Arc<Mutex<LogManager>>,
    app_handle: Option<tauri::AppHandle>, // Store app handle for real-time emission
    udp_event_rx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<crate::plugins::plugin_udp::PssEvent>>>>, // Store UDP event receiver
    
    // Phase 3: Advanced Scaling Components
    event_cache: Arc<EventCache>,
    event_stream_processor: Arc<EventStreamProcessor>,
    event_distributor: Arc<EventDistributor>,
    advanced_analytics: Arc<AdvancedAnalytics>,
}

impl App {
    /// Create a new application instance
    pub async fn new() -> AppResult<Self> {
        log::info!("🚀 Creating new application instance...");
        
        // Initialize global PSS event broadcaster for WebSocket overlays
        PSS_EVENT_BROADCASTER.get_or_init(|| broadcast::channel(1000).0); // Large buffer for real-time performance
        
        let state = Arc::new(RwLock::new(AppState::default()));
        
        // Initialize configuration manager
        let config_dir = PathBuf::from("config");
        let config_manager = ConfigManager::new(&config_dir)
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize config manager: {}", e)))?;
        log::info!("✅ Configuration manager initialized");
        
        // Create event channels for plugins
        let (_obs_event_tx, _obs_event_rx) = tokio::sync::mpsc::unbounded_channel::<crate::plugins::obs::types::ObsEvent>();
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
        let obs_plugin_manager = Arc::new(ObsPluginManager::new()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to create OBS plugin manager: {}", e)))?);
        log::info!("✅ OBS plugin manager initialized");
        
        #[cfg(feature = "obs-obws")]
        let obs_obws_manager = Arc::new(ObsObwsManager::new());
        #[cfg(feature = "obs-obws")]
        log::info!("✅ OBS obws manager initialized");
        
        // Create recording event channel
        #[cfg(feature = "obs-obws")]
        let (recording_event_tx, _recording_event_rx) = tokio::sync::mpsc::unbounded_channel::<crate::plugins::obs_obws::RecordingEvent>();
        
        let youtube_api_plugin = Arc::new(Mutex::new(YouTubeApiPlugin::new()));
        log::info!("✅ YouTube API plugin initialized");
        
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
        
        // Initialize database plugin first (needed for UDP plugin)
        let database_plugin = DatabasePlugin::new().await
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize database plugin: {}", e)))?;
        log::info!("✅ Database plugin initialized");
        
        // Initialize recording event handler (after database plugin)
        #[cfg(feature = "obs-obws")]
        let recording_event_handler = Arc::new(ObsRecordingEventHandler::new(
            crate::plugins::obs_obws::AutomaticRecordingConfig::default(),
            recording_event_tx,
            Arc::new(database_plugin.clone()),
            obs_obws_manager.clone(),
        ));
        #[cfg(feature = "obs-obws")]
        log::info!("✅ Recording event handler initialized");
        
        let protocol_manager_arc = Arc::new(protocol_manager.clone());
        let database_plugin_arc = Arc::new(database_plugin.clone());
        let udp_plugin = UdpPlugin::new(
            udp_event_tx, 
            protocol_manager_arc,
            database_plugin_arc,
        );
        log::info!("✅ UDP plugin initialized");
        
        // Initialize Phase 3: Advanced Scaling Components
        let event_cache = Arc::new(EventCache::new());
        log::info!("✅ Event cache initialized");
        
        let event_stream_processor = Arc::new(EventStreamProcessor::new(event_cache.clone()));
        log::info!("✅ Event stream processor initialized");
        
        let event_distributor = Arc::new(EventDistributor::new(event_cache.clone()));
        log::info!("✅ Event distributor initialized");
        
        let advanced_analytics = Arc::new(AdvancedAnalytics::new(event_cache.clone()));
        log::info!("✅ Advanced analytics initialized");
        
        // Initialize WebSocket plugin for HTML overlays
        let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel::<crate::plugins::plugin_udp::PssEvent>();
        let websocket_plugin = Arc::new(Mutex::new(WebSocketPlugin::new(event_tx))); // Port 3001 for WebSocket server
        log::info!("✅ WebSocket plugin initialized");
        
        // Initialize tournament plugin
        let tournament_plugin = TournamentPlugin::new(database_plugin.get_database_connection());
        log::info!("✅ Tournament plugin initialized");
        
        // Note: UDP event handler will be started in start() method when UDP server starts
        
        // Load OBS connections from config manager
        let config_connections = config_manager.get_obs_connections().await;
        
        // Load connections into the modular OBS plugin system
        if let Err(e) = obs_plugin_manager.load_connections_from_config(config_connections.clone()).await {
            log::warn!("⚠️ Failed to load OBS connections into plugin system: {}", e);
        } else {
            log::info!("✅ OBS connections loaded into modular plugin system");
        }
        
        log::info!("✅ OBS connections configuration loaded ({} connections)", config_connections.len());
        
        Ok(Self {
            state,
            config_manager,
            obs_plugin_manager,
            #[cfg(feature = "obs-obws")]
            obs_obws_manager,
            #[cfg(feature = "obs-obws")]
            recording_event_handler,
            youtube_api_plugin,
            playback_plugin,
            udp_plugin,
            store_plugin,
            license_plugin,
            cpu_monitor_plugin,
            protocol_manager,
            database_plugin,
            websocket_plugin,
            tournament_plugin,
            log_manager,
            app_handle: None, // Will be set when app handle is available
            udp_event_rx: Arc::new(Mutex::new(Some(udp_event_rx))), // Store UDP event receiver for later use
            event_cache,
            event_stream_processor,
            event_distributor,
            advanced_analytics,
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
        
        // Start WebSocket server for HTML overlays
        println!("🔗 Starting WebSocket server for HTML overlays...");
        let websocket_plugin = self.websocket_plugin().lock().await;
        if let Err(e) = websocket_plugin.start(3001).await {
            println!("⚠️ Failed to start WebSocket server: {}", e);
        } else {
            println!("✅ WebSocket server started successfully");
            
            // Connect WebSocket plugin to PSS event broadcaster for real-time overlays
            if let Some(pss_receiver) = Self::subscribe_to_pss_events() {
                let websocket_plugin_clone = self.websocket_plugin().clone();
                tokio::spawn(async move {
                    Self::handle_pss_to_websocket(pss_receiver, websocket_plugin_clone).await;
                });
                println!("✅ WebSocket plugin connected to PSS event broadcaster");
            }
        }
        
        // Check if UDP should auto-start
        let config = self.config_manager.get_config().await;
        if config.app.startup.auto_start_udp {
            println!("🎯 Auto-starting UDP server...");
            if let Err(e) = self.udp_plugin().start(&config).await {
                println!("⚠️ Failed to auto-start UDP server: {}", e);
            } else {
                println!("✅ UDP server auto-started successfully");
                
                // Start UDP event handler when UDP server starts
                if let Some(udp_event_rx) = self.udp_event_rx.lock().await.take() {
                    let log_manager_clone = self.log_manager().clone();
                    tokio::spawn(async move {
                        Self::handle_udp_events(udp_event_rx, log_manager_clone).await;
                    });
                    println!("✅ UDP event handler started");
                }
            }
        }
        
        println!("✅ Application started successfully");
        Ok(())
    }
    
    /// Stop the application
    pub async fn stop(&self) -> AppResult<()> {
        println!("⏹️ Stopping application...");
        
        // Stop all subsystems
        self.udp_plugin.stop().await?;
        
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
    pub fn obs_plugin(&self) -> &Arc<ObsPluginManager> {
        &self.obs_plugin_manager
    }
    
    #[cfg(feature = "obs-obws")]
    pub fn obs_obws_plugin(&self) -> &Arc<ObsObwsManager> {
        &self.obs_obws_manager
    }
    
    #[cfg(feature = "obs-obws")]
    pub fn recording_event_handler(&self) -> &Arc<ObsRecordingEventHandler> {
        &self.recording_event_handler
    }
    
    pub fn youtube_api_plugin(&self) -> &Arc<Mutex<YouTubeApiPlugin>> {
        &self.youtube_api_plugin
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
    
    /// Get WebSocket plugin reference
    pub fn websocket_plugin(&self) -> &Arc<Mutex<WebSocketPlugin>> {
        &self.websocket_plugin
    }
    
    /// Get tournament plugin reference
    pub fn tournament_plugin(&self) -> &TournamentPlugin {
        &self.tournament_plugin
    }
    
    /// Get log manager reference
    pub fn log_manager(&self) -> &Arc<Mutex<LogManager>> {
        &self.log_manager
    }
    
    /// Get configuration manager reference
    pub fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }
    
    // Phase 3: Advanced Scaling Component Getters
    pub fn event_cache(&self) -> &Arc<EventCache> {
        &self.event_cache
    }
    
    pub fn event_stream_processor(&self) -> &Arc<EventStreamProcessor> {
        &self.event_stream_processor
    }
    
    pub fn event_distributor(&self) -> &Arc<EventDistributor> {
        &self.event_distributor
    }
    
    pub fn advanced_analytics(&self) -> &Arc<AdvancedAnalytics> {
        &self.advanced_analytics
    }

    /// Trigger instant round replay: save replay buffer, resolve last file within configured wait, launch mpv
    pub async fn replay_round_now(&self, connection_name: Option<&str>) -> AppResult<()> {
        // Simple debounce to avoid repeated triggers
        static LAST_REPLAY_MS: std::sync::OnceLock<std::sync::Mutex<i64>> = std::sync::OnceLock::new();
        let now_ms = chrono::Utc::now().timestamp_millis();
        let m = LAST_REPLAY_MS.get_or_init(|| std::sync::Mutex::new(0));
        {
            let mut last = m.lock().unwrap();
            if now_ms - *last < 2000 { // 2s
                return Ok(());
            }
            *last = now_ms;
        }
        // Read IVR settings
        use crate::database::operations::UiSettingsOperations as UIOps;
        let conn = self.database_plugin().get_connection().await?;
        let mpv_path = UIOps::get_ui_setting(&*conn, "ivr.replay.mpv_path").ok().flatten()
            .unwrap_or_else(|| "C:/Program Files/mpv/mpv.exe".to_string());
        let seconds_from_end: u32 = UIOps::get_ui_setting(&*conn, "ivr.replay.seconds_from_end").ok().flatten()
            .and_then(|s| s.parse::<u32>().ok()).unwrap_or(10).min(20);
        let max_wait_ms: u32 = UIOps::get_ui_setting(&*conn, "ivr.replay.max_wait_ms").ok().flatten()
            .and_then(|s| s.parse::<u32>().ok()).unwrap_or(500).clamp(50, 500);

        // Save replay buffer via obws
        self.obs_obws_plugin().save_replay_buffer(connection_name).await?;

        // Try to get last replay filename within bounded wait
        let mut filename: Option<String> = None;
        let mut elapsed: u32 = 0;
        let step: u32 = 150;
        while elapsed <= max_wait_ms {
            match self.obs_obws_plugin().get_last_replay_filename(connection_name).await {
                Ok(name) if !name.is_empty() => { filename = Some(name); break; }
                _ => {}
            }
            if elapsed == max_wait_ms { break; }
            let sleep_ms = std::cmp::min(step, max_wait_ms - elapsed);
            tokio::time::sleep(std::time::Duration::from_millis(sleep_ms as u64)).await;
            elapsed += sleep_ms;
        }

        // Build full path using OBS recording directory if available, else Videos root
        let directory = match self.obs_obws_plugin().get_record_directory(connection_name).await {
            Ok(dir) if !dir.is_empty() => std::path::PathBuf::from(dir),
            _ => crate::plugins::obs_obws::PathGeneratorConfig::detect_windows_videos_folder(),
        };
        let file_path = if let Some(name) = filename { directory.join(name) } else { directory };

        // Launch mpv
        if !file_path.is_file() {
            return Err(crate::types::AppError::ConfigError("Replay file not found within time window".to_string()));
        }
        std::process::Command::new(mpv_path)
            .arg(format!("--start=-{}", seconds_from_end))
            .arg(file_path.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to launch mpv: {}", e)))?;

        Ok(())
    }

    /// Get the default connection name for OBS operations
    pub async fn get_default_connection_name(&self) -> AppResult<String> {
        // Try to get the first available connection name
        let connection_names = self.obs_plugin().get_connection_names().await;
        
        if connection_names.is_empty() {
            // If no connections exist, return a default name
            Ok("OBS_REC".to_string())
        } else {
            // Return the first available connection
            Ok(connection_names[0].clone())
        }
    }
    
    /// Set the Tauri app handle for real-time frontend emission
    pub fn set_app_handle(&mut self, app_handle: tauri::AppHandle) {
        self.app_handle = Some(app_handle);
        log::info!("✅ Tauri app handle set for real-time frontend emission");
    }
    
    /// Emit PSS event to Tauri frontend if app handle is available
    pub fn emit_to_frontend(&self, event_json: serde_json::Value) {
        if let Some(app_handle) = &self.app_handle {
            if let Err(e) = app_handle.emit("pss_event", event_json) {
                log::warn!("⚠️ Failed to emit PSS event to Tauri frontend: {}", e);
            }
        }
    }
    
    /// Start UDP event handler manually (for when UDP is started manually)
    pub async fn start_udp_event_handler(&self) {
        if let Some(udp_event_rx) = self.udp_event_rx.lock().await.take() {
            let log_manager_clone = self.log_manager().clone();
            tokio::spawn(async move {
                Self::handle_udp_events(udp_event_rx, log_manager_clone).await;
            });
            println!("✅ UDP event handler started manually");
        }
    }

    /// Set the global Tauri app handle for frontend event emission
    pub fn set_global_app_handle(app_handle: tauri::AppHandle) {
        if let Err(_) = TAURI_APP_HANDLE.set(app_handle) {
            log::warn!("⚠️ Global app handle already set");
        } else {
            log::info!("✅ Global Tauri app handle set for frontend event emission");
        }
    }

    /// Emit a PSS event to both WebSocket overlays and Tauri frontend
    pub fn emit_pss_event(event_json: serde_json::Value) {
        // Emit to frontend via Tauri events
        if let Some(app_handle) = TAURI_APP_HANDLE.get() {
            if let Err(e) = app_handle.emit("pss_event", event_json.clone()) {
                log::warn!("⚠️ Failed to emit PSS event to frontend: {}", e);
            }
        }
        
        // Broadcast to WebSocket overlays
        if let Some(broadcaster) = PSS_EVENT_BROADCASTER.get() {
            if let Err(e) = broadcaster.send(event_json) {
                log::warn!("⚠️ Failed to broadcast PSS event to WebSocket overlays: {}", e);
            }
        }
    }

    /// Emit a custom event name to frontend with JSON payload
    pub fn emit_custom_event(event_name: &str, event_json: serde_json::Value) {
        if let Some(app_handle) = TAURI_APP_HANDLE.get() {
            if let Err(e) = app_handle.emit(event_name, event_json) {
                log::warn!("⚠️ Failed to emit custom event '{}': {}", event_name, e);
            }
        }
    }
    
    /// Emit log events to frontend for Live Data panel
    pub fn emit_log_event(log_message: String) {
        // Emit to frontend via Tauri events
        if let Some(app_handle) = TAURI_APP_HANDLE.get() {
            let log_event = serde_json::json!({
                "type": "log",
                "message": log_message,
                "timestamp": chrono::Utc::now().timestamp_millis()
            });
            
            if let Err(e) = app_handle.emit("log_event", log_event) {
                log::warn!("⚠️ Failed to emit log event to frontend: {}", e);
            }
        }
    }

    /// Get a receiver for PSS events (for WebSocket plugin)
    pub fn subscribe_to_pss_events() -> Option<broadcast::Receiver<serde_json::Value>> {
        PSS_EVENT_BROADCASTER.get().map(|broadcaster| broadcaster.subscribe())
    }
    
    /// Handle PSS events and forward them to WebSocket clients
    async fn handle_pss_to_websocket(
        mut pss_receiver: broadcast::Receiver<serde_json::Value>,
        websocket_plugin: Arc<Mutex<WebSocketPlugin>>,
    ) {
        log::info!("🔗 PSS to WebSocket bridge started");
        
        while let Ok(event) = pss_receiver.recv().await {
            let websocket_plugin_guard = websocket_plugin.lock().await;
            if let Err(e) = websocket_plugin_guard.broadcast_json_event(&event) {
                log::error!("❌ Failed to broadcast PSS event to WebSocket clients: {}", e);
            }
        }
        
        log::warn!("⚠️ PSS to WebSocket bridge stopped");
    }
    
    /// Handle UDP events
    async fn handle_udp_events(
        mut event_rx: tokio::sync::mpsc::UnboundedReceiver<crate::plugins::plugin_udp::PssEvent>,
        log_manager: Arc<Mutex<LogManager>>,
    ) {
        log::info!("🎯 Starting UDP event handler for real-time processing");
        
        while let Some(event) = event_rx.recv().await {
            // Log the event
            let event_log = format!("📡 UDP Event: {:?}", event);
            log::info!("{}", event_log);
            
            // Emit to frontend via Tauri events
            let event_json = crate::plugins::plugin_udp::UdpServer::convert_pss_event_to_json(&event);
            Self::emit_pss_event(event_json);
            
            // Forward to recording event handler for automatic recording control
            #[cfg(feature = "obs-obws")]
            {
                if let Some(app_handle) = TAURI_APP_HANDLE.get() {
                    if let Some(app) = app_handle.try_state::<Arc<App>>() {
                        let recording_handler = app.recording_event_handler();
                        log::info!("🔀 Forwarding PSS event to auto-recording handler: {:?}", event);
                        println!("🔀 Forwarding PSS event to auto-recording handler: {:?}", event);
                        // Handle PSS event for automatic recording
                        if let Err(e) = recording_handler.handle_pss_event(&event).await {
                            log::warn!("⚠️ Failed to handle PSS event for recording: {}", e);
                        }

                        // IVR: Auto round replay on challenge if enabled
                        if let crate::plugins::plugin_udp::PssEvent::Challenge { .. } = event {
                            use crate::database::operations::UiSettingsOperations as UIOps;
                            match app.database_plugin().get_connection().await {
                                Ok(conn) => {
                                    let enabled = UIOps::get_ui_setting(&*conn, "ivr.replay.auto_on_challenge")
                                        .ok().flatten().map(|s| s == "true").unwrap_or(false);
                                    if enabled {
                        if let Err(e) = app.replay_round_now(Some("OBS_REC")).await {
                                            log::warn!("⚠️ Auto IVR replay failed: {}", e);
                                        } else {
                                            log::info!("🎞️ Auto IVR replay triggered by challenge event");
                                        }
                                    }
                                }
                                Err(e) => log::warn!("⚠️ Failed to read IVR settings: {}", e),
                            }
                        }
                    }
                }
            }
            
            // Log to file
            let log_manager_guard = log_manager.lock().await;
            if let Err(e) = log_manager_guard.log("udp", "INFO", &event_log) {
                log::warn!("⚠️ Failed to log UDP event: {}", e);
            }
        }
        
        log::info!("🛑 UDP event handler stopped");
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