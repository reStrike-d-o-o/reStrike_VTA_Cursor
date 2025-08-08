//! OBS Manager implementation for managing multiple OBS connections

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::types::{AppError, AppResult};
use super::client::ObsClient;
use super::types::{
    ObsConnectionConfig, ObsConnectionStatus, ObsStatus, ObsConnectionInfo, ObsEvent
};

/// OBS Manager for handling multiple OBS connections
pub struct ObsManager {
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<ObsClient>>>>>,
    default_connection: Arc<Mutex<Option<String>>>,
}

impl ObsManager {
    /// Create a new OBS manager
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            default_connection: Arc::new(Mutex::new(None)),
        }
    }

    /// Add a new OBS connection
    pub async fn add_connection(&self, config: ObsConnectionConfig) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        if clients.contains_key(&config.name) {
            return Err(AppError::ConfigError(format!("Connection '{}' already exists", config.name)));
        }
        
        let client = ObsClient::new(config.clone());
        clients.insert(config.name.clone(), Arc::new(Mutex::new(client)));
        
        // Set as default if it's the first connection
        if clients.len() == 1 {
            let mut default = self.default_connection.lock().await;
            *default = Some(config.name.clone());
        }
        
        log::info!("✅ Added OBS connection: {}", config.name);
        Ok(())
    }

    /// Update an existing OBS connection configuration
    pub async fn update_connection(&self, old_name: &str, new_config: ObsConnectionConfig) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        
        // Check if the old connection exists
        if !clients.contains_key(old_name) {
            return Err(AppError::ConfigError(format!("Connection '{}' not found", old_name)));
        }
        
        // If the name is being changed, check if the new name already exists
        if old_name != new_config.name && clients.contains_key(&new_config.name) {
            return Err(AppError::ConfigError(format!("Connection '{}' already exists", new_config.name)));
        }
        
        // Get the existing client to preserve its state
        let existing_client_arc = clients.remove(old_name).unwrap();
        let existing_client = existing_client_arc.lock().await;
        let was_connected = existing_client.get_connection_status() == ObsConnectionStatus::Connected;
        
        // Drop the lock to avoid deadlock
        drop(existing_client);
        drop(existing_client_arc);
        
        // Create new client with updated configuration
        let new_client = ObsClient::new(new_config.clone());
        let new_client_arc = Arc::new(Mutex::new(new_client));
        
        // Clone the Arc for later use
        let new_client_arc_clone = new_client_arc.clone();
        
        // Insert the new client
        if old_name == new_config.name {
            // Same name, just update the configuration
            clients.insert(new_config.name.clone(), new_client_arc);
        } else {
            // Different name, insert with new name
            clients.insert(new_config.name.clone(), new_client_arc);
            
            // Update default connection if this was the default
            let mut default = self.default_connection.lock().await;
            if let Some(ref default_name) = *default {
                if default_name == old_name {
                    *default = Some(new_config.name.clone());
                }
            }
        }
        
        // If the connection was connected, try to reconnect with new settings
        if was_connected {
            let mut new_client = new_client_arc_clone.lock().await;
            if let Err(e) = new_client.connect().await {
                log::warn!("Warning: Failed to reconnect after update: {}", e);
            }
        }
        
        log::info!("✅ Updated OBS connection: {} -> {}", old_name, new_config.name);
        Ok(())
    }

    /// Remove an OBS connection
    pub async fn remove_connection(&self, name: &str) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        if let Some(client_arc) = clients.remove(name) {
            // Disconnect the client
            let mut client = client_arc.lock().await;
            if let Err(e) = client.disconnect().await {
                log::warn!("Warning: Failed to disconnect client '{}': {}", name, e);
            }
            
            // Update default connection if this was the default
            let mut default = self.default_connection.lock().await;
            if let Some(ref default_name) = *default {
                if default_name == name {
                    *default = clients.keys().next().cloned();
                }
            }
            
            log::info!("✅ Removed OBS connection: {}", name);
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", name)))
        }
    }

    /// Connect to an OBS instance
    pub async fn connect(&self, name: &str) -> AppResult<()> {
        let clients = self.clients.lock().await;
        if let Some(client_arc) = clients.get(name) {
            let mut client = client_arc.lock().await;
            client.connect().await?;
            log::info!("✅ Connected to OBS: {}", name);
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", name)))
        }
    }

    /// Disconnect from an OBS instance
    pub async fn disconnect(&self, name: &str) -> AppResult<()> {
        let clients = self.clients.lock().await;
        if let Some(client_arc) = clients.get(name) {
            let mut client = client_arc.lock().await;
            client.disconnect().await?;
            log::info!("✅ Disconnected from OBS: {}", name);
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", name)))
        }
    }

    /// Get connection status
    pub async fn get_connection_status(&self, name: &str) -> AppResult<ObsConnectionStatus> {
        let clients = self.clients.lock().await;
        if let Some(client_arc) = clients.get(name) {
            let client = client_arc.lock().await;
            Ok(client.get_connection_status())
        } else {
            Err(AppError::ConfigError(format!("Connection '{}' not found", name)))
        }
    }

    /// Get all connection information
    pub async fn get_connections(&self) -> AppResult<Vec<ObsConnectionInfo>> {
        let clients = self.clients.lock().await;
        let _default = self.default_connection.lock().await;
        
        let mut connections = Vec::new();
        for (name, client_arc) in clients.iter() {
            let client = client_arc.lock().await;
            connections.push(ObsConnectionInfo {
                name: name.clone(),
                host: client.get_config().host.clone(),
                port: client.get_config().port,
                status: client.get_connection_status(),
                last_activity: None, // TODO: Track last activity
            });
        }
        
        Ok(connections)
    }

    /// Set default connection
    pub async fn set_default_connection(&self, name: &str) -> AppResult<()> {
        let clients = self.clients.lock().await;
        if !clients.contains_key(name) {
            return Err(AppError::ConfigError(format!("Connection '{}' not found", name)));
        }
        
        let mut default = self.default_connection.lock().await;
        *default = Some(name.to_string());
        log::info!("✅ Set default OBS connection: {}", name);
        Ok(())
    }

    /// Get default connection name
    pub async fn get_default_connection(&self) -> Option<String> {
        let default = self.default_connection.lock().await;
        default.clone()
    }

    /// Get a client reference by name
    async fn get_client_ref(&self, name: Option<&str>) -> AppResult<Arc<Mutex<ObsClient>>> {
        let connection_name = match name {
            Some(n) => n.to_string(),
            None => {
                let default = self.default_connection.lock().await;
                default.as_ref().ok_or_else(|| {
                    AppError::ConfigError("No default OBS connection set".to_string())
                })?.clone()
            }
        };
        
        let clients = self.clients.lock().await;
        clients.get(&connection_name).cloned().ok_or_else(|| {
            AppError::ConfigError(format!("Connection '{}' not found", connection_name))
        })
    }

    // Recording operations
    pub async fn start_recording(&self, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.start_recording().await
    }

    pub async fn stop_recording(&self, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.stop_recording().await
    }

    pub async fn get_recording_status(&self, connection_name: Option<&str>) -> AppResult<super::types::ObsRecordingStatus> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_recording_status().await
    }

    // Streaming operations
    pub async fn start_streaming(&self, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.start_streaming().await
    }

    pub async fn stop_streaming(&self, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.stop_streaming().await
    }

    pub async fn get_streaming_status(&self, connection_name: Option<&str>) -> AppResult<super::types::ObsStreamingStatus> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_streaming_status().await
    }

    // Replay buffer operations
    pub async fn start_replay_buffer(&self, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.start_replay_buffer().await
    }

    pub async fn stop_replay_buffer(&self, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.stop_replay_buffer().await
    }

    pub async fn save_replay_buffer(&self, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.save_replay_buffer().await
    }

    pub async fn get_replay_buffer_status(&self, connection_name: Option<&str>) -> AppResult<super::types::ObsReplayBufferStatus> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_replay_buffer_status().await
    }

    // Virtual camera operations
    pub async fn start_virtual_camera(&self, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.start_virtual_camera().await
    }

    pub async fn stop_virtual_camera(&self, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.stop_virtual_camera().await
    }

    pub async fn get_virtual_camera_status(&self, connection_name: Option<&str>) -> AppResult<super::types::ObsVirtualCameraStatus> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_virtual_camera_status().await
    }

    // Scene operations
    pub async fn get_current_scene(&self, connection_name: Option<&str>) -> AppResult<String> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_current_scene().await
    }

    pub async fn set_current_scene(&self, scene_name: &str, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.set_current_scene(scene_name).await
    }

    pub async fn get_scenes(&self, connection_name: Option<&str>) -> AppResult<Vec<super::types::ObsScene>> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_scenes().await
    }

    // Status operations
    pub async fn get_status(&self, connection_name: Option<&str>) -> AppResult<ObsStatus> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_status().await
    }

    pub async fn get_version(&self, connection_name: Option<&str>) -> AppResult<super::types::ObsVersion> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_version().await
    }

    pub async fn get_stats(&self, connection_name: Option<&str>) -> AppResult<super::types::ObsStats> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_stats().await
    }

    // Event handling
    pub async fn add_event_handler<F>(&self, event_type: String, handler: F, connection_name: Option<&str>) -> AppResult<()>
    where
        F: Fn(ObsEvent) + Send + Sync + 'static,
    {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.add_event_handler(event_type, handler).await
    }

    pub async fn remove_event_handler(&self, event_type: &str, connection_name: Option<&str>) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.remove_event_handler(event_type).await
    }

    /// Shutdown all connections
    pub async fn shutdown(&self) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        for (name, client_arc) in clients.iter_mut() {
            let mut client = client_arc.lock().await;
            if let Err(e) = client.disconnect().await {
                log::warn!("Warning: Failed to disconnect client '{}': {}", name, e);
            }
        }
        clients.clear();
        
        let mut default = self.default_connection.lock().await;
        *default = None;
        
        log::info!("✅ OBS Manager shutdown complete");
        Ok(())
    }

    /// Get the number of active connections
    pub async fn connection_count(&self) -> usize {
        let clients = self.clients.lock().await;
        clients.len()
    }

    /// Check if any connection is active
    pub async fn has_active_connections(&self) -> bool {
        let clients = self.clients.lock().await;
        for client_arc in clients.values() {
            let client = client_arc.lock().await;
            if client.is_connected() {
                return true;
            }
        }
        false
    }

    /// Get all connection names
    pub async fn get_connection_names(&self) -> Vec<String> {
        let clients = self.clients.lock().await;
        clients.keys().cloned().collect()
    }

    /// Set up status listener for all connections
    pub async fn setup_status_listener(&self) -> AppResult<()> {
        let clients = self.clients.lock().await;
        for (name, client_arc) in clients.iter() {
            let client = client_arc.lock().await;
            if let Err(e) = client.setup_status_listener().await {
                log::warn!("Warning: Failed to set up status listener for '{}': {}", name, e);
            }
        }
        Ok(())
    }
}

impl Default for ObsManager {
    fn default() -> Self {
        Self::new()
    }
}
