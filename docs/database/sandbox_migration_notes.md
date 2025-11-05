# Sandbox Migration Prototype (2025-11-05)

This document records the execution of `scripts/db_migrations/20251105_schema_unification.sql` against a working copy of the database (`src-tauri/restrike_vta_tmp.db`). The goals were to stand up the canonical prefix-free schema and to validate merging logic for athletes and matches before touching the production database.

## Summary

- Lookup tables were renamed to neutral names (`age_group`, `division`, `gender`, `weight_class`, …) and `flags` → `flag`.
- Legacy tables were preserved as `legacy_*` copies; new canonical tables were created:
  - `tournament`
  - `athlete`
  - `match`
  - `match_participant`
- Data migration highlights:
  - `legacy_athlete` (961 rows) combined with `legacy_pss_athlete` (10 rows) → `athlete` (971 rows). Every original record maps to a canonical athlete (0 unmatched entries).
  - `legacy_match` (915 rows) → `match` (915 rows). Missing UUIDs were back-filled with random UUIDs.
  - `legacy_match_participant` (1,816 rows) → `match_participant` (1,810 rows). Six entries lack a resolvable athlete; they were captured in `migration_missing_participants` for manual follow-up.
- Helper tables keep the mapping during migration:
  - `tmp_athlete_map` (dropped at the end of the script)
  - `tmp_match_map` (dropped)
  - `migration_missing_participants` (retained for audit)
- `sqlite_sequence` entries for `tournament` and `match` were refreshed to keep autoincrement behaviour consistent.

## Post-migration Sanity Checks

| Query | Result |
| --- | --- |
| `SELECT COUNT(*) FROM athlete;` | 971 |
| `SELECT COUNT(*) FROM match;` | 915 |
| `SELECT COUNT(*) FROM match_participant;` | 1810 |
| `SELECT COUNT(*) FROM migration_missing_participants;` | 6 |
| `SELECT COUNT(*) FROM athlete WHERE uuid IS NULL;` | 0 |
| `SELECT COUNT(*) FROM match WHERE uuid IS NULL OR uuid='';` | 0 |
| `SELECT COUNT(*) FROM legacy_pss_athlete l LEFT JOIN athlete a ON a.pss_code = l.athlete_code WHERE a.id IS NULL;` | 0 |
| `SELECT COUNT(*) FROM legacy_athlete l LEFT JOIN athlete a ON a.wt_id = l.wtid WHERE a.id IS NULL;` | 0 |

The six unresolved participants correspond to matches `913`, `914`, and `915`, where the legacy rows referenced athlete IDs `5..10` but no matching roster entry exists. These will require manual investigation (likely missing athlete creation in the original ingest).

## Next Steps

1. Extend the migration script to cover additional tables (`pss_scores` → `score`, `pss_rounds` → `round`, `pss_events` → `event`, etc.) and populate compatibility views.
2. Define timestamp triggers for the canonical tables once the schema is finalised.
3. Scaffold SeaORM entities against the new schema to replace ad-hoc SQL.
4. Once the remaining tables are migrated and views are in place, execute the script on a fresh copy of the production database and cut over.

The sandbox database `src-tauri/restrike_vta_tmp.db` now reflects the canonical schema for continued testing.
