//! Daedo GO2025 tournament importer.
//!
//! This module re-implements the logic that previously lived in the Python
//! helper scripts under `scripts/tournament`.  The goal is to expose the
//! importer as a first-class backend command that can be invoked from the UI.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use log::{info, warn};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Transaction};
use serde::Serialize;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::database::DatabaseError;

/// Outcome returned by [`import_tournament`].
#[derive(Debug, Clone, Serialize)]
pub struct TournamentImportStats {
    pub tournament_id: i64,
    pub day_count: usize,
    pub octagon_count: usize,
    pub athletes_upserted: usize,
    pub matches_inserted: usize,
    pub matches_updated: usize,
    pub events_inserted: usize,
}

/// Errors emitted by the importer.
#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error("archive path does not exist: {0}")]
    MissingArchive(PathBuf),
    #[error("archive path is not a directory: {0}")]
    InvalidArchive(PathBuf),
    #[error("archive does not contain any match logs")]
    EmptyArchive,
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

/// Parameters used by the importer.
#[derive(Debug)]
pub struct ImportRequest {
    pub archive_root: PathBuf,
    pub tournament_name: String,
}

/// Public entry point used by the Tauri command.
pub fn import_tournament(
    db_path: &Path,
    request: ImportRequest,
) -> Result<TournamentImportStats, ImportError> {
    if !request.archive_root.exists() {
        return Err(ImportError::MissingArchive(request.archive_root.clone()));
    }
    if !request.archive_root.is_dir() {
        return Err(ImportError::InvalidArchive(request.archive_root.clone()));
    }

    info!(
        "Starting Daedo GO2025 import from {}",
        request.archive_root.display()
    );

    let summary = collect_archive_summary(&request.archive_root)?;
    if summary.matches.is_empty() {
        return Err(ImportError::EmptyArchive);
    }

    let mut conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;

    let stats = import_into_database(&mut conn, &request, &summary)?;

    info!(
        "Import complete (tournament_id={}, matches_inserted={}, matches_updated={}, events_inserted={})",
        stats.tournament_id, stats.matches_inserted, stats.matches_updated, stats.events_inserted
    );

    Ok(stats)
}

// -------------------------------------------------------------------------------------------------
// Archive summary
// -------------------------------------------------------------------------------------------------

/// Aggregated information derived from a Daedo archive.
struct ArchiveSummary {
    matches: Vec<MatchRecord>,
    day_courts: BTreeMap<String, BTreeSet<String>>,
    athletes: HashMap<AthleteKey, AthleteProfile>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
}

