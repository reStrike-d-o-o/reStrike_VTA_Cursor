# Overlay Windows & Trigger Routing — Implementation Plan

This document is a **step‑by‑step execution guide** for an agent to implement fully working, low‑latency overlay windows for the Olympic, Modern, and Arcade scoreboards, plus a settings tab to route triggers → overlay actions.

The goal is that a future agent can start from this file and, by following the tasks in order, get everything working without needing to rediscover context.

> Repo root: `reStrike_VTA_Cursor`  
> Frontend (React SPA): `ui/`  
> Static overlays + SVGs: `ui/public/overlays/**`, `ui/public/assets/scoreboard/**`  
> Tauri backend: `src-tauri/`

---

## 0. Current State (Ground Truth)

Before implementing anything, the agent should understand the existing data flow and overlay implementation.

**Data flow**

- UDP PSS → Rust UDP plugin → PSS events (`PssEvent`) in `src-tauri/src/pss/protocol.rs`.
- Events are broadcast to:
  - **WebSocket plugin** (`src-tauri/src/plugins/plugin_websocket.rs`) → HTML overlays via `ws://127.0.0.1:3001`.
  - **Frontend SPA** via Tauri event bridge (`pss_event`) → `ui/src/hooks/useLiveDataEvents.ts` → `ui/src/utils/pssEventHandler.ts` → Zustand store `usePssMatchStore`.
- SPA sidebar Match Info (names, flags, weights) is rendered by `ui/src/components/molecules/MatchDetailsSection.tsx`, reading from `usePssMatchStore`.

**Overlays**

- Static HTML overlays live under `ui/public/overlays/**`:
  - Olympic scoreboard: `ui/public/overlays/olympic/scoreboard.html`
  - Modern scoreboard: `ui/public/overlays/modern/scoreboard.html`
  - Arcade scoreboard: `ui/public/overlays/arcade/scoreboard.html`
- Each scoreboard HTML loads:
  - `/assets/scoreboard/scoreboard-core.js` — WebSocket client + state.
  - `/assets/scoreboard/scoreboard-utils.js` — SVG update helpers (`ScoreboardOverlay`, flags, timer, rounds, etc.).

**Important: Do not change these behaviors** (they are already correct and fast):

- `ScoreboardCore` WebSocket connect logic in `ui/public/assets/scoreboard/scoreboard-core.js`.
- SVG update functions in `ui/public/assets/scoreboard/scoreboard-utils.js` (except where explicitly requested later).

---

## 1. High‑Level Goals

1. Create **three dedicated Tauri windows**, each rendering one scoreboard overlay:
   - Olympic overlay window.
   - Modern overlay window.
   - Arcade overlay window.
2. Each overlay window:
   - Uses the existing static HTML overlay (no duplication of scoreboard logic).
   - Has a solid, configurable background color (e.g., green) suitable for OBS chroma key.
   - Is borderless, non‑resizable, and optionally always‑on‑top.
   - Connects to the WebSocket server and updates in realtime.
3. Add a **settings tab/page** in the main React app where the operator can define:
   - Which triggers (PSS events / internal triggers) map to which overlay window.
   - Example: `fight_loaded` → show Olympic overlay; `winner` → show Modern overlay, etc.
4. Add a small overlay‑window controller layer that:
   - Listens to PSS events in the SPA (already happening via `handlePssEvent`).
   - Reads the operator’s trigger‑to‑overlay mapping.
   - Sends Tauri commands to show/hide the relevant overlay windows.
5. Keep the system **lightweight and realtime**:
   - Backend always streams live PSS events.
   - WebSocket plugin no‑ops when there are no overlay connections (already implemented via client count check).
   - Overlay windows subscribe early and stay connected so they see live `clock` / `score` events.

---

## Implementation Progress (GPT‑driven updates)

Latest update: 2025‑11‑16

- Implemented a frontend overlay window helper `openOverlayWindow` in `ui/src/utils/overlayWindows.ts` using Tauri’s `WebviewWindow` API with fixed labels (`overlay_olympic`, `overlay_modern`, `overlay_arcade`) and 1920×1080, borderless, always-on-top windows.
- Wired “Overlay windows” buttons (Olympic / Modern / Arcade) in `DockBar` (`ui/src/components/layouts/DockBar.tsx`) and the overlay controller to call `openOverlayWindow(...)`, removing the need for dedicated Rust window commands while keeping behavior identical to the original design.
- Updated overlay backgrounds for OBS chroma key:
  - `ui/public/overlays/olympic/scoreboard.css` → `body { background: #00ff00; }`
  - `ui/public/overlays/modern/scoreboard.css` → `body { background: #00ff00; }`
  - `ui/public/overlays/arcade/scoreboard.html` inline `html, body` style → `background: #00ff00;`
