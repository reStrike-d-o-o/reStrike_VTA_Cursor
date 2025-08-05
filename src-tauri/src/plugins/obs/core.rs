// OBS Core Plugin
// Handles connection management and WebSocket infrastructure
// Extracted from the original plugin_obs.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use serde_json;
use crate::types::{AppError, AppResult};
use crate::logging::LogManager;
use super::types::*;

/// Core OBS Plugin for connection management
pub struct ObsCorePlugin {
    context: ObsPluginContext,
}

impl ObsCorePlugin {
    /// Create a new OBS Core Plugin
    pub fn new(context: ObsPluginContext) -> Self {
        Self { context }
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

        // Emit status change event
        let status_event = ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Connecting,
        };
        if let Err(e) = self.context.event_tx.send(status_event) {
            log::error!("[OBS_CORE] Failed to emit connection status event: {}", e);
        }

        // Spawn WebSocket task
        self.spawn_ws_task(connection_name.to_string()).await;

        Ok(())
    }

    /// Disconnect from OBS WebSocket
    pub async fn disconnect_obs(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_CORE] disconnect_obs called for '{}'", connection_name);
        
        // Update connection status
        {
            let mut connections = self.context.connections.lock().await;
            if let Some(connection) = connections.get_mut(connection_name) {
                connection.status = ObsConnectionStatus::Disconnected;
                connection.websocket = None;
                connection.pending_requests.clear();
                connection.heartbeat_data = None;
            }
        }

        // Emit status change event
        let status_event = ObsEvent::ConnectionStatusChanged {
            connection_name: connection_name.to_string(),
            status: ObsConnectionStatus::Disconnected,
        };
        if let Err(e) = self.context.event_tx.send(status_event) {
            log::error!("[OBS_CORE] Failed to emit connection status event: {}", e);
        }

        log::info!("[OBS_CORE] '{}' disconnected successfully", connection_name);
        Ok(())
    }

    /// Remove a connection
    pub async fn remove_connection(&self, connection_name: &str) -> AppResult<()> {
        log::info!("[OBS_CORE] remove_connection called for '{}'", connection_name);
        
        // First disconnect if connected
        if let Ok(Some(status)) = self.get_connection_status(connection_name).await {
            if status != ObsConnectionStatus::Disconnected {
                if let Err(e) = self.disconnect_obs(connection_name).await {
                    log::warn!("[OBS_CORE] Failed to disconnect '{}' before removal: {}", connection_name, e);
                }
            }
        }

        // Remove from connections
        {
            let mut connections = self.context.connections.lock().await;
            connections.remove(connection_name);
        }

        log::info!("[OBS_CORE] '{}' removed successfully", connection_name);
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

    /// Get connection roles (for OBS_REC and OBS_STR)
    pub async fn get_connection_roles(&self) -> Vec<(String, String)> {
        let connections = self.context.connections.lock().await;
        connections
            .iter()
            .map(|(name, conn)| (name.clone(), conn.config.name.clone()))
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
                let request_id = self.generate_request_id(connection);
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

    /// Generate a unique request ID
    fn generate_request_id(&self, connection: &mut ObsConnection) -> String {
        connection.request_id_counter += 1;
        format!("req_{}", connection.request_id_counter)
    }

    /// Spawn WebSocket task for a connection
    async fn spawn_ws_task(&self, connection_name: String) {
        // This is a simplified version - the full implementation would be copied from the original
        log::info!("[OBS_CORE] Spawning WebSocket task for '{}'", connection_name);
        
        // TODO: Copy the full spawn_ws_task implementation from plugin_obs.rs
        // This includes the WebSocket connection, authentication, and message handling
    }

    /// Authenticate with OBS WebSocket v5
    async fn authenticate_v5(&self, connection_name: &str, ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) -> AppResult<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>> {
        // TODO: Copy the full authenticate_v5 implementation from plugin_obs.rs
        log::info!("[OBS_CORE] Authenticating v5 for '{}'", connection_name);
        
        // Placeholder - will be implemented when we copy the full function
        Err(AppError::ConfigError("Authentication not yet implemented".to_string()))
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