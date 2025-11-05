PRAGMA foreign_keys = OFF;
PRAGMA recursive_triggers = OFF;
BEGIN TRANSACTION;

----------------------------------------------------------------------
-- 1. Rename lookup/catalog tables to prefix-free names (structure kept)
----------------------------------------------------------------------

ALTER TABLE look_age_groups RENAME TO age_group;
ALTER TABLE look_disciplines RENAME TO discipline;
ALTER TABLE look_divisions RENAME TO division;
ALTER TABLE look_genders RENAME TO gender;
ALTER TABLE look_round_configs RENAME TO round_config;
ALTER TABLE look_weight_classes RENAME TO weight_class;

ALTER TABLE flags RENAME TO flag;
ALTER TABLE recognition_history RENAME TO flag_recognition_history;
ALTER TABLE ovr_providers RENAME TO legacy_overlay_provider;
ALTER TABLE ovr_tournaments RENAME TO legacy_overlay_tournament;
ALTER TABLE ovr_categories RENAME TO legacy_overlay_category;
ALTER TABLE ovr_to_local_tournament RENAME TO legacy_overlay_tournament_map;
ALTER TABLE ovr_flag_animations RENAME TO legacy_overlay_flag_animation;
ALTER TABLE ovr_anthems RENAME TO legacy_overlay_anthem;
ALTER TABLE obs_connections RENAME TO legacy_obs_connection;
ALTER TABLE obs_recording_sessions RENAME TO legacy_obs_recording_session;
ALTER TABLE obs_scenes RENAME TO legacy_obs_scene;
ALTER TABLE obs_recording_config RENAME TO legacy_obs_recording_config;
ALTER TABLE udp_server_configs RENAME TO legacy_udp_server_config;
ALTER TABLE udp_server_sessions RENAME TO legacy_udp_server_session;
ALTER TABLE udp_client_connections RENAME TO legacy_udp_client_connection;


ALTER TABLE pss_event_details RENAME TO legacy_event_detail;
ALTER TABLE pss_event_recognition_history RENAME TO legacy_event_recognition_history;
ALTER TABLE pss_event_statistics RENAME TO legacy_event_statistic;
ALTER TABLE pss_event_types RENAME TO legacy_event_type;
ALTER TABLE pss_event_validation_results RENAME TO legacy_event_validation_result;
ALTER TABLE pss_event_validation_rules RENAME TO legacy_event_validation_rule;
ALTER TABLE pss_events RENAME TO legacy_event;
ALTER TABLE pss_rounds RENAME TO legacy_round;
ALTER TABLE pss_scores RENAME TO legacy_score;
ALTER TABLE pss_unknown_events RENAME TO legacy_event_unknown;
ALTER TABLE pss_warnings RENAME TO legacy_event_warning;
ALTER TABLE recorded_videos RENAME TO legacy_video;
ALTER TABLE recorded_video_events RENAME TO legacy_video_event;

----------------------------------------------------------------------
-- 2. Preserve legacy tables that will be merged into the canonical schema
----------------------------------------------------------------------

ALTER TABLE tournaments RENAME TO legacy_tournament;
ALTER TABLE athletes RENAME TO legacy_athlete;
ALTER TABLE pss_athletes RENAME TO legacy_pss_athlete;
ALTER TABLE pss_matches RENAME TO legacy_match;
ALTER TABLE pss_match_athletes RENAME TO legacy_match_participant;

----------------------------------------------------------------------
-- 3. Canonical tables
----------------------------------------------------------------------