#[allow(dead_code)]
struct MatchRecord {
    match_id: String,
    day_folder: String,
    court: String,
    metadata: HashMap<String, String>,
    athletes: HashMap<String, AthleteStub>,
    events: Vec<CanonicalEvent>,
    round_snapshots: Vec<RoundSnapshot>,
    final_snapshot: FinalSnapshot,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AthleteStub {
    corner: String,
    display_name: String,
    wtid: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CanonicalEvent {
    sequence: usize,
    event_type: String,
    event_time_iso: Option<String>,
    round_number: Option<i64>,
    blue_total_penalties: Option<i64>,
    red_total_penalties: Option<i64>,
    score_snapshot: (i64, i64),
    raw_row: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct RoundSnapshot {
    round_number: i64,
    end_iso: Option<String>,
    blue_score: Option<i64>,
    red_score: Option<i64>,
    blue_penalties: Option<i64>,
    red_penalties: Option<i64>,
}

#[derive(Debug, Clone)]
struct FinalSnapshot {
    blue_score: i64,
    red_score: i64,
    blue_penalties: i64,
    red_penalties: i64,
    timestamp_iso: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AthleteKey {
    wtid: Option<String>,
    display_name: String,
    ioc_code: String,
    gender: Option<String>,
}

#[derive(Debug, Clone)]
struct AthleteProfile {
    key: AthleteKey,
    first_name: Option<String>,
    last_name: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    gender: Option<String>,
    age_group: Option<String>,
    division: Option<String>,
    weight_class: Option<String>,
}

/// Walk the archive directory, parse every match bundle and build a summary
/// that can be replayed against the database.
fn collect_archive_summary(root: &Path) -> Result<ArchiveSummary, ImportError> {
    let mut matches = Vec::new();
    let mut day_courts: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut athletes: HashMap<AthleteKey, AthleteProfile> = HashMap::new();
    let mut start_date: Option<NaiveDate> = None;
    let mut end_date: Option<NaiveDate> = None;

    for entry in walk_match_logs(root)? {
        let (record, bundle) = parse_match_bundle(&entry)
            .map_err(|e| ImportError::Parse(format!("{}: {}", entry.display(), e)))?;

        if let Some(day) = bundle.day_folder {
            day_courts.entry(day.clone()).or_default().insert(bundle.court.clone());
        }

        start_date = reduce_min_date(start_date, bundle.match_start);
        end_date = reduce_max_date(end_date, bundle.match_end);

        for profile in bundle.athletes {
            athletes.entry(profile.key.clone()).or_insert(profile);
        }

        matches.push(record);
    }

    Ok(ArchiveSummary {
        matches,
        day_courts,
        athletes,
        start_date,
        end_date,
    })
}

struct BundleSummary {
    match_start: Option<NaiveDate>,
    match_end: Option<NaiveDate>,
    day_folder: Option<String>,
    court: String,
    athletes: Vec<AthleteProfile>,
}

fn reduce_min_date(current: Option<NaiveDate>, candidate: Option<NaiveDate>) -> Option<NaiveDate> {
    match (current, candidate) {
        (None, Some(d)) => Some(d),
        (Some(existing), Some(d)) if d < existing => Some(d),
        (Some(existing), _) => Some(existing),
        _ => None,
    }
}

fn reduce_max_date(current: Option<NaiveDate>, candidate: Option<NaiveDate>) -> Option<NaiveDate> {
    match (current, candidate) {
        (None, Some(d)) => Some(d),
        (Some(existing), Some(d)) if d > existing => Some(d),
        (Some(existing), _) => Some(existing),
        _ => None,
    }
}

// -------------------------------------------------------------------------------------------------
// Database import (skeleton implementations – detailed logic appended later)
// -------------------------------------------------------------------------------------------------

fn import_into_database(
    conn: &mut Connection,
    request: &ImportRequest,
    summary: &ArchiveSummary,
) -> Result<TournamentImportStats, ImportError> {
    let mut tx = conn.transaction()?;

    let (tournament_id, tournament_uuid) = ensure_tournament(&mut tx, request, summary)?;
    let day_mapping = ensure_tournament_days(&mut tx, tournament_id, &summary.day_courts)?;
    let octagon_count = ensure_octagons(&mut tx, tournament_id, &day_mapping, &summary.day_courts)?;
    let athletes_upserted = ensure_athletes(&mut tx, &summary.athletes)?;

    let mut matches_inserted = 0;
    let mut matches_updated = 0;
    let mut events_inserted = 0;

    for record in &summary.matches {
        let outcome = upsert_match(&mut tx, tournament_id, &tournament_uuid, record)?;
        if outcome.inserted {
            matches_inserted += 1;
        } else {
            matches_updated += 1;
        }
        events_inserted += outcome.events_inserted;
    }

    tx.commit()?;

    Ok(TournamentImportStats {
        tournament_id,
        day_count: day_mapping.len(),
        octagon_count,
        athletes_upserted,
        matches_inserted,
        matches_updated,
        events_inserted,
    })
}

#[allow(dead_code)]
struct TournamentDayMeta {
    id: i64,
    uuid: String,
}

/// Ensure a tournament row exists (updating if it already exists with the
/// same name) and return both the numeric id and the UUID.
fn ensure_tournament(
    tx: &mut Transaction<'_>,
    request: &ImportRequest,
    summary: &ArchiveSummary,
) -> Result<(i64, String), ImportError> {
    let start_date = summary
        .start_date
        .map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()).to_rfc3339());
    let end_date = summary
        .end_date
        .map(|d| Utc.from_utc_datetime(&d.and_hms_opt(23, 59, 59).unwrap()).to_rfc3339());

    let duration_days = summary.day_courts.len().max(1) as i32;
    let default_city = "Unknown City".to_string();
    let default_country = "Unknown Country".to_string();

    let existing = tx
        .query_row(
            "SELECT id, uuid FROM tournaments WHERE name = ?1 ORDER BY created_at DESC LIMIT 1",
            params![request.tournament_name],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()?;

    let (id, uuid) = if let Some((id, uuid)) = existing {
        let uuid = uuid.unwrap_or_else(|| Uuid::new_v4().to_string());
        tx.execute(
            r#"
            UPDATE tournaments
            SET duration_days = ?2,
                city = COALESCE(city, ?3),
                country = COALESCE(country, ?4),
                status = 'ended',
                start_date = ?5,
                end_date = ?6,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            "#,
            params![id, duration_days, default_city, default_country, start_date, end_date],
        )?;
        (id, uuid)
    } else {
        let uuid = Uuid::new_v4().to_string();
        tx.execute(
            r#"
            INSERT INTO tournaments (
                uuid, name, duration_days, city, country, status,
                start_date, end_date, ranking_id, location, contact, oc, officials, banner,
                created_at, updated_at, created, updated
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, 'ended',
                ?6, ?7, NULL, '{}', '{}', '{}', '{}', NULL,
                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, strftime('%s','now'), strftime('%s','now')
            )
            "#,
            params![
                uuid,
                request.tournament_name,
                duration_days,
                default_city,
                default_country,
                start_date,
                end_date
            ],
        )?;
        (tx.last_insert_rowid(), uuid)
    };

    Ok((id, uuid))
}

fn ensure_tournament_days(
    tx: &mut Transaction<'_>,
    tournament_id: i64,
    day_courts: &BTreeMap<String, BTreeSet<String>>,
) -> Result<HashMap<String, TournamentDayMeta>, ImportError> {
    let mut mapping = HashMap::new();

    for (idx, (day_folder, _)) in day_courts.iter().enumerate() {
        let day_number = (idx + 1) as i32;
        let date_value = match NaiveDate::parse_from_str(day_folder, "%Y%m%d") {
            Ok(date) => date,
            Err(_) => {
                warn!("Skipping day folder with unexpected format: {}", day_folder);
                continue;
            }
        };
        let date_iso = date_value.to_string();

        let existing = tx
            .query_row(
                "SELECT id, uuid FROM tournament_days WHERE tournament_id = ?1 AND date = ?2",
                params![tournament_id, date_iso],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()?;

        let meta = if let Some((id, uuid)) = existing {
            tx.execute(
                "UPDATE tournament_days SET day_number = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                params![day_number, id],
            )?;
            TournamentDayMeta {
                id,
                uuid: uuid.unwrap_or_else(|| Uuid::new_v4().to_string()),
            }
        } else {
            let uuid = Uuid::new_v4().to_string();
            tx.execute(
                r#"
                INSERT INTO tournament_days (
                    uuid, tournament_id, day_number, date, status, created_at, updated_at, created, updated
                ) VALUES (?1, ?2, ?3, ?4, 'ended', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, strftime('%s','now'), strftime('%s','now'))
                "#,
                params![uuid, tournament_id, day_number, date_iso],
            )?;
            TournamentDayMeta {
                id: tx.last_insert_rowid(),
                uuid,
            }
        };

        mapping.insert(day_folder.clone(), meta);
    }

    Ok(mapping)
}

fn ensure_octagons(
    tx: &mut Transaction<'_>,
    tournament_id: i64,
    day_mapping: &HashMap<String, TournamentDayMeta>,
    day_courts: &BTreeMap<String, BTreeSet<String>>,
) -> Result<usize, ImportError> {
    let mut inserted = 0;

    for (day_folder, courts) in day_courts {
        let Some(day_meta) = day_mapping.get(day_folder) else {
            continue;
        };
        for court in courts {
            let exists: Option<i64> = tx
                .query_row(
                    "SELECT id FROM octagons WHERE tournament_day_id = ?1 AND octagon_number = ?2",
                    params![day_meta.id, court],
                    |row| row.get(0),
                )
                .optional()?;
            if exists.is_some() {
                tx.execute(
                    "UPDATE octagons SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                    params![exists.unwrap()],
                )?;
                continue;
            }
            tx.execute(
                r#"
                INSERT INTO octagons (
                    tournament_id, tournament_day_id, octagon_number, created_at, updated_at, created, updated
                ) VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, strftime('%s','now'), strftime('%s','now'))
                "#,
                params![tournament_id, day_meta.id, court],
            )?;
            inserted += 1;
        }
    }

    Ok(inserted)
}

fn ensure_athletes(
    tx: &mut Transaction<'_>,
    athletes: &HashMap<AthleteKey, AthleteProfile>,
) -> Result<usize, ImportError> {
    if athletes.is_empty() {
        return Ok(0);
    }

    let lookups = LookupTables::load(tx)?;
    let mut processed = 0;

    for profile in athletes.values() {
        upsert_athlete(tx, profile, &lookups)?;
        processed += 1;
    }

    Ok(processed)
}

struct MatchImportOutcome {
    inserted: bool,
    events_inserted: usize,
}

fn upsert_match(
    tx: &mut Transaction<'_>,
    _tournament_id: i64,
    tournament_uuid: &str,
    record: &MatchRecord,
) -> Result<MatchImportOutcome, ImportError> {
    let existing = tx
        .query_row(
            "SELECT id, uuid FROM pss_matches WHERE match_id = ?1",
            params![record.match_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()?;

    let now = Utc::now().to_rfc3339();
    let match_number = record.metadata.get("matchNumber").cloned();
    let category = record.metadata.get("categoryName").cloned();
    let division = record.metadata.get("subCategoryName").cloned();
    let round_duration = record
        .metadata
        .get("roundsConfig.roundTimeSeconds")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(120);

    let total_rounds = if record.round_snapshots.is_empty() {
        3
    } else {
        record.round_snapshots.len() as i64
    };

    let (match_id, match_uuid, inserted) = if let Some((id, uuid)) = existing {
        let uuid = uuid.unwrap_or_else(|| Uuid::new_v4().to_string());
        tx.execute(
            r#"
            UPDATE pss_matches
            SET tournament_id = ?2,
                match_number = ?3,
                category = ?4,
                weight_class = ?5,
                division = ?6,
                total_rounds = ?7,
                round_duration = ?8,
                countdown_type = 'cntDown',
                format_type = NULL,
                creation_mode = 'replay',
                updated_at = ?9
            WHERE id = ?1
            "#,
            params![
                id,
                tournament_uuid,
                match_number,
                category.clone(),
                category.clone(),
                division.clone(),
                total_rounds,
                round_duration,
                now,
            ],
        )?;
        (id, uuid, false)
    } else {
        let uuid = Uuid::new_v4().to_string();
        tx.execute(
            r#"
            INSERT INTO pss_matches (
                uuid, tournament_id, match_id, match_number, category, weight_class, division,
                total_rounds, round_duration, countdown_type, format_type, creation_mode,
                created_at, updated_at, created, updated
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                ?8, ?9, 'cntDown', NULL, 'replay',
                ?10, ?10, strftime('%s','now'), strftime('%s','now')
            )
            "#,
            params![
                uuid,
                tournament_uuid,
                record.match_id,
                match_number,
                category.clone(),
                category.clone(),
                division.clone(),
                total_rounds,
                round_duration,
                now,
            ],
        )?;
        (tx.last_insert_rowid(), uuid, true)
    };

    store_match_athletes(tx, &match_uuid, &record.athletes)?;
    store_rounds(tx, &match_uuid, &record.round_snapshots)?;
    store_scores(tx, &match_uuid, tournament_uuid, record)?;
    store_warnings(tx, &match_uuid, tournament_uuid, record)?;
    let events_inserted = store_events(tx, match_id, tournament_uuid, record)?;

    Ok(MatchImportOutcome {
        inserted,
        events_inserted,
    })
}

fn store_match_athletes(
    tx: &Transaction<'_>,
    match_uuid: &str,
    athletes: &HashMap<String, AthleteStub>,
) -> Result<(), ImportError> {
    tx.execute("DELETE FROM pss_match_athletes WHERE match_id = ?", params![match_uuid])?;

    let mut rows = Vec::new();
    if let Some(blue) = athletes.get("blue") {
        if let Some(id) = find_athlete_id(tx, blue)? {
            rows.push((match_uuid.to_string(), id, 1, "#0000ff", "#ffffff"));
        }
    }
    if let Some(red) = athletes.get("red") {
        if let Some(id) = find_athlete_id(tx, red)? {
            rows.push((match_uuid.to_string(), id, 2, "#ff0000", "#ffffff"));
        }
    }

    if rows.is_empty() {
        return Ok(());
    }

    let mut stmt = tx.prepare(
        r#"
        INSERT INTO pss_match_athletes (
            match_id, athlete_id, athlete_position, bg_color, fg_color, created_at, created
        ) VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP, strftime('%s','now'))
        "#,
    )?;
    for row in rows {
        stmt.execute(params![row.0, row.1, row.2, row.3, row.4])?;
    }
    Ok(())
}

fn find_athlete_id(tx: &Transaction<'_>, stub: &AthleteStub) -> Result<Option<i64>, ImportError> {
    if let Some(wtid) = &stub.wtid {
        let existing = tx
            .query_row(
                "SELECT id FROM athletes WHERE wtid = ?1",
                params![wtid],
                |row| row.get(0),
            )
            .optional()?;
        if existing.is_some() {
            return Ok(existing);
        }
    }

    tx
        .query_row(
            "SELECT id FROM athletes WHERE display_name = ?1",
            params![stub.display_name],
            |row| row.get(0),
        )
        .optional()
        .map_err(ImportError::from)
}

fn store_rounds(
    tx: &Transaction<'_>,
    match_uuid: &str,
    rounds: &[RoundSnapshot],
) -> Result<(), ImportError> {
    tx.execute("DELETE FROM pss_rounds WHERE match_id = ?", params![match_uuid])?;
    let mut stmt = tx.prepare(
        r#"
        INSERT INTO pss_rounds (
            match_id, round_number, winner_position, blue_score, red_score, created_at, created
        ) VALUES (?1, ?2, NULL, ?3, ?4, CURRENT_TIMESTAMP, strftime('%s','now'))
        "#,
    )?;
    for snapshot in rounds {
        stmt.execute(params![
            match_uuid,
            snapshot.round_number,
            snapshot.blue_score,
            snapshot.red_score,
        ])?;
    }
    Ok(())
}

fn store_scores(
    tx: &Transaction<'_>,
    match_uuid: &str,
    tournament_uuid: &str,
    record: &MatchRecord,
) -> Result<(), ImportError> {
    tx.execute("DELETE FROM pss_scores WHERE match_id = ?", params![match_uuid])?;

    let mut stmt = tx.prepare(
        r#"
        INSERT INTO pss_scores (
            match_id, round_id, athlete_position, score_type, score_value, timestamp, tournament_id, created_at, created
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP, strftime('%s','now'))
        "#,
    )?;

    for snapshot in &record.round_snapshots {
        let ts = snapshot
            .end_iso
            .clone()
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        if let Some(score) = snapshot.blue_score {
            stmt.execute(params![match_uuid, snapshot.round_number, 1, "round", score, ts, tournament_uuid])?;
        }
        if let Some(score) = snapshot.red_score {
            stmt.execute(params![match_uuid, snapshot.round_number, 2, "round", score, ts, tournament_uuid])?;
        }
    }

    let final_ts = record
        .final_snapshot
        .timestamp_iso
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());

    stmt.execute(params![
        match_uuid,
        Option::<i64>::None,
        1,
        "total",
        record.final_snapshot.blue_score,
        final_ts.clone(),
        tournament_uuid,
    ])?;
    stmt.execute(params![
        match_uuid,
        Option::<i64>::None,
        2,
        "total",
        record.final_snapshot.red_score,
        final_ts.clone(),
        tournament_uuid,
    ])?;

    Ok(())
}

fn store_warnings(
    tx: &Transaction<'_>,
    match_uuid: &str,
    tournament_uuid: &str,
    record: &MatchRecord,
) -> Result<(), ImportError> {
    tx.execute("DELETE FROM pss_warnings WHERE match_id = ?", params![match_uuid])?;

    let mut stmt = tx.prepare(
        r#"
        INSERT INTO pss_warnings (
            match_id, round_id, athlete_position, warning_type, warning_count, timestamp, tournament_id, created_at, created
        ) VALUES (?1, ?2, ?3, 'gam_jeom', ?4, ?5, ?6, CURRENT_TIMESTAMP, strftime('%s','now'))
        "#,
    )?;

    for snapshot in &record.round_snapshots {
        let ts = snapshot
            .end_iso
            .clone()
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        if let Some(w) = snapshot.blue_penalties {
            stmt.execute(params![match_uuid, snapshot.round_number, 1, w, ts.clone(), tournament_uuid])?;
        }
        if let Some(w) = snapshot.red_penalties {
            stmt.execute(params![match_uuid, snapshot.round_number, 2, w, ts.clone(), tournament_uuid])?;
        }
    }

    let final_ts = record
        .final_snapshot
        .timestamp_iso
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    stmt.execute(params![
        match_uuid,
        Option::<i64>::None,
        1,
        record.final_snapshot.blue_penalties,
        final_ts.clone(),
        tournament_uuid,
    ])?;
    stmt.execute(params![
        match_uuid,
        Option::<i64>::None,
        2,
        record.final_snapshot.red_penalties,
        final_ts.clone(),
        tournament_uuid,
    ])?;

    Ok(())
}

fn store_events(
    tx: &Transaction<'_>,
    match_id: i64,
    tournament_uuid: &str,
    record: &MatchRecord,
) -> Result<usize, ImportError> {
    let existing: Vec<i64> = tx
        .prepare("SELECT id FROM pss_events WHERE match_id = ?1")?
        .query_map(params![match_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for event_id in existing {
        tx.execute("DELETE FROM pss_event_details WHERE event_id = ?", params![event_id])?;
    }
    tx.execute("DELETE FROM pss_events WHERE match_id = ?", params![match_id])?;

    let mut inserted = 0;
    for event in &record.events {
        let event_type_id = ensure_event_type(tx, &event.event_type)?;
        let timestamp = event
            .event_time_iso
            .clone()
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        let raw_payload = serde_json::to_string(&event.raw_row)
            .map_err(|e| ImportError::Parse(e.to_string()))?;

        tx.execute(
            r#"
            INSERT INTO pss_events (
                session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                recognition_status, protocol_version, parser_confidence, validation_errors,
                tournament_id, created_at, created
            ) VALUES (
                1, ?1, NULL, ?2, ?3, ?4,
                ?5, ?6, NULL, 1, NULL,
                'reconstructed', '2.3', NULL, NULL,
                ?7, CURRENT_TIMESTAMP, strftime('%s','now')
            )
            "#,
            params![
                match_id,
                event_type_id,
                timestamp,
                raw_payload.clone(),
                raw_payload,
                event.sequence as i64,
                tournament_uuid,
            ],
        )?;
        inserted += 1;
    }

    Ok(inserted)
}

fn ensure_event_type(tx: &Transaction<'_>, event_code: &str) -> Result<i64, ImportError> {
    let existing = tx
        .query_row(
            "SELECT id FROM pss_event_types WHERE event_code = ?1",
            params![event_code],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }

    tx.execute(
        r#"
        INSERT INTO pss_event_types (
            event_code, event_name, description, category, is_active, created_at, created
        ) VALUES (
            ?1, ?2, ?3, 'replay', 1, CURRENT_TIMESTAMP, strftime('%s','now')
        )
        "#,
        params![
            event_code,
            event_code.replace('_', " "),
            "Reconstructed from Daedo GO2025 logs",
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

// -------------------------------------------------------------------------------------------------
// Placeholder parsing helpers – the full implementations are appended later.
// -------------------------------------------------------------------------------------------------

fn walk_match_logs(root: &Path) -> Result<Vec<PathBuf>, ImportError> {
    let mut results = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if !name.ends_with("-matchlog.csv") {
            continue;
        }
        if name.contains("-test") {
            continue;
        }
        results.push(entry.into_path());
    }
    results.sort();
    Ok(results)
}

/// Parse a `*-matchLog.csv` / `*-matchLogItems.csv` pair into the canonical
/// structures used by the importer.
fn parse_match_bundle(path: &Path) -> Result<(MatchRecord, BundleSummary), anyhow::Error> {
    let match_row = parse_match_log(path)?;
    let items_path = path.with_file_name(
        path.file_name()
            .ok_or_else(|| anyhow!("invalid match log path"))?
            .to_string_lossy()
            .replace("-matchLog.csv", "-matchLogItems.csv"),
    );
    let item_rows = parse_match_log_items(&items_path)?;

    let day_folder = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string());
    let court = path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown-court".to_string());

    let match_id = match_row
        .get("matchNumber")
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    let (events, round_snapshots, final_snapshot) = build_events(&item_rows);

    let metadata = match_row.clone();

    let (match_start, match_end) = (
        metadata
            .get("matchStartTime")
            .and_then(|v| parse_match_datetime(v).map(|dt| dt.date())),
        metadata
            .get("matchEndTime")
            .and_then(|v| parse_match_datetime(v).map(|dt| dt.date())),
    );

    let (athlete_profiles, athlete_stubs) = build_athletes(&match_row);

    let record = MatchRecord {
        match_id: match_id.clone(),
        day_folder: day_folder.clone().unwrap_or_else(|| "unknown-day".to_string()),
        court: court.clone(),
        metadata,
        athletes: athlete_stubs,
        events,
        round_snapshots,
        final_snapshot: final_snapshot.clone(),
    };

    Ok((
        record,
        BundleSummary {
            match_start,
            match_end,
            day_folder,
            court,
            athletes: athlete_profiles,
        },
    ))
}

fn parse_match_log(path: &Path) -> Result<HashMap<String, String>, anyhow::Error> {
    let file = fs::File::open(path)?;
    let mut lines = BufReader::new(file).lines();
    let header_line = lines
        .next()
        .ok_or_else(|| anyhow!("missing header row in {}", path.display()))??;
    let headers: Vec<String> = header_line.split(',').map(|s| s.to_string()).collect();
    let mut records = Vec::new();
    for raw in lines {
        let raw = raw?;
        if raw.trim().is_empty() {
            continue;
        }
        let values = split_match_log_line(&raw);
        if values.len() != headers.len() {
            return Err(anyhow!(
                "header/row mismatch in {} ({} vs {})",
                path.display(),
                headers.len(),
                values.len()
            ));
        }
        let mut map = HashMap::new();
        for (idx, header) in headers.iter().enumerate() {
            map.insert(header.clone(), values[idx].clone());
        }
        records.push(map);
    }
    records
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no data rows in {}", path.display()))
}

fn parse_match_log_items(path: &Path) -> Result<Vec<HashMap<String, String>>, anyhow::Error> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    let headers = reader.headers()?.clone();
    let headers = ensure_unique_headers(&headers);
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        if record.iter().all(|cell| cell.trim().is_empty()) {
            continue;
        }
        let mut map = HashMap::new();
        for (idx, header) in headers.iter().enumerate() {
            map.insert(header.clone(), record.get(idx).unwrap_or("").to_string());
        }
        rows.push(map);
    }
    Ok(rows)
}

fn ensure_unique_headers(headers: &csv::StringRecord) -> Vec<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut result = Vec::with_capacity(headers.len());
    for header in headers.iter() {
        let count = seen.entry(header.to_string()).or_insert(0);
        if *count == 0 {
            result.push(header.to_string());
        } else {
            result.push(format!("{}_{}", header, count));
        }
        *count += 1;
    }
    result
}

/// Convert raw match log item rows into canonical events and derive round
/// snapshots / final scoreboard information.
fn build_events(
    item_rows: &[HashMap<String, String>],
) -> (Vec<CanonicalEvent>, Vec<RoundSnapshot>, FinalSnapshot) {
    let mut events = Vec::new();
    let mut round_map: HashMap<i64, RoundSnapshot> = HashMap::new();
    let mut final_snapshot = FinalSnapshot {
        blue_score: 0,
        red_score: 0,
        blue_penalties: 0,
        red_penalties: 0,
        timestamp_iso: None,
    };

    for (idx, row) in item_rows.iter().enumerate() {
        let event_type = row
            .get("matchLogItemType")
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string());

        let round_number = row
            .get("roundNumber")
            .and_then(|v| v.parse::<i64>().ok());

        let score_snapshot = parse_score_snapshot(row);
        let blue_total_penalties = row
            .get("blueTotalPenalties")
            .and_then(|v| v.parse::<i64>().ok());
        let red_total_penalties = row
            .get("redTotalPenalties")
            .and_then(|v| v.parse::<i64>().ok());

        let event = CanonicalEvent {
            sequence: idx + 1,
            event_type: event_type.clone(),
            event_time_iso: row
                .get("eventTimeMs")
                .and_then(|v| ms_to_iso8601(v)),
            round_number,
            blue_total_penalties,
            red_total_penalties,
            score_snapshot,
            raw_row: row.clone(),
        };

        if let Some(round) = round_number {
            let snapshot = round_map.entry(round).or_insert(RoundSnapshot {
                round_number: round,
                end_iso: None,
                blue_score: None,
                red_score: None,
                blue_penalties: None,
                red_penalties: None,
            });
            snapshot.end_iso = event.event_time_iso.clone();
            snapshot.blue_score = Some(score_snapshot.0);
            snapshot.red_score = Some(score_snapshot.1);
            snapshot.blue_penalties = blue_total_penalties;
            snapshot.red_penalties = red_total_penalties;
        }

        final_snapshot.blue_score = score_snapshot.0;
        final_snapshot.red_score = score_snapshot.1;
        if let Some(bp) = blue_total_penalties {
            final_snapshot.blue_penalties = bp;
        }
        if let Some(rp) = red_total_penalties {
            final_snapshot.red_penalties = rp;
        }
        if let Some(ts) = &event.event_time_iso {
            final_snapshot.timestamp_iso = Some(ts.clone());
        }

        events.push(event);
    }

    let mut round_snapshots: Vec<RoundSnapshot> = round_map.into_values().collect();
    round_snapshots.sort_by_key(|s| s.round_number);

    (events, round_snapshots, final_snapshot)
}

fn build_athletes(
    match_row: &HashMap<String, String>,
) -> (Vec<AthleteProfile>, HashMap<String, AthleteStub>) {
    let mut profiles = Vec::new();
    let mut stubs = HashMap::new();

    let blue_profile = AthleteProfile {
        key: AthleteKey {
            wtid: match_row
                .get("blueAthleteWtfId")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            display_name: match_row
                .get("blueAthleteName")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Blue Athlete".to_string()),
            ioc_code: match_row
                .get("blueAthleteFlagAbbreviation")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "UNK".to_string()),
            gender: match_row
                .get("categoryGender")
                .cloned()
                .or_else(|| match_row.get("gender").cloned()),
        },
        first_name: None,
        last_name: None,
        country: match_row
            .get("blueAthleteFlagName")
            .cloned()
            .filter(|s| !s.trim().is_empty()),
        country_code: match_row
            .get("blueAthleteFlagAbbreviation")
            .cloned()
            .filter(|s| !s.trim().is_empty()),
        gender: match_row
            .get("categoryGender")
            .cloned()
            .or_else(|| match_row.get("gender").cloned()),
        age_group: match_row.get("subCategoryName").cloned(),
        division: match_row.get("categoryName").cloned(),
        weight_class: match_row.get("categoryName").cloned(),
    };

    stubs.insert(
        "blue".to_string(),
        AthleteStub {
            corner: "blue".to_string(),
            display_name: blue_profile.key.display_name.clone(),
            wtid: blue_profile.key.wtid.clone(),
        },
    );

    let red_profile = AthleteProfile {
        key: AthleteKey {
            wtid: match_row
                .get("redAthleteWtfId")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            display_name: match_row
                .get("redAthleteName")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Red Athlete".to_string()),
            ioc_code: match_row
                .get("redAthleteFlagAbbreviation")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "UNK".to_string()),
            gender: match_row
            .get("categoryGender")
            .cloned()
            .or_else(|| match_row.get("gender").cloned()),
        },
        first_name: None,
        last_name: None,
        country: match_row
            .get("redAthleteFlagName")
            .cloned()
            .filter(|s| !s.trim().is_empty()),
        country_code: match_row
            .get("redAthleteFlagAbbreviation")
            .cloned()
            .filter(|s| !s.trim().is_empty()),
        gender: match_row
            .get("categoryGender")
            .cloned()
            .or_else(|| match_row.get("gender").cloned()),
        age_group: match_row.get("subCategoryName").cloned(),
        division: match_row.get("categoryName").cloned(),
        weight_class: match_row.get("categoryName").cloned(),
    };

