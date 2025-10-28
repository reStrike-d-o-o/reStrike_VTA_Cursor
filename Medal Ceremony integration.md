# Medal Ceremony Integration Guide

## 1. Overview
Describes architecture and integration points for the Medal Ceremony module: data model, backend services, frontend UI, asset management, and external screen playback. Use this document to implement, test, and maintain the feature.

## 2. User Experience Summary
- Entry point: OVR → Medal Ceremony tab (mirrors Integration tab styling).
- Global controls: background/break image selectors, animation settings, show external screen toggle, prepare/play buttons.
- Per-division fieldsets: division selector, medalist rows (gold/silver/bronze x2) with IOC auto-fill and asset suggestions.
- Playlist workflow: save → prepare → play next, with “played” state tracking.
- Asset management: OVR → Animation Management (flag Lottie files) and OVR → Anthem Management (audio).
- External display: full-screen window on second monitor showing background, animations, anthem playback, photo countdown, break image.

## 3. Data Model & Persistence
- Tables (initial draft):
  - `medal_ceremonies` (id, tournament_id FK, name/title, background_path, break_path, animation_duration, animation_speed, photo_time, show_external, created_at, updated_at).
  - `medal_ceremony_divisions` (id, ceremony_id FK, division_id FK, order_index, prepared_at, completed_at).
  - `medal_ceremony_medalists` (id, division_entry_id FK, medal_type ENUM [gold, silver, bronze1, bronze2], athlete_id FK, ioc_code, flag_asset, anthem_asset, created_at).
  - Support tables for playback state logs if needed.
- Migrations created via sqlx; ensure indexes for lookups (ceremony_id, division_id, order_index).

## 4. Backend Services
- **Repositories / Services**
  - Ceremony CRUD (create/update/delete divisions/medalists, attach assets).
  - Lookup services for divisions, athletes filtered by division/tournament; fetch IOC data.
  - Asset management services (list/upload/delete flag animations and anthems).
  - Playlist preparation: validate saved state, build ordered list (division name + gold IOC), mark played items.
  - Playback control: play next, mark item complete, toggle show external screen, manage global settings.
  - External screen window controller (spawn window, handle show/hide, pass playlist data).
- **Tauri commands**
  - `medal_ceremony_get`, `medal_ceremony_save`, `medal_ceremony_delete`, `medal_ceremony_prepare`, `medal_ceremony_play_next`, `medal_ceremony_toggle_external`, `medal_ceremony_load_assets`, etc.
  - Asset commands for Animation/Anthem management (list/upload/remove).

## 5. Frontend Architecture
- **Stores (Zustand)**
  - `useMedalCeremonyStore` handling ceremony form state, divisions, medalists, asset suggestions, playlist state.
  - `useExternalScreenStore` to reflect playback status, connection to external window.
- **Components**
  - `MedalCeremonyTab` (main container).
  - `GlobalSettingsCard` (background/break selectors, duration/speed/photo inputs).
  - `DivisionFieldset` (division selector, order input, medalist rows with searchable dropdowns and auto IOC flag).
  - `MedalistRow` (combines athlete dropdown, IOC label, asset selectors).
  - Playlist summary and action buttons.
  - Asset management components under OVR (similar to IVR flag manager).
- **Reusable UI**
  - Searchable dropdowns backed by existing OVR components.
  - File pickers using Tauri dialog plugin.

## 6. External Screen Playback
- Implement external-screen window (React or HTML template) that receives:
  - Background/break image paths.
  - Per-play item: flag animations, anthem path, medalist data.
  - Global settings (duration, speed, photo time).
- Rendering pipeline:
  1. Fade in background.
  2. Play Lottie flags (Silver left, Gold center elevated, Bronze pair) with configured animation speed.
  3. Start anthem audio (gold).
  4. On completion, display photo countdown overlay.
  5. Fade to break image before marking item complete.
- Handle toggling: if external screen disabled during playback, gracefully stop audio and hide window.

## 7. Asset Management
- Locations: `ui/public/assets/animations`, `ui/public/assets/anthems`.
- UI features:
  - List assets grouped by IOC code.
  - Upload (validate extension `.json` for animations, audio formats for anthems).
  - Delete with confirmation (ensure not in use before removal).
  - Search/filter by IOC.
- Backend: file operations with validation, metadata stored either in DB or derived from filenames.

## 8. Workflow & State Handling
- Ceremony must be **saved** before preparation.
- `Prepare` generates playlist; only unplayed items remain playable.
- If ceremony changes after preparation, require re-save + re-prepare, preserving played status (finished items remain skipped).
- `Play Next` auto-enables external screen, executes next item, and marks as played.
- Provide clear UI feedback (disabled states, tooltips) to guide operators.

## 9. Testing Strategy
- Backend: unit tests for playlist generation, asset matching, state transitions; integration tests for Tauri commands with SQLite test DB.
- Frontend: tests for division CRUD, validation (missing medalist, missing flag/anthem), playlist status, external screen toggle behavior.
- Manual QA scripts covering: multi-division ceremonies, editing after preparation, asset fallback, external screen unplugged, re-preparation with already played items.

## 10. Rollout
- Feature flag to hide module until fully tested.
- Migration strategy with rollback instructions.
- Provide sample ceremony template and assets for demos.

## 11. Future Enhancements
- Support for multiple ceremonies per tournament with scheduling.
- Automatic detection of anthem length, fade transitions.
- Integration with OBS or overlay streaming.
- Operator metrics (time per ceremony, errors).
