// Shared types for the modular OBS plugin system
// Extracted from the original plugin_obs.rs to provide common types across all OBS plugins

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::AppResult;
use crate::logging::LogManager;

// OBS WebSocket Protocol Versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObsWebSocketVersion {
    V5,
}

// OBS Connection Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnectionConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub protocol_version: ObsWebSocketVersion,
    pub enabled: bool,
}

/// OBS Status Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsStatusInfo {
    pub is_recording: bool,
    pub is_streaming: bool,
    pub cpu_usage: f64,
    pub recording_connection: Option<String>,
    pub streaming_connection: Option<String>,
    pub connections: Vec<ObsConnectionInfo>,
}

/// OBS Connection Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConnectionInfo {
    pub name: String,
    pub is_connected: bool,
    pub last_heartbeat: Option<DateTime<Utc>>,
}

/// System Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub fps: f64,
    pub timestamp: DateTime<Utc>,
}

/// OBS Connection Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObsConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Authenticating,
    Authenticated,
    Error(String),
}

/// OBS Connection
#[derive(Debug)]
pub struct ObsConnection {
    pub config: ObsConnectionConfig,
    pub status: ObsConnectionStatus,
    pub websocket: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    pub request_id_counter: u64,
    pub pending_requests: HashMap<String, tokio::sync::oneshot::Sender<serde_json::Value>>,
    pub heartbeat_data: Option<serde_json::Value>,
    pub is_connected: bool,
    pub last_heartbeat: Option<DateTime<Utc>>,
}

// Recent events buffer for frontend polling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEvent {
    pub connection_name: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// OBS Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsEvent {
    RecordingStateChanged {
        connection_name: String,
        is_recording: bool,
    },
    ReplayBufferStateChanged {
        connection_name: String,
        is_active: bool,
    },
    StreamingStateChanged {
        connection_name: String,
        is_streaming: bool,
    },
    StatusUpdate {
        connection_name: String,
        status: ObsConnectionStatus,
    },
    ConnectionStateChanged {
        connection_name: String,
        status: ObsConnectionStatus,
    },
    Heartbeat {
        connection_name: String,
        data: serde_json::Value,
    },
    // Add missing variants that are used in the code
    Raw {
        connection_name: String,
        data: serde_json::Value,
    },
    SceneChanged {
        connection_name: String,
        scene_name: String,
    },
    StreamStateChanged {
        connection_name: String,
        is_streaming: bool,
    },
    ConnectionStatusChanged {
        connection_name: String,
        status: ObsConnectionStatus,
    },
    Error {
        connection_name: String,
        error: String,
    },
}

/// Shared context for all OBS plugins
#[derive(Clone)]
pub struct ObsPluginContext {
    pub connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    pub event_tx: mpsc::UnboundedSender<ObsEvent>,
    pub debug_ws_messages: Arc<Mutex<bool>>,
    pub show_full_events: Arc<Mutex<bool>>,
    pub recent_events: Arc<Mutex<Vec<RecentEvent>>>,
    pub log_manager: Arc<Mutex<LogManager>>,
    pub core_plugin: Option<Arc<super::core::ObsCorePlugin>>,
}

impl ObsPluginContext {
    /// Create a new OBS Plugin Context
    pub fn new() -> AppResult<Self> {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        
        let log_manager = Arc::new(Mutex::new(LogManager::new(crate::logging::LogConfig::default())
            .map_err(|e| crate::types::AppError::ConfigError(format!("Failed to initialize logging: {}", e)))?));
        
        Ok(Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            debug_ws_messages: Arc::new(Mutex::new(false)),
            show_full_events: Arc::new(Mutex::new(false)),
            recent_events: Arc::new(Mutex::new(Vec::new())),
            log_manager,
            core_plugin: None,
        })
    }

    /// Log a message to file using the log manager
    pub async fn log_to_file(&self, level: &str, message: &str) {
        let log_manager = self.log_manager.lock().await;
        let _ = log_manager.log("obs", level, message);
    }

    /// Store a recent event for frontend polling
    pub async fn store_recent_event(&self, connection_name: String, event_type: String, data: serde_json::Value) {
        let event = RecentEvent {
            connection_name,
            event_type,
            data,
            timestamp: Utc::now(),
        };

        let mut events = self.recent_events.lock().await;
        events.insert(0, event);
        // Keep only the last 50 events
        if events.len() > 50 {
            events.truncate(50);
        }
    }
}

// Event filtering and routing types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    pub id: String,
    pub name: String,
    pub condition: FilterCondition,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    AllowAll,
    BlockEventType(String),
    AllowEventType(String),
    BlockConnection(String),
    AllowConnection(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRoute {
    pub id: String,
    pub name: String,
    pub condition: RouteCondition,
    pub destination: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteCondition {
    AllEvents,
    EventType(String),
    Connection(String),
    Custom(String), // JSON predicate
}

// Plugin trait for common plugin functionality
pub trait ObsPlugin {
    fn name(&self) -> &str;
    fn init(&self) -> AppResult<()>;
    fn shutdown(&self) -> AppResult<()>;
} 