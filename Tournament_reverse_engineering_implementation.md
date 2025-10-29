# Tournament Reverse Engineering & Replay Implementation

## Overview
- **Objective**: reconstruct complete tournaments from historical Daedo PSS logs so they can be replayed for automated QA, module validation, and workflow demos.
- **Scope**: ingest raw PSS artefacts, map them to the existing database schema (tournaments, matches, athletes, events, medals), and enable deterministic replays through the automatic simulation module.
- **Status**: _initiated (2025-10-28)_ – discovery and data-structure mapping in progress.

## Data Sources
- **Primary logs**: `C:\Users\Damjan\Documents\Daedo log files GO2025`
  - Matched file pairs per bout (`*-matchLog.csv` / `*-matchLogItems.csv`).
  - Folder hierarchy encodes tournament/day/court metadata (to be documented).
- **Supporting references**
  - WT division dataset imported earlier (migration 43).
  - IOC flag/anthem mapping (Reports in `ui/build/assets/flags`).
  - Existing DB schema (`pss_*`, `tournaments`, `medal_ceremonies`, `ovr_*`).

## Workstream Breakdown
- [ ] **Directory taxonomy**
  - 2025-10-28: Confirmed archive root holds `Court01`-`Court12` directories. Each court contains day folders named `YYYYMMDD` (observed `20250913`, `20250914`). Day folders contain per-bout CSV pairs named `YYYYMMDDhhmmss-<court-sequence>-matchLog(.csv|Items.csv)`; timestamp portion matches the `matchStartTime` embedded in the file.
  - 2025-10-28: Observed optional companion exports (PDF/XLS) using the same prefix. No timezone offset detected so far; filename timestamp aligns with local wall-clock encoded in the payload.
- [ ] **Log schema reverse engineering**
  - 2025-10-28: Documented `matchLog.csv` headers. Columns 1-24 cover bout metadata (start/end, match number, division, sensor thresholds). Columns 25-34 hold timing config under `roundsConfig.*`. Winner block (`matchWinner`, `matchWinnerBy`, `matchResult`, `matchVictoryCriteria`) precedes `roundsWinners`, which stores JSON with doubled quotes and no CSV quoting. Golden point tie-breaker metrics follow. Trailing columns repeat division metadata and append athlete identity plus video quotas, concluding with punch configuration fields.
  - 2025-10-28: Documented `matchLogItems.csv` structure. Timeline columns (`eventTime`, `roundTime`, `systemTime`) accompany `matchLogItemType` codes (`START_MATCH`, `BLUE_BODY_HIT`, `BLUE_BODY_POINT`, etc.). Scoring occurs as HIT/POINT row pairs sharing timestamps; the second `entryValue` column captures sensor ids or literal `SENSOR`. Running totals (`score`, `bluePoints`, `redPoints`, penalties, quotas) update per row. `goldenPointRound` stores string booleans.
  - 2025-10-29: Parsed entire GO2025 archive (910 bouts) and catalogued 66 distinct `matchLogItemType` codes. `matchVictoryCriteria` is always `BESTOF3`; `matchWinnerBy` distribution skews to `PTF` (95%) with isolated `WDR`, `PTG`, `RSC`, `SUP`, `PUN`. Golden-point tie breaker flag (`haveTieBreaker`) is uniformly `false` across the dataset.
- [ ] **Canonical dataset definition**
  - Establish per-match JSON model capturing metadata, round flow, scoring, penalties, timeline.
  - Map to DB entities (matches, match athletes, events, scores, warnings).
  - 2025-10-28: Drafted mapping between matchLog/matchLogItems payloads and existing `pss_*` tables (see Canonical Mapping Draft).
  - 2025-10-29: Implemented canonical match payload generator (`scripts/tournament/daedo_log_parser.py extract`), producing structured JSON with metadata, rules config, athlete identities, tie-breaker metrics, sensor thresholds, and fully typed event timeline snapshots.
  - 2025-10-29: Enhanced canonical events with inferred WT UDP mapping (streams + argument list) and preserved hit/penalty metadata to support deterministic replay packet reconstruction.
- [ ] **Transformation pipeline**
  - Parsing scripts (Python/Rust) to read raw logs, normalise, and output canonical format.
  - Validation checks (score totals, round winners, clock consistency).
  - 2025-10-29: Added Python tooling (`scripts/tournament/daedo_log_parser.py`) with brace-aware CSV parsing, duplicate header handling, ISO timestamp conversion, archive summarisation mode, structured logging (`--log-level`), and UDP-packet annotations for each event.
- [ ] **Database ingestion**
  - Migration-safe loader to populate tournaments/days/matches and link to athletes/divisions.
  - Idempotent operations keyed by match uuid / timestamp hash.
