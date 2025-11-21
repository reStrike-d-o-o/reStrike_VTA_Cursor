use std::path::{Path, PathBuf};

use crate::entity::{athlete, match_participant, matches, tournament};
use crate::importers::daedo::{import_tournament, ImportRequest};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

#[tauri::command]
pub async fn validate_tournament_pss_integrity(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let conn = app
        .database_plugin()
        .get_connection()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!(format!("DB connection error: {e}"))))?;
    let conn_ref = &*conn;
    // Counts
    let counts = |sql: &str| -> i64 {
        conn_ref
            .query_row(sql, [], |r| r.get::<_, i64>(0))
            .unwrap_or(0)
    };
    let c_tournaments = counts("SELECT COUNT(*) FROM tournaments");
    let c_days = counts("SELECT COUNT(*) FROM tournament_days");
    let c_matches = counts("SELECT COUNT(*) FROM pss_matches");
    let c_events = counts("SELECT COUNT(*) FROM pss_events");
    let c_vids = counts("SELECT COUNT(*) FROM recorded_videos");
    let c_links = counts("SELECT COUNT(*) FROM recorded_video_events");
    // Orphans
    let orphan_matches = counts("SELECT COUNT(*) FROM pss_matches m LEFT JOIN tournaments t ON t.id = m.tournament_id WHERE m.tournament_id IS NOT NULL AND t.id IS NULL");
    let orphan_events = counts("SELECT COUNT(*) FROM pss_events e LEFT JOIN pss_matches m ON m.id = e.match_id WHERE e.match_id IS NOT NULL AND m.id IS NULL");
    let orphan_vids = counts("SELECT COUNT(*) FROM recorded_videos rv LEFT JOIN pss_matches m ON m.id = rv.match_id WHERE rv.match_id IS NOT NULL AND m.id IS NULL");
    // Missing context
    let events_missing_ctx = counts("SELECT COUNT(*) FROM pss_events WHERE tournament_id IS NULL");
    let matches_missing_ctx =
        counts("SELECT COUNT(*) FROM pss_matches WHERE tournament_id IS NULL");
    Ok(serde_json::json!({
        "counts": {"tournaments": c_tournaments, "days": c_days, "matches": c_matches, "events": c_events, "videos": c_vids, "video_links": c_links},
        "orphans": {"matches": orphan_matches, "events": orphan_events, "videos": orphan_vids},
        "missing_context": {"events": events_missing_ctx, "matches": matches_missing_ctx}
    }))
}

// set_udp_tournament_context defined elsewhere in this module
// (removed) db_backfill_recorded_video_events in favor of purge strategy
// (removed) db_backfill_tournament_context in favor of purge strategy

// Command moved to commands/database.rs

use crate::core::app::App;
use crate::logging::archival::{ArchiveSchedule, AutoArchiveConfig};
use crate::plugins::obs_obws::types::ObsOperationRequest;
use crate::utils::simulation_env::ensure_simulation_env;
use dirs;
use once_cell::sync::{Lazy, OnceCell};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn normalize_fs_path(path: String) -> Result<String, TauriError> {
    if path.trim().is_empty() {
        return Err(TauriError::from(anyhow::anyhow!("Path cannot be empty")));
    }

    let raw_path = PathBuf::from(&path);
    let resolved = if raw_path.is_absolute() {
        raw_path
    } else {
        std::env::current_dir()
            .map_err(|e| {
                TauriError::from(anyhow::anyhow!(format!(
                    "Failed to determine current directory: {e}"
                )))
            })?
            .join(raw_path)
    };

    let normalized = match std::fs::canonicalize(&resolved) {
        Ok(canonical) => canonical,
        Err(_) => resolved.clean(),
    };

    let mut result = normalized.to_string_lossy().to_string();
    if cfg!(target_os = "windows") && result.starts_with("\\\\?\\") {
        result = result.trim_start_matches("\\\\?\\").to_string();
    }

    Ok(result)
}
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{Emitter, Error as TauriError, State};
use tokio::sync::Mutex as AsyncMutex;

async fn ensure_license_ok(app: &State<'_, Arc<App>>) -> Result<(), TauriError> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size: u64,
    pub modified: String,
    pub subsystem: String,
}

// Core app commands
// Command moved to commands/system.rs


// Moved to commands/obs.rs

// Moved to commands/obs.rs

// (legacy recording settings get/set removed)

// Recording Path and Filename Commands
// Moved to commands/obs.rs

// Moved to commands/obs.rs

// Moved to commands/obs.rs

// (legacy set_recording_format removed)

// Recording Settings Templates and Options
// (legacy recording options/templates removed)

// (legacy Replay Buffer Settings Commands removed)

// (legacy Advanced Replay Buffer Commands removed)

// (legacy replay buffer format get/set removed)

// (legacy replay buffer quality get/set removed)

// (legacy replay buffer bitrate get/set removed)
// (legacy replay buffer keyframe interval get/set removed)

// (legacy replay buffer rate control get/set removed)

// (legacy replay buffer preset get/set removed)

// (legacy replay buffer profile get/set removed)

// (legacy replay buffer tune get/set removed)

// (legacy replay buffer bulk settings removed)

// (legacy Replay Buffer Options Commands removed)

// (legacy streaming settings commands removed)

// Unused legacy status/event commands removed

// Moved to commands/obs.rs

// Moved to commands/obs.rs

// Video commands moved to commands/video.rs


