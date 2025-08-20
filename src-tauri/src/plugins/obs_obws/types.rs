//! Types and data structures for OBS obws integration
//!
//! These mirror the logical state we expose to the UI: connection status,
//! recording/streaming/replay buffer state, current scene list, stats, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OBS connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnectionConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub timeout_seconds: u64,
}

impl Default for ObsConnectionConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            host: "localhost".to_string(),
            port: 4455,
            password: None,
            timeout_seconds: 30,
        }
    }
}

/// OBS connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObsConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Authenticated,
    Error(String),
}

/// OBS recording status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObsRecordingStatus {
    Stopped,
    Starting,
    Recording,
    Stopping,
    Error(String),
}

/// OBS streaming status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObsStreamingStatus {
    Stopped,
    Starting,
    Streaming,
    Stopping,
    Error(String),
}

/// OBS replay buffer status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObsReplayBufferStatus {
    Stopped,
    Starting,
    Active,
    Stopping,
    Saving,
    Error(String),
}

/// OBS virtual camera status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObsVirtualCameraStatus {
    Stopped,
    Starting,
    Active,
    Stopping,
    Error(String),
}

/// OBS studio mode status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObsStudioModeStatus {
    Disabled,
    Enabled,
    Transitioning,
    Error(String),
}

/// OBS source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsSource {
    pub name: String,
    pub type_name: String,
    pub enabled: bool,
    pub muted: bool,
    pub volume: Option<f64>,
    pub bounds: Option<ObsBounds>,
    pub transform: Option<ObsTransform>,
}

/// OBS scene information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsScene {
    pub name: String,
    pub scene_index: i32,
    pub sources: Vec<ObsSource>,
}

/// OBS bounds for sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// OBS transform for sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsTransform {
    pub position_x: f64,
    pub position_y: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotation: f64,
    pub crop_top: i32,
    pub crop_right: i32,
    pub crop_bottom: i32,
    pub crop_left: i32,
}

/// OBS output settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsOutputSettings {
    pub output_path: String,
    pub output_format: String,
    pub video_encoder: String,
    pub audio_encoder: String,
    pub video_bitrate: i32,
    pub audio_bitrate: i32,
    pub fps: i32,
    pub width: i32,
    pub height: i32,
}

/// OBS version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsVersion {
    pub obs_version: String,
    pub obs_web_socket_version: String,
    pub rpc_version: i32,
    pub available_requests: Vec<String>,
    pub supported_image_export_formats: Vec<String>,
}

/// OBS statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsStats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub available_disk_space: i64,
    pub active_fps: f64,
    pub average_frame_render_time: f64,
    pub render_skipped_frames: i32,
    pub render_total_frames: i32,
    pub output_skipped_frames: i32,
    pub output_total_frames: i32,
}

/// OBS status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsStatus {
    pub connection_status: ObsConnectionStatus,
    pub recording_status: ObsRecordingStatus,
    pub streaming_status: ObsStreamingStatus,
    pub replay_buffer_status: ObsReplayBufferStatus,
    pub virtual_camera_status: ObsVirtualCameraStatus,
    pub studio_mode: ObsStudioModeStatus,
    pub current_scene: Option<String>,
    pub scenes: Vec<String>,
    pub version: Option<ObsVersion>,
    pub stats: Option<ObsStats>,
}

/// OBS event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsEvent {
    ConnectionEstablished,
    ConnectionLost,
    RecordingStarted,
    RecordingStopped,
    StreamingStarted,
    StreamingStopped,
    ReplayBufferStarted,
    ReplayBufferStopped,
    ReplayBufferSaved,
    VirtualCameraStarted,
    VirtualCameraStopped,
    SceneChanged { scene_name: String },
    SourceCreated { source_name: String, source_type: String },
    SourceRemoved { source_name: String },
    SourceRenamed { old_name: String, new_name: String },
    StudioModeSwitched { enabled: bool },
    Custom { event_type: String, data: serde_json::Value },
}

/// OBS operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsOperationResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// OBS operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsOperationRequest {
    pub operation: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// OBS operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsOperationResponse {
    pub request_id: String,
    pub status: String,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// OBS connection info for multiple connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnectionInfo {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub status: ObsConnectionStatus,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// OBS settings category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsSettingsCategory {
    pub category: String,
    pub settings: HashMap<String, serde_json::Value>,
}

/// OBS hotkey information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsHotkey {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

/// OBS filter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsFilter {
    pub name: String,
    pub type_name: String,
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

/// OBS transition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsTransition {
    pub name: String,
    pub type_name: String,
    pub duration: Option<i32>,
    pub settings: HashMap<String, serde_json::Value>,
}
