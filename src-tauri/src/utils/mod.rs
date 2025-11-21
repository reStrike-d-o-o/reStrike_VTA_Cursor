pub mod logger;
pub mod network;
pub mod simulation_env;

pub use network::*;

use crate::core::app::App;
use std::sync::Arc;
use tauri::{State, Error as TauriError};

/// Generate a new UUID v4 string
pub fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Return current Unix timestamp (seconds since epoch)
pub fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Normalize a filesystem path for cross-platform compatibility
pub fn normalize_fs_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Ensure the license is valid
pub async fn ensure_license_ok(app: &State<'_, Arc<App>>) -> Result<(), TauriError> {
    let status = app
        .license_plugin()
        .validate(app.config_manager())
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(e.to_string())))?;
    let allowed = matches!(
        status.state,
        crate::plugins::plugin_license::LicenseState::Valid
    );
    if allowed {
        Ok(())
    } else {
        Err(TauriError::from(anyhow::anyhow!(format!(
            "License not valid: state={:?}, reason={:?}",
            status.state, status.reason
        ))))
    }
}
