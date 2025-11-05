# Schema Rename Dictionary

This file tracks the canonical naming for tables and columns as we migrate away from subsystem-specific prefixes (`pss_`, `ovr_`, `look_`, `ivr_`, etc.). Each entry defines the source name, the canonical replacement, and any compatibility view or column rewrites needed in the Rust/TypeScript code.

> **Legend**
> - **Table:** Mapping for entire tables. If a view is required during transition, note it under "Bridge".
> - **Column:** Just the columns that change name or meaning. Unlisted columns retain their original name.

## Tables

| Legacy table | Canonical name | Bridge / notes |
| --- | --- | --- |
| `pss_matches` | `match` | View: `CREATE VIEW pss_matches AS SELECT ...` (until code refactor). |
| `pss_match_athletes` | `match_participant` | `(match_id, athlete_position)` -> `(match_id, side)` via view. |
| `pss_athletes` | `athlete` | Replace with canonical table; add view only if legacy SQL must read it. |
| `athletes` | `athlete` | Migrated content; original table renamed `legacy_athlete` during data copy. |
| `look_divisions` | `division` | Direct rename. |
| `look_genders` | `gender` | Direct rename. |
| `look_weight_classes` | `weight_class` | Direct rename. |
| `look_age_groups` | `age_group` | Direct rename. |
| `look_disciplines` | `discipline` | Direct rename. |
| `look_round_configs` | `round_config` | Direct rename. |
| `flags` | `flag` | Direct rename; `recognition_history` -> `flag_recognition_history`. |
| `pss_events` | `event` | View required until Rust ingestion switches to canonical table. |
| `pss_event_types` | `event_type` | Direct rename. |
| `pss_event_details` | `event_detail` | Rename + view if needed. |
| `pss_event_statistics` | `event_statistic` | Rename. |
| `pss_event_validation_rules` | `event_validation_rule` | Rename. |
| `pss_event_validation_results` | `event_validation_result` | Rename. |
| `pss_warnings` | `event_warning` | Rename. |
| `pss_rounds` | `round` | Rename. |
| `pss_scores` | `score` | Rename. |
| `pss_unknown_events` | `event_unknown` | Rename. |
| `recorded_videos` | `video` | Rename. |
| `recorded_video_events` | `video_event` | Rename. |
| `tournaments` | `tournament` | Already migrated. |
| `tournament_days` | `tournament_day` | Rename. |
| `tournament_rankings` | `tournament_ranking` | Rename. |
| `tournament_champions` | `tournament_champion` | Rename. |
| `ovr_categories` | `overlay_category` | Rename. |
| `ovr_flag_animations` | `overlay_flag_animation` | Rename. |
| `ovr_anthems` | `overlay_anthem` | Rename. |
| `ovr_tournaments` | `overlay_tournament` | Rename. |
| `ovr_providers` | `overlay_provider` | Rename. |
| `ovr_to_local_tournament` | `overlay_tournament_map` | Rename (bridge view required). |
| `ovr_templates` / `overlay_templates` | `overlay_template` | Consolidate during migration. |
| `ivr_*` modules | evaluate case-by-case | Prefer `replay_*` or integrate with canonical tables. |
| `udp_server_configs` | `udp_server_config` | Rename (singular). |
| `udp_server_sessions` | `udp_server_session` | Rename. |
| `udp_client_connections` | `udp_client_connection` | Rename. |
| `obs_connections` | `obs_connection` | Rename. |
| `obs_recording_sessions` | `obs_recording_session` | Rename. |
| `obs_recording_config` | `obs_recording_config` | Already canonical. |

## Column Renames

| Table | Legacy column | Canonical column | Notes |
| --- | --- | --- | --- |
| `match` | `match_id` | `match_code` | Keep integer `id` as surrogate. |
| `match_participant` | `athlete_position` | `side` (`blue`/`red`) | Implement via view while code transitions. |
| `athlete` | `wtid` | `wt_id` | Snake case with underscore. |
| `athlete` | `display_name` / `short_name` | Keep but add `pss_code` for former `athlete_code`. |
| `tournament` | `oc` | `organizing_committee` | Expand abbreviation. |
| `flag` | `recognition_status` | Keep; ensure `flag_recognition_history` foreign key renamed accordingly. |
| `round` | `winner_position` | `winner_side` | Convert integer to constrained text. |
| `score` | `score_value` | `value` | Optional clean-up. |
| `score` | `score_type` | `type` | Optional clean-up. |
| `event_warning` | `athlete_position` | `side` | Mirror match participant naming. |
| `video` | `video_type` | `type` | Keep enumerated TEXT; reorganise later if needed. |
| `video` | `record_directory` | `directory` | Optional clean-up. |
| `overlay_*` tables | `ovr_*` columns | Drop prefix (e.g., `ovr_tournament_id` -> `tournament_id`). |

## Code References

To align Rust and TypeScript code, use these mappings when refactoring:

- **Repositories**: Create modules (`repositories/athlete.rs`, `repositories/match.rs`, etc.) to encapsulate SeaORM entities with the canonical names.
- **Event ingest**: Replace raw SQL references to `pss_*` tables with the canonical ones. For example, UDP ingest should upsert through `athlete` and `match_participant` repositories.
- **Frontend**: TS queries/listeners need to request data via new API endpoints (or updated Tauri commands) that already return canonical field names.

Keep this dictionary updated while additional tables/columns are discovered. Once all code paths are migrated and compatibility views removed, this file can become a historical reference.