CREATE TABLE tournament (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    duration_days INTEGER NOT NULL DEFAULT 1,
    city TEXT,
    country TEXT,
    country_code TEXT,
    logo_path TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    start_date TEXT,
    end_date TEXT,
    ranking_id INTEGER,
    location TEXT NOT NULL DEFAULT '{}',
    contact TEXT NOT NULL DEFAULT '{}',
    organizing_committee TEXT NOT NULL DEFAULT '{}',
    officials TEXT NOT NULL DEFAULT '{}',
    banner TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE athlete (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    wt_id TEXT,
    pss_code TEXT UNIQUE,
    country TEXT,
    short_name TEXT,
    display_name TEXT,
    first_name TEXT,
    last_name TEXT,
    country_code TEXT,
    ioc_code TEXT,
    flag_id INTEGER,
    gender_id INTEGER,
    division_id INTEGER,
    weight_class_id INTEGER,
    age_group_id INTEGER,
    image TEXT,
    history TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (flag_id) REFERENCES flag(id),
    FOREIGN KEY (gender_id) REFERENCES gender(id),
    FOREIGN KEY (division_id) REFERENCES division(id),
    FOREIGN KEY (weight_class_id) REFERENCES weight_class(id),
    FOREIGN KEY (age_group_id) REFERENCES age_group(id)
);

CREATE TABLE match (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    tournament_id INTEGER,
    match_code TEXT NOT NULL,
    match_number TEXT,
    category TEXT,
    division_code TEXT,
    weight_class_code TEXT,
    total_rounds INTEGER,
    round_duration INTEGER,
    countdown_type TEXT,
    format_type INTEGER,
    creation_mode TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (tournament_id) REFERENCES tournament(id)
);

CREATE TABLE match_participant (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id INTEGER NOT NULL,
    athlete_id INTEGER NOT NULL,
    side TEXT NOT NULL CHECK (side IN ('blue','red')),
    bg_color TEXT,
    fg_color TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (match_id) REFERENCES match(id) ON DELETE CASCADE,
    FOREIGN KEY (athlete_id) REFERENCES athlete(id)
);

CREATE INDEX idx_match_participant_match ON match_participant(match_id);
CREATE INDEX idx_match_participant_athlete ON match_participant(athlete_id);

----------------------------------------------------------------------
-- 4. Temporary helper mappings
----------------------------------------------------------------------

CREATE TEMP TABLE tmp_athlete_map (
    source TEXT NOT NULL,
    source_id INTEGER NOT NULL,
    athlete_id INTEGER NOT NULL,
    PRIMARY KEY (source, source_id)
);

CREATE TEMP TABLE tmp_match_map (
    uuid TEXT PRIMARY KEY,
    match_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS migration_missing_participants (
    match_uuid TEXT,
    legacy_athlete_id INTEGER
);

----------------------------------------------------------------------
-- 5. Populate canonical tables
----------------------------------------------------------------------

-- Tournaments
INSERT INTO tournament (
    id, uuid, name, duration_days, city, country, country_code, logo_path, status,
    start_date, end_date, ranking_id, location, contact, organizing_committee, officials,
    banner, created_at, updated_at
)
SELECT
    id, uuid, name, duration_days, city, country, country_code, logo_path, status,
    start_date, end_date, ranking_id, location, contact, oc, officials,
    banner, created_at, updated_at
FROM legacy_tournament;

-- Athletes (seed from tournament roster)
INSERT INTO athlete (
    uuid, wt_id, country, short_name, display_name, first_name, last_name,
    country_code, ioc_code, gender_id, division_id, weight_class_id, age_group_id,
    image, history, created_at, updated_at
)
SELECT
    lower(hex(randomblob(16))),
    NULLIF(la.wtid, ''),
    NULLIF(la.country, ''),
    NULLIF(la.display_name, ''),
    NULLIF(la.display_name, ''),
    NULLIF(la.first_name, ''),
    NULLIF(la.last_name, ''),
    NULLIF(la.country_code, ''),
    NULLIF(la.ioc_code, ''),
    la.look_gender_id,
    la.look_division_id,
    la.look_weight_class_id,
    la.look_age_group_id,
    la.image,
    la.history,
    la.created_at,
    la.updated_at
FROM legacy_athlete la;

-- Map legacy athlete IDs (tournament source)
INSERT OR IGNORE INTO tmp_athlete_map (source, source_id, athlete_id)
SELECT 'legacy_athlete', la.id, a.id
FROM legacy_athlete la
JOIN athlete a ON a.wt_id = la.wtid;

-- Update athletes with matching PSS codes (where wtid == athlete_code)
UPDATE athlete
SET pss_code = (
        SELECT lpa.athlete_code
        FROM legacy_pss_athlete lpa
        WHERE lpa.athlete_code = athlete.wt_id
        LIMIT 1
    ),
    short_name = COALESCE(
        short_name,
        (SELECT lpa.short_name FROM legacy_pss_athlete lpa WHERE lpa.athlete_code = athlete.wt_id LIMIT 1)
    ),
    display_name = COALESCE(
        display_name,
        (SELECT COALESCE(lpa.long_name, lpa.short_name)
         FROM legacy_pss_athlete lpa
         WHERE lpa.athlete_code = athlete.wt_id
         LIMIT 1)
    ),
    country_code = COALESCE(
        country_code,
        (SELECT lpa.country_code FROM legacy_pss_athlete lpa WHERE lpa.athlete_code = athlete.wt_id LIMIT 1)
    ),
    ioc_code = COALESCE(
        ioc_code,
        (SELECT lpa.country_code FROM legacy_pss_athlete lpa WHERE lpa.athlete_code = athlete.wt_id LIMIT 1)
    )
WHERE EXISTS (
    SELECT 1 FROM legacy_pss_athlete lpa WHERE lpa.athlete_code = athlete.wt_id
);

-- Link remaining PSS athletes by matching short/long name
UPDATE athlete
SET pss_code = (
        SELECT lpa.athlete_code
        FROM legacy_pss_athlete lpa
        WHERE (lower(lpa.short_name) = lower(COALESCE(athlete.short_name, ''))
               OR lower(COALESCE(lpa.long_name, '')) = lower(COALESCE(athlete.display_name, '')))
        LIMIT 1
    ),
    country_code = COALESCE(
        country_code,
        (SELECT lpa.country_code
         FROM legacy_pss_athlete lpa
         WHERE (lower(lpa.short_name) = lower(COALESCE(athlete.short_name, ''))
                OR lower(COALESCE(lpa.long_name, '')) = lower(COALESCE(athlete.display_name, '')))
         LIMIT 1)
    ),
    ioc_code = COALESCE(
        ioc_code,
        (SELECT lpa.country_code
         FROM legacy_pss_athlete lpa
         WHERE (lower(lpa.short_name) = lower(COALESCE(athlete.short_name, ''))
                OR lower(COALESCE(lpa.long_name, '')) = lower(COALESCE(athlete.display_name, '')))
         LIMIT 1)
    )
WHERE pss_code IS NULL
  AND EXISTS (
        SELECT 1
        FROM legacy_pss_athlete lpa
        WHERE (lower(lpa.short_name) = lower(COALESCE(athlete.short_name, ''))
               OR lower(COALESCE(lpa.long_name, '')) = lower(COALESCE(athlete.display_name, '')))
    );

-- Insert unmatched PSS athletes
INSERT INTO athlete (
    uuid, wt_id, pss_code, country, short_name, display_name,
    country_code, ioc_code, created_at, updated_at
)
SELECT
    lower(hex(randomblob(16))),
    NULL,
    lpa.athlete_code,
    NULL,
    lpa.short_name,
    COALESCE(lpa.long_name, lpa.short_name),
    lpa.country_code,
    lpa.country_code,
    COALESCE(lpa.created_at, datetime('now')),
    COALESCE(lpa.updated_at, datetime('now'))
FROM legacy_pss_athlete lpa
WHERE NOT EXISTS (
    SELECT 1 FROM athlete a WHERE a.pss_code = lpa.athlete_code
);

-- Post-process flags
UPDATE athlete
SET flag_id = (
        SELECT f.id FROM flag f
        WHERE f.ioc_code IS NOT NULL
          AND athlete.ioc_code IS NOT NULL
          AND upper(f.ioc_code) = upper(athlete.ioc_code)
        ORDER BY f.id
        LIMIT 1
    )
WHERE flag_id IS NULL
  AND ioc_code IS NOT NULL;

-- Capture PSS athlete ID mapping
INSERT OR IGNORE INTO tmp_athlete_map (source, source_id, athlete_id)
SELECT 'legacy_pss_athlete', lpa.id, a.id
FROM legacy_pss_athlete lpa
JOIN athlete a ON a.pss_code = lpa.athlete_code;

----------------------------------------------------------------------
-- Matches
----------------------------------------------------------------------

INSERT INTO match (
    id, uuid, tournament_id, match_code, match_number, category,
    division_code, weight_class_code, total_rounds, round_duration,
    countdown_type, format_type, creation_mode, created_at, updated_at
)
SELECT
    lm.id,
    COALESCE(NULLIF(lm.uuid, ''), lower(hex(randomblob(16)))),
    t.id,
    lm.match_id,
    lm.match_number,
    lm.category,
    lm.division,
    lm.weight_class,
    lm.total_rounds,
    lm.round_duration,
    lm.countdown_type,
    lm.format_type,
    lm.creation_mode,
    lm.created_at,
    lm.updated_at
FROM legacy_match lm
LEFT JOIN tournament t ON t.uuid = lm.tournament_id;

INSERT INTO tmp_match_map (uuid, match_id)
SELECT uuid, id FROM match;

----------------------------------------------------------------------
-- Match participants
----------------------------------------------------------------------

-- Record missing participants for manual review
INSERT INTO migration_missing_participants (match_uuid, legacy_athlete_id)
SELECT
    lmp.match_id,
    lmp.athlete_id
FROM legacy_match_participant lmp
LEFT JOIN tmp_match_map mm ON mm.uuid = lmp.match_id
LEFT JOIN tmp_athlete_map map_pss
    ON map_pss.source = 'legacy_pss_athlete' AND map_pss.source_id = lmp.athlete_id
LEFT JOIN tmp_athlete_map map_legacy
    ON map_legacy.source = 'legacy_athlete' AND map_legacy.source_id = lmp.athlete_id
WHERE mm.match_id IS NULL
   OR (map_pss.athlete_id IS NULL AND map_legacy.athlete_id IS NULL);

-- Migrate participants with known athletes
INSERT INTO match_participant (
    match_id, athlete_id, side, bg_color, fg_color, created_at
)
SELECT
    mm.match_id,
    COALESCE(map_pss.athlete_id, map_legacy.athlete_id),
    CASE WHEN lmp.athlete_position = 2 THEN 'red' ELSE 'blue' END,
    lmp.bg_color,
    lmp.fg_color,
    COALESCE(lmp.created_at, datetime('now'))
FROM legacy_match_participant lmp
JOIN tmp_match_map mm ON mm.uuid = lmp.match_id
LEFT JOIN tmp_athlete_map map_pss
    ON map_pss.source = 'legacy_pss_athlete' AND map_pss.source_id = lmp.athlete_id
LEFT JOIN tmp_athlete_map map_legacy
    ON map_legacy.source = 'legacy_athlete' AND map_legacy.source_id = lmp.athlete_id
WHERE COALESCE(map_pss.athlete_id, map_legacy.athlete_id) IS NOT NULL;

----------------------------------------------------------------------
----------------------------------------------------------------------
-- Rounds
----------------------------------------------------------------------

CREATE TABLE round (
    id INTEGER PRIMARY KEY,
    match_id INTEGER NOT NULL,
    round_number INTEGER NOT NULL,
    start_time TEXT,
    end_time TEXT,
    duration_seconds INTEGER,
    winner_side TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (match_id) REFERENCES match(id)
);

INSERT INTO round (
    id, match_id, round_number, start_time, end_time, duration_seconds, winner_side, created_at
)
SELECT
    lr.id,
    mm.match_id,
    lr.round_number,
    lr.start_time,
    lr.end_time,
    lr.duration,
    CASE lr.winner_athlete_position WHEN 2 THEN 'red' WHEN 1 THEN 'blue' ELSE NULL END,
    COALESCE(lr.created_at, datetime('now'))
FROM legacy_round lr
JOIN tmp_match_map mm ON mm.uuid = lr.match_id;

----------------------------------------------------------------------
-- Scores
----------------------------------------------------------------------

CREATE TABLE score (
    id INTEGER PRIMARY KEY,
    match_id INTEGER NOT NULL,
    round_id INTEGER,
    side TEXT NOT NULL CHECK(side IN ('blue','red')),
    type TEXT NOT NULL,
    value INTEGER NOT NULL,
    timestamp TEXT,
    tournament_uuid TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (match_id) REFERENCES match(id),
    FOREIGN KEY (round_id) REFERENCES round(id)
);

INSERT INTO score (
    id, match_id, round_id, side, type, value, timestamp, tournament_uuid, created_at
)
SELECT
    ls.id,
    mm.match_id,
    ls.round_id,
    CASE ls.athlete_position WHEN 2 THEN 'red' ELSE 'blue' END,
    ls.score_type,
    ls.score_value,
    ls.timestamp,
    ls.tournament_id,
    COALESCE(ls.created_at, datetime('now'))
FROM legacy_score ls
JOIN tmp_match_map mm ON mm.uuid = ls.match_id;

----------------------------------------------------------------------
-- Event master data
----------------------------------------------------------------------

CREATE TABLE event_type (
    id INTEGER PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO event_type (id, code, name, description, category, is_active, created_at)
SELECT
    let.id,
    let.event_code,
    let.event_name,
    let.description,
    let.category,
    let.is_active,
    COALESCE(let.created_at, datetime('now'))
FROM legacy_event_type let;

CREATE TABLE event (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL,
    match_id INTEGER,
    round_id INTEGER,
    event_type_id INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    raw_data TEXT NOT NULL,
    parsed_data TEXT,
    event_sequence INTEGER,
    processing_time_ms INTEGER,
    is_valid BOOLEAN NOT NULL DEFAULT 0,
    error_message TEXT,
    recognition_status TEXT NOT NULL,
    protocol_version TEXT,
    parser_confidence REAL,
    validation_errors TEXT,
    tournament_uuid TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (match_id) REFERENCES match(id),
    FOREIGN KEY (round_id) REFERENCES round(id),
    FOREIGN KEY (event_type_id) REFERENCES event_type(id)
);

INSERT INTO event (
    id, session_id, match_id, round_id, event_type_id, timestamp, raw_data, parsed_data,
    event_sequence, processing_time_ms, is_valid, error_message, recognition_status,
    protocol_version, parser_confidence, validation_errors, tournament_uuid, created_at
)
SELECT
    le.id,
    le.session_id,
    le.match_id,
    le.round_id,
    le.event_type_id,
    le.timestamp,
    le.raw_data,
    le.parsed_data,
    le.event_sequence,
    le.processing_time_ms,
    le.is_valid,
    le.error_message,
    le.recognition_status,
    le.protocol_version,
    le.parser_confidence,
    le.validation_errors,
    le.tournament_id,
    COALESCE(le.created_at, datetime('now'))
FROM legacy_event le;

CREATE TABLE event_detail (
    id INTEGER PRIMARY KEY,
    event_id INTEGER NOT NULL,
    detail_key TEXT NOT NULL,
    detail_value TEXT,
    detail_type TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (event_id) REFERENCES event(id),
    UNIQUE(event_id, detail_key)
);

INSERT INTO event_detail (id, event_id, detail_key, detail_value, detail_type, created_at)
SELECT
    led.id,
    led.event_id,
    led.detail_key,
    led.detail_value,
    led.detail_type,
    COALESCE(led.created_at, datetime('now'))
FROM legacy_event_detail led;

CREATE TABLE event_recognition_history (
    id INTEGER PRIMARY KEY,
    event_id INTEGER NOT NULL,
    old_status TEXT NOT NULL,
    new_status TEXT NOT NULL,
    changed_by TEXT NOT NULL DEFAULT 'system',
    change_reason TEXT,
    protocol_version TEXT,
    raw_data TEXT NOT NULL,
    parsed_data TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (event_id) REFERENCES event(id)
);

INSERT INTO event_recognition_history (
    id, event_id, old_status, new_status, changed_by, change_reason,
    protocol_version, raw_data, parsed_data, created_at
)
SELECT
    ler.id,
    ler.event_id,
    ler.old_status,
    ler.new_status,
    ler.changed_by,
    ler.change_reason,
    ler.protocol_version,
    ler.raw_data,
    ler.parsed_data,
    COALESCE(ler.created_at, datetime('now'))
FROM legacy_event_recognition_history ler;

CREATE TABLE event_statistic (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL,
    event_type_id INTEGER,
    total_events INTEGER NOT NULL DEFAULT 0,
    recognized_events INTEGER NOT NULL DEFAULT 0,
    unknown_events INTEGER NOT NULL DEFAULT 0,
    partial_events INTEGER NOT NULL DEFAULT 0,
    deprecated_events INTEGER NOT NULL DEFAULT 0,
    validation_errors INTEGER NOT NULL DEFAULT 0,
    parsing_errors INTEGER NOT NULL DEFAULT 0,
    average_processing_time_ms REAL NOT NULL DEFAULT 0.0,
    min_processing_time_ms INTEGER,
    max_processing_time_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (event_type_id) REFERENCES event_type(id)
);

INSERT INTO event_statistic (
    id, session_id, event_type_id, total_events, recognized_events, unknown_events,
    partial_events, deprecated_events, validation_errors, parsing_errors,
    average_processing_time_ms, min_processing_time_ms, max_processing_time_ms,
    created_at, updated_at
)
SELECT
    les.id,
    les.session_id,
    les.event_type_id,
    les.total_events,
    les.recognized_events,
    les.unknown_events,
    les.partial_events,
    les.deprecated_events,
    les.validation_errors,
    les.parsing_errors,
    les.average_processing_time_ms,
    les.min_processing_time_ms,
    les.max_processing_time_ms,
    COALESCE(les.created_at, datetime('now')),
    COALESCE(les.updated_at, datetime('now'))
FROM legacy_event_statistic les;

CREATE TABLE event_validation_rule (
    id INTEGER PRIMARY KEY,
    event_code TEXT NOT NULL,
    protocol_version TEXT NOT NULL,
    rule_name TEXT NOT NULL,
    rule_type TEXT NOT NULL,
    rule_definition TEXT NOT NULL,
    error_message TEXT,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO event_validation_rule (
    id, event_code, protocol_version, rule_name, rule_type, rule_definition,
    error_message, is_active, created_at, updated_at
)
SELECT
    levr.id,
    levr.event_code,
    levr.protocol_version,
    levr.rule_name,
    levr.rule_type,
    levr.rule_definition,
    levr.error_message,
    levr.is_active,
    COALESCE(levr.created_at, datetime('now')),
    COALESCE(levr.updated_at, datetime('now'))
FROM legacy_event_validation_rule levr;

CREATE TABLE event_validation_result (
    id INTEGER PRIMARY KEY,
    event_id INTEGER NOT NULL,
    rule_id INTEGER NOT NULL,
    validation_passed BOOLEAN NOT NULL,
    error_message TEXT,
    validation_time_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (event_id) REFERENCES event(id),
    FOREIGN KEY (rule_id) REFERENCES event_validation_rule(id)
);

INSERT INTO event_validation_result (
    id, event_id, rule_id, validation_passed, error_message, validation_time_ms, created_at
)
SELECT
    levr.id,
    levr.event_id,
    levr.rule_id,
    levr.validation_passed,
    levr.error_message,
    levr.validation_time_ms,
    COALESCE(levr.created_at, datetime('now'))
FROM legacy_event_validation_result levr;

CREATE TABLE event_warning (
    id INTEGER PRIMARY KEY,
    match_id INTEGER NOT NULL,
    round_id INTEGER,
    side TEXT NOT NULL CHECK (side IN ('blue','red')),
    warning_type TEXT NOT NULL,
    warning_count INTEGER NOT NULL DEFAULT 0,
    timestamp TEXT,
    tournament_uuid TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (match_id) REFERENCES match(id),
    FOREIGN KEY (round_id) REFERENCES round(id)
);

INSERT INTO event_warning (
    id, match_id, round_id, side, warning_type, warning_count, timestamp, tournament_uuid, created_at
)
SELECT
    lew.id,
    mm.match_id,
    lew.round_id,
    CASE lew.athlete_position WHEN 2 THEN 'red' ELSE 'blue' END,
    lew.warning_type,
    lew.warning_count,
    lew.timestamp,
    lew.tournament_id,
    COALESCE(lew.created_at, datetime('now'))
FROM legacy_event_warning lew
JOIN tmp_match_map mm ON mm.uuid = lew.match_id;

CREATE TABLE event_unknown (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL,
    raw_data TEXT NOT NULL,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    pattern_hash TEXT,
    suggested_event_type TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO event_unknown (
    id, session_id, raw_data, first_seen, last_seen, occurrence_count, pattern_hash,
    suggested_event_type, notes, created_at, updated_at
)
SELECT
    leu.id,
    leu.session_id,
    leu.raw_data,
    leu.first_seen,
    leu.last_seen,
    leu.occurrence_count,
    leu.pattern_hash,
    leu.suggested_event_type,
    leu.notes,
    COALESCE(leu.created_at, datetime('now')),
    COALESCE(leu.updated_at, datetime('now'))
FROM legacy_event_unknown leu;

----------------------------------------------------------------------
-- Videos (dependent on events)
----------------------------------------------------------------------

CREATE TABLE video (
    id INTEGER PRIMARY KEY,
    match_id INTEGER NOT NULL,
    event_id INTEGER,
    tournament_uuid TEXT,
    type TEXT NOT NULL,
    file_path TEXT,
    directory TEXT,
    filename_formatting TEXT,
    start_time TEXT NOT NULL,
    duration_seconds INTEGER,
    file_size INTEGER,
    checksum TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (match_id) REFERENCES match(id),
    FOREIGN KEY (event_id) REFERENCES event(id)
);

INSERT INTO video (
    id, match_id, event_id, tournament_uuid, type, file_path, directory,
    filename_formatting, start_time, duration_seconds, file_size, checksum, created_at
)
SELECT
    lv.id,
    lv.match_id,
    lv.event_id,
    lv.tournament_id,
    lv.video_type,
    lv.file_path,
    lv.record_directory,
    lv.filename_formatting,
    lv.start_time,
    lv.duration_seconds,
    lv.file_size,
    lv.checksum,
    COALESCE(lv.created_at, datetime('now'))
FROM legacy_video lv;

CREATE TABLE video_event (
    id INTEGER PRIMARY KEY,
    video_id INTEGER NOT NULL,
    event_id INTEGER NOT NULL,
    offset_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (video_id) REFERENCES video(id),
    FOREIGN KEY (event_id) REFERENCES event(id)
);

INSERT INTO video_event (id, video_id, event_id, offset_ms, created_at)
SELECT
    lve.id,
    lve.recorded_video_id,
    lve.event_id,
    lve.offset_ms,
    COALESCE(lve.created_at, datetime('now'))
FROM legacy_video_event lve;

----------------------------------------------------------------------
-- Overlay providers and related tables
----------------------------------------------------------------------

CREATE TABLE overlay_provider (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    base_url TEXT,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    rate_limit_ms INTEGER NOT NULL DEFAULT 1000,
    last_refreshed_at TEXT,
    last_status TEXT,
    last_error TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO overlay_provider (
    id, name, base_url, enabled, rate_limit_ms, last_refreshed_at, last_status, last_error, created_at, updated_at
)
SELECT
    id,
    name,
    base_url,
    COALESCE(enabled, 0),
    COALESCE(rate_limit_ms, 1000),
    last_refreshed_at,
    last_status,
    last_error,
    COALESCE(created_at, datetime('now')),
    COALESCE(updated_at, datetime('now'))
FROM legacy_overlay_provider;

CREATE TABLE overlay_tournament (
    id INTEGER PRIMARY KEY,
    provider_id INTEGER NOT NULL,
    provider_tournament_id TEXT NOT NULL,
    name TEXT NOT NULL,
    start_date TEXT,
    end_date TEXT,
    city TEXT,
    country TEXT,
    url TEXT,
    status TEXT,
    last_seen_at TEXT,
    hash TEXT,
    etag TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (provider_id) REFERENCES overlay_provider(id)
);

INSERT INTO overlay_tournament (
    id, provider_id, provider_tournament_id, name, start_date, end_date, city, country, url, status, last_seen_at, hash, etag, created_at, updated_at
)
SELECT
    id,
    provider_id,
    provider_tournament_id,
    name,
    start_date,
    end_date,
    city,
    country,
    url,
    status,
    last_seen_at,
    hash,
    etag,
    COALESCE(created_at, datetime('now')),
    COALESCE(updated_at, datetime('now'))
FROM legacy_overlay_tournament;

CREATE TABLE overlay_category (
    id INTEGER PRIMARY KEY,
    overlay_tournament_id INTEGER NOT NULL,
    discipline TEXT,
    age_group TEXT,
    gender TEXT,
    division TEXT,
    weight_class TEXT,
    bracket_stage TEXT,
    provider_raw TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (overlay_tournament_id) REFERENCES overlay_tournament(id)
);

INSERT INTO overlay_category (
    id, overlay_tournament_id, discipline, age_group, gender, division, weight_class, bracket_stage, provider_raw, created_at, updated_at
)
SELECT
    id,
    tournament_id,
    discipline,
    age_group,
    gender,
    division,
    weight_class,
    bracket_stage,
    provider_raw,
    COALESCE(created_at, datetime('now')),
    COALESCE(updated_at, datetime('now'))
FROM legacy_overlay_category;

CREATE TABLE overlay_tournament_map (
    id INTEGER PRIMARY KEY,
    overlay_tournament_id INTEGER NOT NULL,
    tournament_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (overlay_tournament_id) REFERENCES overlay_tournament(id),
    FOREIGN KEY (tournament_id) REFERENCES tournament(id)
);

INSERT INTO overlay_tournament_map (id, overlay_tournament_id, tournament_id, created_at)
SELECT
    id,
    ovr_tournament_id,
    local_tournament_id,
    COALESCE(created_at, datetime('now'))
FROM legacy_overlay_tournament_map;

CREATE TABLE overlay_flag_animation (
    id TEXT PRIMARY KEY,
    ioc_code TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    display_name TEXT,
    duration_ms INTEGER,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO overlay_flag_animation (
    id, ioc_code, file_name, file_path, display_name, duration_ms, is_default, created_at, updated_at
)
SELECT
    id,
    ioc_code,
    file_name,
    file_path,
    display_name,
    duration_ms,
    COALESCE(is_default, 0),
    COALESCE(created_at, datetime('now')),
    COALESCE(updated_at, datetime('now'))
FROM legacy_overlay_flag_animation;

CREATE TABLE overlay_anthem (
    id TEXT PRIMARY KEY,
    ioc_code TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    display_name TEXT,
    duration_ms INTEGER,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO overlay_anthem (
    id, ioc_code, file_name, file_path, display_name, duration_ms, is_default, created_at, updated_at
)
SELECT
    id,
    ioc_code,
    file_name,
    file_path,
    display_name,
    duration_ms,
    COALESCE(is_default, 0),
    COALESCE(created_at, datetime('now')),
    COALESCE(updated_at, datetime('now'))
FROM legacy_overlay_anthem;

----------------------------------------------------------------------
-- OBS connection metadata
----------------------------------------------------------------------

CREATE TABLE obs_connection (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    password TEXT,
    is_active BOOLEAN NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'disconnected',
    error TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO obs_connection (
    id, name, host, port, password, is_active, status, error, created_at, updated_at
)
SELECT
    id,
    name,
    host,
    port,
    password,
    COALESCE(is_active, 0),
    status,
    error,
    COALESCE(created_at, datetime('now')),
    COALESCE(updated_at, datetime('now'))
FROM legacy_obs_connection;

CREATE TABLE obs_scene (
    id INTEGER PRIMARY KEY,
    scene_name TEXT NOT NULL,
    scene_id TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_seen_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO obs_scene (
    id, scene_name, scene_id, is_active, last_seen_at, created_at, updated_at
)
SELECT
    id,
    scene_name,
    scene_id,
    COALESCE(is_active, 0),
    COALESCE(last_seen_at, datetime('now')),
    COALESCE(created_at, datetime('now')),
    COALESCE(updated_at, datetime('now'))
FROM legacy_obs_scene;

CREATE TABLE obs_recording_config (
    id INTEGER PRIMARY KEY,
    obs_connection_name TEXT NOT NULL,
    recording_root_path TEXT NOT NULL,
    recording_format TEXT NOT NULL DEFAULT 'mp4',
    replay_buffer_enabled BOOLEAN NOT NULL DEFAULT 1,
    replay_buffer_duration INTEGER NOT NULL DEFAULT 30,
    auto_start_recording BOOLEAN NOT NULL DEFAULT 1,
    auto_start_replay_buffer BOOLEAN NOT NULL DEFAULT 1,
    filename_template TEXT NOT NULL DEFAULT '{matchNumber}_{player1}_{player2}_{date}',
    folder_pattern TEXT NOT NULL DEFAULT '{tournament}/{tournamentDay}',
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO obs_recording_config (
    id, obs_connection_name, recording_root_path, recording_format, replay_buffer_enabled,
    replay_buffer_duration, auto_start_recording, auto_start_replay_buffer, filename_template,
    folder_pattern, is_active, created_at, updated_at
)
SELECT
    id,
    obs_connection_name,
    recording_root_path,
    recording_format,
    COALESCE(replay_buffer_enabled, 0),
    COALESCE(replay_buffer_duration, 0),
    COALESCE(auto_start_recording, 0),
    COALESCE(auto_start_replay_buffer, 0),
    filename_template,
    folder_pattern,
    COALESCE(is_active, 0),
    COALESCE(created_at, datetime('now')),
    COALESCE(updated_at, datetime('now'))
FROM legacy_obs_recording_config;

CREATE TABLE obs_recording_session (
    id INTEGER PRIMARY KEY,
    obs_connection_name TEXT NOT NULL,
    tournament_id INTEGER,
    match_code TEXT,
    match_number TEXT,
    player1_name TEXT,
    player1_flag TEXT,
    player2_name TEXT,
    player2_flag TEXT,
    recording_path TEXT NOT NULL,
    recording_filename TEXT NOT NULL,
    recording_start_time TEXT,
    recording_end_time TEXT,
    recording_duration INTEGER,
    recording_size_bytes INTEGER,
    replay_buffer_start_time TEXT,
    replay_buffer_end_time TEXT,
    replay_buffer_saved BOOLEAN NOT NULL DEFAULT 0,
    replay_buffer_filename TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO obs_recording_session (
    id, obs_connection_name, tournament_id, match_code, match_number, player1_name, player1_flag,
    player2_name, player2_flag, recording_path, recording_filename, recording_start_time, recording_end_time,
    recording_duration, recording_size_bytes, replay_buffer_start_time, replay_buffer_end_time, replay_buffer_saved,
    replay_buffer_filename, status, error_message, created_at, updated_at
)
SELECT
    id,
    obs_connection_name,
    tournament_id,
    match_id,
    match_number,
    player1_name,
    player1_flag,
      player2_name,
      player2_flag,
      recording_path,
      recording_filename,
      recording_start_time,
      recording_end_time,
      recording_duration,
      recording_size_bytes,
      replay_buffer_start_time,
      replay_buffer_end_time,
      COALESCE(replay_buffer_saved, 0),
      replay_buffer_filename,
      status,
      error_message,
      COALESCE(datetime(created, 'unixepoch'), datetime('now')),
      COALESCE(datetime(updated, 'unixepoch'), datetime('now'))
FROM legacy_obs_recording_session;

----------------------------------------------------------------------
-- UDP server/client telemetry
----------------------------------------------------------------------

CREATE TABLE udp_server_config (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    port INTEGER NOT NULL,
    bind_address TEXT NOT NULL,
    network_interface_id INTEGER,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    auto_start BOOLEAN NOT NULL DEFAULT 0,
    max_packet_size INTEGER NOT NULL DEFAULT 1024,
    buffer_size INTEGER NOT NULL DEFAULT 8192,
    timeout_ms INTEGER NOT NULL DEFAULT 1000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (network_interface_id) REFERENCES network_interfaces(id)
);

INSERT INTO udp_server_config (
    id, name, port, bind_address, network_interface_id, enabled, auto_start,
    max_packet_size, buffer_size, timeout_ms, created_at, updated_at
)
SELECT
    id,
    name,
    port,
    bind_address,
    network_interface_id,
    COALESCE(enabled, 0),
    COALESCE(auto_start, 0),
    COALESCE(max_packet_size, 1024),
    COALESCE(buffer_size, 8192),
    COALESCE(timeout_ms, 1000),
    COALESCE(created_at, datetime('now')),
    COALESCE(updated_at, datetime('now'))
FROM legacy_udp_server_config;

CREATE TABLE udp_server_session (
    id INTEGER PRIMARY KEY,
    server_config_id INTEGER NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    packets_received INTEGER NOT NULL DEFAULT 0,
    packets_parsed INTEGER NOT NULL DEFAULT 0,
    parse_errors INTEGER NOT NULL DEFAULT 0,
    total_bytes_received INTEGER NOT NULL DEFAULT 0,
    average_packet_size REAL NOT NULL DEFAULT 0.0,
    max_packet_size_seen INTEGER NOT NULL DEFAULT 0,
    min_packet_size_seen INTEGER NOT NULL DEFAULT 0,
    unique_clients_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (server_config_id) REFERENCES udp_server_config(id)
);

INSERT INTO udp_server_session (
    id, server_config_id, start_time, end_time, status, packets_received, packets_parsed,
    parse_errors, total_bytes_received, average_packet_size, max_packet_size_seen,
    min_packet_size_seen, unique_clients_count, error_message, created_at, updated_at
)
SELECT
    id,
    server_config_id,
    start_time,
    end_time,
    status,
    packets_received,
    packets_parsed,
    parse_errors,
    total_bytes_received,
    average_packet_size,
    max_packet_size_seen,
    min_packet_size_seen,
    unique_clients_count,
    error_message,
    COALESCE(datetime(created, 'unixepoch'), datetime('now')),
    COALESCE(datetime(updated, 'unixepoch'), datetime('now'))
FROM legacy_udp_server_session;

CREATE TABLE udp_client_connection (
    id INTEGER PRIMARY KEY,
    server_session_id INTEGER NOT NULL,
    client_address TEXT NOT NULL,
    client_port INTEGER NOT NULL,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    packets_received INTEGER NOT NULL DEFAULT 0,
    total_bytes_received INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (server_session_id) REFERENCES udp_server_session(id)
);

INSERT INTO udp_client_connection (
    id, server_session_id, client_address, client_port, first_seen, last_seen,
    packets_received, total_bytes_received, is_active, created_at
)
SELECT
    id,
    session_id,
    client_address,
    client_port,
    first_seen,
    last_seen,
    packets_received,
    total_bytes_received,
    COALESCE(is_active, 0),
    COALESCE(datetime(created, 'unixepoch'), datetime('now'))
FROM legacy_udp_client_connection;

----------------------------------------------------------------------
-- 6. Housekeeping
----------------------------------------------------------------------

-- Refresh sqlite_sequence for tables with explicit id inserts
INSERT OR REPLACE INTO sqlite_sequence (name, seq)
SELECT 'tournament', IFNULL(MAX(id), 0) FROM tournament;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'match', IFNULL(MAX(id), 0) FROM match;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'overlay_provider', IFNULL(MAX(id), 0) FROM overlay_provider;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'overlay_tournament', IFNULL(MAX(id), 0) FROM overlay_tournament;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'overlay_category', IFNULL(MAX(id), 0) FROM overlay_category;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'overlay_tournament_map', IFNULL(MAX(id), 0) FROM overlay_tournament_map;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'obs_connection', IFNULL(MAX(id), 0) FROM obs_connection;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'obs_scene', IFNULL(MAX(id), 0) FROM obs_scene;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'obs_recording_config', IFNULL(MAX(id), 0) FROM obs_recording_config;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'obs_recording_session', IFNULL(MAX(id), 0) FROM obs_recording_session;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'udp_server_config', IFNULL(MAX(id), 0) FROM udp_server_config;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'udp_server_session', IFNULL(MAX(id), 0) FROM udp_server_session;

  INSERT OR REPLACE INTO sqlite_sequence (name, seq)
  SELECT 'udp_client_connection', IFNULL(MAX(id), 0) FROM udp_client_connection;
-----------------------------------------------------------------------
-- 7. Post-migration triggers (timestamp maintenance)
-----------------------------------------------------------------------

PRAGMA recursive_triggers = OFF;

DROP TRIGGER IF EXISTS trg_athlete_update_timestamp;
CREATE TRIGGER trg_athlete_update_timestamp
AFTER UPDATE ON athlete
BEGIN
    UPDATE athlete SET updated_at = datetime('now') WHERE id = NEW.id;
END;

DROP TRIGGER IF EXISTS trg_match_update_timestamp;
CREATE TRIGGER trg_match_update_timestamp
AFTER UPDATE ON match
BEGIN
    UPDATE match SET updated_at = datetime('now') WHERE id = NEW.id;
END;

DROP TRIGGER IF EXISTS trg_event_statistic_update_timestamp;
CREATE TRIGGER trg_event_statistic_update_timestamp
AFTER UPDATE ON event_statistic
BEGIN
    UPDATE event_statistic SET updated_at = datetime('now') WHERE id = NEW.id;
END;

DROP TRIGGER IF EXISTS trg_event_unknown_update_timestamp;
CREATE TRIGGER trg_event_unknown_update_timestamp
AFTER UPDATE ON event_unknown
BEGIN
    UPDATE event_unknown SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-----------------------------------------------------------------------
-- 8. Compatibility views
-----------------------------------------------------------------------

DROP VIEW IF EXISTS athletes;
CREATE VIEW athletes AS
SELECT
    id,
    wt_id AS wtid,
    age_group_id AS look_age_group_id,
    division_id AS look_division_id,
    gender_id AS look_gender_id,
    weight_class_id AS look_weight_class_id,
    first_name,
    last_name,
    display_name,
    image,
    COALESCE(history, '[]') AS history,
    country,
    country_code,
    ioc_code,
    created_at,
    updated_at
FROM athlete;

DROP VIEW IF EXISTS pss_athletes;
CREATE VIEW pss_athletes AS
SELECT
    id,
    pss_code AS athlete_code,
    COALESCE(short_name, display_name) AS short_name,
    display_name AS long_name,
    country_code,
    flag_id,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM athlete
WHERE pss_code IS NOT NULL;

DROP VIEW IF EXISTS tournaments;
CREATE VIEW tournaments AS
SELECT
    id,
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
    organizing_committee AS oc,
    officials,
    banner,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM tournament;

DROP VIEW IF EXISTS pss_matches;
CREATE VIEW pss_matches AS
SELECT
    m.id,
    m.uuid,
    (SELECT uuid FROM tournament t WHERE t.id = m.tournament_id) AS tournament_id,
    m.match_code AS match_id,
    m.match_number,
    m.category,
    m.weight_class_code AS weight_class,
    m.division_code AS division,
    m.total_rounds,
    m.round_duration,
    m.countdown_type,
    m.format_type,
    m.creation_mode,
    m.created_at,
    m.updated_at,
    strftime('%s', m.created_at) AS created,
    strftime('%s', m.updated_at) AS updated
FROM match m;

DROP VIEW IF EXISTS pss_match_athletes;
CREATE VIEW pss_match_athletes AS
SELECT
    mp.id,
    m.uuid AS match_id,
    mp.athlete_id,
    CASE mp.side WHEN 'red' THEN 2 ELSE 1 END AS athlete_position,
    mp.bg_color,
    mp.fg_color,
    mp.created_at
FROM match_participant mp
JOIN match m ON m.id = mp.match_id;

DROP VIEW IF EXISTS pss_rounds;
CREATE VIEW pss_rounds AS
SELECT
    r.id,
    m.uuid AS match_id,
    r.round_number,
    r.start_time,
    r.end_time,
    r.duration_seconds AS duration,
    CASE r.winner_side WHEN 'red' THEN 2 WHEN 'blue' THEN 1 ELSE NULL END AS winner_athlete_position,
    r.created_at
FROM round r
JOIN match m ON m.id = r.match_id;

DROP VIEW IF EXISTS pss_scores;
CREATE VIEW pss_scores AS
SELECT
    s.id,
    m.uuid AS match_id,
    s.round_id,
    CASE s.side WHEN 'red' THEN 2 ELSE 1 END AS athlete_position,
    s.type AS score_type,
    s.value AS score_value,
    s.timestamp,
    s.tournament_uuid AS tournament_id,
    s.created_at,
    strftime('%s', s.created_at) AS created
FROM score s
JOIN match m ON m.id = s.match_id;

DROP VIEW IF EXISTS pss_warnings;
CREATE VIEW pss_warnings AS
SELECT
    ew.id,
    m.uuid AS match_id,
    ew.round_id,
    CASE ew.side WHEN 'red' THEN 2 ELSE 1 END AS athlete_position,
    ew.warning_type,
    ew.warning_count,
    ew.timestamp,
    ew.tournament_uuid AS tournament_id,
    ew.created_at,
    strftime('%s', ew.created_at) AS created
FROM event_warning ew
JOIN match m ON m.id = ew.match_id;

DROP VIEW IF EXISTS recorded_videos;
CREATE VIEW recorded_videos AS
SELECT
    v.id,
    v.match_id,
    v.event_id,
    v.tournament_uuid AS tournament_id,
    v.type AS video_type,
    v.file_path,
    v.directory AS record_directory,
    v.filename_formatting,
    v.start_time,
    v.duration_seconds,
    v.file_size,
    v.checksum,
    v.created_at,
    strftime('%s', v.created_at) AS created
FROM video v;

DROP VIEW IF EXISTS recorded_video_events;
CREATE VIEW recorded_video_events AS
SELECT
    ve.id,
    ve.video_id AS recorded_video_id,
    ve.event_id,
    ve.offset_ms,
    ve.created_at,
    strftime('%s', ve.created_at) AS created
FROM video_event ve;

DROP VIEW IF EXISTS ovr_providers;
CREATE VIEW ovr_providers AS
SELECT
    id,
    name,
    base_url,
    enabled,
    rate_limit_ms,
    last_refreshed_at,
    last_status,
    last_error,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM overlay_provider;

DROP VIEW IF EXISTS ovr_tournaments;
CREATE VIEW ovr_tournaments AS
SELECT
    id,
    provider_id,
    provider_tournament_id,
    name,
    start_date,
    end_date,
    city,
    country,
    url,
    status,
    last_seen_at,
    hash,
    etag,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM overlay_tournament;

DROP VIEW IF EXISTS ovr_categories;
CREATE VIEW ovr_categories AS
SELECT
    id,
    overlay_tournament_id AS tournament_id,
    discipline,
    age_group,
    gender,
    division,
    weight_class,
    bracket_stage,
    provider_raw,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM overlay_category;

DROP VIEW IF EXISTS ovr_to_local_tournament;
CREATE VIEW ovr_to_local_tournament AS
SELECT
    id,
    overlay_tournament_id AS ovr_tournament_id,
    tournament_id AS local_tournament_id,
    created_at,
    strftime('%s', created_at) AS created
FROM overlay_tournament_map;

DROP VIEW IF EXISTS ovr_flag_animations;
CREATE VIEW ovr_flag_animations AS
SELECT
    id,
    ioc_code,
    file_name,
    file_path,
    display_name,
    duration_ms,
    is_default,
    created_at,
    updated_at
FROM overlay_flag_animation;

DROP VIEW IF EXISTS ovr_anthems;
CREATE VIEW ovr_anthems AS
SELECT
    id,
    ioc_code,
    file_name,
    file_path,
    display_name,
    duration_ms,
    is_default,
    created_at,
    updated_at
FROM overlay_anthem;

DROP VIEW IF EXISTS obs_connections;
CREATE VIEW obs_connections AS
SELECT
    id,
    name,
    host,
    port,
    password,
    is_active,
    created_at,
    updated_at,
    status,
    error,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM obs_connection;

DROP VIEW IF EXISTS obs_recording_sessions;
CREATE VIEW obs_recording_sessions AS
SELECT
    id,
    obs_connection_name,
    tournament_id,
    match_code AS match_id,
    match_number,
    player1_name,
    player1_flag,
    player2_name,
    player2_flag,
    recording_path,
    recording_filename,
    recording_start_time,
    recording_end_time,
    recording_duration,
    recording_size_bytes,
    replay_buffer_start_time,
    replay_buffer_end_time,
    replay_buffer_saved,
    replay_buffer_filename,
    status,
    error_message,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM obs_recording_session;

DROP VIEW IF EXISTS obs_scenes;
CREATE VIEW obs_scenes AS
SELECT
    id,
    scene_name,
    scene_id,
    is_active,
    last_seen_at,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM obs_scene;

DROP VIEW IF EXISTS udp_server_configs;
CREATE VIEW udp_server_configs AS
SELECT
    id,
    name,
    port,
    bind_address,
    network_interface_id,
    enabled,
    auto_start,
    max_packet_size,
    buffer_size,
    timeout_ms,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM udp_server_config;

DROP VIEW IF EXISTS udp_server_sessions;
CREATE VIEW udp_server_sessions AS
SELECT
    id,
    server_config_id,
    start_time,
    end_time,
    status,
    packets_received,
    packets_parsed,
    parse_errors,
    total_bytes_received,
    average_packet_size,
    max_packet_size_seen,
    min_packet_size_seen,
    unique_clients_count,
    error_message,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM udp_server_session;

DROP VIEW IF EXISTS udp_client_connections;
CREATE VIEW udp_client_connections AS
SELECT
    id,
    server_session_id AS session_id,
    client_address,
    client_port,
    first_seen,
    last_seen,
    packets_received,
    total_bytes_received,
    is_active,
    strftime('%s', created_at) AS created
FROM udp_client_connection;

DROP VIEW IF EXISTS pss_events;
CREATE VIEW pss_events AS
SELECT
    e.id,
    e.session_id,
    e.match_id,
    e.round_id,
    e.event_type_id,
    e.timestamp,
    e.raw_data,
    e.parsed_data,
    e.event_sequence,
    e.processing_time_ms,
    e.is_valid,
    e.error_message,
    e.recognition_status,
    e.protocol_version,
    e.parser_confidence,
    e.validation_errors,
    e.tournament_uuid AS tournament_id,
    e.created_at,
    strftime('%s', e.created_at) AS created
FROM event e;

DROP VIEW IF EXISTS pss_event_types;
CREATE VIEW pss_event_types AS
SELECT
    id,
    code AS event_code,
    name AS event_name,
    description,
    category,
    is_active,
    created_at,
    strftime('%s', created_at) AS created
FROM event_type;

DROP VIEW IF EXISTS pss_event_details;
CREATE VIEW pss_event_details AS
SELECT
    id,
    event_id,
    detail_key,
    detail_value,
    detail_type,
    created_at
FROM event_detail;

DROP VIEW IF EXISTS pss_event_statistics;
CREATE VIEW pss_event_statistics AS
SELECT
    id,
    session_id,
    event_type_id,
    total_events,
    recognized_events,
    unknown_events,
    partial_events,
    deprecated_events,
    validation_errors,
    parsing_errors,
    average_processing_time_ms,
    min_processing_time_ms,
    max_processing_time_ms,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM event_statistic;

DROP VIEW IF EXISTS pss_event_validation_rules;
CREATE VIEW pss_event_validation_rules AS
SELECT
    id,
    event_code,
    protocol_version,
    rule_name,
    rule_type,
    rule_definition,
    error_message,
    is_active,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM event_validation_rule;

DROP VIEW IF EXISTS pss_event_validation_results;
CREATE VIEW pss_event_validation_results AS
SELECT
    id,
    event_id,
    rule_id,
    validation_passed,
    error_message,
    validation_time_ms,
    created_at,
    strftime('%s', created_at) AS created
FROM event_validation_result;

DROP VIEW IF EXISTS pss_event_recognition_history;
CREATE VIEW pss_event_recognition_history AS
SELECT
    id,
    event_id,
    old_status,
    new_status,
    changed_by,
    change_reason,
    protocol_version,
    raw_data,
    parsed_data,
    created_at,
    strftime('%s', created_at) AS created
FROM event_recognition_history;

DROP VIEW IF EXISTS pss_unknown_events;
CREATE VIEW pss_unknown_events AS
SELECT
    id,
    session_id,
    raw_data,
    first_seen,
    last_seen,
    occurrence_count,
    pattern_hash,
    suggested_event_type,
    notes,
    created_at,
    updated_at,
    strftime('%s', created_at) AS created,
    strftime('%s', updated_at) AS updated
FROM event_unknown;

-----------------------------------------------------------------------
-- 9. Temp cleanup
-----------------------------------------------------------------------

DROP TABLE IF EXISTS tmp_match_map;
DROP TABLE IF EXISTS tmp_athlete_map;

PRAGMA recursive_triggers = ON;




