use crate::core::app::App;
use anyhow;
use std::sync::Arc;
use tauri::State;
use crate::types::TauriError;

// ============================================================================
// UDP Server Commands
// ============================================================================

/// Start the UDP server
#[tauri::command]
pub async fn start_udp_server(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Starting UDP server");
    
    // Check license
    crate::utils::ensure_license_ok(&app).await?;
    
    let config = app.config_manager().get_config().await;
    app.udp_plugin()
        .start(&config)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
    
    // Ensure UDP event handler is running so PSS events reach auto-recording
    app.inner().start_udp_event_handler().await;
    log::info!("UDP event handler started (manual start)");
    
    Ok(())
}

/// Stop the UDP server
#[tauri::command]
pub async fn stop_udp_server(app: State<'_, Arc<App>>) -> Result<(), TauriError> {
    log::info!("Stopping UDP server");
    app.udp_plugin()
        .stop()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
    Ok(())
}

/// Get UDP server status
#[tauri::command]
pub async fn get_udp_status(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting UDP status");
    let status = app.udp_plugin().get_status();

    let (status_str, is_running, error_msg) = match status {
        crate::plugins::plugin_udp::UdpServerStatus::Stopped => ("Stopped", false, None),
        crate::plugins::plugin_udp::UdpServerStatus::Starting => ("Starting", false, None),
        crate::plugins::plugin_udp::UdpServerStatus::Running => ("Running", true, None),
        crate::plugins::plugin_udp::UdpServerStatus::Error(e) => ("Error", false, Some(e)),
    };

    Ok(serde_json::json!({
        "status": status_str,
        "is_running": is_running,
        "error": error_msg
    }))
}

#[tauri::command]
pub async fn update_udp_settings(
    settings: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    log::info!("Updating UDP settings: {settings:?}");

    // Update the app configuration
    app.config_manager()
        .update_udp_settings_from_json(settings)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;

    // If UDP server is running, restart it with new settings
    let status = app.udp_plugin().get_status();
    if matches!(status, crate::plugins::plugin_udp::UdpServerStatus::Running) {
        log::info!("UDP server is running, restarting with new settings");
        app.udp_plugin()
            .stop()
            .await
            .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
        let config = app.config_manager().get_config().await;
        app.udp_plugin()
            .start(&config)
            .await
            .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
        // Re-ensure event handler is started after restart
        app.inner().start_udp_event_handler().await;
        log::info!("UDP event handler started (restart)");
    }

    Ok(())
}

/// Set tournament context for UDP event tracking
#[tauri::command]
pub async fn set_udp_tournament_context(
    app: tauri::State<'_, crate::core::app::App>,
    tournament_id: Option<i64>,
) -> Result<(), TauriError> {
    log::info!("Setting UDP tournament context: tournament_id={tournament_id:?}");

    app.udp_plugin()
        .set_tournament_context(tournament_id)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))
}

/// Get current tournament context from UDP server
#[tauri::command]
pub async fn get_udp_tournament_context(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, TauriError> {
    let tournament_id = app.udp_plugin().get_tournament_context();

    Ok(serde_json::json!({
        "tournament_id": tournament_id,

    }))
}

/// Clear tournament context from UDP server
#[tauri::command]
pub async fn clear_udp_tournament_context(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<(), TauriError> {
    log::info!("Clearing UDP tournament context");

    app.udp_plugin()
        .clear_tournament_context()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))
}

/// Phase 1 Optimization: Get UDP server performance metrics
#[tauri::command]
pub async fn get_udp_performance_metrics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting UDP performance metrics");
    let metrics = app.udp_plugin().get_performance_metrics();
    serde_json::to_value(metrics).map_err(|e| {
        TauriError::from(anyhow::anyhow!(
            "Failed to serialize performance metrics: {e}"
        ))
    })
}

/// Phase 1 Optimization: Get UDP server memory usage
#[tauri::command]
pub async fn get_udp_memory_usage(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting UDP memory usage");
    let usage = app.udp_plugin().get_memory_usage();
    serde_json::to_value(usage)
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to serialize memory usage: {e}")))
}
