//! Application core and lifecycle orchestration
//!
//! Purpose: Central coordinator of configuration, plugins (UDP/PSS, OBS over obws, playback,
//! database, security), and UI-facing behaviors (mpv process control for IVR).
//!
//! Highlights:
//! - Owns and initializes all backend plugins (see `crate::plugins::mod`)
//! - Provides mpv lifecycle management (store/kill child on resume/challenge resolution)
//! - Emits structured events to the frontend via Tauri
//! - Uses obws exclusively for OBS operations; legacy plugin removed
//!
//! Concurrency:
//! - Async functions avoid holding DB connections across await points
//! - Shared state guarded by RwLock/Mutex with short critical sections

use crate::types::{AppResult, AppState, AppView};
use crate::plugins::{PlaybackPlugin, UdpPlugin, StorePlugin, LicensePlugin, CpuMonitorPlugin, ProtocolManager, DatabasePlugin, WebSocketPlugin, TournamentPlugin, EventCache, EventStreamProcessor, EventDistributor, AdvancedAnalytics};
#[cfg(feature = "youtube")]
use crate::plugins::YouTubeApiPlugin;
// Legacy ObsPluginManager removed
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
use std::process::Child;

// Global PSS event broadcaster for real-time event emission to WebSocket overlays
static PSS_EVENT_BROADCASTER: std::sync::OnceLock<broadcast::Sender<serde_json::Value>> = std::sync::OnceLock::new();

// Global Tauri app handle for real-time event emission to frontend
static TAURI_APP_HANDLE: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

/// Main application class that orchestrates all systems
pub struct App {
    state: Arc<RwLock<AppState>>,
    config_manager: ConfigManager,
    // legacy obs_plugin_manager removed
    #[cfg(feature = "obs-obws")]
    obs_obws_manager: Arc<ObsObwsManager>,
    #[cfg(feature = "obs-obws")]
    recording_event_handler: Arc<ObsRecordingEventHandler>, // Recording event handler for PSS integration
    #[cfg(feature = "youtube")]
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
    // Track last launched mpv process to close it when match resumes or challenge is resolved
    mpv_child: Arc<Mutex<Option<Child>>>,
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
        // legacy OBS plugin manager removed
        
        #[cfg(feature = "obs-obws")]
        let obs_obws_manager = Arc::new(ObsObwsManager::new());
        #[cfg(feature = "obs-obws")]
        log::info!("✅ OBS obws manager initialized");
        
        // Create recording event channel
        #[cfg(feature = "obs-obws")]
        let (recording_event_tx, _recording_event_rx) = tokio::sync::mpsc::unbounded_channel::<crate::plugins::obs_obws::RecordingEvent>();
        
        #[cfg(feature = "youtube")]
        let youtube_api_plugin = Arc::new(Mutex::new(YouTubeApiPlugin::new()));
        #[cfg(feature = "youtube")]
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
        {
            log::info!("✅ Recording event handler initialized");
            // Load persisted automatic recording config from DB into handler so it works after restart
            use crate::database::operations::UiSettingsOperations as UIOps;
            if let Ok(conn) = database_plugin.get_connection().await {
                let enabled = UIOps::get_ui_setting(&*conn, "obs.auto.enabled").ok().flatten().map(|v| v=="true").unwrap_or(false);
                let obs_name = UIOps::get_ui_setting(&*conn, "obs.auto.connection").ok().flatten().filter(|s| !s.is_empty()).unwrap_or_else(|| "OBS_REC".to_string());
                let stop_on_end = UIOps::get_ui_setting(&*conn, "obs.auto.stop_on_match_end").ok().flatten().map(|v| v=="true").unwrap_or(true);
                let stop_on_winner = UIOps::get_ui_setting(&*conn, "obs.auto.stop_on_winner").ok().flatten().map(|v| v=="true").unwrap_or(true);
                let stop_delay = UIOps::get_ui_setting(&*conn, "obs.auto.stop_delay_seconds").ok().flatten().and_then(|s| s.parse::<u32>().ok()).unwrap_or(30);
                let include_rb = UIOps::get_ui_setting(&*conn, "obs.auto.include_replay_buffer").ok().flatten().map(|v| v=="true").unwrap_or(true);
                let auto_start_rec = UIOps::get_ui_setting(&*conn, "obs.auto.start_recording_on_match_begin").ok().flatten().map(|v| v=="true").unwrap_or(true);
                let auto_start_rb = UIOps::get_ui_setting(&*conn, "obs.auto.start_replay_on_match_begin").ok().flatten().map(|v| v=="true").unwrap_or(true);
                let cfg = crate::plugins::obs_obws::AutomaticRecordingConfig {
                    enabled,
                    obs_connection_name: Some(obs_name),
                    auto_stop_on_match_end: stop_on_end,
                    auto_stop_on_winner: stop_on_winner,
                    stop_delay_seconds: stop_delay,
                    include_replay_buffer: include_rb,
                    auto_start_recording_on_match_begin: auto_start_rec,
                    auto_start_replay_on_match_begin: auto_start_rb,
                };
                if let Err(e) = recording_event_handler.update_config(cfg) {
                    log::warn!("⚠️ Failed to load automatic recording config into handler: {}", e);
                } else {
                    log::info!("✅ Automatic recording config loaded into handler");
                }
            }
        }
        
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
        
