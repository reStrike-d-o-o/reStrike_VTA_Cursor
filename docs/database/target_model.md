# Target Schema Blueprint (Prefix-Free, Normalized)

This document defines the canonical schema that will replace the current mix of `pss_*`, `ovr_*`, `ivr_*`, and `look_*` tables. The goals are:

- Establish a **single source of truth** for roster, match, tournament, and media data.
- Remove legacy prefixes from fundamental tables and standardise naming conventions.
- Enforce timestamps and foreign keys with triggers instead of manual integer mirrors.
- Support a modern ORM (SeaORM) for the Rust backend.

All table and column names below are the definitive targets. Legacy names will be preserved as read-only compatibility views during the transition.

## Naming & Conventions

- Table names are singular nouns (`athlete`, `match`, `tournament`) or descriptive collections (`match_participant`, `event_type`). Avoid prefixes except when the subsystem is genuinely isolated (e.g. `overlay_asset`).
- Primary keys:
  - Surrogate integer `id` (`INTEGER PRIMARY KEY AUTOINCREMENT`) for local references.
  - UUID columns (`uuid TEXT NOT NULL UNIQUE`) where cross-module references are required.
- Timestamp policy:
  - Every table exposes `created_at TEXT NOT NULL DEFAULT (datetime('now'))` and `updated_at TEXT NOT NULL DEFAULT (datetime('now'))`.
  - `BEFORE UPDATE` triggers set `updated_at = datetime('now')`. No additional integer columns (`created`, `updated`) exist.
- Foreign keys use the `<entity>_id` pattern and reference the integer surrogate keys unless UUID is explicitly required.
- Side/position enumerations use constrained TEXT with check constraints (`CHECK (side IN ('blue', 'red'))`) instead of integers.

## Canonical Tables

| New table | Derived from | Purpose & key columns |
| --- | --- | --- |
| `athlete` | `athletes` ⊔ `pss_athletes` | Canonical roster. Columns: `id`, `uuid`, `wt_id`, `short_name`, `display_name`, `country_code`, `ioc_code`, `flag_id`, lookup FKs (`gender_id`, `division_id`, `weight_class_id`). |
| `flag` | `flags` | Flag catalog (unchanged name, add trigger to sync `country_code`/`ioc_code`). |
| `match` | `pss_matches` | Match metadata. Columns: `id`, `uuid`, `tournament_id`, `match_code` (former `match_id`), `match_number`, `category`, `division_id`, `weight_class_id`, `status`, `created_at`, `updated_at`. |
| `match_participant` | `pss_match_athletes` | Links athletes to matches. Columns: `id`, `match_id` (FK), `athlete_id` (FK), `side` (`blue`/`red`), `bg_color`, `fg_color`. `(match_id, side)` unique. |
| `round` | `pss_rounds` | Round snapshots: `match_id`, `round_number`, `duration_seconds`, `winner_side`. |
| `score` | `pss_scores` | Scoring breakdown: `match_id`, `round_id`, `side`, `score_type`, `value`, `timestamp`. |
| `event` | `pss_events` | Raw event stream with references to `event_type` and `match_id`. |
| `event_type` | `pss_event_types` | Master list of event codes. |
| `event_detail` | `pss_event_details` | Key/value metadata for events. |
| `event_statistic` | `pss_event_statistics` | Derived stats; retains same structure. |
| `event_warning` | `pss_warnings` | Warning records per match. |
| `video` | `recorded_videos` | Recorded assets tied to matches. |
| `video_event` | `recorded_video_events` | Breakdowns within a recorded video. |
| `tournament` | `tournaments` | Tournament metadata. |
| `tournament_day` | `tournament_days` | Daily schedule slots. |
| `tournament_ranking` | `tournament_rankings` | Ranking tables. |
| `tournament_champion` | `tournament_champions` | Medal summaries. |
| `division` | `look_divisions` | Lookup values; include `code`, `name`, `age_group_id`, `gender_id`. |
| `gender` | `look_genders` | Gender lookup. |
| `weight_class` | `look_weight_classes` | Lookup. |
| `age_group` | `look_age_groups` | Lookup. |
| `round_config` | `look_round_configs` | Lookup. |
| `discipline` | `look_disciplines` | Lookup. |
| `overlay_*` tables | `ovr_*` families | Rename to `overlay_provider`, `overlay_category`, `overlay_asset`, etc. |
| `animation`, `anthem` tables | keep but align naming style (`animation`, `anthem_asset`). |

