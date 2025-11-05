use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use uuid::Uuid;

use re_strike_vta::database::{
    connection::DatabaseConnection, migrations::MigrationManager,
    operations::TournamentRankingOperations,
};

const TOURNAMENT_NAME: &str = "German Open - Hamburg 2025";
const START_DATE: &str = "2025-09-13";
const END_DATE: &str = "2025-09-14";
const VENUE: &str = "Sporthalle Hamburg";
const VENUE_ADDRESS: &str = "Krochmannstraße 55";
const VENUE_POSTAL: &str = "22297";
const VENUE_CITY: &str = "Hamburg";
const VENUE_COUNTRY: &str = "Germany";
const VENUE_COUNTRY_CODE: &str = "DEU";
const BANNER_PATH: &str = r"C:\Users\Damjan\Downloads\go2025.jpg";
const OCTAGON_LABEL_PREFIX: &str = "Court";
const COURT_COUNT: usize = 12;

fn main() -> Result<()> {
    env_logger::init();

    let db_path = DatabaseConnection::get_database_path()?;
    let mut conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open database at {}", db_path.display()))?;

    // Apply pending migrations first
    let manager = MigrationManager::new();
    manager
        .migrate(&conn)
        .context("Failed to apply database migrations")?;

    // Lookup ranking id (default to G2)
    let ranking = TournamentRankingOperations::get_by_code(&conn, "G2")
        .context("Failed to lookup ranking code G2")?;
    let ranking_id = ranking.and_then(|r| r.id);

    // Prepare JSON payloads
    let location_json = json!({
        "venue": VENUE,
        "address": VENUE_ADDRESS,
        "postal_code": VENUE_POSTAL,
        "city": VENUE_CITY,
        "country": VENUE_COUNTRY
    })
    .to_string();

    let contact_json = json!({
        "person": "",
        "web": "",
        "email": "",
        "mob_phone": ""
    })
    .to_string();

    let oc_json = json!({
        "governing_org": [
            "Deutsche Taekwondo Union e.V.",
            "Hamburger Sportfreunde e.V. (HSF)"
        ],
        "persons": [],
        "contacts": []
    })
    .to_string();

    let officials_json = json!({
        "TD": {},
        "RC": {},
        "others": []
    })
    .to_string();

    let banner_base64 = read_and_encode_banner(Path::new(BANNER_PATH))?;

    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM tournaments WHERE name = ?1",
            params![TOURNAMENT_NAME],
            |row| row.get(0),
        )
        .optional()?;

    let start_rfc3339 = format!("{START_DATE}T00:00:00Z");
    let end_rfc3339 = format!("{END_DATE}T23:59:59Z");
    let now_iso = chrono::Utc::now().to_rfc3339();
    let now_unix = chrono::Utc::now().timestamp();

    let tournament_id = if let Some(id) = existing_id {
        conn.execute(
            "UPDATE tournaments
             SET duration_days = ?, city = ?, country = ?, country_code = ?, status = 'pending',
                 start_date = ?, end_date = ?, ranking_id = ?, location = ?, contact = ?, oc = ?, officials = ?, banner = ?, updated_at = ?, updated = ?
             WHERE id = ?",
            params![
                2,
                VENUE_CITY,
                VENUE_COUNTRY,
                VENUE_COUNTRY_CODE,
                start_rfc3339,
                end_rfc3339,
                ranking_id,
                location_json,
                contact_json,
                oc_json,
                officials_json,
                banner_base64,
                now_iso,
                now_unix,
                id
            ],
        )?;
        id
    } else {
        let uuid = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO tournaments (
                uuid,
                name,
                duration_days,
                city,
                country,
                country_code,
                logo_path,
                status,
                start_date,
                end_date,
                ranking_id,
                location,
                contact,
                oc,
                officials,
                banner,
                created_at,
                updated_at,
                created,
                updated
            )
            VALUES (
                ?1, ?2, 2, ?3, ?4, ?5, NULL, 'pending', ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14, ?15, ?15
            )",
            params![
                uuid,
                TOURNAMENT_NAME,
                VENUE_CITY,
                VENUE_COUNTRY,
                VENUE_COUNTRY_CODE,
                start_rfc3339,
                end_rfc3339,
                ranking_id,
                location_json,
                contact_json,
                oc_json,
                officials_json,
                banner_base64,
                now_iso,
                now_unix
            ],
        )?;
        conn.last_insert_rowid()
    };

    seed_tournament_days(&mut conn, tournament_id)?;
    seed_octagons(&mut conn, tournament_id)?;

    println!("Tournament '{TOURNAMENT_NAME}' seeded/updated successfully (id = {tournament_id})");
    println!("Database path: {}", db_path.display());
    Ok(())
}

fn read_and_encode_banner(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        log::warn!("Banner image not found at {}", path.display());
        return Ok(None);
    }
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
    if metadata.len() > 2 * 1024 * 1024 {
        log::warn!(
            "Banner image exceeds 2 MB ({} bytes); skipping banner attachment",
            metadata.len()
        );
        return Ok(None);
    }
    let data = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    Ok(Some(STANDARD.encode(data)))
}

fn seed_tournament_days(conn: &mut Connection, tournament_id: i64) -> Result<()> {
    let start_date = NaiveDate::parse_from_str(START_DATE, "%Y-%m-%d")?;
    let end_date = NaiveDate::parse_from_str(END_DATE, "%Y-%m-%d")?;
    let mut day_number = 1;
    let mut date = start_date;

    while date <= end_date {
        let date_iso = format!("{}T00:00:00Z", date.format("%Y-%m-%d"));
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM tournament_days WHERE tournament_id = ?1 AND date = ?2",
                params![tournament_id, date_iso],
                |row| row.get(0),
            )
            .optional()?;

        if existing.is_none() {
            conn.execute(
                "INSERT INTO tournament_days (
                    uuid,
                    tournament_id,
                    day_number,
                    date,
                    status,
                    start_time,
                    end_time,
                    created_at,
                    updated_at,
                    created,
                    updated
                )
                VALUES (
                    ?1, ?2, ?3, ?4, 'pending', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, strftime('%s','now'), strftime('%s','now')
                )",
                params![Uuid::new_v4().to_string(), tournament_id, day_number, date_iso],
            )?;
        }

        day_number += 1;
        date += Duration::days(1);
    }
    Ok(())
}

fn seed_octagons(conn: &mut Connection, tournament_id: i64) -> Result<()> {
    let day_rows = {
        let mut stmt = conn.prepare(
            "SELECT id, date
             FROM tournament_days
             WHERE tournament_id = ?
             ORDER BY day_number",
        )?;
        let rows = stmt.query_map(params![tournament_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut data = Vec::new();
        for row in rows {
            data.push(row?);
        }
        data
    };

    for (day_id, _date) in day_rows {
        // Ensure octagons exist for this day
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM octagons WHERE tournament_day_id = ?1",
            params![day_id],
            |row| row.get(0),
        )?;
        if count >= COURT_COUNT as i64 {
            continue;
        }

        let tx = conn.transaction()?;
        for idx in 1..=COURT_COUNT {
            let label = format!("{OCTAGON_LABEL_PREFIX}{idx:02}");
            tx.execute(
                "INSERT OR IGNORE INTO octagons (
                    tournament_id,
                    tournament_day_id,
                    octagon_number,
                    created_at,
                    updated_at
                )
                VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                params![tournament_id, day_id, label],
            )?;
        }
        tx.commit()?;
    }

    Ok(())
}
