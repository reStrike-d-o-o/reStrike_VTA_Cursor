# OBS (obws) Migration Verification Checklist

This checklist validates that the legacy OBS plugin has been fully replaced by the obws-based implementation and that all related features continue to work end-to-end without regressions.

## Pre-requisites
- [ ] Windows 10/11 environment, Tauri app builds and runs
- [ ] OBS Studio installed with OBS WebSocket v5 enabled (default port 4455)
- [ ] Optional: A second OBS instance or a remote OBS endpoint for multi-connection tests
- [ ] mpv installed and `Ivr Replay Settings` configured with a valid mpv.exe path
- [ ] Fresh database (optional but recommended)

## 1) OBS Connections & Status (obws)
- [ ] Add at least one connection via Control Room (name, host, port, password)
- [ ] Connect and disconnect the connection; verify status updates to Connected/Disconnected
- [ ] Add a second connection and repeat connect/disconnect to validate multi-connection support
- [ ] Get connection names and per-connection status (UI + logs) without errors
- [ ] Get global OBS status (tauri command) returns the new `ObsStatus` shape: recording_status, streaming_status, replay_buffer_status, virtual_camera_status, scenes, stats

## 2) Recording Control (Manual + Event-driven)
- [ ] Start recording via UI (manual); OBS shows recording started; database session updates as expected
- [ ] Stop recording via UI (manual); OBS shows recording stopped; recording indexed in DB
- [ ] Trigger automatic recording by sending PSS events (FightReady / Match start); recording starts automatically
- [ ] Recording stops automatically on match end and indexes into `recorded_videos` with correct folder and filename

## 3) Replay Buffer & IVR
- [ ] Press REPLAY button while live: save replay buffer → wait (maxWaitMs) → read last replay filename → launch mpv with `--start=-secondsFromEnd`
- [ ] Change `Ivr Replay Settings` (secondsFromEnd, maxWaitMs, mpv path), save, and verify settings persist to DB
- [ ] Enable “Auto on PSS Challenge” and send a Challenge event; verify REPLAY sequence is triggered automatically
- [ ] Negative path: invalid mpv path → verify graceful error/log; app continues operating

## 4) Auto-close mpv on Resume/Challenge Resolution
- [ ] While a replay is open in mpv, send `Clock{ action: start }` → mpv closes automatically
- [ ] While a replay is open in mpv, send `Challenge{ accepted: true }` → mpv closes automatically
- [ ] While a replay is open in mpv, send `Challenge{ accepted: false }` → mpv closes automatically

## 5) Match History & Precise Offsets
- [ ] After a recorded match, open Match History → Recorded Videos list shows the new video(s)
- [ ] Double-click a recorded video: app computes offset (from `start_time`) and plays at precise time in mpv
- [ ] Double-click a specific Event from the Event Table in Review Mode: plays the recording at the event’s precise offset
- [ ] Use `VideoEventPicker` (popover) to choose an event within a video and verify timing is accurate
- [ ] Delete a recorded video: DB row removed and filesystem file deleted
- [ ] Upload selected videos to Drive: progress is displayed; list shows success
- [ ] Import videos from Drive/local zip: rows and event links created; listed correctly in Recorded Videos

## 6) Control Room (obws-backed)
- [ ] Get connections: Control Room lists connections via obws
- [ ] Connect/Disconnect: per-connection operations succeed
- [ ] Set Scene (bulk): change scene across all connected instances
- [ ] Start/Stop Streaming (bulk): start and stop streaming across all connected instances
- [ ] Connect All / Disconnect All: successfully processes all connections; partial failures reported with details
- [ ] Add/Update/Remove connection: works correctly; statuses reflect live state
- [ ] Audio Mute/Unmute: currently returns success with empty results (stub) without errors

## 7) OBS Settings & Paths
- [ ] Get Recording Path Settings: returns directory + filename pattern via obws
- [ ] Set Recording Path: updates directory in OBS; read-back matches
- [ ] Set Filename Pattern: updates pattern; read-back matches
- [ ] Filename formatting includes player names and country codes when available

## 8) UDP/PSS Event Pipeline & Filtering
- [ ] UDP listener receives PSS events without panics; logs show normalized events
- [ ] Events persisted into DB correctly (including clock/round/system as needed for accurate timing)
- [ ] Event Table displays only intended event codes (P, K, H, TB, TH, R) in the UI
- [ ] No dead-code validation or unused handlers remain in `plugin_udp.rs`

## 9) Database Integrity
- [ ] `recorded_videos` rows created with valid foreign keys for tournament, day, and match
- [ ] `recorded_video_events` mapping present when linking events to recordings
- [ ] No foreign key violations during tournament/day creation and recording indexing

## 10) Error & Edge Case Handling
- [ ] Opening a video verifies `is_file()`: invalid paths do not spawn mpv and return a clear error
- [ ] Invalid/empty OBS connection names handled gracefully in Tauri commands
- [ ] Build without YouTube feature: YouTube commands return “disabled” without compile errors

## 11) UI/UX Regression Checks
- [ ] DockBar and StatusbarDock layout intact; no gaps; indicators update on status changes
- [ ] Event Table double-click disabled when not in Review Mode
- [ ] All atomic components render correctly (Button, Input, Checkbox, Label, Badge, Icon)
- [ ] No console errors or noisy logs during standard workflows (logger default level)

## 12) Documentation & Code Hygiene
- [ ] `docs/` updated: migration marked complete; legacy `plugins/obs` removed
- [ ] No references to `plugins::obs` remain in codebase
- [ ] No `#[allow(dead_code)]` markers remain; no unused imports (cargo check/clippy)

## 13) Multi-Connection Cross-Validation (Optional)
- [ ] Add two+ connections (local and remote)
- [ ] Start/Stop streaming per-connection and in bulk; scenes set consistently
- [ ] Disconnect one connection and re-run bulk operations; results exclude the disconnected node and report accurately

## 14) Performance Smoke Checks
- [ ] Recording start/stop latency acceptable (consistent with prior baseline)
- [ ] REPLAY sequence completes within configured `maxWaitMs`
- [ ] No fps/UI stutters during bulk operations

## 15) Final Acceptance
- [ ] All above checks pass on Windows
- [ ] Build is green; no runtime panics; logs are clean
- [ ] Feature parity confirmed; legacy OBS plugin fully removed

---

### Notes
- Audio mute/unmute is intentionally stubbed until obws exposes per-source controls. Current actions return success with empty results.
- Precise offset calculation depends on `recorded_videos.start_time` and linked event timestamps. If offsets appear off, verify system clock/timezone consistency and event capture timing.