- [ ] **Replay integration**
  - Extend simulation module to stream canonical events with controllable tempo.
  - UI/UX hooks for selecting matches/divisions and monitoring playback state.
- [ ] **QA & Documentation**
  - Automated tests for parser & loader.
  - Operational guide covering dataset prep, replay commands, troubleshooting.

## Recently Completed / Notes
- _2025-10-28_: catalogued directory naming scheme and documented matchLog/matchLogItems field groups for Court01 sample.
- _2025-10-28_: created initiative document; scanning archive structure in progress.
- _2025-10-28_: sampled `Court01/20250913/20250913091016-101` pair to decode CSV semantics.
- _2025-10-28_: added Migration45 (tournament metadata + rankings/octagons/athletes tables) and seeded tournament rankings lookup.
- _2025-10-28_: seeded "German Open - Hamburg 2025" via `cargo run --bin seed_german_open` (location/contact/OC JSON, banner, days, and Court01-Court12 octagons per day).
- _2025-10-29_: Parser now logs operations, enriches timeline events with UDP reconstruction metadata, and supports archive summaries with streaming-aware statistics.
- _2025-10-29_: Scripted archive scan confirms 910 match pairs, enumerated event taxonomy, and produced canonical JSON output for downstream ingestion experiments.

### Sample Match Dataset (Court01 · 2025-09-13 · Match 101)
```json
{
  "match_id": "20250913091016-101",
  "metadata": {
    "start": "13/09/2025 09:10:16:194",
    "end": "13/09/2025 09:15:12:729",
    "phase": "Round of 32",
    "division": "JUNIORS",
    "category": "W -46KG",
    "gender": "FEMALE",
    "winner": "BLUE",
    "victory": "BESTOF3",
    "result": "13-2"
  },
  "config": {
    "rounds": 3,
    "roundTime": "2:00",
    "restTime": "1:00",
    "kyeShi": "1:00",
    "goldenPointEnabled": false,
    "maxGamJeoms": 5
  },
  "athletes": {
    "blue": {"name": "O. ANDRZEJEWSKA", "wtId": "854", "nation": "POL"},
    "red": {"name": "A. G. PADUA", "wtId": "89", "nation": "CAN"}
  },
  "roundResults": {
    "roundResults": [
      {"round": 1, "roundWinner": "BLUE", "roundResultDecision": "R-PTF"},
      {"round": 2, "roundWinner": "TIE", "roundResultDecision": "R-PTF"},
      {"round": 3, "roundWinner": "TIE", "roundResultDecision": "R-PTF"}
    ]
  },
  "events": [
    {"t": 1757747416194, "type": "START_MATCH"},
    {"t": 1757747426941, "type": "BLUE_BODY_HIT", "raw": "7"},
    {"t": 1757747477170, "type": "BLUE_BODY_POINT", "delta": "2", "score": "2-0"},
    {"t": 1757747485738, "type": "BLUE_HEAD_POINT", "delta": "3", "score": "7-0"},
    {"t": 1757747512844, "type": "RED_BODY_HIT"},
    {"t": 1757747552095, "type": "END_ROUND"},
    {"t": "...", "type": "…additional events for rounds 2–3"}
  ]
}
```

### Key Parsing Findings
- `matchLog.csv` lines embed JSON (`roundsWinners`) without CSV quoting; a brace-aware tokenizer is required to avoid column shifts.
- Per-row metadata repeats discipline fields after `paraTkdMatch`, confirming distinct pre-post calibration settings.
- `matchLogItems.csv` pairs sensor hits with scoring deltas (e.g., `BLUE_BODY_HIT` immediately followed by `BLUE_BODY_POINT`) while maintaining running totals, penalties, and video quota state.
- Event timestamps (`eventTime`, `systemTime`) use 13-digit epoch-style values; `roundTime` is a declining counter relative to round start.
- Golden point flags stay boolean strings (`"false"`/`"true"`); normalization needed during ingestion.
- Duplicate column names inside `matchLogItems.csv` (`entryValue`) require suffixing during parsing; the auxiliary value carries judge/sensor origins for paired scoring events.
- ISO8601 conversion of epoch millisecond fields aligns with local start/end times encoded in filenames (`YYYYMMDDhhmmss`), confirming consistent timezone handling.

