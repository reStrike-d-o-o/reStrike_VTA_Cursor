use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::database::AsyncDatabaseConnection;
use crate::types::AppResult;
use crate::plugins::obs::core::ObsCorePlugin;

/// Connection configuration for a Control Room OBS instance
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
    session_start: Arc<Mutex<Option<chrono::DateTime<chrono::Utc>>>>,
    session_timeout_minutes: u64,
}

impl AsyncControlRoomManager {
    /// Create a new Control Room manager with secure authentication
    pub async fn new(
        master_password: String,
        db: Arc<AsyncDatabaseConnection>,
        obs_core: Arc<ObsCorePlugin>,
    ) -> AppResult<Self> {
        // Check if master password is configured
        let stored_hash = db.query_scalar::<String>("SELECT password_hash FROM control_room_config WHERE id = 1").await
            .unwrap_or(None);
        
        let is_authenticated = if let Some(hash) = stored_hash {
            // Verify against stored bcrypt hash
            let auth_result = Self::verify_password(&master_password, &hash);
            Self::log_authentication_attempt(&db, auth_result, "existing_password").await;
            auth_result
        } else {
            // First-time setup: set the master password
            if master_password.is_empty() {
                log::warn!("Cannot set empty master password on first-time setup");
                false
            } else {
                match Self::setup_master_password(&master_password, &db).await {
                    Ok(_) => {
                        log::info!("Master password configured successfully on first use");
                        true
                    }
                    Err(e) => {
                        log::error!("Failed to configure master password: {}", e);
                        false
                    }
                }
            }
        };
        
        let manager = Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            db,
            obs_core,
            authenticated: Arc::new(Mutex::new(is_authenticated)),
            session_start: Arc::new(Mutex::new(if is_authenticated { Some(chrono::Utc::now()) } else { None })),
            session_timeout_minutes: 720, // 12 hour session timeout (tournament day length)
        };
        
        if is_authenticated {
            manager.load_connections().await?;
        }
        
