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
    uuid, wt_id, short_name, display_name, first_name, last_name,
    country_code, ioc_code, gender_id, division_id, weight_class_id, age_group_id,
    image, history, created_at, updated_at
)
SELECT
    lower(hex(randomblob(16))),
    NULLIF(la.wtid, ''),
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
    uuid, wt_id, pss_code, short_name, display_name,
    country_code, ioc_code, created_at, updated_at
)
SELECT
    lower(hex(randomblob(16))),
    NULL,
    lpa.athlete_code,
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
-- 6. Housekeeping
----------------------------------------------------------------------

-- Refresh sqlite_sequence for tables with explicit id inserts
INSERT OR REPLACE INTO sqlite_sequence (name, seq)
SELECT 'tournament', IFNULL(MAX(id), 0) FROM tournament;

INSERT OR REPLACE INTO sqlite_sequence (name, seq)
SELECT 'match', IFNULL(MAX(id), 0) FROM match;

COMMIT;

PRAGMA foreign_keys = ON;