### Compatibility Views

For modules not yet migrated, create views (read-only) mapping old names to new tables. Examples:

```sql
CREATE VIEW pss_matches AS SELECT * FROM match;
CREATE VIEW pss_match_athletes AS
SELECT id, match_uuid AS match_id, athlete_id, CASE side WHEN 'blue' THEN 1 ELSE 2 END AS athlete_position,
       bg_color, fg_color, created_at
FROM match_participant mp
JOIN match m ON m.id = mp.match_id;
```

Views will be dropped once all code paths are refactored.

## Migration Outline

1. **Athlete merge**: build a temporary table mapping IOC/WT IDs to canonical athlete IDs. Upsert every row from `athletes` and `pss_athletes`, preferring the richer dataset. Attach `flag_id` automatically where `ioc_code` matches `flag.ioc_code`.
2. **Match remap**: copy `pss_matches` into `match`, deriving `match_code` (old `match_id`) and preserving `uuid`. Convert integer timestamps to ISO text once and drop integer mirrors.
3. **Participant remap**: resolve each `pss_match_athletes` row to canonical athlete ID. Store `side = CASE WHEN athlete_position = 2 THEN 'red' ELSE 'blue' END`. Record collisions for manual review.
4. **Lookup rename**: rename `look_*` tables to the neutral names and update foreign keys inside dependent tables.
5. **Trigger installation**: add `BEFORE INSERT` and `BEFORE UPDATE` triggers for each table to set timestamps. Add helper trigger on `athlete` to set `flag_id` if `ioc_code` matches an entry in `flag`.
6. **View creation**: create legacy views for any still-referenced names (`pss_*`, `ovr_*`). Provide synonyms for columns where names change (`match_id` → `match_code`). These views will be phased out after code refactors.

## ORM Strategy

We will adopt **SeaORM** for the Rust backend:

- Async-first (fits with Tokio and existing async plugins).
- Provides entity definitions that map cleanly to the normalized schema.
- Generates migration scaffolding (`sea-orm-cli migrate`) so schema evolution stays consistent.

Once the schema is migrated, we will:

1. Define SeaORM entities for each canonical table (`entity/src/*.rs`).
2. Replace raw SQL in the database plugin with SeaORM repositories.
3. Use SeaORM migrations to manage future schema changes, ensuring the DDL in `docs/database/schema.sql` matches generated migrations.

## Mapping Reference

| Legacy table | Canonical table | Notes |
| --- | --- | --- |
| `pss_matches` | `match` | `match_id` renamed to `match_code`. UUID retained. |
| `pss_match_athletes` | `match_participant` | `athlete_position` → `side`. |
| `pss_athletes` | `athlete` | Merge data; drop source table after migration. |
| `pss_events` | `event` | Column names preserved except `event_type_id` references `event_type`. |
| `pss_event_types` | `event_type` | Rename only. |
| `recorded_videos` | `video` | Rename only. |
| `look_divisions` | `division` | Rename. |
| `look_genders` | `gender` | Rename. |
| `look_weight_classes` | `weight_class` | Rename. |
| `ovr_categories` | `overlay_category` | Rename, update FK references. |
| `ovr_tournaments` | `overlay_tournament` | Rename. |
| `iv r_*` tables | evaluate individually; most will become `replay_*` or integrate with `match`/`video`. |

## Deliverables

- `docs/database/schema.sql`: generated DDL for the canonical schema.
- Migration SQL under `scripts/db_migrations/YYYYMMDD_schema_unification.sql` (includes data copy + view creation).
- SeaORM entity modules reflecting the new schema.
- Updated `AGENTS.md` pointing to this document and the schema reference.

This blueprint is the authoritative target for the upcoming migration work. Any deviation discovered during sandbox testing should be documented here before committing to the production schema.
