// OBS Core Plugin
// Handles connection management and WebSocket infrastructure
// Extracted from the original plugin_obs.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use serde_json;
use crate::types::{AppError, AppResult};
use super::types::*;

/// Core OBS Plugin for connection management
pub struct ObsCorePlugin {
    context: ObsPluginContext,
    events_plugin: Option<Arc<super::events::ObsEventsPlugin>>,
}

impl ObsCorePlugin {
    /// Create a new OBS Core Plugin
    pub fn new(context: ObsPluginContext) -> Self {
        Self { 
            context,
            events_plugin: None,
        }
    }

    /// Set the events plugin reference for event processing
    pub fn set_events_plugin(&mut self, events_plugin: Arc<super::events::ObsEventsPlugin>) {
        self.events_plugin = Some(events_plugin);
    }



    /// Add a new OBS connection
    pub async fn add_connection(&self, config: ObsConnectionConfig) -> AppResult<()> {
        log::info!("[OBS_CORE] add_connection called for '{}', enabled={}", config.name, config.enabled);
        {
            let mut connections = self.context.connections.lock().await;
            if connections.contains_key(&config.name) {
                return Err(AppError::ConfigError(format!("Connection '{}' already exists", config.name)));
            }
            let connection = ObsConnection {
                config: config.clone(),
                status: ObsConnectionStatus::Disconnected,
                websocket: None,
                request_id_counter: 0,
                pending_requests: HashMap::new(),
                heartbeat_data: None,
                is_connected: false,
                last_heartbeat: None,
            };
            connections.insert(config.name.clone(), connection);
        } // lock is dropped here

        // Don't automatically connect - let user explicitly connect when ready
        log::info!("[OBS_CORE] '{}' configuration saved. Use connect_obs() to establish connection.", config.name);

        Ok(())
    }

    /// Load connections from config manager
    pub async fn load_connections_from_config(&self, config_connections: Vec<crate::config::ObsConnectionConfig>) -> AppResult<()> {
        log::info!("[OBS_CORE] Loading {} connections from config", config_connections.len());
        
        for config_conn in config_connections {
            // Clone all fields before moving config_conn
            let connection_name = config_conn.name.clone();
            let connection_host = config_conn.host.clone();
            let connection_port = config_conn.port;
            let connection_password = config_conn.password.clone();
            let connection_enabled = config_conn.enabled;
            
            // Convert config::ObsConnectionConfig to plugin::ObsConnectionConfig
            let plugin_config = ObsConnectionConfig {
                name: connection_name.clone(),
                host: connection_host,
                port: connection_port,
                password: connection_password,
                protocol_version: ObsWebSocketVersion::V5, // Always v5
                enabled: connection_enabled,
            };
            
            // Add to plugin's internal connections
            if let Err(e) = self.add_connection(plugin_config).await {
                log::warn!("[OBS_CORE] Failed to load connection '{}': {}", connection_name, e);
            }
        }
        
        log::info!("[OBS_CORE] Finished loading connections from config");
        Ok(())
    }

    /// Connect to OBS WebSocket
    pub async fn connect_obs(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_CORE] connect_obs called for '{}'", connection_name);
        
