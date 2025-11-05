//! OBS Manager implementation for managing multiple OBS connections

use super::client::ObsClient;
use super::types::{
    ObsConnectionConfig, ObsConnectionHealth, ObsConnectionInfo, ObsConnectionStatus, ObsEvent,
    ObsStatus,
};
use crate::types::{AppError, AppResult};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

/// OBS Manager for handling multiple OBS connections
pub struct ObsManager {
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<ObsClient>>>>>,
    default_connection: Arc<Mutex<Option<String>>>,
    health_tx: broadcast::Sender<ObsConnectionHealth>,
    health_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl ObsManager {
    /// Create a new OBS manager
    pub fn new() -> Self {
        let (health_tx, _) = broadcast::channel(64);
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            default_connection: Arc::new(Mutex::new(None)),
            health_tx,
            health_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Subscribe to connection health snapshots.
    pub fn subscribe_health(&self) -> broadcast::Receiver<ObsConnectionHealth> {
        self.health_tx.subscribe()
    }

    /// Ensure health monitors are running for all known connections.
    pub async fn ensure_health_watchers(&self) -> AppResult<()> {
        let names = {
            let clients = self.clients.lock().await;
            clients.keys().cloned().collect::<Vec<_>>()
        };

        for name in names {
            self.ensure_health_watcher(&name).await?;
        }

        Ok(())
    }

    async fn ensure_health_watcher(&self, name: &str) -> AppResult<()> {
        {
            let tasks = self.health_tasks.lock().await;
            if tasks.contains_key(name) {
                return Ok(());
            }
        }

        let client_arc = {
            let clients = self.clients.lock().await;
            clients
                .get(name)
                .cloned()
                .ok_or_else(|| AppError::ConfigError(format!("Connection '{name}' not found")))?
        };

        let mut tasks = self.health_tasks.lock().await;
        if tasks.contains_key(name) {
            return Ok(());
        }

        let tx = self.health_tx.clone();
        let connection_name = name.to_string();
        let task_name = connection_name.clone();
        let handle = tokio::spawn(async move {
            loop {
                let snapshot_opt = {
                    let client_guard = client_arc.lock().await;
                    if !client_guard.is_connected() {
                        None
                    } else {
                        let role = client_guard.get_config().role.clone();

                        match client_guard.get_stats().await {
                            Ok(stats) => {
                                let mut snapshot = ObsConnectionHealth {
                                    connection: task_name.clone(),
                                    role,
                                    cpu_usage: stats.cpu_usage,
                                    active_fps: stats.active_fps,
                                    skipped_frames: stats.output_skipped_frames.max(0) as u32,
                                    total_frames: stats.output_total_frames.max(0) as u32,
                                    congestion: 0.0,
                                    bytes: 0,
                                    duration_ms: 0,
                                    timestamp: Utc::now().to_rfc3339(),
                                };

                                match client_guard.get_stream_output_stats().await {
                                    Ok(stream_stats) => {
                                        snapshot.congestion = stream_stats.congestion;
                                        snapshot.bytes = stream_stats.bytes;
                                        snapshot.duration_ms = stream_stats.duration;
                                        snapshot.skipped_frames = stream_stats.skipped_frames;
                                        snapshot.total_frames = stream_stats.total_frames;
                                    }
                                    Err(err) => {
                                        log::trace!(
                                            "Stream stats unavailable for {task_name}: {err}"
                                        );
                                    }
                                }

                                Some(snapshot)
                            }
                            Err(err) => {
                                log::trace!("OBS stats unavailable for {task_name}: {err}");
                                None
                            }
                        }
                    }
                };

                if let Some(snapshot) = snapshot_opt {
                    let _ = tx.send(snapshot);
                }

                sleep(Duration::from_millis(1000)).await;
            }
        });

        tasks.insert(connection_name, handle);
        Ok(())
    }

    async fn stop_health_watcher(&self, name: &str) {
        let mut tasks = self.health_tasks.lock().await;
        if let Some(handle) = tasks.remove(name) {
            handle.abort();
        }
    }

    async fn stop_all_health_tasks(&self) {
        let mut tasks = self.health_tasks.lock().await;
        for (_, handle) in tasks.drain() {
            handle.abort();
        }
    }

    /// Add a new OBS connection
    pub async fn add_connection(&self, config: ObsConnectionConfig) -> AppResult<()> {
        {
            let mut clients = self.clients.lock().await;
            if clients.contains_key(&config.name) {
                return Err(AppError::ConfigError(format!(
                    "Connection '{}' already exists",
                    config.name
                )));
            }

            let client = ObsClient::new(config.clone());
            clients.insert(config.name.clone(), Arc::new(Mutex::new(client)));

            // Set as default if it's the first connection
            if clients.len() == 1 {
                let mut default = self.default_connection.lock().await;
                *default = Some(config.name.clone());
            }
        }

        // Spawn health watcher (no-op if already running)
        self.ensure_health_watcher(&config.name).await?;
        log::info!("Added OBS connection: {}", config.name);
        Ok(())
    }

    /// Update an existing OBS connection configuration
    pub async fn update_connection(
        &self,
        old_name: &str,
        new_config: ObsConnectionConfig,
    ) -> AppResult<()> {
        {
            let clients = self.clients.lock().await;
            if !clients.contains_key(old_name) {
                return Err(AppError::ConfigError(format!(
                    "Connection '{old_name}' not found"
                )));
            }
            if old_name != new_config.name && clients.contains_key(&new_config.name) {
                return Err(AppError::ConfigError(format!(
                    "Connection '{}' already exists",
                    new_config.name
                )));
            }
        }

        let existing_client_arc = {
            let mut clients = self.clients.lock().await;
            clients.remove(old_name).ok_or_else(|| {
                AppError::ConfigError(format!("Connection '{old_name}' not found"))
            })?
        };

        let was_connected = {
            let existing_client = existing_client_arc.lock().await;
            matches!(
                existing_client.get_connection_status(),
                ObsConnectionStatus::Connected | ObsConnectionStatus::Authenticated
            )
        };

        self.stop_health_watcher(old_name).await;

        let new_client_arc = Arc::new(Mutex::new(ObsClient::new(new_config.clone())));

        {
            let mut clients = self.clients.lock().await;
            clients.insert(new_config.name.clone(), new_client_arc.clone());
        }

        if old_name != new_config.name {
            let mut default = self.default_connection.lock().await;
            if let Some(ref default_name) = *default {
                if default_name == old_name {
                    *default = Some(new_config.name.clone());
                }
            }
        }

        if was_connected {
            let mut new_client = new_client_arc.lock().await;
            if let Err(e) = new_client.connect().await {
                log::warn!("Warning: Failed to reconnect after update: {e}");
            }
        }

        self.ensure_health_watcher(&new_config.name).await?;

        log::info!(
            "Updated OBS connection: {} -> {}",
            old_name,
            new_config.name
        );
        Ok(())
    }

    /// Remove an OBS connection
    pub async fn remove_connection(&self, name: &str) -> AppResult<()> {
        let client_arc = {
            let mut clients = self.clients.lock().await;
            match clients.remove(name) {
                Some(client) => client,
                None => {
                    return Err(AppError::ConfigError(format!(
                        "Connection '{name}' not found"
                    )))
                }
            }
        };

        self.stop_health_watcher(name).await;

        {
            let mut client = client_arc.lock().await;
            if let Err(e) = client.disconnect().await {
                log::warn!("Warning: Failed to disconnect client '{name}': {e}");
            }
        }

        {
            let mut default = self.default_connection.lock().await;
            if let Some(ref default_name) = *default {
                if default_name == name {
                    let next_default = {
                        let clients = self.clients.lock().await;
                        clients.keys().next().cloned()
                    };
                    *default = next_default;
                }
            }
        }

        log::info!("Removed OBS connection: {name}");
        Ok(())
    }

    /// Connect to an OBS instance
    pub async fn connect(&self, name: &str) -> AppResult<()> {
        let client_arc = {
            let clients = self.clients.lock().await;
            clients.get(name).cloned()
        }
        .ok_or_else(|| AppError::ConfigError(format!("Connection '{name}' not found")))?;

        {
            let mut client = client_arc.lock().await;
            client.connect().await?;
        }

        self.ensure_health_watcher(name).await?;

        log::info!("Connected to OBS: {name}");
        Ok(())
    }

    /// Disconnect from an OBS instance
    pub async fn disconnect(&self, name: &str) -> AppResult<()> {
        let clients = self.clients.lock().await;
        if let Some(client_arc) = clients.get(name) {
            let mut client = client_arc.lock().await;
            client.disconnect().await?;
            log::info!("Disconnected from OBS: {name}");
            Ok(())
        } else {
            Err(AppError::ConfigError(format!(
                "Connection '{name}' not found"
            )))
        }
    }

    /// Get connection status
    pub async fn get_connection_status(&self, name: &str) -> AppResult<ObsConnectionStatus> {
        let clients = self.clients.lock().await;
        if let Some(client_arc) = clients.get(name) {
            let client = client_arc.lock().await;
            Ok(client.get_connection_status())
        } else {
            Err(AppError::ConfigError(format!(
                "Connection '{name}' not found"
            )))
        }
    }

    /// Get a specific connection client
    pub async fn get_connection(&self, name: &str) -> AppResult<Arc<Mutex<ObsClient>>> {
        let clients = self.clients.lock().await;
        match clients.get(name) {
            Some(client) => Ok(client.clone()),
            None => Err(AppError::ConfigError(format!(
                "Connection '{name}' not found"
            ))),
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
                role: client.get_config().role.clone(),
                last_activity: None, // TODO: Track last activity
            });
        }

        Ok(connections)
    }

