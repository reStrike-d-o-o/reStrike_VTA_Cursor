use crate::plugins::plugin_udp::PssEvent;
use crate::types::AppError;
use crate::types::AppResult;
use chrono::Utc;
use serde_json;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio::net::TcpListener;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone, serde::Serialize)]
pub enum WebSocketMessage {
    PssEvent {
        event_type: String,
        event_code: String,
        athlete: String,
        round: u8,
        time: String,
        timestamp: String,
        raw_data: String,
        description: String,
        action: Option<String>,
        // NEW: Structured data fields for direct access
        structured_data: serde_json::Value,
    },
    ConnectionStatus {
        connected: bool,
        timestamp: String,
    },
    Error {
        message: String,
        timestamp: String,
    },
    RawJson(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct WebSocketClient {
    pub id: String,
    pub sender: tokio::sync::mpsc::UnboundedSender<WebSocketMessage>,
    pub connected_at: chrono::DateTime<Utc>,
}

impl WebSocketClient {
    pub fn new(id: String, sender: tokio::sync::mpsc::UnboundedSender<WebSocketMessage>) -> Self {
        Self {
            id,
            sender,
            connected_at: Utc::now(),
        }
    }

    pub fn send(&self, message: WebSocketMessage) -> Result<(), AppError> {
        self.sender.send(message)
            .map_err(|e| AppError::ConfigError(format!("Failed to send message to client {}: {}", self.id, e)))
    }
    
    pub fn send_raw_json(&self, json: serde_json::Value) -> Result<(), AppError> {
        self.sender.send(WebSocketMessage::RawJson(json))
            .map_err(|e| AppError::ConfigError(format!("Failed to send JSON message to client {}: {}", self.id, e)))
    }
}

pub struct WebSocketServer {
    clients: Arc<Mutex<Vec<WebSocketClient>>>,
    event_tx: mpsc::UnboundedSender<PssEvent>,
    server_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    current_time: Arc<Mutex<Option<String>>>, // Track current time from Clock events - changed to Option to handle no time set
    current_round: Arc<Mutex<Option<u8>>>, // Track current round - changed to Option to handle no round set
    match_started: Arc<Mutex<bool>>, // Track if match has started (after clk;{round_duration};start)
    // Match configuration state for proper time tracking
    round_duration: Arc<Mutex<Option<u32>>>, // Round duration in seconds from MatchConfig
    countdown_type: Arc<Mutex<Option<String>>>, // Countdown type from MatchConfig
    count_up: Arc<Mutex<Option<u32>>>, // Count up value from MatchConfig
    format: Arc<Mutex<Option<u8>>>, // Format from MatchConfig
}

impl WebSocketServer {
    /// Format seconds to mm:ss format
    fn format_time_from_seconds(seconds: u32) -> String {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }
    
    /// Check if the clock time matches the expected round duration start time
    fn is_match_start_time(&self, time: &str) -> bool {
        if let Ok(round_duration_guard) = self.round_duration.lock() {
            if let Some(duration) = *round_duration_guard {
                let expected_time = Self::format_time_from_seconds(duration);
                time == expected_time
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn new(event_tx: mpsc::UnboundedSender<PssEvent>) -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
            event_tx,
            server_task: Arc::new(Mutex::new(None)),
            current_time: Arc::new(Mutex::new(None)), // Initialize as None instead of "2:00"
            current_round: Arc::new(Mutex::new(None)), // Initialize as None instead of 1
            match_started: Arc::new(Mutex::new(false)), // Match starts as not ready
            // Initialize new match state fields
            round_duration: Arc::new(Mutex::new(None)),
            countdown_type: Arc::new(Mutex::new(None)),
            count_up: Arc::new(Mutex::new(None)),
            format: Arc::new(Mutex::new(None)),
        }
    }
    
    pub async fn start(&self, port: u16) -> AppResult<()> {
        log::info!("🔌 Starting WebSocket server on port {}", port);
        
        let clients = self.clients.clone();
        let event_tx = self.event_tx.clone();
        
        let task = tokio::spawn(async move {
            if let Err(e) = Self::run_server(port, clients, event_tx).await {
                log::error!("WebSocket server error: {}", e);
            }
        });
        
        if let Ok(mut task_guard) = self.server_task.lock() {
            *task_guard = Some(task);
        }
        Ok(())
    }
    
    pub async fn stop(&self) -> AppResult<()> {
        log::info!("🔌 Stopping WebSocket server");
        
        if let Ok(mut task_guard) = self.server_task.lock() {
            if let Some(task) = task_guard.take() {
                task.abort();
            }
        }
        
        // Clear all clients
        if let Ok(mut clients_guard) = self.clients.lock() {
            clients_guard.clear();
        }
        Ok(())
    }
    
    async fn run_server(
        port: u16,
        clients: Arc<Mutex<Vec<WebSocketClient>>>,
        event_tx: mpsc::UnboundedSender<PssEvent>,
    ) -> AppResult<()> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| AppError::ConfigError(format!("Failed to bind WebSocket server: {}", e)))?;
        
        log::info!("🔌 WebSocket server listening on {}", addr);
        
        while let Ok((stream, addr)) = listener.accept().await {
            log::info!("🔌 New WebSocket connection from {}", addr);
            
            let clients_clone = clients.clone();
            let event_tx_clone = event_tx.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(stream, addr, clients_clone, event_tx_clone).await {
                    log::error!("Client handler error: {}", e);
                }
            });
        }
        
