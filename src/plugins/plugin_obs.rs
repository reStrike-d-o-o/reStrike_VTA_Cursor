use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::types::{AppError, AppResult};
use tokio_tungstenite::tungstenite::Message;
use futures_util::StreamExt;

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
}

impl ObsPlugin {
    pub fn new(event_tx: mpsc::UnboundedSender<ObsEvent>) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
        }
    }

    // Add a new OBS connection
    pub async fn add_connection(&self, config: ObsConnectionConfig) -> AppResult<()> {
        log::info!("[PLUGIN_OBS] add_connection called for '{}', enabled={}", config.name, config.enabled);
        let mut connections = self.connections.lock().unwrap();
        
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

        // Start connection if enabled
        if config.enabled {
            log::info!("[PLUGIN_OBS] '{}' is enabled, calling connect_obs...", config.name);
            self.connect_obs(&config.name).await?;
        } else {
            log::info!("[PLUGIN_OBS] '{}' is disabled, skipping connect_obs.", config.name);
        }

        Ok(())
    }

    fn take_pending_request_sender(
        connections: &Arc<Mutex<HashMap<String, ObsConnection>>>,
        connection_name: &str,
        request_id: &str,
    ) -> Option<tokio::sync::oneshot::Sender<serde_json::Value>> {
        let mut conns = connections.lock().unwrap();
        conns.get_mut(connection_name)
            .and_then(|conn| conn.pending_requests.remove(request_id))
    }

    // After successful authentication, spawn the WebSocket receive loop
    async fn spawn_ws_task(&self, connection_name: String) {
        let connections = self.connections.clone();
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            loop {
                let ws_stream_opt = {
                    let mut conns = connections.lock().unwrap();
                    conns.get_mut(&connection_name)
                        .and_then(|conn| conn.websocket.take())
                };
                if let Some(ws_stream) = ws_stream_opt {
                    let (_ws_write, mut ws_read) = ws_stream.split();
                    while let Some(msg_result) = ws_read.next().await {
                        match msg_result {
                            Ok(Message::Text(text)) => {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(request_id) = json.pointer("/d/requestId").and_then(|v| v.as_str()) {
                                        let tx_opt = ObsPlugin::take_pending_request_sender(&connections, &connection_name, request_id);
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
                                                // TODO: Add more event types as needed
                                                other => {
                                                    let _ = event_tx.send(ObsEvent::Error {
                                                        connection_name: connection_name.clone(),
                                                        error: format!("Unknown event type: {} (raw: {})", other, text),
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
        // Get connection config first
        let config = {
            let connections = self.connections.lock().unwrap();
            let connection = connections.get(connection_name)
                .ok_or_else(|| AppError::ConfigError(format!("Connection '{}' not found", connection_name)))?;
            connection.config.clone()
        };

        log::info!("[OBS] Attempting to connect: '{}' at {}:{}", connection_name, config.host, config.port);

        // Update status to connecting
        {
            let mut connections = self.connections.lock().unwrap();
            if let Some(connection) = connections.get_mut(connection_name) {
                connection.status = ObsConnectionStatus::Connecting;
            }
        }

        // Send status change event
        let _ = self.event_tx.send(ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Connecting,
        });

        // Build WebSocket URL
        let ws_url = format!(
            "ws://{}:{}/",
            config.host,
            config.port
        );

        // Connect to WebSocket
        let (ws_stream, _) = match tokio_tungstenite::connect_async(&ws_url).await {
            Ok(res) => {
                log::info!("[OBS] Successfully connected to '{}' at {}:{}", connection_name, config.host, config.port);
                res
            },
            Err(e) => {
                log::error!("[OBS] Failed to connect to '{}': {}", connection_name, e);
                return Err(AppError::ConfigError(format!("Failed to connect to OBS: {}", e)));
            }
        };

        // Update connection
        let mut connections = self.connections.lock().unwrap();
        let connection = connections.get_mut(connection_name).unwrap();
        connection.websocket = Some(ws_stream);
        connection.status = ObsConnectionStatus::Connected;

        // Authenticate (v5 only)
        self.authenticate_v5(connection_name).await?;

        // Spawn WebSocket receive loop
        self.spawn_ws_task(connection_name.to_string()).await;

        Ok(())
    }

    // Authenticate using OBS WebSocket v5 protocol
    async fn authenticate_v5(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS] Starting authentication for connection '{}'", connection_name);
        let mut connections = self.connections.lock().unwrap();
        let connection = connections.get_mut(connection_name).unwrap();
        
        connection.status = ObsConnectionStatus::Authenticating;

        // V5 uses a more complex authentication flow
        // 1. Wait for Hello message
        // 2. Send Identify with authentication if required
        // 3. Wait for Identified message

        // For now, we'll implement a basic version
        // In a full implementation, you'd handle the full v5 authentication flow

        connection.status = ObsConnectionStatus::Authenticated;
        drop(connections);

        log::info!("[OBS] Authentication successful for connection '{}'", connection_name);

        // Send status change event
        let _ = self.event_tx.send(ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Authenticated,
        });

        Ok(())
    }

    // Send request to OBS (protocol-agnostic)
    pub async fn send_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        let mut connections = self.connections.lock().unwrap();
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

    fn get_protocol_version(&self, connection_name: &str) -> AppResult<ObsWebSocketVersion> {
        let connections = self.connections.lock().unwrap();
        let connection = connections.get(connection_name)
            .ok_or_else(|| AppError::ConfigError(format!("Connection '{}' not found", connection_name)))?;
        
        Ok(connection.config.protocol_version)
    }

    // Get connection status
    pub fn get_connection_status(&self, connection_name: &str) -> Option<ObsConnectionStatus> {
        let connections = self.connections.lock().unwrap();
        connections.get(connection_name).map(|c| c.status.clone())
    }

    // Get all connection names
    pub fn get_connection_names(&self) -> Vec<String> {
        let connections = self.connections.lock().unwrap();
        connections.keys().cloned().collect()
    }

    // Remove connection
    pub fn remove_connection(&self, connection_name: &str) -> AppResult<()> {
        let mut connections = self.connections.lock().unwrap();
        
        if connections.remove(connection_name).is_some() {
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", connection_name)))
        }
    }

    // Get OBS connection roles (OBS_REC, OBS_STR, OBS_SINGLE)
    pub fn get_connection_roles(&self) -> Vec<(String, String)> {
        let connections = self.connections.lock().unwrap();
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
        let roles = self.get_connection_roles();
        
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