    /// Set default connection
    pub async fn set_default_connection(&self, name: &str) -> AppResult<()> {
        let clients = self.clients.lock().await;
        if !clients.contains_key(name) {
            return Err(AppError::ConfigError(format!(
                "Connection '{name}' not found"
            )));
        }

        let mut default = self.default_connection.lock().await;
        *default = Some(name.to_string());
        log::info!("Set default OBS connection: {name}");
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
                default
                    .as_ref()
                    .ok_or_else(|| {
                        AppError::ConfigError("No default OBS connection set".to_string())
                    })?
                    .clone()
            }
        };

        let clients = self.clients.lock().await;
        clients.get(&connection_name).cloned().ok_or_else(|| {
            AppError::ConfigError(format!("Connection '{connection_name}' not found"))
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

    pub async fn get_recording_status(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<super::types::ObsRecordingStatus> {
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

    pub async fn get_streaming_status(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<super::types::ObsStreamingStatus> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_streaming_status().await
    }

    pub async fn get_stream_output_stats(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<super::types::ObsStreamOutputStats> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_stream_output_stats().await
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

    pub async fn get_replay_buffer_status(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<super::types::ObsReplayBufferStatus> {
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

    pub async fn get_virtual_camera_status(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<super::types::ObsVirtualCameraStatus> {
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

    pub async fn set_current_scene(
        &self,
        scene_name: &str,
        connection_name: Option<&str>,
    ) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.set_current_scene(scene_name).await
    }

    pub async fn get_scenes(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<Vec<super::types::ObsScene>> {
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

    pub async fn get_version(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<super::types::ObsVersion> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_version().await
    }

    pub async fn get_stats(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<super::types::ObsStats> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_stats().await
    }

    // Configuration operations
    pub async fn set_record_directory(
        &self,
        directory: &str,
        connection_name: Option<&str>,
    ) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.set_record_directory(directory).await
    }

    pub async fn set_filename_formatting(
        &self,
        formatting: &str,
        connection_name: Option<&str>,
    ) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.set_filename_formatting(formatting).await
    }

    // Replay helpers
    pub async fn get_last_replay_filename(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<String> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_last_replay_filename().await
    }

    // Profile getters
    pub async fn get_record_directory(&self, connection_name: Option<&str>) -> AppResult<String> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_record_directory().await
    }

    pub async fn get_filename_formatting(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<String> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.get_filename_formatting().await
    }

    // Event handling
    pub async fn add_event_handler<F>(
        &self,
        event_type: String,
        handler: F,
        connection_name: Option<&str>,
    ) -> AppResult<()>
    where
        F: Fn(ObsEvent) + Send + Sync + 'static,
    {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.add_event_handler(event_type, handler).await
    }

    pub async fn remove_event_handler(
        &self,
        event_type: &str,
        connection_name: Option<&str>,
    ) -> AppResult<()> {
        let client_arc = self.get_client_ref(connection_name).await?;
        let client = client_arc.lock().await;
        client.remove_event_handler(event_type).await
    }

    /// Shutdown all connections
    pub async fn shutdown(&self) -> AppResult<()> {
        self.stop_all_health_tasks().await;

        let mut clients = self.clients.lock().await;
        for (name, client_arc) in clients.iter_mut() {
            let mut client = client_arc.lock().await;
            if let Err(e) = client.disconnect().await {
                log::warn!("Warning: Failed to disconnect client '{name}': {e}");
            }
        }
        clients.clear();

        let mut default = self.default_connection.lock().await;
        *default = None;

        log::info!("OBS Manager shutdown complete");
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

    /// Get audio sources for a specific connection
    pub async fn get_audio_sources(
        &self,
        connection_name: Option<&str>,
    ) -> AppResult<Vec<super::types::ObsSource>> {
        let client = self.get_client_ref(connection_name).await?;
        let client_guard = client.lock().await;

        client_guard.get_audio_sources().await
    }

    /// Set mute status for a source on a specific connection
    pub async fn set_source_mute(
        &self,
        _source_name: &str,
        _muted: bool,
        _connection_name: Option<&str>,
    ) -> AppResult<()> {
        // Individual input mute control is not available in obws
        Err(AppError::ConfigError(
            "Individual input mute control not supported by obws".to_string(),
        ))
    }

    /// Set volume for a source on a specific connection
    pub async fn set_source_volume(
        &self,
        _source_name: &str,
        _volume: f64,
        _connection_name: Option<&str>,
    ) -> AppResult<()> {
        // Individual input volume control is not available in obws
        Err(AppError::ConfigError(
            "Individual input volume control not supported by obws".to_string(),
        ))
    }

    /// Execute custom operation on a specific connection
    pub async fn execute_custom_operation(
        &self,
        request: super::types::ObsOperationRequest,
        connection_name: Option<&str>,
    ) -> AppResult<super::types::ObsOperationResponse> {
        let client = self.get_client_ref(connection_name).await?;
        let client_guard = client.lock().await;

        client_guard.execute_custom_operation(request).await
    }

    /// Execute raw OBS WebSocket request on a specific connection
    pub async fn execute_raw_request(
        &self,
        request_type: &str,
        request_data: serde_json::Value,
        connection_name: Option<&str>,
    ) -> AppResult<serde_json::Value> {
        let client = self.get_client_ref(connection_name).await?;
        let client_guard = client.lock().await;

        client_guard
            .execute_raw_request(request_type, request_data)
            .await
    }

    /// Set up status listener for all connections
    pub async fn setup_status_listener(&self) -> AppResult<()> {
        let clients = self.clients.lock().await;
        for (name, client_arc) in clients.iter() {
            let client = client_arc.lock().await;
            if let Err(e) = client.setup_status_listener().await {
                log::warn!("Warning: Failed to set up status listener for '{name}': {e}");
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
