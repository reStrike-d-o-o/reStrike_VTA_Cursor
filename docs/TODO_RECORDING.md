# Recording, Replay Buffer and Playback – Migration Plan (obws-first)

## Current Implementation Status (2025-08-16)

### ✅ **COMPLETED FEATURES**

#### **Disk-First Flow & Modal Gating** ✅
- **Disk-First Architecture**: Tournament/Day folders are created on disk first, then OBS settings are applied
- **Smart Modal System**: Modal only appears when Tournament folders already exist on disk (prevents unnecessary prompts during first-time setup)
- **Session Reuse**: If Tournament 1/Day 1 was just created in the current session, reuse those instead of recomputing from disk
- **Path Generation**: Complete path generation with Windows Videos folder detection and dynamic tournament/day creation

#### **Live Athletes Capture & Filename Formatting** ✅
- **Real-time Data Capture**: Athlete names and flags captured immediately from PSS events (MatchConfig, Athletes)
- **Live Data Priority**: Use `session.match_number` and `session.player` names from MatchConfig/Athletes over database rows
- **Filename Placeholder Mapping**: Complete mapping from app placeholders to OBS placeholders with VS insertion logic; default template updated to `{matchNumber} {player1} ({country1}) VS {player2} ({country2}) - {date} - {time}`
- **Template System**: Dynamic filename formatting using live match data with fallback to database values

#### **OBS Recording Flow** ✅
- **FightReady Sequence**: Strict order: set record directory → set filename formatting → wait 500ms → ensure RB → start recording
- **Explicit Logging**: Comprehensive logging on FightReady when applying directory/formatting changes
- **Path Normalization**: Forward slash conversion before applying to OBS for cross-platform compatibility
- **Replay Buffer Management**: Always-on RB with proper status checking and activation

#### **Event Table & UI Integration** ✅
- **Event Table**: "Current" dropdown shows current + previous matches; preview uses `pss_get_events_for_match` with fallback
- **Database Persistence**: Event Table automatically saved to database on Winner event
- **Duplication Fix**: Removed frontend re-broadcast to prevent duplicated events; UI now consumes only backend WebSocket stream

#### **IVR Replay & Debugging** ✅
- REPLAY button and Challenge auto-trigger (respecting IVR toggle) call `replay_round_now` with DB settings
- Added robust println logs for RB status, save, polling, path resolution, and exact mpv command
- mpv auto-close on Clock start (resume) and Challenge accepted/rejected

### 🔄 **IN PROGRESS FEATURES**

#### **Day Creation / Reuse Logic** ✅
- **Status**: Completed
- **Notes**: In-session memo reuses just-created Tournament/Day; no disk rescan; after override path decision, FightReady auto-runs

#### **OBS Connection Roles** 🔄
- **Current Status**: Basic structure exists but needs completion
- **Next Steps**: Extend OBS connection storage with role enum (recording, streaming, none)
- **Priority**: Medium - improves connection management

### 📋 **REMAINING TASKS**

#### **Phase 1 – UDP/PSS Event Wiring** ✅
- [x] Implement `get_current_match_id()` with UDP-first fallback to DB
- [x] Wire all PSS events to `ObsRecordingEventHandler::handle_pss_event`
- [x] Add robust logging around each handled event and FightReady

#### **Phase 3 – OBS Connection Roles** ✅
- [x] Extend OBS connection storage with `role: enum { recording, streaming, none }`
- [x] Default: `OBS_REC` → recording; `OBS_STR` → streaming
- [x] Expose get/set role via Tauri commands: `obs_obws_get_connection_role`, `obs_obws_set_connection_role`
- [x] Wire manager to default recording actions to `recording` role

#### **Phase 4 – Replay Buffer Save + Play** ✅
- [x] Add obws method to return last saved replay filename (via obws `replay_buffer().last_replay()`)
- [x] Combined flow: ensure RB active → save → poll last filename (bounded)
- [x] Frontend integration with success/error feedback

#### **Phase 6 – Session Persistence** 🔄
- [ ] Persist recording sessions with start/end time and effective paths
- [ ] Compute event offsets relative to recording start
- [ ] Frontend integration for match review and seek functionality

#### **Phase 6.1 – Event Table lifecycle & review** ✅
- [x] On FightLoaded/FightReady: clear events; on recording started: wait 500 ms → clear again
- [x] On match end (Winner): store current Event Table to DB for this match (id/number)
- [x] "Current" dropdown: maintain rolling list; selecting a previous match loads its events (read-only)
- [x] Ensure selecting previous matches does NOT trigger OBS logic

#### **Phase 6.2 – Round tracking accuracy** ✅
- [x] Fix Event Table RND/round counter to avoid regression on clock events; backend stamps `current_round`

#### **Phase 7 – Status Indicators** 📋
- [ ] Update DockBar status dots colors for OBS_REC and OBS_STR
- [ ] Add notifications for recording started/stopped, replay saved/played
- [x] Add OBS monitoring commands: `obs_obws_start_monitoring`, `obs_obws_stop_monitoring`
- [ ] Implement activation issue notifications

#### **Phase 8 – Triggers Alignment** 📋
- [ ] Ensure trigger actions call obws Tauri commands
- [ ] Remove legacy `plugins::obs` duplicates
- [ ] Build to confirm compilation success

#### **Phase 9 – Cleanup + Documentation** 📋
- [ ] Delete remaining legacy OBS code paths
- [ ] Update all related documentation

### 🎞️ **NEW: Recorded Videos & Event-to-Video Linking** 🔄
- [ ] Migration: add `recorded_videos` table (id, match_id, event_id?, tournament_id?, tournament_day_id?, video_type, file_path, record_directory, filename_formatting, start_time, duration_seconds?, created_at)
- [ ] Insert 'recording' entries at recording stop (Winner) with match_id and resolved path/duration
- [ ] Insert 'replay' entries in `replay_round_now` after last replay filename resolved; compute start_time=now-RB_duration
- [ ] New command: `ivr_open_event_video(event_id)` → finds appropriate video, computes offset `event.timestamp - video.start_time`, launches mpv `--start=+offset`
- [ ] IVR drawer → Match history tab: Tournament/Day → Matches → Events → Videos table; Delete/Upload/Import actions
- [ ] Tie videos to matches for clean filtering and review
- [ ] Respect IVR toggle for auto-trigger; auto-close mpv on resume and challenge result (already done)

## 🧪 **TESTING & VERIFICATION PLAN**

#### **Completed Tests** ✅
- [x] End-to-end: UDP PSS events → path prep → RB ensured → filename formatting → recording start → Winner delayed stop
- [x] REPLAY/Challenge replay uses DB seconds_from_end; mpv command logs verified
- [x] mpv auto-close on resume/challenge resolution

#### **Remaining Tests** 📋
- [ ] Verify `recorded_videos` insertions for both recording and replay flows
- [ ] Event double-click opens file at correct offset
- [ ] Match history tab lists tournaments/days, matches, events, videos; filters correctly
- [ ] Delete/Upload/Import actions guarded and accurate

---

**Last Updated**: 2025-08-16  
**Current Focus**: Recorded videos & Event-to-Video Linking  
**Next Milestone**: Match history UI + Playback command
