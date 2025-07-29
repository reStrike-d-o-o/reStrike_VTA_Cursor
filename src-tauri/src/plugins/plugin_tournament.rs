use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::database::{DatabaseConnection, operations::TournamentOperations};
use crate::database::models::{Tournament, TournamentDay};
use crate::types::{AppResult, AppError};
use rusqlite::params;

/// Initialize the tournament plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏆 Initializing Tournament Plugin...");
    // Tournament plugin doesn't need special initialization
    println!("✅ Tournament Plugin initialized successfully");
    Ok(())
}

/// Tournament plugin for managing tournaments and tournament days
#[derive(Clone)]
pub struct TournamentPlugin {
    database: Arc<DatabaseConnection>,
}

impl TournamentPlugin {
    /// Create a new tournament plugin
    pub fn new(database: Arc<DatabaseConnection>) -> Self {
        Self { database }
    }
    
    /// Create a new tournament
    pub async fn create_tournament(
        &self,
        name: String,
        duration_days: i32,
        city: String,
        country: String,
        country_code: Option<String>,
        start_date: Option<DateTime<Utc>>,
    ) -> AppResult<i64> {
        let mut conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        let tournament = Tournament::new(
            name,
            duration_days,
            city,
            country,
            country_code,
        );
        
        let tournament_id = TournamentOperations::create_tournament(&mut *conn, &tournament)
            .map_err(|e| AppError::ConfigError(format!("Failed to create tournament: {}", e)))?;
        
        // Create tournament days if start_date is provided
        if let Some(start_date) = start_date {
            TournamentOperations::create_tournament_days(&mut *conn, tournament_id, start_date, duration_days)
                .map_err(|e| AppError::ConfigError(format!("Failed to create tournament days: {}", e)))?;
        }
        
        Ok(tournament_id)
    }
    
    /// Get all tournaments
    pub async fn get_tournaments(&self) -> AppResult<Vec<Tournament>> {
        let conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::get_tournaments(&*conn)
            .map_err(|e| AppError::ConfigError(format!("Failed to get tournaments: {}", e)))
    }
    
