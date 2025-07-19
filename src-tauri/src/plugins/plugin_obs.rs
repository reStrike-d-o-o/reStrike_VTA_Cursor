use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::types::{AppError, AppResult};
use tokio_tungstenite::tungstenite::Message;
use futures_util::StreamExt;
use futures_util::SinkExt;
use crate::logging::LogManager;

/// Initialize the OBS plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔧 Initializing OBS plugin...");
    Ok(())
}

// OBS WebSocket Protocol Versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObsWebSocketVersion {
    V5,
}

// OBS Connection Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnectionConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub protocol_version: ObsWebSocketVersion,
    pub enabled: bool,
}

// OBS Connection Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObsConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Authenticating,
    Authenticated,
    Error(String),
}

// OBS Connection State
#[derive(Debug)]
pub struct ObsConnection {
    pub config: ObsConnectionConfig,
    pub status: ObsConnectionStatus,
    pub websocket: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    pub request_id_counter: u64,
    pub pending_requests: HashMap<String, tokio::sync::oneshot::Sender<serde_json::Value>>,
    pub heartbeat_data: Option<serde_json::Value>,
}

// Recent events buffer for frontend polling
#[derive(Debug, Clone)]
pub struct RecentEvent {
    pub connection_name: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// OBS Plugin Manager
pub struct ObsPlugin {
    connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    event_tx: mpsc::UnboundedSender<ObsEvent>,
    pub debug_ws_messages: Arc<Mutex<bool>>,
    pub show_full_events: Arc<Mutex<bool>>, // Toggle for showing all OBS events vs only recording/streaming
    recent_events: Arc<Mutex<Vec<RecentEvent>>>, // Recent events for frontend polling
    log_manager: Arc<Mutex<LogManager>>,
}

impl Clone for ObsPlugin {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            event_tx: self.event_tx.clone(),
            debug_ws_messages: self.debug_ws_messages.clone(),
            show_full_events: self.show_full_events.clone(),
            recent_events: self.recent_events.clone(),
            log_manager: self.log_manager.clone(),
        }
    }
}

// OBS Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsEvent {
    ConnectionStatusChanged {
        connection_name: String,
        status: ObsConnectionStatus,
    },
    SceneChanged {
        connection_name: String,
        scene_name: String,
    },
    RecordingStateChanged {
        connection_name: String,
        is_recording: bool,
    },
    StreamStateChanged {
        connection_name: String,
        is_streaming: bool,
    },
    ReplayBufferStateChanged {
        connection_name: String,
        is_active: bool,
    },
    Error {
        connection_name: String,
        error: String,
    },
    Raw {
        connection_name: String,
        event_type: String,
        data: serde_json::Value,
    },
}

