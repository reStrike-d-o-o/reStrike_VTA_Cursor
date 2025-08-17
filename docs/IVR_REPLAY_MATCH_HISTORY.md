## IVR Replay + Match History: implementation and debug notes

### Overview
- Backend (Rust, Tauri): precise video opening from events and recorded files, auto-indexing `recorded_videos`, default tournament/day bootstrap, IVR replay settings.
- Frontend (React, atomic): IVR tabs: `IVR playback settings` and `Match history`. History shows Days → Matches → Events + Recorded Videos with standard table styling and Button atoms.

### Key backend commands (names + params)
- `ivr_get_replay_settings()` → { mpv_path, seconds_from_end, max_wait_ms, auto_on_challenge }
- `ivr_save_replay_settings(mpv_path?, seconds_from_end, max_wait_ms, auto_on_challenge)`
- `ivr_list_tournament_days()` → ensures default "Tournament 1 / Day 1" if DB empty
- `ivr_list_matches_for_day(day_id)` → prefers matches with recordings; fallback to day-linked (if column exists) → recent matches
- `ivr_list_recorded_videos(tournament_day_id, match_id?)`
- `ivr_open_event_video(event_id)` → finds video + offset and opens with `mpv`
- `ivr_open_recorded_video(recorded_video_id, event_id?)` → computes precise offset
- `ivr_open_video_path(file_path, offset_seconds?)`
- `ivr_delete_recorded_videos(ids: Vec<i64>)`
- `ivr_upload_recorded_videos(ids: Vec<i64>)` → zips and uploads
- `ivr_import_recorded_videos(source, path_or_id, tournament_day_id, match_id)` → unzip → copy → index
- `pss_get_events_for_match(match_id as string DB id)` → enriched event rows, ordered

Accepted param names in the UI layer are dual-keyed for safety:
- day_id | dayId
- tournament_day_id | tournamentDayId
- match_id | matchId
- event_id | eventId
- recorded_video_id | recordedVideoId
- file_path | filePath
- offset_seconds | offsetSeconds
- path_or_id | pathOrId

### Database
- `recorded_videos(id, match_id, event_id?, tournament_id?, tournament_day_id?, video_type, file_path?, record_directory?, start_time, duration_seconds?, created_at)`
- New default bootstrap: if no tournaments exist → create "Tournament 1" (1 day) and start Day 1.

### Frontend components
- `IvrReplaySettings` (card): loads/saves settings via tauri commands, standard styling.
- `IvrHistoryPanel`: three-column tables (Days, Matches, Events) + Recorded Videos table with actions.
- `VideoEventPicker`: popover to pick an event for a selected video; calls `ivr_open_recorded_video` with precise offset.

### Data flow (Match history)
1) Days: `ivr_list_tournament_days()`
2) Matches for day: `ivr_list_matches_for_day(day_id)`
3) Events for match: `pss_get_events_for_match(match_id)`
4) Videos for day (+optional match): `ivr_list_recorded_videos(tournament_day_id, match_id?)`
5) Open event → `ivr_open_event_video(event_id)`
6) Open video → `ivr_open_recorded_video(recorded_video_id, event_id?)` or `ivr_open_video_path(file_path, offset)`

### Known issues to debug next session
- Verify the UI is sending arguments in the shape Tauri expects at runtime (dual-keying is in place; confirm the new bundle is loaded).
- Inspect return payloads for all invokes in `IvrHistoryPanel` (log success/shape; some handlers were permissive with `any`).
- Validate `recorded_videos` indexing exists (check DB rows after recording/replay stop). If empty, confirm `recording_events.rs` indexing paths run (FightLoaded/stop flows).
- Confirm `tournament_progress_context` or bootstrap runs once in your environment (ensure at least Day 1 exists).
- If events list is empty for a chosen match, confirm `match_id` is DB id; check `pss_get_events_for_match` returns rows for that id.

### Quick test checklist
- Open IVR → Match history → Day 1 exists.
- Day click → Matches populate (at least recent fallback).
- Match click → Events populate.
- Videos table shows rows if recordings exist; Open/Upload/Delete/Import buttons respond.
- Double-click event or video opens `mpv` with expected offset.

### Notes
- Styling mirrors Local Backup tables (`theme-card p-6 shadow-lg`, bordered tables with sticky headers, Button atoms).
- Offsets are computed from `recorded_videos.start_time` vs event `timestamp` (UTC), clamped ≥ 0.

### Second-pass features
- Bulk event-to-video linking on import and record index: `recorded_video_events` stores offsets for all matching events within window.
- Folder-targeted Drive uploads: pass `folder_id` to `ivr_upload_recorded_videos`.
- Progress and cancel: job_id carried through all progress events; cancel with `ivr_cancel_job(job_id)`.
- DriveBrowser supports breadcrumb navigation, new folder, and choose-here selection via `drive_list_children`, `drive_create_folder`.


