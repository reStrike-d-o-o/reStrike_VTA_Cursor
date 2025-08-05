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

// OBS Connection Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObsConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Authenticating,
    Authenticated,
    Error(String),
}

// OBS Connection State
#[derive(Debug)]
pub struct ObsConnection {
    pub config: ObsConnectionConfig,
    pub status: ObsConnectionStatus,
    pub websocket: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    pub request_id_counter: u64,
    pub pending_requests: HashMap<String, tokio::sync::oneshot::Sender<serde_json::Value>>,
    pub heartbeat_data: Option<serde_json::Value>,
}

// Recent events buffer for frontend polling
#[derive(Debug, Clone)]
pub struct RecentEvent {
    pub connection_name: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

// OBS Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsEvent {
    ConnectionStatusChanged {
        connection_name: String,
        status: ObsConnectionStatus,
    },
    SceneChanged {
        connection_name: String,
        scene_name: String,
    },
    RecordingStateChanged {
        connection_name: String,
        is_recording: bool,
    },
    StreamStateChanged {
        connection_name: String,
        is_streaming: bool,
    },
    ReplayBufferStateChanged {
        connection_name: String,
        is_active: bool,
    },
    Error {
        connection_name: String,
        error: String,
    },
    Raw {
        connection_name: String,
        event_type: String,
        data: serde_json::Value,
    },
}

// OBS Status Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsStatusInfo {
    pub is_recording: bool,
    pub is_streaming: bool,
    pub cpu_usage: f64,
    pub recording_connection: Option<String>,
    pub streaming_connection: Option<String>,
}

// Shared plugin context for cross-plugin communication
pub struct ObsPluginContext {
    pub connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    pub event_tx: mpsc::UnboundedSender<ObsEvent>,
    pub debug_ws_messages: Arc<Mutex<bool>>,
    pub show_full_events: Arc<Mutex<bool>>,
    pub recent_events: Arc<Mutex<Vec<RecentEvent>>>,
    pub log_manager: Arc<Mutex<LogManager>>,
}

impl ObsPluginContext {
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

impl Clone for ObsPluginContext {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            event_tx: self.event_tx.clone(),
            debug_ws_messages: self.debug_ws_messages.clone(),
            show_full_events: self.show_full_events.clone(),
            recent_events: self.recent_events.clone(),
            log_manager: self.log_manager.clone(),
        }
    }
}

// Plugin trait for common plugin functionality
pub trait ObsPlugin {
    fn name(&self) -> &str;
    fn init(&self) -> AppResult<()>;
    fn shutdown(&self) -> AppResult<()>;
} 