// Store commands
// Moved to commands/store.rs
                log::warn!("Failed updating flag metadata for IOC {code_key}: {err}");
            }
        }
    }

    Ok(FlagCountryUpdateStats {
        mapping_entries: country_map.len(),
        applied_updates: updated,
    })
}
// Moved to commands/resources.rs
    log::info!("Scanning and populating flags table");

    let conn = match app.database_plugin().get_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get database connection: {}", e)
            }))
        }
    };

    // Path to the SVG flags directory (relative to project root)
    let flags_dir = std::path::Path::new("../ui/public/assets/flags/svg");

    if !flags_dir.exists() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Flags directory does not exist: ../ui/public/assets/flags/svg"
        }));
    }

    let mut processed_count = 0;
    let mut skipped_count = 0;
    let mut errors = Vec::new();

    // Read directory entries
    let entries = match std::fs::read_dir(flags_dir) {
        Ok(entries) => entries,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to read flags directory: {}", e)
            }))
        }
    };

    let current_time = chrono::Utc::now().to_rfc3339();

    for entry in entries {
        let entry = match entry {
// Flag commands moved to commands/resources.rs


#[tauri::command]
// Flag commands moved to commands/resources.rs
// Log commands moved to commands/logging.rs


// WebSocket commands for HTML overlays
// WebSocket commands moved to commands/overlays.rs

// PSS commands moved to commands/pss.rs

// Tournament Management Commands
// Moved to commands/tournament.rs
    log::info!("Creating tournament: {name} in {city}, {country}");

    let start_date_parsed = if let Some(date_str) = start_date {
        if date_str.trim().is_empty() {
            None
        } else {
            Some(
                chrono::DateTime::parse_from_rfc3339(&date_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(|e| {
                        TauriError::from(anyhow::anyhow!(
                            "Invalid start date format '{date_str}': {e}"
                        ))
                    })?,
            )
        }
    } else {
        None
    };

    match app
        .tournament_plugin()
        .create_tournament(
            name,
            duration_days,
            city,
            country,
            country_code,
            start_date_parsed,
        )
        .await
    {
        Ok(tournament_id) => Ok(serde_json::json!({
            "success": true,
            "tournament_id": tournament_id,
            "message": "Tournament created successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

// Moved to commands/tournament.rs
    log::info!("Importing tournament '{tournament_name}' from folder {folder_path}");

    let db_path = app.database_plugin().get_database_path().map_err(|e| {
        TauriError::from(anyhow::anyhow!(format!(
            "Failed to resolve database path: {e}"
        )))
    })?;

    let request = ImportRequest {
        archive_root: PathBuf::from(&folder_path),
        tournament_name: tournament_name.clone(),
    };

    let stats =
        tokio::task::spawn_blocking(move || import_tournament(Path::new(&db_path), request))
            .await
            .map_err(|e| TauriError::from(anyhow::anyhow!(format!("Import task panicked: {e}"))))?
            .map_err(|e| {
                TauriError::from(anyhow::anyhow!(format!("Failed to import tournament: {e}")))
            })?;

    Ok(serde_json::json!({
        "success": true,
        "stats": stats,
    }))
}

#[tauri::command]
pub async fn tournament_get_all(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting all tournaments");

    match app.tournament_plugin().get_tournaments().await {
        Ok(tournaments) => {
            let tournaments_json: Vec<serde_json::Value> = tournaments
                .into_iter()
                .map(|t| {
                    serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "duration_days": t.duration_days,
                        "city": t.city,
                        "country": t.country,
                        "country_code": t.country_code,
                        "logo_path": t.logo_path,
                        "status": t.status,
                        "start_date": t.start_date.map(|d| d.to_rfc3339()),
                        "end_date": t.end_date.map(|d| d.to_rfc3339()),
                        "created_at": t.created_at.to_rfc3339(),
                        "updated_at": t.updated_at.to_rfc3339(),
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "success": true,
                "tournaments": tournaments_json
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn tournament_get(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting tournament: {tournament_id}");

    match app.tournament_plugin().get_tournament(tournament_id).await {
        Ok(Some(tournament)) => {
            let tournament_json = serde_json::json!({
                "id": tournament.id,
                "uuid": tournament.uuid,
                "name": tournament.name,
                "duration_days": tournament.duration_days,
                "city": tournament.city,
                "country": tournament.country,
                "country_code": tournament.country_code,
                "logo_path": tournament.logo_path,
                "status": tournament.status,
                "start_date": tournament.start_date.map(|d| d.to_rfc3339()),
                "end_date": tournament.end_date.map(|d| d.to_rfc3339()),
                "created_at": tournament.created_at.to_rfc3339(),
                "updated_at": tournament.updated_at.to_rfc3339(),
            });

            Ok(serde_json::json!({
                "success": true,
                "tournament": tournament_json
            }))
        }
        Ok(None) => Ok(serde_json::json!({
            "success": false,
            "error": "Tournament not found"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
pub async fn tournament_update(
    tournament_id: i64,
    name: String,
    duration_days: i32,
    city: String,
    country: String,
    country_code: Option<String>,
    logo_path: Option<String>,
    status: String,
    start_date: Option<String>,
    end_date: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Updating tournament: {tournament_id}");

    let start_date_parsed = if let Some(date_str) = start_date {
        if date_str.trim().is_empty() {
            None
        } else {
            Some(
                chrono::DateTime::parse_from_rfc3339(&date_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(|e| {
                        TauriError::from(anyhow::anyhow!(
                            "Invalid start date format '{date_str}': {e}"
                        ))
                    })?,
            )
        }
    } else {
        None
    };

    let end_date_parsed = if let Some(date_str) = end_date {
        if date_str.trim().is_empty() {
            None
        } else {
            Some(
                chrono::DateTime::parse_from_rfc3339(&date_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(|e| {
                        TauriError::from(anyhow::anyhow!(
                            "Invalid end date format '{date_str}': {e}"
                        ))
                    })?,
            )
        }
    } else {
        None
    };

    let tournament = crate::database::models::Tournament {
        id: Some(tournament_id),
        uuid: None,
        name,
        duration_days,
        city,
        country,
        country_code,
        logo_path,
        status,
        start_date: start_date_parsed,
        end_date: end_date_parsed,
        ranking_id: None,
        location: serde_json::json!({}),
        contact: serde_json::json!({}),
        oc: serde_json::json!({}),
        officials: serde_json::json!({}),
        banner: None,
        created_at: chrono::Utc::now(), // This will be ignored in update
        updated_at: chrono::Utc::now(),
    };

    match app
        .tournament_plugin()
        .update_tournament(tournament_id, tournament)
        .await
    {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament updated successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn tournament_delete(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Deleting tournament: {tournament_id}");

    match app
        .tournament_plugin()
        .delete_tournament(tournament_id)
        .await
    {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament deleted successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}
#[tauri::command]
pub async fn tournament_get_days(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting tournament days for tournament: {tournament_id}");

    match app
        .tournament_plugin()
        .get_tournament_days(tournament_id)
        .await
    {
        Ok(days) => {
            let days_json: Vec<serde_json::Value> = days
                .into_iter()
                .map(|d| {
                    serde_json::json!({
                        "id": d.id,
                        "uuid": d.uuid,
                        "tournament_id": d.tournament_id,
                        "day_number": d.day_number,
                        "date": d.date.to_rfc3339(),
                        "status": d.status,
                        "start_time": d.start_time.map(|t| t.to_rfc3339()),
                        "end_time": d.end_time.map(|t| t.to_rfc3339()),
                        "created_at": d.created_at.to_rfc3339(),
                        "updated_at": d.updated_at.to_rfc3339(),
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "success": true,
                "days": days_json
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn tournament_start_day(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Starting tournament day: (removed)");

    let res1: Result<(), anyhow::Error> = Ok(());
    match res1 {
        Ok(()) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament day started successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn tournament_end_day(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Ending tournament day: (removed)");

    let res2: Result<(), anyhow::Error> = Ok(());
    match res2 {
        Ok(()) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament day ended successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn tournament_get_active(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting active tournament");

    match app.tournament_plugin().get_active_tournament().await {
        Ok(Some(tournament)) => {
            let tournament_json = serde_json::json!({
                "id": tournament.id,
                "name": tournament.name,
                "duration_days": tournament.duration_days,
                "city": tournament.city,
                "country": tournament.country,
                "country_code": tournament.country_code,
                "logo_path": tournament.logo_path,
                "status": tournament.status,
                "start_date": tournament.start_date.map(|d| d.to_rfc3339()),
                "end_date": tournament.end_date.map(|d| d.to_rfc3339()),
                "created_at": tournament.created_at.to_rfc3339(),
                "updated_at": tournament.updated_at.to_rfc3339(),
            });

            Ok(serde_json::json!({
                "success": true,
                "tournament": tournament_json
            }))
        }
        Ok(None) => Ok(serde_json::json!({
            "success": true,
            "tournament": null
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn tournament_get_active_day(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting active tournament day for tournament: {tournament_id}");

    match app
        .tournament_plugin()
        .get_active_tournament_day(tournament_id)
        .await
    {
        Ok(Some(day)) => {
            let day_json = serde_json::json!({
                "id": day.id,
                "tournament_id": day.tournament_id,
                "day_number": day.day_number,
                "date": day.date.to_rfc3339(),
                "status": day.status,
                "start_time": day.start_time.map(|t| t.to_rfc3339()),
                "end_time": day.end_time.map(|t| t.to_rfc3339()),
                "created_at": day.created_at.to_rfc3339(),
                "updated_at": day.updated_at.to_rfc3339(),
            });

            Ok(serde_json::json!({
                "success": true,
                "day": day_json
            }))
        }
        Ok(None) => Ok(serde_json::json!({
            "success": true,
            "day": null
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn tournament_update_logo(
    tournament_id: i64,
    logo_path: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Updating tournament logo for tournament: {tournament_id}");

    match app
        .tournament_plugin()
        .update_tournament_logo(tournament_id, logo_path)
        .await
    {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Tournament logo updated successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

#[tauri::command]
pub async fn tournament_verify_location(
    city: String,
    country: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Verifying location: {city}, {country}");

    match app
        .tournament_plugin()
        .verify_city_country(city, country)
        .await
    {
        Ok(verification) => Ok(serde_json::json!({
            "success": true,
            "verified": verification.verified,
            "country_code": verification.country_code,
            "display_name": verification.display_name
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}
#[tauri::command]
pub async fn get_tournament_statistics(
    tournament_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting tournament statistics for tournament: {tournament_id}");

    match app
        .tournament_plugin()
        .get_tournament_statistics(tournament_id)
        .await
    {
        Ok(statistics) => Ok(serde_json::json!({
            "success": true,
            "statistics": statistics
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

// Database optimization commands
#[tauri::command]
pub async fn database_run_vacuum(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database VACUUM operation");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_vacuum(&db_conn).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Database VACUUM completed successfully"
        })),
        Err(e) => {
            log::error!("Database VACUUM failed: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}
#[tauri::command]
pub async fn database_run_integrity_check(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database integrity check");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_integrity_check(&db_conn).await {
        Ok(integrity_ok) => {
            if integrity_ok {
                Ok(serde_json::json!({
                    "success": true,
                    "message": "Database integrity check passed",
                    "integrity_ok": true
                }))
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "message": "Database integrity check failed",
                    "integrity_ok": false
                }))
            }
        }
        Err(e) => {
            log::error!("Database integrity check error: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_analyze(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database ANALYZE operation");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_analyze(&db_conn).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Database ANALYZE completed successfully"
        })),
        Err(e) => {
            log::error!("Database ANALYZE failed: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_optimize(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running database OPTIMIZE operation");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_optimize(&db_conn).await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Database OPTIMIZE completed successfully"
        })),
        Err(e) => {
            log::error!("Database OPTIMIZE failed: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_run_full_maintenance(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running full database maintenance");

    let db_conn = app.database_plugin().get_database_connection();
    let mut maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.run_full_maintenance(&db_conn).await {
        Ok(result) => Ok(serde_json::json!({
            "success": true,
            "message": "Full database maintenance completed",
            "result": {
                "integrity_check_passed": result.integrity_check_passed,
                "analyze_success": result.analyze_success,
                "optimize_success": result.optimize_success,
                "vacuum_success": result.vacuum_success,
                "total_duration_secs": result.total_duration.as_secs()
            }
        })),
        Err(e) => {
            log::error!("Full database maintenance failed: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_get_info(app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database information");

    let db_conn = app.database_plugin().get_database_connection();
    let maintenance = crate::database::DatabaseMaintenance::new_default();

    match maintenance.get_database_info(&db_conn).await {
        Ok(info) => Ok(serde_json::json!({
            "success": true,
            "info": {
                "total_size": info.total_size,
                "used_size": info.used_size,
                "free_size": info.free_size,
                "fragmentation_percentage": info.fragmentation_percentage,
                "page_count": info.page_count,
                "page_size": info.page_size,
                "freelist_count": info.freelist_count,
                "cache_size": info.cache_size,
                "journal_mode": info.journal_mode,
                "synchronous": info.synchronous
            }
        })),
        Err(e) => {
            log::error!("Failed to get database info: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

#[tauri::command]
pub async fn database_get_maintenance_status(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting database maintenance status");

    let maintenance = crate::database::DatabaseMaintenance::new_default();
    let needed = maintenance.check_maintenance_needed();
    let stats = maintenance.get_statistics();
    let config = maintenance.get_config();

    Ok(serde_json::json!({
        "success": true,
        "maintenance_needed": {
            "vacuum_needed": needed.vacuum_needed,
            "integrity_check_needed": needed.integrity_check_needed,
            "analyze_needed": needed.analyze_needed,
            "optimize_needed": needed.optimize_needed,
            "any_needed": needed.any_needed()
        },
        "statistics": {
            "last_vacuum": stats.last_vacuum,
            "last_integrity_check": stats.last_integrity_check,
            "last_analyze": stats.last_analyze,
            "last_optimize": stats.last_optimize,
            "vacuum_count": stats.vacuum_count,
            "integrity_check_count": stats.integrity_check_count,
            "analyze_count": stats.analyze_count,
            "optimize_count": stats.optimize_count,
            "total_maintenance_time_secs": stats.total_maintenance_time_secs
        },
        "config": {
            "vacuum_interval_secs": config.vacuum_interval.as_secs(),
            "integrity_check_interval_secs": config.integrity_check_interval.as_secs(),
            "analyze_interval_secs": config.analyze_interval.as_secs(),
            "optimize_interval_secs": config.optimize_interval.as_secs(),
            "max_vacuum_time_secs": config.max_vacuum_time.as_secs(),
            "backup_before_maintenance": config.backup_before_maintenance
        }
    }))
}

/// Get comprehensive event statistics with status breakdown
#[tauri::command]
pub async fn get_comprehensive_event_statistics(
    session_id: i64,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting comprehensive event statistics for session {session_id}");

    match app
        .database_plugin()
        .get_comprehensive_event_statistics(session_id)
        .await
    {
        Ok(stats) => Ok(stats),
        Err(e) => {
            log::error!("Failed to get comprehensive event statistics: {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to get event statistics: {e}"
            )))
        }
    }
}

/// Get events by recognition status
#[tauri::command]
pub async fn get_events_by_status(
    session_id: i64,
    recognition_status: String,
    limit: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting events by status: {recognition_status} for session {session_id}");

    match app
        .database_plugin()
        .get_events_by_status(session_id, &recognition_status, limit)
        .await
    {
        Ok(events) => {
            let events_json: Vec<serde_json::Value> = events
                .into_iter()
                .map(|event| serde_json::to_value(event).unwrap_or_default())
                .collect();
            Ok(serde_json::json!({
                "success": true,
                "events": events_json,
                "count": events_json.len()
            }))
        }
        Err(e) => {
            log::error!("Failed to get events by status: {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to get events by status: {e}"
            )))
        }
    }
}

/// Get unknown events for analysis
#[tauri::command]
pub async fn get_unknown_events(
    session_id: Option<i64>,
    limit: Option<i64>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting unknown events");

    match app
        .database_plugin()
        .get_unknown_events(session_id, limit)
        .await
    {
        Ok(events) => {
            let events_json: Vec<serde_json::Value> = events
                .into_iter()
                .map(|event| serde_json::to_value(event).unwrap_or_default())
                .collect();
            Ok(serde_json::json!({
                "success": true,
                "events": events_json,
                "count": events_json.len()
            }))
        }
        Err(e) => {
            log::error!("Failed to get unknown events: {e}");
            Err(TauriError::from(anyhow::anyhow!(
                "Failed to get unknown events: {e}"
            )))
        }
    }
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

/// Phase 2 Optimization: Archive events older than specified days
#[tauri::command]
pub async fn archive_old_events(
    app: tauri::State<'_, crate::core::app::App>,
    days_old: i64,
) -> Result<usize, TauriError> {
    log::info!("Archiving events older than {days_old} days");
    let archived_count = app
        .database_plugin()
        .archive_old_events(days_old)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
    log::info!("Archived {archived_count} events");
    Ok(archived_count)
}

/// Phase 2 Optimization: Get archive statistics
#[tauri::command]
pub async fn get_archive_statistics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting archive statistics");
    let stats = app
        .database_plugin()
        .get_archive_statistics()
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
    serde_json::to_value(stats).map_err(|e| {
        TauriError::from(anyhow::anyhow!(
            "Failed to serialize archive statistics: {e}"
        ))
    })
}

/// Phase 2 Optimization: Restore events from archive
#[tauri::command]
pub async fn restore_from_archive(
    app: tauri::State<'_, crate::core::app::App>,
    start_date: String,
    end_date: String,
) -> Result<usize, TauriError> {
    log::info!("Restoring events from archive between {start_date} and {end_date}");
    let restored_count = app
        .database_plugin()
        .restore_from_archive(&start_date, &end_date)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
    log::info!("Restored {restored_count} events from archive");
    Ok(restored_count)
}
/// Phase 2 Optimization: Clean up old archive data
#[tauri::command]
pub async fn cleanup_old_archive_data(
    app: tauri::State<'_, crate::core::app::App>,
    days_old: i64,
) -> Result<usize, TauriError> {
    log::info!("Cleaning up archive data older than {days_old} days");
    let deleted_count = app
        .database_plugin()
        .cleanup_old_archive_data(days_old)
        .await
        .map_err(|e| TauriError::from(anyhow::anyhow!("{e}")))?;
    log::info!("Cleaned up {deleted_count} archived events");
    Ok(deleted_count)
}

/// Phase 2 Optimization: Optimize archive tables
#[tauri::command]
pub async fn optimize_archive_tables(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<(), TauriError> {
    log::info!("Optimizing archive tables");
    app.database_plugin()
        .optimize_archive_tables()
        .await
// Phase 3: Event Stream Commands
#[tauri::command]
pub async fn get_stream_statistics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, tauri::Error> {
    let stream_stats = app.event_stream_processor().get_statistics().await;
    serde_json::to_value(stream_stats).map_err(|e| {
        tauri::Error::from(anyhow::anyhow!(
            "Failed to serialize stream statistics: {e}"
        ))
    })
}

#[tauri::command]
pub async fn send_event_to_stream(
    app: tauri::State<'_, crate::core::app::App>,
    event: crate::database::models::PssEventV2,
) -> Result<(), tauri::Error> {
    app.event_stream_processor()
        .send_event(event)
        .await
        .map_err(|e| {
            tauri::Error::from(anyhow::anyhow!(
                "Failed to enqueue event for stream processing: {e}"
            ))
        })?;
    Ok(())
}

// Phase 3: Load Balancer Commands
#[tauri::command]
pub async fn get_distributor_statistics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, tauri::Error> {
    let distributor_stats = app.event_distributor().get_statistics().await;
    serde_json::to_value(distributor_stats).map_err(|e| {
        tauri::Error::from(anyhow::anyhow!(
            "Failed to serialize distributor statistics: {e}"
        ))
    })
}

#[tauri::command]
pub async fn get_server_statistics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, tauri::Error> {
    let server_stats = app.event_distributor().get_server_statistics().await;
    serde_json::to_value(server_stats).map_err(|e| {
        tauri::Error::from(anyhow::anyhow!(
            "Failed to serialize server statistics: {e}"
        ))
    })
}
#[tauri::command]
pub async fn add_server(
    app: tauri::State<'_, crate::core::app::App>,
    server_id: String,
    bind_address: String,
    port: u16,
) -> Result<(), tauri::Error> {
    app.event_distributor()
        .add_server(server_id, bind_address, port)
        .await
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to add server: {e}")))
}

#[tauri::command]
pub async fn remove_server(
    app: tauri::State<'_, crate::core::app::App>,
    server_id: String,
) -> Result<(), tauri::Error> {
    app.event_distributor()
        .remove_server(&server_id)
        .await
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Failed to remove server: {e}")))
}

// Phase 3: Advanced Analytics Commands
#[tauri::command]
pub async fn get_tournament_analytics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, tauri::Error> {
    let tournament_analytics = app.advanced_analytics().get_tournament_analytics().await;
    serde_json::to_value(tournament_analytics).map_err(|e| {
        tauri::Error::from(anyhow::anyhow!(
            "Failed to serialize tournament analytics: {e}"
        ))
    })
}

#[tauri::command]
pub async fn get_performance_analytics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, tauri::Error> {
    let performance_analytics = app.advanced_analytics().get_performance_analytics().await;
    serde_json::to_value(performance_analytics).map_err(|e| {
        tauri::Error::from(anyhow::anyhow!(
            "Failed to serialize performance analytics: {e}"
        ))
    })
}

#[tauri::command]
pub async fn get_athlete_analytics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, tauri::Error> {
    let athlete_analytics = app.advanced_analytics().get_athlete_analytics().await;
    serde_json::to_value(athlete_analytics).map_err(|e| {
        tauri::Error::from(anyhow::anyhow!(
            "Failed to serialize athlete analytics: {e}"
        ))
    })
}

#[tauri::command]
pub async fn get_match_analytics(
    app: tauri::State<'_, crate::core::app::App>,
) -> Result<serde_json::Value, tauri::Error> {
    let match_analytics = app.advanced_analytics().get_match_analytics().await;
    serde_json::to_value(match_analytics).map_err(|e| {
        tauri::Error::from(anyhow::anyhow!("Failed to serialize match analytics: {e}"))
    })
}

#[tauri::command]
pub async fn get_analytics_history(
    app: tauri::State<'_, crate::core::app::App>,
    limit: Option<usize>,
) -> Result<serde_json::Value, tauri::Error> {
    let analytics_history = app.advanced_analytics().get_analytics_history(limit).await;
    serde_json::to_value(analytics_history).map_err(|e| {
        tauri::Error::from(anyhow::anyhow!(
            "Failed to serialize analytics history: {e}"
        ))
    })
}

#[tauri::command]
// Moved to commands/obs.rs
    log::info!("Getting scenes from all connected OBS instances");

    let mut all_scenes = Vec::new();
    let connection_names = app.obs_obws_plugin().get_connection_names().await;

    // Collect connection statuses first
    let mut connection_statuses = Vec::new();
    for connection_name in &connection_names {
        let status = app
            .obs_obws_plugin()
            .get_connection_status(connection_name)
            .await;
        connection_statuses.push((connection_name.clone(), status));
    }

    for (connection_name, status) in &connection_statuses {
        // Check if connection is connected/authenticated
        let is_connected = status.is_ok();

        if is_connected {
            match app
                .obs_obws_plugin()
                .get_scenes(Some(connection_name.as_str()))
                .await
            {
                Ok(scene_names) => {
                    for (idx, scene_name) in scene_names.iter().enumerate() {
                        all_scenes.push(serde_json::json!({
                            "id": idx,
                            "scene_name": scene_name,
                            "scene_id": scene_name, // OBS WebSocket v5 uses scene name as ID
                            "is_active": true,
                            "connection_name": connection_name
                        }));
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get scenes from connection '{connection_name}': {e}");
                }
            }
        } else {
            log::info!(
                "Skipping connection '{connection_name}' - not connected (status: {status:?})"
            );
        }
    }

    let connected_count = connection_statuses
        .iter()
        .filter(|(_, status)| status.is_ok())
        .count();

    Ok(serde_json::json!({
        "scenes": all_scenes,
        "total_connections": connection_names.len(),
        "connected_connections": connected_count
    }))
}
// Simulation commands
#[tauri::command]
pub async fn simulation_start(
    mode: String,
    scenario: String,
    duration: u32,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Starting simulation: mode={mode}, scenario={scenario}, duration={duration}");

    // Get the actual UDP settings from the app configuration
    let udp_settings = app.config_manager().get_udp_settings().await;
    let host = udp_settings.listener.bind_address.clone();
    let port = udp_settings.listener.port;

    log::info!("Using UDP settings: host={host}, port={port}");

    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };
    let result = std::process::Command::new(&python_cmd)
        .args([
            sim_main.to_str().unwrap(),
            "--mode",
            &mode,
            "--scenario",
            &scenario,
            "--duration",
            &duration.to_string(),
            "--host",
            &host,
            "--port",
            &port.to_string(),
        ])
        .spawn();
    match result {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Simulation started successfully on {}:{}", host, port)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to start simulation: {}", e)
        })),
    }
}

#[tauri::command]
pub async fn simulation_stop(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError> {
    log::info!("Stopping simulation");

    let result = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "python.exe"])
        .output();

    match result {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Simulation stopped successfully"
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to stop simulation: {}", e)
        })),
    }
}
#[tauri::command]
pub async fn simulation_get_status(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting simulation status");

    let result = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq python.exe"])
        .output();

    let is_running = match result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.contains("python.exe")
        }
        Err(_) => false,
    };

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "isRunning": is_running,
            "isConnected": is_running, // Assume connected if running
            "currentScenario": if is_running { "Unknown" } else { "None" },
            "currentMode": if is_running { "Unknown" } else { "None" },
            "eventsSent": 0, // Would need to track this separately
            "lastEvent": if is_running { "Unknown" } else { "None" }
        }
    }))
}

#[tauri::command]
pub async fn simulation_send_event(
    event_type: String,
    params: serde_json::Value,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Sending simulation event: type={event_type}, params={params:?}");

    // Get the actual UDP settings from the app configuration
    let udp_settings = app.config_manager().get_udp_settings().await;
    let host = udp_settings.listener.bind_address.clone();
    let port = udp_settings.listener.port;

    log::info!("Using UDP settings: host={host}, port={port}");

    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };

    // Convert params to JSON string for command line
    let params_str = serde_json::to_string(&params).unwrap_or("{}".to_string());

    let result = std::process::Command::new(&python_cmd)
        .args([
            sim_main.to_str().unwrap(),
            "--mode",
            "interactive",
            "--send-event",
            "--event-type",
            &event_type,
            "--event-params",
            &params_str,
            "--host",
            &host,
            "--port",
            &port.to_string(),
        ])
        .spawn();

    match result {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("{} event sent successfully to {}:{}", event_type, host, port)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to send {} event: {}", event_type, e)
        })),
    }
}

// New automated simulation commands
#[tauri::command]
pub async fn simulation_get_scenarios(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting available automated scenarios");

    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Simulation environment error: {e:?}");
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }));
        }
    };

    log::info!(
        "Running command: {} {} --list-scenarios",
        python_cmd,
        sim_main.to_str().unwrap()
    );

    let result = std::process::Command::new(&python_cmd)
        .args([sim_main.to_str().unwrap(), "--list-scenarios"])
        .output();

    match result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            log::info!("Scenarios output: {output_str}");
            if !stderr_str.is_empty() {
                log::warn!("Scenarios stderr: {stderr_str}");
            }

            if !output.status.success() {
                log::error!(
                    "Command failed with exit code: {}",
                    output.status.code().unwrap_or(-1)
                );
                return Ok(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get scenarios: Command exited with code {}", output.status.code().unwrap_or(-1))
                }));
            }

            // Parse the scenarios from the output
            let scenarios = parse_scenarios_from_output(&output_str);
            log::info!("Parsed {} scenarios: {:?}", scenarios.len(), scenarios);

            if scenarios.is_empty() {
                log::warn!("No scenarios were parsed from the output");
                return Ok(serde_json::json!({
                    "success": false,
                    "error": "No scenarios found in the output"
                }));
            }

            Ok(serde_json::json!({
                "success": true,
                "data": scenarios
            }))
        }
        Err(e) => {
            log::error!("Failed to execute simulation command: {e}");
            Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to get scenarios: {}", e)
            }))
        }
    }
}

#[tauri::command]
pub async fn simulation_run_automated(
    scenario_name: String,
    custom_config: Option<serde_json::Value>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running automated simulation: scenario={scenario_name}");

    // Get the actual UDP settings from the app configuration
    let udp_settings = app.config_manager().get_udp_settings().await;
    let host = udp_settings.listener.bind_address.clone();
    let port = udp_settings.listener.port;

    log::info!("Using UDP settings: host={host}, port={port}");

    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };

    let mut args = vec![
        sim_main.to_str().unwrap().to_string(),
        "--mode".to_string(),
        "automated".to_string(),
        "--scenario".to_string(),
        scenario_name.clone(),
        "--host".to_string(),
        host.clone(),
        "--port".to_string(),
        port.to_string(),
    ];

    // Add custom config if provided
    if let Some(config) = custom_config {
        if let Some(config_str) = config.as_str() {
            args.extend_from_slice(&["--config".to_string(), config_str.to_string()]);
        }
    }

    let result = std::process::Command::new(&python_cmd).args(&args).spawn();

    match result {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("Automated {} simulation started successfully on {}:{}", scenario_name, host, port)
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to start automated simulation: {}", e)
        })),
    }
}
#[tauri::command]
pub async fn simulation_get_detailed_status(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::debug!("Getting detailed simulation status");

    // Check if Python process is running
    let process_result = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq python.exe"])
        .output();

    let is_running = match process_result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.contains("python.exe")
        }
        Err(_) => false,
    };

    // Retrieve scenarios with caching to avoid repeated filesystem/process churn
    let scenarios = cached_scenarios();

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "isRunning": is_running,
            "isConnected": is_running,
            "currentScenario": if is_running { "Running" } else { "None" },
            "currentMode": if is_running { "Automated" } else { "None" },
            "eventsSent": 0,
            "lastEvent": if is_running { "Processing" } else { "None" },
            "automatedScenarios": scenarios
        }
    }))
}

#[tauri::command]
pub async fn simulation_run_self_test(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running comprehensive self-test");

    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };

    let result = std::process::Command::new(&python_cmd)
        .args([sim_main.to_str().unwrap(), "--self-test"])
        .output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                Ok(serde_json::json!({
                    "success": true,
                    "data": {
                        "output": stdout.to_string(),
                        "error": stderr.to_string(),
                        "exitCode": output.status.code().unwrap_or(0)
                    }
                }))
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "error": format!("Self-test failed: {}", stderr),
                    "data": {
                        "output": stdout.to_string(),
                        "error": stderr.to_string(),
                        "exitCode": output.status.code().unwrap_or(1)
                    }
                }))
            }
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to run self-test: {}", e)
        })),
    }
}

#[tauri::command]
pub async fn simulation_get_self_test_report(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting self-test report");

    let report_path = match crate::utils::simulation_env::get_simulation_main_py() {
        Ok(sim_main) => {
            let sim_dir = sim_main.parent().unwrap();
            sim_dir.join("self_test_report.md")
        }
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Failed to resolve simulation path: {:?}", e)
            }))
        }
    };

    let result = std::fs::read_to_string(&report_path);

    match result {
        Ok(content) => Ok(serde_json::json!({
            "success": true,
            "data": {
                "report": content,
                "path": report_path.to_str().unwrap_or("unknown")
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to read self-test report: {}", e)
        })),
    }
}