- Added overlay routing types in `ui/src/types/index.ts` (`OverlayId`, `OverlayTriggerType`, `OverlayRoutingRule`, `OverlayRoutingConfig`).
- Created Zustand store `useOverlayRoutingStore` in `ui/src/stores/overlayRoutingStore.ts` with defaults (`fight_loaded` → Olympic, `winner` → Modern) plus load/save helpers that call new Tauri commands.
- Implemented `OverlayRoutingSettings` UI component in `ui/src/components/molecules/OverlayRoutingSettings.tsx` and mounted it inside the existing **PSS → Scoreboard** sub‑tab in `ScoreboardManager` (`ui/src/components/molecules/ScoreboardManager.tsx`).
- Added `useOverlayController` hook in `ui/src/hooks/useOverlayController.ts` that listens to browser `pss-event` custom events emitted by `handlePssEvent` and calls overlay window commands based on routing rules; hook is mounted once in `ui/src/App.tsx`.
- Overlay controller reuses the same PSS event stream as the rest of the app (no extra WebSocket/Tauri listeners), keeping latency low and behavior aligned with existing overlay updates.
- Implemented backend persistence for overlay routing:
  - New Rust types `OverlayRoutingRule` in `src-tauri/src/types/mod.rs`.
  - New Tauri commands `get_overlay_routing_config` / `set_overlay_routing_config` in `src-tauri/src/tauri_commands_overlays.rs`, wired into `src-tauri/src/main.rs`.
- On app startup, routing config is loaded once in `ui/src/App.tsx`, and UI changes are immediately persisted back to the backend.

Next steps:
- Refine trigger-to-event mapping once final PSS event types are confirmed.
- Manually verify end-to-end behavior with live PSS data and OBS, adjusting any UX details (e.g., labels, defaults, debounce) as needed.

---

## 2. Phase 1 — Add Olympic overlay Tauri window

**Goal:** One dedicated window that loads the existing Olympic scoreboard HTML and is ready for OBS Window Capture.

### 2.1. Add Tauri command to open Olympic overlay window

**Files:**

- `src-tauri/src/tauri_commands_overlays.rs` (or new `tauri_commands_overlay_windows.rs` if you prefer)
- `src-tauri/src/main.rs`

**Steps:**

1. Define a new Tauri command, e.g. `open_olympic_overlay_window`:
   - Signature: `#[tauri::command] pub async fn open_olympic_overlay_window(window: tauri::Window, app: State<'_, Arc<App>>) -> Result<(), TauriError>`
   - Use `window.app_handle()` (or `app_handle` from the builder) to call `tauri::WindowBuilder::new`:
     - Label: `"overlay_olympic"` (constant string; used later in OBS).
     - URL:
       - For dev: `"/overlays/olympic/scoreboard.html"` (served by the dev server).
       - For bundled app: `"overlays/olympic/scoreboard.html"` via `tauri::WindowUrl::App`.
     - Properties:
       - `.decorations(false)`
       - `.resizable(false)`
       - `.fullscreen(false)`
       - `.always_on_top(true)`
       - `.transparent(false)`
       - `.inner_size(1920.0, 1080.0)` (or configurable later).
   - Before creating, check if a window with label `"overlay_olympic"` already exists; if yes, focus it instead of creating a second instance.
2. Wire the command into the Tauri builder:
   - In `src-tauri/src/main.rs`, add `tauri_commands_overlays::open_olympic_overlay_window` to the `.invoke_handler` macro list.
3. Decide on a background color for chroma key:
   - In `ui/public/overlays/olympic/scoreboard.css` (or inline `<style>` in `scoreboard.html`), set:
     - `body { background: #00ff00; margin: 0; overflow: hidden; }`
   - Ensure the SVG fills the viewport so OBS can capture a predictable frame.

### 2.2. Add a simple UI entry point to open the Olympic overlay window

