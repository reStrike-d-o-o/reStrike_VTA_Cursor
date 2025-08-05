// OBS Recording Plugin
// Handles recording operations (start, stop, replay buffer)
// Extracted from the original plugin_obs.rs

use crate::types::AppResult;
use super::types::*;
use super::core::ObsCorePlugin;
use std::sync::Arc;

/// Recording plugin for OBS operations
pub struct ObsRecordingPlugin {
    context: ObsPluginContext,
    core_plugin: Arc<ObsCorePlugin>,
}

impl ObsRecordingPlugin {
    /// Create a new OBS Recording Plugin
    pub fn new(context: ObsPluginContext, core_plugin: Arc<ObsCorePlugin>) -> Self {
        Self { 
            context,
            core_plugin,
        }
    }

    /// Start recording
    pub async fn start_recording(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "StartRecord", None).await?;
        Ok(())
    }

    /// Stop recording
    pub async fn stop_recording(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "StopRecord", None).await?;
        Ok(())
    }

    /// Start replay buffer
    pub async fn start_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "StartReplayBuffer", None).await?;
        Ok(())
    }

    /// Stop replay buffer
    pub async fn stop_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "StopReplayBuffer", None).await?;
        Ok(())
    }

    /// Save replay buffer
    pub async fn save_replay_buffer(&self, connection_name: &str) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "SaveReplayBuffer", None).await?;
        Ok(())
    }

    /// Get recording status
    pub async fn get_recording_status(&self, connection_name: &str) -> AppResult<bool> {
        let response = self.send_recording_request(connection_name, "GetRecordStatus", None).await?;
        
        if let Some(output_active) = response["outputActive"].as_bool() {
            Ok(output_active)
        } else {
            Ok(false)
        }
    }

    /// Get replay buffer status
    pub async fn get_replay_buffer_status(&self, connection_name: &str) -> AppResult<bool> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferStatus", None).await?;
        
        if let Some(output_active) = response["outputActive"].as_bool() {
            Ok(output_active)
        } else {
            Ok(false)
        }
    }

    /// Get detailed replay buffer status
    pub async fn get_detailed_replay_buffer_status(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferStatus", None).await?;
        Ok(response)
    }

    /// Get replay buffer duration
    pub async fn get_replay_buffer_duration(&self, connection_name: &str) -> AppResult<i32> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(duration) = response["replayBufferDuration"].as_i64() {
            Ok(duration as i32)
        } else {
            Ok(30) // Default duration
        }
    }

    /// Set replay buffer duration
    pub async fn set_replay_buffer_duration(&self, connection_name: &str, duration_seconds: i32) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferDuration": duration_seconds
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer path
    pub async fn get_replay_buffer_path(&self, connection_name: &str) -> AppResult<String> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(path) = response["replayBufferPath"].as_str() {
            Ok(path.to_string())
        } else {
            Ok("C:\\Users\\%username%\\Videos\\OBS\\ReplayBuffer".to_string())
        }
    }

    /// Set replay buffer path
    pub async fn set_replay_buffer_path(&self, connection_name: &str, path: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferPath": path
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer filename format
    pub async fn get_replay_buffer_filename(&self, connection_name: &str) -> AppResult<String> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(filename) = response["replayBufferFilename"].as_str() {
            Ok(filename.to_string())
        } else {
            Ok("replay_%date%_%time%".to_string())
        }
    }

    /// Set replay buffer filename format
    pub async fn set_replay_buffer_filename(&self, connection_name: &str, filename_format: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferFilename": filename_format
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer format
    pub async fn get_replay_buffer_format(&self, connection_name: &str) -> AppResult<String> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(format) = response["replayBufferFormat"].as_str() {
            Ok(format.to_string())
        } else {
            Ok("mp4".to_string())
        }
    }

    /// Set replay buffer format
    pub async fn set_replay_buffer_format(&self, connection_name: &str, format: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferFormat": format
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer quality
    pub async fn get_replay_buffer_quality(&self, connection_name: &str) -> AppResult<String> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(quality) = response["replayBufferQuality"].as_str() {
            Ok(quality.to_string())
        } else {
            Ok("high".to_string())
        }
    }

    /// Set replay buffer quality
    pub async fn set_replay_buffer_quality(&self, connection_name: &str, quality: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferQuality": quality
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer bitrate
    pub async fn get_replay_buffer_bitrate(&self, connection_name: &str) -> AppResult<i32> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(bitrate) = response["replayBufferBitrate"].as_i64() {
            Ok(bitrate as i32)
        } else {
            Ok(2500) // Default bitrate
        }
    }

    /// Set replay buffer bitrate
    pub async fn set_replay_buffer_bitrate(&self, connection_name: &str, bitrate: i32) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferBitrate": bitrate
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer keyframe interval
    pub async fn get_replay_buffer_keyframe_interval(&self, connection_name: &str) -> AppResult<i32> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(interval) = response["replayBufferKeyframeInterval"].as_i64() {
            Ok(interval as i32)
        } else {
            Ok(2) // Default interval
        }
    }

    /// Set replay buffer keyframe interval
    pub async fn set_replay_buffer_keyframe_interval(&self, connection_name: &str, interval: i32) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferKeyframeInterval": interval
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer rate control
    pub async fn get_replay_buffer_rate_control(&self, connection_name: &str) -> AppResult<String> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(rate_control) = response["replayBufferRateControl"].as_str() {
            Ok(rate_control.to_string())
        } else {
            Ok("CBR".to_string())
        }
    }

    /// Set replay buffer rate control
    pub async fn set_replay_buffer_rate_control(&self, connection_name: &str, rate_control: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferRateControl": rate_control
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer preset
    pub async fn get_replay_buffer_preset(&self, connection_name: &str) -> AppResult<String> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(preset) = response["replayBufferPreset"].as_str() {
            Ok(preset.to_string())
        } else {
            Ok("veryfast".to_string())
        }
    }

    /// Set replay buffer preset
    pub async fn set_replay_buffer_preset(&self, connection_name: &str, preset: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferPreset": preset
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer profile
    pub async fn get_replay_buffer_profile(&self, connection_name: &str) -> AppResult<String> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(profile) = response["replayBufferProfile"].as_str() {
            Ok(profile.to_string())
        } else {
            Ok("main".to_string())
        }
    }

    /// Set replay buffer profile
    pub async fn set_replay_buffer_profile(&self, connection_name: &str, profile: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferProfile": profile
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get replay buffer tune
    pub async fn get_replay_buffer_tune(&self, connection_name: &str) -> AppResult<String> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        
        if let Some(tune) = response["replayBufferTune"].as_str() {
            Ok(tune.to_string())
        } else {
            Ok("zerolatency".to_string())
        }
    }

    /// Set replay buffer tune
    pub async fn set_replay_buffer_tune(&self, connection_name: &str, tune: &str) -> AppResult<()> {
        let request_data = serde_json::json!({
            "replayBufferTune": tune
        });
        
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(request_data)).await?;
        Ok(())
    }

    /// Get all replay buffer settings
    pub async fn get_all_replay_buffer_settings(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        let response = self.send_recording_request(connection_name, "GetReplayBufferSettings", None).await?;
        Ok(response)
    }

    /// Set all replay buffer settings at once
    pub async fn set_all_replay_buffer_settings(&self, connection_name: &str, settings: serde_json::Value) -> AppResult<()> {
        let _response = self.send_recording_request(connection_name, "SetReplayBufferSettings", Some(settings)).await?;
        Ok(())
    }

    /// Get available replay buffer formats
    pub async fn get_available_replay_buffer_formats(&self) -> AppResult<Vec<String>> {
        // Common OBS replay buffer formats
        let formats = vec![
            "mp4".to_string(),
            "mkv".to_string(),
            "mov".to_string(),
            "flv".to_string(),
        ];
        Ok(formats)
    }

    /// Get available replay buffer qualities
    pub async fn get_available_replay_buffer_qualities(&self) -> AppResult<Vec<String>> {
        let qualities = vec![
            "ultra".to_string(),
            "high".to_string(),
            "medium".to_string(),
            "low".to_string(),
        ];
        Ok(qualities)
    }

    /// Get available replay buffer rate controls
    pub async fn get_available_replay_buffer_rate_controls(&self) -> AppResult<Vec<String>> {
        let rate_controls = vec![
            "CBR".to_string(),
            "VBR".to_string(),
            "ABR".to_string(),
        ];
        Ok(rate_controls)
    }

    /// Get available replay buffer presets
    pub async fn get_available_replay_buffer_presets(&self) -> AppResult<Vec<String>> {
        let presets = vec![
            "ultrafast".to_string(),
            "superfast".to_string(),
            "veryfast".to_string(),
            "faster".to_string(),
            "fast".to_string(),
            "medium".to_string(),
            "slow".to_string(),
            "slower".to_string(),
            "veryslow".to_string(),
        ];
        Ok(presets)
    }

    /// Get available replay buffer profiles
    pub async fn get_available_replay_buffer_profiles(&self) -> AppResult<Vec<String>> {
        let profiles = vec![
            "baseline".to_string(),
            "main".to_string(),
            "high".to_string(),
            "high10".to_string(),
            "high422".to_string(),
            "high444".to_string(),
        ];
        Ok(profiles)
    }

    /// Get available replay buffer tunes
    pub async fn get_available_replay_buffer_tunes(&self) -> AppResult<Vec<String>> {
        let tunes = vec![
            "zerolatency".to_string(),
            "fastdecode".to_string(),
            "grain".to_string(),
            "film".to_string(),
            "animation".to_string(),
            "stillimage".to_string(),
            "psnr".to_string(),
            "ssim".to_string(),
        ];
        Ok(tunes)
    }

    /// Send a recording request to OBS
    async fn send_recording_request(
        &self,
        connection_name: &str,
        request_type: &str,
        request_data: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        // Delegate to core plugin for actual WebSocket communication
        let data = request_data.unwrap_or_else(|| serde_json::json!({}));
        self.core_plugin.send_request(connection_name, request_type, Some(data)).await
    }

    /// Handle recording state change events
    pub async fn handle_recording_state_change(&self, connection_name: &str, is_recording: bool) {
        log::info!("[OBS_RECORDING] Recording state changed for '{}': {}", connection_name, is_recording);
        
        // Emit recording state change event
        let event = ObsEvent::RecordingStateChanged {
            connection_name: connection_name.to_string(),
            is_recording,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_RECORDING] Failed to emit recording state change event: {}", e);
        }
    }

    /// Handle replay buffer state change events
    pub async fn handle_replay_buffer_state_change(&self, connection_name: &str, is_active: bool) {
        log::info!("[OBS_RECORDING] Replay buffer state changed for '{}': {}", connection_name, is_active);
        
        // Emit replay buffer state change event
        let event = ObsEvent::ReplayBufferStateChanged {
            connection_name: connection_name.to_string(),
            is_active,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_RECORDING] Failed to emit replay buffer state change event: {}", e);
        }
    }
}

// Implement ObsPlugin trait for the recording plugin
impl ObsPlugin for ObsRecordingPlugin {
    fn name(&self) -> &str {
        "obs_recording"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Recording Plugin");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Recording Plugin");
        Ok(())
    }
} 