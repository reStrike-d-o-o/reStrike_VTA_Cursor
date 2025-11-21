use crate::database::models::ObsConnection;
use chrono::Utc;
use std::sync::Arc;
use tauri::{command, Error as TauriError, State};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObsConnectionPayload {
    pub id: Option<i64>,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub is_active: bool,
    pub status: String,
    pub error: Option<String>,
}

// Commands moved to commands/obs.rs