**Files:**

- `ui/src/components/layouts/DockBar.tsx` (or whichever component hosts main toolbar/settings)

**Steps:**

1. Add a minimal control in the SPA (e.g., a button in an “Overlays”/“Tools” section) that calls the Tauri command:
   - Use `@tauri-apps/api/core` → `invoke('open_olympic_overlay_window')`.
2. For now, no need for a full settings UI; just verify that clicking the button opens the new Olympic overlay window with the HTML scoreboard inside.
3. Run in dev (`pnpm dev` + `cargo tauri dev`) and verify:
   - WebSocket server logs `New WebSocket connection` when the Olympic window loads.
   - The Olympic scoreboard updates when PSS events arrive (at least timer and scores).

**Checklist before moving on:**

- [ ] `open_olympic_overlay_window` Tauri command exists and is registered.
- [ ] Clicking the new button in the SPA opens a borderless 1920×1080 window labeled `overlay_olympic`.
- [ ] The window shows the Olympic scoreboard and connects to the WS server.
- [ ] Closing and reopening the window reuses the same label or creates a fresh window without errors.

---

## 3. Phase 2 — Add Modern and Arcade overlay windows

**Goal:** Replicate the Olympic window pattern for Modern and Arcade scoreboards.

### 3.1. Modern overlay window

**Files:**

- `src-tauri/src/tauri_commands_overlays.rs`
- `ui/public/overlays/modern/scoreboard.html`

**Tasks:**

1. Add `open_modern_overlay_window` Tauri command:
   - Label: `"overlay_modern"`.
   - URL: `"overlays/modern/scoreboard.html"`.
   - Same window properties as Olympic.
   - Reuse the “single instance per label” pattern.
2. Add a SPA control (button/toggle) that invokes this command.
3. Ensure `modern/scoreboard.html` loads the same JS bundle as the existing standalone overlay (scoreboard-core + scoreboard-utils).

### 3.2. Arcade overlay window

**Files:**

- `src-tauri/src/tauri_commands_overlays.rs`
- `ui/public/overlays/arcade/scoreboard.html`

**Tasks:**

1. Add `open_arcade_overlay_window` Tauri command:
   - Label: `"overlay_arcade"`.
   - URL: `"overlays/arcade/scoreboard.html"`.
2. Add SPA control to invoke this command.
3. Verify Arcade overlay connects to WS and responds to PSS events.

**Checklist:**

- [ ] All three commands (`open_olympic_overlay_window`, `open_modern_overlay_window`, `open_arcade_overlay_window`) exist and are registered.
- [ ] Each overlay window opens on demand and shows its respective scoreboard.
- [ ] When each overlay is open and PSS events are flowing, timers and scores update in realtime.

---

## 4. Phase 3 — Overlay window manager (backend)

**Goal:** Centralize overlay window creation/lookup and avoid scattered window‑management logic.

**Files:**

- `src-tauri/src/tauri_commands_overlays.rs` (or new module under `src-tauri/src/overlays/`)

**Tasks:**

1. Extract common code into helper functions:
   - `fn get_or_create_overlay_window(app: &tauri::AppHandle, label: &str, url: &str) -> Result<tauri::Window, TauriError>`:
     - Checks `app.get_window(label)`, returns it if present (and maybe calls `set_focus`).
     - Otherwise builds a new window with standard properties and returns it.
2. Update each overlay‑open command to call this helper.
3. (Optional) Add a command `close_overlay_window(label: String)` to allow the SPA to close overlays programmatically.

**Checklist:**

- [ ] Overlay window creation is centralized.
- [ ] No duplicated `WindowBuilder` logic across commands.

---

## 5. Phase 4 — Overlay trigger routing settings (frontend, PSS → Scoreboard sub‑tab)

**Goal:** Reuse the existing **PSS → Scoreboard** sub‑tab as the place where the operator configures trigger → overlay actions (no new top‑level tab; this lives inside the PSS section).

### 5.1. Define overlay routing config types

**Files:**

- `ui/src/types/index.ts` (or new `ui/src/types/overlayRouting.ts`)

**Tasks:**

1. Define a TypeScript enum/string union for overlay identifiers:
   - `'none' | 'olympic' | 'modern' | 'arcade'`.
