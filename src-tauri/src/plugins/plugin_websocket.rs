use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use log::{info, warn, error, debug};

/// WebSocket server for broadcasting PSS events to HTML overlays
pub struct WebSocketPlugin {
    clients: Arc<RwLock<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>,
    event_sender: broadcast::Sender<Value>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    port: u16,
}

impl WebSocketPlugin {
    pub fn new(port: u16) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            server_handle: None,
            port,
        }
    }

    /// Start the WebSocket server
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        
        info!("WebSocket server started on {}", addr);
        
        let clients = self.clients.clone();
        let event_sender = self.event_sender.clone();
        
        let server_handle = tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let clients = clients.clone();
                let event_sender = event_sender.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = Self::handle_connection(stream, addr, clients, event_sender).await {
                        error!("WebSocket connection error: {}", e);
                    }
                });
            }
        });
        
        self.server_handle = Some(server_handle);
        Ok(())
    }

    /// Handle individual WebSocket connections
    async fn handle_connection(
        stream: tokio::net::TcpStream,
        addr: std::net::SocketAddr,
        clients: Arc<RwLock<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>,
        event_sender: broadcast::Sender<Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ws_stream = accept_async(stream).await?;
        info!("WebSocket client connected: {}", addr);
        
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
        
        let client_id = format!("client_{}", addr);
        
        // Store client
        {
            let mut clients_guard = clients.write().await;
            clients_guard.insert(client_id.clone(), tx);
        }
        
        // Send welcome message
        let welcome_msg = serde_json::json!({
            "type": "connection",
            "message": "Connected to reStrike VTA PSS Event Server",
            "client_id": client_id
        });
        
        if let Err(e) = ws_sender.send(Message::Text(welcome_msg.to_string())).await {
            error!("Failed to send welcome message: {}", e);
        }
        
        // Listen for events from the broadcast channel
        let mut event_receiver = event_sender.subscribe();
        
        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            debug!("Received from client {}: {}", client_id, text);
                            
                            // Handle client messages (ping, etc.)
                            if text == "ping" {
                                if let Err(e) = ws_sender.send(Message::Text("pong".to_string())).await {
                                    error!("Failed to send pong: {}", e);
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Binary(_))) => {
                            // Ignore binary messages for now
                            debug!("Received binary message from client {}", client_id);
                        }
                        Some(Ok(Message::Ping(data))) => {
                            // Respond to ping with pong
                            if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                                error!("Failed to send pong: {}", e);
                                break;
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            // Ignore pong messages
                            debug!("Received pong from client {}", client_id);
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket client disconnected: {}", client_id);
                            break;
                        }
                        Some(Ok(Message::Frame(_))) => {
                            // Ignore raw frames
                            debug!("Received frame from client {}", client_id);
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => break,
                    }
                }
                
                // Handle events from broadcast channel
                event_result = event_receiver.recv() => {
                    match event_result {
                        Ok(event) => {
                            let event_msg = Message::Text(event.to_string());
                            if let Err(e) = ws_sender.send(event_msg).await {
                                error!("Failed to send event to client {}: {}", client_id, e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Event receiver error: {}", e);
                            break;
                        }
                    }
                }
                
                // Handle messages from our channel
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => {
                            if let Err(e) = ws_sender.send(msg).await {
                                error!("Failed to send message to client {}: {}", client_id, e);
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
        
        // Remove client
        {
            let mut clients_guard = clients.write().await;
            clients_guard.remove(&client_id);
        }
        
        info!("WebSocket client disconnected: {}", client_id);
        Ok(())
    }

    /// Broadcast PSS event to all connected clients
    pub async fn broadcast_pss_event(&self, event: Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_data = serde_json::json!({
            "type": "pss_event",
            "data": event,
            "timestamp": chrono::Utc::now().timestamp_millis()
        });
        
        if let Err(e) = self.event_sender.send(event_data) {
            warn!("Failed to broadcast PSS event: {}", e);
        } else {
            debug!("Broadcasted PSS event to {} clients", self.event_sender.receiver_count());
        }
        
        Ok(())
    }

    /// Get connection status
    pub async fn get_status(&self) -> serde_json::Value {
        let clients_guard = self.clients.read().await;
        let client_count = clients_guard.len();
        
        serde_json::json!({
            "running": self.server_handle.is_some(),
            "port": self.port,
            "clients": client_count,
            "address": format!("ws://0.0.0.0:{}", self.port),
            "network_accessible": true
        })
    }

    /// Stop the WebSocket server
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            let _ = handle.await;
        }
        
        // Clear all clients
        {
            let mut clients_guard = self.clients.write().await;
            clients_guard.clear();
        }
        
        info!("WebSocket server stopped");
        Ok(())
    }
}

impl Drop for WebSocketPlugin {
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}

/// Initialize the WebSocket plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    info!("WebSocket plugin initialized");
    Ok(())
} 