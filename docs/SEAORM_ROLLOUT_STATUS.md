# SeaORM Migration Tracker

This document tracks the remaining work needed to migrate the backend from the legacy `rusqlite` operations layer to the canonical SeaORM entities. It is meant to be the single source of truth so we can take subsystems one by one until the legacy layer can be retired.

| Area / Feature | Current Persistence Usage | SeaORM Coverage | Notes / Next Steps |
| --- | --- | --- | --- |
| **PSS Events (core rows)** | `PssUdpOperations::store_pss_event` and friends | ✅ `seaorm_ops::pss::insert_event` (plugin paths updated) | Done via `plugins::plugin_database::store_pss_event` & migration script. |
| **PSS Event Details** | Legacy table `pss_event_details` | ✅ `seaorm_ops::pss::insert_event_details` | Duplicate-key semantics handled via upsert. |
| **PSS Scores / Warnings** | `pss_scores` / `pss_warnings` | ✅ SeaORM insert/fetch helpers; plugin switched | Final verification after UDP pipeline migration. |
| **Event Status / Validation / Unknown Events** | `PssEventStatusOperations` (rusqlite heavy) | ⛔ Not migrated | Needs SeaORM equivalents for history, statistics, validation results, unknown caches. |
| **Match Catalogue (match, match_participant)** | `PssUdpOperations` + ad-hoc SQL in `operations.rs` | ⛔ Not migrated | Create `seaorm_ops::matches`, update tournament + UDP flows, ensure manual mode already uses SeaORM. |
| **Athletes** | `operations.rs` + legacy tables | ⛔ Not migrated | Manual mode writes via SeaORM, but general path (PSS ingest, UI) still using rusqlite. |
| **Tournament / Days / Ranking / Champions** | Multiple helpers in `operations.rs` | ⛔ Not migrated | Required for OBS overlays, reporting, medal ceremony tooling. |
| **UI Settings** | `UiSettingsOperations` in `operations.rs` | ⛔ Not migrated | Provide SeaORM backed key/value service; update Tauri commands & React settings views. |
| **OBS Connections & Recording Config** | `operations.rs::ObsRecordingOperations` | ⛔ Not migrated | Add SeaORM helpers for `obs_connection`, `obs_recording_*` entities; switch `plugin_websocket`, `tauri_commands_obws`. |
| **UDP Server Config / Sessions / Clients** | Partially using SeaORM (`plugin_database::get_udp_server_configs`) | ⚠️ Partial | Reads use SeaORM; writes/sessions still rely on rusqlite operations. Finish migration and remove dual path. |
| **Overlay Providers / Flags / Anthems** | `operations.rs` overlay section | ⛔ Not migrated | Needed for overlays UI and OBS. |
| **Security Keys / Encryption** | `security::key_manager` / `security::encryption` with rusqlite | ⛔ Not migrated | Must ensure SeaORM provides secure storage semantics. |
| **Maintenance / Archives** | `maintenance.rs`, `operations.rs::DataArchivalOperations` | ⛔ Not migrated | Evaluate whether to keep as raw SQL or expose minimal SeaORM wrappers. |
| **Docs & Schema Reference** | Migration script `20251105_schema_unification.sql` | ✅ Bundled via `include_str!` | Update once new SeaORM modules land to keep docs aligned. |

## Working Approach

1. **Pick the next subsystem** – preferably one that unblocks multiple UI paths (e.g. match/athlete catalogue or tournament scaffolding).  
2. **Introduce a dedicated SeaORM operations module** (similar to `seaorm_ops::pss`). Keep them small and feature-focused.  
3. **Update the relevant plugin / Tauri command** to call the new helper. Avoid touching unrelated code.  
4. **Document completion** by changing the table above to ✅ and removing the legacy callsites.  
5. **Iterate** until all rows are checked off, then delete `operations.rs` (or reduce it to thin wrappers) and drop the legacy views.

## Immediate Candidates

- **Event Status / Validation**: natural follow-up to the event core migration; still blocks the dashboard and analytics.  
- **Match & Athlete catalogue**: most other subsystems (tournaments, overlays, OBS) depend on this data, so migrating it early pays dividends.  
- **UI Settings**: high-surface-area API used throughout the frontend; migrating it removes a large chunk of `operations.rs`.

Keep this file updated whenever a subsystem is migrated so we always know what remains before declaring SeaORM as the sole backend layer.
