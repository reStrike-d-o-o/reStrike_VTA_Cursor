use crate::database::models::{
    PssEventV2 as DbPssEvent, PssMatch, PssMatchAthlete, UdpServerConfig as DbUdpServerConfig,
};
use crate::plugins::performance_monitor::PerformanceMonitor;
use crate::plugins::plugin_database::DatabasePlugin;
use crate::plugins::plugin_websocket::WebSocketServer;
use crate::plugins::ProtocolManager;
use crate::pss::protocol::{PssEvent, PssProtocol};
use crate::types::{AppError, AppResult};
use chrono::Utc;
use std::collections::VecDeque;
use std::io;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Initialize the UDP plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Initializing UDP plugin...");
    Ok(())
}

// Re-export the main plugin type
pub type UdpPlugin = UdpServer;

type RecentHitMap = Arc<Mutex<std::collections::HashMap<u8, Vec<(u8, std::time::SystemTime)>>>>;

#[derive(Debug, Clone)]
pub struct UdpServerConfig {
    pub port: u16,
    pub bind_address: String,
    pub enabled: bool,
    pub auto_start: bool,
}

impl Default for UdpServerConfig {
    fn default() -> Self {
        Self {
            port: 8888,
            bind_address: "127.0.0.1".to_string(),
            enabled: true,
            auto_start: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UdpServerStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

#[derive(Clone)]
pub struct UdpServer {
    status: Arc<Mutex<UdpServerStatus>>,
    event_tx: mpsc::UnboundedSender<PssEvent>,
    socket: Arc<Mutex<Option<UdpSocket>>>,
    stats: Arc<Mutex<UdpStats>>,
    protocol_manager: Arc<ProtocolManager>,
    protocol_parser: Arc<PssProtocol>,
    recent_events: Arc<Mutex<VecDeque<PssEvent>>>,
    database: Arc<DatabasePlugin>,
    current_session_id: Arc<Mutex<Option<i64>>>,
    current_match_id: Arc<Mutex<Option<i64>>>,
    athlete_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
    event_type_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
    listener_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    // Hit level tracking for statistics
    recent_hit_levels: RecentHitMap, // athlete -> [(level, timestamp)]
    // Tournament context tracking
    current_tournament_id: Arc<Mutex<Option<i64>>>,

    // Phase 1 Optimization: Event batching for high-volume processing
    event_batch: Arc<Mutex<Vec<PssEvent>>>,
    batch_processor_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    batch_tx: mpsc::UnboundedSender<PssEvent>,
    // Phase 1 Optimization: Performance monitoring
    performance_monitor: Arc<PerformanceMonitor>,
    // WebSocket server for real-time event broadcasting
    websocket_server: Arc<WebSocketServer>,
}

// Phase 1 Optimization: Performance monitoring structs

#[derive(Debug, Clone, Default)]
pub struct UdpStats {
    pub packets_received: u64,
    pub packets_parsed: u64,
    pub parse_errors: u64,
    pub last_packet_time: Option<std::time::SystemTime>,
    pub connected_clients: usize,
    pub active_connections: std::collections::HashMap<std::net::SocketAddr, std::time::SystemTime>,
    pub server_start_time: Option<std::time::SystemTime>,
    pub total_bytes_received: u64,
    pub average_packet_size: f64,
}

impl UdpServer {
    pub fn new(
        event_tx: mpsc::UnboundedSender<PssEvent>,
        protocol_manager: Arc<ProtocolManager>,
        database: Arc<DatabasePlugin>,
    ) -> Self {
        let (batch_tx, batch_rx) = mpsc::unbounded_channel::<PssEvent>();
        let performance_monitor = Arc::new(PerformanceMonitor::new());

        // Create WebSocket server for real-time event broadcasting
        let websocket_server = Arc::new(WebSocketServer::new(event_tx.clone()));

        let server = Self {
            status: Arc::new(Mutex::new(UdpServerStatus::Stopped)),
            event_tx,
            socket: Arc::new(Mutex::new(None)),
            stats: Arc::new(Mutex::new(UdpStats::default())),
            protocol_manager,
            protocol_parser: Arc::new(PssProtocol::new()),
            recent_events: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            database,
            current_session_id: Arc::new(Mutex::new(None)),
            current_match_id: Arc::new(Mutex::new(None)),
            athlete_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            event_type_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            listener_task: Arc::new(Mutex::new(None)),
            recent_hit_levels: Arc::new(Mutex::new(std::collections::HashMap::new())),
            current_tournament_id: Arc::new(Mutex::new(None)),

            event_batch: Arc::new(Mutex::new(Vec::new())),
            batch_processor_task: Arc::new(Mutex::new(None)),
            batch_tx,
            performance_monitor: performance_monitor.clone(),
            websocket_server,
        };

        // Start batch processor
        let server_clone = server.clone_for_batch_processor();
        let batch_task = tokio::spawn(async move {
            Self::batch_processor_loop(batch_rx, server_clone).await;
        });
        if let Ok(mut task_guard) = server.batch_processor_task.lock() {
            *task_guard = Some(batch_task);
        }

        server
    }

    /// Clone the server for batch processor (without the batch_tx to avoid double ownership)
    fn clone_for_batch_processor(&self) -> Self {
        Self {
            status: self.status.clone(),
            event_tx: self.event_tx.clone(),
            socket: self.socket.clone(),
            stats: self.stats.clone(),
            protocol_manager: self.protocol_manager.clone(),
            protocol_parser: self.protocol_parser.clone(),
            recent_events: self.recent_events.clone(),
            database: self.database.clone(),
            current_session_id: self.current_session_id.clone(),
            current_match_id: self.current_match_id.clone(),
            athlete_cache: self.athlete_cache.clone(),
            event_type_cache: self.event_type_cache.clone(),
            listener_task: self.listener_task.clone(),
            recent_hit_levels: self.recent_hit_levels.clone(),
            current_tournament_id: self.current_tournament_id.clone(),

            event_batch: self.event_batch.clone(),
            batch_processor_task: self.batch_processor_task.clone(),
            batch_tx: mpsc::unbounded_channel().0, // Dummy channel for clone
            performance_monitor: self.performance_monitor.clone(),
            websocket_server: self.websocket_server.clone(),
        }
    }

    /// Phase 1 Optimization: Batch processor loop for high-volume event processing
    async fn batch_processor_loop(mut batch_rx: mpsc::UnboundedReceiver<PssEvent>, server: Self) {
        const BATCH_SIZE: usize = 100; // Process 100 events per batch
        const BATCH_TIMEOUT: Duration = Duration::from_millis(500); // 500ms timeout

        let mut batch = Vec::with_capacity(BATCH_SIZE);
        let mut last_batch_time = Instant::now();

        log::info!("Starting batch processor for high-volume event processing");

        while let Some(event) = batch_rx.recv().await {
            batch.push(event);

            let should_process =
                batch.len() >= BATCH_SIZE || last_batch_time.elapsed() >= BATCH_TIMEOUT;

            if should_process {
                let events_to_process =
                    std::mem::replace(&mut batch, Vec::with_capacity(BATCH_SIZE));
                last_batch_time = Instant::now();

                // Process batch asynchronously
                let server_clone = server.clone_for_batch_processor();
                tokio::spawn(async move {
                    if let Err(e) = Self::process_event_batch(server_clone, events_to_process).await
                    {
                        log::error!("Batch processing failed: {e}");
                    }
                });
            }
        }

        // Process remaining events
        if !batch.is_empty() {
            if let Err(e) = Self::process_event_batch(server, batch).await {
                log::error!("Final batch processing failed: {e}");
            }
        }

        log::info!("Batch processor stopped");
    }

    /// Phase 1 Optimization: Process a batch of events efficiently
    async fn process_event_batch(server: Self, events: Vec<PssEvent>) -> AppResult<()> {
        let start_time = Instant::now();
        let batch_size = events.len();

        log::debug!("Processing batch of {batch_size} events");

        // Phase 1 Optimization: Update memory usage
        server.performance_monitor.update_memory_usage();

        // Use a single transaction for the entire batch
        let database = server.database.clone();
        let current_session_id = server.current_session_id.clone();
        let current_match_id = server.current_match_id.clone();
        let athlete_cache = server.athlete_cache.clone();
        let event_type_cache = server.event_type_cache.clone();
        let recent_hit_levels = server.recent_hit_levels.clone();
        let current_tournament_id = server.current_tournament_id.clone();

        // Process events in parallel within the batch
        let mut tasks = Vec::new();
        for event in events {
            let db_clone = database.clone();
            let session_clone = current_session_id.clone();
            let match_clone = current_match_id.clone();
            let athlete_clone = athlete_cache.clone();
            let event_type_clone = event_type_cache.clone();
            let hit_levels_clone = recent_hit_levels.clone();
            let tournament_clone = current_tournament_id.clone();

            let websocket_server_clone = server.websocket_server.clone();

            let task = tokio::spawn(async move {
                Self::store_event_in_database(
                    &db_clone,
                    &session_clone,
                    &match_clone,
                    &athlete_clone,
                    &event_type_clone,
                    &event,
                    &hit_levels_clone,
                    &tournament_clone,
                    &websocket_server_clone,
                )
                .await
            });
            tasks.push(task);
        }

        // Wait for all tasks to complete
        let mut success_count = 0;
        let mut error_count = 0;
        for task in tasks {
            match task.await {
                Ok(Ok(_)) => {
                    success_count += 1;
                    // Phase 1 Optimization: Record successful event processing
                    server.performance_monitor.record_event_arrival();
                }
                Ok(Err(e)) => {
                    error_count += 1;
                    log::warn!("Event storage failed in batch: {e}");
                }
                Err(e) => {
                    error_count += 1;
                    log::warn!("Task failed in batch: {e}");
                }
            }
        }

        let processing_time = start_time.elapsed();

        // Phase 1 Optimization: Record batch processing metrics
        server
            .performance_monitor
            .record_event_processed(processing_time.as_millis() as u64);

        log::info!(
            "Batch processed: {success_count}/{batch_size} events in {processing_time:?} ({error_count} errors)"
        );

        // Update statistics
        if let Ok(mut stats) = server.stats.lock() {
            stats.packets_parsed += success_count as u64;
            stats.parse_errors += error_count as u64;
        }

        Ok(())
    }

    pub async fn start(&self, config: &crate::config::types::AppConfig) -> AppResult<()> {
        log::info!("Starting UDP server...");

        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Starting;
        }

        // WebSocket server is started by the core app, no need to start it here
        log::info!("WebSocket server will be managed by core app");

        // Initialize event type cache
        if let Err(e) = self.initialize_event_type_cache().await {
            log::error!("Failed to initialize event type cache: {e}");
            return Err(e);
        }

        let network_settings = &config.udp.listener.network_interface;

        // Check if already running
        {
            let status = self.status.lock().unwrap();
            if matches!(*status, UdpServerStatus::Running) {
                return Ok(()); // Already running
            }
        }

        // Use app config settings instead of internal config
        let port = config.udp.listener.port;
        let bind_address = config.udp.listener.bind_address.clone();
        let enabled = config.udp.listener.enabled;

        // Create database session first
        let db_config = DbUdpServerConfig {
            id: None,
            name: "Default PSS Server".to_string(),
            port,
            bind_address: bind_address.clone(),
            network_interface_id: None,
            enabled,
            auto_start: true,
            max_packet_size: 8192,
            buffer_size: 8192,
            timeout_ms: 30000,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let config_id = self.database.upsert_udp_server_config(&db_config).await?;
        let session_id = self.database.create_udp_server_session(config_id).await?;

        {
            let mut current_session = self.current_session_id.lock().unwrap();
            *current_session = Some(session_id);
        }

        // Determine the best IP address to bind to
        let bind_ip = if network_settings.auto_detect {
            match crate::utils::NetworkDetector::get_best_ip_address(network_settings) {
                Ok(ip) => {
                    log::info!("Auto-detected network interface IP: {ip}");
                    ip.to_string()
                }
                Err(e) => {
                    log::warn!("Failed to auto-detect network interface: {e}");
                    log::info!("Falling back to configured bind address: {bind_address}");
                    bind_address.clone()
                }
            }
        } else {
            log::info!("Using configured bind address: {bind_address}");
            bind_address.clone()
        };

        let mut bind_candidates = Vec::new();
        bind_candidates.push(bind_ip.clone());
        if bind_ip != bind_address {
            bind_candidates.push(bind_address.clone());
        }
        if network_settings.fallback_to_localhost
            && !bind_candidates
                .iter()
                .any(|candidate| candidate == "127.0.0.1")
        {
            bind_candidates.push("127.0.0.1".to_string());
        }

        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Starting;
        }

        let mut last_error: Option<io::Error> = None;
        let mut bind_addr = String::new();
        let mut socket: Option<UdpSocket> = None;

        for candidate in bind_candidates {
            let candidate_addr = format!("{candidate}:{port}");
            log::info!("Attempting to bind UDP server to: {candidate_addr}");

            match UdpSocket::bind(&candidate_addr) {
                Ok(s) => {
                    if let Err(e) = s.set_nonblocking(true) {
                        log::warn!("Failed to configure UDP socket for {candidate_addr}: {e}");
                        last_error = Some(e);
                        continue;
                    }
                    bind_addr = candidate_addr;
                    socket = Some(s);
                    break;
                }
                Err(e) => {
                    log::warn!("Failed to bind UDP socket to {candidate_addr}: {e}");
                    last_error = Some(e);
                }
            }
        }

        let socket = if let Some(socket) = socket {
            socket
        } else {
            let error_msg = if let Some(e) = last_error {
                format!("Failed to bind UDP socket after trying all candidates: {e}")
            } else {
                "Failed to bind UDP socket: no candidates available".to_string()
            };
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Error(error_msg.clone());
            return Err(AppError::ConfigError(error_msg));
        };

        // Store the socket
        {
            let mut socket_guard = self.socket.lock().unwrap();
            *socket_guard = Some(socket);
        }

        // Update status to running and set start time
        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Running;
        }

        // Set server start time
        {
            let mut stats = self.stats.lock().unwrap();
            stats.server_start_time = Some(std::time::SystemTime::now());
        }

        // Start the listening loop in a tokio task
        let socket_clone = self.socket.clone();
        let event_tx = self.event_tx.clone();
        let status_clone = self.status.clone();
        let stats_clone = self.stats.clone();
        let protocol_manager = self.protocol_manager.clone();
        let protocol_parser = self.protocol_parser.clone();
        let recent_events_clone = self.recent_events.clone();
        let database_clone = self.database.clone();
        let current_session_id_clone = self.current_session_id.clone();
        let current_match_id_clone = self.current_match_id.clone();
        let athlete_cache_clone = self.athlete_cache.clone();
        let event_type_cache_clone = self.event_type_cache.clone();
        let recent_hit_levels_clone = self.recent_hit_levels.clone();
        let tournament_id_clone = self.current_tournament_id.clone();

        let websocket_server_clone = self.websocket_server.clone();

        let listener_task = tokio::spawn(async move {
            Self::listen_loop_async(
                socket_clone,
                event_tx,
                status_clone,
                stats_clone,
                protocol_manager,
                protocol_parser,
                recent_events_clone,
                database_clone,
                current_session_id_clone,
                current_match_id_clone,
                athlete_cache_clone,
                event_type_cache_clone,
                recent_hit_levels_clone,
                tournament_id_clone,
                websocket_server_clone,
            )
            .await;
        });

        {
            let mut listener_task_guard = self.listener_task.lock().unwrap();
            *listener_task_guard = Some(listener_task);
        }

        log::info!("UDP server started on {bind_addr}");

        // Log server start for Live Data panel
        let start_log_message = format!(" UDP server started on {bind_addr}");
        crate::core::app::App::emit_log_event(start_log_message);

        Ok(())
    }

    pub async fn stop(&self) -> AppResult<()> {
        log::info!("Stopping UDP server...");

        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Stopped;
        }

        // Stop WebSocket server
        if let Err(e) = self.websocket_server.stop().await {
            log::warn!("Failed to stop WebSocket server: {e}");
        } else {
            log::info!("WebSocket server stopped");
        }

        // Stop listener task
        if let Ok(mut task_guard) = self.listener_task.lock() {
            if let Some(task) = task_guard.take() {
                task.abort();
            }
        }

        // Stop batch processor task
        if let Ok(mut task_guard) = self.batch_processor_task.lock() {
            if let Some(task) = task_guard.take() {
                task.abort();
            }
        }

        // Close socket
        {
            if let Ok(mut socket_guard) = self.socket.lock() {
                *socket_guard = None;
            }
        }

        log::info!("UDP server stopped successfully");
        Ok(())
    }

