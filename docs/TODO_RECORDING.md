# Recording, Replay Buffer and Playback – Migration Plan (obws-first)

## Current Implementation Status (2025-08-14)

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

#### **Phase 3 – OBS Connection Roles** 📋
- [ ] Extend OBS connection storage with `role: enum { recording, streaming, none }`
- [ ] Default: `OBS_REC` → recording; `OBS_STR` → streaming
- [ ] Expose get/set role via Tauri commands
- [ ] Wire manager to default recording actions to `recording` role

#### **Phase 4 – Replay Buffer Save + Play** 📋
- [ ] Add obws method to return last saved replay filename
- [ ] Ensure combined flow: start RB → wait ready → save → return filename
- [ ] Optionally add combined command `obs_obws_save_replay_and_play`
- [ ] Frontend integration with success/error feedback

#### **Phase 6 – Session Persistence** 📋
- [ ] On recording start: store session start_time, path, filename, obs_connection_name
- [ ] On stop: store end_time and finalize session
- [ ] Persist PSS events with absolute times and compute offsets
- [ ] Frontend integration for match review and seek functionality

#### **Phase 6.1 – Event Table lifecycle & review** ✅
- [x] On FightLoaded/FightReady: clear events; on recording started (RecordStateChanged=true): wait 500 ms → clear again
- [x] On match end (Winner): store current Event Table to DB for this match (id/number)
- [x] "Current" dropdown: maintain rolling list of current + previous matches; selecting a previous match loads its events (read-only)
- [x] Ensure selecting previous matches does NOT trigger any recording/path/formatting logic
- [x] Added `pss_get_events_for_match` Tauri command; UI falls back to `pss_get_events` if not yet present

#### **Phase 6.2 – Round tracking accuracy** 🔄
- [ ] Fix Event Table RND column: capture and persist live round changes; ensure events are linked to the correct round (R1/R2/R3)
- [ ] Add regression test where 2–3 rounds occur; verify RND increments and table shows correct round per event
- Note: Clock events no longer change round; only explicit Round events update it

#### **Phase 7 – Status Indicators** 📋
- [ ] Update DockBar status dots colors for OBS_REC and OBS_STR
- [ ] Add notifications for recording started/stopped, replay saved/played
- [ ] Implement activation issue notifications

#### **Phase 8 – Triggers Alignment** 📋
- [ ] Ensure trigger actions call obws Tauri commands
- [ ] Remove legacy `plugins::obs` duplicates
- [ ] Build to confirm compilation success

#### **Phase 9 – Cleanup + Documentation** 📋
- [ ] Delete remaining legacy OBS recording/streaming code paths
- [ ] Document new flow and APIs with examples
- [ ] Update all related documentation

### 🧪 **TESTING & VERIFICATION PLAN**

#### **Completed Tests** ✅
- [x] Local end-to-end: simulate UDP PSS events → verified path prep, RB ensured, filename formatting uses current match, recording starts, Winner-only delayed stop
- [x] Path generation for multiple tournaments/days/flags
- [x] Modal gating logic (no modal on clean disk, modal only when folders exist)
- [x] Live athletes capture and filename formatting
- [x] Post-decision FightReady auto-run (no double click), directory/formatting/RB/record invoke sequence with read-backs
- [x] UDP-first precedence: Athletes/MatchConfig names and flags override database values for filename formatting

#### **Remaining Tests** 📋
- [ ] Verify session persisted and event offsets computed
- [ ] Test opening recording at event-times via double-click
- [ ] Multi-connection: confirm only `role=recording` used for recording and replay buffer
- [ ] Verify no modal on clean disk; modal only when Tournament folders already exist
- [ ] Clean-disk first-run: No `Tournament *` folders exist
  - [ ] App silently creates `Tournament 1/Day 1`
  - [ ] Sends directory to OBS (forward slashes)
  - [ ] Resolves effective template; sends filename formatting; read-back matches
  - [ ] Wait 500 ms → Ensure RB Active → Start recording
  - [ ] Logs show the full sequence without any modal and without double-clicks
- [ ] Event Table lifecycle: 500 ms post-recording start → table cleared; at Winner → events stored; dropdown review of previous matches populates table without side-effects
- [ ] Round tracking across 2–3 rounds; verify table RND values and DB persistence

### 🔧 **WORK PROTOCOL**

After completing any task above:
1. Remove any equivalent legacy method in the same change
2. Build backend and UI; fix compile/lint immediately
3. Update this TODO file – check the item and add short notes (date, SHA, reviewer)

### 🎯 **NEXT PRIORITY TASKS**

1. **Complete OBS Connection Roles** - Add role-based connection management
2. **Enhance Replay Buffer Integration** - Improve save/play functionality with mpv
3. **Session Persistence** - Implement complete session tracking and event mapping
4. **Regression Test Suite** - Add scripted tests/log assertions for clean disk vs existing folders, UDP-first formatting, RB/record start timing

### 📚 **KEY IMPLEMENTATION FILES**

- **Frontend obws bridge**: `ui/src/utils/tauriCommandsObws.ts`
- **Tauri obws commands**: `src-tauri/src/tauri_commands_obws.rs`
- **Recording event handler**: `src-tauri/src/plugins/obs_obws/recording_events.rs`
- **Path generator**: `src-tauri/src/plugins/obs_obws/path_generator.rs`
- **UDP/PSS events**: `src-tauri/src/plugins/plugin_udp`
- **Triggers UI**: `ui/src/components/molecules/TriggersRuleBuilder.tsx` + `ui/src/stores/triggersStore.ts`
- **IVR replay**: `ivr_*` in `src-tauri/src/tauri_commands_obws.rs`

### 🚨 **GLOBAL GUARDRAILS**

- **obws only**: New work MUST use `plugins::obs_obws`. Do not call legacy `plugins::obs` from new code
- **Compile often**: After each logical change, build the Rust backend and rebuild UI. Fix errors immediately
- **Centralized messages**: User notifications via `useMessageCenter`
- **OBS connections semantics**: Add/persist connection roles (Recording vs Streaming). Default recording actions to role=Recording (`OBS_REC`)
- **Removed feature**: Save replay buffer on match end (no longer persisted or exposed). Use explicit Save Replay control paths instead

---

**Last Updated**: 2025-08-14  
**Current Focus**: OBS Connection Roles & Round Tracking  
**Next Milestone**: Replay Buffer Save/Play & Session Persistence
