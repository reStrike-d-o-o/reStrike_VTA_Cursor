use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::database::AsyncDatabaseConnection;
use crate::types::AppResult;
use crate::plugins::obs::core::ObsCorePlugin;

/// Connection configuration for a Control Room STR instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRoomConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub notes: Option<String>,
}

/// Status of a Control Room connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlRoomStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Active Control Room connection instance
#[derive(Debug)]
pub struct ControlRoomInstance {
    pub config: ControlRoomConnection,
    pub status: ControlRoomStatus,
    pub obs_connection_name: Option<String>, // Maps to actual OBS connection
    pub last_connected: Option<chrono::DateTime<chrono::Utc>>,
}

/// Async Control Room Manager - thread-safe for Tauri commands
pub struct AsyncControlRoomManager {
    connections: Arc<Mutex<HashMap<String, ControlRoomInstance>>>,
    db: Arc<AsyncDatabaseConnection>,
    obs_core: Arc<ObsCorePlugin>,
    authenticated: Arc<Mutex<bool>>,
}

impl AsyncControlRoomManager {
    /// Create a new Control Room manager
    pub async fn new(
        master_password: String,
        db: Arc<AsyncDatabaseConnection>,
        obs_core: Arc<ObsCorePlugin>,
    ) -> AppResult<Self> {
        // Simple authentication check - just verify the password is not empty
        // In production, this would verify against a stored hash
        let is_authenticated = !master_password.is_empty();
        
        let manager = Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            db,
            obs_core,
            authenticated: Arc::new(Mutex::new(is_authenticated)),
        };
        
        if is_authenticated {
            manager.load_connections().await?;
        }
        
        Ok(manager)
    }

    /// Check if the manager is authenticated
    pub async fn is_authenticated(&self) -> bool {
        *self.authenticated.lock().await
    }

    /// Add a new STR connection configuration
    pub async fn add_connection(&self, config: ControlRoomConnection) -> AppResult<()> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        // Store in database
        let config_json = serde_json::to_string(&config)
            .map_err(|e| crate::types::AppError::ConfigError(e.to_string()))?;
        
        self.db.execute_with_string_params(
            "INSERT OR REPLACE INTO control_room_connections (name, config) VALUES (?, ?)",
            vec![config.name.clone(), config_json]
        ).await?;

        // Add to memory
        let mut connections = self.connections.lock().await;
        connections.insert(config.name.clone(), ControlRoomInstance {
            config,
            status: ControlRoomStatus::Disconnected,
            obs_connection_name: None,
            last_connected: None,
        });

        Ok(())
    }

    /// Remove a STR connection
    pub async fn remove_connection(&self, name: &str) -> AppResult<()> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        // Remove from database
        self.db.execute_with_string_params(
            "DELETE FROM control_room_connections WHERE name = ?",
            vec![name.to_string()]
        ).await?;

        // Remove from memory
        let mut connections = self.connections.lock().await;
        connections.remove(name);

        Ok(())
    }

    /// Load connections from database
    async fn load_connections(&self) -> AppResult<()> {
        // Ensure table exists
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS control_room_connections (name TEXT PRIMARY KEY, config TEXT NOT NULL, created_at DATETIME DEFAULT CURRENT_TIMESTAMP)"
        ).await?;

        // Load configurations
        let rows = self.db.query_rows(
            "SELECT name, config FROM control_room_connections",
            |row| {
                use sqlx::Row;
                Ok((
                    row.try_get::<String, _>(0)?,
                    row.try_get::<String, _>(1)?
                ))
            }
        ).await?;

        let mut connections = self.connections.lock().await;
        for (name, config_json) in rows {
            match serde_json::from_str::<ControlRoomConnection>(&config_json) {
                Ok(config) => {
                    connections.insert(name.clone(), ControlRoomInstance {
                        config,
                        status: ControlRoomStatus::Disconnected,
                        obs_connection_name: None,
                        last_connected: None,
                    });
                }
                Err(e) => {
                    log::warn!("Failed to deserialize control room connection '{}': {}", name, e);
                }
            }
        }

        Ok(())
    }

    /// Connect to a STR instance
    pub async fn connect_str(&self, name: &str) -> AppResult<()> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        let mut connections = self.connections.lock().await;
        
        if let Some(instance) = connections.get_mut(name) {
            instance.status = ControlRoomStatus::Connecting;
            
            // Create OBS connection name
            let obs_connection_name = format!("control_room_{}", name);
            
            // Use the OBS core plugin to create the connection
            let obs_config = crate::plugins::obs::types::ObsConnectionConfig {
                name: obs_connection_name.clone(),
                host: instance.config.host.clone(),
                port: instance.config.port,
                password: instance.config.password.clone(),
                protocol_version: crate::plugins::obs::types::ObsWebSocketVersion::V5,
                enabled: true,
            };
            let connection_result = self.obs_core.add_connection(obs_config).await;

            match connection_result {
                Ok(_) => {
                    instance.status = ControlRoomStatus::Connected;
                    instance.obs_connection_name = Some(obs_connection_name);
                    instance.last_connected = Some(chrono::Utc::now());
                }
                Err(e) => {
                    instance.status = ControlRoomStatus::Error(e.to_string());
                }
            }
            
            Ok(())
        } else {
            Err(crate::types::AppError::ConfigError(format!("STR connection '{}' not found", name)))
        }
    }

    /// Disconnect from a STR instance
    pub async fn disconnect_str(&self, name: &str) -> AppResult<()> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        let mut connections = self.connections.lock().await;
        
        if let Some(instance) = connections.get_mut(name) {
            if let Some(obs_connection_name) = &instance.obs_connection_name {
                // Remove the OBS connection
                self.obs_core.remove_connection(obs_connection_name).await?;
            }
            
            instance.status = ControlRoomStatus::Disconnected;
            instance.obs_connection_name = None;
            
            Ok(())
        } else {
            Err(crate::types::AppError::ConfigError(format!("STR connection '{}' not found", name)))
        }
    }

    /// Get the OBS connection name for a STR instance
    pub async fn get_obs_connection_name(&self, str_name: &str) -> AppResult<String> {
        let connections = self.connections.lock().await;
        
        if let Some(instance) = connections.get(str_name) {
            if let Some(obs_name) = &instance.obs_connection_name {
                Ok(obs_name.clone())
            } else {
                Err(crate::types::AppError::ConfigError(format!("STR '{}' is not connected", str_name)))
            }
        } else {
            Err(crate::types::AppError::ConfigError(format!("STR connection '{}' not found", str_name)))
        }
    }

    /// Get list of STR connection names
    pub async fn get_connection_names(&self) -> Vec<String> {
        let connections = self.connections.lock().await;
        connections.keys().cloned().collect()
    }

    /// Get all connections with their status
    pub async fn get_all_connections(&self) -> AppResult<Vec<(String, ControlRoomStatus)>> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        let connections = self.connections.lock().await;
        Ok(connections.iter()
            .map(|(name, instance)| (name.clone(), instance.status.clone()))
            .collect())
    }
}