use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::types::{AppError, AppResult};
use crate::plugins::plugin_udp::PssEvent;
use chrono::Utc;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    },
    ConnectionStatus {
        connected: bool,
        timestamp: String,
    },
    Error {
        message: String,
        timestamp: String,
    },
}

/// WebSocket client connection
#[derive(Debug)]
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
            .map_err(|_| AppError::ConfigError("Failed to send WebSocket message".to_string()))
    }
}

/// WebSocket server for real-time PSS events
pub struct WebSocketServer {
    clients: Arc<Mutex<Vec<WebSocketClient>>>,
    event_tx: mpsc::UnboundedSender<PssEvent>,
    server_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl WebSocketServer {
    pub fn new(event_tx: mpsc::UnboundedSender<PssEvent>) -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
            event_tx,
            server_task: Arc::new(Mutex::new(None)),
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
        // For now, we'll use a simple TCP server approach
        // In a real implementation, you'd use a proper WebSocket library like tokio-tungstenite
        
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await
            .map_err(|e| AppError::ConfigError(format!("Failed to bind WebSocket server: {}", e)))?;
        
        log::info!("🔌 WebSocket server listening on port {}", port);
        
        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    log::info!("🔌 New WebSocket connection from {}", addr);
                    
                    let clients_clone = clients.clone();
                    let event_tx_clone = event_tx.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(socket, addr, clients_clone, event_tx_clone).await {
                            log::error!("Client handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("Failed to accept WebSocket connection: {}", e);
                }
            }
        }
    }
    
    async fn handle_client(
        socket: tokio::net::TcpStream,
        addr: std::net::SocketAddr,
        clients: Arc<Mutex<Vec<WebSocketClient>>>,
        event_tx: mpsc::UnboundedSender<PssEvent>,
    ) -> AppResult<()> {
        // For now, we'll implement a simple message passing system
        // In a real implementation, you'd upgrade the TCP connection to WebSocket
        
        let client_id = format!("client_{}", addr.port());
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WebSocketMessage>();
        
        let client = WebSocketClient::new(client_id.clone(), tx);
        clients.lock().unwrap().push(client);
        
        log::info!("🔌 Client {} connected", client_id);
        
        // Send connection status
        let status_msg = WebSocketMessage::ConnectionStatus {
            connected: true,
            timestamp: Utc::now().to_rfc3339(),
        };
        
        if let Err(e) = tx.send(status_msg) {
            log::error!("Failed to send connection status: {}", e);
        }
        
        // Handle incoming messages (simplified for now)
        while let Some(message) = rx.recv().await {
            // Process the message and potentially forward PSS events
            match message {
                WebSocketMessage::PssEvent { .. } => {
                    // Forward to PSS event system
                    log::debug!("Received PSS event from client {}", client_id);
                }
                _ => {
                    log::debug!("Received message from client {}: {:?}", client_id, message);
                }
            }
        }
        
        // Remove client when disconnected
        clients.lock().unwrap().retain(|c| c.id != client_id);
        log::info!("🔌 Client {} disconnected", client_id);
        
        Ok(())
    }
    
    pub fn broadcast_event(&self, event: &PssEvent) -> AppResult<()> {
        let message = self.convert_pss_event_to_ws_message(event);
        
        let clients = self.clients.lock().unwrap();
        for client in clients.iter() {
            if let Err(e) = client.send(message.clone()) {
                log::warn!("Failed to send event to client {}: {}", client.id, e);
            }
        }
        
        Ok(())
    }
    
    fn convert_pss_event_to_ws_message(&self, event: &PssEvent) -> WebSocketMessage {
        match event {
            PssEvent::Points { athlete, point_type } => {
                let event_code = match point_type {
                    1 => "P".to_string(),
                    2 => "K".to_string(),
                    3 => "H".to_string(),
                    4 => "TB".to_string(),
                    5 => "TH".to_string(),
                    _ => "K".to_string(),
                };
                
                let athlete_str = if *athlete == 1 { "blue" } else { "red" };
                
                WebSocketMessage::PssEvent {
                    event_type: format!("pt{}", athlete),
                    event_code,
                    athlete: athlete_str.to_string(),
                    round: 1, // Will be updated by round events
                    time: "0:00".to_string(), // Will be updated by clock events
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("pt{};{};", athlete, point_type),
                    description: format!("{} {}", athlete_str, event_code),
                }
            }
            
            PssEvent::Warnings { athlete1_warnings, athlete2_warnings } => {
                // Create warning events for both athletes
                let mut messages = Vec::new();
                
                if *athlete1_warnings > 0 {
                    messages.push(WebSocketMessage::PssEvent {
                        event_type: "wg1".to_string(),
                        event_code: "R".to_string(),
                        athlete: "blue".to_string(),
                        round: 1,
                        time: "0:00".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        raw_data: format!("wg1;{};wg2;{};", athlete1_warnings, athlete2_warnings),
                        description: format!("blue warning ({})", athlete1_warnings),
                    });
                }
                
                if *athlete2_warnings > 0 {
                    messages.push(WebSocketMessage::PssEvent {
                        event_type: "wg2".to_string(),
                        event_code: "R".to_string(),
                        athlete: "red".to_string(),
                        round: 1,
                        time: "0:00".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        raw_data: format!("wg1;{};wg2;{};", athlete1_warnings, athlete2_warnings),
                        description: format!("red warning ({})", athlete2_warnings),
                    });
                }
                
                // Return the first message for now (in a real implementation, you'd send both)
                messages.first().cloned().unwrap_or_else(|| WebSocketMessage::Error {
                    message: "Invalid warning event".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                })
            }
            
            PssEvent::Clock { time, .. } => {
                WebSocketMessage::PssEvent {
                    event_type: "clk".to_string(),
                    event_code: "T".to_string(),
                    athlete: "referee".to_string(),
                    round: 1,
                    time: time.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("clk;{};", time),
                    description: format!("Clock: {}", time),
                }
            }
            
            PssEvent::Round { current_round } => {
                WebSocketMessage::PssEvent {
                    event_type: "rnd".to_string(),
                    event_code: "R".to_string(),
                    athlete: "referee".to_string(),
                    round: *current_round,
                    time: "0:00".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("rnd;{};", current_round),
                    description: format!("Round {}", current_round),
                }
            }
            
            PssEvent::Challenge { source, .. } => {
                let athlete = match source {
                    0 => "referee",
                    1 => "blue",
                    2 => "red",
                    _ => "referee",
                };
                
                WebSocketMessage::PssEvent {
                    event_type: format!("ch{}", source),
                    event_code: "R".to_string(),
                    athlete: athlete.to_string(),
                    round: 1,
                    time: "0:00".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("ch{};", source),
                    description: format!("{} challenge", athlete),
                }
            }
            
            PssEvent::HitLevel { athlete, level } => {
                let athlete_str = if *athlete == 1 { "blue" } else { "red" };
                
                WebSocketMessage::PssEvent {
                    event_type: format!("hl{}", athlete),
                    event_code: "H".to_string(),
                    athlete: athlete_str.to_string(),
                    round: 1,
                    time: "0:00".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                    raw_data: format!("hl{};{};", athlete, level),
                    description: format!("{} hit (level {})", athlete_str, level),
                }
            }
            
            _ => {
                WebSocketMessage::Error {
                    message: format!("Unhandled PSS event: {:?}", event),
                    timestamp: Utc::now().to_rfc3339(),
                }
            }
        }
    }
    
    pub fn get_client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }
}

/// Initialize the WebSocket plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔌 Initializing WebSocket plugin...");
    Ok(())
}

// Re-export the main plugin type
pub type WebSocketPlugin = WebSocketServer; 