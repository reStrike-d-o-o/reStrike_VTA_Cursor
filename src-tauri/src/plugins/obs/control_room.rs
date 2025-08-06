// Control Room Connection Management
// Separate from WebSocket tab connection management
// User-defined STR connections with encrypted storage

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use crate::types::AppResult;
use crate::security::{SecureConfigManager, AccessLevel};
use crate::database::DatabaseConnection;

/// Control Room STR connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRoomConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub last_used: Option<String>,
    pub notes: Option<String>,
}

/// Control Room connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlRoomStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Active Control Room connection instance
#[derive(Debug, Clone)]
pub struct ControlRoomInstance {
    pub config: ControlRoomConnection,
    pub status: ControlRoomStatus,
    pub obs_connection_name: Option<String>, // Maps to actual OBS connection
}

/// Control Room Manager - handles separate STR connections
pub struct ControlRoomManager {
    connections: Arc<Mutex<HashMap<String, ControlRoomInstance>>>,
    config_manager: Arc<SecureConfigManager>,
    obs_core: Arc<super::core::ObsCorePlugin>,
    session_id: String,
}

impl ControlRoomManager {
    /// Create new Control Room Manager
    pub async fn new(
        master_password: String,
        database: Arc<DatabaseConnection>,
        obs_core: Arc<super::core::ObsCorePlugin>,
    ) -> AppResult<Self> {
        let config_manager = Arc::new(SecureConfigManager::new(master_password, database).await?);
        
        // Create a session for Control Room operations
        let session = config_manager.create_session(
            "control_room_system".to_string(),
            AccessLevel::Administrator,
            None,
            Some("Control Room System".to_string()),
        ).await?;
        
        let manager = Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            config_manager,
            obs_core,
            session_id: session.session_id,
        };
        
        // Load existing connections from encrypted storage
        manager.load_connections().await?;
        
