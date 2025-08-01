use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};
use crate::plugins::ProtocolManager;
use crate::plugins::plugin_database::DatabasePlugin;
use crate::database::models::{
    UdpServerConfig as DbUdpServerConfig, PssEventV2, 
    PssScore, PssWarning
};
use chrono::Utc;

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

pub struct UdpServer {
    config: UdpServerConfig,
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
}

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
        config: UdpServerConfig, 
        event_tx: mpsc::UnboundedSender<PssEvent>, 
        protocol_manager: Arc<ProtocolManager>,
        database: Arc<DatabasePlugin>,
    ) -> Self {
        Self {
            config,
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
        }
    }

    pub async fn start(&self, config: &crate::config::types::AppConfig) -> AppResult<()> {
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

        // Initialize event type cache
        self.initialize_event_type_cache().await?;
        
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
        // Update status to stopping first
        {
            let mut status = self.status.lock().unwrap();
            if matches!(*status, UdpServerStatus::Stopped) {
                return Ok(()); // Already stopped
            }
            *status = UdpServerStatus::Stopped;
        }

        // End database session
        if let Some(session_id) = *self.current_session_id.lock().unwrap() {
            // Use async operation to end the database session
            let database = self.database.clone();
            let session_id_clone = session_id;
            
            // Spawn async task to end the database session
            tokio::spawn(async move {
                if let Err(e) = database.end_udp_server_session(session_id_clone, "stopped", None).await {
                    log::error!("Failed to end database session: {}", e);
                }
            });
            
            *self.current_session_id.lock().unwrap() = None;
        }

        // Close socket
        {
            let mut socket_guard = self.socket.lock().unwrap();
            *socket_guard = None;
        }

        // Cancel the listener task if it's running
        {
            let mut listener_task_guard = self.listener_task.lock().unwrap();
            if let Some(task) = listener_task_guard.take() {
                task.abort();
                log::info!("Listener task aborted.");
            }
        }

        log::info!("🛑 UDP server stopped");
        
        // Log server stop for Live Data panel
        let stop_log_message = "🛑 UDP server stopped".to_string();
        crate::core::app::App::emit_log_event(stop_log_message);
        
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

    pub fn add_event(&self, event: PssEvent) {
        // Add to recent events (existing logic)
        {
            let mut recent = self.recent_events.lock().unwrap();
            if recent.len() >= 100 {
                recent.pop_front();
            }
            recent.push_back(event.clone());
        }

        // Store in database (async operation)
        let database = self.database.clone();
        let current_session_id = self.current_session_id.clone();
        let current_match_id = self.current_match_id.clone();
        let athlete_cache = self.athlete_cache.clone();
        let event_type_cache = self.event_type_cache.clone();
        let event_clone = event.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::store_event_in_database(
                &database,
                &current_session_id,
                &current_match_id,
                &athlete_cache,
                &event_type_cache,
                &event_clone
            ).await {
                log::error!("Failed to store event in database: {}", e);
            }
        });

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
    ) -> AppResult<()> {
        let session_id = match *current_session_id.lock().unwrap() {
            Some(id) => id,
            None => return Ok(()), // No active session
        };

        // Convert PSS event to database model
        let db_event = Self::convert_pss_event_to_db_model(
            event, 
            session_id, 
            current_match_id,
            event_type_cache,
            database
        ).await?;
        
        let event_id = database.store_pss_event(&db_event).await?;

        // Store event details if available
        if let Some(details) = Self::extract_event_details(event) {
            database.store_pss_event_details(event_id, &details).await?;
        }

        // Handle special event types
        match event {
            PssEvent::MatchConfig { match_id, .. } => {
                Self::handle_match_config_event(database, current_match_id, match_id).await?;
            }
            PssEvent::Athletes { athlete1_short, athlete2_short, .. } => {
                Self::handle_athletes_event(database, athlete_cache, athlete1_short, athlete2_short).await?;
            }
            PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                Self::handle_scores_event(database, current_match_id, *athlete1_score, *athlete2_score).await?;
            }
            PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                Self::handle_warnings_event(database, current_match_id, *athlete1_warnings, *athlete2_warnings).await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn convert_pss_event_to_db_model(
        event: &PssEvent, 
        session_id: i64,
        current_match_id: &Arc<Mutex<Option<i64>>>,
        event_type_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
        database: &DatabasePlugin,
    ) -> AppResult<PssEventV2> {
        let event_type_id = Self::get_event_type_id(event, event_type_cache, database).await?;
        let match_id = {
            let guard = current_match_id.lock().unwrap();
            *guard
        };

        Ok(PssEventV2 {
            id: None,
            session_id,
            match_id,
            round_id: None,
            event_type_id,
            timestamp: Utc::now(),
            raw_data: serde_json::to_string(event)?,
            parsed_data: None,
            event_sequence: 0,
            processing_time_ms: None,
            is_valid: true,
            error_message: None,
            created_at: Utc::now(),
        })
    }

    async fn get_event_type_id(
        event: &PssEvent,
        event_type_cache: &Arc<Mutex<std::collections::HashMap<String, i64>>>,
        database: &DatabasePlugin,
    ) -> AppResult<i64> {
        let event_code = Self::get_event_code(event);
        
        // Check cache first
        {
            let cache = event_type_cache.lock().unwrap();
            if let Some(&id) = cache.get(&event_code) {
                return Ok(id);
            }
        }

        // Get from database
        let event_type = database.get_pss_event_type_by_code(&event_code).await?;
        if let Some(event_type) = event_type {
            if let Some(id) = event_type.id {
                // Update cache
                {
                    let mut cache = event_type_cache.lock().unwrap();
                    cache.insert(event_code, id);
                }
                return Ok(id);
            }
        }

        // For now, return a default ID (event types should be pre-populated)
        Ok(1)
    }

    fn get_event_code(event: &PssEvent) -> String {
        match event {
            PssEvent::Points { .. } => "POINTS".to_string(),
            PssEvent::HitLevel { .. } => "HIT_LEVEL".to_string(),
            PssEvent::Warnings { .. } => "WARNINGS".to_string(),
            PssEvent::Injury { .. } => "INJURY".to_string(),
            PssEvent::Challenge { .. } => "CHALLENGE".to_string(),
            PssEvent::Break { .. } => "BREAK".to_string(),
            PssEvent::WinnerRounds { .. } => "WINNER_ROUNDS".to_string(),
            PssEvent::Winner { .. } => "WINNER".to_string(),
            PssEvent::Athletes { .. } => "ATHLETES".to_string(),
            PssEvent::MatchConfig { .. } => "MATCH_CONFIG".to_string(),
            PssEvent::Scores { .. } => "SCORES".to_string(),
            PssEvent::CurrentScores { .. } => "CURRENT_SCORES".to_string(),
            PssEvent::Clock { .. } => "CLOCK".to_string(),
            PssEvent::Round { .. } => "ROUND".to_string(),
            PssEvent::FightLoaded => "FIGHT_LOADED".to_string(),
            PssEvent::FightReady => "FIGHT_READY".to_string(),
            PssEvent::Raw(_) => "RAW".to_string(),
        }
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

    fn extract_event_details(event: &PssEvent) -> Option<Vec<(String, Option<String>, String)>> {
        match event {
            PssEvent::Points { athlete, point_type } => Some(vec![
                ("athlete".to_string(), Some(athlete.to_string()), "u8".to_string()),
                ("point_type".to_string(), Some(point_type.to_string()), "u8".to_string()),
            ]),
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
        athlete2_short: &str,
    ) -> AppResult<()> {
        let athlete1_id = database.get_or_create_pss_athlete(athlete1_short, athlete1_short).await?;
        let athlete2_id = database.get_or_create_pss_athlete(athlete2_short, athlete2_short).await?;
        
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

                                    // Store event in database (now properly async)
                                    let database_clone = database.clone();
                                    let current_session_id_clone = current_session_id.clone();
                                    let current_match_id_clone = current_match_id.clone();
                                    let athlete_cache_clone = athlete_cache.clone();
                                    let event_type_cache_clone = event_type_cache.clone();
                                    let event_clone = event.clone();
                                    
                                    // Spawn async task for database operation
                                    tokio::spawn(async move {
                                        if let Err(e) = Self::store_event_in_database(
                                            &database_clone,
                                            &current_session_id_clone,
                                            &current_match_id_clone,
                                            &athlete_cache_clone,
                                            &event_type_cache_clone,
                                            &event_clone
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
        // Clean the message: remove trailing semicolons and normalize
        let clean_message = message.trim_end_matches(';');
        let parts: Vec<&str> = clean_message.split(';').collect();
        
        if parts.is_empty() {
            return Err(AppError::ConfigError("Empty message".to_string()));
        }

        // Handle connection status messages (not PSS events)
        if message.contains("Udp Port") && (message.contains("connected") || message.contains("disconnected")) {
            return Ok(PssEvent::Raw(message.to_string()));
        }

        // Get protocol parsing rules from the protocol manager
        let protocol_rules = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context, use block_in_place
                handle.block_on(async {
                    protocol_manager.get_parsing_rules().await
                })
            }
            Err(_) => {
                // We're not in an async context, skip protocol rules for now
                // This prevents runtime creation issues in standard threads
                log::debug!("Skipping protocol rules in non-async context");
                Ok(std::collections::HashMap::new())
            }
        }.unwrap_or_default();

        // TODO: Use protocol_rules for validation and enhanced parsing
        // For now, we'll use the existing parsing logic but log protocol usage
        if !protocol_rules.is_empty() {
            log::debug!("Using protocol rules for parsing: {:?}", protocol_rules);
        }

        // Ensure we have at least one part before accessing parts[0]
        if parts.is_empty() {
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

        // Helper function to safely parse a part as u8
        let parse_u8 = |index: usize, field_name: &str| -> AppResult<u8> {
            get_part(index)
                .ok_or_else(|| AppError::ConfigError(format!("Missing {} at position {}", field_name, index)))?
                .parse::<u8>()
                .map_err(|_| AppError::ConfigError(format!("Invalid {}: {}", field_name, get_part(index).unwrap_or(""))))
        };

        // Helper function to safely parse a part as u32
        let parse_u32 = |index: usize, field_name: &str| -> AppResult<u32> {
            get_part(index)
                .ok_or_else(|| AppError::ConfigError(format!("Missing {} at position {}", field_name, index)))?
                .parse::<u32>()
                .map_err(|_| AppError::ConfigError(format!("Invalid {}: {}", field_name, get_part(index).unwrap_or(""))))
        };

        match parts[0] {
            // Points events
            "pt1" => {
                let point_type = parse_u8(1, "point type")?;
                Ok(PssEvent::Points { athlete: 1, point_type })
            }
            "pt2" => {
                let point_type = parse_u8(1, "point type")?;
                Ok(PssEvent::Points { athlete: 2, point_type })
            }

            // Hit level events
            "hl1" => {
                let level = parse_u8(1, "hit level")?;
                Ok(PssEvent::HitLevel { athlete: 1, level })
            }
            "hl2" => {
                let level = parse_u8(1, "hit level")?;
                Ok(PssEvent::HitLevel { athlete: 2, level })
            }

            // Warning events (wg1;1;wg2;2;)
            "wg1" => {
                // This is a complex parsing as it includes both athletes
                // Expected format: wg1;1;wg2;2;
                if get_part(2) == Some("wg2") {
                    let athlete1_warnings = parse_u8(1, "athlete1 warnings")?;
                    let athlete2_warnings = parse_u8(3, "athlete2 warnings")?;
                    
                    Ok(PssEvent::Warnings { athlete1_warnings, athlete2_warnings })
                } else {
                    Err(AppError::ConfigError("Invalid warning format".to_string()))
                }
            }

            // Injury events
            "ij0" | "ij1" | "ij2" => {
                let athlete = match parts[0] {
                    "ij0" => 0,
                    "ij1" => 1,
                    "ij2" => 2,
                    _ => return Err(AppError::ConfigError("Invalid injury athlete".to_string())),
                };

                let time = get_part(1)
                    .ok_or_else(|| AppError::ConfigError("Missing injury time".to_string()))?
                    .to_string();
                let action = get_part(2).map(|s| s.to_string());

                Ok(PssEvent::Injury { athlete, time, action })
            }

            // Challenge events
            "ch0" | "ch1" | "ch2" => {
                let source = match parts[0] {
                    "ch0" => 0,
                    "ch1" => 1,
                    "ch2" => 2,
                    _ => return Err(AppError::ConfigError("Invalid challenge source".to_string())),
                };

                let (accepted, won, canceled) = match (get_part(1), get_part(2)) {
                    (None, None) => (None, None, false),
                    (Some("-1"), None) => (None, None, true),
                    (Some(val1), None) => {
                        let acc = val1.parse::<u8>().ok().map(|v| v == 1);
                        (acc, None, false)
                    }
                    (Some(val1), Some(val2)) => {
                        let acc = val1.parse::<u8>().ok().map(|v| v == 1);
                        let won_val = val2.parse::<u8>().ok().map(|v| v == 1);
                        (acc, won_val, false)
                    }
                    _ => (None, None, false),
                };

                Ok(PssEvent::Challenge { source, accepted, won, canceled })
            }

            // Break events
            "brk" => {
                let time = get_part(1)
                    .ok_or_else(|| AppError::ConfigError("Missing break time".to_string()))?
                    .to_string();
                let action = get_part(2).map(|s| s.to_string());

                Ok(PssEvent::Break { time, action })
            }

            // Winner rounds
            "wrd" => {
                // Expected format: wrd;rd1;0;rd2;0;rd3;0
                if get_part(1) == Some("rd1") && get_part(3) == Some("rd2") && get_part(5) == Some("rd3") {
                    let round1_winner = parse_u8(2, "round1 winner")?;
                    let round2_winner = parse_u8(4, "round2 winner")?;
                    let round3_winner = parse_u8(6, "round3 winner")?;

                    Ok(PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner })
                } else {
                    Err(AppError::ConfigError("Invalid winner rounds format".to_string()))
                }
            }

            // Final winner
            "wmh" => {
                let name = get_part(1)
                    .ok_or_else(|| AppError::ConfigError("Missing winner name".to_string()))?
                    .to_string();
                let classification = get_part(2).map(|s| s.to_string());

                Ok(PssEvent::Winner { name, classification })
            }

            // Athletes info
            "at1" => {
                // Expected format: at1;short;long;country;at2;short;long;country;
                if get_part(4) == Some("at2") {
                    Ok(PssEvent::Athletes {
                        athlete1_short: get_part(1).unwrap_or("").to_string(),
                        athlete1_long: get_part(2).unwrap_or("").to_string(),
                        athlete1_country: get_part(3).unwrap_or("").to_string(),
                        athlete2_short: get_part(5).unwrap_or("").to_string(),
                        athlete2_long: get_part(6).unwrap_or("").to_string(),
                        athlete2_country: get_part(7).unwrap_or("").to_string(),
                    })
                } else {
                    Err(AppError::ConfigError("Invalid athletes format".to_string()))
                }
            }

            // Clock events
            "clk" => {
                let time = get_part(1)
                    .ok_or_else(|| AppError::ConfigError("Missing clock time".to_string()))?
                    .to_string();
                let action = get_part(2).map(|s| s.to_string());

                Ok(PssEvent::Clock { time, action })
            }

            // Round events
            "rnd" => {
                let current_round = parse_u8(1, "round")?;
                Ok(PssEvent::Round { current_round })
            }

            // Match configuration events
            "mch" => {
                // Expected format: mch;number;category;weight;rounds;bg1;fg1;bg2;fg2;match_id;division;total_rounds;round_duration;countdown_type;format;
                let number = parse_u32(1, "match number")?;
                let category = get_part(2).unwrap_or("").to_string();
                let weight = get_part(3).unwrap_or("").to_string();
                let rounds = parse_u8(4, "rounds")?;
                let colors = (
                    get_part(5).unwrap_or("").to_string(),
                    get_part(6).unwrap_or("").to_string(),
                    get_part(7).unwrap_or("").to_string(),
                    get_part(8).unwrap_or("").to_string()
                );
                let match_id = get_part(9).unwrap_or("").to_string();
                let division = get_part(10).unwrap_or("").to_string();
                let total_rounds = parse_u8(11, "total rounds")?;
                let round_duration = parse_u32(12, "round duration")?;
                let countdown_type = get_part(13).unwrap_or("").to_string();
                let format = parse_u8(14, "format")?;
                
                Ok(PssEvent::MatchConfig {
                    number,
                    category,
                    weight,
                    rounds,
                    colors,
                    match_id,
                    division,
                    total_rounds,
                    round_duration,
                    countdown_type,
                    count_up: 0, // Not used in this format
                    format,
                })
            }

            // Scores events (round-by-round)
            "s11" | "s21" | "s12" | "s22" | "s13" | "s23" => {
                // These are individual score events, we'll handle them as raw for now
                // and let the frontend parse them from the raw message
                Ok(PssEvent::Raw(message.to_string()))
            }

            // Current scores events
            "sc1" | "sc2" => {
                // These are current total scores, we'll handle them as raw for now
                Ok(PssEvent::Raw(message.to_string()))
            }

            // System events
            "pre" => {
                if get_part(1) == Some("FightLoaded") {
                    Ok(PssEvent::FightLoaded)
                } else {
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }
            "rdy" => {
                if get_part(1) == Some("FightReady") {
                    Ok(PssEvent::FightReady)
                } else {
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }

            // Default: return as raw message
            _ => Ok(PssEvent::Raw(message.to_string())),
        }
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
