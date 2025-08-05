// OBS Events Plugin
// Handles event handling, routing, filtering, and frontend broadcasting
// Extracted from the original plugin_obs.rs

use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use crate::types::{AppError, AppResult};
use super::types::*;

/// OBS Events Plugin for event handling
pub struct ObsEventsPlugin {
    context: ObsPluginContext,
}

impl ObsEventsPlugin {
    /// Create a new OBS Events Plugin
    pub fn new(context: ObsPluginContext) -> Self {
        Self { context }
    }

    /// Get latest events for a connection
    pub async fn get_latest_events(&self, connection_name: &str) -> AppResult<serde_json::Value> {
        log::debug!("[OBS_EVENTS] get_latest_events called for '{}'", connection_name);
        
        let recent_events = self.context.recent_events.lock().await;
        
        // Filter events for the specific connection
        let connection_events: Vec<&RecentEvent> = recent_events
            .iter()
            .filter(|event| event.connection_name == connection_name)
            .collect();
        
        // Convert to JSON format
        let events_json: Vec<serde_json::Value> = connection_events
            .iter()
            .map(|event| {
                serde_json::json!({
                    "eventType": event.event_type,
                    "data": event.data,
                    "timestamp": event.timestamp.to_rfc3339()
                })
            })
            .collect();
        
        let result = serde_json::json!({
            "connectionName": connection_name,
            "events": events_json,
            "count": events_json.len()
        });
        
        log::debug!("[OBS_EVENTS] Returning {} events for '{}'", events_json.len(), connection_name);
        Ok(result)
    }

    /// Add a recent event to the buffer
    pub async fn add_recent_event(&self, connection_name: &str, event_type: &str, data: serde_json::Value) {
        let mut recent_events = self.context.recent_events.lock().await;
        
        let event = RecentEvent {
            connection_name: connection_name.to_string(),
            event_type: event_type.to_string(),
            data,
            timestamp: Utc::now(),
        };
        
        // Add to the beginning of the list
        recent_events.insert(0, event);
        
        // Keep only the last 100 events
        if recent_events.len() > 100 {
            recent_events.truncate(100);
        }
        
        log::debug!("[OBS_EVENTS] Added event '{}' for '{}'", event_type, connection_name);
    }

    /// Clear recent events for a connection
    pub async fn clear_recent_events(&self, connection_name: &str) {
        let mut recent_events = self.context.recent_events.lock().await;
        
        recent_events.retain(|event| event.connection_name != connection_name);
        
        log::info!("[OBS_EVENTS] Cleared events for '{}'", connection_name);
    }

    /// Clear all recent events
    pub async fn clear_all_recent_events(&self) {
        let mut recent_events = self.context.recent_events.lock().await;
        recent_events.clear();
        
        log::info!("[OBS_EVENTS] Cleared all events");
    }

    /// Handle incoming OBS WebSocket events
    pub async fn handle_obs_event(&self, connection_name: &str, event_type: &str, event_data: serde_json::Value) {
        log::debug!("[OBS_EVENTS] Handling event '{}' for '{}'", event_type, connection_name);
        
        // Add to recent events buffer
        self.add_recent_event(connection_name, event_type, event_data.clone()).await;
        
        // Route event to appropriate handler based on event type
        match event_type {
            "SceneChanged" => {
                if let Some(scene_name) = event_data.get("sceneName").and_then(|s| s.as_str()) {
                    self.handle_scene_changed(connection_name, scene_name).await;
                }
            },
            "RecordStateChanged" => {
                if let Some(is_recording) = event_data.get("outputActive").and_then(|b| b.as_bool()) {
                    self.handle_recording_state_changed(connection_name, is_recording).await;
                }
            },
            "StreamStateChanged" => {
                if let Some(is_streaming) = event_data.get("outputActive").and_then(|b| b.as_bool()) {
                    self.handle_streaming_state_changed(connection_name, is_streaming).await;
                }
            },
            "ReplayBufferStateChanged" => {
                if let Some(is_active) = event_data.get("outputActive").and_then(|b| b.as_bool()) {
                    self.handle_replay_buffer_state_changed(connection_name, is_active).await;
                }
            },
            _ => {
                // Handle as raw event
                self.handle_raw_event(connection_name, event_type, event_data).await;
            }
        }
    }

    /// Handle scene changed events
    async fn handle_scene_changed(&self, connection_name: &str, scene_name: &str) {
        log::info!("[OBS_EVENTS] Scene changed for '{}' to '{}'", connection_name, scene_name);
        
        let event = ObsEvent::SceneChanged {
            connection_name: connection_name.to_string(),
            scene_name: scene_name.to_string(),
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_EVENTS] Failed to emit scene changed event: {}", e);
        }
    }

    /// Handle recording state changed events
    async fn handle_recording_state_changed(&self, connection_name: &str, is_recording: bool) {
        log::info!("[OBS_EVENTS] Recording state changed for '{}': {}", connection_name, is_recording);
        
        let event = ObsEvent::RecordingStateChanged {
            connection_name: connection_name.to_string(),
            is_recording,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_EVENTS] Failed to emit recording state changed event: {}", e);
        }
    }

    /// Handle streaming state changed events
    async fn handle_streaming_state_changed(&self, connection_name: &str, is_streaming: bool) {
        log::info!("[OBS_EVENTS] Streaming state changed for '{}': {}", connection_name, is_streaming);
        
        let event = ObsEvent::StreamStateChanged {
            connection_name: connection_name.to_string(),
            is_streaming,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_EVENTS] Failed to emit streaming state changed event: {}", e);
        }
    }

    /// Handle replay buffer state changed events
    async fn handle_replay_buffer_state_changed(&self, connection_name: &str, is_active: bool) {
        log::info!("[OBS_EVENTS] Replay buffer state changed for '{}': {}", connection_name, is_active);
        
        let event = ObsEvent::ReplayBufferStateChanged {
            connection_name: connection_name.to_string(),
            is_active,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_EVENTS] Failed to emit replay buffer state changed event: {}", e);
        }
    }

    /// Handle raw events
    async fn handle_raw_event(&self, connection_name: &str, event_type: &str, event_data: serde_json::Value) {
        log::debug!("[OBS_EVENTS] Raw event '{}' for '{}'", event_type, connection_name);
        
        let event = ObsEvent::Raw {
            connection_name: connection_name.to_string(),
            data: event_data,
        };
        
        if let Err(e) = self.context.event_tx.send(event) {
            log::error!("[OBS_EVENTS] Failed to emit raw event: {}", e);
        }
    }

    /// Set debug WebSocket messages flag
    pub async fn set_debug_ws_messages(&self, enabled: bool) {
        let mut debug_flag = self.context.debug_ws_messages.lock().await;
        *debug_flag = enabled;
        
        log::info!("[OBS_EVENTS] Debug WebSocket messages: {}", enabled);
    }

    /// Set show full events flag
    pub async fn set_show_full_events(&self, enabled: bool) {
        let mut show_flag = self.context.show_full_events.lock().await;
        *show_flag = enabled;
        
        log::info!("[OBS_EVENTS] Show full events: {}", enabled);
    }
}

// Implement ObsPlugin trait for the events plugin
impl ObsPlugin for ObsEventsPlugin {
    fn name(&self) -> &str {
        "obs_events"
    }

    fn init(&self) -> AppResult<()> {
        log::info!("🔧 Initializing OBS Events Plugin");
        Ok(())
    }

    fn shutdown(&self) -> AppResult<()> {
        log::info!("🔧 Shutting down OBS Events Plugin");
        Ok(())
    }
} 