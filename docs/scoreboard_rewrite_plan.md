# Scoreboard Rewrite Task Tracker

This document tracks every task required to rebuild the scoreboard stack strictly following the WT UDP protocol (`protocol/pss_v2.3.txt`) and the overlay mapping (`docs/overlay_field_mapping_template.md`). Tasks are grouped by phase; each will be updated as soon as progress is made.

---

## Phase 0 – Ground Truth

- [x] **P0.1** Extract a structured summary of every stream from `protocol/pss_v2.3.txt`/`pss_schema.txt`. _(See `docs/protocol_summary.md`.)_
- [x] **P0.2** Reconcile overlay SVG IDs with protocol fields, producing a single contract table per overlay. _(See `docs/overlay_protocol_contract.md`.)_

## Phase 1 – Backend Protocol Layer (`src-tauri/src/pss/protocol.rs`)

- [x] **P1.1** Define enums/structs for every protocol stream (clock, match config, athletes, warnings, injury, challenges, etc.). _(See `src-tauri/src/pss/protocol.rs`.)_
- [x] **P1.2** Implement the parser that tokenizes UDP payloads into the typed events with validation. _(Used by `PssProtocol::parse_message`.)_
- [x] **P1.3** Integrate the new protocol module with the existing Tauri event pipeline so WebSocket/Store consumers receive normalized JSON. _(UDP plugin now uses `PssProtocol`.)_

## Phase 2 – Frontend Data Contract

- [x] **P2.1** Specify the normalized scoreboard event schema (TypeScript interfaces + precedence notes). _(See `docs/scoreboard_event_schema.md`.)_
- [x] **P2.2** Implement `scoreboard-core` controller that handles connections, caching, and event fan-out. _(See `ui/public/assets/scoreboard/scoreboard-core.js`.)_

## Phase 3 – Overlay Rewrites

- [x] **P3.1** Modern overlay: replace script with thin adapter wired to `scoreboard-core`.
- [x] **P3.2** Olympic overlay: same rewrite with its SVG IDs.
- [x] **P3.3** Arcade overlay: same rewrite, including dual-timer & warning visuals.

## Phase 4 – Utilities & Validation

- [ ] **P4.1** Clean up `scoreboard-utils.js`, leaving only shared helpers used by new overlays.
- [ ] **P4.2** Create and run deterministic verification sequences for each overlay (doc the procedure).
- [ ] **P4.3** Publish `docs/scoreboard_contract.md` summarizing the normalized event schema and testing steps.

---

Updates will be applied here after each task or sub-task is completed.