### Dataset Scan Snapshot (2025-10-29)
- **Coverage**: 910 bout pairs across Courts 01-12 (`summarize` mode completes in ~4 seconds on local SSD).
- **Phases**: Round of 16 (314), Round of 32 (212), Quarterfinals (208), Semifinals (108), Finals (58), Round of 64 (10).
- **Victory paths**: `PTF` dominates (864); rarer outcomes include `WDR` (28), `PTG` (10), `RSC` (5), `SUP` (2), `PUN` (1).
- **Event taxonomy**: 66 unique `matchLogItemType` codes. Top volume signals are `TIMEOUT` (14,745), `RESUME` (13,940), body hits/points (~16k combined), and head hits/points (~3k combined). Golden-point prefixed events appear but tie-breaker activation stays `false`.
- **Punch config**: All matches run `SEMI_AUTO` mode with punch enabled, reinforcing baseline calibration defaults.
- **Video reviews**: Quota change/request events occur sparsely (≤53 per code); both central review (`CR_*`) and coach review flows represented.

### PSS UDP Schema Reference (2025-10-29)
- Source specification: `protocol/pss_schema.txt` (WT UDP v2.3, 2024 edition) – covers broadcast streams for scoring, infractions, timing, IVR, and winner presentation.
- **Points & hits**: `pt1/pt2` map to body/head/punch deltas; `hl1/hl2` mirror sensor hit levels. These align with `BLUE_*` / `RED_*` HIT/POINT rows in `matchLogItems.csv`, confirming sequencing (hit → point).
- **Warnings**: `wg1/wg2` streams correspond to Gam-Jeom adjustments captured as `*_ADD/REMOVE_GAME_JEON` events; totals reconcile with `blueTotalPenalties` / `redTotalPenalties`.
- **Injury & breaks**: `ij*` (doctor/kye-shi) and `brk` (inter-round clock) explain `DOCTOR`, `KYE_SHI`, and intermission events in timeline logs.
- **IVR/Challenge**: `ch0/ch1/ch2` streams map to `*_VIDEO_REQUEST`, `*_VIDEO_QUOTA_*`, and `CR_*` decisions.
- **Round winners & match winner**: `wrd` packets deliver per-round outcomes matching the `roundsWinners` JSON, while `wmh` aligns with terminal `MATCH_FINISHED` context (`matchResult` string formatting).
- **Clock control**: `clk`, `rnd`, `start/stop` semantics underpin the `TIMEOUT`/`RESUME` cadence seen throughout the logs.
- **Load/ready lifecycle**: `FightLoaded`, athlete metadata (`at1/at2`), and configuration payloads (`mch`, `sc*`, `wg*`) provide external keys that we can infer from filename + metadata when recreating sessions.

## Tournament Reconstruction Plan (2025-10-29)

Goal: recreate the complete German Open 2025 tournament inside the existing schema and enable deterministic UDP replay that mirrors the original Daedo broadcasts.

### Guiding Principles
- **Stay compatible**: populate current tables (`tournaments`, `tournament_days`, `octagons`, `athletes`, `pss_matches`, `pss_match_athletes`, `pss_rounds`, `pss_scores`, `pss_warnings`, `pss_events`, `pss_event_details`) instead of inventing new structures.
- **Replay fidelity**: each stored event must carry enough metadata to reconstruct WT UDP packets (`pt*`, `hl*`, `wg*`, `wrd`, `wmh`, etc.) in order. Replay loops will stream these events back to our UDP server using original timestamps.
- **Logging & hygiene**: when adding code, hook into the existing logging framework, remove dead utilities, and keep modules tidy.

### Task Order (sequential, update doc after each)
1. **Canon data finalization** – enhance the parser output to include UDP stream identifiers, argument payloads, and logging; verify edge cases (DQ, GP, para flags).
2. **Tournament scaffold seeding** – ensure tournament metadata, days, octagons, and athlete roster are generated/updated idempotently using DB operations with structured logging.
3. **Match population** – create `pss_matches`, `pss_match_athletes`, `pss_rounds`, `pss_scores`, `pss_warnings` per log bundle; validate relationships and timestamps.
4. **Timeline ingestion** – insert full `pss_events` (and `pss_event_details` where needed) using canonical payloads so replay services can rebuild UDP messages exactly.
5. **Replay bridge** – design the replay orchestrator: query events in order, rebuild UDP packets, respect timing controls, and integrate logging hooks.
6. **Validation & cleanup** – run verification scripts/tests, prune obsolete code paths, and document any residual risks or follow-ups.

We will only move to the next task when the current one is complete, verified, and captured in this document.

### Canonical Mapping Draft (2025-10-28)
- Tournament metadata from folder hierarchy maps to `tournaments` (name/ranking already seeded), `tournament_days` (date folder), and `octagons` (court id). File prefix `YYYYMMDDhhmmss-###` can become `external_match_key`.
- `matchLog.csv` metadata populates `pss_matches` (timings, phase, victory method, result) and `pss_match_athletes` via blue/red sections. Sensor thresholds inform calibration JSON stored alongside match configuration.
- `roundsConfig.*` and `roundsWinners` JSON consolidate into a `pss_match_rounds` payload referenced by `pss_matches.rounds_state`.
- `matchLogItems.csv` rows translate to `pss_events` timeline entries; HIT vs POINT rows captured as distinct event types but share a derived `sequence_group_id` when timestamps match.
- Aggregated score columns (`bluePoints`, `redPoints`, penalties) yield periodic snapshots for `pss_scores` while deltas feed `pss_event_statistics`.
- Video quota counters map to new match properties controlling review availability (to confirm with simulation module).

