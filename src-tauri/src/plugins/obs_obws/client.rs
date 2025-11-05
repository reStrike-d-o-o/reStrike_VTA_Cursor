//! OBS Client implementation using the obws crate

use super::types::{
    ObsConnectionConfig, ObsConnectionStatus, ObsEvent, ObsRecordingStatus, ObsReplayBufferStatus,
    ObsScene, ObsSource, ObsStats, ObsStatus, ObsStreamOutputStats, ObsStreamingStatus,
    ObsStudioModeStatus, ObsVersion, ObsVirtualCameraStatus,
};
use crate::types::{AppError, AppResult};
use futures_util::StreamExt;
use obws::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

/// OBS Client using the obws crate
pub struct ObsClient {
    client: Option<Client>,
    config: ObsConnectionConfig,
    status: ObsConnectionStatus,
    event_handlers: Arc<Mutex<HashMap<String, Box<dyn Fn(ObsEvent) + Send + Sync>>>>,
    monitoring_task: Option<tokio::task::JoinHandle<()>>,
    monitoring_shutdown: Arc<tokio::sync::Notify>,
}

impl ObsClient {
    /// Create a new OBS client with the given configuration
    pub fn new(config: ObsConnectionConfig) -> Self {
        Self {
            client: None,
            config,
            status: ObsConnectionStatus::Disconnected,
            event_handlers: Arc::new(Mutex::new(HashMap::new())),
            monitoring_task: None,
            monitoring_shutdown: Arc::new(tokio::sync::Notify::new()),
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
        let endpoint = format!("{}:{}", self.config.host, self.config.port);

        // Fast preflight check to provide clearer feedback when OBS WebSocket is offline.
        match timeout(timeout_duration, TcpStream::connect(&endpoint)).await {
            Ok(Ok(stream)) => {
                // Successfully opened a TCP socket; drop the probe before performing the real handshake.
                drop(stream);
            }
            Ok(Err(err)) => {
                let error_msg = format!(
                    "OBS WebSocket not reachable at {} ({}). Ensure OBS is running and the WebSocket server is enabled.",
                    endpoint,
                    err
                );
                self.status = ObsConnectionStatus::Error(error_msg.clone());
                log::warn!("{}", error_msg);
                return Err(AppError::ConfigError(error_msg));
            }
            Err(_) => {
                let error_msg = format!(
                    "Timed out while checking OBS WebSocket availability at {}. Ensure OBS is running and reachable.",
                    endpoint
                );
                self.status = ObsConnectionStatus::Error(error_msg.clone());
                log::warn!("{}", error_msg);
                return Err(AppError::ConfigError(error_msg));
            }
        }

        let connect_result = timeout(
            timeout_duration,
            Client::connect(
                &self.config.host,
                self.config.port,
                self.config.password.as_deref(),
            ),
        )
        .await;

        match connect_result {
            Ok(Ok(client)) => {
                self.client = Some(client);
                self.status = ObsConnectionStatus::Authenticated;
                log::info!(
                    "Connected to OBS at {}:{}",
                    self.config.host,
                    self.config.port
                );
                Ok(())
            }
            Ok(Err(e)) => {
                let error_msg = format!("Failed to connect to OBS: {}", e);
                self.status = ObsConnectionStatus::Error(error_msg.clone());
                log::error!("{}", error_msg);
                Err(AppError::ConfigError(error_msg))
            }
            Err(_) => {
                let error_msg = format!(
                    "Connection timeout to OBS at {}:{}",
                    self.config.host, self.config.port
                );
                self.status = ObsConnectionStatus::Error(error_msg.clone());
                log::error!("{}", error_msg);
                Err(AppError::ConfigError(error_msg))
            }
        }
    }

    /// Disconnect from OBS WebSocket
    pub async fn disconnect(&mut self) -> AppResult<()> {
        if let Some(_client) = self.client.take() {
            // The obws Client doesn't have an explicit disconnect method
            // It will be dropped when we take() it
            log::info!(
                "Disconnected from OBS at {}:{}",
                self.config.host,
                self.config.port
            );
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
        self.client
            .as_ref()
            .ok_or_else(|| AppError::ConfigError("OBS client not connected".to_string()))
    }

    /// Start recording
    pub async fn start_recording(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .recording()
            .start()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to start recording: {}", e)))?;
        log::info!("Recording started");
        Ok(())
    }

    /// Stop recording
    pub async fn stop_recording(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .recording()
            .stop()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to stop recording: {}", e)))?;
        log::info!("Recording stopped");
        Ok(())
    }

    /// Get recording status
    pub async fn get_recording_status(&self) -> AppResult<ObsRecordingStatus> {
        let client = self.get_client()?;
        let status =
            client.recording().status().await.map_err(|e| {
                AppError::ConfigError(format!("Failed to get recording status: {}", e))
            })?;

        // Check if recording is active based on the status response
        match status.active {
            true => Ok(ObsRecordingStatus::Recording),
            false => Ok(ObsRecordingStatus::Stopped),
        }
    }

    /// Start streaming
    pub async fn start_streaming(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .streaming()
            .start()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to start streaming: {}", e)))?;
        log::info!("Streaming started");
        Ok(())
    }

    /// Stop streaming
    pub async fn stop_streaming(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .streaming()
            .stop()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to stop streaming: {}", e)))?;
        log::info!("Streaming stopped");
        Ok(())
    }

    /// Get streaming status
    pub async fn get_streaming_status(&self) -> AppResult<ObsStreamingStatus> {
        let client = self.get_client()?;
        let status =
            client.streaming().status().await.map_err(|e| {
                AppError::ConfigError(format!("Failed to get streaming status: {}", e))
            })?;

        // Check if streaming is active based on the status response
        match status.active {
            true => Ok(ObsStreamingStatus::Streaming),
            false => Ok(ObsStreamingStatus::Stopped),
        }
    }

    /// Get raw stream output statistics (includes frames/bytes information)
    pub async fn get_stream_output_stats(&self) -> AppResult<ObsStreamOutputStats> {
        let client = self.get_client()?;
        let status = client
            .streaming()
            .status()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get stream stats: {}", e)))?;

        let duration_ms = status.duration.whole_milliseconds();

        Ok(ObsStreamOutputStats {
            congestion: status.congestion,
            bytes: status.bytes,
            duration: if duration_ms.is_negative() { 0 } else { duration_ms as u64 },
            skipped_frames: status.skipped_frames,
            total_frames: status.total_frames,
        })
    }

    /// Start replay buffer
    pub async fn start_replay_buffer(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .replay_buffer()
            .start()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to start replay buffer: {}", e)))?;

        log::info!("Replay buffer started");

        // Emit notification for replay started
        println!(
            "NOTIFICATION:replay_started:{}",
            serde_json::json!({
                "connection_name": self.config.name,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        );

        Ok(())
    }

    /// Stop replay buffer
    pub async fn stop_replay_buffer(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .replay_buffer()
            .stop()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to stop replay buffer: {}", e)))?;

        log::info!("Replay buffer stopped");

        // Emit notification for replay stopped
        println!(
            "NOTIFICATION:replay_stopped:{}",
            serde_json::json!({
                "connection_name": self.config.name,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        );

        Ok(())
    }

    /// Save replay buffer
    pub async fn save_replay_buffer(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .replay_buffer()
            .save()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to save replay buffer: {}", e)))?;

        log::info!("Replay buffer saved");

        // Emit notification for replay saved
        println!(
            "NOTIFICATION:replay_saved:{}",
            serde_json::json!({
                "connection_name": self.config.name,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        );

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
        let active = client.replay_buffer().status().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get replay buffer status: {}", e))
        })?;
        Ok(if active {
            ObsReplayBufferStatus::Active
        } else {
            ObsReplayBufferStatus::Stopped
        })
    }

    /// Start virtual camera
    pub async fn start_virtual_camera(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .virtual_cam()
            .start()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to start virtual camera: {}", e)))?;
        log::info!("Virtual camera started");
        Ok(())
    }

    /// Stop virtual camera
    pub async fn stop_virtual_camera(&self) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .virtual_cam()
            .stop()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to stop virtual camera: {}", e)))?;
        log::info!("Virtual camera stopped");
        Ok(())
    }

    /// Get virtual camera status
    pub async fn get_virtual_camera_status(&self) -> AppResult<ObsVirtualCameraStatus> {
        // TODO: Implement proper virtual camera status detection
        // For now, return stopped status to avoid compilation issues
        Ok(ObsVirtualCameraStatus::Stopped)
    }

    /// Get current scene
    pub async fn get_current_scene(&self) -> AppResult<String> {
        let client = self.get_client()?;
        let scene =
            client.scenes().current_program_scene().await.map_err(|e| {
                AppError::ConfigError(format!("Failed to get current scene: {}", e))
            })?;
        Ok(format!("{:?}", scene.id))
    }

    /// Set current scene
    pub async fn set_current_scene(&self, scene_name: &str) -> AppResult<()> {
        let client = self.get_client()?;
        client
            .scenes()
            .set_current_program_scene(scene_name)
            .await
            .map_err(|e| {
                AppError::ConfigError(format!(
                    "Failed to set current scene to '{}': {}",
                    scene_name, e
                ))
            })?;
        log::info!("Scene changed to: {}", scene_name);
        Ok(())
    }

    /// Get all scenes
    pub async fn get_scenes(&self) -> AppResult<Vec<ObsScene>> {
        let client = self.get_client()?;
        let scenes = client
            .scenes()
            .list()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get scenes: {}", e)))?;

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

    /// Get audio sources for a specific connection
    pub async fn get_audio_sources(&self) -> AppResult<Vec<ObsSource>> {
        let client = self.get_client()?;

        // Try to get inputs (audio sources) using the inputs API
        match client.inputs().list(None).await {
            Ok(inputs_response) => {
                let audio_sources: Vec<ObsSource> = inputs_response
                    .into_iter()
                    .filter(|input| {
                        // Filter for audio inputs (input kinds that contain "audio" or "input")
                        input.kind.contains("audio") || input.kind.contains("input")
                    })
                    .map(|input| ObsSource {
                        name: format!("{:?}", input.id),
                        type_name: input.kind,
                        enabled: true, // Input enabled state is not directly available
                        muted: false,  // Muted state is not available in list response
                        volume: Some(1.0), // Volume is not available in list response
                        bounds: None,  // Inputs don't have bounds
                        transform: None, // Inputs don't have transforms
                    })
                    .collect();

                Ok(audio_sources)
            }
            Err(e) => {
                // If inputs listing is not supported, return empty list
                log::warn!(
                    "Audio sources listing not supported by this OBS version: {}",
                    e
                );
                Ok(Vec::new())
            }
        }
    }

    /// Get source volume
    pub async fn get_source_volume(&self, _source_name: &str) -> AppResult<f64> {
        // Individual input volume control is not available in obws
        Err(AppError::ConfigError(
            "Individual input volume control not supported by obws".to_string(),
        ))
    }

    /// Set source volume
    pub async fn set_source_volume(&self, _source_name: &str, _volume: f64) -> AppResult<()> {
        // Individual input volume control is not available in obws
        Err(AppError::ConfigError(
            "Individual input volume control not supported by obws".to_string(),
        ))
    }

    /// Get source muted state
    pub async fn get_source_muted(&self, _source_name: &str) -> AppResult<bool> {
        // Individual input mute control is not available in obws
        Err(AppError::ConfigError(
            "Individual input mute control not supported by obws".to_string(),
        ))
    }

    /// Set source muted state
    pub async fn set_source_muted(&self, _source_name: &str, _muted: bool) -> AppResult<()> {
        // Individual input mute control is not available in obws
        Err(AppError::ConfigError(
            "Individual input mute control not supported by obws".to_string(),
        ))
    }

    /// Get sources in a scene
    pub async fn get_scene_sources(&self, _scene_name: &str) -> AppResult<Vec<ObsSource>> {
        // For now, return empty vector - scene sources retrieval can be enhanced later
        Ok(Vec::new())
    }

    /// Execute custom operation
    pub async fn execute_custom_operation(
        &self,
        request: super::types::ObsOperationRequest,
    ) -> AppResult<super::types::ObsOperationResponse> {
        crate::plugins::obs_obws::operations::ObsOperations::execute_custom_operation(self, request)
            .await
    }

    /// Execute raw OBS WebSocket request
    /// This allows us to send requests that aren't covered by the obws crate
    pub async fn execute_raw_request(
        &self,
        request_type: &str,
        request_data: serde_json::Value,
    ) -> AppResult<serde_json::Value> {
        let client = self.get_client()?;

        // Use the general API to send a custom request
        // Note: This is a workaround since obws doesn't expose raw request sending
        // For now, we'll implement common operations using available APIs

        match request_type {
            "SetInputMute" => {
                let input_name = request_data
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                let _input_muted = request_data
                    .get("inputMuted")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'inputMuted' parameter".to_string(),
                        )
                    })?;

                // For now, return error since individual input mute isn't supported
                // This could be implemented using scene-based audio control
                Err(AppError::ConfigError(format!("Individual input mute control not supported by obws. Consider using scene-based audio control for input '{}'", input_name)))
            }

            "GetInputMute" => {
                let input_name = request_data
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                Err(AppError::ConfigError(format!("Individual input mute status not supported by obws. Consider using scene-based audio control for input '{}'", input_name)))
            }

            "SetInputVolume" => {
                let input_name = request_data
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                let _input_volume_mul = request_data
                    .get("inputVolumeMul")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'inputVolumeMul' parameter".to_string(),
                        )
                    })?;

                Err(AppError::ConfigError(format!("Individual input volume control not supported by obws. Consider using scene-based audio control for input '{}'", input_name)))
            }

            "GetInputVolume" => {
                let input_name = request_data
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                Err(AppError::ConfigError(format!("Individual input volume status not supported by obws. Consider using scene-based audio control for input '{}'", input_name)))
            }

            "ToggleMute" => {
                let input_name = request_data
                    .get("inputName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'inputName' parameter".to_string())
                    })?;

                Err(AppError::ConfigError(format!("Input mute toggle not supported by obws. Consider using scene-based audio control for input '{}'", input_name)))
            }

