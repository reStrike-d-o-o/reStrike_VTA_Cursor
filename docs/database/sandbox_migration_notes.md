# Sandbox Migration Prototype (2025-11-05 refresh)

This log captures the latest dry-run of `scripts/db_migrations/20251105_schema_unification.sql`. The migration was executed against a clean snapshot (`src-tauri/restrike_vta_sandbox.db`) copied from `src-tauri/backups/manual_pre_cleanup_20251101_015755.db`. The objective is to ensure the canonical, prefix-free schema can be built without data loss before touching the production database.

## Summary

- Lookup catalogs have been renamed to neutral names (`age_group`, `division`, `gender`, `weight_class`, `round_config`, `discipline`, `flag`). The original tables live on as `legacy_*` copies while compatibility views expose the legacy names.
- Core competition data now resides in canonical tables:
  - Roster & schedule: `tournament`, `tournament_day`, `tournament_ranking`, `tournament_champion`, `athlete`, `match`, `match_participant`, `octagon`.
  - Scoring/event stream: `round`, `score`, `event_type`, `event`, `event_detail`, `event_warning`, `event_unknown`, `event_statistic`, `event_validation_rule`, `event_validation_result`, `event_recognition_history`.
  - Media & ceremonies: `video`, `video_event`, `medal_ceremony`, `medal_ceremony_division`, `medal_ceremony_medalist`.
- Legacy ingest artifacts (`legacy_*` tables) remain read-only for reference. A pair of helper temp tables (`tmp_athlete_map`, `tmp_match_map`) drive ID reconciliation during the migration and are dropped at the end; unresolved athlete references are parked in `migration_missing_participants`.
- Timestamp maintenance triggers now cover `tournament`, `tournament_day`, `tournament_ranking`, `tournament_champion`, `medal_ceremony`, `medal_ceremony_division`, `medal_ceremony_medalist`, `octagon`, `athlete`, `match`, `event_statistic`, and `event_unknown` so the application no longer needs to juggle `created`/`updated` epoch integers.
- Global `PRAGMA foreign_key_check` still reports mismatches for `legacy_*` tables because their foreign keys point at names that are now implemented as views. Canonical tables pass targeted FK checks (see below).

## Canonical Row Counts (sandbox snapshot)

| Table | Rows |
| --- | ---: |
| `tournament` | 2 |
| `tournament_day` | 4 |
| `tournament_ranking` | 16 |
| `tournament_champion` | 198 |
| `medal_ceremony` | 0 |
| `medal_ceremony_division` | 0 |
| `medal_ceremony_medalist` | 0 |
| `octagon` | 48 |
| `athlete` | 961 |
| `match` | 905 |
| `round` | 1,977 |
| `score` | 7,574 |
| `event` | 83,688 |
| `event_warning` | 5,764 |
| `event_unknown` | 0 |
| `video` / `video_event` | 0 |

> Note: counts reflect the backup snapshot; production totals may differ slightly.

## Post-migration sanity checks

| Query | Result |
| --- | --- |
| `SELECT COUNT(*) FROM athlete WHERE uuid IS NULL;` | 0 |
| `SELECT COUNT(*) FROM match WHERE uuid IS NULL OR uuid='';` | 0 |
| `SELECT COUNT(*) FROM migration_missing_participants;` | 6 (same unresolved legacy rows) |
| `PRAGMA foreign_key_check('match_participant');` | no violations |
| `PRAGMA foreign_key_check('octagon');` | no violations |
| `PRAGMA foreign_key_check('medal_ceremony_medalist');` | no violations |

Global `PRAGMA foreign_key_check;` raises expected warnings for `legacy_octagon` and a few other `legacy_*` tables because their foreign keys still reference the old table names (now backed by views). These tables are retained only for compatibility during the application refactor and can be ignored for migration validation.

## Follow-up actions

1. Port the Rust backend to SeaORM entities that align with the canonical tables and retire the raw SQL accessors. Compatibility views can then be removed in stages.
2. Add end-to-end migration tests (or at least scripted checks) that run `PRAGMA foreign_key_check` on every canonical table and verify row counts before/after.
3. Once all code paths rely on the canonical schema, drop the `legacy_*` tables and remove the temp mapping structures from the migration script.
4. Keep the docs (`target_model.md`, `rename_dictionary.md`, `AGENTS.md`) updated whenever additional tables are surfaced or renamed so the schema stays the single source of truth.
