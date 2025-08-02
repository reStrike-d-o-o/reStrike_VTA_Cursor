use crate::{App, AppResult, types::AppError};
use crate::database::models::{PssMatch, PssAthlete, PssMatchAthlete};
use crate::database::operations::PssUdpOperations;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;

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
    log::info!("Creating manual match: {:?}", match_data);
    
    let conn = app.database_plugin().get_database_connection().await
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    // Create the match
    let mut pss_match = PssMatch::new(format!("manual_{}", match_data.match_number));
    pss_match.match_number = Some(match_data.match_number);
    pss_match.category = Some(match_data.category);
    pss_match.weight_class = Some(match_data.weight);
    pss_match.division = Some(match_data.division);
    pss_match.creation_mode = "Manual".to_string();
    
    let match_id = PssUdpOperations::insert_pss_match(&*conn, &pss_match)
        .map_err(|e| format!("Failed to insert match: {}", e))?;
    
    // Create athletes
    let athlete1 = PssAthlete::new(
        format!("P1_{}", match_data.player1.ioc_code),
        match_data.player1.name.clone(),
    );
    let athlete1_id = PssUdpOperations::insert_pss_athlete(&*conn, &athlete1)
        .map_err(|e| format!("Failed to insert athlete 1: {}", e))?;
    
    let athlete2 = PssAthlete::new(
        format!("P2_{}", match_data.player2.ioc_code),
        match_data.player2.name.clone(),
    );
    let athlete2_id = PssUdpOperations::insert_pss_athlete(&*conn, &athlete2)
        .map_err(|e| format!("Failed to insert athlete 2: {}", e))?;
    
    // Create match-athlete relationships
    let match_athlete1 = PssMatchAthlete::new(match_id, athlete1_id, 1);
    PssUdpOperations::insert_pss_match_athlete(&*conn, &match_athlete1)
        .map_err(|e| format!("Failed to insert match athlete 1: {}", e))?;
    
    let match_athlete2 = PssMatchAthlete::new(match_id, athlete2_id, 2);
    PssUdpOperations::insert_pss_match_athlete(&*conn, &match_athlete2)
        .map_err(|e| format!("Failed to insert match athlete 2: {}", e))?;
    
    log::info!("Successfully created manual match with ID: {}", match_id);
    
    Ok(serde_json::json!({
        "success": true,
        "match_id": match_id,
        "message": "Manual match created successfully"
    }))
}

/// Restore all app data from database
#[tauri::command]
pub async fn manual_restore_data(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("Restoring all app data from database");
    
    let conn = app.database_plugin().get_database_connection().await
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    // Get the latest match data
    let matches = PssUdpOperations::get_pss_matches(&*conn, Some(1))
        .map_err(|e| format!("Failed to get matches: {}", e))?;
    
    let latest_match = matches.first();
    
    // Get OBS connections
    let obs_connections = PssUdpOperations::get_obs_connections(&*conn)
        .map_err(|e| format!("Failed to get OBS connections: {}", e))?;
    
    // Get UDP server configs
    let udp_configs = PssUdpOperations::get_udp_server_configs(&*conn)
        .map_err(|e| format!("Failed to get UDP configs: {}", e))?;
    
    // Get settings
    let settings = PssUdpOperations::get_all_settings(&*conn)
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    
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
pub async fn manual_get_statistics(
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    log::info!("Getting manual match statistics");
    
    let conn = app.database_plugin().get_database_connection().await
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    // Get all manual matches
    let manual_matches = PssUdpOperations::get_pss_matches_by_creation_mode(&*conn, "Manual")
        .map_err(|e| format!("Failed to get manual matches: {}", e))?;
    
    // Get all automatic matches
    let automatic_matches = PssUdpOperations::get_pss_matches_by_creation_mode(&*conn, "Automatic")
        .map_err(|e| format!("Failed to get automatic matches: {}", e))?;
    
    let statistics = serde_json::json!({
        "success": true,
        "statistics": {
            "manual_matches_count": manual_matches.len(),
            "automatic_matches_count": automatic_matches.len(),
            "total_matches_count": manual_matches.len() + automatic_matches.len(),
            "manual_matches": manual_matches,
            "automatic_matches": automatic_matches
        }
    });
    
    log::info!("Successfully retrieved manual match statistics");
    
    Ok(statistics)
} 