#[tauri::command]
pub async fn simulation_get_self_test_categories(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Getting self-test categories");

    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };

    let result = std::process::Command::new(&python_cmd)
        .args([sim_main.to_str().unwrap(), "--list-test-categories"])
        .output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                // Parse categories from output
                let categories: Vec<String> = stdout
                    .lines()
                    .filter(|line| line.trim().starts_with("• "))
                    .map(|line| line.trim()[2..].to_string())
                    .collect();

                Ok(serde_json::json!({
                    "success": true,
                    "data": {
                        "categories": categories,
                        "output": stdout.to_string(),
                        "error": stderr.to_string()
                    }
                }))
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get categories: {}", stderr),
                    "data": {
                        "output": stdout.to_string(),
                        "error": stderr.to_string()
                    }
                }))
            }
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to get categories: {}", e)
        })),
    }
}

#[tauri::command]
pub async fn simulation_run_selective_self_test(
    selected_categories: Vec<String>,
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    log::info!("Running selective self-test for categories: {selected_categories:?}");

    let (python_cmd, sim_main) = match ensure_simulation_env() {
        Ok(v) => v,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("Simulation environment error: {:?}", e)
            }))
        }
    };

    let mut args = vec![sim_main.to_str().unwrap(), "--self-test"];
    args.extend(selected_categories.iter().map(|s| s.as_str()));

    let result = std::process::Command::new(&python_cmd).args(&args).output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                Ok(serde_json::json!({
                    "success": true,
                    "data": {
                        "output": stdout.to_string(),
                        "error": stderr.to_string(),
                        "exitCode": output.status.code().unwrap_or(0)
                    }
                }))
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "error": format!("Selective self-test failed: {}", stderr),
                    "data": {
                        "output": stdout.to_string(),
                        "error": stderr.to_string(),
                        "exitCode": output.status.code().unwrap_or(1)
                    }
                }))
            }
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to run selective self-test: {}", e)
        })),
    }
}
// Helper function to parse scenarios from command output
type ScenarioCacheStore = std::sync::Mutex<Option<(Instant, Vec<serde_json::Value>)>>;

