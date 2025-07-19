use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::types::{AppError, AppResult};
use tokio_tungstenite::tungstenite::Message;
use futures_util::StreamExt;
use futures_util::SinkExt;

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
}

// OBS Plugin Manager
pub struct ObsPlugin {
    connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    event_tx: mpsc::UnboundedSender<ObsEvent>,
    pub debug_ws_messages: Arc<Mutex<bool>>, // Add this line
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
    pub fn new(event_tx: mpsc::UnboundedSender<ObsEvent>) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            debug_ws_messages: Arc::new(Mutex::new(false)), // Initialize to false
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
            };
            connections.insert(config.name.clone(), connection);
        } // lock is dropped here

        // Don't automatically connect - let user explicitly connect when ready
        log::info!("[PLUGIN_OBS] '{}' configuration saved. Use connect_obs() to establish connection.", config.name);

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
        tokio::spawn(async move {
            loop {
                let ws_stream_opt = {
                    let mut conns = connections.lock().await;
                    conns.get_mut(&connection_name)
                        .and_then(|conn| conn.websocket.take())
                };
                if let Some(ws_stream) = ws_stream_opt {
                    let (_ws_write, mut ws_read) = ws_stream.split();
                    while let Some(msg_result) = ws_read.next().await {
                        // Log all incoming messages if debug_ws_messages is enabled
                        let flag = debug_ws_messages.lock().await;
                        if *flag {
                            match &msg_result {
                                Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                    println!("[WS-DEBUG][{}] Text: {}", connection_name, text);
                                },
                                Ok(tokio_tungstenite::tungstenite::Message::Binary(bin)) => {
                                    println!("[WS-DEBUG][{}] Binary: {:02X?}", connection_name, bin);
                                },
                                Ok(other) => {
                                    println!("[WS-DEBUG][{}] Other: {:?}", connection_name, other);
                                },
                                Err(e) => {
                                    println!("[WS-DEBUG][{}] Error: {}", connection_name, e);
                                }
                            }
                        }
                        match msg_result {
                            Ok(Message::Text(text)) => {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(request_id) = json.pointer("/d/requestId").and_then(|v| v.as_str()) {
                                        let tx_opt = ObsPlugin::take_pending_request_sender(&connections, &connection_name, request_id).await;
                                        if let Some(tx) = tx_opt {
                                            let _ = tx.send(json["d"].clone());
                                        }
                                    } else if let Some(op) = json["op"].as_u64() {
                                        if op == 5 {
                                            // Full event parsing
                                            let event_type = json.pointer("/d/eventType").and_then(|v| v.as_str()).unwrap_or("");
                                            let event_data = &json["d"]["eventData"];
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
                                                        let _ = event_tx.send(ObsEvent::RecordingStateChanged {
                                                            connection_name: connection_name.clone(),
                                                            is_recording,
                                                        });
                                                    }
                                                }
                                                "StreamStateChanged" => {
                                                    if let Some(is_streaming) = event_data["outputActive"].as_bool() {
                                                        let _ = event_tx.send(ObsEvent::StreamStateChanged {
                                                            connection_name: connection_name.clone(),
                                                            is_streaming,
                                                        });
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
        println!("[DEBUG] Entered connect_obs for {}", connection_name);
        // Get connection config first
        let config = {
            println!("[DEBUG] Attempting to lock connections for config");
            let connections = self.connections.lock().await;
            let connection = connections.get(connection_name)
                .ok_or_else(|| {
                    println!("[DEBUG] Connection '{}' not found in config lookup", connection_name);
                    AppError::ConfigError(format!("Connection '{}' not found", connection_name))
                })?;
            println!("[DEBUG] Got config for {}", connection_name);
            connection.config.clone()
        };
        println!("[DEBUG] Got config for {}", connection_name);

        // Update status to connecting
        {
            println!("[DEBUG] Attempting to lock connections for status update");
            let mut connections = self.connections.lock().await;
            if let Some(connection) = connections.get_mut(connection_name) {
                connection.status = ObsConnectionStatus::Connecting;
            }
        }
        println!("[DEBUG] Updated status to Connecting for {}", connection_name);

        // Send status change event
        let _ = self.event_tx.send(ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Connecting,
        });
        println!("[DEBUG] Sent status change event for {}", connection_name);

        // Build WebSocket URL
        let ws_url = format!(
            "ws://{}:{}/",
            config.host,
            config.port
        );
        println!("[DEBUG] Built ws_url for {}: {}", connection_name, ws_url);

        // Connect to WebSocket
        let (ws_stream, _) = match tokio_tungstenite::connect_async(&ws_url).await {
            Ok(res) => {
                println!("[DEBUG] Successfully connected to WebSocket for {}", connection_name);
                log::info!("[OBS] Successfully connected to '{}' at {}:{}", connection_name, config.host, config.port);
                res
            },
            Err(e) => {
                println!("[DEBUG] Failed to connect to WebSocket for {}: {}", connection_name, e);
                log::error!("[OBS] Failed to connect to '{}': {}", connection_name, e);
                return Err(AppError::ConfigError(format!("Failed to connect to OBS: {}", e)));
            }
        };
        println!("[DEBUG] Got ws_stream for {}", connection_name);

        // Authenticate (v5 only) and get the stream back
        println!("[DEBUG] Calling authenticate_v5 for {}", connection_name);
        let ws_stream = self.authenticate_v5(connection_name, ws_stream).await?;
        println!("[DEBUG] authenticate_v5 returned for {}", connection_name);

        // Put the stream back in the connection
        {
            let mut connections = self.connections.lock().await;
            let connection = connections.get_mut(connection_name).unwrap();
            connection.websocket = Some(ws_stream);
        }

        // Spawn WebSocket receive loop
        println!("[DEBUG] Spawning ws_task for {}", connection_name);
        self.spawn_ws_task(connection_name.to_string()).await;
        println!("[DEBUG] spawn_ws_task returned for {}", connection_name);

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
        println!("[AUTH-DEBUG] Waiting for Hello message...");
        let hello_msg = match ws_read.next().await {
            Some(Ok(msg)) => msg,
            Some(Err(e)) => return Err(AppError::ConfigError(format!("WebSocket error: {}", e))),
            None => return Err(AppError::ConfigError("No Hello message from OBS".to_string())),
        };
        let hello_json: serde_json::Value = match &hello_msg {
            Message::Text(text) => {
                println!("[AUTH-DEBUG] Received Hello message: {}", text);
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
                println!("[AUTH-DEBUG] Using password: '{}' (length: {})", if password.is_empty() { "<empty>" } else { "***" }, password.len());
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
        println!("[AUTH-DEBUG] Sending Identify message: {}", identify_str);
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
                    println!("[AUTH-DEBUG] Timeout waiting for response, attempt {}/{}", timeout_counter, MAX_TIMEOUT);
                    continue;
                }
            };
            
            if let Message::Text(text) = &msg {
                println!("[AUTH-DEBUG] Received message in handshake loop: {}", text);
                let json: serde_json::Value = serde_json::from_str(text).map_err(|e| AppError::ConfigError(format!("Invalid JSON after Identify: {}", e)))?;
                let op = json["op"].as_u64().unwrap_or(0);
                println!("[AUTH-DEBUG] Message opcode: {}", op);
                
                if op == 2 {
                    // Identified
                    println!("[AUTH-DEBUG] Authentication successful - received Identified message");
                    break;
                } else if op == 8 {
                    // Error
                    let reason = json["d"]["reason"].as_str().unwrap_or("Unknown error");
                    println!("[AUTH-DEBUG] Authentication failed - received error: {}", reason);
                    return Err(AppError::ConfigError(format!("OBS authentication failed: {}", reason)));
                } else {
                    println!("[AUTH-DEBUG] Unexpected opcode: {}, continuing to wait...", op);
                }
            } else {
                println!("[AUTH-DEBUG] Received non-text message: {:?}", msg);
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
        let mut connections = self.connections.lock().await;
        let connection = connections.get_mut(connection_name)
            .ok_or_else(|| AppError::ConfigError(format!("Connection '{}' not found", connection_name)))?;

        if connection.status != ObsConnectionStatus::Authenticated {
            return Err(AppError::ConfigError("OBS connection not authenticated".to_string()));
        }

        let request_id = self.generate_request_id(connection);
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Store pending request
        connection.pending_requests.insert(request_id.clone(), response_tx);

        // Create request based on protocol version
        let _request = serde_json::json!({
            "op": 6, // Request opcode
            "d": {
                "requestType": request_type,
                "requestId": request_id,
                "requestData": request_data
            }
        });

        // Send request via WebSocket
        // Implementation would send the request through the WebSocket connection

        // Wait for response
        let response = response_rx.await
            .map_err(|_| AppError::ConfigError("Request timeout or connection lost".to_string()))?;

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

    // Get comprehensive OBS status for status bar
    pub async fn get_obs_status(&self) -> AppResult<ObsStatusInfo> {
        let roles = self.get_connection_roles().await;
        
        let mut status = ObsStatusInfo {
            is_recording: false,
            is_streaming: false,
            cpu_usage: 0.0,
            recording_connection: None,
            streaming_connection: None,
        };

        for (connection_name, role) in roles {
            match role.as_str() {
                "OBS_SINGLE" => {
                    // Single connection handles both recording and streaming
                    if let Ok(is_recording) = self.get_recording_status(&connection_name).await {
                        status.is_recording = is_recording;
                        if is_recording {
                            status.recording_connection = Some(connection_name.clone());
                        }
                    }
                    
                    if let Ok(is_streaming) = self.get_streaming_status(&connection_name).await {
                        status.is_streaming = is_streaming;
                        if is_streaming {
                            status.streaming_connection = Some(connection_name.clone());
                        }
                    }
                    
                    if let Ok(cpu_usage) = self.get_obs_cpu_usage(&connection_name).await {
                        status.cpu_usage = cpu_usage;
                    }
                }
                "OBS_REC" => {
                    // Recording connection
                    if let Ok(is_recording) = self.get_recording_status(&connection_name).await {
                        status.is_recording = is_recording;
                        if is_recording {
                            status.recording_connection = Some(connection_name.clone());
                        }
                    }
                    
                    if let Ok(cpu_usage) = self.get_obs_cpu_usage(&connection_name).await {
                        status.cpu_usage = cpu_usage;
                    }
                }
                "OBS_STR" => {
                    // Streaming connection
                    if let Ok(is_streaming) = self.get_streaming_status(&connection_name).await {
                        status.is_streaming = is_streaming;
                        if is_streaming {
                            status.streaming_connection = Some(connection_name.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(status)
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
    println!("OBS WebSocket plugin initialized with dual-protocol support");
    println!("Use ObsPlugin::new() to create a plugin instance");
}
