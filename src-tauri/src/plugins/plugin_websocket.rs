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
    current_time: Arc<Mutex<String>>, // Track current time from Clock events
    current_round: Arc<Mutex<u8>>, // Track current round
}

impl WebSocketServer {
    pub fn new(event_tx: mpsc::UnboundedSender<PssEvent>) -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
            event_tx,
            server_task: Arc::new(Mutex::new(None)),
            current_time: Arc::new(Mutex::new("2:00".to_string())), // Default time
            current_round: Arc::new(Mutex::new(1)), // Default round
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
        
        *self.server_task.lock().unwrap() = Some(task);
        Ok(())
    }
    
    pub async fn stop(&self) -> AppResult<()> {
        log::info!("🔌 Stopping WebSocket server");
        
        if let Some(task) = self.server_task.lock().unwrap().take() {
            task.abort();
        }
        
        // Clear all clients
        self.clients.lock().unwrap().clear();
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
        clients.lock().unwrap().push(client);
        
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
        clients_clone.lock().unwrap().retain(|c| c.id != client_id);
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
        let mut clients = self.clients.lock().unwrap();
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
        
        for (index, client) in clients.iter().enumerate() {
            if let Err(_) = client.send_raw_json(overlay_message.clone()) {
                disconnected_clients.push(index);
            }
        }
        
        // Remove disconnected clients
        for index in disconnected_clients.iter().rev() {
            clients.remove(*index);
        }
        
        Ok(())
    }
    
    fn convert_pss_event_to_ws_message(&self, event: &PssEvent) -> WebSocketMessage {
        // Update current time and round based on Clock and Round events
        match event {
            PssEvent::Clock { time, .. } => {
                *self.current_time.lock().unwrap() = time.clone();
            }
            PssEvent::Round { current_round } => {
                *self.current_round.lock().unwrap() = *current_round;
            }
            _ => {}
        }
        
        // Get current time and round for use in events
        let current_time = self.current_time.lock().unwrap().clone();
        let current_round = *self.current_round.lock().unwrap();
        
        // Get event code from UDP plugin's get_event_code function
        let event_code = crate::plugins::plugin_udp::UdpServer::get_event_code(event);
        
        match event {
            PssEvent::Points { athlete, point_type } => {
                
                let athlete_str = match athlete {
                    1 => "blue".to_string(),
                    2 => "red".to_string(),
                    _ => "unknown".to_string(),
                };
                
                WebSocketMessage::PssEvent {
                    event_type: "points".to_string(),
                    event_code: event_code.clone(),
                    athlete: athlete_str.clone(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("pt{}", point_type),
                    description: format!("{} {}", athlete_str, event_code),
                    action: None,
                    structured_data: serde_json::json!({
                        "athlete": *athlete,
                        "point_type": *point_type
                    }),
                }
            }
            
            PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                WebSocketMessage::PssEvent {
                    event_type: "warnings".to_string(),
                    event_code: event_code.clone(),
                    athlete: "yellow".to_string(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("wg1;{};wg2;{}", athlete1_warnings, athlete2_warnings),
                    description: format!("Warnings - Blue: {}, Red: {}", athlete1_warnings, athlete2_warnings),
                    action: None,
                    structured_data: serde_json::json!({
                        "athlete1_warnings": *athlete1_warnings,
                        "athlete2_warnings": *athlete2_warnings
                    }),
                }
            }
            
            PssEvent::WinnerRounds { round1_winner, round2_winner, round3_winner } => {
                WebSocketMessage::PssEvent {
                    event_type: "winner_rounds".to_string(),
                    event_code: event_code.clone(),
                    athlete: "yellow".to_string(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
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
            
            PssEvent::MatchConfig { number, category, weight, rounds, colors, match_id, division, total_rounds, round_duration, countdown_type, count_up, format } => {
                WebSocketMessage::PssEvent {
                    event_type: "match_config".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: "0:00".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
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
                WebSocketMessage::PssEvent {
                    event_type: "athletes".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: "0:00".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
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
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
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
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
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
            
            PssEvent::Clock { time, action } => {
                WebSocketMessage::PssEvent {
                    event_type: "clock".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("clk;{}", time),
                    description: format!("Clock - {}", time),
                    action: action.clone(),
                    structured_data: serde_json::json!({
                        "time": time,
                        "action": action
                    }),
                }
            }
            
            PssEvent::Round { current_round } => {
                WebSocketMessage::PssEvent {
                    event_type: "round".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: *current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("rnd;{}", current_round),
                    description: format!("Round {}", current_round),
                    action: None,
                    structured_data: serde_json::json!({
                        "current_round": *current_round
                    }),
                }
            }
            
            PssEvent::Injury { athlete, time, action } => {
                let athlete_str = match athlete {
                    0 => "unidentified".to_string(),
                    1 => "blue".to_string(),
                    2 => "red".to_string(),
                    _ => "unknown".to_string(),
                };
                
                WebSocketMessage::PssEvent {
                    event_type: "injury".to_string(),
                    event_code: event_code.clone(),
                    athlete: athlete_str.clone(),
                    round: current_round,
                    time: time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
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
            
            PssEvent::Challenge { source, accepted, won, canceled } => {
                let source_str = match source {
                    0 => "referee".to_string(),
                    1 => "blue".to_string(),
                    2 => "red".to_string(),
                    _ => "unknown".to_string(),
                };
                
                WebSocketMessage::PssEvent {
                    event_type: "challenge".to_string(),
                    event_code: event_code.clone(),
                    athlete: source_str.clone(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("chg;{};{};{};{}", source, 
                        accepted.map(|a| if a { "1" } else { "0" }).unwrap_or(""),
                        won.map(|w| if w { "1" } else { "0" }).unwrap_or(""),
                        if *canceled { "1" } else { "0" }),
                    description: format!("Challenge - {} (accepted: {:?}, won: {:?}, canceled: {})", 
                        source_str, accepted, won, canceled),
                    action: None,
                    structured_data: serde_json::json!({
                        "source": *source,
                        "accepted": accepted,
                        "won": won,
                        "canceled": *canceled
                    }),
                }
            }
            
            PssEvent::Break { time, action } => {
                WebSocketMessage::PssEvent {
                    event_type: "break".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("brk;{}", time),
                    description: format!("Break - {}", time),
                    action: action.clone(),
                    structured_data: serde_json::json!({
                        "time": time,
                        "action": action
                    }),
                }
            }
            
            PssEvent::Winner { name, classification } => {
                WebSocketMessage::PssEvent {
                    event_type: "winner".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("win;{}", name),
                    description: format!("Winner - {}", name),
                    action: None,
                    structured_data: serde_json::json!({
                        "name": name,
                        "classification": classification
                    }),
                }
            }
            
            PssEvent::HitLevel { athlete, level } => {
                let athlete_str = match athlete {
                    1 => "blue".to_string(),
                    2 => "red".to_string(),
                    _ => "unknown".to_string(),
                };
                
                WebSocketMessage::PssEvent {
                    event_type: "hit_level".to_string(),
                    event_code: event_code.clone(),
                    athlete: athlete_str.clone(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("hl;{};{}", athlete, level),
                    description: format!("Hit Level - {} {}", athlete_str, level),
                    action: None,
                    structured_data: serde_json::json!({
                        "athlete": *athlete,
                        "level": *level
                    }),
                }
            }
            
            PssEvent::FightLoaded => {
                WebSocketMessage::PssEvent {
                    event_type: "fight_loaded".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: "fl".to_string(),
                    description: "Fight Loaded".to_string(),
                    action: None,
                    structured_data: serde_json::json!({}),
                }
            }
            
            PssEvent::FightReady => {
                WebSocketMessage::PssEvent {
                    event_type: "fight_ready".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: "rdy;FightReady".to_string(),
                    description: "Fight Ready".to_string(),
                    action: None,
                    structured_data: serde_json::json!({
                        "status": "ready"
                    }),
                }
            }
            
            PssEvent::Supremacy { value } => {
                WebSocketMessage::PssEvent {
                    event_type: "supremacy".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("sup;{}", value),
                    description: format!("Supremacy - Value: {}", value),
                    action: None,
                    structured_data: serde_json::json!({
                        "value": value
                    }),
                }
            }
            
            PssEvent::Raw(raw_data) => {
                WebSocketMessage::PssEvent {
                    event_type: "raw".to_string(),
                    event_code: event_code.clone(),
                    athlete: "".to_string(),
                    round: current_round,
                    time: current_time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: raw_data.clone(),
                    description: format!("Raw data: {}", raw_data),
                    action: None,
                    structured_data: serde_json::json!({
                        "raw_data": raw_data
                    }),
                }
            }
        }
    }
    
    pub fn get_client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub type WebSocketPlugin = WebSocketServer; 