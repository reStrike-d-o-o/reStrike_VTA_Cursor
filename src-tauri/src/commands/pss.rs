use crate::core::app::App;
use crate::types::TauriError;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn store_pss_event_cmd(
    event_data: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Storing PSS event to database: {event_data:?}");

    // Extract basic fields expected from the UI store
    let match_id_str = event_data
        .get("match_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let event_code = event_data
        .get("event_code")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let athlete = event_data
        .get("athlete")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let round_num = event_data
        .get("round")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let time_str = event_data
        .get("time")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let timestamp_str = event_data
        .get("timestamp")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let raw_data = event_data
        .get("raw_data")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Guard: require a match id and event_code
    if match_id_str.is_empty() || event_code.is_empty() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Missing match_id or event_code"
        }));
    }

    let _conn = app
        .database_plugin()
        .get_connection()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Database connection error: {e}")))?;

    // Resolve DB ids
    let db_match_id = app
        .database_plugin()
        .get_or_create_pss_match(match_id_str)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to get/create match: {e}")))?;

    // Map event_code to event_type_id
    let event_type = app
        .database_plugin()
        .get_pss_event_type_by_code(event_code)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("Failed to resolve event type: {e}")))?;
    let event_type_id = if let Some(t) = event_type {
        t.id.unwrap_or(0)
    } else {
        0
    };

    // Current UDP session if available (fallback to 0)
    let session_id = 0i64;

    // Build model
    let timestamp = if !timestamp_str.is_empty() {
        chrono::DateTime::parse_from_rfc3339(timestamp_str)
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or(chrono::Utc::now())
    } else {
        chrono::Utc::now()
    };

    let round_id: Option<i64> = None;
    let parsed_data: Option<String> = None;
    let processing_time_ms: Option<i32> = None;
    let error_message: Option<String> = None;

    let event_model = crate::database::models::PssEventV2 {
        id: None,
        session_id,
        match_id: Some(db_match_id),
        round_id,
        event_type_id,
        timestamp,
        raw_data: raw_data.to_string(),
        parsed_data,
        event_sequence: 0,
        processing_time_ms,
        is_valid: true,
        error_message,
        recognition_status: "recognized".to_string(),
        protocol_version: Some("2.3".to_string()),
        parser_confidence: Some(1.0),
        validation_errors: None,
        tournament_id: None,
        created_at: chrono::Utc::now(),
    };

    match app.database_plugin().store_pss_event(&event_model).await {
        Ok(event_id) => {
            // Store a few details for convenience
            let details = vec![
                (
                    "round".to_string(),
                    Some(round_num.to_string()),
                    "i32".to_string(),
                ),
                (
                    "time".to_string(),
                    Some(time_str.to_string()),
                    "String".to_string(),
                ),
                (
                    "athlete".to_string(),
                    Some(athlete.to_string()),
                    "String".to_string(),
                ),
            ];
            let _ = app
                .database_plugin()
                .store_pss_event_details(event_id, &details)
                .await;

            Ok(serde_json::json!({
                "success": true,
                "event_id": event_id
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn protocol_get_versions(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting protocol versions");
    
    let protocol_manager = app.protocol_manager();
    
    // Get versions
    let versions = match protocol_manager.get_versions().await {
        Ok(v) => v.into_iter().map(|v| crate::config::types::ProtocolVersion {
            version: v.version,
            filename: v.filename,
            file_path: v.file_path,
            description: v.description,
            created_date: v.created_date,
            last_modified: v.last_modified,
            is_active: v.is_active,
            file_size: v.file_size,
            checksum: v.checksum,
        }).collect::<Vec<_>>(),
        Err(e) => {
            log::error!("Failed to get protocol versions: {e}");
            return Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }));
        }
    };

    // Get current protocol
    let current_protocol = match protocol_manager.get_current_protocol().await {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Failed to get current protocol: {e}");
            None
        }
    };

    log::info!("Successfully retrieved {} protocol versions", versions.len());

    Ok(serde_json::json!({
        "success": true,
        "versions": versions,
        "current_protocol": current_protocol
    }))
}