        Ok(manager)
    }
    
    /// Add a new Control Room STR connection
    pub async fn add_connection(&self, mut config: ControlRoomConnection) -> AppResult<()> {
        config.created_at = chrono::Utc::now().to_rfc3339();
        
        // Store encrypted configuration
        let config_key = format!("control_room_connection_{}", config.name);
        let config_value = serde_json::to_string(&config)?;
        self.config_manager.set_config(
            &self.session_id,
            &config_key,
            &config_value,
            crate::security::ConfigCategory::ControlRoom,
            Some("Control Room STR connection configuration"),
        ).await?;
        
        // Create instance
        let instance = ControlRoomInstance {
            config: config.clone(),
            status: ControlRoomStatus::Disconnected,
            obs_connection_name: None,
        };
        
        // Add to memory
        let mut connections = self.connections.lock().await;
        connections.insert(config.name.clone(), instance);
        
        log::info!("[CONTROL_ROOM] Added STR connection: {}", config.name);
        Ok(())
    }
    
    /// Remove a Control Room connection
    pub async fn remove_connection(&self, name: &str) -> AppResult<()> {
        // Remove from encrypted storage
        let config_key = format!("control_room_connection_{}", name);
        self.config_manager.delete_config(&self.session_id, &config_key).await?;
        
        // Disconnect if connected
        self.disconnect_str(name).await?;
        
        // Remove from memory
        let mut connections = self.connections.lock().await;
        connections.remove(name);
        
        log::info!("[CONTROL_ROOM] Removed STR connection: {}", name);
        Ok(())
    }
    
    /// Get all Control Room connections
    pub async fn get_connections(&self) -> AppResult<Vec<ControlRoomConnection>> {
        let connections = self.connections.lock().await;
        let configs: Vec<ControlRoomConnection> = connections.values()
            .map(|instance| instance.config.clone())
            .collect();
        Ok(configs)
    }
    
    /// Get connection names only
    pub async fn get_connection_names(&self) -> AppResult<Vec<String>> {
        let connections = self.connections.lock().await;
        let names: Vec<String> = connections.keys().cloned().collect();
        Ok(names)
    }
    
    /// Connect to a Control Room STR instance
    pub async fn connect_str(&self, name: &str) -> AppResult<()> {
        let mut connections = self.connections.lock().await;
        
        if let Some(instance) = connections.get_mut(name) {
            instance.status = ControlRoomStatus::Connecting;
            
            // Create OBS connection using the core plugin
            let obs_config = super::types::ObsConnectionConfig {
                name: format!("CR_{}", name), // Prefix to distinguish from WebSocket tab
                host: instance.config.host.clone(),
                port: instance.config.port,
                password: instance.config.password.clone(),
                protocol_version: super::types::ObsWebSocketVersion::V5,
                enabled: true,
            };
            
            match self.obs_core.add_connection(obs_config).await {
                Ok(_) => {
                    instance.status = ControlRoomStatus::Connected;
                    instance.obs_connection_name = Some(format!("CR_{}", name));
                    
                    // Update last_used timestamp
                    let mut config = instance.config.clone();
                    config.last_used = Some(chrono::Utc::now().to_rfc3339());
                    instance.config = config.clone();
                    
                    // Update encrypted storage
                    let config_key = format!("control_room_connection_{}", name);
                    let config_value = serde_json::to_string(&config)?;
                    drop(connections); // Release lock before async call
                    self.config_manager.set_config(
                        &self.session_id,
                        &config_key,
                        &config_value,
                        crate::security::ConfigCategory::ControlRoom,
                        Some("Control Room STR connection configuration"),
                    ).await?;
                    
                    log::info!("[CONTROL_ROOM] Connected to STR: {}", name);
                    Ok(())
                }
                Err(e) => {
                    instance.status = ControlRoomStatus::Error(e.to_string());
                    Err(e)
                }
            }
        } else {
            Err(crate::types::AppError::ConfigError(format!("Control Room connection '{}' not found", name)))
        }
    }
    
    /// Disconnect from a Control Room STR instance
    pub async fn disconnect_str(&self, name: &str) -> AppResult<()> {
        let mut connections = self.connections.lock().await;
        
        if let Some(instance) = connections.get_mut(name) {
            if let Some(obs_name) = &instance.obs_connection_name {
                // Remove OBS connection
                match self.obs_core.remove_connection(obs_name).await {
                    Ok(_) => {
                        instance.status = ControlRoomStatus::Disconnected;
                        instance.obs_connection_name = None;
                        log::info!("[CONTROL_ROOM] Disconnected from STR: {}", name);
                        Ok(())
                    }
                    Err(e) => {
                        instance.status = ControlRoomStatus::Error(e.to_string());
                        Err(e)
                    }
                }
            } else {
                instance.status = ControlRoomStatus::Disconnected;
                Ok(())
            }
        } else {
            Err(crate::types::AppError::ConfigError(format!("Control Room connection '{}' not found", name)))
        }
    }
    
    /// Get Control Room connection status
    pub async fn get_connection_status(&self, name: &str) -> AppResult<ControlRoomStatus> {
        let connections = self.connections.lock().await;
        if let Some(instance) = connections.get(name) {
            Ok(instance.status.clone())
        } else {
            Err(crate::types::AppError::ConfigError(format!("Control Room connection '{}' not found", name)))
        }
    }
    
    /// Get OBS connection name for Control Room connection
    pub async fn get_obs_connection_name(&self, name: &str) -> AppResult<Option<String>> {
        let connections = self.connections.lock().await;
        if let Some(instance) = connections.get(name) {
            Ok(instance.obs_connection_name.clone())
        } else {
            Err(crate::types::AppError::ConfigError(format!("Control Room connection '{}' not found", name)))
        }
    }
    
    /// Load connections from encrypted storage
    async fn load_connections(&self) -> AppResult<()> {
        let config_keys = self.config_manager.list_config_keys(&self.session_id, Some(crate::security::ConfigCategory::ControlRoom)).await?;
        let mut connections = self.connections.lock().await;
        
        for key in config_keys {
            if key.starts_with("control_room_connection_") {
                if let Ok(Some(config_value)) = self.config_manager.get_config(&self.session_id, &key).await {
                    if let Ok(config) = serde_json::from_str::<ControlRoomConnection>(&config_value) {
                        let instance = ControlRoomInstance {
                            config: config.clone(),
                            status: ControlRoomStatus::Disconnected,
                            obs_connection_name: None,
                        };
                        connections.insert(config.name.clone(), instance);
                    }
                }
            }
        }
        
        log::info!("[CONTROL_ROOM] Loaded {} connections from storage", connections.len());
        Ok(())
    }
    
    /// Validate session for Control Room operations
    pub async fn validate_session(&self, session_id: &str) -> AppResult<bool> {
        // TODO: Implement proper session validation
        // For now, just check if session_id is not empty
        Ok(!session_id.is_empty())
    }
}