    profiles.push(blue_profile);
    profiles.push(red_profile);
    (profiles, stubs)
}

fn parse_score_snapshot(row: &HashMap<String, String>) -> (i64, i64) {
    if let Some(raw) = row.get("score") {
        let parts: Vec<&str> = raw.split('-').collect();
        if parts.len() == 2 {
            let blue = parts[0].trim().parse::<i64>().unwrap_or(0);
            let red = parts[1].trim().parse::<i64>().unwrap_or(0);
            return (blue, red);
        }
    }
    (0, 0)
}

fn ms_to_iso8601(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let millis = value.parse::<i64>().ok()?;
    let ts = Utc.timestamp_millis_opt(millis).single()?;
    Some(ts.to_rfc3339())
}

fn parse_match_datetime(value: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, "%d/%m/%Y %H:%M:%S:%f").ok()
}

struct LookupTables {
    gender: HashMap<String, i64>,
    age_group: HashMap<String, i64>,
    division: HashMap<String, i64>,
    weight_class: HashMap<String, i64>,
}

impl LookupTables {
    fn load(tx: &Transaction<'_>) -> Result<Self, ImportError> {
        Ok(Self {
            gender: fetch_lookup(tx, "look_genders")?,
            age_group: fetch_lookup(tx, "look_age_groups")?,
            division: fetch_lookup(tx, "look_divisions")?,
            weight_class: fetch_lookup(tx, "look_weight_classes")?,
        })
    }
}

