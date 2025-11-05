# SeaORM Migration Tracker

This log keeps track of every subsystem that still touches the legacy `rusqlite` layer so we can migrate them one by one to the canonical SeaORM entities. When a row is marked **Done** the subsystem writes exclusively through the new helpers and legacy call sites can be retired.

| Area / Feature | Current Usage | SeaORM Coverage | Notes / Next Steps |
| --- | --- | --- | --- |
| PSS events (core rows) | `PssUdpOperations::store_pss_event` etc. | **Done** (`seaorm_ops::pss::insert_event`) | Plugin paths updated and migration script seeds the canonical table. |
| PSS event details | `pss_event_details` table | **Done** (`seaorm_ops::pss::insert_event_details`) | Handles duplicate keys via update-or-insert. |
| PSS scores / warnings | `pss_scores`, `pss_warnings` | **Done** (SeaORM helpers + plugin rewired) | Pending full UDP pipeline switch to confirm end-to-end behaviour. |
| Event status / validation / unknown events | `PssEventStatusOperations` | **Done** (`seaorm_ops::pss_status`) | Includes recognition history, statistics, validation results, and unknown cache. |
| Match catalogue (match, match_participant) | `PssUdpOperations`, raw SQL in `operations.rs` | **Pending** | Create `seaorm_ops::matches`, update tournament + UDP flows, verify manual mode still works. |
| Athletes | `operations.rs` | **Pending** | Manual mode already SeaORM; ingest/UI still relies on rusqlite. |
| Tournaments / days / ranking / champions | `operations.rs` | **Pending** | Needed for OBS overlays, reporting, medal ceremony tooling. |
| UI settings | `UiSettingsOperations` | **Pending** | Build SeaORM-backed key/value service; update Tauri commands & React settings views. |
| OBS connections & recording config | `operations.rs::ObsRecordingOperations` | **Pending** | Add SeaORM helpers for `obs_connection`, `obs_recording_*`; switch `plugin_websocket`, `tauri_commands_obws`. |
| UDP server config / sessions / clients | Mixed (SeaORM reads, rusqlite writes) | **Partial** | Finish SeaORM writes for sessions/clients and remove fallback ops. |
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