2. Define a type for triggers you care about (these should be real event types you already see in PSS):
   - Examples: `'fight_loaded'`, `'fight_ready'`, `'round_start'`, `'round_end'`, `'winner'`, `'injury_show'`, `'injury_hide'`.
3. Define an interface:

   ```ts
   export interface OverlayRoutingRule {
     trigger: OverlayTriggerType;
     overlay: OverlayId;       // e.g., 'olympic' | 'modern' | 'arcade' | 'none'
     action: 'show' | 'hide' | 'toggle';
   }

   export interface OverlayRoutingConfig {
     rules: OverlayRoutingRule[];
   }
   ```

### 5.2. Overlay routing store (Zustand)

**Files:**

- `ui/src/stores/overlayRoutingStore.ts` (new)

**Tasks:**

1. Create a Zustand store that holds `OverlayRoutingConfig`:
   - State: `rules: OverlayRoutingRule[]`.
   - Actions:
     - `setRules(rules: OverlayRoutingRule[])`.
     - `updateRule(index, rule)`.
     - `resetToDefaults()`.
   - Defaults can be something simple, e.g.:
     - `fight_loaded` → show `'olympic'`.
     - `winner` → show `'modern'`.
     - Others → `'none'`.
2. (Optional now, required later): wire persistence through Tauri commands (Phase 6).

### 5.3. Settings UI for routing (inside PSS → Scoreboard sub‑tab)

**Files:**

- `ui/src/components/molecules/OverlayRoutingSettings.tsx` (new)
- Existing PSS tab layout (where the Scoreboard sub‑tab is defined)

**Tasks:**

1. Build `OverlayRoutingSettings` component:
   - Reads `rules` from `useOverlayRoutingStore`.
   - Renders a table/list of rows: `Trigger` | `Overlay` | `Action`.
     - `Trigger`: read‑only label (e.g., `'Fight Loaded'`).
     - `Overlay`: `<select>` with options `None / Olympic / Modern / Arcade`.
     - `Action`: `<select>` with `Show / Hide / Toggle`.
   - On change, calls store actions to update the rule.
2. Integrate this component into the **existing PSS → Scoreboard** sub‑tab:
   - Do **not** create a new top‑level tab; instead, embed the routing table in the Scoreboard sub‑tab alongside any existing scoreboard settings.
   - If necessary, add a “Overlay Routing” section header within the Scoreboard sub‑tab to visually separate it from other controls.

**Checklist:**

- [ ] Overlay routing store exists and holds a list of rules.
- [ ] Overlay routing settings UI is visible within the PSS → Scoreboard sub‑tab and lets the user change which overlay is used for each trigger.

---

## 6. Phase 5 — Overlay controller (frontend → backend)

**Goal:** React to PSS triggers in the SPA and open/close overlay windows using the routing config.

### 6.1. Overlay controller hook

**Files:**

- `ui/src/hooks/useOverlayController.ts` (new)

**Tasks:**

1. Create a React hook that:
   - Subscribes to the **same PSS events** used by `handlePssEvent`:
     - The simplest is to piggyback on `handlePssEvent` by adding a call there, or:
     - Add your own listener in `useOverlayController` using `useLiveDataEvents` or a Tauri `listen('pss_event', ...)` wrapper.
   - For each incoming event, computes a logical trigger, e.g.:
     - `event.type === 'fight_loaded'` → `'fight_loaded'` trigger.
     - `event.type === 'winner'` → `'winner'` trigger.
   - Looks up matching rules from `useOverlayRoutingStore`.
   - For each rule, calls a helper that invokes the appropriate Tauri command:

     ```ts
     import { invoke } from '@tauri-apps/api/core';

     async function applyOverlayAction(overlay: OverlayId, action: 'show' | 'hide' | 'toggle') {
       switch (overlay) {
         case 'olympic':
           if (action === 'show' || action === 'toggle') await invoke('open_olympic_overlay_window');
           // optionally add a 'close' command for hide
           break;
         case 'modern':
           if (action === 'show' || action === 'toggle') await invoke('open_modern_overlay_window');
           break;
         case 'arcade':
           if (action === 'show' || action === 'toggle') await invoke('open_arcade_overlay_window');
           break;
         case 'none':
         default:
           break;
       }
     }
     ```

2. Mount `useOverlayController` at the top level of the SPA (e.g. in `App.tsx`), so it is active whenever the main app is running.

