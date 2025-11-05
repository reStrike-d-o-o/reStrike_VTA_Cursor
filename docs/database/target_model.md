# Target Schema Blueprint (Prefix-Free, Normalized)

This blueprint defines the canonical schema that replaces the current mixture of `pss_*`, `ovr_*`, `ivr_*`, and `look_*` tables. The goals are to provide a single source of truth for roster, scheduling, event, overlay, and ceremony data; remove subsystem prefixes; and align the backend with an async-friendly ORM (SeaORM).

## Naming & Conventions

- Tables use singular nouns (`athlete`, `match`, `tournament`) or descriptive groupings (`match_participant`, `event_type`). Prefixes are avoided unless the subsystem is intentionally isolated (for example `overlay_*`).
- Primary keys are surrogate integers (`INTEGER PRIMARY KEY AUTOINCREMENT`). UUID columns are added when cross-module references are required.
- Each table exposes `created_at` and `updated_at` columns with ISO-8601 timestamps. `AFTER UPDATE` triggers keep `updated_at` in sync; integer mirrors (`created`, `updated`) are eliminated.
- Foreign keys follow the `<entity>_id` pattern and reference the surrogate keys unless a UUID is expressly required.
- Side/position values are stored as constrained text (`CHECK (side IN ('blue','red'))`) rather than integer codes.

## Canonical Tables

| New table | Derived from | Purpose & key columns |
| --- | --- | --- |
| `athlete` | `athletes` + `pss_athletes` | Unified roster (`uuid`, `wt_id`, `pss_code`, lookup FKs, `flag_id`). |
| `flag` | `flags` | Nation/flag catalog; retains recognition metadata. |
| `match` | `pss_matches` | Match metadata with `match_code` (legacy `match_id`), `uuid`, tournament linkage, timing config. |
| `match_participant` | `pss_match_athletes` | Athlete assignments per match with normalized `side` (`blue`/`red`). |
| `round` | `pss_rounds` | Round snapshots including `winner_side`, timing, and match FK. |
| `score` | `pss_scores` | Scoring breakdown per round and side. |
| `event` | `pss_events` | Raw event stream keyed by `event_type_id`, `match_id`, `timestamp`. |
| `event_type` | `pss_event_types` | Master list of event codes/descriptions. |
| `event_detail` | `pss_event_details` | Key/value payloads attached to an `event`. |
| `event_statistic` | `pss_event_statistics` | Aggregated metrics per session/event type. |
| `event_warning` | `pss_warnings` | Validation or parser warnings per match. |
| `event_unknown` | `pss_unknown_events` | Unclassified packets captured during ingest. |
| `video` | `recorded_videos` | Recorded media assets tied to matches. |
| `video_event` | `recorded_video_events` | Timeline markers inside a recorded video. |
| `tournament` | `tournaments` | Tournament metadata, contacts, branding. |
| `tournament_day` | `tournament_days` | Daily schedule blocks with status and timing. |
| `tournament_ranking` | `tournament_rankings` | Ranking code/label catalog (includes para flag). |
| `tournament_champion` | `tournament_champions` | Medal outcomes linked to canonical matches. |
| `medal_ceremony` | `medal_ceremonies` | Ceremony configuration per tournament. |
| `medal_ceremony_division` | `medal_ceremony_divisions` | Ceremony order entries referencing divisions. |
| `medal_ceremony_medalist` | `medal_ceremony_medalists` | Podium allocations referencing canonical athletes. |
| `octagon` | `octagons` | Physical rings/stages tied to tournaments and tournament days. |
| `division` | `look_divisions` | Division lookup (with gender/age references). |
| `gender` | `look_genders` | Gender lookup. |
| `weight_class` | `look_weight_classes` | Weight class lookup. |
| `age_group` | `look_age_groups` | Age group lookup. |
| `round_config` | `look_round_configs` | Round timing/config presets. |
| `discipline` | `look_disciplines` | Discipline lookup. |
| `overlay_*` tables | `ovr_*` family | Normalised overlay provider/category/asset tables. |
| `animation`, `anthem` tables | Legacy media assets retained with neutral names. |
| `udp_*` tables | `udp_*` family | UDP server/session/client tracking with singular names. |
| `obs_*` tables | `obs_*` family | OBS connection/session metadata with singular names. |

## Compatibility Views

Legacy queries keep functioning via read-only views that project the canonical tables back to their former names. Example:

```sql
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
```

Views can be removed once all Rust and TypeScript code paths consume the canonical names.