        // Load OBS connections from config manager (obws)
        let config_connections = config_manager.get_obs_connections().await;
        #[cfg(feature = "obs-obws")]
        {
            for cfg in &config_connections {
                let _ = obs_obws_manager.add_connection(crate::plugins::obs_obws::types::ObsConnectionConfig {
                    name: cfg.name.clone(),
                    host: cfg.host.clone(),
                    port: cfg.port as u16,
                    password: cfg.password.clone(),
                    timeout_seconds: 30,
                }).await;
            }
            log::info!("✅ OBS obws connections configured ({} connections)", config_connections.len());
        }
        
        Ok(Self {
            state,
            config_manager,
            // legacy obs_plugin_manager removed,
            #[cfg(feature = "obs-obws")]
            obs_obws_manager,
            #[cfg(feature = "obs-obws")]
            recording_event_handler,
            #[cfg(feature = "youtube")]
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
            mpv_child: Arc::new(Mutex::new(None)),
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
                // This task does not capture any non-Send types
                tokio::task::spawn(async move {
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
                    // This task does not capture any non-Send types
                    tokio::task::spawn(async move {
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
    
    // Legacy obs_plugin accessor fully removed
    
    #[cfg(feature = "obs-obws")]
    pub fn obs_obws_plugin(&self) -> &Arc<ObsObwsManager> {
        &self.obs_obws_manager
    }
    
    #[cfg(feature = "obs-obws")]
    pub fn recording_event_handler(&self) -> &Arc<ObsRecordingEventHandler> {
        &self.recording_event_handler
    }
    
    #[cfg(feature = "youtube")]
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
    /// Triggers an Instant Video Replay flow.
    ///
    /// Workflow: save replay buffer in OBS → wait up to maxWaitMs (DB-backed) → resolve last replay filename →
    /// spawn mpv with `--start=-secondsFromEnd` (DB-backed; default 10s). Logs every step for debugging.
    ///
    /// Arguments:
    /// - `connection_name`: optional OBS connection override; falls back to default connection.
    ///
    /// Errors: returns `AppError` if any OBS or filesystem step fails. mpv spawn failures are logged and returned.
    pub async fn replay_round_now(&self, connection_name: Option<&str>) -> AppResult<()> {
        let conn_dbg = connection_name.unwrap_or("OBS_REC");
        println!("🎞️ replay_round_now: invoked for OBS connection='{}'", conn_dbg);
        // Simple debounce to avoid repeated triggers
        static LAST_REPLAY_MS: std::sync::OnceLock<std::sync::Mutex<i64>> = std::sync::OnceLock::new();
        let now_ms = chrono::Utc::now().timestamp_millis();
        let m = LAST_REPLAY_MS.get_or_init(|| std::sync::Mutex::new(0));
        {
            let mut last = m.lock().unwrap();
            if now_ms - *last < 2000 { // 2s
                println!("⏳ replay_round_now: debounced (called again within 2s)");
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
        println!("🛠️ replay_round_now settings: mpv='{}', seconds_from_end={}, max_wait_ms={}", mpv_path, seconds_from_end, max_wait_ms);

        // Ensure RB is enabled+active: start if not active, then save
        println!("🔍 Checking replay buffer status (conn='{}')", conn_dbg);
        match self.obs_obws_plugin().get_replay_buffer_status(connection_name).await {
            Ok(status) => {
                use crate::plugins::obs_obws::types::ObsReplayBufferStatus;
                if status != ObsReplayBufferStatus::Active {
                    println!("▶️ Replay buffer inactive → starting (conn='{}')", conn_dbg);
                    let _ = self.obs_obws_plugin().start_replay_buffer(connection_name).await;
                }
            }
            Err(_) => {
                println!("⚠️ Unable to query RB status → attempting start (conn='{}')", conn_dbg);
                let _ = self.obs_obws_plugin().start_replay_buffer(connection_name).await;
            }
        }
        // Save replay buffer via obws (creates clip)
        println!("💾 Saving replay buffer (conn='{}')", conn_dbg);
        self.obs_obws_plugin().save_replay_buffer(connection_name).await?;
        println!("✅ Save replay requested");

        // Try to get last replay filename within bounded wait
        let mut filename: Option<String> = None;
        let mut elapsed: u32 = 0;
        let step: u32 = 150;
        while elapsed <= max_wait_ms {
            println!("📥 Polling last replay filename: attempt at {} ms", elapsed);
            match self.obs_obws_plugin().get_last_replay_filename(connection_name).await {
                Ok(name) if !name.is_empty() => { filename = Some(name); break; }
                _ => {}
            }
            if elapsed == max_wait_ms { break; }
            let sleep_ms = std::cmp::min(step, max_wait_ms - elapsed);
            tokio::time::sleep(std::time::Duration::from_millis(sleep_ms as u64)).await;
            elapsed += sleep_ms;
        }
        if let Some(ref name) = filename { println!("🧾 Last replay filename detected='{}'", name); } else { println!("⛔ No replay filename detected within {} ms", max_wait_ms); }

        // Build full path using OBS recording directory if available, else Videos root
        let directory = match self.obs_obws_plugin().get_record_directory(connection_name).await {
            Ok(dir) if !dir.is_empty() => std::path::PathBuf::from(dir),
            _ => crate::plugins::obs_obws::PathGeneratorConfig::detect_windows_videos_folder(),
        };
        println!("📁 Using record directory='{}'", directory.to_string_lossy());
        let file_path = match &filename {
            Some(name) => directory.join(name),
            None => directory.clone(),
        };
        println!("🔗 Resolved replay file path='{}'", file_path.to_string_lossy());

        // Launch mpv
        if !file_path.is_file() {
            return Err(crate::types::AppError::ConfigError("Replay file not found within time window".to_string()));
        }
        let start_arg = format!("--start=-{}", seconds_from_end);
        let file_arg = file_path.to_string_lossy().to_string();
        println!("🚀 Launching mpv: '{}' '{}' '{}'", mpv_path, start_arg, file_arg);
        let child = std::process::Command::new(&mpv_path)
            .arg(&start_arg)
            .arg(&file_arg)
            .spawn()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to launch mpv: {}", e)))?;
        // Track mpv process so it can be closed later
        {
            let mut slot = self.mpv_child.lock().await;
            if let Some(old) = slot.as_mut() {
                let _ = old.try_wait();
            }
            *slot = Some(child);
        }
        println!("✅ mpv launched");

        // Index replay video into recorded_videos
        if let Some(name) = &filename {
            let name_owned = name.clone();
            let created = chrono::Utc::now();
            let start_time = created - chrono::Duration::seconds(seconds_from_end as i64);
            // Resolve current match DB id from WebSocket plugin (fast path); fallback to most recent match
            let match_id_db: Option<i64> = {
                let ws = self.websocket_plugin().lock().await;
                ws.get_current_match_db_id()
            };
            // Resolve active tournament/day IDs for better indexing
            let (tid_opt, day_opt) = {
                match self.database_plugin().get_connection().await {
                    Ok(conn2) => {
                        use crate::database::operations::TournamentOperations as TOps;
                        let t = TOps::get_active_tournament(&*conn2).ok().flatten();
                        let d = t.as_ref().and_then(|tt| TOps::get_active_tournament_day(&*conn2, tt.id.unwrap()).ok().flatten());
                        (t.and_then(|tt| tt.id), d.and_then(|dd| dd.id))
                    },
                    Err(_) => (None, None)
                }
            };
            let directory = match self.obs_obws_plugin().get_record_directory(connection_name).await {
                Ok(dir) if !dir.is_empty() => dir,
                _ => crate::plugins::obs_obws::PathGeneratorConfig::detect_windows_videos_folder().to_string_lossy().to_string(),
            };
            let file_path_str = std::path::PathBuf::from(&directory).join(name_owned).to_string_lossy().to_string();
            // Acquire a fresh short-lived connection only for DB writes to avoid holding across await
            if let Ok(conn2) = self.database_plugin().get_connection().await {
                let conn_ref2 = &*conn2;
                let _ = if let Some(dbid) = match_id_db {
                    conn_ref2.execute(
                        "INSERT INTO recorded_videos (match_id, event_id, tournament_id, tournament_day_id, video_type, file_path, record_directory, filename_formatting, start_time, duration_seconds, created_at) VALUES (?, NULL, ?, ?, 'replay', ?, ?, NULL, ?, ?, ?)",
                        rusqlite::params![ dbid, tid_opt, day_opt, file_path_str, directory, start_time.to_rfc3339(), seconds_from_end as i32, created.to_rfc3339() ]
                    )
                } else {
                    // Fallback: map pss_matches by most recent created_at
                    conn_ref2.execute(
                        "INSERT INTO recorded_videos (match_id, event_id, tournament_id, tournament_day_id, video_type, file_path, record_directory, filename_formatting, start_time, duration_seconds, created_at) VALUES ((SELECT id FROM pss_matches ORDER BY created_at DESC LIMIT 1), NULL, ?, ?, 'replay', ?, ?, NULL, ?, ?, ?)",
                        rusqlite::params![ tid_opt, day_opt, file_path_str, directory, start_time.to_rfc3339(), seconds_from_end as i32, created.to_rfc3339() ]
                    )
                };
            }
        }

        Ok(())
    }

    /// Close mpv if it is currently running (used on resume or challenge accept/reject)
    /// Closes a running mpv instance if it was launched by the app.
    ///
    /// Called on match resume or challenge resolution to avoid replay overlapping with live action.
    pub async fn close_mpv_if_running(&self) -> AppResult<()> {
        let mut slot = self.mpv_child.lock().await;
        if let Some(mut child) = slot.take() {
            if child.try_wait().ok().flatten().is_some() {
                println!("🧹 mpv already exited");
                return Ok(());
            }
            println!("⏹️ Attempting to close mpv (killing process)");
            if let Err(e) = child.kill() {
                println!("⚠️ Failed to kill mpv: {}", e);
            } else {
                println!("✅ mpv closed");
            }
        }
        Ok(())
    }

    /// Open the recorded video for a given event at the exact event timestamp
    /// Opens the recording that contains the specified event and seeks to the precise event offset.
    ///
    /// Resolves the matching `recorded_videos` row by match/day/tournament context, computes offset from
    /// `recorded_videos.start_time` and the event timestamp, then launches mpv.
    pub async fn open_event_video(&self, event_id: i64) -> AppResult<()> {
        // Query event (match_id, timestamp) without holding connection across await boundaries
        let (match_db_id, event_time_str) = {
            let conn_guard = self.database_plugin().get_connection().await.map_err(|e| crate::types::AppError::ConfigError(e.to_string()))?;
            let conn_ref = &*conn_guard;
            let row: (i64, String) = conn_ref
                .query_row(
                    "SELECT match_id, timestamp FROM pss_events_v2 WHERE id = ?",
                    rusqlite::params![event_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|e| crate::types::AppError::ConfigError(e.to_string()))?;
            row
        };

        let event_time = match chrono::DateTime::parse_from_rfc3339(&event_time_str) {
            Ok(dt) => dt.with_timezone(&chrono::Utc),
            Err(e) => return Err(crate::types::AppError::ConfigError(format!("Invalid event time: {}", e)))
        };

        // Prefer recording that started before event_time; fallback to the earliest after
        let (record_path_opt, _record_dir_opt, record_start_str_opt) = {
            let conn_guard = self.database_plugin().get_connection().await.map_err(|e| crate::types::AppError::ConfigError(e.to_string()))?;
            let conn_ref = &*conn_guard;
            let before: Option<(Option<String>, Option<String>, Option<String>)> = conn_ref
                .query_row(
                    "SELECT file_path, record_directory, start_time FROM recorded_videos
                     WHERE match_id = ? AND video_type = 'recording' AND start_time <= ?
                     ORDER BY start_time DESC LIMIT 1",
                    rusqlite::params![match_db_id, event_time.to_rfc3339()],
                    |row| Ok((row.get(0).ok(), row.get(1).ok(), row.get(2).ok())),
                )
                .ok();
            if let Some(t) = before { t } else {
                conn_ref
                    .query_row(
                        "SELECT file_path, record_directory, start_time FROM recorded_videos
                         WHERE match_id = ? AND video_type = 'recording' AND start_time > ?
                         ORDER BY start_time ASC LIMIT 1",
                        rusqlite::params![match_db_id, event_time.to_rfc3339()],
                        |row| Ok((row.get(0).ok(), row.get(1).ok(), row.get(2).ok())),
                    )
                    .unwrap_or((None, None, None))
            }
        };

        // Resolve a concrete file path. Prefer the recording row with a valid file_path; else try replay rows near the event time.
        let mut file_path: Option<std::path::PathBuf> = record_path_opt.clone().map(std::path::PathBuf::from);
        if file_path.as_ref().map(|p| p.is_file()).unwrap_or(false) == false {
            // Try to find a replay near the event time
            let (rp_opt, _rd_opt): (Option<String>, Option<String>) = {
                let conn_guard = self.database_plugin().get_connection().await.map_err(|e| crate::types::AppError::ConfigError(e.to_string()))?;
                let conn_ref = &*conn_guard;
                // Prefer replay started before event
                let before: Option<(Option<String>, Option<String>)> = conn_ref
                    .query_row(
                        "SELECT file_path, record_directory FROM recorded_videos
                         WHERE match_id = ? AND video_type = 'replay' AND start_time <= ?
                         ORDER BY start_time DESC LIMIT 1",
                        rusqlite::params![match_db_id, event_time.to_rfc3339()],
                        |row| Ok((row.get(0).ok(), row.get(1).ok())),
                    )
                    .ok();
                if let Some(t) = before { t } else {
                    conn_ref
                        .query_row(
                            "SELECT file_path, record_directory FROM recorded_videos
                             WHERE match_id = ? AND video_type = 'replay' AND start_time > ?
                             ORDER BY start_time ASC LIMIT 1",
                            rusqlite::params![match_db_id, event_time.to_rfc3339()],
                            |row| Ok((row.get(0).ok(), row.get(1).ok())),
                        )
                        .unwrap_or((None, None))
                }
            };
            if let Some(fp) = rp_opt { file_path = Some(std::path::PathBuf::from(fp)); }
        }
        let file_path = match file_path {
            Some(p) if p.is_file() => p,
            _ => return Err(crate::types::AppError::ConfigError("No recorded video file found for this event".to_string()))
        };
        let start_time = record_start_str_opt
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or(event_time);
        let mut offset_secs = (event_time - start_time).num_seconds();
        if offset_secs < 0 {
            offset_secs = 0;
        }

        // Resolve mpv path
        let mpv_path: String = {
            use crate::database::operations::UiSettingsOperations as UIOps;
            let conn_guard = self.database_plugin().get_connection().await.map_err(|e| crate::types::AppError::ConfigError(e.to_string()))?;
            let conn_ref = &*conn_guard;
            UIOps::get_ui_setting(conn_ref, "ivr.replay.mpv_path")
                .ok()
                .flatten()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "mpv".to_string())
        };

        // Close existing mpv if any
        let _ = self.close_mpv_if_running().await;

        // Launch mpv with positive offset from recording start
        let start_arg = format!("--start=+{}", offset_secs.max(0));
        println!(
            "🎬 Opening event video: '{}' '{}'",
            start_arg,
            file_path.to_string_lossy()
        );
        let child = std::process::Command::new(&mpv_path)
            .arg(&start_arg)
            .arg(file_path.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to launch mpv: {}", e)))?;
        {
            let mut slot = self.mpv_child.lock().await;
            *slot = Some(child);
        }

        Ok(())
    }

    /// Get the default connection name for OBS operations
    pub async fn get_default_connection_name(&self) -> AppResult<String> {
        // Try to get the first available obws connection name
        #[cfg(feature = "obs-obws")]
        let connection_names = self.obs_obws_manager.get_connection_names().await;
        #[cfg(not(feature = "obs-obws"))]
        let connection_names: Vec<String> = Vec::new();
        
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

    /// Open a video file with mpv at the given positive offset (seconds)
    /// Opens a video file at the given offset in seconds using mpv.
    ///
    /// Validates the path exists (`is_file()`) and closes any active mpv before launching a new one.
    pub async fn open_video_at(&self, file_path: String, offset_seconds: i64) -> AppResult<()> {
        // Resolve mpv path from settings
        let mpv_path: String = {
            use crate::database::operations::UiSettingsOperations as UIOps;
            let conn_guard = self.database_plugin().get_connection().await.map_err(|e| crate::types::AppError::ConfigError(e.to_string()))?;
            let conn_ref = &*conn_guard;
            UIOps::get_ui_setting(conn_ref, "ivr.replay.mpv_path")
                .ok()
                .flatten()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "mpv".to_string())
        };

        // Close any running mpv first
        let _ = self.close_mpv_if_running().await;

        // Launch mpv
        let start_arg = format!("--start=+{}", std::cmp::max(0, offset_seconds));
        println!("🎬 Opening video: '{}' '{}'", start_arg, &file_path);
        let child = std::process::Command::new(&mpv_path)
            .arg(&start_arg)
            .arg(file_path)
            .spawn()
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to launch mpv: {}", e)))?;
        {
            let mut slot = self.mpv_child.lock().await;
            *slot = Some(child);
        }
        Ok(())
    }
    
    /// Start UDP event handler manually (for when UDP is started manually)
    pub async fn start_udp_event_handler(&self) {
        if let Some(udp_event_rx) = self.udp_event_rx.lock().await.take() {
            let log_manager_clone = self.log_manager().clone();
            // This task does not capture any non-Send types
            tokio::task::spawn(async move {
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

                        // Close mpv if match resumes or challenge resolved
                        match event {
                            crate::plugins::plugin_udp::PssEvent::Clock { action: Some(ref a), .. } if a == "start" => {
                                println!("⏹️ Closing mpv on clock start (resume)");
                                if let Err(e) = app.close_mpv_if_running().await { println!("⚠️ close_mpv_if_running error: {}", e); }
                            }
                            crate::plugins::plugin_udp::PssEvent::Challenge { accepted, .. } => {
                                if matches!(accepted, Some(true) | Some(false)) {
                                    println!("⏹️ Closing mpv on challenge resolution (accepted/rejected)");
                                    if let Err(e) = app.close_mpv_if_running().await { println!("⚠️ close_mpv_if_running error: {}", e); }
                                }
                            }
                            _ => {}
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