impl ObsPlugin {
    pub fn new(event_tx: mpsc::UnboundedSender<ObsEvent>, log_manager: Arc<Mutex<LogManager>>) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            debug_ws_messages: Arc::new(Mutex::new(true)), // Initialize to true for debugging
            show_full_events: Arc::new(Mutex::new(false)), // Initialize to false - only show recording/streaming by default
            recent_events: Arc::new(Mutex::new(Vec::new())),
            log_manager,
        }
    }

    // Helper method to log messages using the custom LogManager
    async fn log_to_file(&self, level: &str, message: &str) {
        let log_manager = self.log_manager.lock().await;
        if let Err(e) = log_manager.log("obs", level, message) {
            eprintln!("Failed to log to obs.log: {}", e);
        }
    }

    // Add a new OBS connection
    pub async fn add_connection(&self, config: ObsConnectionConfig) -> AppResult<()> {
        log::info!("[PLUGIN_OBS] add_connection called for '{}', enabled={}", config.name, config.enabled);
        {
            let mut connections = self.connections.lock().await;
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
            };
            connections.insert(config.name.clone(), connection);
        } // lock is dropped here

        // Don't automatically connect - let user explicitly connect when ready
        log::info!("[PLUGIN_OBS] '{}' configuration saved. Use connect_obs() to establish connection.", config.name);

        Ok(())
    }

    // Load connections from config manager
    pub async fn load_connections_from_config(&self, config_connections: Vec<crate::config::ObsConnectionConfig>) -> AppResult<()> {
        log::info!("[PLUGIN_OBS] Loading {} connections from config", config_connections.len());
        
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
                log::warn!("[PLUGIN_OBS] Failed to load connection '{}': {}", connection_name, e);
            }
        }
        
        log::info!("[PLUGIN_OBS] Finished loading connections from config");
        Ok(())
    }

    async fn take_pending_request_sender(
        connections: &Arc<Mutex<HashMap<String, ObsConnection>>>,
        connection_name: &str,
        request_id: &str,
    ) -> Option<tokio::sync::oneshot::Sender<serde_json::Value>> {
        let mut conns = connections.lock().await;
        conns.get_mut(connection_name)
            .and_then(|conn| conn.pending_requests.remove(request_id))
    }

    // After successful authentication, spawn the WebSocket receive loop
    async fn spawn_ws_task(&self, connection_name: String) {
        let connections = self.connections.clone();
        let event_tx = self.event_tx.clone();
        let debug_ws_messages = self.debug_ws_messages.clone(); // Clone the flag
        let show_full_events = self.show_full_events.clone(); // Clone the full events flag
        let plugin = self.clone(); // Clone the entire plugin for event storage
        
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
                                                let tx_opt = ObsPlugin::take_pending_request_sender(&connections, &connection_name, request_id).await;
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
                                                        event_type: event_type.to_string(),
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
                                                "StudioModeStateChanged" => {
                                                    if let Some(enabled) = event_data["studioModeEnabled"].as_bool() {
                                                        plugin.log_to_file("INFO", &format!("[OBS-EVENT][{}] Studio mode changed: {}", connection_name, enabled)).await;
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

                                                // === BEGIN: All official OBS v5 event types as stubs ===
                                                "SceneTransitionStarted" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "SceneTransitionEnded" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "SceneTransitionVideoEnded" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputCreated" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputRemoved" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputNameChanged" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputActiveStateChanged" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputShowStateChanged" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputMuteStateChanged" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputVolumeChanged" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputAudioBalanceChanged" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputAudioSyncOffsetChanged" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputAudioTracksChanged" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputAudioMonitorTypeChanged" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "InputVolumeMeters" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "MediaInputPlaybackStarted" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "MediaInputPlaybackEnded" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "MediaInputActionTriggered" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                "VendorEvent" => {
                                                    let _ = event_tx.send(ObsEvent::Raw { connection_name: connection_name.clone(), event_type: event_type.to_string(), data: event_data.clone() });
                                                }
                                                // ... (add all other official event types as needed) ...
                                                // === END: All official OBS v5 event types as stubs ===
                                                _other => {
                                                    let _ = event_tx.send(ObsEvent::Raw {
                                                        connection_name: connection_name.clone(),
                                                        event_type: event_type.to_string(),
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

    // Connect to OBS instance
    pub async fn connect_obs(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[DEBUG] Entered connect_obs for {}", connection_name);
        // Get connection config first
        let config = {
            log::info!("[DEBUG] Attempting to lock connections for config");
            let connections = self.connections.lock().await;
            let connection = connections.get(connection_name)
                .ok_or_else(|| {
                    log::warn!("[DEBUG] Connection '{}' not found in config lookup", connection_name);
                    AppError::ConfigError(format!("Connection '{}' not found", connection_name))
                })?;
            log::info!("[DEBUG] Got config for {}", connection_name);
            connection.config.clone()
        };
        log::info!("[DEBUG] Got config for {}", connection_name);

        // Update status to connecting
        {
            log::info!("[DEBUG] Attempting to lock connections for status update");
            let mut connections = self.connections.lock().await;
            if let Some(connection) = connections.get_mut(connection_name) {
                connection.status = ObsConnectionStatus::Connecting;
            }
        }
        log::info!("[DEBUG] Updated status to Connecting for {}", connection_name);

        // Send status change event
        let _ = self.event_tx.send(ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Connecting,
        });
        log::info!("[DEBUG] Sent status change event for {}", connection_name);

        // Build WebSocket URL
        let ws_url = format!(
            "ws://{}:{}/",
            config.host,
            config.port
        );
        log::info!("[DEBUG] Built ws_url for {}: {}", connection_name, ws_url);

        // Connect to WebSocket
        let (ws_stream, _) = match tokio_tungstenite::connect_async(&ws_url).await {
            Ok(res) => {
                log::info!("[DEBUG] Successfully connected to WebSocket for {}", connection_name);
                log::info!("[OBS] Successfully connected to '{}' at {}:{}", connection_name, config.host, config.port);
                res
            },
            Err(e) => {
                log::error!("[DEBUG] Failed to connect to WebSocket for {}: {}", connection_name, e);
                log::error!("[OBS] Failed to connect to '{}': {}", connection_name, e);
                return Err(AppError::ConfigError(format!("Failed to connect to OBS: {}", e)));
            }
        };
        log::info!("[DEBUG] Got ws_stream for {}", connection_name);

        // Authenticate (v5 only) and get the stream back
        log::info!("[DEBUG] Calling authenticate_v5 for {}", connection_name);
        let ws_stream = self.authenticate_v5(connection_name, ws_stream).await?;
        log::info!("[DEBUG] authenticate_v5 returned for {}", connection_name);

        // Put the stream back in the connection
        {
            let mut connections = self.connections.lock().await;
            let connection = connections.get_mut(connection_name).unwrap();
            connection.websocket = Some(ws_stream);
        }

        // Spawn WebSocket receive loop
        log::info!("[DEBUG] Spawning ws_task for {}", connection_name);
        self.spawn_ws_task(connection_name.to_string()).await;
        log::info!("[DEBUG] spawn_ws_task returned for {}", connection_name);

        // Note: Heartbeat request removed temporarily to fix connection loop
        // TODO: Implement proper heartbeat request after fixing WebSocket sharing

        // Update status to Connected and send event
        {
            let mut connections = self.connections.lock().await;
            if let Some(connection) = connections.get_mut(connection_name) {
                connection.status = ObsConnectionStatus::Connected;
            }
        }
        log::info!("[DEBUG] Updated status to Connected for {}", connection_name);

        // Send status change event
        let _ = self.event_tx.send(ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Connected,
        });
        log::info!("[DEBUG] Sent Connected status change event for {}", connection_name);

        // Enable heartbeat and request initial statuses
        let connection_name_clone = connection_name.to_string();
        let plugin_clone = self.clone();
        tokio::spawn(async move {
            // Wait a bit for the connection to stabilize
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            // Enable heartbeat
            if let Err(e) = plugin_clone.request_heartbeat(&connection_name_clone).await {
                log::warn!("[PLUGIN_OBS] Failed to enable heartbeat for '{}': {}", connection_name_clone, e);
            }
            
            // Request initial statuses
            if let Err(e) = plugin_clone.request_stream_status(&connection_name_clone).await {
                log::warn!("[PLUGIN_OBS] Failed to request stream status for '{}': {}", connection_name_clone, e);
            }
            
            if let Err(e) = plugin_clone.request_record_status(&connection_name_clone).await {
                log::warn!("[PLUGIN_OBS] Failed to request record status for '{}': {}", connection_name_clone, e);
            }
            
            if let Err(e) = plugin_clone.request_replay_buffer_status(&connection_name_clone).await {
                log::warn!("[PLUGIN_OBS] Failed to request replay buffer status for '{}': {}", connection_name_clone, e);
            }
        });

        Ok(())
    }

    // Authenticate using OBS WebSocket v5 protocol
    async fn authenticate_v5(&self, connection_name: &str, ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) -> AppResult<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>> {
        use tokio_tungstenite::tungstenite::Message;
        use sha2::{Digest, Sha256};
        use base64::{engine::general_purpose, Engine as _};
        use serde_json::json;

        log::info!("[OBS] Starting authentication for connection '{}'", connection_name);
        // Set status to Authenticating
        {
            let mut connections = self.connections.lock().await;
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
                let connections = self.connections.lock().await;
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
        // Set status to Authenticated
        {
            let mut connections = self.connections.lock().await;
            let connection = connections.get_mut(connection_name).unwrap();
            connection.status = ObsConnectionStatus::Authenticated;
        }
        log::info!("[OBS] Authentication successful for connection '{}'", connection_name);
        // Send status change event
        let _ = self.event_tx.send(ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Authenticated,
        });
        Ok(ws_stream)
    }

    // Send request to OBS (protocol-agnostic)
    pub async fn send_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        log::info!("[PLUGIN_OBS] send_request called for '{}' with type '{}'", connection_name, request_type);
        
        let mut connections = self.connections.lock().await;
        let connection = connections.get_mut(connection_name)
            .ok_or_else(|| AppError::ConfigError(format!("Connection '{}' not found", connection_name)))?;

        if connection.status != ObsConnectionStatus::Authenticated {
            log::warn!("[PLUGIN_OBS] Connection '{}' not authenticated, status: {:?}", connection_name, connection.status);
            return Err(AppError::ConfigError("OBS connection not authenticated".to_string()));
        }

        let request_id = self.generate_request_id(connection);
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Store pending request
        connection.pending_requests.insert(request_id.clone(), response_tx);

        // Create request based on protocol version
        let request = serde_json::json!({
            "op": 6, // Request opcode
            "d": {
                "requestType": request_type,
                "requestId": request_id,
                "requestData": request_data
            }
        });

        log::info!("[PLUGIN_OBS] Sending request to '{}': {}", connection_name, serde_json::to_string(&request).unwrap_or_default());

        // Create a new WebSocket connection for this request
        let config = &connection.config;
        let ws_url = format!("ws://{}:{}/", config.host, config.port);
        
        let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url).await
            .map_err(|e| AppError::ConfigError(format!("Failed to connect to OBS: {}", e)))?;
        
        // Authenticate the new connection
        let ws_stream = self.authenticate_v5(connection_name, ws_stream).await?;
        let (mut ws_write, _) = ws_stream.split();
        
        // Send the request
        let request_text = serde_json::to_string(&request)
            .map_err(|e| AppError::ConfigError(format!("Failed to serialize request: {}", e)))?;
        
        if let Err(e) = ws_write.send(Message::Text(request_text)).await {
            log::error!("[PLUGIN_OBS] Failed to send request to '{}': {}", connection_name, e);
            return Err(AppError::ConfigError(format!("WebSocket send error: {}", e)));
        }
        
        log::info!("[PLUGIN_OBS] Request sent successfully to '{}'", connection_name);

        // Wait for response with timeout
        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            response_rx
        ).await
        .map_err(|_| AppError::ConfigError("Request timeout".to_string()))?
        .map_err(|_| AppError::ConfigError("Request timeout or connection lost".to_string()))?;

        log::info!("[PLUGIN_OBS] Received response from '{}': {}", connection_name, serde_json::to_string(&response).unwrap_or_default());
        Ok(response)
    }

    // Get current scene
    pub async fn get_current_scene(&self, connection_name: &str) -> AppResult<String> {
        let response = self.send_request(connection_name, "GetCurrentProgramScene", None).await?;
        
        Ok(response["sceneName"]
            .as_str()
            .ok_or_else(|| AppError::ConfigError("Invalid response format".to_string()))
            .map(|s| s.to_string())?)
    }

    // Set current scene
    pub async fn set_current_scene(&self, connection_name: &str, scene_name: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "sceneName": scene_name
        });

        self.send_request(connection_name, "SetCurrentProgramScene", Some(request_data)).await?;
        Ok(())
    }

    // Start recording
    pub async fn start_recording(&self, connection_name: &str) -> AppResult<()> {
        self.send_request(connection_name, "StartRecording", None).await?;
        Ok(())
    }

    // Stop recording
    pub async fn stop_recording(&self, connection_name: &str) -> AppResult<()> {
        self.send_request(connection_name, "StopRecording", None).await?;
        Ok(())
    }

    // Start replay buffer
    pub async fn start_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        self.send_request(connection_name, "StartReplayBuffer", None).await?;
        Ok(())
    }

    // Stop replay buffer
    pub async fn stop_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        self.send_request(connection_name, "StopReplayBuffer", None).await?;
        Ok(())
    }

    // Save replay buffer
    pub async fn save_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        self.send_request(connection_name, "SaveReplayBuffer", None).await?;
        Ok(())
    }

    // Get recording status
    pub async fn get_recording_status(&self, connection_name: &str) -> AppResult<bool> {
        let response = self.send_request(connection_name, "GetRecordingStatus", None).await?;
        
        Ok(response["outputActive"].as_bool().unwrap_or(false))
    }

    // Get streaming status
    pub async fn get_streaming_status(&self, connection_name: &str) -> AppResult<bool> {
        let response = self.send_request(connection_name, "GetStreamStatus", None).await?;
        
        Ok(response["outputActive"].as_bool().unwrap_or(false))
    }

    // Get OBS CPU usage
    pub async fn get_obs_cpu_usage(&self, connection_name: &str) -> AppResult<f64> {
        let response = self.send_request(connection_name, "GetStats", None).await?;
        
        Ok(response["cpuUsage"].as_f64().unwrap_or(0.0))
    }

    // Get replay buffer status
    pub async fn get_replay_buffer_status(&self, connection_name: &str) -> AppResult<bool> {
        let response = self.send_request(connection_name, "GetReplayBufferStatus", None).await?;
        
        Ok(response["outputActive"].as_bool().unwrap_or(false))
    }

    // Get all scenes
    pub async fn get_scenes(&self, connection_name: &str) -> AppResult<Vec<String>> {
        let response = self.send_request(connection_name, "GetSceneList", None).await?;
        
        let scenes = response["scenes"].as_array()
            .ok_or_else(|| AppError::ConfigError("Invalid response format".to_string()))?;
        
        Ok(scenes.iter()
            .filter_map(|scene| scene["sceneName"].as_str())
            .map(|s| s.to_string())
            .collect())
    }

    // Helper methods
    fn generate_request_id(&self, connection: &mut ObsConnection) -> String {
        connection.request_id_counter += 1;
        Uuid::new_v4().to_string()
    }



    // Get connection status
    pub async fn get_connection_status(&self, connection_name: &str) -> Option<ObsConnectionStatus> {
        let connections = self.connections.lock().await;
        connections.get(connection_name).map(|c| c.status.clone())
    }

    // Get all connection names
    pub async fn get_connection_names(&self) -> Vec<String> {
        let connections = self.connections.lock().await;
        connections.keys().cloned().collect()
    }

    // Disconnect OBS connection (close WebSocket but keep configuration)
    pub async fn disconnect_obs(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[PLUGIN_OBS] disconnect_obs called for '{}'", connection_name);
        
        let mut connections = self.connections.lock().await;
        
        if let Some(connection) = connections.get_mut(connection_name) {
            // Close the WebSocket connection if it exists
            if let Some(ws_stream) = connection.websocket.take() {
                let (mut ws_write, _) = ws_stream.split();
                if let Err(e) = ws_write.close().await {
                    log::warn!("[PLUGIN_OBS] Error closing WebSocket for '{}': {}", connection_name, e);
                }
            }
            
            // Update status to Disconnected
            connection.status = ObsConnectionStatus::Disconnected;
            
            // Clear pending requests
            connection.pending_requests.clear();
            
            // Send status change event
            let _ = self.event_tx.send(ObsEvent::ConnectionStatusChanged {
                connection_name: connection_name.to_string(),
                status: ObsConnectionStatus::Disconnected,
            });
            
            log::info!("[PLUGIN_OBS] Successfully disconnected '{}'", connection_name);
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }

    // Remove connection
    pub async fn remove_connection(&self, connection_name: &str) -> AppResult<()> {
        let mut connections = self.connections.lock().await;
        
        if connections.remove(connection_name).is_some() {
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }

    // Get OBS connection roles (OBS_REC, OBS_STR, OBS_SINGLE)
    pub async fn get_connection_roles(&self) -> Vec<(String, String)> {
        let connections = self.connections.lock().await;
        let connection_names: Vec<String> = connections.keys().cloned().collect();
        
        match connection_names.len() {
            0 => vec![],
            1 => vec![(connection_names[0].clone(), "OBS_SINGLE".to_string())],
            _ => {
                // Multiple connections: first is REC, second is STR
                let mut roles = vec![];
                if connection_names.len() >= 1 {
                    roles.push((connection_names[0].clone(), "OBS_REC".to_string()));
                }
                if connection_names.len() >= 2 {
                    roles.push((connection_names[1].clone(), "OBS_STR".to_string()));
                }
                roles
            }
        }
    }

    // Get latest events for debugging
    pub async fn get_latest_events(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        let connections = self.connections.lock().await;
        if let Some(conn) = connections.get(connection_name) {
            Ok(serde_json::json!({
                "connection_name": connection_name,
                "status": format!("{:?}", conn.status),
                "heartbeat_data": conn.heartbeat_data,
                "has_websocket": conn.websocket.is_some(),
                "pending_requests": conn.pending_requests.len()
            }))
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }

    // Request heartbeat messages from OBS
    async fn request_heartbeat(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[PLUGIN_OBS] Requesting heartbeat for '{}'", connection_name);
        
        // Request heartbeat messages with specific event types
        let _heartbeat_request = serde_json::json!({
            "requestType": "SetHeartbeat",
            "requestData": {
                "enable": true
            }
        });
        
        // Send the heartbeat request
        match self.send_request(connection_name, "SetHeartbeat", Some(serde_json::json!({ "enable": true }))).await {
            Ok(_) => {
                log::info!("[PLUGIN_OBS] Successfully enabled heartbeat for '{}'", connection_name);
                Ok(())
            }
            Err(e) => {
                log::warn!("[PLUGIN_OBS] Failed to enable heartbeat for '{}': {}", connection_name, e);
                Err(e)
            }
        }
    }

    // Request stream status from OBS
    async fn request_stream_status(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[PLUGIN_OBS] Requesting stream status for connection: {}", connection_name);
        
        match self.send_request(connection_name, "GetStreamStatus", None).await {
            Ok(_) => {
                log::info!("[PLUGIN_OBS] Successfully requested stream status for '{}'", connection_name);
                Ok(())
            }
            Err(e) => {
                log::warn!("[PLUGIN_OBS] Failed to request stream status for '{}': {}", connection_name, e);
                Err(e)
            }
        }
    }

    // Request record status from OBS
    async fn request_record_status(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[PLUGIN_OBS] Requesting record status for connection: {}", connection_name);
        
        match self.send_request(connection_name, "GetRecordStatus", None).await {
            Ok(_) => {
                log::info!("[PLUGIN_OBS] Successfully requested record status for '{}'", connection_name);
                Ok(())
            }
            Err(e) => {
                log::warn!("[PLUGIN_OBS] Failed to request record status for '{}': {}", connection_name, e);
                Err(e)
            }
        }
    }

    // Request replay buffer status from OBS
    async fn request_replay_buffer_status(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[PLUGIN_OBS] Requesting replay buffer status for connection: {}", connection_name);
        
        match self.send_request(connection_name, "GetReplayBufferStatus", None).await {
            Ok(_) => {
                log::info!("[PLUGIN_OBS] Successfully requested replay buffer status for '{}'", connection_name);
                Ok(())
            }
            Err(e) => {
                log::warn!("[PLUGIN_OBS] Failed to request replay buffer status for '{}': {}", connection_name, e);
                Err(e)
            }
        }
    }

    // Toggle full OBS events display
    pub async fn toggle_full_events(&self, enabled: bool) {
        let mut show_full = self.show_full_events.lock().await;
        *show_full = enabled;
        log::info!("[PLUGIN_OBS] Full OBS events display: {}", if enabled { "enabled" } else { "disabled" });
    }

    // Get current full events setting
    pub async fn get_full_events_setting(&self) -> bool {
        let show_full = self.show_full_events.lock().await;
        *show_full
    }

    // Store recent event for frontend polling
    async fn store_recent_event(&self, connection_name: String, event_type: String, data: serde_json::Value) {
        let mut events = self.recent_events.lock().await;
        
        // Add new event
        events.push(RecentEvent {
            connection_name,
            event_type,
            data,
            timestamp: chrono::Utc::now(),
        });
        
        // Keep only last 100 events
        if events.len() > 100 {
            events.remove(0);
        }
    }

    // Get recent events for frontend polling
    pub async fn get_recent_events(&self) -> Vec<RecentEvent> {
        let events = self.recent_events.lock().await;
        events.clone()
    }

    // Get comprehensive OBS status for status bar
    pub async fn get_obs_status(&self) -> AppResult<ObsStatusInfo> {
        log::info!("[PLUGIN_OBS] get_obs_status called");
        let roles = self.get_connection_roles().await;
        log::info!("[PLUGIN_OBS] Found {} connection roles: {:?}", roles.len(), roles);
        
        // Debug: Show all connections and their heartbeat data
        {
            let connections = self.connections.lock().await;
            for (name, conn) in connections.iter() {
                log::info!("[PLUGIN_OBS] Connection '{}': status={:?}, heartbeat_data={:?}", 
                    name, conn.status, conn.heartbeat_data);
            }
        }
        
        let mut status = ObsStatusInfo {
            is_recording: false,
            is_streaming: false,
            cpu_usage: 0.0,
            recording_connection: None,
            streaming_connection: None,
        };

        for (connection_name, role) in roles {
            log::info!("[PLUGIN_OBS] Processing connection '{}' with role '{}'", connection_name, role);
            
            // Check if this connection is actually connected
            let connection_status = self.get_connection_status(&connection_name).await;
            let is_connected = matches!(connection_status, Some(ObsConnectionStatus::Connected));
            log::info!("[PLUGIN_OBS] Connection '{}' status: {:?}, is_connected: {}", connection_name, connection_status, is_connected);
            
            // Get heartbeat data if available
            let heartbeat_data = {
                let connections = self.connections.lock().await;
                connections.get(&connection_name)
                    .and_then(|conn| conn.heartbeat_data.clone())
            };
            
            if let Some(heartbeat) = heartbeat_data {
                log::info!("[PLUGIN_OBS] Using heartbeat data for '{}': {:?}", connection_name, heartbeat);
                
                // Extract data from heartbeat
                if let Some(recording) = heartbeat["recording"].as_bool() {
                    status.is_recording = recording;
                    log::info!("[PLUGIN_OBS] Heartbeat recording status for '{}': {}", connection_name, recording);
                }
                if let Some(streaming) = heartbeat["streaming"].as_bool() {
                    status.is_streaming = streaming;
                    log::info!("[PLUGIN_OBS] Heartbeat streaming status for '{}': {}", connection_name, streaming);
                }
                if let Some(cpu_usage) = heartbeat["cpuUsage"].as_f64() {
                    status.cpu_usage = cpu_usage;
                    log::info!("[PLUGIN_OBS] Heartbeat CPU usage for '{}': {}", connection_name, cpu_usage);
                } else {
                    // CPU monitoring moved to separate Diagnostics module
                    status.cpu_usage = 0.0; // Will be handled by CPU monitoring module
                    log::info!("[PLUGIN_OBS] CPU monitoring moved to Diagnostics module for '{}'", connection_name);
                }
                
                // Set connection names based on role
                match role.as_str() {
                    "OBS_SINGLE" => {
                        if status.is_recording {
                            status.recording_connection = Some(connection_name.clone());
                        } else if is_connected {
                            status.recording_connection = Some(format!("{} (Connected)", connection_name));
                        }
                        if status.is_streaming {
                            status.streaming_connection = Some(connection_name.clone());
                        } else if is_connected {
                            status.streaming_connection = Some(format!("{} (Connected)", connection_name));
                        }
                    }
                    "OBS_REC" => {
                        if status.is_recording {
                            status.recording_connection = Some(connection_name.clone());
                        } else if is_connected {
                            status.recording_connection = Some(format!("{} (Connected)", connection_name));
                        }
                    }
                    "OBS_STR" => {
                        if status.is_streaming {
                            status.streaming_connection = Some(connection_name.clone());
                        } else if is_connected {
                            status.streaming_connection = Some(format!("{} (Connected)", connection_name));
                        }
                    }
                    _ => {
                        log::warn!("[PLUGIN_OBS] Unknown role '{}' for connection '{}'", role, connection_name);
                    }
                }
            } else {
                log::info!("[PLUGIN_OBS] No heartbeat data available for '{}', using fallback requests", connection_name);
                
                // Fallback to individual requests if no heartbeat data
                match role.as_str() {
                    "OBS_SINGLE" => {
                        if let Ok(is_recording) = self.get_recording_status(&connection_name).await {
                            status.is_recording = is_recording;
                            if is_recording {
                                status.recording_connection = Some(connection_name.clone());
                            } else if is_connected {
                                status.recording_connection = Some(format!("{} (Connected)", connection_name));
                            }
                        }
                        if let Ok(is_streaming) = self.get_streaming_status(&connection_name).await {
                            status.is_streaming = is_streaming;
                            if is_streaming {
                                status.streaming_connection = Some(connection_name.clone());
                            } else if is_connected {
                                status.streaming_connection = Some(format!("{} (Connected)", connection_name));
                            }
                        }
                        // CPU monitoring moved to separate Diagnostics module
                        status.cpu_usage = 0.0; // Will be handled by CPU monitoring module
                    }
                    "OBS_REC" => {
                        if let Ok(is_recording) = self.get_recording_status(&connection_name).await {
                            status.is_recording = is_recording;
                            if is_recording {
                                status.recording_connection = Some(connection_name.clone());
                            } else if is_connected {
                                status.recording_connection = Some(format!("{} (Connected)", connection_name));
                            }
                        }
                        // CPU monitoring moved to separate Diagnostics module
                        status.cpu_usage = 0.0; // Will be handled by CPU monitoring module
                    }
                    "OBS_STR" => {
                        if let Ok(is_streaming) = self.get_streaming_status(&connection_name).await {
                            status.is_streaming = is_streaming;
                            if is_streaming {
                                status.streaming_connection = Some(connection_name.clone());
                            } else if is_connected {
                                status.streaming_connection = Some(format!("{} (Connected)", connection_name));
                            }
                        }
                    }
                    _ => {
                        log::warn!("[PLUGIN_OBS] Unknown role '{}' for connection '{}'", role, connection_name);
                    }
                }
            }
        }

        log::info!("[PLUGIN_OBS] Final status: {:?}", status);
        Ok(status)
    }

    // Emit OBS event to frontend via Tauri
    pub async fn emit_event_to_frontend(&self, event: ObsEvent) {
        // Convert ObsEvent to JSON
        let event_json = match event {
            ObsEvent::ConnectionStatusChanged { connection_name, status } => {
                serde_json::json!({
                    "eventType": "ConnectionStatusChanged",
                    "connection_name": connection_name,
                    "status": match status {
                        ObsConnectionStatus::Disconnected => "Disconnected",
                        ObsConnectionStatus::Connecting => "Connecting",
                        ObsConnectionStatus::Connected => "Connected",
                        ObsConnectionStatus::Authenticating => "Authenticating",
                        ObsConnectionStatus::Authenticated => "Authenticated",
                        ObsConnectionStatus::Error(_e) => "Error",
                    },
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            },
            ObsEvent::SceneChanged { connection_name, scene_name } => {
                serde_json::json!({
                    "eventType": "SceneChanged",
                    "connection_name": connection_name,
                    "scene_name": scene_name,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            },
            ObsEvent::RecordingStateChanged { connection_name, is_recording } => {
                serde_json::json!({
                    "eventType": "RecordingStateChanged",
                    "connection_name": connection_name,
                    "is_recording": is_recording,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            },
            ObsEvent::StreamStateChanged { connection_name, is_streaming } => {
                serde_json::json!({
                    "eventType": "StreamStateChanged",
                    "connection_name": connection_name,
                    "is_streaming": is_streaming,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            },
            ObsEvent::ReplayBufferStateChanged { connection_name, is_active } => {
                serde_json::json!({
                    "eventType": "ReplayBufferStateChanged",
                    "connection_name": connection_name,
                    "is_active": is_active,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            },
            ObsEvent::Error { connection_name, error } => {
                serde_json::json!({
                    "eventType": "Error",
                    "connection_name": connection_name,
                    "error": error,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            },
            ObsEvent::Raw { connection_name, event_type, data } => {
                serde_json::json!({
                    "eventType": "Raw",
                    "connection_name": connection_name,
                    "obs_event_type": event_type,
                    "data": data,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            },
        };

        // For now, we'll log the event. In a full implementation, we'd emit it via Tauri
        log::info!("[PLUGIN_OBS] Event to frontend: {}", serde_json::to_string(&event_json).unwrap_or_default());
        
        // TODO: Emit via Tauri event system
        // This would require access to the Tauri app handle, which is complex from within the plugin
        // For now, we'll use the existing event_tx channel and handle it in the main app
    }
}

// OBS Status Information for Status Bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsStatusInfo {
    pub is_recording: bool,
    pub is_streaming: bool,
    pub cpu_usage: f64,
    pub recording_connection: Option<String>,
    pub streaming_connection: Option<String>,
}

// Legacy function for backward compatibility
pub fn connect_obs() {
    log::info!("OBS WebSocket plugin initialized with dual-protocol support");
    log::info!("Use ObsPlugin::new() to create a plugin instance");
}