static SCENARIOS_CACHE: Lazy<ScenarioCacheStore> = Lazy::new(|| std::sync::Mutex::new(None));
const SCENARIOS_CACHE_TTL: Duration = Duration::from_secs(60);

fn parse_scenarios_from_output(output: &str) -> Vec<serde_json::Value> {
    let mut scenarios = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    log::debug!("Parsing {} lines from output", lines.len());
    log::debug!("Raw output: {output}");

    let mut current_scenario = serde_json::Map::new();
    let mut in_scenario = false;

    for (line_num, line) in lines.iter().enumerate() {
        let line = line.trim();
        log::debug!("Line {line_num}: '{line}'");

        // Check for scenario start with multiple possible bullet characters
        if line.starts_with("• ")
            || line.starts_with("- ")
            || line.starts_with("* ")
            || line.starts_with("  ")
            || line.starts_with("ò ")
        {
            // New scenario
            if in_scenario && !current_scenario.is_empty() {
                scenarios.push(serde_json::Value::Object(current_scenario.clone()));
                log::debug!("Added scenario: {current_scenario:?}");
            }

            current_scenario.clear();
            in_scenario = true;

            // Extract name after bullet point
            const BULLET_PREFIXES: [&str; 5] = ["• ", "- ", "* ", "  ", "ò "];
            let name = BULLET_PREFIXES
                .iter()
                .find_map(|prefix| line.strip_prefix(prefix))
                .unwrap_or(line)
                .trim();

            if !name.is_empty() {
                current_scenario.insert(
                    "display_name".to_string(),
                    serde_json::Value::String(name.to_string()),
                );
                current_scenario.insert(
                    "name".to_string(),
                    serde_json::Value::String(name.to_lowercase().replace(" ", "_")),
                );
                log::debug!("Found scenario: {name}");
            }
        } else if line.starts_with("  Description: ") && in_scenario {
            let description = line[14..].trim();
            current_scenario.insert(
                "description".to_string(),
                serde_json::Value::String(description.to_string()),
            );
            log::debug!("Added description: {description}");
        } else if line.starts_with("  Matches: ") && in_scenario {
            let matches = line[10..].trim();
            if let Ok(count) = matches.parse::<i32>() {
                current_scenario.insert(
                    "match_count".to_string(),
                    serde_json::Value::Number(count.into()),
                );
                log::debug!("Added match count: {count}");
            }
        } else if line.starts_with("  Est. Duration: ") && in_scenario {
            let duration = line[16..].trim();
            // Handle "45.0 seconds" format
            let duration_parts: Vec<&str> = duration.split_whitespace().collect();
            if let Some(seconds_str) = duration_parts.first() {
                if let Ok(seconds) = seconds_str.parse::<f64>() {
                    current_scenario.insert(
                        "estimated_duration".to_string(),
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(seconds)
                                .unwrap_or(serde_json::Number::from(0)),
                        ),
                    );
                    log::debug!("Added duration: {seconds}");
                }
            }
        }
    }

    // Add the last scenario
    if in_scenario && !current_scenario.is_empty() {
        scenarios.push(serde_json::Value::Object(current_scenario.clone()));
        log::debug!("Added final scenario: {current_scenario:?}");
    }

    log::debug!("Parsed {} scenarios successfully", scenarios.len());
    scenarios
}