## Migration Outline

1. **Athlete merge** – build temporary mapping tables (`tmp_athlete_map`, `tmp_match_map`) and merge `athletes` plus `pss_athletes` into `athlete`, stitching `flag_id` from IOC codes when possible.
2. **Match remap** – copy `pss_matches` into `match`, convert `match_id` -> `match_code`, ensure every row has a UUID, and link to canonical tournaments.
3. **Participant remap** – rewrite `pss_match_athletes` into `match_participant`, convert numeric sides to `red`/`blue`, and log unresolved athlete references in `migration_missing_participants` for manual follow-up.
4. **Lookup & ceremony tables** – rename `look_*`, `medal_ceremony_*`, and `octagons` tables to neutral names, update foreign keys, and populate `tournament_day`, `medal_ceremony`, `medal_ceremony_division`, `medal_ceremony_medalist`, and `octagon` from their legacy counterparts.
5. **Event/overlay/media tables** – copy the remaining `pss_*`, `ovr_*`, `recorded_*`, `obs_*`, and `udp_*` structures into their canonical tables, normalising column names in the process.
6. **Trigger installation** – create `AFTER UPDATE` timestamp triggers for every canonical table so `updated_at` stays current without app-side logic.
7. **View creation** – define compatibility views for every renamed table (`pss_*`, `ovr_*`, `medal_ceremony_*`, etc.) to keep the application running until the refactor is complete.

## ORM Strategy

- Adopt **SeaORM** to model each canonical table as a Rust entity (`entity/src/*.rs`), enabling async database access under Tokio.
- Use `sea-orm-cli migrate` to generate future schema changes; keep the generated migrations in sync with the SQL reference in `docs/database/schema.sql`.
- Replace existing raw SQL (daedo importer, UDP ingest, OBS persistence, IVR tooling) with repository-style SeaORM queries so the codebase relies solely on the canonical schema.

## Mapping Reference

| Legacy table | Canonical table | Notes |
| --- | --- | --- |
| `pss_matches` | `match` | `match_id` renamed to `match_code`; UUIDs retained/backfilled. |
| `pss_match_athletes` | `match_participant` | `athlete_position` converted to `side`. |
| `pss_athletes` | `athlete` | Merged into canonical roster with `pss_code`. |
| `pss_events` | `event` | Column names preserved; foreign keys updated. |
| `pss_event_types` | `event_type` | Direct rename. |
| `pss_scores` | `score` | Direct rename with side normalization. |
| `pss_rounds` | `round` | Direct rename with `winner_side`. |
| `pss_warnings` | `event_warning` | Direct rename with `side`. |
| `pss_unknown_events` | `event_unknown` | Direct rename; timestamps normalized. |
| `recorded_videos` | `video` | Direct rename. |
| `recorded_video_events` | `video_event` | Direct rename. |
| `tournaments` | `tournament` | Adds UUID backfill and JSON fields for contacts/location. |
| `tournament_days` | `tournament_day` | Same columns, normalized timestamps. |
| `tournament_rankings` | `tournament_ranking` | Same columns, stricter timestamp defaults. |
| `tournament_champions` | `tournament_champion` | `winner_color` -> `winner_side`; `match_id` -> `match_code`. |
| `medal_ceremonies` | `medal_ceremony` | Ceremony FK points to canonical tournament. |
| `medal_ceremony_divisions` | `medal_ceremony_division` | Division FK points to canonical lookup. |
| `medal_ceremony_medalists` | `medal_ceremony_medalist` | Athlete FK points to canonical roster. |
| `octagons` | `octagon` | Column `octagon_number` renamed to `number`. |
| `ovr_*` tables | `overlay_*` | Prefix removed; foreign keys updated to canonical IDs. |
| `obs_*` tables | `obs_*` | Singular naming; timestamp triggers added. |
| `udp_*` tables | `udp_*` | Singular naming; timestamp triggers added. |

## Deliverables

- `scripts/db_migrations/20251105_schema_unification.sql` – end-to-end migration script (data copy, triggers, views).
- `docs/database/schema.sql` – canonical schema reference generated from the migrated database (pending SeaORM scaffolding).
- `docs/database/rename_dictionary.md` – running lookup for table/column rename mappings.
- Updates to `AGENTS.md` to reference the canonical schema documentation.

This blueprint is the authoritative reference for ongoing refactors. Any deviations discovered during sandbox testing should be recorded here before the production migration is attempted.
