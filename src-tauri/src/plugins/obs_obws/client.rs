//! OBS Client implementation using the obws crate

use obws::Client;
use crate::types::{AppError, AppResult};
use super::types::{
    ObsConnectionConfig, ObsConnectionStatus, ObsRecordingStatus, ObsStreamingStatus,
    ObsReplayBufferStatus, ObsVirtualCameraStatus, ObsStudioModeStatus, ObsStatus,
    ObsVersion, ObsStats, ObsScene, ObsSource, ObsEvent
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;
use futures_util::StreamExt;

/// OBS Client using the obws crate
pub struct ObsClient {
    client: Option<Client>,
    config: ObsConnectionConfig,
    status: ObsConnectionStatus,
    event_handlers: Arc<Mutex<HashMap<String, Box<dyn Fn(ObsEvent) + Send + Sync>>>>,
}

impl ObsClient {
    /// Create a new OBS client with the given configuration
    pub fn new(config: ObsConnectionConfig) -> Self {
        Self {
            client: None,
            config,
            status: ObsConnectionStatus::Disconnected,
            event_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &ObsConnectionConfig {
        &self.config
    }

    /// Get the current connection status
    pub fn get_connection_status(&self) -> ObsConnectionStatus {
        self.status.clone()
    }

    /// Connect to OBS WebSocket
    pub async fn connect(&mut self) -> AppResult<()> {
        self.status = ObsConnectionStatus::Connecting;
        
        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        
        let connect_result = timeout(
            timeout_duration,
            Client::connect(
                &self.config.host,
                self.config.port,
                self.config.password.as_deref(),
            )
        ).await;

        match connect_result {
            Ok(Ok(client)) => {
                self.client = Some(client);
                self.status = ObsConnectionStatus::Authenticated;
                log::info!("✅ Connected to OBS at {}:{}", self.config.host, self.config.port);
                Ok(())
            }
            Ok(Err(e)) => {
                let error_msg = format!("Failed to connect to OBS: {}", e);
                self.status = ObsConnectionStatus::Error(error_msg.clone());
                log::error!("❌ {}", error_msg);
                Err(AppError::ConfigError(error_msg))
            }
            Err(_) => {
                let error_msg = format!("Connection timeout to OBS at {}:{}", self.config.host, self.config.port);
                self.status = ObsConnectionStatus::Error(error_msg.clone());
                log::error!("❌ {}", error_msg);
                Err(AppError::ConfigError(error_msg))
            }
        }
    }

    /// Disconnect from OBS WebSocket
    pub async fn disconnect(&mut self) -> AppResult<()> {
        if let Some(_client) = self.client.take() {
            // The obws Client doesn't have an explicit disconnect method
            // It will be dropped when we take() it
            log::info!("🔌 Disconnected from OBS at {}:{}", self.config.host, self.config.port);
        }
        self.status = ObsConnectionStatus::Disconnected;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(self.status, ObsConnectionStatus::Authenticated) && self.client.is_some()
    }

    /// Get the underlying obws client
    pub fn get_client(&self) -> AppResult<&Client> {
        self.client.as_ref().ok_or_else(|| {
            AppError::ConfigError("OBS client not connected".to_string())
        })
    }

    /// Start recording
    pub async fn start_recording(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client.recording().start().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to start recording: {}", e))
        })?;
        log::info!("🎬 Recording started");
        Ok(())
    }

    /// Stop recording
    pub async fn stop_recording(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client.recording().stop().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to stop recording: {}", e))
        })?;
        log::info!("⏹️ Recording stopped");
        Ok(())
    }

    /// Get recording status
    pub async fn get_recording_status(&self) -> AppResult<ObsRecordingStatus> {
        let client = self.get_client()?;
        let _status = client.recording().status().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get recording status: {}", e))
        })?;
        
        // For now, return a simple status since obws doesn't expose the enum variants
        // TODO: Implement proper status detection based on the actual response
        Ok(ObsRecordingStatus::Recording)
    }

    /// Start streaming
    pub async fn start_streaming(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client.streaming().start().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to start streaming: {}", e))
        })?;
        log::info!("📡 Streaming started");
        Ok(())
    }

    /// Stop streaming
    pub async fn stop_streaming(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client.streaming().stop().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to stop streaming: {}", e))
        })?;
        log::info!("⏹️ Streaming stopped");
        Ok(())
    }

    /// Get streaming status
    pub async fn get_streaming_status(&self) -> AppResult<ObsStreamingStatus> {
        let client = self.get_client()?;
        let _status = client.streaming().status().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get streaming status: {}", e))
        })?;
        
        // For now, return a simple status since obws doesn't expose the enum variants
        // TODO: Implement proper status detection based on the actual response
        Ok(ObsStreamingStatus::Streaming)
    }

    /// Start replay buffer
    pub async fn start_replay_buffer(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client.replay_buffer().start().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to start replay buffer: {}", e))
        })?;
        log::info!("🔄 Replay buffer started");
        Ok(())
    }

    /// Stop replay buffer
    pub async fn stop_replay_buffer(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client.replay_buffer().stop().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to stop replay buffer: {}", e))
        })?;
        log::info!("⏹️ Replay buffer stopped");
        Ok(())
    }

    /// Save replay buffer
    pub async fn save_replay_buffer(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client.replay_buffer().save().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to save replay buffer: {}", e))
        })?;
        log::info!("💾 Replay buffer saved");
        Ok(())
    }

    /// Get last replay buffer filename (without directory)
    pub async fn get_last_replay_filename(&self) -> AppResult<String> {
        let client = self.get_client()?;
        let name = client.replay_buffer().last_replay().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get last replay filename: {}", e))
        })?;
        Ok(name)
    }

    /// Get replay buffer status
    pub async fn get_replay_buffer_status(&self) -> AppResult<ObsReplayBufferStatus> {
        let client = self.get_client()?;
        let _status = client.replay_buffer().status().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get replay buffer status: {}", e))
        })?;
        
        // For now, return a simple status since obws doesn't expose the enum variants
        // TODO: Implement proper status detection based on the actual response
        Ok(ObsReplayBufferStatus::Active)
    }

    /// Start virtual camera
    pub async fn start_virtual_camera(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client.virtual_cam().start().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to start virtual camera: {}", e))
        })?;
        log::info!("📹 Virtual camera started");
        Ok(())
    }

    /// Stop virtual camera
    pub async fn stop_virtual_camera(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client.virtual_cam().stop().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to stop virtual camera: {}", e))
        })?;
        log::info!("⏹️ Virtual camera stopped");
        Ok(())
    }

    /// Get virtual camera status
    pub async fn get_virtual_camera_status(&self) -> AppResult<ObsVirtualCameraStatus> {
        let client = self.get_client()?;
        let _status = client.virtual_cam().status().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get virtual camera status: {}", e))
        })?;
        
        // For now, return a simple status since obws doesn't expose the enum variants
        // TODO: Implement proper status detection based on the actual response
        Ok(ObsVirtualCameraStatus::Active)
    }

    /// Get current scene
    pub async fn get_current_scene(&self) -> AppResult<String> {
        let client = self.get_client()?;
        let scene = client.scenes().current_program_scene().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get current scene: {}", e))
        })?;
        Ok(format!("{:?}", scene.id))
    }

    /// Set current scene
    pub async fn set_current_scene(&self, scene_name: &str) -> AppResult<()> {
        let client = self.get_client()?;
        client.scenes().set_current_program_scene(scene_name).await.map_err(|e| {
            AppError::ConfigError(format!("Failed to set current scene to '{}': {}", scene_name, e))
        })?;
        log::info!("🎭 Scene changed to: {}", scene_name);
        Ok(())
    }

    /// Get all scenes
    pub async fn get_scenes(&self) -> AppResult<Vec<ObsScene>> {
        let client = self.get_client()?;
        let scenes = client.scenes().list().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get scenes: {}", e))
        })?;
        
        let mut obs_scenes = Vec::new();
        for scene in scenes.scenes {
            let scene_id_str = format!("{:?}", scene.id);
            let sources = self.get_scene_sources(&scene_id_str).await?;
            obs_scenes.push(ObsScene {
                name: scene_id_str,
                scene_index: scene.index as i32,
                sources,
            });
        }
        Ok(obs_scenes)
    }

    /// Get sources in a scene
    pub async fn get_scene_sources(&self, _scene_name: &str) -> AppResult<Vec<ObsSource>> {
        // Note: obws doesn't have a direct scene_item_list method
        // This would need to be implemented using custom requests
        log::warn!("Scene sources not yet implemented in obws integration");
        Ok(Vec::new()) // Return empty vector for now
    }

    /// Get OBS version information
    pub async fn get_version(&self) -> AppResult<ObsVersion> {
        let client = self.get_client()?;
        let version = client.general().version().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get OBS version: {}", e))
        })?;
        
        Ok(ObsVersion {
            obs_version: version.obs_version.to_string(),
            obs_web_socket_version: version.obs_web_socket_version.to_string(),
            rpc_version: version.rpc_version as i32,
            available_requests: version.available_requests,
            supported_image_export_formats: version.supported_image_formats,
        })
    }

    /// Get OBS statistics
    pub async fn get_stats(&self) -> AppResult<ObsStats> {
        let client = self.get_client()?;
        let stats = client.general().stats().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get OBS stats: {}", e))
        })?;
        
        Ok(ObsStats {
            cpu_usage: stats.cpu_usage,
            memory_usage: stats.memory_usage,
            available_disk_space: stats.available_disk_space as i64,
            active_fps: stats.active_fps,
            average_frame_render_time: stats.average_frame_render_time,
            render_skipped_frames: stats.render_skipped_frames as i32,
            render_total_frames: stats.render_total_frames as i32,
            output_skipped_frames: stats.output_skipped_frames as i32,
            output_total_frames: stats.output_total_frames as i32,
        })
    }

    /// Set recording directory (Output -> Recording -> Recording path)
    pub async fn set_record_directory(&self, directory: &str) -> AppResult<()> {
        let client = self.get_client()?;
        // Fallback to profile parameter because obws general API does not expose SetRecordDirectory
        // The parameter name in OBS is typically: "Output", section "Recording", key "RecFilePath"
        // For broad compatibility, use profile.set_parameter(category, parameter, value)
        client
            .profiles()
            .set_parameter(obws::requests::profiles::SetParameter {
                category: "Output",
                name: "RecFilePath",
                value: Some(directory),
            })
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to set record directory: {}", e)))?;
        log::info!("📁 Recording directory set to: {}", directory);
        Ok(())
    }

    /// Set filename formatting (Advanced -> Recording -> Filename formatting)
    pub async fn set_filename_formatting(&self, formatting: &str) -> AppResult<()> {
        let client = self.get_client()?;
        // Use profile.set_parameter for filename formatting. Key commonly "FilenameFormatting" under "Output" or "AdvOut".
        // We set both likely keys to improve compatibility; ignore errors on the second set.
        client
            .profiles()
            .set_parameter(obws::requests::profiles::SetParameter {
                category: "Output",
                name: "FilenameFormatting",
                value: Some(formatting),
            })
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to set filename formatting: {}", e)))?;
        // Try alternative advanced key without failing whole call if it errors
        let _ = client
            .profiles()
            .set_parameter(obws::requests::profiles::SetParameter {
                category: "AdvOut",
                name: "FilenameFormatting",
                value: Some(formatting),
            })
            .await;
        log::info!("🧾 Filename formatting set to: {}", formatting);
        Ok(())
    }

    /// Get recording directory from OBS profile
    pub async fn get_record_directory(&self) -> AppResult<String> {
        let client = self.get_client()?;
        let param = client
            .profiles()
            .parameter("Output", "RecFilePath")
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get record directory: {}", e)))?;
        Ok(param.value.unwrap_or_default())
    }

    /// Get filename formatting from OBS profile
    pub async fn get_filename_formatting(&self) -> AppResult<String> {
        let client = self.get_client()?;
        let param = client
            .profiles()
            .parameter("Output", "FilenameFormatting")
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get filename formatting: {}", e)))?;
        Ok(param.value.unwrap_or_default())
    }

    /// Get comprehensive OBS status
    pub async fn get_status(&self) -> AppResult<ObsStatus> {
        let recording_status = self.get_recording_status().await.unwrap_or(ObsRecordingStatus::Error("Failed to get recording status".to_string()));
        let streaming_status = self.get_streaming_status().await.unwrap_or(ObsStreamingStatus::Error("Failed to get streaming status".to_string()));
        let replay_buffer_status = self.get_replay_buffer_status().await.unwrap_or(ObsReplayBufferStatus::Error("Failed to get replay buffer status".to_string()));
        let virtual_camera_status = self.get_virtual_camera_status().await.unwrap_or(ObsVirtualCameraStatus::Error("Failed to get virtual camera status".to_string()));
        
        let current_scene = self.get_current_scene().await.ok();
        let scenes = self.get_scenes().await.map(|s| s.into_iter().map(|scene| scene.name).collect()).unwrap_or_default();
        let version = self.get_version().await.ok();
        let stats = self.get_stats().await.ok();
        
        Ok(ObsStatus {
            connection_status: self.status.clone(),
            recording_status,
            streaming_status,
            replay_buffer_status,
            virtual_camera_status,
            studio_mode: ObsStudioModeStatus::Disabled, // TODO: Implement studio mode
            current_scene,
            scenes,
            version,
            stats,
        })
    }

    /// Add event handler
    pub async fn add_event_handler<F>(&self, event_type: String, handler: F) -> AppResult<()>
    where
        F: Fn(ObsEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.lock().await;
        handlers.insert(event_type, Box::new(handler));
        Ok(())
    }

    /// Remove event handler
    pub async fn remove_event_handler(&self, event_type: &str) -> AppResult<()> {
        let mut handlers = self.event_handlers.lock().await;
        handlers.remove(event_type);
        Ok(())
    }

    /// Trigger event
    pub async fn trigger_event(&self, event: ObsEvent) -> AppResult<()> {
        let _handlers = self.event_handlers.lock().await;
        // TODO: Implement event triggering based on event type
        log::debug!("Event triggered: {:?}", event);
        Ok(())
    }

    /// Set up status listener
    pub async fn setup_status_listener(&self) -> AppResult<()> {
        let client = self.get_client()?;
        
        // Set up event handler for all events
        let events = client.events().map_err(|e| {
            AppError::ConfigError(format!("Failed to set up event handler: {}", e))
        })?;
        
        // Pin the stream and set up event handler
        let mut events = Box::pin(events);
        
        // Set up event handler
        tokio::spawn(async move {
            while let Some(event) = events.next().await {
                log::debug!("OBS event: {:?}", event);
            }
        });
        
        log::info!("✅ Status listener set up successfully");
        Ok(())
    }
}