## Tournament Data Model Expansion

### Core Tables & Relationships
- **tournaments**
  - New fields: `ranking_id` (FK → `tournament_rankings`), `location` JSON, `contact` JSON, `oc` JSON, `officials` JSON, `banner` (base64 TEXT).
  - `status` values standardized to `pending|running|ended` (legacy `active` mapped to `running`).
  - JSON defaults: `location {}`, `contact {}`, `oc {}`, `officials {}`; app enforces max banner size 2 MB.
  - Location structure example:
    ```json
    {
      "venue": "Sporthalle Hamburg",
      "address": "Krochmannstraße 55",
      "postal_code": "22297",
      "city": "Hamburg",
      "country": "Germany"
    }
    ```
  - Contact JSON: `{ "person": "", "web": "", "email": "", "mob_phone": "" }`.
  - Organizing committee JSON: `{ "governing_org": "", "persons": [ { "name": "", "role": "", "email": "", "phone": "" } ], "contacts": [ ... ] }`.
  - Officials JSON: `{ "TD": { ... }, "RC": { ... }, "others": [ ... ] }`.

- **tournament_rankings** (lookup)
  - Seed list covers WT & Para tiers: LOCAL, NATIONAL, OPEN, WT_CHALLENGE, G1, G2, G3, G4, G6 (GP), G8 (GP Final), E1, E2, E3, plus P_G1, P_G2, P_G6.
  - Columns: `id`, `code`, `label`, optional `is_para`, timestamps.

- **tournament_days**
  - Unique constraint on (`tournament_id`, `date`); statuses align with tournaments.
  - German Open seed: Day 1 `2025-09-13`, Day 2 `2025-09-14`.

- **octagons** (per-day court assignments)
  - Columns: `id`, `tournament_id`, `tournament_day_id`, `octagon_number` (TEXT), timestamps.
  - Unique index on (`tournament_day_id`, `octagon_number`) to reflect per-day uniqueness.

- **athletes**
  - Columns: `wtid`, lookup FK ids (`look_age_group_id`, `look_division_id`, `look_gender_id`, `look_weight_class_id`), `first_name`, `last_name`, `display_name`, `image` base64 (≤ 2 MB), `history` JSON (default `[]`), `country`, `country_code`, `ioc_code` (NOT NULL), timestamps.
  - History entries appended as `{ "changed_at": ISO8601, "changes": { "field": { "old": "", "new": "" } } }`.
  - Indexes: unique on `wtid` when populated; secondary on (`ioc_code`, `display_name`).

### Migration Plan
1. Alter `tournaments` table to add new JSON/text fields and FK; normalize `status`.
2. Create `tournament_rankings` with seed dataset.
3. Create `octagons` with FK links and per-day unique constraint.
4. Create `athletes` roster table with indices.
5. Enforce `tournament_days` uniqueness and status normalization.

### German Open – Hamburg 2025 Seed Data
- Tournament: name `"German Open - Hamburg 2025"`, ranking `G2`, location JSON per venue info, contact/OC/officials JSON following above schemas, banner encoded from `C:\Users\Damjan\Downloads\go2025.jpg`.
- Days: 2025-09-13 (Day 1), 2025-09-14 (Day 2).
- Octagons: Court01–Court12 mapped to per-day entries (`"1"` … `"12"` or actual labels if provided).
- Athletes: to be ingested from log parsing; history arrays start empty (`[]`).
- **Archive structure observations (2025-10-28)**
  - Root contains `Court01` … `Court12` directories ⇒ physical octagon assignments.
  - Each court folder holds date-stamped subdirectories (e.g., `20250913`, `20250914`) ⇒ competition day.
  - Date folders contain artefact triples per bout:
    - `YYYYMMDDhhmmss-XXX-matchLog.csv` (metadata summary).
    - `YYYYMMDDhhmmss-XXX-matchLogItems.csv` (event timeline).
    - Optional PDF/XLS exports sharing the same prefix.
  - Filename components:
    - Leading 14-digit timestamp = match start (local time, `YYYYMMDDhhmmss`).
    - Middle numeric id (e.g., `101`) = court-local match number.
    - Suffix denotes content type (`matchLog`, `matchLogItems`).
