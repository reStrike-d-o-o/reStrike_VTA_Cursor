# Scoreboard Verification Checklist

Use this checklist to validate every overlay against the WT UDP protocol after making changes. Run the steps while connected to the live simulator or PSS feed.

## 1. Initial Load
1. Start the app (`cargo tauri dev`) and open the three overlays (modern, olympic, arcade) in the browser.
2. Confirm the connection indicator toggles green once the WebSocket is established.
3. Ensure that before any events arrive, the overlays show zeroed scores, round 1, and hidden injury timers.

## 2. Match Configuration
1. Trigger a `FightLoaded` + `MatchConfig` event (load a match in the simulator).
2. Verify:
   - Match number, category, and weight update on all overlays.
   - Athlete names and flags change to the new contestants.
   - Round wins / warnings reset to zero.
   - Clock resets to the round duration (e.g., `02:00`).

## 3. Score Updates
1. Send `current_scores` packets for blue and red.
2. Confirm the large score digits update instantly on each overlay.
3. Send per-round `scores` packets and ensure the small round-score columns update accordingly (modern + olympic only).

## 4. Round Winners
1. Emit `winner_rounds` with values `[1, 2, 0]`.
2. Verify the round-win indicators show one win per athlete.

## 5. Warnings / Gam-jeom
1. Send `warnings` with increasing counts (0→3).
2. Check that the warning number (modern/olympic) and warning icons (arcade) update exactly to the provided value.

## 6. Clock Flow
1. Emit `clock;2:00;start`. Ensure the clock shows `2:00` and overlays react to the `start` action (no countdown logic—just display).
2. Send decreasing clock packets (`1:59`, `1:58`, ...). Confirm the text matches each payload.
3. Send `clock;1:30;stop` and ensure the timer shows `1:30` with no additional changes until the next packet.

## 7. Injury Timer
1. Emit `ij1;1:00;show`. The injury timer should appear with `1:00` on all overlays (Arcade switches to the combined timer block).
2. Send `ij1;0:45;` to ensure the displayed injury time changes while visible.
3. Send `ij1;0:30;hide` and confirm the injury timer hides everywhere.

## 8. Fight Ready / Supremacy
1. Emit `fight_ready` to ensure internal flags are set (no visible change expected).
2. Emit `sup;5;` and verify the debug metadata (if exposed) logs the value (optional).

## 9. Network Resilience
1. Stop the simulator (or unplug the feed) to drop the WebSocket connection.
2. Confirm the indicator turns red and overlaps remain static (no errors in console).
3. Restart the feed and ensure the overlays reconnect automatically and replay the latest state (athletes, match info, etc.).

Document any discrepancies, including the stream payload and the incorrect visual result, so the issue can be reproduced.