    pub fn get_status(&self) -> UdpServerStatus {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or(UdpServerStatus::Stopped)
    }

    pub fn get_stats(&self) -> UdpStats {
        self.stats
            .lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }

    pub fn get_recent_events(&self) -> Vec<PssEvent> {
        self.recent_events
            .lock()
            .map(|events| events.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Phase 1 Optimization: Get performance metrics
    pub fn get_performance_metrics(
        &self,
    ) -> crate::plugins::performance_monitor::PerformanceMetrics {
        self.performance_monitor.get_performance_metrics()
    }

    pub fn websocket_client_count(&self) -> usize {
        self.websocket_server.get_client_count()
    }

    pub fn match_in_progress(&self) -> bool {
        self.websocket_server.get_match_started()
    }

    pub fn current_match_db_id(&self) -> Option<i64> {
        self.websocket_server.get_current_match_db_id()
    }

    pub fn status_snapshot(&self) -> UdpStats {
        self.stats
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub async fn set_tournament_context(&self, tournament_id: Option<i64>) -> AppResult<()> {
        if let Ok(mut guard) = self.current_tournament_id.lock() {
            *guard = tournament_id;
        }
        Ok(())
    }

    pub fn get_tournament_context(&self) -> Option<i64> {
        self
            .current_tournament_id
            .lock()
            .ok()
            .and_then(|guard| *guard)
    }

    pub async fn clear_tournament_context(&self) -> AppResult<()> {
        self.set_tournament_context(None).await
    }

    /// Phase 1 Optimization: Get memory usage
    pub fn get_memory_usage(&self) -> crate::plugins::performance_monitor::MemoryUsageStats {
        self.performance_monitor.get_memory_stats()
    }

    pub fn add_event(&self, event: PssEvent) {
        // Add to recent events (existing logic)
        {
            if let Ok(mut recent) = self.recent_events.lock() {
                if recent.len() >= 100 {
                    recent.pop_front();
                }
                recent.push_back(event.clone());
            }
        }

        // Phase 1 Optimization: Send to batch processor for high-volume processing
        // This will handle database storage and WebSocket broadcasting
        if let Err(e) = self.batch_tx.send(event) {
            log::error!("Failed to send event to batch processor: {e}");
        }

        // Removed duplicate event_tx.send() to prevent event duplication
    }

    pub async fn update_config(&self, port: u16, bind_address: String) {
        log::info!(
            "Updating UDP server configuration to port: {port} and bind address: {bind_address}"
        );
        // Note: The actual configuration update will be handled in the start() method
        // This method is called to log the configuration change
    }

    async fn initialize_event_type_cache(&self) -> AppResult<()> {
        match self.database.get_pss_event_types().await {
            Ok(event_types) => {
                if let Ok(mut cache) = self.event_type_cache.lock() {
                    for event_type in event_types {
                        if let Some(id) = event_type.id {
                            cache.insert(event_type.event_code.clone(), id);
                        }
                    }
                    log::info!("Event type cache initialized with {} types", cache.len());
                }
                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to initialize event type cache: {e}. Continuing without cache.");
                // Don't fail the entire UDP server startup if event type cache fails
                Ok(())
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn store_event_in_database(
        database: &DatabasePlugin,
        current_session_id: &Arc<Mutex<Option<i64>>>,
        current_match_id: &Arc<Mutex<Option<i64>>>,
        _athlete_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
        event_type_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
        event: &PssEvent,
        recent_hit_levels: &RecentHitMap,
        current_tournament_id: &Arc<Mutex<Option<i64>>>,
        websocket_server: &Arc<WebSocketServer>,
    ) -> AppResult<()> {
        let start_time = Instant::now();

        // Do not filter persistence: keep all events (including Clock) so time/round can be reconstructed accurately.

        // Get session ID
        let session_id = {
            let session_guard = current_session_id.lock().unwrap();
            session_guard.ok_or_else(|| AppError::ConfigError("No active session".to_string()))?
        };

        // Ensure a current match exists so every event is attached to a valid match
        {
            let has_match = { current_match_id.lock().unwrap().is_some() };
            if !has_match {
                let auto_match_key = format!("auto_{}", Utc::now().format("%Y%m%d%H%M%S%3f"));
                let mut pss_match = PssMatch::new(auto_match_key.clone());
                pss_match.creation_mode = "Automatic".to_string();
                match database.insert_pss_match(&pss_match).await {
                    Ok(new_id) if new_id > 0 => {
                        {
                            let mut guard = current_match_id.lock().unwrap();
                            *guard = Some(new_id);
                        }
                        websocket_server.set_current_match_db_id(Some(new_id));
                        log::info!(
                            "ensure_current_match: created match {auto_match_key} (db id {new_id})"
                        );

                        let tournament_id_snapshot = {
                            let guard = current_tournament_id.lock().unwrap();
                            *guard
                        };
                        if let Some(tid) = tournament_id_snapshot {
                            if let Err(err) = database
                                .set_pss_match_tournament_context(new_id, Some(tid))
                                .await
                            {
                                log::warn!(
                                    "Failed to apply tournament context to match {new_id}: {err}"
                                );
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => log::warn!(
                        "ensure_current_match: failed to insert match {auto_match_key}: {e}"
                    ),
                }
            }
        }

        // Pre-process specific PSS events to maintain DB context (match + athletes)
        match event {
            // Start a fresh match context at FightLoaded to avoid attaching early
            // system events of a new fight to the previous match
            PssEvent::FightLoaded => {
                // Always create a brand new match row for each fight instance
                let auto_match_key = format!("auto_{}", Utc::now().format("%Y%m%d%H%M%S%3f"));
                let mut pss_match = PssMatch::new(auto_match_key.clone());
                pss_match.creation_mode = "Automatic".to_string();
                match database.insert_pss_match(&pss_match).await {
                    Ok(db_match_id) if db_match_id > 0 => {
                        {
                            let mut guard = current_match_id.lock().unwrap();
                            *guard = Some(db_match_id);
                        }
                        websocket_server.set_current_match_db_id(Some(db_match_id));
                        log::info!(
                            "FightLoaded: started new match {auto_match_key} (db id {db_match_id})"
                        );

                        let tournament_id_snapshot = {
                            let guard = current_tournament_id.lock().unwrap();
                            *guard
                        };
                        if let Some(tid) = tournament_id_snapshot {
                            if let Err(err) = database
                                .set_pss_match_tournament_context(db_match_id, Some(tid))
                                .await
                            {
                                log::warn!(
                                    "Failed to set tournament context for match {db_match_id}: {err}"
                                );
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        log::warn!("FightLoaded: failed to insert match {auto_match_key}: {e}")
                    }
                }
            }
            PssEvent::MatchConfig {
                number,
                category,
                weight,
                match_id,
                division,
                total_rounds,
                round_duration,
                countdown_type,
                format,
                ..
            } => {
                // Determine effective match identifier (fallback to number when match_id is empty/null)
                let effective_match_id =
                    if match_id.trim().is_empty() || match_id.eq_ignore_ascii_case("null") {
                        format!("mch:{number}")
                    } else {
                        match_id.clone()
                    };

                // Update only the metadata of the existing current match row.
                let db_match_id = {
                    let guard = current_match_id.lock().unwrap();
                    guard.unwrap_or_default()
                };
                if db_match_id == 0 {
                    log::warn!("MatchConfig without current match id; ignoring metadata update");
                    return Ok(());
                }
                if let Err(err) = database
                    .rename_pss_match_id(db_match_id, &effective_match_id)
                    .await
                {
                    log::warn!(
                        "Failed to rename match {db_match_id} to {effective_match_id}: {err}"
                    );
                }

                // Update match metadata
                let mut pss_match =
                    crate::database::models::PssMatch::new(effective_match_id.clone());
                pss_match.id = Some(db_match_id);
                pss_match.match_number = Some(number.clone());
                pss_match.category = Some(category.clone());
                pss_match.weight_class = Some(weight.clone());
                pss_match.division = Some(division.clone());
                pss_match.total_rounds = *total_rounds as i32;
                pss_match.round_duration = Some(*round_duration as i32);
                pss_match.countdown_type = Some(countdown_type.clone());
                pss_match.format_type = Some(*format as i32);
                pss_match.updated_at = Utc::now();

                if let Err(e) = database.update_pss_match(db_match_id, &pss_match).await {
                    log::warn!("Failed to update PSS match {effective_match_id}: {e}");
                } else {
                    log::info!("Current match set: {effective_match_id} (db id {db_match_id})");
                    log::info!("Current match set: {effective_match_id} (db id {db_match_id})");

                    let tournament_id_snapshot = {
                        let guard = current_tournament_id.lock().unwrap();
                        *guard
                    };
                    if let Some(tid) = tournament_id_snapshot {
                        if let Err(err) = database
                            .set_pss_match_tournament_context(db_match_id, Some(tid))
                            .await
                        {
                            log::warn!(
                                "Failed to set tournament context for match {db_match_id}: {err}"
                            );
                        }
                    }
                }

                // Attempt to link any pending athletes captured before match was configured
                let (pending_a1, pending_a2) = {
                    // Read pending ids without holding the lock across await
                    if let Ok(cache) = _athlete_cache.lock() {
                        (
                            cache.get("__PENDING_A1_ID").copied(),
                            cache.get("__PENDING_A2_ID").copied(),
                        )
                    } else {
                        (None, None)
                    }
                };

                if let (Some(a1_id), Some(a2_id)) = (pending_a1, pending_a2) {
                    match database.get_pss_match_athletes(db_match_id).await {
                        Ok(existing) => {
                            let mut have1 = existing.iter().any(|(ma, _)| ma.athlete_position == 1);
                            let mut have2 = existing.iter().any(|(ma, _)| ma.athlete_position == 2);

                            if !have1 {
                                let ma = PssMatchAthlete {
                                    id: None,
                                    match_id: db_match_id,
                                    athlete_id: a1_id,
                                    athlete_position: 1,
                                    bg_color: None,
                                    fg_color: None,
                                    created_at: Utc::now(),
                                };
                                if database.insert_pss_match_athlete(&ma).await.is_ok() {
                                    have1 = true;
                                } else {
                                    log::warn!(
                                        "Failed to link pending athlete1 to match {db_match_id}"
                                    );
                                }
                            }

                            if !have2 {
                                let ma = PssMatchAthlete {
                                    id: None,
                                    match_id: db_match_id,
                                    athlete_id: a2_id,
                                    athlete_position: 2,
                                    bg_color: None,
                                    fg_color: None,
                                    created_at: Utc::now(),
                                };
                                if database.insert_pss_match_athlete(&ma).await.is_ok() {
                                    have2 = true;
                                } else {
                                    log::warn!(
                                        "Failed to link pending athlete2 to match {db_match_id}"
                                    );
                                }
                            }

                            if have1 && have2 {
                                log::info!("Linked pending athletes to match {db_match_id}");
                                log::info!("Linked pending athletes to match {db_match_id}");
                                if let Ok(mut cache) = _athlete_cache.lock() {
                                    cache.remove("__PENDING_A1_ID");
                                    cache.remove("__PENDING_A2_ID");
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to fetch match athletes for {db_match_id}: {e}")
                        }
                    }
                }
            }
            PssEvent::Athletes {
                athlete1_short,
                athlete1_long,
                athlete1_country,
                athlete2_short,
                athlete2_long,
                athlete2_country,
            } => {
                // Create or get athletes
                let a1_id = database
                    .get_or_create_pss_athlete(athlete1_short, athlete1_long)
                    .await?;
                let a2_id = database
                    .get_or_create_pss_athlete(athlete2_short, athlete2_long)
                    .await?;

                // Persist extended athlete info (long name, country)
                {
                    let mut a1 = crate::database::models::PssAthlete::new(
                        athlete1_short.clone(),
                        athlete1_short.clone(),
                    );
                    a1.long_name = Some(athlete1_long.clone());
                    a1.country_code = Some(athlete1_country.clone());
                    if let Err(e) = database.update_pss_athlete(a1_id, &a1).await {
                        log::warn!("Failed to update athlete1 info: {e}");
                    }

                    let mut a2 = crate::database::models::PssAthlete::new(
                        athlete2_short.clone(),
                        athlete2_short.clone(),
                    );
                    a2.long_name = Some(athlete2_long.clone());
                    a2.country_code = Some(athlete2_country.clone());
                    if let Err(e) = database.update_pss_athlete(a2_id, &a2).await {
                        log::warn!("Failed to update athlete2 info: {e}");
                    }
                }

                // Cache athlete ids by short code for quick lookup
                if let Ok(mut cache) = _athlete_cache.lock() {
                    cache.insert(athlete1_short.clone(), a1_id);
                    cache.insert(athlete2_short.clone(), a2_id);
                }

                // If we already have a current match, link athletes to match (positions 1 and 2)
                let mid_opt = { *current_match_id.lock().unwrap() };
                if let Some(mid) = mid_opt {
                    match database.get_pss_match_athletes(mid).await {
                        Ok(existing) => {
                            let mut have1 = existing.iter().any(|(ma, _)| ma.athlete_position == 1);
                            let mut have2 = existing.iter().any(|(ma, _)| ma.athlete_position == 2);

                            if !have1 {
                                let ma = PssMatchAthlete {
                                    id: None,
                                    match_id: mid,
                                    athlete_id: a1_id,
                                    athlete_position: 1,
                                    bg_color: None,
                                    fg_color: None,
                                    created_at: Utc::now(),
                                };
                                if database.insert_pss_match_athlete(&ma).await.is_ok() {
                                    have1 = true;
                                } else {
                                    log::warn!("Failed to link athlete1 to match {mid}");
                                }
                            }

                            if !have2 {
                                let ma = PssMatchAthlete {
                                    id: None,
                                    match_id: mid,
                                    athlete_id: a2_id,
                                    athlete_position: 2,
                                    bg_color: None,
                                    fg_color: None,
                                    created_at: Utc::now(),
                                };
                                if database.insert_pss_match_athlete(&ma).await.is_ok() {
                                    have2 = true;
                                } else {
                                    log::warn!("Failed to link athlete2 to match {mid}");
                                }
                            }

                            if have1 && have2 {
                                log::info!("Linked athletes to match {mid}");
                                log::info!("Linked athletes to match {mid}");
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to fetch match athletes for {mid}: {e}")
                        }
                    }
                } else {
                    log::info!(
                        "Athletes received before match configured; will link once match is set"
                    );
                    log::info!(
                        "Athletes received before match configured; will link once match is set"
                    );
                    // Stash pending athlete ids for later linking on MatchConfig
                    if let Ok(mut cache) = _athlete_cache.lock() {
                        cache.insert("__PENDING_A1_ID".to_string(), a1_id);
                        cache.insert("__PENDING_A2_ID".to_string(), a2_id);
                    }
                }
            }
            _ => {}
        }

        // Convert PSS event to database model
        let event_model = Self::convert_pss_event_to_db_model(
            event,
            session_id,
            current_match_id,
            event_type_cache,
            database,
            current_tournament_id,
        )
        .await?;

        // Store event in database only when session and match context are valid
        if event_model.session_id <= 0 {
            log::warn!("Skip storing event: missing session_id");
            return Ok(());
        }
        if event_model.match_id.is_none() {
            // Allow some pre-match/initialization messages to skip persistence safely
            log::warn!("Skip storing event: missing match_id (pre-match context)");
            return Ok(());
        }

        let event_id = database.store_pss_event(&event_model).await?;

        if let Some(details) = Self::extract_event_details(event, recent_hit_levels) {
            if let Err(e) = database.store_pss_event_details(event_id, &details).await {
                log::warn!("Skipping event details for event {event_id} due to error: {e}");
            }
        }

        // Note: WebSocket broadcast happens immediately after parsing to preserve order

        // Update performance metrics
        let processing_time = start_time.elapsed().as_millis() as i32;
        log::debug!("Event processed in {processing_time}ms: {event:?}");

        Ok(())
    }

    // removed unused validation stubs

    /// Get event code for validation
    pub fn get_event_code(event: &PssEvent) -> String {
        match event {
            PssEvent::Points { point_type, .. } => {
                // Map point types to specific event codes according to PSS protocol
                match point_type {
                    1 => "P".to_string(),  // Punch
                    2 => "K".to_string(),  // Body point (Kick) - CHANGED from TB to K
                    3 => "H".to_string(),  // Head point
                    4 => "TB".to_string(), // Technical Body
                    5 => "TH".to_string(), // Technical Head
                    _ => "P".to_string(),  // Default to Punch
                }
            }
            PssEvent::HitLevel { .. } => "HL".to_string(), // Hit Level stored distinctly for review
            PssEvent::Warnings { .. } => "R".to_string(),  // Warning/Gam-jeom
            PssEvent::Challenge { .. } => "R".to_string(), // Challenge/IVR
            PssEvent::Injury { .. } => "O".to_string(),    // Injury time -> Other
            PssEvent::Break { .. } => "O".to_string(),     // Break -> Other
            PssEvent::WinnerRounds { .. } => "O".to_string(), // Winner rounds -> Other
            PssEvent::Winner { .. } => "O".to_string(),    // Winner -> Other
            PssEvent::Athletes { .. } => "O".to_string(),  // Athletes info (pre-match)
            PssEvent::MatchConfig { .. } => "O".to_string(), // Match config (pre-match)
            PssEvent::Scores { .. } => "O".to_string(),    // Scores (system event)
            PssEvent::CurrentScores { .. } => "O".to_string(), // Current scores (system event)
            PssEvent::Clock { .. } => "CLK".to_string(), // Clock (system event) - CHANGED from O to CLK
            PssEvent::Round { .. } => "RND".to_string(), // Round (system event) - CHANGED from O to RND
            PssEvent::FightLoaded => "O".to_string(),    // Fight loaded (pre-match)
            PssEvent::FightReady => "O".to_string(),     // Fight ready (pre-match)
            PssEvent::Supremacy { .. } => "O".to_string(), // Supremacy (system event)
            PssEvent::VideoTime { .. } => "O".to_string(),
            PssEvent::Raw(_raw_msg) => {
                // Raw protocol events are treated as "Other" unless explicitly categorized elsewhere.
                "O".to_string()
            }
        }
    }

    async fn convert_pss_event_to_db_model(
        event: &PssEvent,
        session_id: i64,
        current_match_id: &Arc<Mutex<Option<i64>>>,
        event_type_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
        database: &DatabasePlugin,
        current_tournament_id: &Arc<Mutex<Option<i64>>>,
    ) -> AppResult<DbPssEvent> {
        // Get event type ID
        let event_code = Self::get_event_code(event);
        // Check cache first without holding the lock across await
        let event_type_id = {
            let cached_id = {
                let cache = event_type_cache.lock().unwrap();
                cache.get(&event_code).copied()
            };

            if let Some(id) = cached_id {
                id
            } else {
                // Get or create event type
                let event_type = database.get_pss_event_type_by_code(&event_code).await?;
                let id = if let Some(et) = event_type {
                    et.id.unwrap_or(0)
                } else {
                    // Create new event type if it doesn't exist
                    let new_event_type = crate::database::models::PssEventType::new(
                        event_code.clone(),
                        format!("PSS Event: {event_code}"),
                        "PSS protocol event".to_string(),
                        Some("PSS protocol event".to_string()),
                    );
                    database.upsert_pss_event_type(&new_event_type).await?
                };

                // Update cache after the async operation
                {
                    let mut cache = event_type_cache.lock().unwrap();
                    cache.insert(event_code.clone(), id);
                }
                id
            }
        };

        // Get match ID if available
        let match_id = {
            let match_guard = current_match_id.lock().unwrap();
            *match_guard
        };

        // Get tournament context if available
        let _tournament_id = {
            let tournament_guard = current_tournament_id.lock().unwrap();
            *tournament_guard
        };

        // Create database event model
        let db_event = DbPssEvent::new(
            session_id,
            event_type_id,
            Utc::now(),
            format!("{event:?}"), // Raw data representation
            0,                    // Event sequence will be set by database
        );

        // Set match, round, and tournament IDs
        let mut db_event = db_event;
        db_event.match_id = match_id;
        db_event.round_id = None; // TODO: Track current round
                                  // Defer tournament UUID binding; match context will set it, and events can be updated downstream if needed
        db_event.tournament_id = None;

        // Set parsed data as JSON
        if let Ok(json_data) = serde_json::to_string(event) {
            db_event.parsed_data = Some(json_data);
        }

        Ok(db_event)
    }

    pub fn convert_pss_event_to_json(event: &PssEvent) -> serde_json::Value {
        // Add defensive programming to handle any potential issues
        let event_code = Self::get_event_code(event);

        match event {
            PssEvent::Points {
                athlete,
                point_type,
            } => {
                let athlete_str = match athlete {
                    1 => "blue",
                    2 => "red",
                    _ => "unknown",
                };
                serde_json::json!({
                    "type": "points",
                    "event_code": event_code,
                    "athlete": athlete_str,
                    "point_type": point_type,
                    "description": format!("Athlete {} scored {} points", athlete, point_type),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::HitLevel { athlete, level } => {
                let athlete_str = match athlete {
                    1 => "blue",
                    2 => "red",
                    _ => "unknown",
                };
                serde_json::json!({
                    "type": "hit_level",
                    "event_code": event_code,
                    "athlete": athlete_str,
                    "level": level,
                    "description": format!("Athlete {} hit level {}", athlete, level),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Warnings {
                athlete1_warnings,
                athlete2_warnings,
            } => {
                serde_json::json!({
                    "type": "warnings",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_warnings": athlete1_warnings,
                    "athlete2_warnings": athlete2_warnings,
                    "description": format!("Warnings - Athlete1: {}, Athlete2: {}", athlete1_warnings, athlete2_warnings),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Injury {
                athlete,
                time,
                action,
            } => {
                let athlete_str = match athlete {
                    0 => "unknown",
                    1 => "blue",
                    2 => "red",
                    _ => "unknown",
                };
                serde_json::json!({
                    "type": "injury",
                    "event_code": event_code,
                    "athlete": athlete_str,
                    "time": time,
                    "action": action,
                    "round": 1, // Will be updated by WebSocket plugin
                    "description": format!("Injury - Athlete: {}, Time: {}, Action: {:?}", athlete, time, action),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Challenge {
                source,
                accepted,
                won,
                canceled,
            } => {
                let athlete_str = match source {
                    0 => "yellow",
                    1 => "blue",
                    2 => "red",
                    _ => "unknown",
                };
                serde_json::json!({
                    "type": "challenge",
                    "event_code": event_code,
                    "athlete": athlete_str,
                    "source": source,
                    "accepted": accepted,
                    "won": won,
                    "canceled": canceled,
                    "description": format!("Challenge - Source: {}, Accepted: {:?}, Won: {:?}, Canceled: {}", source, accepted, won, canceled),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Break { time, action } => {
                serde_json::json!({
                    "type": "break",
                    "event_code": event_code,
                    "athlete": "",
                    "time": time,
                    "action": action,
                    "round": 1, // Will be updated by WebSocket plugin
                    "description": format!("Break - Time: {}, Action: {:?}", time, action),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::WinnerRounds {
                round1_winner,
                round2_winner,
                round3_winner,
            } => {
                serde_json::json!({
                    "type": "winner_rounds",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "round1_winner": round1_winner,
                    "round2_winner": round2_winner,
                    "round3_winner": round3_winner,
                    "description": format!("Winner Rounds - R1: {}, R2: {}, R3: {}", round1_winner, round2_winner, round3_winner),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Winner {
                name,
                classification,
            } => {
                serde_json::json!({
                    "type": "winner",
                    "event_code": event_code,
                    "athlete": "",
                    "name": name,
                    "classification": classification,
                    "description": format!("Winner: {} ({:?})", name, classification),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Athletes {
                athlete1_short,
                athlete1_long,
                athlete1_country,
                athlete2_short,
                athlete2_long,
                athlete2_country,
            } => {
                serde_json::json!({
                    "type": "athletes",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1": {
                        "short": athlete1_short,
                        "long": athlete1_long,
                        "country": athlete1_country
                    },
                    "athlete2": {
                        "short": athlete2_short,
                        "long": athlete2_long,
                        "country": athlete2_country
                    },
                    "description": format!("Athletes - {} vs {}", athlete1_short, athlete2_short),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::MatchConfig {
                number,
                category,
                weight,
                rounds,
                colors: _,
                match_id,
                division,
                total_rounds,
                round_duration,
                countdown_type,
                count_up,
                format,
            } => {
                serde_json::json!({
                    "type": "match_config",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "number": number,
                    "category": category,
                    "weight": weight,
                    "rounds": rounds,
                    "match_id": match_id,
                    "division": division,
                    "total_rounds": total_rounds,
                    "round_duration": round_duration,
                    "countdown_type": countdown_type,
                    "count_up": count_up,
                    "format": format,
                    "description": format!("Match Config - #{} {} {} ({} rounds)", number, category, weight, total_rounds),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Scores {
                athlete1_r1,
                athlete2_r1,
                athlete1_r2,
                athlete2_r2,
                athlete1_r3,
                athlete2_r3,
            } => {
                serde_json::json!({
                    "type": "scores",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_r1": athlete1_r1,
                    "athlete2_r1": athlete2_r1,
                    "athlete1_r2": athlete1_r2,
                    "athlete2_r2": athlete2_r2,
                    "athlete1_r3": athlete1_r3,
                    "athlete2_r3": athlete2_r3,
                    "description": format!("Scores - A1: R1={}, R2={}, R3={} | A2: R1={}, R2={}, R3={}",
                        athlete1_r1, athlete1_r2, athlete1_r3, athlete2_r1, athlete2_r2, athlete2_r3),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::CurrentScores {
                athlete1_score,
                athlete2_score,
            } => {
                serde_json::json!({
                    "type": "current_scores",
                    "event_code": event_code,
                    "athlete": "yellow",
                    "athlete1_score": athlete1_score,
                    "athlete2_score": athlete2_score,
                    "description": format!("Current Scores - A1: {}, A2: {}", athlete1_score, athlete2_score),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Clock { time, action } => {
                // Defensive programming for clock events
                let safe_time = time.as_str();
                let safe_action = action.as_ref().map(|a| a.as_str()).unwrap_or("");
                let description = format!("Clock: {safe_time} {safe_action:?}");

                serde_json::json!({
                    "type": "clock",
                    "event_code": event_code,
                    "athlete": "",
                    "time": safe_time,
                    "action": safe_action,
                    "round": 1, // Will be updated by WebSocket plugin
                    "description": description,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Round { current_round } => {
                serde_json::json!({
                    "type": "round",
                    "event_code": event_code,
                    "athlete": "",
                    "current_round": current_round,
                    "round": current_round,
                    "description": format!("Round {}", current_round),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::FightLoaded => {
                serde_json::json!({
                    "type": "fight_loaded",
                    "event_code": event_code,
                    "athlete": "",
                    "description": "Fight loaded",
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::FightReady => {
                serde_json::json!({
                    "type": "fight_ready",
                    "event_code": event_code,
                    "athlete": "",
                    "description": "Fight ready",
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Supremacy { value } => {
                serde_json::json!({
                    "type": "supremacy",
                    "event_code": event_code,
                    "athlete": "",
                    "value": value,
                    "description": format!("Supremacy - Value: {}", value),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::VideoTime { value } => {
                serde_json::json!({
                    "type": "video_time",
                    "event_code": event_code,
                    "athlete": "",
                    "value": value,
                    "description": format!("Video time marker: {}", value),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Raw(message) => {
                // Defensive programming for raw messages
                let safe_message = message.as_str();
                let description = format!("Raw message: {safe_message}");

                serde_json::json!({
                    "type": "raw",
                    "event_code": event_code,
                    "athlete": "",
                    "message": safe_message,
                    "description": description,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
        }
    }

    fn extract_event_details(
        event: &PssEvent,
        _recent_hit_levels: &RecentHitMap,
    ) -> Option<Vec<(String, Option<String>, String)>> {
        match event {
            PssEvent::Points {
                athlete,
                point_type,
            } => {
                let details = vec![
                    (
                        "athlete".to_string(),
                        Some(athlete.to_string()),
                        "u8".to_string(),
                    ),
                    (
                        "point_type".to_string(),
                        Some(point_type.to_string()),
                        "u8".to_string(),
                    ),
                ];
                Some(details)
            }
            PssEvent::HitLevel { athlete, level } => Some(vec![
                (
                    "athlete".to_string(),
                    Some(athlete.to_string()),
                    "u8".to_string(),
                ),
                (
                    "level".to_string(),
                    Some(level.to_string()),
                    "u8".to_string(),
                ),
            ]),
            PssEvent::Warnings {
                athlete1_warnings,
                athlete2_warnings,
            } => Some(vec![
                (
                    "athlete1_warnings".to_string(),
                    Some(athlete1_warnings.to_string()),
                    "u8".to_string(),
                ),
                (
                    "athlete2_warnings".to_string(),
                    Some(athlete2_warnings.to_string()),
                    "u8".to_string(),
                ),
            ]),
            PssEvent::Injury {
                athlete,
                time,
                action,
            } => Some(vec![
                (
                    "athlete".to_string(),
                    Some(athlete.to_string()),
                    "u8".to_string(),
                ),
                ("time".to_string(), Some(time.clone()), "String".to_string()),
                (
                    "action".to_string(),
                    action.clone(),
                    "Option<String>".to_string(),
                ),
            ]),
            PssEvent::Challenge {
                source,
                accepted,
                won,
                canceled,
            } => Some(vec![
                (
                    "source".to_string(),
                    Some(source.to_string()),
                    "u8".to_string(),
                ),
                (
                    "accepted".to_string(),
                    accepted.map(|a| a.to_string()),
                    "Option<bool>".to_string(),
                ),
                (
                    "won".to_string(),
                    won.map(|w| w.to_string()),
                    "Option<bool>".to_string(),
                ),
                (
                    "canceled".to_string(),
                    Some(canceled.to_string()),
                    "bool".to_string(),
                ),
            ]),
            PssEvent::Break { time, action } => Some(vec![
                ("time".to_string(), Some(time.clone()), "String".to_string()),
                (
                    "action".to_string(),
                    action.clone(),
                    "Option<String>".to_string(),
                ),
            ]),
            PssEvent::WinnerRounds {
                round1_winner,
                round2_winner,
                round3_winner,
            } => Some(vec![
                (
                    "round1_winner".to_string(),
                    Some(round1_winner.to_string()),
                    "u8".to_string(),
                ),
                (
                    "round2_winner".to_string(),
                    Some(round2_winner.to_string()),
                    "u8".to_string(),
                ),
                (
                    "round3_winner".to_string(),
                    Some(round3_winner.to_string()),
                    "u8".to_string(),
                ),
            ]),
            PssEvent::Winner {
                name,
                classification,
            } => Some(vec![
                ("name".to_string(), Some(name.clone()), "String".to_string()),
                (
                    "classification".to_string(),
                    classification.clone(),
                    "Option<String>".to_string(),
                ),
            ]),
            PssEvent::Athletes {
                athlete1_short,
                athlete1_long,
                athlete1_country,
                athlete2_short,
                athlete2_long,
                athlete2_country,
            } => Some(vec![
                (
                    "athlete1_short".to_string(),
                    Some(athlete1_short.clone()),
                    "String".to_string(),
                ),
                (
                    "athlete1_long".to_string(),
                    Some(athlete1_long.clone()),
                    "String".to_string(),
                ),
                (
                    "athlete1_country".to_string(),
                    Some(athlete1_country.clone()),
                    "String".to_string(),
                ),
                (
                    "athlete2_short".to_string(),
                    Some(athlete2_short.clone()),
                    "String".to_string(),
                ),
                (
                    "athlete2_long".to_string(),
                    Some(athlete2_long.clone()),
                    "String".to_string(),
                ),
                (
                    "athlete2_country".to_string(),
                    Some(athlete2_country.clone()),
                    "String".to_string(),
                ),
            ]),
            PssEvent::MatchConfig {
                number,
                category,
                weight,
                rounds,
                colors: _,
                match_id,
                division,
                total_rounds,
                round_duration,
                countdown_type,
                count_up,
                format,
            } => Some(vec![
                (
                    "number".to_string(),
                    Some(number.to_string()),
                    "u32".to_string(),
                ),
                (
                    "category".to_string(),
                    Some(category.clone()),
                    "String".to_string(),
                ),
                (
                    "weight".to_string(),
                    Some(weight.clone()),
                    "String".to_string(),
                ),
                (
                    "rounds".to_string(),
                    Some(rounds.to_string()),
                    "u8".to_string(),
                ),
                (
                    "match_id".to_string(),
                    Some(match_id.clone()),
                    "String".to_string(),
                ),
                (
                    "division".to_string(),
                    Some(division.clone()),
                    "String".to_string(),
                ),
                (
                    "total_rounds".to_string(),
                    Some(total_rounds.to_string()),
                    "u8".to_string(),
                ),
                (
                    "round_duration".to_string(),
                    Some(round_duration.to_string()),
                    "u32".to_string(),
                ),
                (
                    "countdown_type".to_string(),
                    Some(countdown_type.clone()),
                    "String".to_string(),
                ),
                (
                    "count_up".to_string(),
                    Some(count_up.to_string()),
                    "u32".to_string(),
                ),
                (
                    "format".to_string(),
                    Some(format.to_string()),
                    "u8".to_string(),
                ),
            ]),
            PssEvent::Scores {
                athlete1_r1,
                athlete2_r1,
                athlete1_r2,
                athlete2_r2,
                athlete1_r3,
                athlete2_r3,
            } => Some(vec![
                (
                    "athlete1_r1".to_string(),
                    Some(athlete1_r1.to_string()),
                    "u8".to_string(),
                ),
                (
                    "athlete2_r1".to_string(),
                    Some(athlete2_r1.to_string()),
                    "u8".to_string(),
                ),
                (
                    "athlete1_r2".to_string(),
                    Some(athlete1_r2.to_string()),
                    "u8".to_string(),
                ),
                (
                    "athlete2_r2".to_string(),
                    Some(athlete2_r2.to_string()),
                    "u8".to_string(),
                ),
                (
                    "athlete1_r3".to_string(),
                    Some(athlete1_r3.to_string()),
                    "u8".to_string(),
                ),
                (
                    "athlete2_r3".to_string(),
                    Some(athlete2_r3.to_string()),
                    "u8".to_string(),
                ),
            ]),
            PssEvent::CurrentScores {
                athlete1_score,
                athlete2_score,
            } => Some(vec![
                (
                    "athlete1_score".to_string(),
                    Some(athlete1_score.to_string()),
                    "u8".to_string(),
                ),
                (
                    "athlete2_score".to_string(),
                    Some(athlete2_score.to_string()),
                    "u8".to_string(),
                ),
            ]),
            PssEvent::Clock { time, action } => Some(vec![
                ("time".to_string(), Some(time.clone()), "String".to_string()),
                (
                    "action".to_string(),
                    action.clone(),
                    "Option<String>".to_string(),
                ),
            ]),
            PssEvent::Round { current_round } => Some(vec![(
                "current_round".to_string(),
                Some(current_round.to_string()),
                "u8".to_string(),
            )]),
            PssEvent::Supremacy { value } => Some(vec![(
                "value".to_string(),
                Some(value.to_string()),
                "u8".to_string(),
            )]),
            PssEvent::VideoTime { value } => Some(vec![(
                "value".to_string(),
                Some(value.to_string()),
                "u8".to_string(),
            )]),
            PssEvent::Raw(message) => Some(vec![(
                "message".to_string(),
                Some(message.clone()),
                "String".to_string(),
            )]),
            _ => None,
        }
    }

    // removed unused event handlers

    #[allow(clippy::too_many_arguments)]
    async fn listen_loop_async(
        socket: Arc<Mutex<Option<UdpSocket>>>,
        event_tx: mpsc::UnboundedSender<PssEvent>,
        status: Arc<Mutex<UdpServerStatus>>,
        stats: Arc<Mutex<UdpStats>>,
        _protocol_manager: Arc<ProtocolManager>,
        protocol_parser: Arc<PssProtocol>,
        recent_events: Arc<Mutex<VecDeque<PssEvent>>>,
        database: Arc<DatabasePlugin>,
        current_session_id: Arc<Mutex<Option<i64>>>,
        current_match_id: Arc<Mutex<Option<i64>>>,
        athlete_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
        event_type_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
        recent_hit_levels: RecentHitMap,
        tournament_id: Arc<Mutex<Option<i64>>>,
        websocket_server: Arc<WebSocketServer>,
    ) {
        log::info!("UDP PSS Server listening loop started (async)");

        let mut buffer = [0u8; 8192];

        loop {
            // Check if we should stop
            {
                let status_guard = status.lock().unwrap();
                if matches!(*status_guard, UdpServerStatus::Stopped) {
                    break;
                }
            }

            // Get socket reference and try to receive data
            let recv_result = {
                let socket_guard = socket.lock().unwrap();
                match &*socket_guard {
                    Some(s) => s.recv_from(&mut buffer),
                    None => {
                        log::error!("UDP socket is None, stopping listen loop");
                        break;
                    }
                }
            };

            match recv_result {
                Ok((len, src_addr)) => {
                    // Update stats
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.packets_received += 1;
                        stats_guard.last_packet_time = Some(std::time::SystemTime::now());
                        stats_guard.total_bytes_received += len as u64;

                        // Update average packet size
                        let total_packets = stats_guard.packets_received;
                        let total_bytes = stats_guard.total_bytes_received;
                        stats_guard.average_packet_size = total_bytes as f64 / total_packets as f64;

                        // Track active connections
                        stats_guard
                            .active_connections
                            .insert(src_addr, std::time::SystemTime::now());
                        stats_guard.connected_clients = stats_guard.active_connections.len();
                    }

                    // Convert received data to string
                    let message = match String::from_utf8_lossy(&buffer[..len]).to_string() {
                        msg if msg.trim().is_empty() => continue,
                        msg => msg,
                    };

                    log::debug!("Received PSS message from {src_addr}: {message}");

                    let is_connection_event = message.contains("Udp Port")
                        && (message.contains("connected") || message.contains("disconnected"));
                    if is_connection_event && message.contains("disconnected") {
                        if let Ok(mut stats_guard) = stats.lock() {
                            stats_guard.active_connections.remove(&src_addr);
                            stats_guard.connected_clients = stats_guard.active_connections.len();
                        }
                        {
                            if let Ok(mut match_guard) = current_match_id.lock() {
                                *match_guard = None;
                            }
                        }
                        if let Err(e) = websocket_server.reset_match_state() {
                            log::debug!("Failed to reset match state after disconnect: {e}");
                        }
                        log::info!("PSS connection {src_addr} reported disconnect");
                    }

                    // Log raw UDP message for Live Data panel
                    let raw_log_message = format!(" Raw UDP message: {message}");
                    crate::core::app::App::emit_log_event(raw_log_message);

                    // Parse the message with panic protection
                    let parser = protocol_parser.clone();
                    let message_for_parse = message.clone();
                    let parse_result =
                        std::panic::catch_unwind(move || parser.parse_message(&message_for_parse));

                    match parse_result {
                        Ok(parse_result) => {
                            match parse_result {
                                Ok(event) => {
                                    // Update stats
                                    {
                                        let mut stats_guard = stats.lock().unwrap();
                                        stats_guard.packets_parsed += 1;
                                    }

                                    // Track hit level events for statistics
                                    match &event {
                                        PssEvent::HitLevel { .. } => { /* no aggregation needed */ }
                                        PssEvent::FightLoaded | PssEvent::FightReady => { /* nothing to clear */
                                        }
                                        _ => {}
                                    }

                                    // Broadcast to WebSocket clients immediately to preserve event order
                                    if let Err(e) = websocket_server.broadcast_event(&event) {
                                        log::warn!("Failed to broadcast event to WebSocket: {e}");
                                    }

                                    // Store event in database asynchronously (do not block IO path)
                                    let event_clone = event.clone();
                                    let database_clone = database.clone();
                                    let current_session_id_clone = current_session_id.clone();
                                    let current_match_id_clone = current_match_id.clone();
                                    let athlete_cache_clone = athlete_cache.clone();
                                    let event_type_cache_clone = event_type_cache.clone();
                                    let recent_hit_levels_clone = recent_hit_levels.clone();
                                    let tournament_id_clone = tournament_id.clone();
                                    let websocket_server_clone = websocket_server.clone();
                                    tokio::spawn(async move {
                                        if let Err(e) = Self::store_event_in_database(
                                            &database_clone,
                                            &current_session_id_clone,
                                            &current_match_id_clone,
                                            &athlete_cache_clone,
                                            &event_type_cache_clone,
                                            &event_clone,
                                            &recent_hit_levels_clone,
                                            &tournament_id_clone,
                                            &websocket_server_clone,
                                        )
                                        .await
                                        {
                                            log::error!("Failed to store event in database: {e}");
                                        }
                                    });

                                    // Add event to recent events storage
                                    {
                                        let mut events_guard = match recent_events.lock() {
                                            Ok(g) => g,
                                            Err(poisoned) => {
                                                log::warn!(
                                                    "recent_events mutex poisoned; recovering"
                                                );
                                                poisoned.into_inner()
                                            }
                                        };
                                        events_guard.push_back(event.clone());

                                        // Keep only the last 100 events
                                        if events_guard.len() > 100 {
                                            events_guard.pop_front();
                                        }
                                    }

                                    // Send event to frontend via Tauri events
                                    let event_json = Self::convert_pss_event_to_json(&event);

                                    // Log the parsed event and JSON for debugging
                                    log::info!("Parsed PSS event: {event:?}");

                                    // Safely serialize JSON with error handling
                                    match serde_json::to_string(&event_json) {
                                        Ok(json_string) => {
                                            log::info!("Emitting event JSON: {json_string}");

                                            // Emit to Tauri frontend
                                            if let Err(e) = event_tx.send(event.clone()) {
                                                log::warn!("Failed to send PSS event to internal channel: {e}");
                                            }

                                            // Emit to frontend (Tauri) only to avoid double WebSocket broadcast
                                            // WebSocket broadcasting is already handled directly below via websocket_server.broadcast_event(event)
                                            crate::core::app::App::emit_custom_event(
                                                "pss_event",
                                                event_json,
                                            );

                                            // Stream log to frontend for Live Data panel
                                            let log_message = format!(" UDP-EVENT: {event:?}");
                                            crate::core::app::App::emit_log_event(log_message);
                                        }
                                        Err(e) => {
                                            log::error!(
                                                "Failed to serialize PSS event to JSON: {e}"
                                            );
                                            log::error!("Event that failed: {event:?}");

                                            // Still try to send the event to internal channel
                                            if let Err(e) = event_tx.send(event.clone()) {
                                                log::warn!("Failed to send PSS event to internal channel: {e}");
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    // Update error stats
                                    {
                                        let mut stats_guard = stats.lock().unwrap();
                                        stats_guard.parse_errors += 1;
                                    }

                                    log::warn!("Failed to parse PSS message '{message}': {e}");

                                    // Create raw event and add to storage
                                    let raw_event = PssEvent::Raw(message.clone());

                                    // Add raw event to recent events storage
                                    {
                                        let mut events_guard = recent_events.lock().unwrap();
                                        events_guard.push_back(raw_event.clone());

                                        // Keep only the last 100 events
                                        if events_guard.len() > 100 {
                                            events_guard.pop_front();
                                        }
                                    }

                                    // Send raw message as fallback (ignore errors if no receiver)
                                    if event_tx.send(raw_event).is_err() {
                                        // Don't break the loop, just continue
                                    }
                                }
                            }
                        }
                        Err(panic_info) => {
                            // Handle panic in parsing
                            log::error!(
                                "Panic occurred while parsing message '{message}': {panic_info:?}"
                            );

                            // Update error stats
                            {
                                let mut stats_guard = stats.lock().unwrap();
                                stats_guard.parse_errors += 1;
                            }

                            // Create raw event and add to storage
                            let raw_event = PssEvent::Raw(message.clone());

                            // Add raw event to recent events storage
                            {
                                let mut events_guard = recent_events.lock().unwrap();
                                events_guard.push_back(raw_event.clone());

                                // Keep only the last 100 events
                                if events_guard.len() > 100 {
                                    events_guard.pop_front();
                                }
                            }

                            // Send raw message as fallback (ignore errors if no receiver)
                            if event_tx.send(raw_event).is_err() {
                                // Don't break the loop, just continue
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        let error_msg = format!("UDP receive error: {e}");
                        log::error!("{error_msg}");

                        let mut status_guard = status.lock().unwrap();
                        *status_guard = UdpServerStatus::Error(error_msg);
                        break;
                    }
                }
            }

            // Small sleep to make the loop responsive to stop requests
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        log::info!("UDP PSS Server listening loop ended");
    }

    #[test]
    fn test_parse_points() {
        let parser = PssProtocol::new();
        let event = parser.parse_message("pt1;3;").unwrap();
        match event {
            PssEvent::Points {
                athlete,
                point_type,
            } => {
                assert_eq!(athlete, 1);
                assert_eq!(point_type, 3);
            }
            _ => panic!("Expected Points event"),
        }
    }

    #[test]
    fn test_parse_warnings() {
        let parser = PssProtocol::new();
        let event = parser.parse_message("wg1;1;wg2;2;").unwrap();
        match event {
            PssEvent::Warnings {
                athlete1_warnings,
                athlete2_warnings,
            } => {
                assert_eq!(athlete1_warnings, 1);
                assert_eq!(athlete2_warnings, 2);
            }
            _ => panic!("Expected Warnings event"),
        }
    }

    #[test]
    fn test_parse_clock() {
        let parser = PssProtocol::new();
        let event = parser.parse_message("clk;1:23;start;").unwrap();
        match event {
            PssEvent::Clock { time, action } => {
                assert_eq!(time, "1:23");
                assert_eq!(action, Some("start".to_string()));
            }
            _ => panic!("Expected Clock event"),
        }
    }
}
