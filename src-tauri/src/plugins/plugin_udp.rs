use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};
use crate::plugins::ProtocolManager;
use crate::plugins::plugin_database::DatabasePlugin;
use crate::plugins::performance_monitor::PerformanceMonitor;
use crate::plugins::plugin_websocket::WebSocketServer;
use crate::database::models::{
    UdpServerConfig as DbUdpServerConfig, PssEventV2, 
    PssScore, PssWarning
};
use chrono::Utc;
use std::time::{Duration, Instant};

/// Validation result enum
#[derive(Debug)]
enum ValidationResult {
    Valid,
    Partial(Vec<String>),
    Invalid(Vec<String>),
}

/// Initialize the UDP plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Initializing UDP plugin...");
    Ok(())
}

// Re-export the main plugin type
pub type UdpPlugin = UdpServer;

// PSS Event Types based on protocol specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PssEvent {
    // Points events
    Points {
        athlete: u8,      // 1 or 2
        point_type: u8,   // 1=punch, 2=body, 3=head, 4=tech_body, 5=tech_head
    },
    
    // Hit level events
    HitLevel {
        athlete: u8,      // 1 or 2
        level: u8,        // 1-100
    },
    
    // Warnings/Gam-jeom events
    Warnings {
        athlete1_warnings: u8,
        athlete2_warnings: u8,
    },
    
    // Injury time events
    Injury {
        athlete: u8,      // 0=unidentified, 1=athlete1, 2=athlete2
        time: String,     // format: "m:ss"
        action: Option<String>, // show, hide, reset
    },
    
    // Challenge/IVR events
    Challenge {
        source: u8,       // 0=referee, 1=athlete1, 2=athlete2
        accepted: Option<bool>,
        won: Option<bool>,
        canceled: bool,
    },
    
    // Break events
    Break {
        time: String,     // format: "m:ss" or just seconds
        action: Option<String>, // stop, stopEnd
    },
    
    // Winner rounds events
    WinnerRounds {
        round1_winner: u8, // 0=none, 1=athlete1, 2=athlete2
        round2_winner: u8,
        round3_winner: u8,
    },
    
    // Final winner events
    Winner {
        name: String,
        classification: Option<String>,
    },
    
    // Match info events
    Athletes {
        athlete1_short: String,
        athlete1_long: String,
        athlete1_country: String,
        athlete2_short: String,
        athlete2_long: String,
        athlete2_country: String,
    },
    
    // Match configuration
    MatchConfig {
        number: u32,
        category: String,
        weight: String,
        rounds: u8,
        colors: (String, String, String, String), // bg1, fg1, bg2, fg2
        match_id: String,
        division: String,
        total_rounds: u8,
        round_duration: u32, // seconds
        countdown_type: String,
        count_up: u32,
        format: u8,
    },
    
    // Scores
    Scores {
        athlete1_r1: u8,
        athlete2_r1: u8,
        athlete1_r2: u8,
        athlete2_r2: u8,
        athlete1_r3: u8,
        athlete2_r3: u8,
    },
    
    // Current scores
    CurrentScores {
        athlete1_score: u8,
        athlete2_score: u8,
    },
    
    // Clock events
    Clock {
        time: String,     // format: "m:ss"
        action: Option<String>, // start, stop
    },
    
    // Round events
    Round {
        current_round: u8,
    },
    
    // System events
    FightLoaded,
    FightReady,
    
    // Raw message for unrecognized patterns
    Raw(String),
}

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
    recent_events: Arc<Mutex<VecDeque<PssEvent>>>,
    database: Arc<DatabasePlugin>,
    current_session_id: Arc<Mutex<Option<i64>>>,
    current_match_id: Arc<Mutex<Option<i64>>>,
    athlete_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
    event_type_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
    listener_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    // Hit level tracking for statistics
    recent_hit_levels: Arc<Mutex<std::collections::HashMap<u8, Vec<(u8, std::time::SystemTime)>>>>, // athlete -> [(level, timestamp)]
    // Tournament context tracking
    current_tournament_id: Arc<Mutex<Option<i64>>>,
    current_tournament_day_id: Arc<Mutex<Option<i64>>>,
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
            recent_events: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            database,
            current_session_id: Arc::new(Mutex::new(None)),
            current_match_id: Arc::new(Mutex::new(None)),
            athlete_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            event_type_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            listener_task: Arc::new(Mutex::new(None)),
            recent_hit_levels: Arc::new(Mutex::new(std::collections::HashMap::new())),
            current_tournament_id: Arc::new(Mutex::new(None)),
            current_tournament_day_id: Arc::new(Mutex::new(None)),
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
        *server.batch_processor_task.lock().unwrap() = Some(batch_task);
        
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
            recent_events: self.recent_events.clone(),
            database: self.database.clone(),
            current_session_id: self.current_session_id.clone(),
            current_match_id: self.current_match_id.clone(),
            athlete_cache: self.athlete_cache.clone(),
            event_type_cache: self.event_type_cache.clone(),
            listener_task: self.listener_task.clone(),
            recent_hit_levels: self.recent_hit_levels.clone(),
            current_tournament_id: self.current_tournament_id.clone(),
            current_tournament_day_id: self.current_tournament_day_id.clone(),
            event_batch: self.event_batch.clone(),
            batch_processor_task: self.batch_processor_task.clone(),
            batch_tx: mpsc::unbounded_channel().0, // Dummy channel for clone
            performance_monitor: self.performance_monitor.clone(),
            websocket_server: self.websocket_server.clone(),
        }
    }

    /// Phase 1 Optimization: Batch processor loop for high-volume event processing
    async fn batch_processor_loop(
        mut batch_rx: mpsc::UnboundedReceiver<PssEvent>,
        server: Self,
    ) {
        const BATCH_SIZE: usize = 100; // Process 100 events per batch
        const BATCH_TIMEOUT: Duration = Duration::from_millis(500); // 500ms timeout
        
        let mut batch = Vec::with_capacity(BATCH_SIZE);
        let mut last_batch_time = Instant::now();
        
        log::info!("🚀 Starting batch processor for high-volume event processing");
        
        while let Some(event) = batch_rx.recv().await {
            batch.push(event);
            
            let should_process = batch.len() >= BATCH_SIZE || 
                               last_batch_time.elapsed() >= BATCH_TIMEOUT;
            
            if should_process {
                let events_to_process = std::mem::replace(&mut batch, Vec::with_capacity(BATCH_SIZE));
                last_batch_time = Instant::now();
                
                // Process batch asynchronously
                let server_clone = server.clone_for_batch_processor();
                tokio::spawn(async move {
                    if let Err(e) = Self::process_event_batch(server_clone, events_to_process).await {
                        log::error!("❌ Batch processing failed: {}", e);
                    }
                });
            }
        }
        
        // Process remaining events
        if !batch.is_empty() {
            if let Err(e) = Self::process_event_batch(server, batch).await {
                log::error!("❌ Final batch processing failed: {}", e);
            }
        }
        
        log::info!("🛑 Batch processor stopped");
    }

    /// Phase 1 Optimization: Process a batch of events efficiently
    async fn process_event_batch(server: Self, events: Vec<PssEvent>) -> AppResult<()> {
        let start_time = Instant::now();
        let batch_size = events.len();
        
        log::debug!("📦 Processing batch of {} events", batch_size);
        
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
        let current_tournament_day_id = server.current_tournament_day_id.clone();
        
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
            let tournament_day_clone = current_tournament_day_id.clone();
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
                    &tournament_day_clone,
                    &websocket_server_clone,
                ).await
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
                    log::warn!("⚠️ Event storage failed in batch: {}", e);
                }
                Err(e) => {
                    error_count += 1;
                    log::warn!("⚠️ Task failed in batch: {}", e);
                }
            }
        }
        
        let processing_time = start_time.elapsed();
        
        // Phase 1 Optimization: Record batch processing metrics
        server.performance_monitor.record_event_processed(processing_time.as_millis() as u64);
        
        log::info!("✅ Batch processed: {}/{} events in {:?} ({} errors)", 
                  success_count, batch_size, processing_time, error_count);
        
        // Update statistics
        if let Ok(mut stats) = server.stats.lock() {
            stats.packets_parsed += success_count as u64;
            stats.parse_errors += error_count as u64;
        }
        
        Ok(())
    }

    pub async fn start(&self, config: &crate::config::types::AppConfig) -> AppResult<()> {
        log::info!("🚀 Starting UDP server...");
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Starting;
        }
        
        // Start WebSocket server for real-time event broadcasting
        if let Err(e) = self.websocket_server.start(8080).await {
            log::warn!("Failed to start WebSocket server: {}", e);
        } else {
            log::info!("🔌 WebSocket server started on port 8080");
        }
        
        // Initialize event type cache
        if let Err(e) = self.initialize_event_type_cache().await {
            log::error!("Failed to initialize event type cache: {}", e);
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
                    println!("🎯 Auto-detected network interface IP: {}", ip);
                    ip.to_string()
                }
                Err(e) => {
                    println!("⚠️ Failed to auto-detect network interface: {}", e);
                    println!("🔄 Falling back to configured bind address: {}", bind_address);
                    bind_address.clone()
                }
            }
        } else {
            println!("🎯 Using configured bind address: {}", bind_address);
            bind_address.clone()
        };
        
        let bind_addr = format!("{}:{}", bind_ip, port);
        println!("🚀 Attempting to bind UDP server to: {}", bind_addr);
        
        // Update status to starting
        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Starting;
        }

        // Try to bind the socket
        let socket = match UdpSocket::bind(&bind_addr) {
            Ok(socket) => {
                socket.set_nonblocking(true).map_err(|e| AppError::ConfigError(e.to_string()))?;
                socket
            }
            Err(e) => {
                let error_msg = format!("Failed to bind UDP socket to {}: {}", bind_addr, e);
                let mut status = self.status.lock().unwrap();
                *status = UdpServerStatus::Error(error_msg.clone());
                return Err(AppError::ConfigError(error_msg));
            }
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
        let recent_events_clone = self.recent_events.clone();
        let database_clone = self.database.clone();
        let current_session_id_clone = self.current_session_id.clone();
        let current_match_id_clone = self.current_match_id.clone();
        let athlete_cache_clone = self.athlete_cache.clone();
        let event_type_cache_clone = self.event_type_cache.clone();
        let recent_hit_levels_clone = self.recent_hit_levels.clone();
        let tournament_id_clone = self.current_tournament_id.clone();
        let tournament_day_id_clone = self.current_tournament_day_id.clone();
        let websocket_server_clone = self.websocket_server.clone();

        let listener_task = tokio::spawn(async move {
            Self::listen_loop_async(
                socket_clone, 
                event_tx, 
                status_clone, 
                stats_clone, 
                protocol_manager, 
                recent_events_clone,
                database_clone,
                current_session_id_clone,
                current_match_id_clone,
                athlete_cache_clone,
                event_type_cache_clone,
                recent_hit_levels_clone,
                tournament_id_clone,
                tournament_day_id_clone,
                websocket_server_clone,
            ).await;
        });

        {
            let mut listener_task_guard = self.listener_task.lock().unwrap();
            *listener_task_guard = Some(listener_task);
        }

        log::info!("🚀 UDP server started on {}", bind_addr);
        
        // Log server start for Live Data panel
        let start_log_message = format!("🚀 UDP server started on {}", bind_addr);
        crate::core::app::App::emit_log_event(start_log_message);
        
        Ok(())
    }

    pub async fn stop(&self) -> AppResult<()> {
        log::info!("🛑 Stopping UDP server...");
        
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = UdpServerStatus::Stopped;
        }
        
        // Stop WebSocket server
        if let Err(e) = self.websocket_server.stop().await {
            log::warn!("Failed to stop WebSocket server: {}", e);
        } else {
            log::info!("🔌 WebSocket server stopped");
        }
        
        // Stop listener task
        if let Some(task) = self.listener_task.lock().unwrap().take() {
            task.abort();
        }
        
        // Stop batch processor task
        if let Some(task) = self.batch_processor_task.lock().unwrap().take() {
            task.abort();
        }
        
        // Close socket
        {
            let mut socket_guard = self.socket.lock().unwrap();
            *socket_guard = None;
        }
        
        log::info!("✅ UDP server stopped successfully");
        Ok(())
    }

    pub fn get_status(&self) -> UdpServerStatus {
        let status = self.status.lock().unwrap();
        status.clone()
    }

    pub fn get_stats(&self) -> UdpStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    pub fn get_recent_events(&self) -> Vec<PssEvent> {
        let events = self.recent_events.lock().unwrap();
        events.iter().cloned().collect()
    }

    /// Phase 1 Optimization: Get performance metrics
    pub fn get_performance_metrics(&self) -> crate::plugins::performance_monitor::PerformanceMetrics {
        self.performance_monitor.get_performance_metrics()
    }

    /// Phase 1 Optimization: Get memory usage
    pub fn get_memory_usage(&self) -> crate::plugins::performance_monitor::MemoryUsageStats {
        self.performance_monitor.get_memory_stats()
    }

    pub fn add_event(&self, event: PssEvent) {
        // Add to recent events (existing logic)
        {
            let mut recent = self.recent_events.lock().unwrap();
            if recent.len() >= 100 {
                recent.pop_front();
            }
            recent.push_back(event.clone());
        }

        // Phase 1 Optimization: Send to batch processor for high-volume processing
        if let Err(e) = self.batch_tx.send(event.clone()) {
            log::error!("❌ Failed to send event to batch processor: {}", e);
        }

        // Send to event channel (existing logic)
        if let Err(e) = self.event_tx.send(event) {
            log::error!("Failed to send event: {}", e);
        }
    }

    pub async fn update_config(&self, port: u16, bind_address: String) {
        log::info!("Updating UDP server configuration to port: {} and bind address: {}", port, bind_address);
        // Note: The actual configuration update will be handled in the start() method
        // This method is called to log the configuration change
    }

    async fn initialize_event_type_cache(&self) -> AppResult<()> {
        match self.database.get_pss_event_types().await {
            Ok(event_types) => {
                let mut cache = self.event_type_cache.lock().unwrap();
                
                for event_type in event_types {
                    if let Some(id) = event_type.id {
                        cache.insert(event_type.event_code.clone(), id);
                    }
                }
                
                log::info!("✅ Event type cache initialized with {} types", cache.len());
                Ok(())
            }
            Err(e) => {
                log::warn!("⚠️ Failed to initialize event type cache: {}. Continuing without cache.", e);
                // Don't fail the entire UDP server startup if event type cache fails
                Ok(())
            }
        }
    }

    async fn store_event_in_database(
        database: &DatabasePlugin,
        current_session_id: &Arc<Mutex<Option<i64>>>,
        current_match_id: &Arc<Mutex<Option<i64>>>,
        athlete_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
        event_type_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
        event: &PssEvent,
        recent_hit_levels: &Arc<Mutex<std::collections::HashMap<u8, Vec<(u8, std::time::SystemTime)>>>>,
        current_tournament_id: &Arc<Mutex<Option<i64>>>,
        current_tournament_day_id: &Arc<Mutex<Option<i64>>>,
        websocket_server: &Arc<WebSocketServer>,
    ) -> AppResult<()> {
        let start_time = Instant::now();
        
        // Get session ID
        let session_id = {
            let session_guard = current_session_id.lock().unwrap();
            session_guard.ok_or_else(|| AppError::ConfigError("No active session".to_string()))?
        };
        
        // Convert PSS event to database model
        let event_model = Self::convert_pss_event_to_db_model(
            event,
            session_id,
            current_match_id,
            event_type_cache,
            database,
            current_tournament_id,
            current_tournament_day_id,
        ).await?;
        
        // Store event in database
        let event_id = database.store_pss_event(&event_model).await?;
        
        // Extract and store event details
        if let Some(details) = Self::extract_event_details(event, recent_hit_levels) {
            database.store_pss_event_details(event_id, &details).await?;
        }
        
        // Broadcast event to WebSocket clients for real-time updates
        if let Err(e) = websocket_server.broadcast_event(event) {
            log::warn!("Failed to broadcast event to WebSocket: {}", e);
        }
        
        // Update performance metrics
        let processing_time = start_time.elapsed().as_millis() as i32;
        log::debug!("Event processed in {}ms: {:?}", processing_time, event);
        
        Ok(())
    }

    /// Determine event recognition status and confidence
    async fn determine_event_status(event: &PssEvent, database: &DatabasePlugin) -> AppResult<(String, f64, Option<String>)> {
        let event_code = Self::get_event_code(event);
        
        // Check if this is a recognized event type
        let event_type = database.get_pss_event_type_by_code(&event_code).await?;
        
        if event_type.is_none() {
            // Unknown event type
            return Ok(("unknown".to_string(), 0.0, Some("Event type not recognized".to_string())));
        }
        
        // Validate event against protocol rules
        let validation_result = Self::validate_event_against_protocol(event, &event_code, database).await?;
        
        match validation_result {
            ValidationResult::Valid => {
                Ok(("recognized".to_string(), 1.0, None))
            }
            ValidationResult::Partial(errors) => {
                let error_msg = format!("Partial validation: {}", errors.join("; "));
                Ok(("partial".to_string(), 0.7, Some(error_msg)))
            }
            ValidationResult::Invalid(errors) => {
                let error_msg = format!("Validation failed: {}", errors.join("; "));
                Ok(("unknown".to_string(), 0.3, Some(error_msg)))
            }
        }
    }

    /// Validation result enum


    /// Validate event against PSS protocol rules
    async fn validate_event_against_protocol(
        event: &PssEvent, 
        event_code: &str, 
        database: &DatabasePlugin
    ) -> AppResult<ValidationResult> {
        let start_time = std::time::Instant::now();
        
        // Get validation rules for this event type and protocol version
        let rules = database.get_validation_rules(event_code, "2.3").await?;
        
        if rules.is_empty() {
            // No validation rules found, consider it valid but with lower confidence
            return Ok(ValidationResult::Partial(vec!["No validation rules defined".to_string()]));
        }
        
        let mut errors = Vec::new();
        let mut passed_rules = 0;
        
        for rule in &rules {
            if !rule.is_active {
                continue;
            }
            
            match Self::apply_validation_rule(event, rule) {
                Ok(ValidationResult::Valid) => {
                    passed_rules += 1;
                }
                Ok(ValidationResult::Partial(partial_errors)) => {
                    errors.extend(partial_errors);
                    passed_rules += 1; // Partial is still considered passed
                }
                Ok(ValidationResult::Invalid(rule_errors)) => {
                    errors.extend(rule_errors);
                }
                Err(e) => {
                    errors.push(format!("Validation rule '{}' failed: {}", rule.rule_name, e));
                }
            }
        }
        
        let _validation_time = start_time.elapsed().as_millis() as i32;
        
        // Determine overall validation result
        if errors.is_empty() {
            Ok(ValidationResult::Valid)
        } else if passed_rules > 0 && passed_rules >= rules.len() / 2 {
            // At least half the rules passed
            Ok(ValidationResult::Partial(errors))
        } else {
            // Most rules failed
            Ok(ValidationResult::Invalid(errors))
        }
    }

    /// Apply a single validation rule
    fn apply_validation_rule(event: &PssEvent, rule: &crate::database::models::PssEventValidationRule) -> AppResult<ValidationResult> {
        match rule.rule_type.as_str() {
            "range" => Self::validate_range_rule(event, rule),
            "format" => Self::validate_format_rule(event, rule),
            "data_type" => Self::validate_data_type_rule(event, rule),
            "required" => Self::validate_required_rule(event, rule),
            "custom" => Self::validate_custom_rule(event, rule),
            _ => {
                let error = format!("Unknown validation rule type: {}", rule.rule_type);
                Ok(ValidationResult::Invalid(vec![error]))
            }
        }
    }

    /// Validate range rule
    fn validate_range_rule(event: &PssEvent, rule: &crate::database::models::PssEventValidationRule) -> AppResult<ValidationResult> {
        // Parse range from rule definition (e.g., "1-5", "0-4")
        let range_parts: Vec<&str> = rule.rule_definition.split('-').collect();
        if range_parts.len() != 2 {
            return Ok(ValidationResult::Valid); // Invalid range format, skip
        }

        let min_val: i32 = range_parts.get(0).unwrap_or(&"0").parse().unwrap_or(0);
        let max_val: i32 = range_parts.get(1).unwrap_or(&"999").parse().unwrap_or(999);

        let value = match event {
            PssEvent::Points { point_type, .. } if rule.rule_name.contains("point_type") => Some(*point_type as i32),
            PssEvent::HitLevel { level, .. } if rule.rule_name.contains("hit_level") => Some(*level as i32),
            PssEvent::Warnings { athlete1_warnings, .. } if rule.rule_name.contains("warning_count") => Some(*athlete1_warnings as i32),
            PssEvent::Warnings { athlete2_warnings, .. } if rule.rule_name.contains("warning_count") => Some(*athlete2_warnings as i32),
            PssEvent::Round { current_round } if rule.rule_name.contains("round_number") => Some(*current_round as i32),
            _ => None,
        };

        if let Some(val) = value {
            if val < min_val || val > max_val {
                let error_msg = rule.error_message.clone().unwrap_or_else(|| {
                    format!("Value {} is outside valid range {}-{}", val, min_val, max_val)
                });
                return Ok(ValidationResult::Invalid(vec![error_msg]));
            }
        }

        Ok(ValidationResult::Valid)
    }

    /// Validate format rule
    fn validate_format_rule(event: &PssEvent, rule: &crate::database::models::PssEventValidationRule) -> AppResult<ValidationResult> {
        match event {
            PssEvent::Clock { time, .. } |
            PssEvent::Injury { time, .. } |
            PssEvent::Break { time, .. } if rule.rule_name.contains("time_format") => {
                if !Self::is_valid_time_format(time) {
                    let error_msg = rule.error_message.clone().unwrap_or_else(|| {
                        format!("Invalid time format: {}", time)
                    });
                    return Ok(ValidationResult::Invalid(vec![error_msg]));
                }
            }
            _ => {}
        }

        Ok(ValidationResult::Valid)
    }

    /// Validate data type rule
    fn validate_data_type_rule(_event: &PssEvent, _rule: &crate::database::models::PssEventValidationRule) -> AppResult<ValidationResult> {
        // This is mostly handled by the parsing stage, but we can add additional checks here
        Ok(ValidationResult::Valid)
    }

    /// Validate required rule
    fn validate_required_rule(event: &PssEvent, rule: &crate::database::models::PssEventValidationRule) -> AppResult<ValidationResult> {
        // Check if required fields are present and not empty
        match event {
            PssEvent::Athletes { athlete1_short, athlete2_short, .. } => {
                if athlete1_short.is_empty() || athlete2_short.is_empty() {
                    let error_msg = rule.error_message.clone().unwrap_or_else(|| {
                        "Athlete names are required".to_string()
                    });
                    return Ok(ValidationResult::Invalid(vec![error_msg]));
                }
            }
            PssEvent::MatchConfig { match_id, .. } => {
                if match_id.is_empty() {
                    let error_msg = rule.error_message.clone().unwrap_or_else(|| {
                        "Match ID is required".to_string()
                    });
                    return Ok(ValidationResult::Invalid(vec![error_msg]));
                }
            }
            _ => {}
        }

        Ok(ValidationResult::Valid)
    }

    /// Validate custom rule
    fn validate_custom_rule(event: &PssEvent, rule: &crate::database::models::PssEventValidationRule) -> AppResult<ValidationResult> {
        // Parse the custom rule definition
        let rule_def = &rule.rule_definition;
        
        // Handle different custom validation scenarios
        match rule_def.as_str() {
            "athlete_number_valid" => {
                // Validate athlete numbers are 1 or 2
                let mut errors = Vec::new();
                match event {
                    PssEvent::Points { athlete, .. } |
                    PssEvent::HitLevel { athlete, .. } => {
                        if *athlete != 1 && *athlete != 2 {
                            errors.push(format!("Invalid athlete number: {}", athlete));
                        }
                    }
                    PssEvent::Injury { athlete, .. } => {
                        if *athlete > 2 {
                            errors.push(format!("Invalid athlete number: {}", athlete));
                        }
                    }
                    _ => {}
                }
                
                if errors.is_empty() {
                    Ok(ValidationResult::Valid)
                } else {
                    Ok(ValidationResult::Invalid(errors))
                }
            }
            "time_format_valid" => {
                // Validate time format for clock and injury events
                let mut errors = Vec::new();
                match event {
                    PssEvent::Clock { time, .. } |
                    PssEvent::Injury { time, .. } |
                    PssEvent::Break { time, .. } => {
                        if !Self::is_valid_time_format(time) {
                            errors.push(format!("Invalid time format: {}", time));
                        }
                    }
                    _ => {}
                }
                
                if errors.is_empty() {
                    Ok(ValidationResult::Valid)
                } else {
                    Ok(ValidationResult::Invalid(errors))
                }
            }
            "challenge_status_valid" => {
                // Validate challenge status values
                let mut errors = Vec::new();
                match event {
                    PssEvent::Challenge { accepted, won, canceled, .. } => {
                        if *canceled && accepted.is_some() {
                            errors.push("Challenge cannot be both canceled and have acceptance status".to_string());
                        }
                        if let Some(_won_val) = won {
                            if accepted != &Some(true) {
                                errors.push("Challenge result cannot be set without acceptance".to_string());
                            }
                        }
                    }
                    _ => {}
                }
                
                if errors.is_empty() {
                    Ok(ValidationResult::Valid)
                } else {
                    Ok(ValidationResult::Invalid(errors))
                }
            }
            "match_config_valid" => {
                // Validate match configuration
                let mut errors = Vec::new();
                match event {
                    PssEvent::MatchConfig { number, rounds, round_duration, .. } => {
                        if *number == 0 {
                            errors.push("Match number cannot be zero".to_string());
                        }
                        if *rounds < 1 || *rounds > 5 {
                            errors.push(format!("Invalid number of rounds: {}", rounds));
                        }
                        if *round_duration < 30 || *round_duration > 3600 {
                            errors.push(format!("Invalid round duration: {} seconds", round_duration));
                        }
                    }
                    _ => {}
                }
                
                if errors.is_empty() {
                    Ok(ValidationResult::Valid)
                } else {
                    Ok(ValidationResult::Invalid(errors))
                }
            }
            _ => {
                // Unknown custom rule, log warning but don't fail
                log::warn!("Unknown custom validation rule: {}", rule_def);
                Ok(ValidationResult::Valid)
            }
        }
    }

    /// Check if time format is valid (m:ss)
    fn is_valid_time_format(time: &str) -> bool {
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let minutes: Result<i32, _> = parts.get(0).unwrap_or(&"0").parse();
        let seconds: Result<i32, _> = parts.get(1).unwrap_or(&"0").parse();
        
        minutes.is_ok() && seconds.is_ok() && 
        minutes.unwrap() >= 0 && *seconds.as_ref().unwrap() >= 0 && seconds.unwrap() < 60
    }

    /// Get event code for validation
    fn get_event_code(event: &PssEvent) -> String {
        match event {
            PssEvent::Points { .. } => "pt".to_string(),
            PssEvent::HitLevel { .. } => "hl".to_string(),
            PssEvent::Warnings { .. } => "wg".to_string(),
            PssEvent::Injury { .. } => "ij".to_string(),
            PssEvent::Challenge { .. } => "ch".to_string(),
            PssEvent::Break { .. } => "brk".to_string(),
            PssEvent::WinnerRounds { .. } => "wrd".to_string(),
            PssEvent::Winner { .. } => "wmh".to_string(),
            PssEvent::Athletes { .. } => "at".to_string(),
            PssEvent::MatchConfig { .. } => "mch".to_string(),
            PssEvent::Scores { .. } => "s".to_string(),
            PssEvent::CurrentScores { .. } => "sc".to_string(),
            PssEvent::Clock { .. } => "clk".to_string(),
            PssEvent::Round { .. } => "rnd".to_string(),
            PssEvent::FightLoaded => "pre".to_string(),
            PssEvent::FightReady => "rdy".to_string(),
            PssEvent::Raw(raw_msg) => {
                // Try to extract event code from raw messages for better categorization
                if raw_msg.starts_with("avt;") {
                    "avt".to_string()
                } else if raw_msg.starts_with("ref;") {
                    "ref".to_string()
                } else if raw_msg.starts_with("sup;") {
                    "sup".to_string()
                } else if raw_msg.starts_with("rst;") {
                    "rst".to_string()
                } else if raw_msg.starts_with("rsr;") {
                    "rsr".to_string()
                } else if raw_msg.starts_with("win;") {
                    "win".to_string()
                } else {
                    "raw".to_string()
                }
            },
        }
    }

    async fn convert_pss_event_to_db_model(
        event: &PssEvent,
        session_id: i64,
        current_match_id: &Arc<Mutex<Option<i64>>>,
        event_type_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
        database: &DatabasePlugin,
        current_tournament_id: &Arc<Mutex<Option<i64>>>,
        current_tournament_day_id: &Arc<Mutex<Option<i64>>>,
    ) -> AppResult<PssEventV2> {
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
                        format!("PSS Event: {}", event_code),
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
        let tournament_id = {
            let tournament_guard = current_tournament_id.lock().unwrap();
            *tournament_guard
        };

        let tournament_day_id = {
            let tournament_day_guard = current_tournament_day_id.lock().unwrap();
            *tournament_day_guard
        };

        // Create database event model
        let db_event = PssEventV2::new(
            session_id,
            event_type_id,
            Utc::now(),
            format!("{:?}", event), // Raw data representation
            0, // Event sequence will be set by database
        );

        // Set match, round, and tournament IDs
        let mut db_event = db_event;
        db_event.match_id = match_id;
        db_event.round_id = None; // TODO: Track current round
        db_event.tournament_id = tournament_id;
        db_event.tournament_day_id = tournament_day_id;

        // Set parsed data as JSON
        if let Ok(json_data) = serde_json::to_string(event) {
            db_event.parsed_data = Some(json_data);
        }

        Ok(db_event)
    }

    fn convert_pss_event_to_json(event: &PssEvent) -> serde_json::Value {
        match event {
            PssEvent::Points { athlete, point_type } => {
                serde_json::json!({
                    "type": "points",
                    "athlete": athlete,
                    "point_type": point_type,
                    "description": format!("Athlete {} scored {} points", athlete, point_type),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::HitLevel { athlete, level } => {
                serde_json::json!({
                    "type": "hit_level",
                    "athlete": athlete,
                    "level": level,
                    "description": format!("Athlete {} hit level {}", athlete, level),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                serde_json::json!({
                    "type": "warnings",
                    "athlete1_warnings": athlete1_warnings,
                    "athlete2_warnings": athlete2_warnings,
                    "description": format!("Warnings - Athlete1: {}, Athlete2: {}", athlete1_warnings, athlete2_warnings),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Injury { athlete, time, action } => {
                serde_json::json!({
                    "type": "injury",
                    "athlete": athlete,
                    "time": time,
                    "action": action,
                    "description": format!("Injury - Athlete: {}, Time: {}, Action: {:?}", athlete, time, action),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Challenge { source, accepted, won, canceled } => {
                serde_json::json!({
                    "type": "challenge",
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
                    "time": time,
                    "action": action,
                    "description": format!("Break - Time: {}, Action: {:?}", time, action),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                serde_json::json!({
                    "type": "winner_rounds",
                    "round1_winner": round1_winner,
                    "round2_winner": round2_winner,
                    "round3_winner": round3_winner,
                    "description": format!("Winner Rounds - R1: {}, R2: {}, R3: {}", round1_winner, round2_winner, round3_winner),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Winner { name, classification } => {
                serde_json::json!({
                    "type": "winner",
                    "name": name,
                    "classification": classification,
                    "description": format!("Winner: {} ({:?})", name, classification),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Athletes { athlete1_short, athlete1_long, athlete1_country, athlete2_short, athlete2_long, athlete2_country } => {
                serde_json::json!({
                    "type": "athletes",
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
            PssEvent::MatchConfig { number, category, weight, rounds, colors: _, match_id, division, total_rounds, round_duration, countdown_type, count_up, format } => {
                serde_json::json!({
                    "type": "match_config",
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
            PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => {
                serde_json::json!({
                    "type": "scores",
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
            PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                serde_json::json!({
                    "type": "current_scores",
                    "athlete1_score": athlete1_score,
                    "athlete2_score": athlete2_score,
                    "description": format!("Current Scores - A1: {}, A2: {}", athlete1_score, athlete2_score),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Clock { time, action } => {
                serde_json::json!({
                    "type": "clock",
                    "time": time,
                    "action": action,
                    "description": format!("Clock: {} {:?}", time, action.as_ref().unwrap_or(&String::new())),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Round { current_round } => {
                serde_json::json!({
                    "type": "round",
                    "current_round": current_round,
                    "description": format!("Round {}", current_round),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::FightLoaded => {
                serde_json::json!({
                    "type": "fight_loaded",
                    "description": "Fight loaded",
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::FightReady => {
                serde_json::json!({
                    "type": "fight_ready",
                    "description": "Fight ready",
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
            PssEvent::Raw(message) => {
                serde_json::json!({
                    "type": "raw",
                    "message": message,
                    "description": format!("Raw message: {}", message),
                    "timestamp": chrono::Utc::now().timestamp_millis()
                })
            }
        }
    }

    fn extract_event_details(event: &PssEvent, recent_hit_levels: &Arc<Mutex<std::collections::HashMap<u8, Vec<(u8, std::time::SystemTime)>>>>) -> Option<Vec<(String, Option<String>, String)>> {
        match event {
            PssEvent::Points { athlete, point_type } => {
                let mut details = vec![
                ("athlete".to_string(), Some(athlete.to_string()), "u8".to_string()),
                ("point_type".to_string(), Some(point_type.to_string()), "u8".to_string()),
                ];
                
                // Add recent hit levels for this athlete (within last 5 seconds)
                let hit_levels_data = recent_hit_levels.lock().unwrap();
                if let Some(athlete_hit_levels) = hit_levels_data.get(athlete) {
                    let now = std::time::SystemTime::now();
                    let time_window_ms = 5000; // 5 seconds
                    
                    // Filter hit levels within the time window
                    let recent_hit_levels: Vec<u8> = athlete_hit_levels
                        .iter()
                        .filter_map(|(level, timestamp)| {
                            if let Ok(duration) = now.duration_since(*timestamp) {
                                if duration.as_millis() <= time_window_ms as u128 {
                                    Some(*level)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    if !recent_hit_levels.is_empty() {
                        let hit_levels_str = recent_hit_levels.iter()
                            .map(|level| level.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        details.push(("recent_hit_levels".to_string(), Some(hit_levels_str), "String".to_string()));
                        
                        // Add the highest hit level in the recent window
                        if let Some(max_level) = recent_hit_levels.iter().max() {
                            details.push(("max_hit_level".to_string(), Some(max_level.to_string()), "u8".to_string()));
                        }
                        
                        // Add the average hit level in the recent window
                        let avg_level = recent_hit_levels.iter().sum::<u8>() as f32 / recent_hit_levels.len() as f32;
                        details.push(("avg_hit_level".to_string(), Some(format!("{:.1}", avg_level)), "float".to_string()));
                    }
                }
                
                Some(details)
            },
            PssEvent::HitLevel { athlete, level } => Some(vec![
                ("athlete".to_string(), Some(athlete.to_string()), "u8".to_string()),
                ("level".to_string(), Some(level.to_string()), "u8".to_string()),
            ]),
            PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => Some(vec![
                ("athlete1_warnings".to_string(), Some(athlete1_warnings.to_string()), "u8".to_string()),
                ("athlete2_warnings".to_string(), Some(athlete2_warnings.to_string()), "u8".to_string()),
            ]),
            PssEvent::Injury { athlete, time, action } => Some(vec![
                ("athlete".to_string(), Some(athlete.to_string()), "u8".to_string()),
                ("time".to_string(), Some(time.clone()), "String".to_string()),
                ("action".to_string(), action.as_ref().map(|a| a.clone()), "Option<String>".to_string()),
            ]),
            PssEvent::Challenge { source, accepted, won, canceled } => Some(vec![
                ("source".to_string(), Some(source.to_string()), "u8".to_string()),
                ("accepted".to_string(), accepted.map(|a| a.to_string()), "Option<bool>".to_string()),
                ("won".to_string(), won.map(|w| w.to_string()), "Option<bool>".to_string()),
                ("canceled".to_string(), Some(canceled.to_string()), "bool".to_string()),
            ]),
            PssEvent::Break { time, action } => Some(vec![
                ("time".to_string(), Some(time.clone()), "String".to_string()),
                ("action".to_string(), action.as_ref().map(|a| a.clone()), "Option<String>".to_string()),
            ]),
            PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => Some(vec![
                ("round1_winner".to_string(), Some(round1_winner.to_string()), "u8".to_string()),
                ("round2_winner".to_string(), Some(round2_winner.to_string()), "u8".to_string()),
                ("round3_winner".to_string(), Some(round3_winner.to_string()), "u8".to_string()),
            ]),
            PssEvent::Winner { name, classification } => Some(vec![
                ("name".to_string(), Some(name.clone()), "String".to_string()),
                ("classification".to_string(), classification.as_ref().map(|c| c.clone()), "Option<String>".to_string()),
            ]),
            PssEvent::Athletes { athlete1_short, athlete1_long, athlete1_country, athlete2_short, athlete2_long, athlete2_country } => Some(vec![
                ("athlete1_short".to_string(), Some(athlete1_short.clone()), "String".to_string()),
                ("athlete1_long".to_string(), Some(athlete1_long.clone()), "String".to_string()),
                ("athlete1_country".to_string(), Some(athlete1_country.clone()), "String".to_string()),
                ("athlete2_short".to_string(), Some(athlete2_short.clone()), "String".to_string()),
                ("athlete2_long".to_string(), Some(athlete2_long.clone()), "String".to_string()),
                ("athlete2_country".to_string(), Some(athlete2_country.clone()), "String".to_string()),
            ]),
            PssEvent::MatchConfig { number, category, weight, rounds, colors: _, match_id, division, total_rounds, round_duration, countdown_type, count_up, format } => Some(vec![
                ("number".to_string(), Some(number.to_string()), "u32".to_string()),
                ("category".to_string(), Some(category.clone()), "String".to_string()),
                ("weight".to_string(), Some(weight.clone()), "String".to_string()),
                ("rounds".to_string(), Some(rounds.to_string()), "u8".to_string()),
                ("match_id".to_string(), Some(match_id.clone()), "String".to_string()),
                ("division".to_string(), Some(division.clone()), "String".to_string()),
                ("total_rounds".to_string(), Some(total_rounds.to_string()), "u8".to_string()),
                ("round_duration".to_string(), Some(round_duration.to_string()), "u32".to_string()),
                ("countdown_type".to_string(), Some(countdown_type.clone()), "String".to_string()),
                ("count_up".to_string(), Some(count_up.to_string()), "u32".to_string()),
                ("format".to_string(), Some(format.to_string()), "u8".to_string()),
            ]),
            PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => Some(vec![
                ("athlete1_r1".to_string(), Some(athlete1_r1.to_string()), "u8".to_string()),
                ("athlete2_r1".to_string(), Some(athlete2_r1.to_string()), "u8".to_string()),
                ("athlete1_r2".to_string(), Some(athlete1_r2.to_string()), "u8".to_string()),
                ("athlete2_r2".to_string(), Some(athlete2_r2.to_string()), "u8".to_string()),
                ("athlete1_r3".to_string(), Some(athlete1_r3.to_string()), "u8".to_string()),
                ("athlete2_r3".to_string(), Some(athlete2_r3.to_string()), "u8".to_string()),
            ]),
            PssEvent::CurrentScores { athlete1_score, athlete2_score } => Some(vec![
                ("athlete1_score".to_string(), Some(athlete1_score.to_string()), "u8".to_string()),
                ("athlete2_score".to_string(), Some(athlete2_score.to_string()), "u8".to_string()),
            ]),
            PssEvent::Clock { time, action } => Some(vec![
                ("time".to_string(), Some(time.clone()), "String".to_string()),
                ("action".to_string(), action.as_ref().map(|a| a.clone()), "Option<String>".to_string()),
            ]),
            PssEvent::Round { current_round } => Some(vec![
                ("current_round".to_string(), Some(current_round.to_string()), "u8".to_string()),
            ]),
            PssEvent::Raw(message) => Some(vec![
                ("message".to_string(), Some(message.clone()), "String".to_string()),
            ]),
            _ => None,
        }
    }

    async fn handle_match_config_event(
        database: &DatabasePlugin,
        current_match_id: &Arc<Mutex<Option<i64>>>,
        match_id: &str,
    ) -> AppResult<()> {
        let db_match_id = database.get_or_create_pss_match(match_id).await?;
        *current_match_id.lock().unwrap() = Some(db_match_id);
        Ok(())
    }

    async fn handle_athletes_event(
        database: &DatabasePlugin,
        athlete_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
        athlete1_short: &str,
        athlete1_long: &str,
        _athlete1_country: &str,
        athlete2_short: &str,
        athlete2_long: &str,
        _athlete2_country: &str,
    ) -> AppResult<()> {
        let athlete1_id = database.get_or_create_pss_athlete(athlete1_short, athlete1_long).await?;
        let athlete2_id = database.get_or_create_pss_athlete(athlete2_short, athlete2_long).await?;
        
        // Update cache
        {
            let mut cache = athlete_cache.lock().unwrap();
            cache.insert(athlete1_short.to_string(), athlete1_id);
            cache.insert(athlete2_short.to_string(), athlete2_id);
        }
        
        Ok(())
    }

    async fn handle_scores_event(
        database: &DatabasePlugin,
        current_match_id: &Arc<Mutex<Option<i64>>>,
        athlete1_score: u8,
        athlete2_score: u8,
    ) -> AppResult<()> {
        let match_id = {
            let guard = current_match_id.lock().unwrap();
            *guard
        };
        
        if let Some(match_id) = match_id {
            let score1 = PssScore {
                id: None,
                match_id,
                round_id: None,
                athlete_position: 1, // Use athlete_position instead of athlete_id
                score_type: "current".to_string(),
                score_value: athlete1_score as i32,
                timestamp: Utc::now(),
                created_at: Utc::now(),
            };
            
            let score2 = PssScore {
                id: None,
                match_id,
                round_id: None,
                athlete_position: 2, // Use athlete_position instead of athlete_id
                score_type: "current".to_string(),
                score_value: athlete2_score as i32,
                timestamp: Utc::now(),
                created_at: Utc::now(),
            };
            
            database.store_pss_score(&score1).await?;
            database.store_pss_score(&score2).await?;
        }
        
        Ok(())
    }

    async fn handle_warnings_event(
        database: &DatabasePlugin,
        current_match_id: &Arc<Mutex<Option<i64>>>,
        athlete1_warnings: u8,
        athlete2_warnings: u8,
    ) -> AppResult<()> {
        let match_id = {
            let guard = current_match_id.lock().unwrap();
            *guard
        };
        
        if let Some(match_id) = match_id {
            let warning1 = PssWarning {
                id: None,
                match_id,
                round_id: None,
                athlete_position: 1, // Use athlete_position instead of athlete_id
                warning_type: "gam_jeom".to_string(),
                warning_count: athlete1_warnings as i32,
                timestamp: Utc::now(),
                created_at: Utc::now(),
            };
            
            let warning2 = PssWarning {
                id: None,
                match_id,
                round_id: None,
                athlete_position: 2, // Use athlete_position instead of athlete_id
                warning_type: "gam_jeom".to_string(),
                warning_count: athlete2_warnings as i32,
                timestamp: Utc::now(),
                created_at: Utc::now(),
            };
            
            database.store_pss_warning(&warning1).await?;
            database.store_pss_warning(&warning2).await?;
        }
        
        Ok(())
    }

    async fn listen_loop_async(
        socket: Arc<Mutex<Option<UdpSocket>>>,
        event_tx: mpsc::UnboundedSender<PssEvent>,
        status: Arc<Mutex<UdpServerStatus>>,
        stats: Arc<Mutex<UdpStats>>,
        protocol_manager: Arc<ProtocolManager>,
        recent_events: Arc<Mutex<VecDeque<PssEvent>>>,
        database: Arc<DatabasePlugin>,
        current_session_id: Arc<Mutex<Option<i64>>>,
        current_match_id: Arc<Mutex<Option<i64>>>,
        athlete_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
        event_type_cache: Arc<Mutex<std::collections::HashMap<String, i64>>>,
        recent_hit_levels: Arc<Mutex<std::collections::HashMap<u8, Vec<(u8, std::time::SystemTime)>>>>,
        tournament_id: Arc<Mutex<Option<i64>>>,
        tournament_day_id: Arc<Mutex<Option<i64>>>,
        websocket_server: Arc<WebSocketServer>,
    ) {
        println!("🎯 UDP PSS Server listening loop started (async)");
        
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
                        println!("❌ UDP socket is None, stopping listen loop");
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
                        stats_guard.active_connections.insert(src_addr, std::time::SystemTime::now());
                        stats_guard.connected_clients = stats_guard.active_connections.len();
                    }
                    
                    // Convert received data to string
                    let message = match String::from_utf8_lossy(&buffer[..len]).to_string() {
                        msg if msg.trim().is_empty() => continue,
                        msg => msg,
                    };
                    
                    println!("📨 Received PSS message from {}: {}", src_addr, message);
                    
                    // Log raw UDP message for Live Data panel
                    let raw_log_message = format!("📡 Raw UDP message: {}", message);
                    crate::core::app::App::emit_log_event(raw_log_message);
                    
                    // Parse the message with panic protection
                    let parse_result = std::panic::catch_unwind(|| {
                        Self::parse_pss_message(&message, &protocol_manager)
                    });
                    
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
                                        PssEvent::HitLevel { athlete, level } => {
                                            // Track this hit level for potential linking with point events
                                            let mut hit_levels = recent_hit_levels.lock().unwrap();
                                            let now = std::time::SystemTime::now();
                                            
                                            // Get or create the athlete's hit level history
                                            let athlete_hit_levels = hit_levels.entry(*athlete).or_insert_with(Vec::new);
                                            
                                            // Add the new hit level with timestamp
                                            athlete_hit_levels.push((*level, now));
                                            
                                            // Keep only the last 10 hit levels per athlete (to avoid memory bloat)
                                            if athlete_hit_levels.len() > 10 {
                                                athlete_hit_levels.remove(0);
                                            }
                                            
                                            log::debug!("🎯 Tracked hit level for athlete {}: level {}", athlete, level);
                                        }
                                        PssEvent::FightLoaded | PssEvent::FightReady => {
                                            // Clear hit level tracking when a new fight starts
                                            let mut hit_levels = recent_hit_levels.lock().unwrap();
                                            hit_levels.clear();
                                            log::debug!("🧹 Cleared hit level tracking for new fight");
                                        }
                                        _ => {}
                                    }

                            // Store event in database (now properly async)
                            let event_clone = event.clone();
                            let database_clone = database.clone();
                            let current_session_id_clone = current_session_id.clone();
                            let current_match_id_clone = current_match_id.clone();
                            let athlete_cache_clone = athlete_cache.clone();
                            let event_type_cache_clone = event_type_cache.clone();
                            let recent_hit_levels_clone = recent_hit_levels.clone();
                            let tournament_id_clone = tournament_id.clone();
                            let tournament_day_id_clone = tournament_day_id.clone();
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
                                    &tournament_day_id_clone,
                                    &websocket_server_clone,
                                ).await {
                                    log::error!("Failed to store event in database: {}", e);
                                }
                            });

                            // Add event to recent events storage
                            {
                                let mut events_guard = recent_events.lock().unwrap();
                                events_guard.push_back(event.clone());
                                
                                // Keep only the last 100 events
                                if events_guard.len() > 100 {
                                    events_guard.pop_front();
                                }
                            }

                                    // Send event to frontend via Tauri events
                                    let event_json = Self::convert_pss_event_to_json(&event);
                                    
                                    // Log the parsed event and JSON for debugging
                                    log::info!("🎯 Parsed PSS event: {:?}", event);
                                    log::info!("📤 Emitting event JSON: {}", serde_json::to_string(&event_json).unwrap_or_default());
                                    
                                    // Emit to Tauri frontend
                                    if let Err(e) = event_tx.send(event.clone()) {
                                        log::warn!("⚠️ Failed to send PSS event to internal channel: {}", e);
                                    }
                                    
                                    // Emit to frontend via core app's unified event emission
                                    crate::core::app::App::emit_pss_event(event_json);
                                    
                                    // Stream log to frontend for Live Data panel
                                    let log_message = format!("🎯 UDP-EVENT: {:?}", event);
                                    crate::core::app::App::emit_log_event(log_message);
                        }
                        Err(e) => {
                            // Update error stats
                            {
                                let mut stats_guard = stats.lock().unwrap();
                                stats_guard.parse_errors += 1;
                            }
                            
                            println!("⚠️ Failed to parse PSS message '{}': {}", message, e);
                                    
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
                                    if let Err(_) = event_tx.send(raw_event) {
                                        // Don't break the loop, just continue
                                    }
                                }
                            }
                        }
                        Err(panic_info) => {
                            // Handle panic in parsing
                            println!("🚨 Panic occurred while parsing message '{}': {:?}", message, panic_info);
                            
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
                            if let Err(_) = event_tx.send(raw_event) {
                                // Don't break the loop, just continue
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        let error_msg = format!("UDP receive error: {}", e);
                        println!("❌ {}", error_msg);
                        
                        let mut status_guard = status.lock().unwrap();
                        *status_guard = UdpServerStatus::Error(error_msg);
                        break;
                    }
                }
            }
            
            // Small sleep to make the loop responsive to stop requests
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        println!("🎯 UDP PSS Server listening loop ended");
    }



    fn parse_pss_message(message: &str, protocol_manager: &ProtocolManager) -> AppResult<PssEvent> {
        // Log the incoming message for debugging
        log::debug!("🔍 Parsing PSS message: '{}'", message);
        
        // Clean the message: remove trailing semicolons and normalize
        let clean_message = message.trim_end_matches(';').trim();
        let parts: Vec<&str> = clean_message.split(';').collect();
        
        // Handle empty or whitespace-only messages
        if clean_message.is_empty() {
            log::warn!("⚠️ Received empty message, returning Raw event");
            return Ok(PssEvent::Raw(message.to_string()));
        }

        // Handle connection status messages (not PSS events)
        if message.contains("Udp Port") && (message.contains("connected") || message.contains("disconnected")) {
            log::debug!("📡 Connection status message: {}", message);
            return Ok(PssEvent::Raw(message.to_string()));
        }

        // Get protocol parsing rules from the protocol manager in a context-safe way
        let _protocol_rules = if tokio::runtime::Handle::try_current().is_ok() {
            // We are already inside a Tokio runtime. Use block_in_place to run a blocking
            // operation without panicking. This avoids the `Handle::block_on` panic that
            // occurs when called from an async context.
            tokio::task::block_in_place(|| {
                // futures::executor::block_on is allowed inside a blocking section.
                futures::executor::block_on(async { protocol_manager.get_parsing_rules().await })
            })
        } else {
            // We are in a pure synchronous context. Create a lightweight runtime just for
            // this call so we can await the async function safely.
            match tokio::runtime::Runtime::new() {
                Ok(rt) => rt.block_on(async { protocol_manager.get_parsing_rules().await }),
                Err(e) => {
                    log::error!("Failed to create temporary Tokio runtime: {}", e);
                    Ok(std::collections::HashMap::new())
                }
            }
        }
        .unwrap_or_default();

        // Ensure we have at least one part before accessing parts[0]
        if parts.is_empty() {
            log::warn!("⚠️ Message has no parts after splitting: '{}'", message);
            return Ok(PssEvent::Raw(message.to_string()));
        }

        // Helper function to safely get a part with bounds checking
        let get_part = |index: usize| -> Option<&str> {
            if index < parts.len() {
                Some(parts[index])
                } else {
                None
            }
        };

        // Helper function to safely parse a part as u8 with validation
        let parse_u8 = |index: usize, field_name: &str, min: u8, max: u8| -> AppResult<u8> {
            let value = get_part(index)
                .ok_or_else(|| AppError::ConfigError(format!("Missing {} at position {}", field_name, index)))?;
            
            let parsed = value.parse::<u8>()
                .map_err(|_| AppError::ConfigError(format!("Invalid {}: '{}' (not a valid u8)", field_name, value)))?;
            
            if parsed < min || parsed > max {
                return Err(AppError::ConfigError(format!("{} value {} is out of range [{}, {}]", field_name, parsed, min, max)));
            }
            
            Ok(parsed)
        };

        // Helper function to safely parse a part as u32 with validation
        let parse_u32 = |index: usize, field_name: &str, min: u32, max: u32| -> AppResult<u32> {
            let value = get_part(index)
                .ok_or_else(|| AppError::ConfigError(format!("Missing {} at position {}", field_name, index)))?;
            
            let parsed = value.parse::<u32>()
                .map_err(|_| AppError::ConfigError(format!("Invalid {}: '{}' (not a valid u32)", field_name, value)))?;
            
            if parsed < min || parsed > max {
                return Err(AppError::ConfigError(format!("{} value {} is out of range [{}, {}]", field_name, parsed, min, max)));
            }
            
            Ok(parsed)
        };

        // Helper function to safely get a string part with validation
        let get_string = |index: usize, field_name: &str, max_length: usize| -> AppResult<String> {
            let value = get_part(index)
                .ok_or_else(|| AppError::ConfigError(format!("Missing {} at position {}", field_name, index)))?;
            
            if value.len() > max_length {
                return Err(AppError::ConfigError(format!("{} too long: {} chars (max {})", field_name, value.len(), max_length)));
            }
            
            Ok(value.to_string())
        };

        // Helper function to validate time format (m:ss or ss)
        let validate_time_format = |time: &str| -> bool {
            if time.contains(':') {
                // Format: m:ss
                let parts: Vec<&str> = time.split(':').collect();
                if parts.len() != 2 {
                    return false;
                }
                parts.get(0).unwrap_or(&"0").parse::<u8>().is_ok() && parts.get(1).unwrap_or(&"0").parse::<u8>().is_ok()
                } else {
                // Format: ss
                time.parse::<u8>().is_ok()
            }
        };

        // Helper function to validate color format (#RRGGBB)
        let validate_color_format = |color: &str| -> bool {
            color.starts_with('#') && color.len() == 7 && color[1..].chars().all(|c| c.is_ascii_hexdigit())
        };

        // Helper function to safely parse with fallback to raw
        let parse_with_fallback = |result: AppResult<PssEvent>| -> AppResult<PssEvent> {
            match result {
                Ok(event) => Ok(event),
                Err(e) => {
                    log::warn!("⚠️ Parsing failed for '{}': {}. Returning as Raw event.", message, e);
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }
        };

        // Main parsing logic with comprehensive error handling
        let result = match *parts.get(0).unwrap_or(&"") {
            // Points events (pt1, pt2)
            "pt1" => {
                let point_type = parse_u8(1, "point type", 1, 5)?;
                log::debug!("✅ Parsed Points event: athlete=1, type={}", point_type);
                Ok(PssEvent::Points { athlete: 1, point_type })
            }
            "pt2" => {
                let point_type = parse_u8(1, "point type", 1, 5)?;
                log::debug!("✅ Parsed Points event: athlete=2, type={}", point_type);
                Ok(PssEvent::Points { athlete: 2, point_type })
            }

            // Hit level events (hl1, hl2)
            "hl1" => {
                let level = parse_u8(1, "hit level", 1, 100)?;
                log::debug!("✅ Parsed HitLevel event: athlete=1, level={}", level);
                    Ok(PssEvent::HitLevel { athlete: 1, level })
            }
            "hl2" => {
                let level = parse_u8(1, "hit level", 1, 100)?;
                log::debug!("✅ Parsed HitLevel event: athlete=2, level={}", level);
                    Ok(PssEvent::HitLevel { athlete: 2, level })
            }

            // Warnings/Gam-jeom events (wg1, wg2)
            "wg1" => {
                // Parse warnings: wg1;1;wg2;2;
                let athlete1_warnings = parse_u8(1, "athlete1 warnings", 0, 10)?;
                let athlete2_warnings = if parts.len() >= 4 && *parts.get(2).unwrap_or(&"") == "wg2" {
                    parse_u8(3, "athlete2 warnings", 0, 10)?
                } else {
                    0
                };
                log::debug!("✅ Parsed Warnings event: a1={}, a2={}", athlete1_warnings, athlete2_warnings);
                    Ok(PssEvent::Warnings { athlete1_warnings, athlete2_warnings })
            }
            "wg2" => {
                // Handle wg2 as part of wg1 event or standalone
                if parts.len() >= 2 {
                    let athlete2_warnings = parse_u8(1, "athlete2 warnings", 0, 10)?;
                    log::debug!("✅ Parsed Warnings event: a1=0, a2={}", athlete2_warnings);
                    Ok(PssEvent::Warnings { athlete1_warnings: 0, athlete2_warnings })
                } else {
                    log::warn!("⚠️ Incomplete wg2 event, defaulting to 0 warnings");
                    Ok(PssEvent::Warnings { athlete1_warnings: 0, athlete2_warnings: 0 })
                }
            }

            // Injury events (ij0, ij1, ij2)
            "ij0" | "ij1" | "ij2" => {
                let athlete = match *parts.get(0).unwrap_or(&"") {
                    "ij0" => 0,
                    "ij1" => 1,
                    "ij2" => 2,
                    _ => 0,
                };
                
                if parts.len() < 2 {
                    log::warn!("⚠️ Incomplete injury event, missing time");
                    return Ok(PssEvent::Raw(message.to_string()));
                }
                
                let time = get_string(1, "injury time", 10)?;
                if !validate_time_format(&time) {
                    log::warn!("⚠️ Invalid injury time format: '{}'", time);
                    return Ok(PssEvent::Raw(message.to_string()));
                }
                
                let action = if parts.len() > 2 {
                    let action_str = get_string(2, "injury action", 10)?;
                    match action_str.as_str() {
                        "show" | "hide" | "reset" => Some(action_str),
                        _ => {
                            log::warn!("⚠️ Unknown injury action: '{}'", action_str);
                            None
                        }
                    }
                    } else {
                        None
                    };

                log::debug!("✅ Parsed Injury event: athlete={}, time={}, action={:?}", athlete, time, action);
                    Ok(PssEvent::Injury { athlete, time, action })
            }

            // Challenge/IVR events (ch0, ch1, ch2)
            "ch0" | "ch1" | "ch2" => {
                let source = match *parts.get(0).unwrap_or(&"") {
                    "ch0" => 0, // Referee
                    "ch1" => 1, // Athlete 1
                    "ch2" => 2, // Athlete 2
                    _ => 0,
                };
                
                let accepted = if parts.len() > 1 {
                    let val = parse_u8(1, "challenge accepted", 0, 255)?;
                    if val == 255 { // -1 in u8 representation
                        Some(false)
                        } else {
                        Some(val == 1)
                    }
                } else {
                    None
                };
                
                let won = if parts.len() > 2 {
                    Some(parse_u8(2, "challenge won", 0, 1)? == 1)
                } else {
                    None
                };
                
                let canceled = accepted == Some(false);
                log::debug!("✅ Parsed Challenge event: source={}, accepted={:?}, won={:?}, canceled={}", source, accepted, won, canceled);
                Ok(PssEvent::Challenge { source, accepted, won, canceled })
            }

            // Break events (brk)
            "brk" => {
                if parts.len() < 2 {
                    log::warn!("⚠️ Incomplete break event, missing time");
                    return Ok(PssEvent::Raw(message.to_string()));
                }
                
                let time = get_string(1, "break time", 10)?;
                if !validate_time_format(&time) {
                    log::warn!("⚠️ Invalid break time format: '{}'", time);
                    return Ok(PssEvent::Raw(message.to_string()));
                }
                
                let action = if parts.len() > 2 {
                    let action_str = get_string(2, "break action", 10)?;
                    match action_str.as_str() {
                        "stop" | "stopEnd" => Some(action_str),
                        _ => {
                            log::warn!("⚠️ Unknown break action: '{}'", action_str);
                            None
                        }
                    }
                    } else {
                        None
                    };

                log::debug!("✅ Parsed Break event: time={}, action={:?}", time, action);
                    Ok(PssEvent::Break { time, action })
            }

            // Winner rounds events (wrd)
            "wrd" => {
                // Parse: wrd;rd1;0;rd2;0;rd3;0
                let mut round1_winner = 0;
                let mut round2_winner = 0;
                let mut round3_winner = 0;
                
                for i in 1..parts.len() {
                    match *parts.get(i).unwrap_or(&"") {
                        "rd1" if i + 1 < parts.len() => {
                            round1_winner = parse_u8(i + 1, "round1 winner", 0, 2).unwrap_or(0);
                        }
                        "rd2" if i + 1 < parts.len() => {
                            round2_winner = parse_u8(i + 1, "round2 winner", 0, 2).unwrap_or(0);
                        }
                        "rd3" if i + 1 < parts.len() => {
                            round3_winner = parse_u8(i + 1, "round3 winner", 0, 2).unwrap_or(0);
                        }
                        _ => {}
                    }
                }
                
                log::debug!("✅ Parsed WinnerRounds event: r1={}, r2={}, r3={}", round1_winner, round2_winner, round3_winner);
                Ok(PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner })
            }

            // Winner events (wmh)
            "wmh" => {
                if parts.len() < 2 {
                    log::warn!("⚠️ Incomplete winner event, missing name");
                    return Ok(PssEvent::Raw(message.to_string()));
                }
                
                let name = get_string(1, "winner name", 100)?;
                let classification = if parts.len() > 2 {
                    Some(get_string(2, "classification", 50)?)
                    } else {
                        None
                    };

                log::debug!("✅ Parsed Winner event: name={}, classification={:?}", name, classification);
                    Ok(PssEvent::Winner { name, classification })
            }

            // Athletes events (at1)
            "at1" => {
                // Parse: at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;
                if parts.len() >= 7 {
                    let athlete1_short = get_string(1, "athlete1 short", 50)?;
                    let athlete1_long = get_string(2, "athlete1 long", 100)?;
                    let athlete1_country = get_string(3, "athlete1 country", 10)?;
                    let athlete2_short = get_string(5, "athlete2 short", 50)?;
                    let athlete2_long = get_string(6, "athlete2 long", 100)?;
                    let athlete2_country = get_string(7, "athlete2 country", 10)?;
                    
                    log::debug!("✅ Parsed Athletes event: a1='{}'({}), a2='{}'({})", 
                               athlete1_short, athlete1_country, athlete2_short, athlete2_country);
                    
                    Ok(PssEvent::Athletes {
                        athlete1_short,
                        athlete1_long,
                        athlete1_country,
                        athlete2_short,
                        athlete2_long,
                        athlete2_country,
                    })
                } else {
                    log::warn!("⚠️ Incomplete athletes event, expected 7+ parts, got {}", parts.len());
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }

            // Match configuration events (mch)
            "mch" => {
                // Parse: mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;
                if parts.len() >= 15 {
                    let number = parse_u32(1, "match number", 1, 9999)?;
                    let category = get_string(2, "category", 100)?;
                    let weight = get_string(3, "weight", 50)?;
                    let rounds = parse_u8(4, "rounds", 1, 10)?;
                    let bg1 = get_string(5, "bg1", 10)?;
                    let fg1 = get_string(6, "fg1", 10)?;
                    let bg2 = get_string(7, "bg2", 10)?;
                    let fg2 = get_string(8, "fg2", 10)?;
                    let match_id = get_string(9, "match_id", 50)?;
                    let division = get_string(10, "division", 50)?;
                    let total_rounds = parse_u8(11, "total_rounds", 1, 10)?;
                    let round_duration = parse_u32(12, "round_duration", 30, 600)?;
                    let countdown_type = get_string(13, "countdown_type", 20)?;
                    let count_up = parse_u32(14, "count_up", 0, 999)?;
                    let format = parse_u8(15, "format", 1, 10)?;
                    
                    // Validate color formats
                    if !validate_color_format(&bg1) || !validate_color_format(&fg1) || 
                       !validate_color_format(&bg2) || !validate_color_format(&fg2) {
                        log::warn!("⚠️ Invalid color format in match config");
                    }
                    
                    log::debug!("✅ Parsed MatchConfig event: #{} {} {} ({} rounds)", number, category, weight, total_rounds);
                    
                    Ok(PssEvent::MatchConfig {
                        number,
                        category,
                        weight,
                        rounds,
                        colors: (bg1, fg1, bg2, fg2),
                        match_id,
                        division,
                        total_rounds,
                        round_duration,
                        countdown_type,
                        count_up,
                        format,
                    })
                } else {
                    log::warn!("⚠️ Incomplete match config event, expected 15+ parts, got {}", parts.len());
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }

            // Scores events (s11, s21, s12, s22, s13, s23)
            "s11" | "s21" | "s12" | "s22" | "s13" | "s23" => {
                // Parse individual score updates
                let score = parse_u8(1, "score", 0, 50)?;
                let (athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3) = match *parts.get(0).unwrap_or(&"") {
                    "s11" => (score, 0, 0, 0, 0, 0),
                    "s21" => (0, score, 0, 0, 0, 0),
                    "s12" => (0, 0, score, 0, 0, 0),
                    "s22" => (0, 0, 0, score, 0, 0),
                    "s13" => (0, 0, 0, 0, score, 0),
                    "s23" => (0, 0, 0, 0, 0, score),
                    _ => (0, 0, 0, 0, 0, 0),
                };
                
                log::debug!("✅ Parsed Scores event: {}={}", *parts.get(0).unwrap_or(&""), score);
                Ok(PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 })
            }

            // Current scores events (sc1, sc2)
            "sc1" | "sc2" => {
                let score = parse_u8(1, "current score", 0, 50)?;
                let (athlete1_score, athlete2_score) = match *parts.get(0).unwrap_or(&"") {
                    "sc1" => (score, 0),
                    "sc2" => (0, score),
                    _ => (0, 0),
                };
                
                log::debug!("✅ Parsed CurrentScores event: {}={}", *parts.get(0).unwrap_or(&""), score);
                Ok(PssEvent::CurrentScores { athlete1_score, athlete2_score })
            }

            // Clock events (clk)
            "clk" => {
                if parts.len() < 2 {
                    log::warn!("⚠️ Incomplete clock event, missing time");
                    return Ok(PssEvent::Raw(message.to_string()));
                }
                
                let time = get_string(1, "clock time", 10)?;
                if !validate_time_format(&time) {
                    log::warn!("⚠️ Invalid clock time format: '{}'", time);
                    return Ok(PssEvent::Raw(message.to_string()));
                }
                
                let action = if parts.len() > 2 {
                    let action_str = get_string(2, "clock action", 10)?;
                    match action_str.as_str() {
                        "start" | "stop" => Some(action_str),
                        _ => {
                            log::warn!("⚠️ Unknown clock action: '{}'", action_str);
                            None
                        }
                    }
                    } else {
                        None
                    };

                log::debug!("✅ Parsed Clock event: time={}, action={:?}", time, action);
                    Ok(PssEvent::Clock { time, action })
            }

            // Round events (rnd)
            "rnd" => {
                let current_round = parse_u8(1, "current round", 1, 10)?;
                log::debug!("✅ Parsed Round event: round={}", current_round);
                    Ok(PssEvent::Round { current_round })
            }

            // Fight loaded events (pre)
            "pre" => {
                if parts.len() > 1 && *parts.get(1).unwrap_or(&"") == "FightLoaded" {
                    log::debug!("✅ Parsed FightLoaded event");
                    Ok(PssEvent::FightLoaded)
                } else {
                    log::warn!("⚠️ Unknown pre event: '{}'", message);
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }

            // Fight ready events (rdy)
            "rdy" => {
                if parts.len() > 1 && *parts.get(1).unwrap_or(&"") == "FightReady" {
                    log::debug!("✅ Parsed FightReady event");
                    Ok(PssEvent::FightReady)
                } else {
                    log::warn!("⚠️ Unknown rdy event: '{}'", message);
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }

            // Winner events (win)
            "win" => {
                if parts.len() > 1 {
                    let winner = get_string(1, "winner", 20)?;
                    let winner_upper = winner.to_uppercase();
                    if winner_upper != "BLUE" && winner_upper != "RED" {
                        log::warn!("⚠️ Unknown winner value: '{}'", winner);
                    }
                    log::debug!("✅ Parsed Winner event: {}", winner);
                    Ok(PssEvent::Winner { name: winner, classification: None })
                } else {
                    log::warn!("⚠️ Incomplete win event, missing winner");
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }

            // Athlete video time events (avt)
            "avt" => {
                let video_time = parse_u8(1, "video time", 0, 255)?;
                log::debug!("✅ Parsed AthleteVideoTime event: {}", video_time);
                // Handle as raw for now since we don't have a specific event type
                Ok(PssEvent::Raw(format!("avt;{};", video_time)))
            }

            // Additional events that were missing and causing panics
            "ref" => {
                // Referee/judge event - handle as raw for now
                log::debug!("📋 Referee event: {}", message);
                Ok(PssEvent::Raw(message.to_string()))
            }
            "sup" => {
                // Supervision event - handle as raw for now
                log::debug!("📋 Supervision event: {}", message);
                Ok(PssEvent::Raw(message.to_string()))
            }
            "rst" => {
                // Reset/statistics event - handle as raw for now
                log::debug!("📋 Reset/Statistics event: {}", message);
                Ok(PssEvent::Raw(message.to_string()))
            }
            "rsr" => {
                // Reset event - handle as raw for now
                log::debug!("📋 Reset event: {}", message);
                Ok(PssEvent::Raw(message.to_string()))
            }

            // Handle any other unknown event types gracefully
            unknown_event => {
                log::info!("❓ Unknown PSS event type: '{}' in message: '{}'", unknown_event, message);
                Ok(PssEvent::Raw(message.to_string()))
            }
        };

        // Apply fallback logic to prevent crashes
        parse_with_fallback(result)
    }

    /// Set the current tournament context for event tracking
    pub async fn set_tournament_context(&self, tournament_id: Option<i64>, tournament_day_id: Option<i64>) -> AppResult<()> {
        {
            let mut tournament_guard = self.current_tournament_id.lock().unwrap();
            *tournament_guard = tournament_id;
        }
        
        {
            let mut tournament_day_guard = self.current_tournament_day_id.lock().unwrap();
            *tournament_day_guard = tournament_day_id;
        }
        
        log::info!("🎯 Tournament context set: tournament_id={:?}, tournament_day_id={:?}", tournament_id, tournament_day_id);
        Ok(())
    }

    /// Get the current tournament context
    pub fn get_tournament_context(&self) -> (Option<i64>, Option<i64>) {
        let tournament_id = {
            let guard = self.current_tournament_id.lock().unwrap();
            *guard
        };
        
        let tournament_day_id = {
            let guard = self.current_tournament_day_id.lock().unwrap();
            *guard
        };
        
        (tournament_id, tournament_day_id)
    }

    /// Clear tournament context
    pub async fn clear_tournament_context(&self) -> AppResult<()> {
        self.set_tournament_context(None, None).await
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    // Mock protocol manager for testing
    fn create_mock_protocol_manager() -> ProtocolManager {
        ProtocolManager::new().unwrap()
    }

    #[test]
    fn test_parse_points() {
        let protocol_manager = create_mock_protocol_manager();
        let event = UdpServer::parse_pss_message("pt1;3;", &protocol_manager).unwrap();
        match event {
            PssEvent::Points { athlete, point_type } => {
                assert_eq!(athlete, 1);
                assert_eq!(point_type, 3);
            }
            _ => panic!("Expected Points event"),
        }
    }

    #[test]
    fn test_parse_warnings() {
        let protocol_manager = create_mock_protocol_manager();
        let event = UdpServer::parse_pss_message("wg1;1;wg2;2;", &protocol_manager).unwrap();
        match event {
            PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                assert_eq!(athlete1_warnings, 1);
                assert_eq!(athlete2_warnings, 2);
            }
            _ => panic!("Expected Warnings event"),
        }
    }

    #[test]
    fn test_parse_clock() {
        let protocol_manager = create_mock_protocol_manager();
        let event = UdpServer::parse_pss_message("clk;1:23;start;", &protocol_manager).unwrap();
        match event {
            PssEvent::Clock { time, action } => {
                assert_eq!(time, "1:23");
                assert_eq!(action, Some("start".to_string()));
            }
            _ => panic!("Expected Clock event"),
        }
    }
}

