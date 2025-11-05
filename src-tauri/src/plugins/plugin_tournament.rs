use crate::database::models::{Tournament, TournamentDay};
use crate::database::{operations::TournamentOperations, DatabaseConnection};
use crate::types::{AppError, AppResult};
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Initialize the tournament plugin
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Initializing Tournament Plugin...");
    // Tournament plugin doesn't need special initialization
    log::info!("Tournament Plugin initialized successfully");
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
        let mut conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        let tournament = Tournament::new(name, duration_days, city, country, country_code);

        let tournament_id = TournamentOperations::create_tournament(&mut conn, &tournament)
            .map_err(|e| AppError::ConfigError(format!("Failed to create tournament: {e}")))?;

        // Always create tournament days
        let start_date_for_days = start_date.unwrap_or_else(Utc::now);
        TournamentOperations::create_tournament_days(
            &mut conn,
            tournament_id,
            start_date_for_days,
            duration_days,
        )
        .map_err(|e| AppError::ConfigError(format!("Failed to create tournament days: {e}")))?;

        Ok(tournament_id)
    }

    /// Get all tournaments
    pub async fn get_tournaments(&self) -> AppResult<Vec<Tournament>> {
        let conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        TournamentOperations::get_tournaments(&conn)
            .map_err(|e| AppError::ConfigError(format!("Failed to get tournaments: {e}")))
    }

    /// Get tournament by ID
    pub async fn get_tournament(&self, tournament_id: i64) -> AppResult<Option<Tournament>> {
        let conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        TournamentOperations::get_tournament(&conn, tournament_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to get tournament: {e}")))
    }

    /// Update tournament
    pub async fn update_tournament(
        &self,
        tournament_id: i64,
        tournament: Tournament,
    ) -> AppResult<()> {
        let mut conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        // Fetch existing tournament to detect changes
        let existing = TournamentOperations::get_tournament(&conn, tournament_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to get tournament: {e}")))?;

        TournamentOperations::update_tournament(&mut conn, tournament_id, &tournament)
            .map_err(|e| AppError::ConfigError(format!("Failed to update tournament: {e}")))?;

        // If start_date or duration_days changed, regenerate tournament days
        if let Some(old) = existing {
            let old_start = old.start_date;
            let old_duration = old.duration_days;
            let new_start = tournament.start_date;
            let new_duration = tournament.duration_days;

            if old_start != new_start || old_duration != new_duration {
                // Remove existing days and recreate
                conn.execute(
                    "DELETE FROM tournament_days WHERE tournament_id = ?",
                    params![tournament_id],
                )
                .map_err(|e| {
                    AppError::ConfigError(format!("Failed clearing tournament days: {e}"))
                })?;

                let start_for_days = new_start.unwrap_or_else(Utc::now);
                TournamentOperations::create_tournament_days(
                    &mut conn,
                    tournament_id,
                    start_for_days,
                    new_duration,
                )
                .map_err(|e| {
                    AppError::ConfigError(format!("Failed to recreate tournament days: {e}"))
                })?;
            } else {
                // Ensure days exist for older tournaments created before day generation logic
                let existing_count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM tournament_days WHERE tournament_id = ?",
                        params![tournament_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                if existing_count == 0 {
                    let start_for_days = new_start.unwrap_or_else(Utc::now);
                    TournamentOperations::create_tournament_days(
                        &mut conn,
                        tournament_id,
                        start_for_days,
                        new_duration,
                    )
                    .map_err(|e| {
                        AppError::ConfigError(format!(
                            "Failed to create missing tournament days: {e}"
                        ))
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Delete tournament
    pub async fn delete_tournament(&self, tournament_id: i64) -> AppResult<()> {
        let mut conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        TournamentOperations::delete_tournament(&mut conn, tournament_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to delete tournament: {e}")))
    }

    /// Get tournament days for a tournament
    pub async fn get_tournament_days(&self, tournament_id: i64) -> AppResult<Vec<TournamentDay>> {
        let conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        TournamentOperations::get_tournament_days(&conn, tournament_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to get tournament days: {e}")))
    }

    /// Start a tournament day
    pub async fn start_tournament_day(&self, tournament_day_id: i64) -> AppResult<()> {
        let mut conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        TournamentOperations::start_tournament_day(&mut conn, tournament_day_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to start tournament day: {e}")))
    }

    /// End a tournament day
    pub async fn end_tournament_day(&self, tournament_day_id: i64) -> AppResult<()> {
        let mut conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        TournamentOperations::end_tournament_day(&mut conn, tournament_day_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to end tournament day: {e}")))
    }

    /// Get active tournament
    pub async fn get_active_tournament(&self) -> AppResult<Option<Tournament>> {
        let conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        TournamentOperations::get_active_tournament(&conn)
            .map_err(|e| AppError::ConfigError(format!("Failed to get active tournament: {e}")))
    }

    /// Get active tournament day
    pub async fn get_active_tournament_day(
        &self,
        tournament_id: i64,
    ) -> AppResult<Option<TournamentDay>> {
        let conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        TournamentOperations::get_active_tournament_day(&conn, tournament_id)
            .map_err(|e| AppError::ConfigError(format!("Failed to get active tournament day: {e}")))
    }

    /// Update tournament logo
    pub async fn update_tournament_logo(
        &self,
        tournament_id: i64,
        logo_path: String,
    ) -> AppResult<()> {
        let mut conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        TournamentOperations::update_tournament_logo(&mut conn, tournament_id, &logo_path)
            .map_err(|e| AppError::ConfigError(format!("Failed to update tournament logo: {e}")))
    }

    /// Verify city and country using OpenStreetMap Nominatim API
    pub async fn verify_city_country(
        &self,
        city: String,
        country: String,
    ) -> AppResult<LocationVerification> {
        tokio::task::spawn_blocking(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .map_err(|e| AppError::ConfigError(format!("Failed to create HTTP client: {e}")))?;

            let query = format!("{city}, {country}");
            let encoded_query = urlencoding::encode(&query);
            let url = format!(
                "https://nominatim.openstreetmap.org/search?q={encoded_query}&format=json&limit=1"
            );

            let response = client
                .get(&url)
                .header("User-Agent", "reStrike-VTA-Tournament-Manager/1.0")
                .send();

            match response {
                Ok(response) => {
                    if !response.status().is_success() {
                        return Ok(LocationVerification {
                            verified: false,
                            country_code: None,
                            display_name: None,
                        });
                    }

                    match response.json::<Vec<serde_json::Value>>() {
                        Ok(data) => {
                            if data.is_empty() {
                                return Ok(LocationVerification {
                                    verified: false,
                                    country_code: None,
                                    display_name: None,
                                });
                            }

                            let result = &data[0];
                            let display_name =
                                result["display_name"].as_str().unwrap_or("").to_string();
                            let country_code = result["address"]["country_code"]
                                .as_str()
                                .map(|s| s.to_uppercase());

                            Ok(LocationVerification {
                                verified: true,
                                country_code,
                                display_name: Some(display_name),
                            })
                        }
                        Err(_) => {
                            // JSON parsing failed, return unverified
                            Ok(LocationVerification {
                                verified: false,
                                country_code: None,
                                display_name: None,
                            })
                        }
                    }
                }
                Err(e) => {
                    // Network error - return unverified instead of failing
                    log::warn!("Location verification failed for '{query}': {e}");
                    Ok(LocationVerification {
                        verified: false,
                        country_code: None,
                        display_name: None,
                    })
                }
            }
        })
        .await
        .map_err(|e| AppError::ConfigError(format!("Task join error: {e}")))?
    }

    /// Get tournament statistics from PSS tables
    pub async fn get_tournament_statistics(
        &self,
        tournament_id: i64,
    ) -> AppResult<TournamentStatistics> {
        let conn = self.database.get_connection().await.map_err(|e| {
            AppError::ConfigError(format!("Failed to get database connection: {e}"))
        })?;

        let tournament_uuid: String = conn
            .query_row(
                "SELECT uuid FROM tournaments WHERE id = ?",
                params![tournament_id],
                |row| row.get(0),
            )
            .map_err(|e| AppError::ConfigError(format!("Failed to load tournament uuid: {e}")))?;

        let total_matches: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pss_matches WHERE tournament_id = ?",
                params![tournament_uuid.clone()],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let total_events: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pss_events WHERE tournament_id = ?",
                params![tournament_uuid.clone()],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let total_scores: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pss_scores WHERE tournament_id = ?",
                params![tournament_uuid.clone()],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let total_warnings: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pss_warnings WHERE tournament_id = ?",
                params![tournament_uuid.clone()],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let female_gender_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM look_genders WHERE UPPER(name) = 'WOMEN'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to resolve female gender id: {e}"))
            })?;
        let male_gender_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM look_genders WHERE UPPER(name) = 'MEN'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| AppError::ConfigError(format!("Failed to resolve male gender id: {e}")))?;

        let tournament_days = TournamentOperations::get_tournament_days(&conn, tournament_id)?;
        let mut day_stats: Vec<TournamentDayStats> = Vec::new();

        for day in &tournament_days {
            let date_str = day.date.format("%Y-%m-%d").to_string();
            let prefix = day.date.format("%Y%m%d").to_string();
            let like_pattern = format!("{prefix}%");

            let total_matches_for_day: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM pss_matches WHERE tournament_id = ? AND match_id LIKE ?",
                    params![tournament_uuid.clone(), like_pattern.clone()],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let female_athletes = if let Some(gender_id) = female_gender_id {
                conn
                    .query_row(
                        "SELECT COUNT(DISTINCT ma.athlete_id)
                           FROM pss_match_athletes ma
                           JOIN pss_matches m ON ma.match_id = m.uuid
                           JOIN athletes a ON a.id = ma.athlete_id
                           WHERE m.tournament_id = ? AND m.match_id LIKE ? AND a.look_gender_id = ?",
                        params![tournament_uuid.clone(), like_pattern.clone(), gender_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0)
            } else {
                0
            };

            let male_athletes = if let Some(gender_id) = male_gender_id {
                conn
                    .query_row(
                        "SELECT COUNT(DISTINCT ma.athlete_id)
                           FROM pss_match_athletes ma
                           JOIN pss_matches m ON ma.match_id = m.uuid
                           JOIN athletes a ON a.id = ma.athlete_id
                           WHERE m.tournament_id = ? AND m.match_id LIKE ? AND a.look_gender_id = ?",
                        params![tournament_uuid.clone(), like_pattern.clone(), gender_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0)
            } else {
                0
            };

            day_stats.push(TournamentDayStats {
                day_id: day.id.unwrap_or_default(),
                day_number: day.day_number,
                date: date_str,
                status: day.status.clone(),
                total_matches: total_matches_for_day,
                female_athletes,
                male_athletes,
            });
        }

        let mut champions: Vec<TournamentChampion> = Vec::new();
        let mut medal_stmt = conn
            .prepare(
                r#"
                SELECT category,
                       match_uuid,
                       match_id,
                       winner_color,
                       winner_name,
                       winner_country_code,
                       blue_score,
                       red_score,
                       medal_type,
                       medal_rank
                FROM tournament_champions
                WHERE tournament_uuid = ?
                ORDER BY COALESCE(category, ''), medal_rank ASC, match_id ASC
                "#,
            )
            .map_err(|e| AppError::ConfigError(format!("Failed to prepare medalist query: {e}")))?;
        let medal_rows = medal_stmt
            .query_map(params![tournament_uuid.clone()], |row| {
                let winner_color: String = row.get(3)?;
                let medal_type: String = row.get(8)?;
                Ok(TournamentChampion {
                    category: row.get(0)?,
                    match_uuid: row.get(1)?,
                    match_id: row.get(2)?,
                    winner_color: winner_color.to_uppercase(),
                    winner_name: row.get(4)?,
                    winner_country_code: row.get(5)?,
                    blue_score: row.get(6)?,
                    red_score: row.get(7)?,
                    medal_type,
                    medal_rank: row.get(9)?,
                })
            })
            .map_err(|e| AppError::ConfigError(format!("Failed to iterate medalist rows: {e}")))?;
        for row in medal_rows {
            champions.push(
                row.map_err(|e| {
                    AppError::ConfigError(format!("Failed to read medalist row: {e}"))
                })?,
            );
        }

        if champions.is_empty() {
            let mut champion_stmt = conn
                .prepare(
                    r#"
                    SELECT m.id, m.uuid, m.match_id, m.category
                    FROM pss_matches m
                    JOIN (
                        SELECT category, MAX(match_id) AS max_match_id
                        FROM pss_matches
                        WHERE tournament_id = ?
                        GROUP BY category
                    ) latest ON latest.category = m.category AND latest.max_match_id = m.match_id
                    WHERE m.tournament_id = ? AND m.category IS NOT NULL"#,
                )
                .map_err(|e| {
                    AppError::ConfigError(format!("Failed to prepare champion fallback query: {e}"))
                })?;

            let mut score_stmt = conn
                .prepare(
                    "SELECT athlete_position, score_value FROM pss_scores WHERE match_id = ? AND score_type = 'total'",
                )
                .map_err(|e| AppError::ConfigError(format!("Failed to prepare score query: {e}")))?;
            let mut winner_stmt = conn
                .prepare(
                    "SELECT a.display_name, a.country_code FROM pss_match_athletes ma JOIN athletes a ON a.id = ma.athlete_id WHERE ma.match_id = ? AND ma.athlete_position = ?",
                )
                .map_err(|e| AppError::ConfigError(format!("Failed to prepare winner query: {e}")))?;
            let mut final_event_stmt = conn
                .prepare(
                    "SELECT raw_data FROM pss_events WHERE match_id = ? AND event_type_id = (SELECT id FROM pss_event_types WHERE event_code = 'MATCH_FINISHED') ORDER BY event_sequence DESC LIMIT 1",
                )
                .map_err(|e| AppError::ConfigError(format!("Failed to prepare final event query: {e}")))?;

            let champion_rows = champion_stmt
                .query_map(
                    params![tournament_uuid.clone(), tournament_uuid.clone()],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, Option<String>>(3)?,
                        ))
                    },
                )
                .map_err(|e| {
                    AppError::ConfigError(format!("Failed to iterate champion rows: {e}"))
                })?;

            for row_result in champion_rows {
                let (match_db_id, match_uuid, external_match_id, category) =
                    row_result.map_err(|e| {
                        AppError::ConfigError(format!("Failed to read champion row: {e}"))
                    })?;
                let mut blue_score: i64 = 0;
                let mut red_score: i64 = 0;

                let scores = score_stmt
                    .query_map(params![match_uuid.clone()], |row| {
                        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
                    })
                    .map_err(|e| {
                        AppError::ConfigError(format!("Failed to query match scores: {e}"))
                    })?;
                for score in scores {
                    let (position, value) = score.map_err(|e| {
                        AppError::ConfigError(format!("Failed to read score row: {e}"))
                    })?;
                    match position {
                        1 => blue_score = value,
                        2 => red_score = value,
                        _ => {}
                    }
                }

                let mut winner_color = if blue_score >= red_score {
                    "BLUE".to_string()
                } else {
                    "RED".to_string()
                };
                if blue_score == red_score {
                    if let Some(raw) = final_event_stmt
                        .query_row(params![match_db_id], |row| row.get::<_, String>(0))
                        .optional()
                        .map_err(|e| {
                            AppError::ConfigError(format!(
                                "Failed to resolve final event payload: {e}"
                            ))
                        })?
                    {
                        if let Ok(value) = serde_json::from_str::<Value>(&raw) {
                            if let Some(entry) = value.get("entry_value").and_then(|v| v.as_str()) {
                                let upper = entry.to_uppercase();
                                if upper.starts_with("BLUE") {
                                    winner_color = "BLUE".to_string();
                                } else if upper.starts_with("RED") {
                                    winner_color = "RED".to_string();
                                }
                            }
                            if let Some(snapshot) = value.get("score_snapshot") {
                                blue_score = snapshot
                                    .get("blue")
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(blue_score);
                                red_score = snapshot
                                    .get("red")
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(red_score);
                            }
                        }
                    }
                }

                let winner_position = if winner_color.eq_ignore_ascii_case("RED") {
                    2
                } else {
                    1
                };
                let winner_info = winner_stmt
                    .query_row(params![match_uuid.clone(), winner_position], |row| {
                        Ok((
                            row.get::<_, Option<String>>(0)?,
                            row.get::<_, Option<String>>(1)?,
                        ))
                    })
                    .optional()
                    .map_err(|e| {
                        AppError::ConfigError(format!("Failed to resolve winner info: {e}"))
                    })?;

                let (winner_name, winner_country): (Option<String>, Option<String>) =
                    winner_info.unwrap_or_default();

                champions.push(TournamentChampion {
                    category,
                    match_uuid: match_uuid.clone(),
                    match_id: external_match_id,
                    winner_color: winner_color.to_uppercase(),
                    winner_name,
                    winner_country_code: winner_country,
                    blue_score,
                    red_score,
                    medal_type: "gold".to_string(),
                    medal_rank: 1,
                });
            }
        }

        champions.sort_by(|a, b| {
            let category_a = a.category.as_deref().unwrap_or("");
            let category_b = b.category.as_deref().unwrap_or("");
            category_a
                .cmp(category_b)
                .then(a.medal_rank.cmp(&b.medal_rank))
                .then(a.match_id.cmp(&b.match_id))
        });

        Ok(TournamentStatistics {
            total_matches,
            total_events,
            total_scores,
            total_warnings,
            day_stats,
            champions,
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
    pub day_stats: Vec<TournamentDayStats>,
    pub champions: Vec<TournamentChampion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentDayStats {
    pub day_id: i64,
    pub day_number: i32,
    pub date: String,
    pub status: String,
    pub total_matches: i64,
    pub female_athletes: i64,
    pub male_athletes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentChampion {
    pub category: Option<String>,
    pub match_uuid: String,
    pub match_id: String,
    pub winner_color: String,
    pub winner_name: Option<String>,
    pub winner_country_code: Option<String>,
    pub blue_score: i64,
    pub red_score: i64,
    pub medal_type: String,
    pub medal_rank: i64,
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
