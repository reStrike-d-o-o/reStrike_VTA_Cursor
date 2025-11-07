# PSS Global Listener & Overlay Tasks

_Last updated: 2025-11-07_

## Active Tasks
- [ ] Validate WT UDP v2.3 mapping and document canonical event keys (points, triggers, break, winner rounds) for the listener taxonomy.
- [ ] Implement spinning-hit round tracker in the modern scoreboard overlay (count occurrences of point types 4 & 5, reset on round transitions).
- [ ] Repurpose `modern_player_stat_overlay.svg` for round-end strike summaries (rename labels, wire to scoreboard trigger, remove biometric assumptions).
- [ ] Add sandbox smoke coverage that simulates spinning hits and asserts stat overlay population/timing.
- [ ] Design and prototype the global PSS listener service (central dispatcher, filter pipeline, action routing API for OBS/overlays/statistics).
- [ ] Guarantee sub-50 ms dispatch for OBS IVR actions (record start/stop, replay save) through the listener to satisfy competition requirements.
- [ ] Extend the Triggering tab to configure listener filters → actions (persistence, validation, preview), including OBS, overlay, and analytics targets.
- [ ] Audit and replace legacy per-module PSS listeners/queues with calls into the global listener (scoreboard, OBS automation, analytics, sandbox harness).
- [ ] Provide performance guardrails: scoreboard updates must remain direct/no-buffer, with listener bypass modes documented for official overlay accuracy.

## Migration Roadmap
1. **Discovery** – Catalogue every subsystem currently consuming UDP events (Rust backend handlers, frontend sandbox bridges, OBS triggers) and note custom buffering/queue logic earmarked for removal.
2. **Listener Core** – Implement the central dispatcher crate/module, then refactor one pilot consumer (OBS automation) to subscribe via the new API; measure latency to confirm parity.
3. **Overlay Integration** – Wire the scoreboard and stat overlays to the listener while preserving zero-latency behaviour for scoreboard updates and using dedicated hooks for round summaries.
4. **Analytics & Storage** – Move event persistence/stat tracking to listener hooks, ensuring deduplicated writes and consolidated error handling.
5. **UI Configuration** – Upgrade the Triggering tab to edit listener filters, serialize configs, and push live updates to the dispatcher.
6. **Cleanup** – Delete superseded listeners/relays, remove redundant queues, and document the new architecture (diagrams + implementation notes).

### Discovery Findings — 2025-11-07

**Backend touchpoints**

| Component | Location | Feed | Buffering / Queues | Notes |
| --- | --- | --- | --- | --- |
| `UdpServer` ingestion & batcher | `src-tauri/src/plugins/plugin_udp.rs` | Parses raw WT UDP packets, forwards via `tokio::mpsc::UnboundedSender<PssEvent>` (`event_tx`) and writes to DB/WebSocket | Maintains `batch_tx` queue (`Vec` batches of 100) plus `VecDeque` recent history; spawns per-batch tasks that also call `websocket_server.broadcast_event` | Provides first touch; embedded WebSocket server duplicates overlay broadcast already handled downstream. |
| `App::handle_udp_events` | `src-tauri/src/core/app.rs` | Consumes primary `UnboundedReceiver<PssEvent>` from `UdpServer` | No extra queues; immediately converts to JSON and pushes into global `broadcast::Sender` (buffer 1000) via `emit_pss_event` | Fan-out point for Tauri UI, WebSocket plugin bridge, OBS automation, MPV cleanup, logging. |
| `App::handle_pss_to_websocket` + `WebSocketPlugin` | `src-tauri/src/core/app.rs`, `src-tauri/src/plugins/plugin_websocket.rs` | Subscribes to the global broadcast channel and pushes JSON to overlays on port 3001 | Uses broadcast receiver and per-client unbounded channels; no batching beyond broadcast buffer | This is the supported overlay path; current overlays rely on it. Conflicts with the legacy embedded server above. |
| OBS auto-record handler | `src-tauri/src/plugins/obs_obws/recording_events.rs` | Invoked from `handle_udp_events` for each PSS event when `obs-obws` feature is enabled | No buffering, but async DB lookups per event; closes MPV and optionally triggers replay | Needs listener hook integration while keeping <50 ms path for record/start/stop. |
| Trigger plugin (manual) | `src-tauri/src/plugins/plugin_triggers.rs` | Only receives events via explicit `tauri_commands_triggers` preview APIs today | Internal buffered `rdy` cache for resume delay | Not currently wired to live PSS feed; remains manual until listener work lands. |
| Logging | `src-tauri/src/core/app.rs` → `LogManager` | Writes each event to rotating log files | Straight write (no queue) | Keep for audit; ensure listener preserves hook. |

**Frontend / overlay touchpoints**

| Component | Location | Feed | Buffering / Queues | Notes |
| --- | --- | --- | --- | --- |
| Tauri event hook | `ui/src/hooks/usePssEvents.ts` | Listens to `pss_event` emitted from backend | None; forwards immediately to stores | Primary React app integration; clears Event Table on fight load/ready. |
| PSS match store handler | `ui/src/utils/pssEventHandler.ts` | Updates Zustand store, emits DOM `CustomEvent`, (unused) prep for WebSocket rebroadcast | No queue; synchronous updates | Emits local `pss-event` for components; retains legacy `websocket_broadcast_pss_event` helper that is no longer invoked. |
| LocalStorage broadcaster | `ui/src/utils/eventBroadcaster.ts` + overlays | Mirrors events into `localStorage` key `pss_event` for multi-window overlays | Relies on browser storage events; no buffering | Used by HTML overlays when running outside React shell. |
| HTML overlays | `ui/public/overlays/*/scoreboard.html`, `intro.html`, etc. | Connect to WebSocket port 3001 (and fall back to `localStorage` events) | Browser event loop only | Expect zero-delay updates; requires listener to keep direct scoreboard path.

**Duplication / risk notes**
- Two separate WebSocket broadcasters exist today: the `UdpServer`-embedded server and the `App`/`WebSocketPlugin` path. Listener refactor needs to collapse to a single broadcast surface (prefer the plugin version used by overlays).
- Tauri command `websocket_broadcast_pss_event` currently acquires the plugin lock but does not push messages; safe to sunset once the listener owns fan-out.
- Trigger plugin is not subscribed to live traffic yet; listener should provide a filtered feed instead of relying on manual preview calls.
- Placeholder `event_stream`/`event_cache` modules still compile but are no-ops; ensure the listener either replaces or removes them to avoid future confusion.

## Parking Lot
- Clarify whether additional PSS streams (e.g., `hl*`, `brk`, `wrd`) need default handlers in the listener before UI integration.