        Ok(())
    }
    
    async fn handle_client(
        stream: tokio::net::TcpStream,
        addr: std::net::SocketAddr,
        clients: Arc<Mutex<Vec<WebSocketClient>>>,
        _event_tx: mpsc::UnboundedSender<PssEvent>,
    ) -> AppResult<()> {
        let client_id = format!("client_{}", addr);
        log::info!("🔌 New WebSocket client connected: {}", client_id);
        
        // Accept the WebSocket connection
        let ws_stream = accept_async(stream).await
            .map_err(|e| AppError::ConfigError(format!("Failed to accept WebSocket: {}", e)))?;
        
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WebSocketMessage>();
        let client = WebSocketClient::new(client_id.clone(), tx.clone());
        
        // Add client to the list
        if let Ok(mut clients_guard) = clients.lock() {
            clients_guard.push(client);
        }
        
        // Send connection status message
        let status_msg = WebSocketMessage::ConnectionStatus {
            connected: true,
            timestamp: Utc::now().to_rfc3339(),
        };
        
        if let Err(e) = tx.send(status_msg) {
            log::error!("Failed to send connection status: {}", e);
        }
        
        // Split the WebSocket stream
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Handle incoming WebSocket messages
        let client_id_receive = client_id.clone();
        let client_id_send = client_id.clone();
        let clients_clone = clients.clone();
        
        let receive_task = tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        log::debug!("Received text message from {}: {}", client_id_receive, text);
                        // Handle text messages (ping, etc.)
                        if text == "ping" {
                            // Just log the ping, don't send a response to avoid loops
                            log::debug!("Received ping from {}", client_id_receive);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        log::info!("Client {} requested close", client_id_receive);
                        break;
                    }
                    Ok(Message::Ping(_data)) => {
                        // Note: We can't send pong here because ws_sender is moved
                        log::debug!("Received ping from {}", client_id_receive);
                    }
                    Err(e) => {
                        log::error!("WebSocket error from {}: {}", client_id_receive, e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        // Handle outgoing messages
        let send_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let json = match message {
                    WebSocketMessage::RawJson(json_value) => {
                        serde_json::to_string(&json_value)
                            .map_err(|e| AppError::ConfigError(format!("Failed to serialize raw JSON: {}", e)))?
                    },
                    _ => {
                        serde_json::to_string(&message)
                            .map_err(|e| AppError::ConfigError(format!("Failed to serialize message: {}", e)))?
                    }
                };
                
                if let Err(e) = ws_sender.send(Message::Text(json)).await {
                    log::error!("Failed to send message to {}: {}", client_id_send, e);
                    break;
                }
            }
            Ok::<(), AppError>(())
        });
        
        // Wait for either task to complete
        tokio::select! {
            _ = receive_task => {},
            _ = send_task => {},
        }
        
        // Remove client when disconnected
        if let Ok(mut clients_guard) = clients_clone.lock() {
            clients_guard.retain(|c| c.id != client_id);
        }
        log::info!("🔌 Client {} disconnected", client_id);
        
        Ok(())
    }
    
    pub fn broadcast_event(&self, event: &PssEvent) -> AppResult<()> {
        let message = self.convert_pss_event_to_ws_message(event);
        self.broadcast_message(message)
    }
    
    /// Broadcast a JSON event to all connected WebSocket clients (for scoreboard overlay)
    pub fn broadcast_json_event(&self, event_json: &serde_json::Value) -> AppResult<()> {
        // Convert JSON to WebSocket message format
        let message = match event_json {
            serde_json::Value::Object(obj) => {
                let event_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                let description = obj.get("description").and_then(|v| v.as_str()).unwrap_or("");
                
                WebSocketMessage::PssEvent {
                    event_type: event_type.to_string(),
                    event_code: obj.get("event_code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    athlete: obj.get("athlete").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    round: obj.get("round").and_then(|v| v.as_u64()).unwrap_or(1) as u8,
                    time: obj.get("time").and_then(|v| v.as_str()).unwrap_or("0:00").to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    raw_data: serde_json::to_string(event_json).unwrap_or_default(),
                    description: description.to_string(),
                    action: obj.get("action").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    structured_data: serde_json::json!({}), // No structured data for this format
                }
            }
            _ => {
                WebSocketMessage::PssEvent {
                    event_type: "unknown".to_string(),
                    event_code: "".to_string(),
                    athlete: "".to_string(),
                    round: 1,
                    time: "0:00".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    raw_data: "".to_string(),
                    description: "Unknown event".to_string(),
                    action: None,
                    structured_data: serde_json::json!({}), // No structured data for this format
                }
            }
        };
        
        self.broadcast_message(message)
    }
    
    /// Broadcast a WebSocket message to all connected clients
    fn broadcast_message(&self, message: WebSocketMessage) -> AppResult<()> {
        let mut clients = self.clients.lock()
            .map_err(|e| AppError::ConfigError(format!("Failed to lock clients mutex: {}", e)))?;
        
        let client_count = clients.len();
        log::info!("🔌 Broadcasting message to {} connected clients", client_count);
        
        let mut disconnected_clients = Vec::new();
        
        // Convert WebSocketMessage to the format expected by overlays
        let overlay_message = match message {
            WebSocketMessage::PssEvent { 
                event_type, 
                event_code, 
                athlete, 
                round, 
                time, 
                timestamp, 
                raw_data, 
                description, 
                action,
                structured_data 
            } => {
                
                
                // Create base JSON with all fields
                let mut json_data = serde_json::json!({
                    "type": event_type,
                    "event_code": event_code,
                    "athlete": athlete,
                    "round": round,
                    "time": time,
                    "timestamp": timestamp,
                    "raw_data": raw_data,
                    "description": description,
                    "action": action
                });
                
                // Merge structured data fields directly into the JSON
                if let serde_json::Value::Object(mut base_obj) = json_data {
                    if let serde_json::Value::Object(structured_obj) = structured_data {
                        // Merge structured data fields into base object
                        for (key, value) in structured_obj {
                            base_obj.insert(key, value);
                        }
                    }
                    json_data = serde_json::Value::Object(base_obj);
                }
                
                serde_json::json!({
                    "type": "pss_event",
                    "data": json_data
                })
            },
            WebSocketMessage::ConnectionStatus { connected, timestamp } => {
                serde_json::json!({
                    "type": "connection",
                    "connected": connected,
                    "timestamp": timestamp
                })
            },
            WebSocketMessage::Error { message, timestamp } => {
                serde_json::json!({
                    "type": "error",
                    "message": message,
                    "timestamp": timestamp
                })
            },
            WebSocketMessage::RawJson(json_value) => json_value
        };
        
        // Collect indices of disconnected clients
        for (index, client) in clients.iter().enumerate() {
            if let Err(_) = client.send_raw_json(overlay_message.clone()) {
                log::warn!("🔌 Client {} disconnected during broadcast", client.id);
                disconnected_clients.push(index);
            }
        }
        
        // Remove disconnected clients in reverse order to maintain correct indices
        if !disconnected_clients.is_empty() {
            // Sort in descending order to remove from highest index first
            disconnected_clients.sort_by(|a, b| b.cmp(a));
            for &index in &disconnected_clients {
                if index < clients.len() {
                    clients.remove(index);
                }
            }
            log::info!("🔌 Removed {} disconnected clients, {} remaining", disconnected_clients.len(), clients.len());
        }
        
        Ok(())
    }
    
    fn convert_pss_event_to_ws_message(&self, event: &PssEvent) -> WebSocketMessage {
        // Get current time and round state
        let current_time = self.current_time.lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| None);
        let current_round = self.current_round.lock()
            .map(|guard| *guard)
            .unwrap_or_else(|_| None);
        
        // Check if match has started - only filter out events before clk;{round_duration};start
        let match_started = self.match_started.lock()
            .map(|guard| *guard)
            .unwrap_or_else(|_| false);

        // Only filter out events if match hasn't started AND this is not a Clock event
        if !match_started {
            match event {
                PssEvent::Clock { .. } | PssEvent::Round { .. } => {
                    // Allow Clock and Round events to pass through for time/round tracking
                }
                _ => {
                    return WebSocketMessage::PssEvent {
                        event_type: "pre_match".to_string(),
                        event_code: "O".to_string(), // Won't show in table
                        athlete: "".to_string(),
                        round: current_round.unwrap_or(1), // Use last known round or 1
                        time: "0:00".to_string(), // Pre-match events always use 0:00
                        timestamp: Utc::now().to_rfc3339(),
                        raw_data: "".to_string(),
                        description: "Pre-match event (hidden)".to_string(),
                        action: None,
                        structured_data: serde_json::json!({}),
                    };
                }
            }
        }

        // Generate timestamp for this event
        let pss_timestamp = if let Some(_raw_data) = self.extract_raw_data(event) {
            // Try to parse PSS time and create a timestamp
            // For now, use current time but this could be enhanced to use actual PSS timestamp
            Utc::now().to_rfc3339()
        } else {
            Utc::now().to_rfc3339()
        };
        
        // Helper function to get appropriate time for events
        let get_event_time = |event_time: Option<String>| -> String {
            match event_time {
                Some(time) => time, // Use provided time if available
                None => {
                    // For events that should have time, use last known time or "0:00"
                    current_time.clone().unwrap_or_else(|| "0:00".to_string())
                }
            }
        };
        
        // Helper function to get appropriate round for events
        let get_event_round = |event_round: Option<u8>| -> u8 {
            match event_round {
                Some(round) => round, // Use provided round if available
                None => {
                    // For events that should have round, use last known round or 1
                    current_round.unwrap_or(1)
                }
            }
        };
        
        match event {
            PssEvent::Clock { time, action } => {
                // Only update current_time when we receive a valid Clock event
                if let Ok(mut time_guard) = self.current_time.lock() {
                    *time_guard = Some(time.clone());
                    log::info!("🕐 Updated current_time to: {}", time);
                }
                // Mark match as started when we see clk;{round_duration};start
                if self.is_match_start_time(&time) && action.as_deref() == Some("start") {
                    if let Ok(mut match_guard) = self.match_started.lock() {
                        *match_guard = true;
                        log::info!("🏁 Match started! (clk;{};start detected)", time);
                    }
                }
                
                WebSocketMessage::PssEvent {
                    event_type: "clock".to_string(),
                    event_code: "CLK".to_string(), // Clock events are system events - CHANGED from O to CLK
                    athlete: "yellow".to_string(), // Clock events are referee-controlled
                    round: get_event_round(None), // Use last known round
                    time: time.clone(), // Clock events always have their own time
                    timestamp: pss_timestamp,
                    raw_data: format!("clk;{};", time),
                    description: format!("Clock: {}", time),
                    action: action.clone(),
                    structured_data: serde_json::json!({
                        "time": time,
                        "action": action
                    }),
                }
            }
            
            PssEvent::Round { current_round } => {
                if let Ok(mut round_guard) = self.current_round.lock() {
                    *round_guard = Some(*current_round);
                }
                
                WebSocketMessage::PssEvent {
                    event_type: "round".to_string(),
                    event_code: "RND".to_string(), // Round events are system events - CHANGED from O to RND
                    athlete: "yellow".to_string(), // Round events are referee-controlled
                    round: *current_round, // Round events always have their own round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp,
                    raw_data: format!("rnd;{};", current_round),
                    description: format!("Round {}", current_round),
                    action: None,
                    structured_data: serde_json::json!({
                        "current_round": current_round
                    }),
                }
            }
            
            PssEvent::Points { athlete, point_type } => {
                // Map athlete number to color string
                let athlete_str = match athlete {
                    1 => "blue".to_string(),
                    2 => "red".to_string(),
                    _ => "yellow".to_string(), // Default to referee
                };
                
                // Get event code from UDP plugin function
                let event_code = crate::plugins::plugin_udp::UdpServer::get_event_code(event);
                
                // Log important events with raw message
                if ["K", "P", "H", "TH", "TB", "R"].contains(&event_code.as_str()) {
                    log::info!("🎯 IMPORTANT EVENT - {}: athlete={}, point_type={}, raw=pt{}, time={}", event_code, athlete, point_type, point_type, get_event_time(None));
                }
                
                // Create appropriate description based on point type
                let description = match point_type {
                    1 => format!("{} punch point", athlete_str),
                    2 => format!("{} body kick", athlete_str), // CHANGED: body point -> body kick
                    3 => format!("{} head point", athlete_str),
                    4 => format!("{} technical body", athlete_str),
                    5 => format!("{} technical head", athlete_str),
                    _ => format!("{} point", athlete_str),
                };
                
                WebSocketMessage::PssEvent {
                    event_type: "points".to_string(),
                    event_code: event_code.clone(),
                    athlete: athlete_str.clone(),
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("pt{}", point_type),
                    description: description,
                    action: None,
                    structured_data: serde_json::json!({
                        "athlete": *athlete,
                        "point_type": *point_type,
                        "match_started": match_started
                    }),
                }
            }
            
            PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                // Log important events with raw message
                log::info!("🎯 IMPORTANT EVENT - R: athlete1_warnings={}, athlete2_warnings={}, raw=wg1;{};wg2;{}, time={}", athlete1_warnings, athlete2_warnings, athlete1_warnings, athlete2_warnings, get_event_time(None));
                
                WebSocketMessage::PssEvent {
                    event_type: "warnings".to_string(),
                    event_code: "R".to_string(),
                    athlete: "yellow".to_string(), // Warnings are referee events
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("wg1;{};wg2;{}", athlete1_warnings, athlete2_warnings),
                    description: format!("Warnings - Blue: {}, Red: {}", athlete1_warnings, athlete2_warnings),
                    action: None,
                    structured_data: serde_json::json!({
                        "athlete1_warnings": *athlete1_warnings,
                        "athlete2_warnings": *athlete2_warnings
                    }),
                }
            }
            
            PssEvent::HitLevel { athlete, level } => {
                let athlete_str = match athlete {
                    1 => "blue".to_string(),
                    2 => "red".to_string(),
                    _ => "yellow".to_string(),
                };
                
                // Log important events with raw message
                log::info!("🎯 IMPORTANT EVENT - O: athlete={}, level={}, raw=hl{};{};", athlete, level, athlete, level);
                
                WebSocketMessage::PssEvent {
                    event_type: "hit_level".to_string(),
                    event_code: "O".to_string(), // Hit Level -> Other (CHANGED from K to O)
                    athlete: athlete_str.clone(),
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("hl{};{};", athlete, level),
                    description: format!("{} hit level: {}", athlete_str, level),
                    action: None,
                    structured_data: serde_json::json!({
                        "athlete": *athlete,
                        "level": *level
                    }),
                }
            }
            
            PssEvent::Challenge { source, accepted, won, canceled } => {
                let athlete_str = match source {
                    0 => "yellow".to_string(), // Referee
                    1 => "blue".to_string(),   // Athlete 1
                    2 => "red".to_string(),    // Athlete 2
                    _ => "yellow".to_string(),
                };
                
                // Log important events with raw message
                log::info!("🎯 IMPORTANT EVENT - R: source={}, accepted={:?}, won={:?}, canceled={}, raw=ch{};", source, accepted, won, canceled, source);
                
                WebSocketMessage::PssEvent {
                    event_type: "challenge".to_string(),
                    event_code: "R".to_string(), // Challenge -> Referee
                    athlete: athlete_str.clone(),
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("ch{};", source),
                    description: format!("{} challenge", athlete_str),
                    action: None,
                    structured_data: serde_json::json!({
                        "source": *source,
                        "accepted": accepted,
                        "won": won,
                        "canceled": *canceled
                    }),
                }
            }
            
            PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                WebSocketMessage::PssEvent {
                    event_type: "winner_rounds".to_string(),
                    event_code: "O".to_string(), // Winner rounds -> Other
                    athlete: "".to_string(), // Changed from yellow to empty (less frequent)
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("wr1;{};wr2;{};wr3;{}", round1_winner, round2_winner, round3_winner),
                    description: format!("Winner Rounds - R1: {}, R2: {}, R3: {}", round1_winner, round2_winner, round3_winner),
                    action: None,
                    structured_data: serde_json::json!({
                        "round1_winner": *round1_winner,
                        "round2_winner": *round2_winner,
                        "round3_winner": *round3_winner
                    }),
                }
            }
            
            PssEvent::Winner { name, classification } => {
                WebSocketMessage::PssEvent {
                    event_type: "winner".to_string(),
                    event_code: "O".to_string(), // Winner -> Other
                    athlete: "".to_string(), // Changed from yellow to empty (less frequent)
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("win;{};", name),
                    description: format!("Winner: {}", name),
                    action: None,
                    structured_data: serde_json::json!({
                        "name": name,
                        "classification": classification
                    }),
                }
            }
            
            PssEvent::MatchConfig { number, category, weight, rounds, colors, match_id, division, total_rounds, round_duration, countdown_type, count_up, format } => {
                // Update match configuration state
                if let Ok(mut round_duration_guard) = self.round_duration.lock() {
                    *round_duration_guard = Some(*round_duration);
                }
                if let Ok(mut countdown_type_guard) = self.countdown_type.lock() {
                    *countdown_type_guard = Some(countdown_type.to_string());
                }
                if let Ok(mut count_up_guard) = self.count_up.lock() {
                    *count_up_guard = Some(*count_up);
                }
                if let Ok(mut format_guard) = self.format.lock() {
                    *format_guard = Some(*format);
                }

                // Skip pre-match configuration events - don't show in Event Table
                WebSocketMessage::PssEvent {
                    event_type: "match_config".to_string(),
                    event_code: "O".to_string(), // Changed to O so it won't show in Event Table
                    athlete: "".to_string(),
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("mch;{};{};{}", number, category, weight),
                    description: format!("Match Config - #{} {} {}", number, category, weight),
                    action: None,
                    structured_data: serde_json::json!({
                        "number": number,
                        "category": category,
                        "weight": weight,
                        "rounds": *rounds,
                        "colors": colors,
                        "match_id": match_id,
                        "division": division,
                        "total_rounds": *total_rounds,
                        "round_duration": *round_duration,
                        "countdown_type": countdown_type,
                        "count_up": *count_up,
                        "format": *format
                    }),
                }
            }
            
            PssEvent::Athletes { athlete1_short, athlete1_long, athlete1_country, athlete2_short, athlete2_long, athlete2_country } => {
                // Skip pre-match athlete info events - don't show in Event Table
                WebSocketMessage::PssEvent {
                    event_type: "athletes".to_string(),
                    event_code: "O".to_string(), // Changed to O so it won't show in Event Table
                    athlete: "".to_string(),
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("ath1;{};{};{};ath2;{};{};{}", 
                        athlete1_short, athlete1_long, athlete1_country,
                        athlete2_short, athlete2_long, athlete2_country),
                    description: format!("Athletes - {} vs {}", athlete1_short, athlete2_short),
                    action: None,
                    structured_data: serde_json::json!({
                        "athlete1_short": athlete1_short,
                        "athlete1_long": athlete1_long,
                        "athlete1_country": athlete1_country,
                        "athlete2_short": athlete2_short,
                        "athlete2_long": athlete2_long,
                        "athlete2_country": athlete2_country
                    }),
                }
            }
            
            PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                WebSocketMessage::PssEvent {
                    event_type: "current_scores".to_string(),
                    event_code: "O".to_string(),
                    athlete: "".to_string(),
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("sc1;{};sc2;{}", athlete1_score, athlete2_score),
                    description: format!("Current Scores - Blue: {}, Red: {}", athlete1_score, athlete2_score),
                    action: None,
                    structured_data: serde_json::json!({
                        "athlete1_score": *athlete1_score,
                        "athlete2_score": *athlete2_score
                    }),
                }
            }
            
            PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => {
                WebSocketMessage::PssEvent {
                    event_type: "scores".to_string(),
                    event_code: "O".to_string(),
                    athlete: "".to_string(),
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("s11;{};s21;{};s12;{};s22;{};s13;{};s23;{}", 
                        athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3),
                    description: format!("Scores - R1: {}-{}, R2: {}-{}, R3: {}-{}", 
                        athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3),
                    action: None,
                    structured_data: serde_json::json!({
                        "athlete1_r1": *athlete1_r1,
                        "athlete1_r2": *athlete1_r2,
                        "athlete1_r3": *athlete1_r3,
                        "athlete2_r1": *athlete2_r1,
                        "athlete2_r2": *athlete2_r2,
                        "athlete2_r3": *athlete2_r3
                    }),
                }
            }
            
            PssEvent::FightLoaded => {
                if let Ok(mut match_guard) = self.match_started.lock() {
                    *match_guard = false;
                }
                
                // Reset time when a new fight is loaded
                if let Ok(mut time_guard) = self.current_time.lock() {
                    *time_guard = None;
                    log::info!("🔄 Reset current_time for new fight");
                }
                
                // Reset round when a new fight is loaded
                if let Ok(mut round_guard) = self.current_round.lock() {
                    *round_guard = None;
                    log::info!("🔄 Reset current_round for new fight");
                }
                
                // Reset match configuration state
                if let Ok(mut round_duration_guard) = self.round_duration.lock() {
                    *round_duration_guard = None;
                }
                if let Ok(mut countdown_type_guard) = self.countdown_type.lock() {
                    *countdown_type_guard = None;
                }
                if let Ok(mut count_up_guard) = self.count_up.lock() {
                    *count_up_guard = None;
                }
                if let Ok(mut format_guard) = self.format.lock() {
                    *format_guard = None;
                }
                log::info!("🔄 Reset match configuration state for new fight");
                
                WebSocketMessage::PssEvent {
                    event_type: "fight_loaded".to_string(),
                    event_code: "O".to_string(), // Pre-match event
                    athlete: "".to_string(),
                    round: 1, // Default to 1 for pre-match events
                    time: "0:00".to_string(), // Pre-match events always use 0:00
                    timestamp: pss_timestamp.clone(),
                    raw_data: "fld;".to_string(),
                    description: "Fight Loaded".to_string(),
                    action: None,
                    structured_data: serde_json::json!({}),
                }
            }
            
            PssEvent::FightReady => {
                // Don't mark match as started yet - wait for clk;{round_duration};start
                
                WebSocketMessage::PssEvent {
                    event_type: "fight_ready".to_string(),
                    event_code: "O".to_string(), // Pre-match event
                    athlete: "".to_string(),
                    round: get_event_round(None), // Use last known round
                    time: "0:00".to_string(), // Pre-match events always use 0:00
                    timestamp: pss_timestamp.clone(),
                    raw_data: "rdy;".to_string(),
                    description: "Fight Ready".to_string(),
                    action: None,
                    structured_data: serde_json::json!({}),
                }
            }
            
            PssEvent::Injury { athlete, time, action } => {
                let athlete_str = match athlete {
                    0 => "yellow".to_string(), // Unidentified -> referee
                    1 => "blue".to_string(),
                    2 => "red".to_string(),
                    _ => "yellow".to_string(),
                };
                
                WebSocketMessage::PssEvent {
                    event_type: "injury".to_string(),
                    event_code: "O".to_string(), // Injury -> Other
                    athlete: athlete_str.clone(),
                    round: get_event_round(None), // Use last known round
                    time: time.clone(), // Injury events have their own time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("inj;{};{}", athlete, time),
                    description: format!("Injury - {} {}", athlete_str, time),
                    action: action.clone(),
                    structured_data: serde_json::json!({
                        "athlete": *athlete,
                        "time": time,
                        "action": action
                    }),
                }
            }
            
            PssEvent::Break { time, action } => {
                WebSocketMessage::PssEvent {
                    event_type: "break".to_string(),
                    event_code: "O".to_string(), // Break -> Other
                    athlete: "".to_string(), // Changed from yellow to empty (less frequent)
                    round: get_event_round(None), // Use last known round
                    time: time.clone(), // Break events have their own time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("brk;{}", time),
                    description: format!("Break - {}", time),
                    action: action.clone(),
                    structured_data: serde_json::json!({
                        "time": time,
                        "action": action
                    }),
                }
            }
            
            PssEvent::Supremacy { value } => {
                WebSocketMessage::PssEvent {
                    event_type: "supremacy".to_string(),
                    event_code: "O".to_string(), // Supremacy (system event)
                    athlete: "".to_string(),
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: format!("sup;{}", value),
                    description: format!("Supremacy: {}", value),
                    action: None,
                    structured_data: serde_json::json!({
                        "value": *value
                    }),
                }
            }
            
            PssEvent::Raw(raw_msg) => {
                WebSocketMessage::PssEvent {
                    event_type: "raw".to_string(),
                    event_code: "O".to_string(), // Raw messages are system events
                    athlete: "".to_string(),
                    round: get_event_round(None), // Use last known round
                    time: get_event_time(None), // Use last known time
                    timestamp: pss_timestamp.clone(),
                    raw_data: raw_msg.clone(),
                    description: format!("Raw: {}", raw_msg),
                    action: None,
                    structured_data: serde_json::json!({
                        "raw_message": raw_msg
                    }),
                }
            }
        }
    }
    
    /// Extract raw data string from PSS event for timestamp generation
    fn extract_raw_data(&self, event: &PssEvent) -> Option<String> {
        match event {
            PssEvent::Points { point_type, .. } => Some(format!("pt{}", point_type)),
            PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                Some(format!("wg1;{};wg2;{}", athlete1_warnings, athlete2_warnings))
            }
            PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                Some(format!("wr1;{};wr2;{};wr3;{}", round1_winner, round2_winner, round3_winner))
            }
            PssEvent::MatchConfig { number, category, weight, .. } => {
                Some(format!("mch;{};{};{}", number, category, weight))
            }
            PssEvent::Athletes { athlete1_short, athlete1_long, athlete1_country, athlete2_short, athlete2_long, athlete2_country } => {
                Some(format!("ath1;{};{};{};ath2;{};{};{}", 
                    athlete1_short, athlete1_long, athlete1_country,
                    athlete2_short, athlete2_long, athlete2_country))
            }
            PssEvent::CurrentScores { athlete1_score, athlete2_score } => {
                Some(format!("sc1;{};sc2;{}", athlete1_score, athlete2_score))
            }
            PssEvent::Scores { athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3 } => {
                Some(format!("s11;{};s21;{};s12;{};s22;{};s13;{};s23;{}", 
                    athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3))
            }
            PssEvent::Clock { time, .. } => Some(format!("clk;{}", time)),
            PssEvent::Round { current_round } => Some(format!("rnd;{}", current_round)),
            PssEvent::Injury { athlete, time, .. } => Some(format!("inj;{};{}", athlete, time)),
            PssEvent::Challenge { source, accepted, won, canceled } => {
                Some(format!("chg;{};{};{};{}", source, 
                    accepted.map(|a| if a { "1" } else { "0" }).unwrap_or(""),
                    won.map(|w| if w { "1" } else { "0" }).unwrap_or(""),
                    if *canceled { "1" } else { "0" }))
            }
            PssEvent::Break { time, .. } => Some(format!("brk;{}", time)),
            PssEvent::HitLevel { athlete, level } => Some(format!("hl;{};{}", athlete, level)),
            PssEvent::FightLoaded => Some("fld;".to_string()),
            PssEvent::FightReady => Some("rdy;".to_string()),
            PssEvent::Supremacy { value } => Some(format!("sup;{}", value)),
            PssEvent::Winner { .. } => Some("win;".to_string()),
            PssEvent::Raw(raw_data) => Some(raw_data.clone()),
        }
    }
    
    pub fn get_client_count(&self) -> usize {
        self.clients.lock().map(|guard| guard.len()).unwrap_or(0)
    }
    
    /// Reset the current time state (useful for new matches)
    pub fn reset_time(&self) -> AppResult<()> {
        if let Ok(mut time_guard) = self.current_time.lock() {
            *time_guard = None;
            log::info!("🔄 Reset current_time manually");
        }
        Ok(())
    }
    
    /// Reset the current round state (useful for new matches)
    pub fn reset_round(&self) -> AppResult<()> {
        if let Ok(mut round_guard) = self.current_round.lock() {
            *round_guard = None;
            log::info!("🔄 Reset current_round manually");
        }
        Ok(())
    }
    
    /// Reset both time and round state (useful for new matches)
    pub fn reset_match_state(&self) -> AppResult<()> {
        if let Ok(mut time_guard) = self.current_time.lock() {
            *time_guard = None;
        }
        if let Ok(mut round_guard) = self.current_round.lock() {
            *round_guard = None;
        }
        if let Ok(mut match_guard) = self.match_started.lock() {
            *match_guard = false;
        }
        // Reset match configuration state
        if let Ok(mut round_duration_guard) = self.round_duration.lock() {
            *round_duration_guard = None;
        }
        if let Ok(mut countdown_type_guard) = self.countdown_type.lock() {
            *countdown_type_guard = None;
        }
        if let Ok(mut count_up_guard) = self.count_up.lock() {
            *count_up_guard = None;
        }
        if let Ok(mut format_guard) = self.format.lock() {
            *format_guard = None;
        }
        log::info!("🔄 Reset complete match state (time, round, match_started, config)");
        Ok(())
    }
    
    /// Get the current time state for debugging
    pub fn get_current_time(&self) -> Option<String> {
        self.current_time.lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| None)
    }
    
    /// Get the current round state for debugging
    pub fn get_current_round(&self) -> Option<u8> {
        self.current_round.lock()
            .map(|guard| *guard)
            .unwrap_or_else(|_| None)
    }
    
    /// Get the current round duration for debugging
    pub fn get_round_duration(&self) -> Option<u32> {
        self.round_duration.lock()
            .map(|guard| *guard)
            .unwrap_or_else(|_| None)
    }
    
    /// Get the current countdown type for debugging
    pub fn get_countdown_type(&self) -> Option<String> {
        self.countdown_type.lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| None)
    }
    
    /// Get the current count up value for debugging
    pub fn get_count_up(&self) -> Option<u32> {
        self.count_up.lock()
            .map(|guard| *guard)
            .unwrap_or_else(|_| None)
    }
    
    /// Get the current format for debugging
    pub fn get_format(&self) -> Option<u8> {
        self.format.lock()
            .map(|guard| *guard)
            .unwrap_or_else(|_| None)
    }
    
    /// Get the current match started state for debugging
    pub fn get_match_started(&self) -> bool {
        self.match_started.lock()
            .map(|guard| *guard)
            .unwrap_or_else(|_| false)
    }
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub type WebSocketPlugin = WebSocketServer; 