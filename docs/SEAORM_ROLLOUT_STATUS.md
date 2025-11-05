# SeaORM Migration Tracker

This log keeps track of every subsystem that still touches the legacy `rusqlite` layer so we can migrate them one by one to the canonical SeaORM entities. When a row is marked **Done** the subsystem writes exclusively through the new helpers and legacy call sites can be retired.

| Area / Feature | Current Usage | SeaORM Coverage | Notes / Next Steps |
| --- | --- | --- | --- |
| PSS events (core rows) | `PssUdpOperations::store_pss_event` etc. | **Done** (`seaorm_ops::pss::insert_event`) | Plugin paths updated and migration script seeds the canonical table. |
| PSS event details | `pss_event_details` table | **Done** (`seaorm_ops::pss::insert_event_details`) | Handles duplicate keys via update-or-insert. |
| PSS scores / warnings | `pss_scores`, `pss_warnings` | **Done** (SeaORM helpers + plugin rewired) | Pending full UDP pipeline switch to confirm end-to-end behaviour. |
| Event status / validation / unknown events | `PssEventStatusOperations` | **Done** (`seaorm_ops::pss_status`) | Includes recognition history, statistics, validation results, and unknown cache. |
| Match catalogue (match, match_participant) | `PssUdpOperations`, raw SQL in `operations.rs` | **Done** (`seaorm_ops::pss_catalog`) | Plugin paths (UDP, OBS, Tauri commands) now call SeaORM helpers; legacy functions remain only for reference. |
| Athletes | `operations.rs` | **Done** (SeaORM-backed) | Manual match creation and UDP ingest now upsert via SeaORM; legacy helpers unused and ready for removal. |
| Tournaments / days / ranking / champions | `operations.rs` | **Pending** | Needed for OBS overlays, reporting, medal ceremony tooling. |
| UI settings | `seaorm_ops::ui_settings` | **Done** | Database plugin, HybridSettingsProvider, core app, and Tauri commands now use SeaORM helpers; legacy rusqlite module removed. |
| OBS connections & recording config | `operations.rs::ObsRecordingOperations` | **Pending** | Add SeaORM helpers for `obs_connection`, `obs_recording_*`; switch `plugin_websocket`, `tauri_commands_obws`. |
| UDP server config / sessions / clients | Mixed (SeaORM reads, rusqlite writes) | **Done** (SeaORM-backed) | Config, session lifecycle, and client tracking handled via SeaORM; keep verifying telemetry before deleting legacy helpers. |
| Network interfaces | `PssUdpOperations` | **Pending** | Build SeaORM helpers for NIC catalog + recommendations, then swap plugin and Tauri commands away from rusqlite. |
| Overlay providers / flags / anthems | `operations.rs` | **Pending** | Required for overlays UI and OBS scene builder. |
| Security keys / encryption | `security::key_manager`, `security::encryption` | **Pending** | Migrate secure storage semantics before dropping rusqlite. |
| Maintenance / archives | `maintenance.rs`, `operations.rs::DataArchivalOperations` | **Pending** | Decide whether to keep raw SQL or add thin SeaORM wrappers. |
| Docs & schema reference | `20251105_schema_unification.sql` | **Done** (`include_str!`) | Keep docs aligned as new SeaORM modules land. |

## Working Approach

1. Pick the next subsystem with the biggest downstream impact (match catalogue, tournaments, UI settings, etc.).
2. Add a focused `seaorm_ops::*` module that exposes high-level helpers returning the existing model structs.
3. Switch the relevant plugin / Tauri command to call the new helper and delete the rusqlite call site.
4. Update this tracker to mark the row **Done** (or **Partial** while both paths coexist).
5. Repeat until every row is complete, then remove `database::operations` and the legacy compatibility views.

## Immediate Candidates

- Match & athlete catalogue — unblocks tournaments, overlays, OBS, and analytics.
- UI settings — high touch area across the desktop UI; removing rusqlite here eliminates a large portion of `operations.rs`.

Keep this document current so the team always knows what remains before we can declare the SeaORM rollout finished.

## Pre-Data-Transfer Checklist

- Confirm end-to-end tests for SeaORM match/event flows (UDP ingest, OBS recorder, manual mode) against a seeded sandbox database.
- Replace remaining rusqlite paths (OBS recording/config, overlay providers, medal ceremony, archival jobs, security storage) with targeted SeaORM modules.
- Port network interface helpers to SeaORM so UDP server provisioning no longer requires the legacy pool connection.
- Audit migrations in `scripts/db_migrations/20251105_schema_unification.sql` to ensure new tables/entities match runtime expectations (timestamps, UUIDs, FKs).
- Draft data validation scripts (pre/post migration row counts, checksum comparisons) to catch divergence during the transfer window.
- Document rollback and recovery procedures once rusqlite dependencies are fully removed from production code paths.