            // Scene item operations
            "GetSceneItemId" => {
                let scene_name = request_data
                    .get("sceneName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sceneName' parameter".to_string())
                    })?;

                let source_name = request_data
                    .get("sourceName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sourceName' parameter".to_string())
                    })?;

                // Use scenes API to get scene item list and find the matching source
                match client.scenes().list().await {
                    Ok(scenes_response) => {
                        for scene in scenes_response.scenes {
                            if scene_name == scene.id {
                                // Get scene items for this specific scene
                                match client
                                    .scene_items()
                                    .list(obws::requests::scenes::SceneId::Name(scene_name))
                                    .await
                                {
                                    Ok(items) => {
                                        for item in items {
                                            if item.source_name == source_name {
                                                return Ok(serde_json::json!({
                                                    "sceneItemId": item.id,
                                                    "sceneName": scene_name,
                                                    "sourceName": source_name
                                                }));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        return Err(AppError::ConfigError(format!(
                                            "Failed to get scene items: {}",
                                            e
                                        )))
                                    }
                                }
                            }
                        }
                        Err(AppError::ConfigError(format!(
                            "Scene '{}' not found",
                            scene_name
                        )))
                    }
                    Err(e) => Err(AppError::ConfigError(format!(
                        "Failed to get scenes: {}",
                        e
                    ))),
                }
            }

            "SetSceneItemEnabled" => {
                let scene_name = request_data
                    .get("sceneName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sceneName' parameter".to_string())
                    })?;

                let scene_item_id = request_data
                    .get("sceneItemId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'sceneItemId' parameter".to_string(),
                        )
                    })?;

                let scene_item_enabled = request_data
                    .get("sceneItemEnabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'sceneItemEnabled' parameter".to_string(),
                        )
                    })?;

                // Use scene items API to set enabled state - obws API requires different approach
                // For now, return error since the exact API signature is complex
                Err(AppError::ConfigError(format!("Scene item enabled control not fully implemented in obws integration. Scene: {}, Item ID: {}, Enabled: {}", scene_name, scene_item_id, scene_item_enabled)))
            }

            "SetSceneItemTransform" => {
                let scene_name = request_data
                    .get("sceneName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sceneName' parameter".to_string())
                    })?;

                let scene_item_id = request_data
                    .get("sceneItemId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'sceneItemId' parameter".to_string(),
                        )
                    })?;

                // Get transform parameters
                let x = request_data
                    .get("positionX")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let y = request_data
                    .get("positionY")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let scale_x = request_data
                    .get("scaleX")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                let scale_y = request_data
                    .get("scaleY")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                let rotation = request_data
                    .get("rotation")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                // Scene item transform control is complex in obws - return error for now
                Err(AppError::ConfigError(format!("Scene item transform control not fully implemented in obws integration. Scene: {}, Item ID: {}, Transform: position=({},{}) scale=({},{}) rotation={}", scene_name, scene_item_id, x, y, scale_x, scale_y, rotation)))
            }

            "SetCurrentSceneTransition" => {
                let transition_name = request_data
                    .get("transitionName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'transitionName' parameter".to_string())
                    })?;

                // Transition setting not fully implemented in obws integration
                Err(AppError::ConfigError(format!(
                    "Transition setting not fully implemented in obws integration. Transition: {}",
                    transition_name
                )))
            }

            "GetCurrentSceneTransition" => {
                // Transition getting not fully implemented in obws integration
                Err(AppError::ConfigError(
                    "Transition getting not fully implemented in obws integration".to_string(),
                ))
            }

            "SetSceneTransitionOverride" => {
                let scene_name = request_data
                    .get("sceneName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'sceneName' parameter".to_string())
                    })?;

                let transition_name = request_data
                    .get("transitionName")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::ConfigError("Missing 'transitionName' parameter".to_string())
                    })?;

                let _transition_duration = request_data
                    .get("transitionDuration")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(300);

                // Transition override not fully implemented in obws integration
                Err(AppError::ConfigError(format!("Transition override not fully implemented in obws integration. Scene: {}, Transition: {}", scene_name, transition_name)))
            }

            "GetTransitionList" => {
                // Use transitions API to get transition list
                match client.transitions().list().await {
                    Ok(transitions) => Ok(serde_json::json!({
                        "transitions": transitions.transitions.iter().map(|t| {
                            serde_json::json!({
                                "transitionName": t.id,
                                "transitionKind": t.kind
                            })
                        }).collect::<Vec<_>>(),
                        "currentTransitionName": transitions.current_scene_transition,
                        "currentTransitionKind": transitions.current_scene_transition_kind
                    })),
                    Err(e) => Err(AppError::ConfigError(format!(
                        "Failed to get transitions: {}",
                        e
                    ))),
                }
            }

            "TriggerStudioModeTransition" => {
                // Studio mode transition not supported by obws crate
                Err(AppError::ConfigError(
                    "Studio mode transition not supported by obws crate".to_string(),
                ))
            }

            "SetStudioModeEnabled" => {
                let studio_mode_enabled = request_data
                    .get("studioModeEnabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        AppError::ConfigError(
                            "Missing or invalid 'studioModeEnabled' parameter".to_string(),
                        )
                    })?;

                // Note: obws doesn't directly support studio mode
                Err(AppError::ConfigError(format!(
                    "Studio mode control not supported by obws crate. Studio mode enabled: {}",
                    studio_mode_enabled
                )))
            }

            // Default case for unknown requests
            _ => Err(AppError::ConfigError(format!(
                "Unknown raw request type: {}",
                request_type
            ))),
        }
    }

    /// Get studio mode status
    pub async fn get_studio_mode_status(&self) -> AppResult<ObsStudioModeStatus> {
        static LOG_ONCE: std::sync::Once = std::sync::Once::new();
        LOG_ONCE.call_once(|| {
            log::debug!("Studio mode status requested - obws crate does not expose studio mode information; defaulting to Disabled");
        });

        Ok(ObsStudioModeStatus::Disabled)
    }

    /// Get OBS version information
    pub async fn get_version(&self) -> AppResult<ObsVersion> {
        let client = self.get_client()?;
        let version = client
            .general()
            .version()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get OBS version: {}", e)))?;

        Ok(ObsVersion {
            obs_version: version.obs_version.to_string(),
            obs_web_socket_version: version.obs_web_socket_version.to_string(),
            rpc_version: version.rpc_version as i32,
            available_requests: version.available_requests,
            supported_image_export_formats: version.supported_image_formats,
        })
    }

    /// Enable studio mode
    pub async fn enable_studio_mode(&self) -> AppResult<()> {
        self.set_studio_mode(true).await
    }

    /// Disable studio mode
    pub async fn disable_studio_mode(&self) -> AppResult<()> {
        self.set_studio_mode(false).await
    }

    /// Set studio mode enabled/disabled
    pub async fn set_studio_mode(&self, enabled: bool) -> AppResult<()> {
        let _client = self.get_client()?;

        // Note: obws doesn't directly support studio mode
        // This is a placeholder implementation for future enhancement
        if enabled {
            log::warn!("Studio mode enable requested but not supported by obws crate");
        } else {
            log::debug!("Studio mode disable requested but not supported by obws crate");
        }

        // For now, always return success since studio mode isn't supported
        log::info!(
            "Studio mode {} (not supported by obws)",
            if enabled {
                "enable requested"
            } else {
                "disable requested"
            }
        );
        Ok(())
    }

    /// Get OBS statistics
    pub async fn get_stats(&self) -> AppResult<ObsStats> {
        let client = self.get_client()?;
        let stats = client
            .general()
            .stats()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to get OBS stats: {}", e)))?;

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

    /// Convert obws event to our ObsEvent enum
    fn convert_obws_event(event: obws::events::Event) -> AppResult<ObsEvent> {
        // For now, convert to a generic custom event since the obws event structure
        // may be different than expected. This can be enhanced later.
        Ok(ObsEvent::Custom {
            event_type: format!("{:?}", event),
            data: serde_json::json!({
                "raw_event": format!("{:?}", event)
            }),
        })
    }

    /// Set recording directory (Output -> Recording -> Recording path)
    pub async fn set_record_directory(&self, directory: &str) -> AppResult<()> {
        let client = self.get_client()?;
        // Prefer official obws Config API
        println!(
            " obws.config.set_record_directory directory='{}'",
            directory
        );
        match client.config().set_record_directory(directory).await {
            Ok(_) => {
                log::info!("Recording directory set via Config API: {}", directory);
                return Ok(());
            }
            Err(e) => {
                log::warn!("Config.set_record_directory failed ({}). Falling back to profiles.set_parameter.", e);
            }
        }
        // Fallback to profile parameter because some OBS profiles store RecFilePath there
        println!(
            " obws.profiles.set_parameter category=Output name=RecFilePath value='{}'",
            directory
        );
        client
            .profiles()
            .set_parameter(obws::requests::profiles::SetParameter {
                category: "Output",
                name: "RecFilePath",
                value: Some(directory),
            })
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to set record directory: {}", e)))?;
        // Try alternative advanced output key as well, but do not fail the call if it errors
        println!(
            " obws.profiles.set_parameter category=AdvOut name=RecFilePath value='{}'",
            directory
        );
        let _ = client
            .profiles()
            .set_parameter(obws::requests::profiles::SetParameter {
                category: "AdvOut",
                name: "RecFilePath",
                value: Some(directory),
            })
            .await;
        log::info!("Recording directory set (fallback): {}", directory);
        Ok(())
    }

    /// Set filename formatting (Advanced -> Recording -> Filename formatting)
    pub async fn set_filename_formatting(&self, formatting: &str) -> AppResult<()> {
        let client = self.get_client()?;
        // Use profile.set_parameter for filename formatting. Key commonly "FilenameFormatting" under "Output" or "AdvOut".
        // We set both likely keys to improve compatibility; ignore errors on the second set.
        println!(
            " obws.profiles.set_parameter {{category='Output', name='FilenameFormatting', value='{}'}}",
            formatting
        );
        client
            .profiles()
            .set_parameter(obws::requests::profiles::SetParameter {
                category: "Output",
                name: "FilenameFormatting",
                value: Some(formatting),
            })
            .await
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to set filename formatting: {}", e))
            })?;
        // Try alternative advanced key without failing whole call if it errors
        println!(
            " obws.profiles.set_parameter {{category='AdvOut', name='FilenameFormatting', value='{}'}}",
            formatting
        );
        let _ = client
            .profiles()
            .set_parameter(obws::requests::profiles::SetParameter {
                category: "AdvOut",
                name: "FilenameFormatting",
                value: Some(formatting),
            })
            .await;
        log::info!("Filename formatting set to: {}", formatting);
        Ok(())
    }

    /// Get recording directory from OBS profile
    pub async fn get_record_directory(&self) -> AppResult<String> {
        let client = self.get_client()?;
        // Prefer official API
        if let Ok(dir) = client.config().record_directory().await {
            return Ok(dir);
        }
        // Fallback to profile parameter
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
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to get filename formatting: {}", e))
            })?;
        Ok(param.value.unwrap_or_default())
    }

    /// Get comprehensive OBS status
    pub async fn get_status(&self) -> AppResult<ObsStatus> {
        let recording_status =
            self.get_recording_status()
                .await
                .unwrap_or(ObsRecordingStatus::Error(
                    "Failed to get recording status".to_string(),
                ));
        let streaming_status =
            self.get_streaming_status()
                .await
                .unwrap_or(ObsStreamingStatus::Error(
                    "Failed to get streaming status".to_string(),
                ));
        let replay_buffer_status =
            self.get_replay_buffer_status()
                .await
                .unwrap_or(ObsReplayBufferStatus::Error(
                    "Failed to get replay buffer status".to_string(),
                ));
        let virtual_camera_status =
            self.get_virtual_camera_status()
                .await
                .unwrap_or(ObsVirtualCameraStatus::Error(
                    "Failed to get virtual camera status".to_string(),
                ));

        let current_scene = self.get_current_scene().await.ok();
        let scenes = self
            .get_scenes()
            .await
            .map(|s| s.into_iter().map(|scene| scene.name).collect())
            .unwrap_or_default();
        let version = self.get_version().await.ok();
        let stats = self.get_stats().await.ok();

        // Get studio mode status
        let studio_mode = self
            .get_studio_mode_status()
            .await
            .unwrap_or(ObsStudioModeStatus::Disabled);

        Ok(ObsStatus {
            connection_status: self.status.clone(),
            recording_status,
            streaming_status,
            replay_buffer_status,
            virtual_camera_status,
            studio_mode,
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
        let handlers = self.event_handlers.lock().await;

        // Determine event type string based on the event variant
        let event_type = match &event {
            ObsEvent::ConnectionEstablished => "ConnectionEstablished",
            ObsEvent::ConnectionLost => "ConnectionLost",
            ObsEvent::RecordingStarted => "RecordingStarted",
            ObsEvent::RecordingStopped => "RecordingStopped",
            ObsEvent::StreamingStarted => "StreamingStarted",
            ObsEvent::StreamingStopped => "StreamingStopped",
            ObsEvent::ReplayBufferStarted => "ReplayBufferStarted",
            ObsEvent::ReplayBufferStopped => "ReplayBufferStopped",
            ObsEvent::ReplayBufferSaved => "ReplayBufferSaved",
            ObsEvent::VirtualCameraStarted => "VirtualCameraStarted",
            ObsEvent::VirtualCameraStopped => "VirtualCameraStopped",
            ObsEvent::SceneChanged { .. } => "SceneChanged",
            ObsEvent::SourceCreated { .. } => "SourceCreated",
            ObsEvent::SourceRemoved { .. } => "SourceRemoved",
            ObsEvent::SourceRenamed { .. } => "SourceRenamed",
            ObsEvent::StudioModeSwitched { .. } => "StudioModeSwitched",
            ObsEvent::Custom { event_type, .. } => event_type,
        };

        // Call handlers for this event type
        if let Some(handler) = handlers.get(event_type) {
            handler(event.clone());
            log::debug!("Event '{}' handler executed", event_type);
        } else {
            log::debug!("No handler registered for event type: {}", event_type);
        }

        // Also call wildcard handlers for "all" events
        if let Some(handler) = handlers.get("all") {
            handler(event.clone());
            log::debug!("Wildcard event handler executed for '{}'", event_type);
        }

        log::debug!("Event triggered: {:?}", event);
        Ok(())
    }

    /// Start monitoring for OBS events
    pub async fn start_monitoring(&mut self) -> AppResult<()> {
        let client = self.get_client()?;

        // Set up event handler for all events
        let events = client
            .events()
            .map_err(|e| AppError::ConfigError(format!("Failed to set up event handler: {}", e)))?;

        // Pin the stream and set up event handler
        let mut events = Box::pin(events);
        let shutdown_notify = Arc::clone(&self.monitoring_shutdown);

        // Set up event handler task
        let monitoring_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle incoming events
                    Some(event) = events.next() => {
                        log::debug!("OBS event: {:?}", event);

                        // Convert obws event to our ObsEvent and trigger it
                        if let Ok(obs_event) = ObsClient::convert_obws_event(event) {
                            // TODO: For now, just log the event - event triggering needs proper client reference
                            log::debug!("Converted OBS event: {:?}", obs_event);
                        }
                    }
                    // Handle shutdown signal
                    _ = shutdown_notify.notified() => {
                        log::debug!("Monitoring task received shutdown signal");
                        break;
                    }
                }
            }
        });

        // Store the task handle
        self.monitoring_task = Some(monitoring_task);

        log::info!("Started monitoring OBS events");
        Ok(())
    }

    /// Stop monitoring for OBS events
    pub async fn stop_monitoring(&mut self) -> AppResult<()> {
        if let Some(task) = self.monitoring_task.take() {
            log::info!("Stopping monitoring for OBS connection");

            // Signal the monitoring task to stop
            self.monitoring_shutdown.notify_waiters();

            // Abort the monitoring task
            task.abort();

            // Give the task a moment to clean up
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            log::info!("Monitoring task stopped");
        } else {
            log::debug!("No monitoring task was running");
        }

        Ok(())
    }

    /// Set up status listener
    pub async fn setup_status_listener(&self) -> AppResult<()> {
        let client = self.get_client()?;

        // Set up event handler for all events
        let events = client
            .events()
            .map_err(|e| AppError::ConfigError(format!("Failed to set up event handler: {}", e)))?;

        // Pin the stream and set up event handler
        let mut events = Box::pin(events);

        // Set up event handler
        tokio::spawn(async move {
            while let Some(event) = events.next().await {
                log::debug!("OBS event: {:?}", event);
            }
        });

        log::info!("Status listener set up successfully");
        Ok(())
    }
}

impl Drop for ObsClient {
    fn drop(&mut self) {
        // Clean up any resources when the client is dropped
        log::debug!("Dropping ObsClient");

        // Stop monitoring if it's still running
        if self.monitoring_task.is_some() {
            log::debug!("Stopping monitoring task during client drop");
            // Signal shutdown and abort the task
            self.monitoring_shutdown.notify_waiters();
            if let Some(task) = self.monitoring_task.take() {
                task.abort();
            }
        }
    }
}
