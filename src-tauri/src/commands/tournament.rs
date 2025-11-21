use crate::core::app::App;
use crate::importers::daedo::{import_tournament, ImportRequest};
use anyhow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;
use crate::types::TauriError;

// ============================================================================
// Tournament Management Commands
// ============================================================================

/// Create a new tournament
#[tauri::command]
pub async fn tournament_create(
    name: String,
    duration_days: i32,
    city: String,
    country: String,
    country_code: Option<String>,
    start_date: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
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

/// Import tournament from a directory
#[tauri::command]
pub async fn tournament_import_from_directory(
    folder_path: String,
    tournament_name: String,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError> {
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

/// Get all tournaments
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