        Ok(manager)
    }

    /// Set up master password on first use with bcrypt hashing
    async fn setup_master_password(password: &str, db: &AsyncDatabaseConnection) -> AppResult<()> {
        // Create config table if it doesn't exist
        db.execute("CREATE TABLE IF NOT EXISTS control_room_config (id INTEGER PRIMARY KEY, password_hash TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)").await?;

        // Generate secure bcrypt hash
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| crate::types::AppError::SecurityError(format!("Password hashing failed: {}", e)))?;
        
        let now = chrono::Utc::now().to_rfc3339();

        // Store the password hash
        db.execute_with_string_params(
            "INSERT OR REPLACE INTO control_room_config (id, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?)",
            vec!["1".to_string(), password_hash, now.clone(), now]
        ).await?;

        log::info!("Master password configured with bcrypt hashing");
        Ok(())
    }

    /// Verify a password against its bcrypt hash
    fn verify_password(password: &str, hash: &str) -> bool {
        bcrypt::verify(password, hash).unwrap_or(false)
    }

    /// Change the master password (admin function)
    pub async fn change_master_password(&self, current_password: &str, new_password: &str) -> AppResult<()> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        // Verify current password
        let stored_hash = self.db.query_scalar::<String>("SELECT password_hash FROM control_room_config WHERE id = 1").await
            .unwrap_or(None);
        
        if let Some(hash) = stored_hash {
            if !Self::verify_password(current_password, &hash) {
                return Err(crate::types::AppError::SecurityError("Current password is incorrect".to_string()));
            }
        } else {
            return Err(crate::types::AppError::SecurityError("No master password configured".to_string()));
        }

        // Generate new bcrypt hash
        let new_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|e| crate::types::AppError::SecurityError(format!("Password hashing failed: {}", e)))?;
        
        let now = chrono::Utc::now().to_rfc3339();

        // Update the password hash
        self.db.execute_with_string_params(
            "UPDATE control_room_config SET password_hash = ?, updated_at = ? WHERE id = 1",
            vec![new_hash, now]
        ).await?;

        log::info!("Master password changed successfully");
        Ok(())
    }

    /// Log authentication attempts for security audit
    async fn log_authentication_attempt(db: &AsyncDatabaseConnection, success: bool, attempt_type: &str) {
        let _ = db.execute("CREATE TABLE IF NOT EXISTS control_room_audit (id INTEGER PRIMARY KEY AUTOINCREMENT, attempt_type TEXT, success BOOLEAN, timestamp TEXT, ip_address TEXT)").await;
        
        let timestamp = chrono::Utc::now().to_rfc3339();
        let ip_address = "localhost".to_string(); // In a real app, you'd get the actual IP
        
        let result = db.execute_with_string_params(
            "INSERT INTO control_room_audit (attempt_type, success, timestamp, ip_address) VALUES (?, ?, ?, ?)",
            vec![attempt_type.to_string(), success.to_string(), timestamp, ip_address]
        ).await;
        
        if let Err(e) = result {
            log::warn!("Failed to log authentication attempt: {}", e);
        } else {
            log::info!("Authentication attempt logged: {} - {}", attempt_type, if success { "SUCCESS" } else { "FAILED" });
        }
    }

    /// Get audit log entries
    pub async fn get_audit_log(&self) -> AppResult<Vec<serde_json::Value>> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        let rows = self.db.query_rows(
            "SELECT attempt_type, success, timestamp, ip_address FROM control_room_audit ORDER BY timestamp DESC LIMIT 100",
            |row| {
                use sqlx::Row;
                Ok(serde_json::json!({
                    "attempt_type": row.try_get::<String, _>(0)?,
                    "success": row.try_get::<String, _>(1)? == "true",
                    "timestamp": row.try_get::<String, _>(2)?,
                    "ip_address": row.try_get::<String, _>(3)?
                }))
            }
        ).await.unwrap_or_else(|_| Vec::new());

        Ok(rows)
    }

    /// Check if the manager is authenticated and session is still valid
    pub async fn is_authenticated(&self) -> bool {
        let authenticated = *self.authenticated.lock().await;
        if !authenticated {
            return false;
        }

        // Check session timeout
        if self.is_session_expired().await {
            self.expire_session().await;
            return false;
        }

        true
    }

    /// Check if the current session has expired
    async fn is_session_expired(&self) -> bool {
        let session_start = self.session_start.lock().await;
        if let Some(start_time) = *session_start {
            let elapsed = chrono::Utc::now().signed_duration_since(start_time);
            elapsed.num_minutes() > self.session_timeout_minutes as i64
        } else {
            true // No session means expired
        }
    }

    /// Expire the current session
    async fn expire_session(&self) {
        let mut authenticated = self.authenticated.lock().await;
        let mut session_start = self.session_start.lock().await;
        *authenticated = false;
        *session_start = None;
        log::info!("Control Room session expired due to timeout");
    }

    /// Refresh the session (extend timeout)
    pub async fn refresh_session(&self) -> AppResult<()> {
        if self.is_authenticated().await {
            let mut session_start = self.session_start.lock().await;
            *session_start = Some(chrono::Utc::now());
            log::debug!("Control Room session refreshed");
            Ok(())
        } else {
            Err(crate::types::AppError::SecurityError("Session expired or not authenticated".to_string()))
        }
    }

    /// Get session info
    pub async fn get_session_info(&self) -> serde_json::Value {
        let authenticated = *self.authenticated.lock().await;
        let session_start = *self.session_start.lock().await;
        
        if authenticated && session_start.is_some() {
            let start_time = session_start.unwrap();
            let elapsed = chrono::Utc::now().signed_duration_since(start_time);
            let remaining_minutes = self.session_timeout_minutes as i64 - elapsed.num_minutes();
            
            serde_json::json!({
                "authenticated": true,
                "session_start": start_time.to_rfc3339(),
                "elapsed_minutes": elapsed.num_minutes(),
                "remaining_minutes": remaining_minutes.max(0),
                "timeout_minutes": self.session_timeout_minutes
            })
        } else {
            serde_json::json!({
                "authenticated": false,
                "session_start": null,
                "elapsed_minutes": 0,
                "remaining_minutes": 0,
                "timeout_minutes": self.session_timeout_minutes
            })
        }
    }

    /// Manual logout
    pub async fn logout(&self) {
        self.expire_session().await;
        log::info!("Control Room manual logout");
    }

    /// Add a new OBS connection configuration
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

    /// Remove an OBS connection
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

    /// Connect to an OBS instance
    pub async fn connect_obs(&self, name: &str) -> AppResult<()> {
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
            
            // First add the connection configuration
            let add_result = self.obs_core.add_connection(obs_config).await;
            
            match add_result {
                Ok(_) => {
                    // Then establish the actual WebSocket connection
                    let connect_result = self.obs_core.connect_obs(&obs_connection_name).await;
                    
                    match connect_result {
                        Ok(_) => {
                            instance.status = ControlRoomStatus::Connected;
                            instance.obs_connection_name = Some(obs_connection_name);
                            instance.last_connected = Some(chrono::Utc::now());
                        }
                        Err(e) => {
                            instance.status = ControlRoomStatus::Error(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    instance.status = ControlRoomStatus::Error(e.to_string());
                }
            }
            
            Ok(())
        } else {
            Err(crate::types::AppError::ConfigError(format!("OBS connection '{}' not found", name)))
        }
    }

    /// Disconnect from an OBS instance
    pub async fn disconnect_obs(&self, name: &str) -> AppResult<()> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        let mut connections = self.connections.lock().await;
        
        if let Some(instance) = connections.get_mut(name) {
            if let Some(obs_connection_name) = &instance.obs_connection_name {
                // First disconnect the WebSocket
                let _ = self.obs_core.disconnect_obs(obs_connection_name).await;
                // Then remove the connection configuration
                self.obs_core.remove_connection(obs_connection_name).await?;
            }
            
            instance.status = ControlRoomStatus::Disconnected;
            instance.obs_connection_name = None;
            
            Ok(())
        } else {
            Err(crate::types::AppError::ConfigError(format!("OBS connection '{}' not found", name)))
        }
    }

    /// Get the OBS connection name for an OBS instance
    pub async fn get_obs_connection_name(&self, obs_name: &str) -> AppResult<String> {
        let connections = self.connections.lock().await;
        
        if let Some(instance) = connections.get(obs_name) {
            if let Some(obs_name) = &instance.obs_connection_name {
                Ok(obs_name.clone())
            } else {
                Err(crate::types::AppError::ConfigError(format!("OBS '{}' is not connected", obs_name)))
            }
        } else {
            Err(crate::types::AppError::ConfigError(format!("OBS connection '{}' not found", obs_name)))
        }
    }

    /// Get list of OBS connection names
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

    /// Get a specific connection configuration
    pub async fn get_connection(&self, name: &str) -> AppResult<ControlRoomConnection> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        let connections = self.connections.lock().await;
        if let Some(instance) = connections.get(name) {
            Ok(instance.config.clone())
        } else {
            Err(crate::types::AppError::ConfigError(format!("OBS connection '{}' not found", name)))
        }
    }

    /// Update an existing OBS connection configuration
    pub async fn update_connection(&self, name: &str, config: ControlRoomConnection) -> AppResult<()> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        // Check if the connection exists
        {
            let connections = self.connections.lock().await;
            if !connections.contains_key(name) {
                return Err(crate::types::AppError::ConfigError(format!("OBS connection '{}' not found", name)));
            }
        }

        // Store updated config in database
        let config_json = serde_json::to_string(&config)
            .map_err(|e| crate::types::AppError::ConfigError(e.to_string()))?;
        
        self.db.execute_with_string_params(
            "UPDATE control_room_connections SET config = ? WHERE name = ?",
            vec![config_json, name.to_string()]
        ).await?;

        // Update in memory
        let mut connections = self.connections.lock().await;
        if let Some(instance) = connections.get_mut(name) {
            instance.config = config;
        }

        Ok(())
    }

    /// Connect all disconnected OBS connections
    pub async fn connect_all_obs(&self) -> AppResult<Vec<(String, AppResult<()>)>> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        let mut results = Vec::new();
        let connections = self.connections.lock().await;
        let connection_names: Vec<String> = connections
            .iter()
            .filter(|(_, instance)| {
                matches!(instance.status, ControlRoomStatus::Disconnected | ControlRoomStatus::Error(_))
            })
            .map(|(name, _)| name.clone())
            .collect();
        drop(connections); // Release lock before async operations

        for name in connection_names {
            let result = self.connect_obs(&name).await;
            results.push((name, result));
        }

        Ok(results)
    }

    /// Disconnect all connected OBS connections
    pub async fn disconnect_all_obs(&self) -> AppResult<Vec<(String, AppResult<()>)>> {
        if !self.is_authenticated().await {
            return Err(crate::types::AppError::SecurityError("Not authenticated".to_string()));
        }

        let mut results = Vec::new();
        let connections = self.connections.lock().await;
        let connection_names: Vec<String> = connections
            .iter()
            .filter(|(_, instance)| {
                matches!(instance.status, ControlRoomStatus::Connected)
            })
            .map(|(name, _)| name.clone())
            .collect();
        drop(connections); // Release lock before async operations

        for name in connection_names {
            let result = self.disconnect_obs(&name).await;
            results.push((name, result));
        }

        Ok(results)
    }
}