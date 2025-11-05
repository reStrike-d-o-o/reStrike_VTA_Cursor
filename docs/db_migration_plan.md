# Database Migration Plan (from db_schema_review.xlsx)

Global rules:
- IDs: switch to UUID v4 (TEXT) primary keys; generate in backend.
- Foreign keys: rename to table_id (e.g., user_id, match_id).
- Timestamps: rename created_at/updated_at -> created/updated; add if missing (Unix timestamp; render local time).
- NOTES: if 'NO CHANGES', keep structure (still apply global rules unless explicitly contradicted).
- DATA: save|delete|dump governs row retention and code removal for delete/dump.

## Current status snapshot (SeaORM rollout)
- SeaORM modules now back core PSS ingest paths: matches, match athletes, events, details, scores, warnings, recognition stats, UDP server config/sessions/clients, and athlete upserts.
- Legacy rusqlite usage remains for UI settings, OBS recording/config, overlay providers and assets, network interface catalog, medal ceremony tooling, archival jobs, and secure config storage.
- Prior to data transfer, finish the pending SeaORM modules, stage validation scripts (row counts, checksum diffs), and plan rollback steps once `database::operations` is fully retired.

## app_config
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)

## config_audit
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Add created (INTEGER, not null)
  - Add updated (INTEGER, not null)

## config_categories
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)

## event_triggers
- DATA: delete
- NOTES:
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT
  - Remove all records; remove relations and references in code

## flag_mappings
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)

## flags
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Add created (INTEGER, not null)
  - Add updated (INTEGER, not null)

## look_age_groups
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)

## look_disciplines
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)

## look_divisions
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)

## look_genders
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)

## look_round_configs
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)

## look_weight_classes
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## network_interfaces
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)

## obs_connections
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)

## obs_scenes
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT

## overlay_templates
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)

## ovr_categories
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT

## ovr_providers
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)

## ovr_tournaments
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT

## pss_athletes
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT

## pss_event_details
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## pss_event_types
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)

## pss_events
- DATA: dump
- NOTES:
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT
  - Drop table after migration (dump); remove relations and references in code

## pss_events_v2
- DATA: save | delete | dump
- NOTES:
  - RENAME TABLE TO pss_events and update all relations acordingly
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## pss_match_athletes
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## pss_matches
- DATA: save
- NOTES:
  - after changes updater all relations acordingly
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT

## pss_rounds
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## pss_scores
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## pss_unknown_events
- DATA: save | delete | dump
- NOTES:
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT

## pss_warnings
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## recognition_history
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Add created (INTEGER, not null)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## recorded_videos
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## schema_version
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Add created (INTEGER, not null)
  - Add updated (INTEGER, not null)

## secure_config
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)

## security_sessions
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## settings_categories
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)

## settings_history
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## settings_keys
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT

## settings_values
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT

## tournament_days
- DATA: dump
- NOTES:
  - delete all relations after deletion
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT
  - Drop table after migration (dump); remove relations and references in code

## tournaments
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)

## udp_server_configs
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Rename created_at/updated_at -> created/updated (INTEGER Unix timestamp)
  - Ensure FK names use table_id form and reference UUID TEXT

## udp_server_sessions
- DATA: save | delete | dump
- NOTES:
  - NO CHANGES
  - CREATED: unix timestamp automatically set on insert
  - UPDATED: unix timestamp automatically set on update (if applicable)
- Proposed changes:
  - Primary key: set id TEXT (UUID v4) as PK
  - Add created (INTEGER, not null)
  - Add updated (INTEGER, not null)
  - Ensure FK names use table_id form and reference UUID TEXT