    /// Get tournament by ID
    pub async fn get_tournament(&self, tournament_id: i64) -> AppResult<Option<Tournament>> {
        let conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::get_tournament(&*conn, tournament_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to get tournament: {}", e)))
    }
    
    /// Update tournament
    pub async fn update_tournament(
        &self,
        tournament_id: i64,
        tournament: Tournament,
    ) -> AppResult<()> {
        let mut conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::update_tournament(&mut *conn, tournament_id, &tournament)
            .map_err(|e| AppError::ConfigError(format!("Failed to update tournament: {}", e)))
    }
    
    /// Delete tournament
    pub async fn delete_tournament(&self, tournament_id: i64) -> AppResult<()> {
        let mut conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::delete_tournament(&mut *conn, tournament_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to delete tournament: {}", e)))
    }
    
    /// Get tournament days for a tournament
    pub async fn get_tournament_days(&self, tournament_id: i64) -> AppResult<Vec<TournamentDay>> {
        let conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::get_tournament_days(&*conn, tournament_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to get tournament days: {}", e)))
    }
    
    /// Start a tournament day
    pub async fn start_tournament_day(&self, tournament_day_id: i64) -> AppResult<()> {
        let mut conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::start_tournament_day(&mut *conn, tournament_day_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to start tournament day: {}", e)))
    }
    
    /// End a tournament day
    pub async fn end_tournament_day(&self, tournament_day_id: i64) -> AppResult<()> {
        let mut conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::end_tournament_day(&mut *conn, tournament_day_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to end tournament day: {}", e)))
    }
    
    /// Get active tournament
    pub async fn get_active_tournament(&self) -> AppResult<Option<Tournament>> {
        let conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::get_active_tournament(&*conn)
            .map_err(|e| AppError::ConfigError(format!("Failed to get active tournament: {}", e)))
    }
    
    /// Get active tournament day
    pub async fn get_active_tournament_day(&self, tournament_id: i64) -> AppResult<Option<TournamentDay>> {
        let conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::get_active_tournament_day(&*conn, tournament_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to get active tournament day: {}", e)))
    }
    
    /// Update tournament logo
    pub async fn update_tournament_logo(&self, tournament_id: i64, logo_path: String) -> AppResult<()> {
        let mut conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        TournamentOperations::update_tournament_logo(&mut *conn, tournament_id, &logo_path)
            .map_err(|e| AppError::ConfigError(format!("Failed to update tournament logo: {}", e)))
    }
    
    /// Verify city and country using OpenStreetMap Nominatim API
    pub async fn verify_city_country(&self, city: String, country: String) -> AppResult<LocationVerification> {
        tokio::task::spawn_blocking(move || {
            let client = reqwest::blocking::Client::new();
            let query = format!("{}, {}", city, country);
            let encoded_query = urlencoding::encode(&query);
            let url = format!("https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1", encoded_query);
            
            let response = client.get(&url)
                .header("User-Agent", "reStrike-VTA-Tournament-Manager/1.0")
                .send()
                .map_err(|e| AppError::ConfigError(format!("Failed to send request: {}", e)))?;
            
            if !response.status().is_success() {
                return Err(AppError::ConfigError(format!("API request failed with status: {}", response.status())));
            }
            
            let data: Vec<serde_json::Value> = response.json()
                .map_err(|e| AppError::ConfigError(format!("Failed to parse JSON response: {}", e)))?;
            
            if data.is_empty() {
                return Ok(LocationVerification {
                    verified: false,
                    country_code: None,
                    display_name: None,
                });
            }
            
            let result = &data[0];
            let display_name = result["display_name"].as_str().unwrap_or("").to_string();
            let country_code = result["address"]["country_code"].as_str().map(|s| s.to_uppercase());
            
            Ok(LocationVerification {
                verified: true,
                country_code,
                display_name: Some(display_name),
            })
        }).await
        .map_err(|e| AppError::ConfigError(format!("Task join error: {}", e)))?
    }

    /// Get tournament statistics from PSS tables
    pub async fn get_tournament_statistics(&self, tournament_id: i64) -> AppResult<TournamentStatistics> {
        let conn = self.database.get_connection().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get database connection: {}", e)))?;
        
        // Get total matches
        let total_matches: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pss_matches WHERE tournament_id = ?",
            params![tournament_id],
            |row| row.get(0)
        ).unwrap_or(0);
        
        // Get total events
        let total_events: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pss_events_v2 WHERE tournament_id = ?",
            params![tournament_id],
            |row| row.get(0)
        ).unwrap_or(0);
        
        // Get total scores
        let total_scores: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pss_scores WHERE tournament_id = ?",
            params![tournament_id],
            |row| row.get(0)
        ).unwrap_or(0);
        
        // Get total warnings
        let total_warnings: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pss_warnings WHERE tournament_id = ?",
            params![tournament_id],
            |row| row.get(0)
        ).unwrap_or(0);
        
        Ok(TournamentStatistics {
            total_matches,
            total_events,
            total_scores,
            total_warnings,
        })
    }
}

/// Location verification result from OpenStreetMap API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationVerification {
    pub verified: bool,
    pub country_code: Option<String>,
    pub display_name: Option<String>,
}

/// Tournament statistics from PSS tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentStatistics {
    pub total_matches: i64,
    pub total_events: i64,
    pub total_scores: i64,
    pub total_warnings: i64,
}

/// Request for creating a tournament
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTournamentRequest {
    pub name: String,
    pub duration_days: i32,
    pub city: String,
    pub country: String,
    pub country_code: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
}

/// Request for updating a tournament
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTournamentRequest {
    pub name: String,
    pub duration_days: i32,
    pub city: String,
    pub country: String,
    pub country_code: Option<String>,
    pub logo_path: Option<String>,
    pub status: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}