        // Check if connection exists
        {
            let connections = self.context.connections.lock().await;
            if !connections.contains_key(connection_name) {
                return Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)));
            }
        }

        // Update status to Connecting
        {
            let mut connections = self.context.connections.lock().await;
            if let Some(connection) = connections.get_mut(connection_name) {
                connection.status = ObsConnectionStatus::Connecting;
            }
        }

        // Get connection config
        let config = {
            let connections = self.context.connections.lock().await;
            connections.get(connection_name).unwrap().config.clone()
        };

        // Build WebSocket URL
        let ws_url = format!("ws://{}:{}", config.host, config.port);
        log::info!("[OBS_CORE] Connecting to OBS WebSocket at {}", ws_url);

        // Connect to WebSocket
        let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to connect to OBS WebSocket: {}", e)))?;

        log::info!("[OBS_CORE] WebSocket connection established for '{}'", connection_name);

        // Authenticate if needed
        let authenticated_stream = match self.authenticate_v5(connection_name, ws_stream).await {
            Ok(stream) => stream,
            Err(e) => {
                // Set connection status to error and mark as disconnected
                {
                    let mut connections = self.context.connections.lock().await;
                    if let Some(connection) = connections.get_mut(connection_name) {
                        connection.status = ObsConnectionStatus::Error(e.to_string());
                        connection.is_connected = false;
                    }
                }
                return Err(e);
            }
        };

        // Store the authenticated WebSocket stream
        {
            let mut connections = self.context.connections.lock().await;
            if let Some(connection) = connections.get_mut(connection_name) {
                connection.websocket = Some(authenticated_stream);
            }
        }

        // Spawn WebSocket task for message handling
        self.spawn_ws_task(connection_name.to_string()).await;

        log::info!("[OBS_CORE] Successfully connected to OBS '{}'", connection_name);
        Ok(())
    }

    /// Disconnect from OBS WebSocket
    pub async fn disconnect_obs(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_CORE] disconnect_obs called for '{}'", connection_name);
        
        {
            let mut connections = self.context.connections.lock().await;
            if let Some(connection) = connections.get_mut(connection_name) {
                connection.status = ObsConnectionStatus::Disconnected;
                connection.is_connected = false;
                connection.websocket = None;
                connection.pending_requests.clear();
                connection.heartbeat_data = None;
            }
        }

        log::info!("[OBS_CORE] Disconnected from OBS '{}'", connection_name);
        Ok(())
    }

    /// Remove a connection
    pub async fn remove_connection(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_CORE] remove_connection called for '{}'", connection_name);
        
        // First disconnect if connected
        if let Some(status) = self.get_connection_status(connection_name).await {
            if status == ObsConnectionStatus::Connected || status == ObsConnectionStatus::Authenticated {
                self.disconnect_obs(connection_name).await?;
            }
        }

        // Remove from connections map
        {
            let mut connections = self.context.connections.lock().await;
            connections.remove(connection_name);
        }

        log::info!("[OBS_CORE] Removed connection '{}'", connection_name);
        Ok(())
    }

    /// Get connection status
    pub async fn get_connection_status(&self, connection_name: &str) -> Option<ObsConnectionStatus> {
        let connections = self.context.connections.lock().await;
        connections.get(connection_name).map(|conn| conn.status.clone())
    }

    /// Get all connection names
    pub async fn get_connection_names(&self) -> Vec<String> {
        let connections = self.context.connections.lock().await;
        connections.keys().cloned().collect()
    }

    /// Get connection roles (for display purposes)
    pub async fn get_connection_roles(&self) -> Vec<(String, String)> {
        let connections = self.context.connections.lock().await;
        connections
            .iter()
            .map(|(name, conn)| {
                let role = if conn.status == ObsConnectionStatus::Authenticated {
                    "Connected"
                } else if conn.status == ObsConnectionStatus::Connecting || conn.status == ObsConnectionStatus::Authenticating {
                    "Connecting"
                } else {
                    "Disconnected"
                };
                (name.clone(), role.to_string())
            })
            .collect()
    }

    /// Send a request to OBS WebSocket
    pub async fn send_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        let mut connections = self.context.connections.lock().await;
        
        if let Some(connection) = connections.get_mut(connection_name) {
            if connection.status != ObsConnectionStatus::Authenticated {
                return Err(AppError::ConfigError(format!(
                    "Connection '{}' is not authenticated (status: {:?})",
                    connection_name, connection.status
                )));
            }

            if let Some(ws_stream) = &mut connection.websocket {
                // Generate request ID and prepare request before borrowing
                let request_id = format!("req_{}", connection.request_id_counter);
                connection.request_id_counter += 1;
                
                let request = serde_json::json!({
                    "op": 6, // Request
                    "requestType": request_type,
                    "requestId": request_id,
                    "requestData": request_data.unwrap_or(serde_json::json!({}))
                });

                let request_json = serde_json::to_string(&request)
                    .map_err(|e| AppError::ConfigError(format!("Failed to serialize request: {}", e)))?;

                // Create response channel
                let (response_tx, response_rx) = tokio::sync::oneshot::channel();
                connection.pending_requests.insert(request_id.clone(), response_tx);

                // Send request
                if let Err(e) = ws_stream.send(Message::Text(request_json)).await {
                    connection.pending_requests.remove(&request_id);
                    return Err(AppError::ConfigError(format!("Failed to send request: {}", e)));
                }

                // Wait for response
                match tokio::time::timeout(std::time::Duration::from_secs(10), response_rx).await {
                    Ok(Ok(response)) => Ok(response),
                    Ok(Err(_)) => Err(AppError::ConfigError("Response channel closed".to_string())),
                    Err(_) => {
                        connection.pending_requests.remove(&request_id);
                        Err(AppError::ConfigError("Request timeout".to_string()))
                    }
                }
            } else {
                Err(AppError::ConfigError("WebSocket connection not available".to_string()))
            }
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }

    /// Take pending request sender (helper function)
    async fn take_pending_request_sender(
        connections: &Arc<Mutex<HashMap<String, ObsConnection>>>,
        connection_name: &str,
        request_id: &str,
    ) -> Option<tokio::sync::oneshot::Sender<serde_json::Value>> {
        let mut conns = connections.lock().await;
        if let Some(connection) = conns.get_mut(connection_name) {
            connection.pending_requests.remove(request_id)
        } else {
            None
        }
    }

    /// Spawn WebSocket task for a connection
    async fn spawn_ws_task(&self, connection_name: String) {
        let connections = self.context.connections.clone();
        let event_tx = self.context.event_tx.clone();
        let debug_ws_messages = self.context.debug_ws_messages.clone();
        let show_full_events = self.context.show_full_events.clone();
        let plugin = self.context.clone();
        let events_plugin_clone = self.events_plugin.clone();
        
        tokio::spawn(async move {
            loop {
                let ws_stream_opt = {
                    let mut conns = connections.lock().await;
                    conns.get_mut(&connection_name)
                        .and_then(|conn| conn.websocket.take())
                };
                if let Some(ws_stream) = ws_stream_opt {
                    let (_, mut ws_read) = ws_stream.split();
                    
                    // Handle incoming messages
                    while let Some(msg_result) = ws_read.next().await {
                        // Log all incoming messages if debug_ws_messages is enabled
                        let flag = debug_ws_messages.lock().await;
                        if *flag {
                            match &msg_result {
                                Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                    plugin.log_to_file("DEBUG", &format!("[WS-DEBUG][{}] Text: {}", connection_name, text)).await;
                                },
                                Ok(tokio_tungstenite::tungstenite::Message::Binary(bin)) => {
                                    plugin.log_to_file("DEBUG", &format!("[WS-DEBUG][{}] Binary: {:02X?}", connection_name, bin)).await;
                                },
                                Ok(other) => {
                                    plugin.log_to_file("DEBUG", &format!("[WS-DEBUG][{}] Other: {:?}", connection_name, other)).await;
                                },
                                Err(e) => {
                                    plugin.log_to_file("ERROR", &format!("[WS-DEBUG][{}] Error: {}", connection_name, e)).await;
                                }
                            }
                        }
                        match msg_result {
                            Ok(Message::Text(text)) => {
                                // Always log incoming messages for debugging
                                plugin.log_to_file("INFO", &format!("[OBS-RESPONSE][{}] Received: {}", connection_name, text)).await;
                                
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                    // Handle request responses (opcode 7)
                                    if let Some(op) = json["op"].as_u64() {
                                        if op == 7 {
                                            // Request response
                                            if let Some(request_id) = json.pointer("/d/requestId").and_then(|v| v.as_str()) {
                                                plugin.log_to_file("INFO", &format!("[OBS-RESPONSE][{}] Request response for ID: {}", connection_name, request_id)).await;
                                                let tx_opt = ObsCorePlugin::take_pending_request_sender(&connections, &connection_name, request_id).await;
                                                if let Some(tx) = tx_opt {
                                                    let _ = tx.send(json["d"].clone());
                                                } else {
                                                    plugin.log_to_file("WARN", &format!("[OBS-RESPONSE][{}] No pending request found for ID: {}", connection_name, request_id)).await;
                                                }
                                            }
                                        } else if op == 5 {
                                            // Event messages
                                            let event_type = json.pointer("/d/eventType").and_then(|v| v.as_str()).unwrap_or("");
                                            let event_data = &json["d"]["eventData"];
                                            
                                            // Process event through events plugin for filtering and routing
                                            if let Some(events_plugin) = &events_plugin_clone {
                                                events_plugin.handle_obs_event(&connection_name, event_type, event_data.clone()).await;
                                            }
                                            
                                            // Check if we should show all events or only recording/streaming
                                            let show_full = show_full_events.lock().await;
                                            let should_show_event = *show_full || 
                                                event_type == "RecordStateChanged" || 
                                                event_type == "StreamStateChanged" ||
                                                event_type == "ReplayBufferStateChanged";
                                            
                                            if should_show_event {
                                                plugin.log_to_file("INFO", &format!("[OBS-EVENT][{}] Event: {} - Data: {}", connection_name, event_type, serde_json::to_string(event_data).unwrap_or_default())).await;
                                                
                                                // Emit event to frontend if full events are enabled
                                                if *show_full {
                                                    let _ = event_tx.send(ObsEvent::Raw {
                                                        connection_name: connection_name.clone(),
                                                        data: event_data.clone(),
                                                    });
                                                    
                                                    // Store event for frontend polling
                                                    let plugin_clone = plugin.clone();
                                                    let conn_name = connection_name.clone();
                                                    let event_type_clone = event_type.to_string();
                                                    let event_data_clone = event_data.clone();
                                                    tokio::spawn(async move {
                                                        plugin_clone.store_recent_event(conn_name, event_type_clone, event_data_clone).await;
                                                    });
                                                    
                                                    // Also log the event for frontend polling
                                                    plugin.log_to_file("INFO", &format!("[OBS-FRONTEND-EVENT] {}: {}", event_type, serde_json::to_string(&event_data).unwrap_or_default())).await;
                                                }
                                            }
                                            match event_type {
                                                "CurrentProgramSceneChanged" => {
                                                    if let Some(scene_name) = event_data["sceneName"].as_str() {
                                                        let _ = event_tx.send(ObsEvent::SceneChanged {
                                                            connection_name: connection_name.clone(),
                                                            scene_name: scene_name.to_string(),
                                                        });
                                                    }
                                                }
                                                "RecordStateChanged" => {
                                                    if let Some(is_recording) = event_data["outputActive"].as_bool() {
                                                        plugin.log_to_file("INFO", &format!("[OBS-RECORD][{}] Recording: {}", connection_name, is_recording)).await;
                                                        
                                                        // Send event to frontend
                                                        let _ = event_tx.send(ObsEvent::RecordingStateChanged {
                                                            connection_name: connection_name.clone(),
                                                            is_recording,
                                                        });
                                                        
                                                        // Store recording state in connection
                                                        {
                                                            let mut conns = connections.lock().await;
                                                            if let Some(conn) = conns.get_mut(&connection_name) {
                                                                // Create or update heartbeat data with recording state
                                                                let mut heartbeat_data = conn.heartbeat_data.clone().unwrap_or_else(|| serde_json::json!({}));
                                                                if let Some(obj) = heartbeat_data.as_object_mut() {
                                                                    obj.insert("recording".to_string(), serde_json::Value::Bool(is_recording));
                                                                }
                                                                conn.heartbeat_data = Some(heartbeat_data);
                                                                plugin.log_to_file("INFO", &format!("[OBS-RECORD][{}] Updated heartbeat data with recording state", connection_name)).await;
                                                            }
                                                        }
                                                    }
                                                }
                                                "StreamStateChanged" => {
                                                    if let Some(is_streaming) = event_data["outputActive"].as_bool() {
                                                        plugin.log_to_file("INFO", &format!("[OBS-STREAM][{}] Streaming: {}", connection_name, is_streaming)).await;
                                                        
                                                        // Send event to frontend
                                                        let _ = event_tx.send(ObsEvent::StreamStateChanged {
                                                            connection_name: connection_name.clone(),
                                                            is_streaming,
                                                        });
                                                        
                                                        // Store streaming state in connection
                                                        {
                                                            let mut conns = connections.lock().await;
                                                            if let Some(conn) = conns.get_mut(&connection_name) {
                                                                // Create or update heartbeat data with streaming state
                                                                let mut heartbeat_data = conn.heartbeat_data.clone().unwrap_or_else(|| serde_json::json!({}));
                                                                if let Some(obj) = heartbeat_data.as_object_mut() {
                                                                    obj.insert("streaming".to_string(), serde_json::Value::Bool(is_streaming));
                                                                }
                                                                conn.heartbeat_data = Some(heartbeat_data);
                                                                plugin.log_to_file("INFO", &format!("[OBS-STREAM][{}] Updated heartbeat data with streaming state", connection_name)).await;
                                                            }
                                                        }
                                                    }
                                                }
                                                "ReplayBufferStateChanged" => {
                                                    if let Some(is_active) = event_data["outputActive"].as_bool() {
                                                        let _ = event_tx.send(ObsEvent::ReplayBufferStateChanged {
                                                            connection_name: connection_name.clone(),
                                                            is_active,
                                                        });
                                                    }
                                                }
                                                "Heartbeat" => {
                                                    // OBS sends heartbeat messages with status info
                                                    if let Some(recording) = event_data["recording"].as_bool() {
                                                        plugin.log_to_file("INFO", &format!("[OBS-HEARTBEAT][{}] Recording: {}", connection_name, recording)).await;
                                                    }
                                                    if let Some(streaming) = event_data["streaming"].as_bool() {
                                                        plugin.log_to_file("INFO", &format!("[OBS-HEARTBEAT][{}] Streaming: {}", connection_name, streaming)).await;
                                                    }
                                                    if let Some(cpu_usage) = event_data["cpuUsage"].as_f64() {
                                                        plugin.log_to_file("INFO", &format!("[OBS-HEARTBEAT][{}] CPU Usage: {}%", connection_name, cpu_usage)).await;
                                                    }
                                                    
                                                    // Store heartbeat data in connection
                                                    {
                                                        let mut conns = connections.lock().await;
                                                        if let Some(conn) = conns.get_mut(&connection_name) {
                                                            conn.heartbeat_data = Some(event_data.clone());
                                                            plugin.log_to_file("INFO", &format!("[OBS-HEARTBEAT][{}] Stored heartbeat data", connection_name)).await;
                                                        }
                                                    }
                                                }
                                                _other => {
                                                    let _ = event_tx.send(ObsEvent::Raw {
                                                        connection_name: connection_name.clone(),
                                                        data: event_data.clone(),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(Message::Close(_)) => {
                                let _ = event_tx.send(ObsEvent::ConnectionStatusChanged {
                                    connection_name: connection_name.clone(),
                                    status: ObsConnectionStatus::Disconnected,
                                });
                                break;
                            }
                            Err(e) => {
                                let _ = event_tx.send(ObsEvent::Error {
                                    connection_name: connection_name.clone(),
                                    error: format!("WebSocket error: {}", e),
                                });
                                break;
                            }
                            _ => {}
                        }
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Authenticate with OBS WebSocket v5
    async fn authenticate_v5(&self, connection_name: &str, ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) -> AppResult<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>> {
        use tokio_tungstenite::tungstenite::Message;
        use sha2::{Digest, Sha256};
        use base64::{engine::general_purpose, Engine as _};
        use serde_json::json;

        log::info!("[OBS] Starting authentication for connection '{}'", connection_name);
        // Set status to Authenticating
        {
            let mut connections = self.context.connections.lock().await;
            let connection = connections.get_mut(connection_name).unwrap();
            connection.status = ObsConnectionStatus::Authenticating;
        }

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // 1. Wait for Hello message
        log::info!("[AUTH-DEBUG] Waiting for Hello message...");
        let hello_msg = match ws_read.next().await {
            Some(Ok(msg)) => msg,
            Some(Err(e)) => return Err(AppError::ConfigError(format!("WebSocket error: {}", e))),
            None => return Err(AppError::ConfigError("No Hello message from OBS".to_string())),
        };
        let hello_json: serde_json::Value = match &hello_msg {
            Message::Text(text) => {
                log::info!("[AUTH-DEBUG] Received Hello message: {}", text);
                serde_json::from_str(text).map_err(|e| AppError::ConfigError(format!("Invalid Hello JSON: {}", e)))?
            },
            _ => return Err(AppError::ConfigError("Expected Hello text message".to_string())),
        };
        let op = hello_json["op"].as_u64().unwrap_or(0);
        if op != 0 {
            return Err(AppError::ConfigError("First message from OBS was not Hello (op 0)".to_string()));
        }
        let d = &hello_json["d"];
        let rpc_version = d["rpcVersion"].as_u64().unwrap_or(0);
        let authentication_required = d["authentication"].is_object();

        // 2. Prepare Identify message
        let mut identify = json!({
            "op": 1,
            "d": {
                "rpcVersion": rpc_version
            }
        });
        if authentication_required {
            let auth = &d["authentication"];
            let salt = auth["salt"].as_str().unwrap_or("");
            let challenge = auth["challenge"].as_str().unwrap_or("");
            // Get password from config
            let password = {
                let connections = self.context.connections.lock().await;
                let connection = connections.get(connection_name).unwrap();
                let password = connection.config.password.clone().unwrap_or_default();
                log::info!("[AUTH-DEBUG] Using password: '{}' (length: {})", if password.is_empty() { "<empty>" } else { "***" }, password.len());
                password
            };
            // Compute secret = base64(sha256(password + salt))
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            hasher.update(salt.as_bytes());
            let secret = general_purpose::STANDARD.encode(hasher.finalize());
            // Compute auth = base64(sha256(secret + challenge))
            let mut hasher2 = Sha256::new();
            hasher2.update(secret.as_bytes());
            hasher2.update(challenge.as_bytes());
            let auth_str = general_purpose::STANDARD.encode(hasher2.finalize());
            identify["d"]["authentication"] = serde_json::Value::String(auth_str);
        }

        let identify_str = serde_json::to_string(&identify).unwrap();
        log::info!("[AUTH-DEBUG] Sending Identify message: {}", identify_str);
        ws_write.send(Message::Text(identify_str)).await.map_err(|e| AppError::ConfigError(format!("Failed to send Identify: {}", e)))?;

        // 4. Wait for Identified or error
        let mut timeout_counter = 0;
        const MAX_TIMEOUT: u32 = 30; // 30 attempts with 100ms each = 3 seconds
        
        loop {
            timeout_counter += 1;
            if timeout_counter > MAX_TIMEOUT {
                return Err(AppError::ConfigError("Authentication timeout - no response from OBS after 3 seconds".to_string()));
            }
            
            let msg = match tokio::time::timeout(std::time::Duration::from_millis(100), ws_read.next()).await {
                Ok(Some(Ok(msg))) => msg,
                Ok(Some(Err(e))) => return Err(AppError::ConfigError(format!("WebSocket error: {}", e))),
                Ok(None) => return Err(AppError::ConfigError("No response after Identify".to_string())),
                Err(_) => {
                    log::info!("[AUTH-DEBUG] Timeout waiting for response, attempt {}/{}", timeout_counter, MAX_TIMEOUT);
                    continue;
                }
            };
            
            if let Message::Text(text) = &msg {
                log::info!("[AUTH-DEBUG] Received message in handshake loop: {}", text);
                let json: serde_json::Value = serde_json::from_str(text).map_err(|e| AppError::ConfigError(format!("Invalid JSON after Identify: {}", e)))?;
                let op = json["op"].as_u64().unwrap_or(0);
                log::info!("[AUTH-DEBUG] Message opcode: {}", op);
                
                if op == 2 {
                    // Identified
                    log::info!("[AUTH-DEBUG] Authentication successful - received Identified message");
                    break;
                } else if op == 8 {
                    // Error
                    let reason = json["d"]["reason"].as_str().unwrap_or("Unknown error");
                    log::info!("[AUTH-DEBUG] Authentication failed - received error: {}", reason);
                    return Err(AppError::ConfigError(format!("OBS authentication failed: {}", reason)));
                } else {
                    log::info!("[AUTH-DEBUG] Unexpected opcode: {}, continuing to wait...", op);
                }
            } else {
                log::info!("[AUTH-DEBUG] Received non-text message: {:?}", msg);
            }
        }
        // 5. Reunite the stream and return it
        let ws_stream = ws_write.reunite(ws_read).map_err(|_| AppError::ConfigError("Failed to reunite ws stream".to_string()))?;
        // Set status to Authenticated and mark as connected
        {
            let mut connections = self.context.connections.lock().await;
            let connection = connections.get_mut(connection_name).unwrap();
            connection.status = ObsConnectionStatus::Authenticated;
            connection.is_connected = true;
        }
        log::info!("[OBS] Authentication successful for connection '{}'", connection_name);
        // Send status change event
        let _ = self.context.event_tx.send(ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Authenticated,
        });
        Ok(ws_stream)
    }
}

// Implement ObsPlugin trait for the core plugin
impl ObsPlugin for ObsCorePlugin {
    fn name(&self) -> &str {
        "obs_core"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Core Plugin");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Core Plugin");
        Ok(())
    }
} 