fn fetch_lookup(tx: &Transaction<'_>, table: &str) -> Result<HashMap<String, i64>, ImportError> {
    let mut stmt = tx.prepare(&format!("SELECT id, name FROM {}", table))?;
    let mut rows = stmt.query([])?;
    let mut map = HashMap::new();
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        map.insert(name.trim().to_uppercase(), id);
    }
    Ok(map)
}

fn upsert_athlete(
    tx: &Transaction<'_>,
    profile: &AthleteProfile,
    lookups: &LookupTables,
) -> Result<(), ImportError> {
    let gender_id = profile
        .gender
        .as_ref()
        .and_then(|g| lookups.gender.get(&normalize_gender(g)).copied());

    let age_id = profile
        .age_group
        .as_ref()
        .and_then(|value| lookups.age_group.get(&value.trim().to_uppercase()).copied());

    let division_id = profile
        .division
        .as_ref()
        .and_then(|value| lookups.division.get(&value.trim().to_uppercase()).copied());

    let weight_id = profile
        .weight_class
        .as_ref()
        .and_then(|value| lookups.weight_class.get(&value.trim().to_uppercase()).copied());

    let mut candidates: Vec<i64> = Vec::new();

    if let Some(wtid) = &profile.key.wtid {
        let existing = tx
            .query_row(
                "SELECT id FROM athletes WHERE wtid = ?1",
                params![wtid],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = existing {
            candidates.push(id);
        }
    }

    if candidates.is_empty() {
        if let (Some(first), Some(last)) = (&profile.first_name, &profile.last_name) {
            let existing = tx
                .query_row(
                    "SELECT id FROM athletes WHERE LOWER(COALESCE(first_name, '')) = LOWER(?1) AND LOWER(COALESCE(last_name, '')) = LOWER(?2) AND (?3 IS NULL OR look_gender_id = ?3)",
                    params![first.trim(), last.trim(), gender_id],
                    |row| row.get(0),
                )
                .optional()?;
            if let Some(id) = existing {
                candidates.push(id);
            }
        }
    }

    if candidates.is_empty() {
        let existing = tx
            .query_row(
                "SELECT id FROM athletes WHERE display_name = ?1 AND ioc_code = ?2",
                params![profile.key.display_name, profile.key.ioc_code],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = existing {
            candidates.push(id);
        }
    }

    if let Some(id) = candidates.first() {
        update_athlete(tx, *id, profile, gender_id, age_id, division_id, weight_id)?;
    } else {
        insert_athlete(tx, profile, gender_id, age_id, division_id, weight_id)?;
    }

    Ok(())
}

fn insert_athlete(
    tx: &Transaction<'_>,
    profile: &AthleteProfile,
    gender_id: Option<i64>,
    age_id: Option<i64>,
    division_id: Option<i64>,
    weight_id: Option<i64>,
) -> Result<(), ImportError> {
    tx.execute(
        r#"
        INSERT INTO athletes (
            wtid, first_name, last_name, display_name, history,
            country, country_code, ioc_code,
            look_gender_id, look_age_group_id, look_division_id, look_weight_class_id,
            created_at, updated_at, created, updated
        ) VALUES (
            ?1, ?2, ?3, ?4, '[]',
            ?5, ?6, ?7,
            ?8, ?9, ?10, ?11,
            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, strftime('%s','now'), strftime('%s','now')
        )
        "#,
        params![
            profile.key.wtid,
            profile.first_name,
            profile.last_name,
            profile.key.display_name,
            profile.country,
            profile.country_code,
            profile.key.ioc_code,
            gender_id,
            age_id,
            division_id,
            weight_id
        ],
    )?;
    Ok(())
}

fn update_athlete(
    tx: &Transaction<'_>,
    athlete_id: i64,
    profile: &AthleteProfile,
    gender_id: Option<i64>,
    age_id: Option<i64>,
    division_id: Option<i64>,
    weight_id: Option<i64>,
) -> Result<(), ImportError> {
    let mut stmt = tx.prepare(
        "SELECT country, country_code, look_age_group_id, look_division_id, look_weight_class_id, history FROM athletes WHERE id = ?1",
    )?;
    let row = stmt.query_row(params![athlete_id], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<i64>>(2)?,
            row.get::<_, Option<i64>>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, Option<String>>(5)?,
        ))
    })?;

    let mut history: Vec<serde_json::Value> = row
        .5
        .as_deref()
        .and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or_else(|| Vec::new());

    let mut changes = serde_json::Map::new();
    if profile.country != row.0 {
        changes.insert(
            "country".to_string(),
            serde_json::json!({"old": row.0, "new": profile.country}),
        );
    }
    if profile.country_code != row.1 {
        changes.insert(
            "country_code".to_string(),
            serde_json::json!({"old": row.1, "new": profile.country_code}),
        );
    }
    if age_id != row.2 {
        changes.insert(
            "age_group".to_string(),
            serde_json::json!({"old": row.2, "new": age_id}),
        );
    }
    if division_id != row.3 {
        changes.insert(
            "division".to_string(),
            serde_json::json!({"old": row.3, "new": division_id}),
        );
    }
    if weight_id != row.4 {
        changes.insert(
            "weight_class".to_string(),
            serde_json::json!({"old": row.4, "new": weight_id}),
        );
    }

    if !changes.is_empty() {
        history.push(serde_json::json!({
            "changed_at": Utc::now().to_rfc3339(),
            "source": "tournament_import",
            "changes": changes,
        }));
    }

    let history_json = serde_json::to_string(&history)
        .map_err(|e| ImportError::Parse(e.to_string()))?;

    tx.execute(
        r#"
        UPDATE athletes
        SET first_name = COALESCE(?2, first_name),
            last_name = COALESCE(?3, last_name),
            country = COALESCE(?4, country),
            country_code = COALESCE(?5, country_code),
            display_name = ?6,
            look_gender_id = COALESCE(?7, look_gender_id),
            look_age_group_id = COALESCE(?8, look_age_group_id),
            look_division_id = COALESCE(?9, look_division_id),
            look_weight_class_id = COALESCE(?10, look_weight_class_id),
            history = ?11,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        "#,
        params![
            athlete_id,
            profile.first_name,
            profile.last_name,
            profile.country,
            profile.country_code,
            profile.key.display_name,
            gender_id,
            age_id,
            division_id,
            weight_id,
            history_json,
        ],
    )?;

    Ok(())
}

fn normalize_gender(value: &str) -> String {
    match value.trim().to_uppercase().as_str() {
        "F" | "FEMALE" | "W" | "WOMEN" | "WOMAN" => "WOMEN".to_string(),
        "M" | "MALE" | "MEN" | "MAN" => "MEN".to_string(),
        other => other.to_string(),
    }
}

fn split_match_log_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut depth_brace = 0;
    let mut depth_bracket = 0;

    for ch in line.chars() {
        match ch {
            ',' if depth_brace == 0 && depth_bracket == 0 => {
                fields.push(current.clone());
                current.clear();
            }
            '{' => {
                depth_brace += 1;
                current.push(ch);
            }
            '}' => {
                if depth_brace > 0 {
                    depth_brace -= 1;
                }
                current.push(ch);
            }
            '[' => {
                depth_bracket += 1;
                current.push(ch);
            }
            ']' => {
                if depth_bracket > 0 {
                    depth_bracket -= 1;
                }
                current.push(ch);
            }
            _ => current.push(ch),
        }
    }
    fields.push(current);
    fields
}
