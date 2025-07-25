use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// OBS WebSocket Protocol Versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObsWebSocketVersion {
    V4,
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
    pub async fn add_connection(&self, config: ObsConnectionConfig) -> Result<(), String> {
        let mut connections = self.connections.lock().unwrap();
        
        if connections.contains_key(&config.name) {
            return Err(format!("Connection '{}' already exists", config.name));
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
            self.connect_obs(&config.name).await?;
        }

        Ok(())
    }

    // Connect to OBS instance
    pub async fn connect_obs(&self, connection_name: &str) -> Result<(), String> {
        // Get connection config first
        let config = {
            let connections = self.connections.lock().unwrap();
            let connection = connections.get(connection_name)
                .ok_or_else(|| format!("Connection '{}' not found", connection_name))?;
            connection.config.clone()
        };

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
        let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .map_err(|e| format!("Failed to connect to OBS: {}", e))?;

        // Update connection
        let mut connections = self.connections.lock().unwrap();
        let connection = connections.get_mut(connection_name).unwrap();
        connection.websocket = Some(ws_stream);
        connection.status = ObsConnectionStatus::Connected;

        // Handle protocol-specific authentication
        match config.protocol_version {
            ObsWebSocketVersion::V4 => {
                self.authenticate_v4(connection_name).await?;
            }
            ObsWebSocketVersion::V5 => {
                self.authenticate_v5(connection_name).await?;
            }
        }

        Ok(())
    }

    // Authenticate using OBS WebSocket v4 protocol
    async fn authenticate_v4(&self, connection_name: &str) -> Result<(), String> {
        let mut connections = self.connections.lock().unwrap();
        let connection = connections.get_mut(connection_name).unwrap();
        
        connection.status = ObsConnectionStatus::Authenticating;

        // V4 authentication is simpler - just send password if required
        if let Some(_password) = &connection.config.password {
            let _auth_request = serde_json::json!({
                "request-type": "GetAuthRequired",
                "message-id": self.generate_request_id(connection)
            });

            // Send authentication request
            // Implementation would send the request and handle response
        }

        connection.status = ObsConnectionStatus::Authenticated;
        drop(connections);

        // Send status change event
        let _ = self.event_tx.send(ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Authenticated,
        });

        Ok(())
    }

    // Authenticate using OBS WebSocket v5 protocol
    async fn authenticate_v5(&self, connection_name: &str) -> Result<(), String> {
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
    ) -> Result<serde_json::Value, String> {
        let mut connections = self.connections.lock().unwrap();
        let connection = connections.get_mut(connection_name)
            .ok_or_else(|| format!("Connection '{}' not found", connection_name))?;

        if connection.status != ObsConnectionStatus::Authenticated {
            return Err("OBS connection not authenticated".to_string());
        }

        let request_id = self.generate_request_id(connection);
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Store pending request
        connection.pending_requests.insert(request_id.clone(), response_tx);

        // Create request based on protocol version
        let request = match connection.config.protocol_version {
            ObsWebSocketVersion::V4 => {
                serde_json::json!({
                    "request-type": request_type,
                    "message-id": request_id,
                    "request-data": request_data
                })
            }
            ObsWebSocketVersion::V5 => {
                serde_json::json!({
                    "op": 6, // Request opcode
                    "d": {
                        "requestType": request_type,
                        "requestId": request_id,
                        "requestData": request_data
                    }
                })
            }
        };

        // Send request via WebSocket
        // Implementation would send the request through the WebSocket connection

        // Wait for response
        let response = response_rx.await
            .map_err(|_| "Request timeout or connection lost".to_string())?;

        Ok(response)
    }

    // Get current scene
    pub async fn get_current_scene(&self, connection_name: &str) -> Result<String, String> {
        let request_type = match self.get_protocol_version(connection_name)? {
            ObsWebSocketVersion::V4 => "GetCurrentScene",
            ObsWebSocketVersion::V5 => "GetCurrentProgramScene",
        };

        let response = self.send_request(connection_name, request_type, None).await?;
        
        match self.get_protocol_version(connection_name)? {
            ObsWebSocketVersion::V4 => {
                response["scene-name"]
                    .as_str()
                    .ok_or_else(|| "Invalid response format".to_string())
                    .map(|s| s.to_string())
            }
            ObsWebSocketVersion::V5 => {
                response["sceneName"]
                    .as_str()
                    .ok_or_else(|| "Invalid response format".to_string())
                    .map(|s| s.to_string())
            }
        }
    }

    // Set current scene
    pub async fn set_current_scene(&self, connection_name: &str, scene_name: &str) -> Result<(), String> {
        let request_type = match self.get_protocol_version(connection_name)? {
            ObsWebSocketVersion::V4 => "SetCurrentScene",
            ObsWebSocketVersion::V5 => "SetCurrentProgramScene",
        };

        let request_data = match self.get_protocol_version(connection_name)? {
            ObsWebSocketVersion::V4 => {
                serde_json::json!({
                    "scene-name": scene_name
                })
            }
            ObsWebSocketVersion::V5 => {
                serde_json::json!({
                    "sceneName": scene_name
                })
            }
        };

        self.send_request(connection_name, request_type, Some(request_data)).await?;
        Ok(())
    }

    // Start recording
    pub async fn start_recording(&self, connection_name: &str) -> Result<(), String> {
        self.send_request(connection_name, "StartRecording", None).await?;
        Ok(())
    }

    // Stop recording
    pub async fn stop_recording(&self, connection_name: &str) -> Result<(), String> {
        self.send_request(connection_name, "StopRecording", None).await?;
        Ok(())
    }

    // Start replay buffer
    pub async fn start_replay_buffer(&self, connection_name: &str) -> Result<(), String> {
        self.send_request(connection_name, "StartReplayBuffer", None).await?;
        Ok(())
    }

    // Stop replay buffer
    pub async fn stop_replay_buffer(&self, connection_name: &str) -> Result<(), String> {
        self.send_request(connection_name, "StopReplayBuffer", None).await?;
        Ok(())
    }

    // Save replay buffer
    pub async fn save_replay_buffer(&self, connection_name: &str) -> Result<(), String> {
        self.send_request(connection_name, "SaveReplayBuffer", None).await?;
        Ok(())
    }

    // Get recording status
    pub async fn get_recording_status(&self, connection_name: &str) -> Result<bool, String> {
        let response = self.send_request(connection_name, "GetRecordingStatus", None).await?;
        
        match self.get_protocol_version(connection_name)? {
            ObsWebSocketVersion::V4 => {
                Ok(response["is-recording"].as_bool().unwrap_or(false))
            }
            ObsWebSocketVersion::V5 => {
                Ok(response["outputActive"].as_bool().unwrap_or(false))
            }
        }
    }

    // Get replay buffer status
    pub async fn get_replay_buffer_status(&self, connection_name: &str) -> Result<bool, String> {
        let response = self.send_request(connection_name, "GetReplayBufferStatus", None).await?;
        
        match self.get_protocol_version(connection_name)? {
            ObsWebSocketVersion::V4 => {
                Ok(response["is-replay-buffer-active"].as_bool().unwrap_or(false))
            }
            ObsWebSocketVersion::V5 => {
                Ok(response["outputActive"].as_bool().unwrap_or(false))
            }
        }
    }

    // Get all scenes
    pub async fn get_scenes(&self, connection_name: &str) -> Result<Vec<String>, String> {
        let response = self.send_request(connection_name, "GetSceneList", None).await?;
        
        match self.get_protocol_version(connection_name)? {
            ObsWebSocketVersion::V4 => {
                let scenes = response["scenes"].as_array()
                    .ok_or_else(|| "Invalid response format".to_string())?;
                
                Ok(scenes.iter()
                    .filter_map(|scene| scene["scene-name"].as_str())
                    .map(|s| s.to_string())
                    .collect())
            }
            ObsWebSocketVersion::V5 => {
                let scenes = response["scenes"].as_array()
                    .ok_or_else(|| "Invalid response format".to_string())?;
                
                Ok(scenes.iter()
                    .filter_map(|scene| scene["sceneName"].as_str())
                    .map(|s| s.to_string())
                    .collect())
            }
        }
    }

    // Helper methods
    fn generate_request_id(&self, connection: &mut ObsConnection) -> String {
        connection.request_id_counter += 1;
        Uuid::new_v4().to_string()
    }

    fn get_protocol_version(&self, connection_name: &str) -> Result<ObsWebSocketVersion, String> {
        let connections = self.connections.lock().unwrap();
        let connection = connections.get(connection_name)
            .ok_or_else(|| format!("Connection '{}' not found", connection_name))?;
        
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
    pub fn remove_connection(&self, connection_name: &str) -> Result<(), String> {
        let mut connections = self.connections.lock().unwrap();
        
        if connections.remove(connection_name).is_some() {
            Ok(())
        } else {
            Err(format!("Connection '{}' not found", connection_name))
        }
    }
}

// Legacy function for backward compatibility
pub fn connect_obs() {
    println!("OBS WebSocket plugin initialized with dual-protocol support");
    println!("Use ObsPlugin::new() to create a plugin instance");
}
