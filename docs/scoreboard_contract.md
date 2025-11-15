# Scoreboard Contract

This document summarizes the agreement between the WT UDP protocol, the normalized scoreboard state, and the HTML overlays.

## Source Documents
- **Protocol summary:** `docs/protocol_summary.md`
- **Overlay ↔ protocol mapping:** `docs/overlay_protocol_contract.md`
- **Normalized event schema:** `docs/scoreboard_event_schema.md`
- **Verification checklist:** `docs/scoreboard_verification.md`

## Data Flow
1. The UDP backend parses WT packets into `PssEvent` variants (`src-tauri/src/pss/protocol.rs`).
2. Events are broadcast over the WebSocket to overlays (JSON payload: `{ type, timestamp, data }`).
3. `scoreboard-core.js` consumes those events, maintains the canonical `ScoreboardState`, and emits `state` updates.
4. Each overlay subscribes to `ScoreboardCore` and applies the latest state to its SVG IDs as per the mapping table.

## State Guarantees
- **Reset:** A `match_config` or `fight_loaded` event resets scores, warnings, rounds, clock, and injury state before applying new values.
- **Clock:** Every `clock` packet overwrites `clock.time` and its optional `action` sets `lastAction`.
- **Injury:** `injury` packets show/hide the injury timer and update the displayed time, regardless of athlete.
- **Scores:** `current_scores` updates total scores; `scores` populates per-round values without clearing other fields.
- **Warnings:** `warnings` updates both athlete counts atomically.
- **Rounds:** `winner_rounds` replaces the three round-winner slots; overlays convert them to counts as needed.

## Overlay Responsibilities
- Subscribe to `scoreboardCore.on('state', handler)` once the SVG is ready.
- Use `scoreboardCore.on('connection', ({ connected }) => …)` to toggle connection indicators.
- Call the existing `ScoreboardOverlay` helpers to update text, scores, warnings, injuries, and rounds using the normalized state.
- Do **not** implement additional WebSocket logic inside overlays.

## Testing
Run through the scenarios in `docs/scoreboard_verification.md` after every change to ensure the contract still holds.
