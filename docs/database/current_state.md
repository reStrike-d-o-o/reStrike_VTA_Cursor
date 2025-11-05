# Database Schema Snapshot (2025-11-05)

This document captures the current (pre-unification) state of the `restrike_vta.db` schema and highlights the areas that require consolidation. All statistics were collected on **2025‑11‑05** from `src-tauri/restrike_vta.db`.

## High-Level Observations

- Multiple parallel vocabularies are in use. For example, the tournament importer writes to `athletes` while the UDP/PSS ingest writes to `pss_athletes`. Join tables (`pss_match_athletes`) therefore point to different parent tables depending on the ingestion path.
- Many tables persist both ISO8601 timestamp columns (`created_at`, `updated_at`) *and* integer epoch mirrors (`created`, `updated`) without triggers to keep them in sync. Inserts/updates must currently manage both sets manually.
- Table naming prefixes (`pss_`, `ovr_`, `ivr_`, `look_`) leak implementation detail into the schema. The goal is to replace these with neutral names (`matches`, `match_participants`, `divisions`, …) and use views for backwards compatibility where needed.

## Table Inventory (selected)

| Table | Row count | Notes |
| --- | ---: | --- |
| `athletes` | 961 | Canonical roster used by tournament tooling. Includes lookup FKs (`look_*`) and IOC data. |
| `pss_athletes` | 10 | Minimal athlete information created by UDP ingest. Redundant with `athletes`. |
| `pss_match_athletes` | 1 816 | Links matches to athletes. Mixed referential integrity: some rows reference `athletes.id`, others `pss_athletes.id`. |
| `pss_matches` | 915 | Source of truth for match metadata. Stores both integer `id` and UUID. Contains duplicate timestamp columns. |
| `recorded_videos` | 0 | Present but currently empty in this snapshot. |
| `tournaments` | 3 | Tournament management module. Uses UUID columns plus redundant timestamps. |
| `flags` | 260 | Flag catalog. Not currently linked automatically from `pss_athletes`/`athletes`. |
| `look_divisions` | 27 | Lookup values for tournament taxonomy (will be renamed to `divisions`). |
| `look_genders` | 7 | Lookup values for genders. |
| `look_weight_classes` | 231 | Lookup values for weight classes. |

Additional supporting tables (`pss_events`, `pss_scores`, `pss_rounds`, `pss_warnings`, `recorded_video_events`, …) follow the same `pss_` prefixing pattern and depend on `pss_matches.id` or `pss_matches.uuid`.

## Timestamp Patterns

Many tables define both of the following sets of columns:

- `created_at TEXT NOT NULL`, `updated_at TEXT NOT NULL`
- `created INTEGER`, `updated INTEGER`

Examples: `obs_connections`, `app_config`, `network_interfaces`, `udp_server_configs`, `pss_matches`, `pss_scores`, `pss_rounds`, `pss_event_details`, `tournaments`, `recorded_videos`, …

These integer columns are inconsistently populated (some rows contain NULLs). The plan is to drop the integer columns everywhere and enforce ISO timestamps via triggers.

## Key Relationship Mismatches

- **Athlete Identity**
  - UDP ingest (`plugin_udp.rs`) creates/upserts rows in `pss_athletes` and then writes `pss_match_athletes` rows referencing those IDs.
  - Tournament importer (`importers/daedo.rs`) resolves athletes against the richer `athletes` table and writes participant rows referencing `athletes.id`.
  - Downstream queries (e.g. IVR match history snapshot) join `pss_match_athletes` to `pss_athletes`, so matches ingested via the tournament path lose names/flags in the UI.

- **Match Identifier**
  - `pss_matches` stores both an integer `id` and a UUID `uuid`. Some child tables use the integer (`pss_events.match_id`), others use the UUID (`pss_match_athletes.match_id`, `pss_scores.match_id`). This complicates joins and migrations.

- **Flags**
  - `pss_athletes.flag_id` is never populated. `athletes` holds `country_code` / `ioc_code` but no automatic linkage to the `flags` catalog.

## Existing Documentation Artifacts

- `docs/db_migration_plan.md` documents historic migration ideas (including adding `created`/`updated` integers). This file will be superseded once the unified schema is defined.
- `docs/architecture/DATABASE_INTEGRATION_GUIDE.md` refers to the `pss_*` naming convention and will need updates after renaming.

## Next Steps (per approved plan)

1. Produce `docs/database/target_model.md` outlining the normalized, prefix-free schema (`athletes`, `matches`, `match_participants`, etc.) and the mapping from legacy tables.
2. Design data migration SQL that merges `pss_athletes` into `athletes`, re-keys `pss_match_athletes`, renames lookup tables, and introduces consistent triggers.
3. Prototype the migration inside a sandbox database (`restrike_vta_tmp.db`) before touching the live DB.

This document will remain as the pre-migration reference point until the unified schema is committed.
