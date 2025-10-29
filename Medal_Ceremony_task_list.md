# Medal Ceremony Integration Task List

## Overview
Tracks milestones for the Medal Ceremony module: backend data/model work, UI, external screen integration, asset management, testing, and documentation. Update this file with progress notes, code references, and follow-ups after each milestone.

## 1. Planning & Requirements
- [x] Define module scope, UI/UX expectations, playback flow, and dependencies. *(Plan approved by stakeholder.)*
- [x] Produce high-level implementation plan (approved).
- [ ] Document asset requirements (flag animations, anthem files) and sourcing guidelines.

## 2. Backend Foundations
- [x] Add database schema (medal_ceremonies, divisions, medalists, global settings) with migrations.
- [x] Implement Rust models/services for CRUD, playlist preparation, playback state, asset discovery.
- [x] Expose Tauri commands for Medal Ceremony CRUD, playlist prepare/save, playback controls, asset listings.
- [x] Manage external screen window lifecycle and data pipeline for ceremony playback. *(Added Tauri commands, event broadcasting, and external display listeners.)*

## 3. Frontend OVR Module
- [x] Create Medal Ceremony tab replicating OVR Integration styling with Background/Break selectors. *(Implemented in `ui/src/components/ovr/MedalCeremonyPanel.tsx` with background/break pickers.)*
- [x] Implement expandable division fieldsets with medalist selectors, order editing, validation, CRUD. *(Dynamic fieldsets using searchable dropdowns, order inputs, and medalist CRUD.)*
- [x] Add global settings controls (animation duration, speed, photo time) and action buttons (Prepare, Show external, Play Next). *(Controls wired to store; actions trigger backend.)*
- [x] Build Zustand store for ceremony state, playlist, and playback status. *(Extended store with asset/playlist management and Tauri event broadcasting.)*

## 4. Asset Management UI
- [x] Introduce OVR Animation Management (Lottie flag assets) with CRUD/upload. *(New `MedalCeremonyAnimationManager` tab.)*
- [x] Introduce OVR Anthem Management with CRUD/upload. *(New `MedalCeremonyAnthemManager` tab.)*
- [x] Auto-link flag/anthem fields to asset libraries (filter by IOC code). *(Medalist selection auto-populates default assets and dropdowns filter by IOC.)*

## 5. External Screen Playback
- [x] Create full-screen window rendering backgrounds, animations, anthem playback, transitions. *(Initial React external display with background/phase handling.)*
- [x] Implement playlist execution (Play Next), handle show/hide toggle, and item state (played/unplayed). *(Emit playback events, auto-open window, update state.)*
- [~] Support global settings for animation duration/speed and photo countdown. *(Animation duration & photo time applied; animation speed integration pending follow-up.)*

## 6. Testing & QA
- [ ] Backend unit/integration tests (playlist generation, DB persistence, command responses).
- [ ] Frontend component tests for form validation, playlist state management.
- [ ] Manual QA: multi-division ceremonies, asset fallback, external screen detection, repeated prepare/play flows.

## 7. Documentation & Release
- [ ] Draft `Medal Ceremony integration.md` covering architecture, APIs, UI instructions.
- [ ] Update operator guides/release notes.
- [ ] Add sample assets/demo data for testing.

## Notes
- Assets live under `ui/public/assets/animations` and `ui/public/assets/anthems`; ensure consistent naming with IOC codes.
- Ensure migrations handle existing deployments cleanly (no downtime).
- Consider feature flag to hide module until fully tested.
