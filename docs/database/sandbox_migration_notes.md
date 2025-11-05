# Sandbox Migration Prototype (2025-11-05)

This document records the execution of `scripts/db_migrations/20251105_schema_unification.sql` against a working copy of the database (`src-tauri/restrike_vta_tmp.db`). The goals were to stand up the canonical prefix-free schema and to validate merging logic before touching the production database.

## Summary

- Lookup tables were renamed to neutral names (`age_group`, `division`, `gender`, `weight_class`, `round_config`, `discipline`) and `flags` -> `flag`.
- Legacy data tables were preserved as `legacy_*` copies; canonical tables now include:
  - Core roster & schedule: `tournament`, `athlete`, `match`, `match_participant`
  - Competition data: `round`, `score`, `event_type`, `event`, `event_detail`, `event_statistic`, `event_validation_rule`, `event_validation_result`, `event_warning`, `event_unknown`
  - Media: `video`, `video_event`
- Data migration highlights:
  - `legacy_athlete` (961 rows) + `legacy_pss_athlete` (10 rows) -> `athlete` (971 rows). Every original record maps to a canonical athlete (0 unmatched entries). IOC codes are now leveraged to attach `flag_id`.
  - `legacy_match` (915 rows) -> `match` (915 rows). Missing UUIDs were back-filled with random UUIDs.
  - `legacy_match_participant` (1,816 rows) -> `match_participant` (1,810 rows). Six entries still lack a resolvable athlete; they were captured in `migration_missing_participants` for manual follow-up.
  - `legacy_round`, `legacy_score`, `legacy_event*`, and `legacy_video*` tables were migrated wholesale with schema clean-up (timestamps normalised, enumerations rewritten as `side`).
- Helper tables keep the mapping during migration (`tmp_athlete_map`, `tmp_match_map`) and are dropped at the end; `migration_missing_participants` remains for audit.
- Compatibility views (`athletes`, `pss_athletes`, `pss_matches`, `pss_events`, `recorded_videos`, etc.) mirror the legacy table names so existing code can continue to query without yet being refactored.
- Timestamp maintenance triggers were added for `athlete`, `match`, `event_statistic`, and `event_unknown`.

Current canonical row counts:

| Table | Rows |
| --- | ---: |
| `athlete` | 971 |
| `match` | 915 |
| `round` | 1977 |
| `score` | 7574 |
| `event` | 83688 |
| `event_warning` | 5764 |
| `event_unknown` | 0 |
| `video` / `video_event` | 0 |

## Post-migration Sanity Checks

| Query | Result |
| --- | --- |
| `SELECT COUNT(*) FROM athlete WHERE uuid IS NULL;` | 0 |
| `SELECT COUNT(*) FROM match WHERE uuid IS NULL OR uuid=''` | 0 |
| `SELECT COUNT(*) FROM migration_missing_participants;` | 6 |
| `SELECT COUNT(*) FROM legacy_pss_athlete l LEFT JOIN athlete a ON a.pss_code = l.athlete_code WHERE a.id IS NULL;` | 0 |
| `SELECT COUNT(*) FROM legacy_athlete l LEFT JOIN athlete a ON a.wt_id = l.wtid WHERE a.id IS NULL;` | 0 |

The six unresolved participants correspond to matches `913`, `914`, and `915`, where the legacy rows referenced athlete IDs `5..10` but no matching roster entry exists. These will require manual investigation (likely missing athlete creation in the original ingest).

## Next Steps

1. Rename and migrate the remaining overlay-/OBS-specific tables (`ovr_*`, `obs_*`, UDP telemetry tables) following the same pattern and extend compatibility views where necessary.
2. Finalise trigger coverage or replace with DEFAULT constraints so every canonical table maintains `created_at`/`updated_at` without application logic.
3. Scaffold SeaORM entities/migrations on top of the canonical schema so the Rust code can stop issuing raw SQL.
4. After code refactors land, drop the compatibility views and remove the `legacy_*` staging tables before cutover.

The sandbox database `src-tauri/restrike_vta_tmp.db` now reflects the expanded canonical schema for continued testing.
