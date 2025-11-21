use crate::database::operations::PssUdpOperations;
use crate::entity::{athlete, match_participant, matches};
use crate::App;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};

type SeaConn = sea_orm::DatabaseConnection;

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualMatchData {
    pub player1: PlayerData,
    pub player2: PlayerData,
    pub match_number: String,
    pub category: String,
    pub weight: String,
    pub division: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerData {
    pub name: String,
    pub ioc_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreData {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Create a new manual match
#[tauri::command]
pub async fn manual_create_match(
    match_data: ManualMatchData,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("Creating manual match: {match_data:?}");

    let sea = app.database_plugin().seaorm();
    let txn = sea
        .begin()
        .await
        .map_err(|e| format!("Failed to start database transaction: {e}"))?;

    let now = Utc::now();
    let match_code = format!("manual_{}", match_data.match_number);

    let match_model = matches::ActiveModel {
        uuid: Set(Uuid::new_v4().to_string()),
        match_code: Set(match_code),
        match_number: Set(Some(match_data.match_number.clone())),
        category: Set(Some(match_data.category.clone())),
        weight_class_code: Set(Some(match_data.weight.clone())),
        division_code: Set(Some(match_data.division.clone())),
        total_rounds: Set(Some(3)),
        round_duration: Set(None),
        countdown_type: Set(None),
        format_type: Set(None),
        creation_mode: Set(Some("Manual".to_string())),
        created_at: Set(now.naive_utc()),
        updated_at: Set(now.naive_utc()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| format!("Failed to insert match: {e}"))?;

    let athlete_blue = athlete::ActiveModel {
        uuid: Set(Uuid::new_v4().to_string()),
        pss_code: Set(Some(format!("P1_{}", match_data.player1.ioc_code))),
        short_name: Set(Some(match_data.player1.name.clone())),
        display_name: Set(Some(match_data.player1.name.clone())),
        country_code: Set(Some(match_data.player1.ioc_code.clone())),
        ioc_code: Set(Some(match_data.player1.ioc_code.clone())),
        created_at: Set(now.naive_utc()),
        updated_at: Set(now.naive_utc()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| format!("Failed to insert athlete 1: {e}"))?;

    let athlete_red = athlete::ActiveModel {
        uuid: Set(Uuid::new_v4().to_string()),
        pss_code: Set(Some(format!("P2_{}", match_data.player2.ioc_code))),
        short_name: Set(Some(match_data.player2.name.clone())),
        display_name: Set(Some(match_data.player2.name.clone())),
        country_code: Set(Some(match_data.player2.ioc_code.clone())),
        ioc_code: Set(Some(match_data.player2.ioc_code.clone())),
        created_at: Set(now.naive_utc()),
        updated_at: Set(now.naive_utc()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| format!("Failed to insert athlete 2: {e}"))?;

    match_participant::ActiveModel {
        match_id: Set(match_model.id),
        athlete_id: Set(athlete_blue.id),
        side: Set("blue".to_string()),
        created_at: Set(now.naive_utc()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| format!("Failed to link athlete 1 to match: {e}"))?;

    match_participant::ActiveModel {
        match_id: Set(match_model.id),
        athlete_id: Set(athlete_red.id),
        side: Set("red".to_string()),
        created_at: Set(now.naive_utc()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| format!("Failed to link athlete 2 to match: {e}"))?;

    txn.commit()
        .await
        .map_err(|e| format!("Failed to commit manual match transaction: {e}"))?;

    let match_id = match_model.id as i64;

    log::info!("Successfully created manual match with ID: {match_id}");

    Ok(serde_json::json!({
        "success": true,
        "match_id": match_id,
        "message": "Manual match created successfully"
    }))
}

/// Restore all app data from database
#[tauri::command]
pub async fn manual_restore_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Restoring all app data from database");

    let db_conn = app.database_plugin().get_database_connection();
    let conn = db_conn
        .get_connection()
        .await
        .map_err(|e| format!("Failed to get database connection: {e}"))?;

    let sea = app.database_plugin().seaorm();
    let latest_match = match matches::Entity::find()
        .order_by_desc(matches::Column::CreatedAt)
        .one(&sea)
        .await
        .map_err(|e| format!("Failed to get latest match: {e}"))?
    {
        Some(model) => Some(
            serialize_match_json(&sea, model)
                .await
                .map_err(|err| format!("Failed to serialize match: {err}"))?,
        ),
        None => None,
    };

    // Get OBS connections
    let obs_connections = app
        .database_plugin()
        .list_obs_connections()
        .await
        .map_err(|e| format!("Failed to get OBS connections: {e}"))?;

    // Get UDP server configs via SeaORM-backed plugin helper
    let udp_configs = app
        .database_plugin()
        .get_udp_server_configs()
        .await
        .map_err(|e| format!("Failed to get UDP configs: {e}"))?;

    // Get settings
    let settings = PssUdpOperations::get_all_settings(&conn)
        .map_err(|e| format!("Failed to get settings: {e}"))?;

    let restore_data = serde_json::json!({
        "success": true,
        "message": "Data restored successfully",
        "data": {
            "latest_match": latest_match,
            "obs_connections": obs_connections,
            "udp_configs": udp_configs,
            "settings": settings
        }
    });

    log::info!("Successfully restored app data");

    Ok(restore_data)
}

/// Get manual match statistics
#[tauri::command]
pub async fn manual_get_statistics(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    log::info!("Getting manual match statistics");

    let sea = app.database_plugin().seaorm();

    let manual_matches = fetch_matches_by_creation_mode(&sea, "Manual")
        .await
        .map_err(|e| format!("Failed to get manual matches: {e}"))?;

    let automatic_matches = fetch_matches_by_creation_mode(&sea, "Automatic")
        .await
        .map_err(|e| format!("Failed to get automatic matches: {e}"))?;

    let manual_count = manual_matches.len();
    let automatic_count = automatic_matches.len();

    let statistics = serde_json::json!({
        "success": true,
        "statistics": {
            "manual_matches_count": manual_count,
            "automatic_matches_count": automatic_count,
            "total_matches_count": manual_count + automatic_count,
            "manual_matches": manual_matches,
            "automatic_matches": automatic_matches
        }
    });

    log::info!("Successfully retrieved manual match statistics");

    Ok(statistics)
}

async fn serialize_match_json(
    sea: &SeaConn,
    model: matches::Model,
) -> Result<serde_json::Value, DbErr> {
    let created_at = Utc.from_utc_datetime(&model.created_at);
    let updated_at = Utc.from_utc_datetime(&model.updated_at);

    let participants = match_participant::Entity::find()
        .filter(match_participant::Column::MatchId.eq(model.id))
        .order_by_asc(match_participant::Column::Side)
        .find_also_related(athlete::Entity)
        .all(sea)
        .await?;

    let participants_json: Vec<serde_json::Value> = participants
        .into_iter()
        .map(|(participant, athlete_opt)| {
            let participant_created = Utc.from_utc_datetime(&participant.created_at);
            let athlete_json = athlete_opt.map(|ath| {
                let athlete_created = Utc.from_utc_datetime(&ath.created_at);
                let athlete_updated = Utc.from_utc_datetime(&ath.updated_at);
                serde_json::json!({
                    "id": ath.id as i64,
                    "uuid": ath.uuid,
                    "pss_code": ath.pss_code,
                    "short_name": ath.short_name,
                    "display_name": ath.display_name,
                    "country_code": ath.country_code,
                    "ioc_code": ath.ioc_code,
                    "created_at": athlete_created.to_rfc3339(),
                    "updated_at": athlete_updated.to_rfc3339(),
                })
            });

            serde_json::json!({
                "id": participant.id as i64,
                "match_id": participant.match_id as i64,
                "athlete_id": participant.athlete_id as i64,
                "side": participant.side,
                "bg_color": participant.bg_color,
                "fg_color": participant.fg_color,
                "created_at": participant_created.to_rfc3339(),
                "athlete": athlete_json
            })
        })
        .collect();

    Ok(serde_json::json!({
        "id": model.id as i64,
        "uuid": model.uuid,
        "tournament_id": model.tournament_id.map(|v| v as i64),
        "tournament_day_id": Option::<i64>::None,
        "match_id": model.match_code,
        "match_number": model.match_number,
        "category": model.category,
        "weight_class": model.weight_class_code,
        "division": model.division_code,
        "total_rounds": model.total_rounds.unwrap_or(0),
        "round_duration": model.round_duration,
        "countdown_type": model.countdown_type,
        "format_type": model.format_type,
        "creation_mode": model.creation_mode.clone().unwrap_or_else(|| "Unknown".to_string()),
        "created_at": created_at.to_rfc3339(),
        "updated_at": updated_at.to_rfc3339(),
        "created": created_at.timestamp(),
        "updated": updated_at.timestamp(),
        "participants": participants_json
    }))
}

async fn fetch_matches_by_creation_mode(
    sea: &SeaConn,
    mode: &str,
) -> Result<Vec<serde_json::Value>, DbErr> {
    let records = matches::Entity::find()
        .filter(matches::Column::CreationMode.eq(Some(mode.to_string())))
        .order_by_desc(matches::Column::CreatedAt)
        .all(sea)
        .await?;

    let mut result = Vec::with_capacity(records.len());
    for record in records {
        result.push(serialize_match_json(sea, record).await?);
    }

    Ok(result)
}