**Checklist:**

- [ ] PSS events now trigger overlay window actions according to the rules defined in the settings tab.
- [ ] No infinite loops or double‑invocation of PSS handlers (ensure you only react once per event).

---

## 7. Phase 6 — Persist overlay routing config

**Goal:** Save/load the operator’s trigger → overlay mapping between app runs.

**Files:**

- Backend: new functions in `src-tauri/src/tauri_commands_overlays.rs`.
- Frontend: `ui/src/stores/overlayRoutingStore.ts`.

**Tasks:**

1. Backend Tauri commands:
   - `get_overlay_routing_config() -> OverlayRoutingConfig`.
   - `set_overlay_routing_config(config: OverlayRoutingConfig)`.
   - Store this config either:
     - In an existing config table/file under `App.config_manager()`, or
     - In a dedicated table (if you already have overlay templates in DB, you can reuse that for routing metadata).
2. Frontend store:
   - On app start, fetch the config once (`invoke('get_overlay_routing_config')`) and call `setRules(...)`.
   - Whenever a rule is changed in the settings UI, debounce and call `set_overlay_routing_config` with the new config.

**Checklist:**

- [ ] Overlay routing rules survive app restarts.
- [ ] Changes in the settings UI are reflected in stored config within a reasonable delay (e.g., 500–1000 ms debounce).

---

## 8. Phase 7 — Polishing & Testing

**Goal:** Ensure the whole system is smooth, realtime, and OBS‑friendly.

**Tasks:**

1. **Latency check:**
   - With only the Olympic overlay window open and the main app running, fire a test PSS event sequence (FightLoaded → Athletes → MatchConfig → Clock start).
   - Verify:
     - WebSocket server shows `Broadcasting message to 1 connected clients` for each event.
     - Overlay updates within a frame or two (visually “instant”).
2. **Timer behavior:**
   - Confirm that the overlay’s timer only changes when PSS `clock` events arrive (no synthetic ticking).
   - Ensure there are no gaps in `clock` events while the overlay is connected.
3. **OBS integration:**
   - Open each overlay window.
   - In OBS, set up Window Capture sources for each window and apply a Chroma Key filter using the background color (`#00ff00`).
   - Confirm that flags, scores, and category info look correct and respond to live PSS events.
4. **Error handling:**
   - Ensure that if the overlay window is closed manually, the system doesn’t panic.
   - Reopening an overlay window after a fight has already started should render the *current* state as soon as it connects.
5. **Logging hygiene:**
   - Make sure debug logging for overlays is controllable (you already have `window.ScoreboardFieldLogging`, `ScoreboardDebugLogging`, etc.). Leave them `false` by default for production.

---

## 9. Guardrails & Non‑Goals

- Do **not**:
  - Introduce new global event buses beyond what already exists (`pss_event` Tauri events, WebSocket, localStorage broadcast) unless strictly necessary.
  - Change the semantics of PSS events; overlays and SPA must remain faithful to the live PSS stream.
  - Add heavy caching layers on the backend that delay events.
- Do:
  - Keep overlay windows lightweight (no heavy React state inside them unless explicitly decided).
  - Prefer explicit, testable Tauri commands over implicit behavior.
  - Use small, reviewable pull requests/changesets per phase.

---

## 10. Quick Iteration Summary (for the next agent)

If you need a bare‑bones TODO list to follow:

1. Add `open_olympic_overlay_window` Tauri command + SPA button → verify Olympic window.
2. Add `open_modern_overlay_window` / `open_arcade_overlay_window` + SPA controls → verify both.
3. Refactor overlay window creation into a helper (`get_or_create_overlay_window`).
4. Create `OverlayRoutingConfig` types + `useOverlayRoutingStore` Zustand store.
5. Build `OverlayRoutingSettings` UI and mount it inside the existing **PSS → Scoreboard** sub‑tab (no new top‑level tab).
6. Implement `useOverlayController` hook that listens to PSS events and calls Tauri overlay commands per routing rules.
7. Add Tauri commands to persist overlay routing config and wire them into the store.
8. Test end‑to‑end with live PSS data and OBS window capture, adjusting any small issues in window sizing, background color, or SVG scaling.

Follow these steps in order, and you’ll get three dedicated overlay windows plus a trigger‑to‑overlay routing system that operates in realtime with minimal overhead.