fn cached_scenarios() -> Vec<serde_json::Value> {
    if let Some((timestamp, scenarios)) = SCENARIOS_CACHE.lock().unwrap().clone() {
        if timestamp.elapsed() < SCENARIOS_CACHE_TTL {
            log::debug!(
                "Using cached simulation scenarios ({} entries).",
                scenarios.len()
            );
            return scenarios;
        }
    }

    match ensure_simulation_env() {
        Ok((python_cmd, sim_main)) => {
            match std::process::Command::new(&python_cmd)
                .args([sim_main.to_str().unwrap(), "--list-scenarios"])
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let scenarios = parse_scenarios_from_output(&output_str);
                    let mut cache = SCENARIOS_CACHE.lock().unwrap();
                    *cache = Some((Instant::now(), scenarios.clone()));
                    scenarios
                }
                Err(err) => {
                    log::warn!("Failed to execute scenario listing command: {err}");
                    Vec::new()
                }
            }
        }
        Err(err) => {
            log::warn!("Simulation environment unavailable for scenario listing: {err:?}");
            Vec::new()
        }
    }
}

// (legacy streaming accounts/channels/events removed)

// ===== YOUTUBE STREAMING MANAGEMENT COMMANDS =====

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_accounts(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;

    match app
        .obs_obws_plugin()
        .get_status(Some(&connection_name))
        .await
    {
        Ok(accounts) => Ok(serde_json::json!({
            "success": true,
            "data": accounts
        })),
        Err(e) => {
            log::error!("Failed to get YouTube accounts: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_accounts(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_channels(
    app: State<'_, Arc<App>>,
    account_id: String,
) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;

    match app
        .obs_obws_plugin()
        .get_status(Some(&connection_name))
        .await
    {
        Ok(channels) => Ok(serde_json::json!({
            "success": true,
            "data": channels
        })),
        Err(e) => {
            log::error!("Failed to get YouTube channels: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_channels(
    _app: State<'_, Arc<App>>,
    _account_id: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_stream_key(
    app: State<'_, Arc<App>>,
    channel_id: String,
) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;

    match app
        .obs_obws_plugin()
        .get_status(Some(&connection_name))
        .await
    {
        Ok(stream_key_info) => Ok(serde_json::json!({
            "success": true,
            "data": stream_key_info
        })),
        Err(e) => {
            log::error!("Failed to get YouTube stream key: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_stream_key(
    _app: State<'_, Arc<App>>,
    _channel_id: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_set_youtube_streaming_config(
    app: State<'_, Arc<App>>,
    channel_id: String,
    config: serde_json::Value,
) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;

    match app
        .obs_obws_plugin()
        .get_status(Some(&connection_name))
        .await
    {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "YouTube streaming configuration updated successfully"
        })),
        Err(e) => {
            log::error!("Failed to set YouTube streaming config: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_set_youtube_streaming_config(
    _app: State<'_, Arc<App>>,
    _channel_id: String,
    _config: serde_json::Value,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_categories(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    match app.obs_obws_plugin().get_status(None).await {
        Ok(categories) => Ok(serde_json::json!({
            "success": true,
            "data": categories
        })),
        Err(e) => {
            log::error!("Failed to get YouTube categories: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_categories(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_privacy_options(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    match app.obs_obws_plugin().get_status(None).await {
        Ok(privacy_options) => Ok(serde_json::json!({
            "success": true,
            "data": privacy_options
        })),
        Err(e) => {
            log::error!("Failed to get YouTube privacy options: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_privacy_options(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_latency_options(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    match app.obs_obws_plugin().get_status(None).await {
        Ok(latency_options) => Ok(serde_json::json!({
            "success": true,
            "data": latency_options
        })),
        Err(e) => {
            log::error!("Failed to get YouTube latency options: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_latency_options(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_server_urls(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    match app.obs_obws_plugin().get_status(None).await {
        Ok(server_urls) => Ok(serde_json::json!({
            "success": true,
            "data": server_urls
        })),
        Err(e) => {
            log::error!("Failed to get YouTube server URLs: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_server_urls(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_regenerate_youtube_stream_key(
    app: State<'_, Arc<App>>,
    channel_id: String,
) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;

    match app
        .obs_obws_plugin()
        .get_status(Some(&connection_name))
        .await
    {
        Ok(new_stream_key_info) => Ok(serde_json::json!({
            "success": true,
            "data": new_stream_key_info
        })),
        Err(e) => {
            log::error!("Failed to regenerate YouTube stream key: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_regenerate_youtube_stream_key(
    _app: State<'_, Arc<App>>,
    _channel_id: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}
#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_streaming_analytics(
    app: State<'_, Arc<App>>,
    channel_id: String,
) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;

    match app
        .obs_obws_plugin()
        .get_status(Some(&connection_name))
        .await
    {
        Ok(analytics) => Ok(serde_json::json!({
            "success": true,
            "data": analytics
        })),
        Err(e) => {
            log::error!("Failed to get YouTube streaming analytics: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_streaming_analytics(
    _app: State<'_, Arc<App>>,
    _channel_id: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_get_youtube_streaming_schedule(
    app: State<'_, Arc<App>>,
    channel_id: String,
) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;

    match app
        .obs_obws_plugin()
        .get_status(Some(&connection_name))
        .await
    {
        Ok(schedule) => Ok(serde_json::json!({
            "success": true,
            "data": schedule
        })),
        Err(e) => {
            log::error!("Failed to get YouTube streaming schedule: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_get_youtube_streaming_schedule(
    _app: State<'_, Arc<App>>,
    _channel_id: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

#[cfg(feature = "youtube")]
pub async fn obs_create_youtube_streaming_schedule(
    app: State<'_, Arc<App>>,
    channel_id: String,
    schedule_data: serde_json::Value,
) -> Result<serde_json::Value, TauriError> {
    let connection_name = app.get_default_connection_name().await?;

    match app
        .obs_obws_plugin()
        .get_status(Some(&connection_name))
        .await
    {
        Ok(created_schedule) => Ok(serde_json::json!({
            "success": true,
            "data": created_schedule
        })),
        Err(e) => {
            log::error!("Failed to create YouTube streaming schedule: {}", e);
            Err(TauriError::from(e))
        }
    }
}
#[cfg(not(feature = "youtube"))]
pub async fn obs_create_youtube_streaming_schedule(
    _app: State<'_, Arc<App>>,
    _channel_id: String,
    _schedule_data: serde_json::Value,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "disabled": true }))
}

// (legacy streaming destination commands removed)

// YouTube API Commands (disabled by default)
#[tauri::command]
pub async fn youtube_get_auth_url(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_authenticate(
    _app: State<'_, Arc<App>>,
    _code: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_create_playlist(
    _app: State<'_, Arc<App>>,
    _title: String,
    _description: Option<String>,
    _privacy: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_playlists(
    _app: State<'_, Arc<App>>,
    _max_results: Option<u32>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_add_video_to_playlist(
    _app: State<'_, Arc<App>>,
    _playlist_id: String,
    _video_id: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_playlist_videos(
    _app: State<'_, Arc<App>>,
    _playlist_id: String,
    _max_results: Option<u32>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_update_playlist(
    _app: State<'_, Arc<App>>,
    _playlist_id: String,
    _title: Option<String>,
    _description: Option<String>,
    _privacy: Option<String>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_delete_playlist(
    _app: State<'_, Arc<App>>,
    _playlist_id: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_create_scheduled_stream(
    _app: State<'_, Arc<App>>,
    _title: String,
    _description: Option<String>,
    _scheduled_time: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_live_streams(
    _app: State<'_, Arc<App>>,
    _max_results: Option<u32>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_scheduled_streams(
    _app: State<'_, Arc<App>>,
    _max_results: Option<u32>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_completed_streams(
    _app: State<'_, Arc<App>>,
    _max_results: Option<u32>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_end_stream(
    _app: State<'_, Arc<App>>,
    _stream_id: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_channel_info(
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_get_video_analytics(
    _app: State<'_, Arc<App>>,
    _video_id: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}
#[tauri::command]
pub async fn youtube_initialize(
    _app: State<'_, Arc<App>>,
    _client_id: String,
    _client_secret: String,
    _redirect_uri: String,
) -> Result<serde_json::Value, TauriError> {
    Ok(serde_json::json!({ "success": true, "disabled": true }))
}


// Control Room commands moved to commands/control_room.rs
// OVR commands moved to commands/ovr.rs
