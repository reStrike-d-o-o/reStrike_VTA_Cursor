# Recording, Replay Buffer and Playback – Migration Plan (obws-first)

Purpose
- Implement fully automated recording and reliable replay buffer/save/playback using the new obws plugin only (no legacy plugins::obs).
- Ensure tournament/day/match context drives path and filename formatting.
- Persist sessions and map events to recorded files for later review/seek.
- Keep the code compiling green after each edit; remove legacy methods only after their obws equivalents are live and referenced.

Key UI/Backend Entry Points (today)
- Frontend obws bridge: `ui/src/utils/tauriCommandsObws.ts`
- Tauri obws commands: `src-tauri/src/tauri_commands_obws.rs`
- Recording event handler: `src-tauri/src/plugins/obs_obws/recording_events.rs`
- Path generator: `src-tauri/src/plugins/obs_obws/path_generator.rs`
- UDP/PSS events (source of truth for match lifecycle): `src-tauri/src/plugins/plugin_udp` (and DB ops consumed in recording)
- Triggers UI: `ui/src/components/molecules/TriggersRuleBuilder.tsx` + `ui/src/stores/triggersStore.ts`
- IVR replay (mpv) Tauri commands: `ivr_*` in `src-tauri/src/tauri_commands_obws.rs`

Global guardrails
- obws only: New work MUST use `plugins::obs_obws`. Do not call legacy `plugins::obs` from new code. As we port features, delete the replaced legacy code (in the same edit sweep) and fix compile.
- Compile often: After each logical change, build the Rust backend and rebuild UI. Fix errors immediately.
- Centralized messages: User notifications via `useMessageCenter`.
- OBS connections semantics: Add/persist connection roles (Recording vs Streaming). Default recording actions to role=Recording (`OBS_REC`).
 - Removed feature: Save replay buffer on match end (no longer persisted or exposed). Use explicit Save Replay control paths instead.

Phases and tasks (checklist)

Phase 0 – Baseline sanity and tooling
- [ ] Verify backend builds (in `src-tauri`) and UI builds (in `ui`).
- [ ] Ensure `useMessageCenter` mounted in `ui/src/App.tsx` for notifications.
- [ ] Confirm `useEnvironmentObs` status listener is running and returns obws states.

Phase 1 – Wire UDP/PSS → Recording handler
Backend
- [ ] Implement `get_current_match_id()` in `recording_events.rs` to read the active match from the UDP context (PSS subsystem). If no active match, return `Ok(None)` and log at debug level.
- [ ] Ensure UDP plugin emits PSS events to `ObsRecordingEventHandler::handle_pss_event` (FightLoaded, FightReady, Clock start/stop, Winner/WinnerRounds). Route through `App` central event bus if present.
- [ ] Add robust logging around each handled event.

Phase 2 – Stop delay and auto-record flow
Backend
- [x] Stop delay: WinnerRounds is a no-op; on Winner, wait `stop_delay_seconds` then stop recording.
- [x] FightLoaded: generate concrete recording path (no placeholders) + ensure directory exists; apply directory once per tournament day.
- [x] FightReady: Always-on RB. Ensure Replay Buffer is Active (start if not), then apply filename formatting for the current match and start recording.
Frontend
- [x] Add `Automatic recording` UI controls for `stop_delay_seconds` and `replay_buffer_duration`.
- [x] Remove "Save replay buffer on match end" toggle from UI to avoid confusion.
- [x] Preserve OBS connection status across tabs; treat `Authenticated` as Connected. WebSocket list updates are non-destructive and followed by an immediate status refresh.

Phase 3 – OBS connection roles (Recording / Streaming)
DB + Backend
- [ ] Extend OBS connection storage with `role: enum { recording, streaming, none }`. Default: `OBS_REC` → recording; `OBS_STR` → streaming.
- [ ] Expose get/set role via Tauri (`obs_obws_*` as needed) and wire manager to default recording actions to the `recording` role when no `connection_name` is passed.
Frontend
- [ ] Add role dropdown to OBS connections management; persist to DB.

Phase 4 – Replay buffer save + play (mpv) with feedback
Backend
- [ ] Add obws method to return last saved replay filename (or capture path on save). Expose via Tauri, e.g., `obs_obws_get_last_replay_filename` (or embed path in `save_replay_buffer` response if available).
- [ ] Ensure a combined flow exists: if replay buffer not running → start → wait ready → save → return filename. Maintain minimal latency.
- [ ] Optionally, add a combined command `obs_obws_save_replay_and_play` that saves replay and launches mpv (using IVR settings) for immediate playback.
Frontend
- [ ] When user hits REPLAY or Save Replay: call new API; on success, show success message with filename; on failure, show error; auto-play via mpv if configured.

Phase 5 – Tournament/day activation guard
Backend
- [x] If no active tournament/day, compute suggestions: Continue (`Tournament N / Day X+1`) or New (`Tournament N+1 / Day 1`) and emit a centralized message event.
Frontend
- [x] Show decision modal via Message Center; apply user choice back to backend which creates folders, applies record directory, and re-applies filename formatting.

Phase 6 – Persist sessions and map events to recorded files
DB + Backend
- [ ] On recording start: store session `start_time`, path, filename, `obs_connection_name`.
- [ ] On stop: store `end_time` and finalize session.
- [ ] Persist PSS events with absolute times; on session stop, compute offsets = `event_time - start_time` and store them (or store offsets as events arrive if session started).
Frontend
- [ ] On match review view, double-click event → compute `seek = start_time + offset` and open the recorded file via player (mpv preferred) one second before the event.

Phase 7 – Status indicators and notifications
Frontend
- [ ] Update DockBar status dots colors: OBS_REC – green=connected, yellow=connecting, red=recording; OBS_STR – green=connected, yellow=connecting, red=streaming.
- [ ] Add notifications for: recording started/stopped; replay saved/played; activation issues.

Phase 8 – Triggers alignment / legacy removal
Frontend
- [ ] Ensure trigger actions `record_start`, `record_stop`, `replay_save` all call obws Tauri commands.
Backend
- [ ] Where legacy `plugins::obs` duplicates exist, remove after the obws method is wired and referenced. Build to confirm green.

Phase 9 – Cleanup + docs
- [ ] Delete remaining legacy OBS recording/streaming code paths once feature parity is confirmed.
- [ ] Document the new flow and APIs under `docs/` with examples.

Testing & verification plan
- [ ] Unit-test path generator for multiple tournaments/days/flags.
- [x] Local end-to-end: simulate UDP PSS events → verified: path prep, RB ensured, filename formatting uses current match, recording starts, Winner-only delayed stop.
- [ ] Verify session persisted and event offsets computed; try opening recording at event-times via double-click.
- [ ] Multi-connection: confirm only `role=recording` is used for recording and replay buffer; streaming unaffected.

Rollback strategy
- obws-first edits are incremental. If any step introduces regressions, revert the specific change (Git) and re-open the corresponding TODO item.

Work protocol
- After completing any task above:
  1) Remove any equivalent legacy method in the same change.
  2) Build backend and UI; fix compile/lint immediately.
  3) Update this TODO file – check the item and add short notes (date, SHA, reviewer).

Next suggested starting point
- Phase 1: implement match id wiring from UDP into `get_current_match_id()` and register PSS → recording handler event calls; then Phase 2: stop delay and replay buffer auto-start.
