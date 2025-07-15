//! Shared types and data structures for the reStrike VTA application

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// OBS Integration Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub protocol_version: ObsProtocolVersion,
    pub enabled: bool,
    pub status: ObsConnectionStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsProtocolVersion {
    V4,
    V5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Authenticating,
    Authenticated,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsStatusInfo {
    pub is_recording: bool,
    pub is_streaming: bool,
    pub cpu_usage: f64,
    pub recording_connection: Option<String>,
    pub streaming_connection: Option<String>,
}

// ============================================================================
// Video System Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoClip {
    pub id: String,
    pub name: String,
    pub path: String,
    pub duration: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub volume: f64,
    pub playback_rate: f64,
    pub loop_enabled: bool,
    pub hardware_acceleration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlaySettings {
    pub opacity: f64,
    pub position: OverlayPosition,
    pub scale: f64,
    pub visible: bool,
    pub theme: OverlayTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverlayTheme {
    Dark,
    Light,
    Transparent,
}

// ============================================================================
// PSS Protocol Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PssEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: PssEventType,
    pub player: PssPlayer,
    pub description: String,
    pub value: Option<String>,
    pub raw_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PssEventType {
    Point,
    Warning,
    Clock,
    Round,
    Score,
    Athlete,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PssPlayer {
    Red,
    Blue,
    Yellow,
    None,
}

// ============================================================================
// Application State Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub obs_connections: Vec<ObsConnection>,
    pub active_obs_connection: Option<String>,
    pub obs_status: Option<ObsStatusInfo>,
    pub overlay_settings: OverlaySettings,
    pub video_clips: Vec<VideoClip>,
    pub current_clip: Option<VideoClip>,
    pub is_playing: bool,
    pub current_view: AppView,
    pub is_loading: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppView {
    SidebarTest,
    Overlay,
    Settings,
    Clips,
    ObsManager,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("OBS connection failed: {0}")]
    ObsConnectionError(String),
    
    #[error("Video playback error: {0}")]
    VideoError(String),
    
    #[error("PSS protocol error: {0}")]
    PssError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// ============================================================================
// Result Types
// ============================================================================

pub type AppResult<T> = Result<T, AppError>;

// ============================================================================
// Constants
// ============================================================================

pub const DEFAULT_OBS_PORT: u16 = 4455;
pub const DEFAULT_OBS_PASSWORD: &str = "cekPIbj@245";
pub const DEFAULT_VIDEO_VOLUME: f64 = 1.0;
pub const DEFAULT_PLAYBACK_RATE: f64 = 1.0; 