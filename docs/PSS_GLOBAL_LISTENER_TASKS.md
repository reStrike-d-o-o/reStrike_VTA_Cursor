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

## Parking Lot
- Clarify whether additional PSS streams (e.g., `hl*`, `brk`, `wrd`) need default handlers in the listener before UI integration.
