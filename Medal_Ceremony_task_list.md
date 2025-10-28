# Medal Ceremony Integration Task List

## Overview
Tracks milestones for the Medal Ceremony module: backend data/model work, UI, external screen integration, asset management, testing, and documentation. Update this file with progress notes, code references, and follow-ups after each milestone.

## 1. Planning & Requirements
- [x] Define module scope, UI/UX expectations, playback flow, and dependencies. *(Plan approved by stakeholder.)*
- [x] Produce high-level implementation plan (approved).
- [ ] Document asset requirements (flag animations, anthem files) and sourcing guidelines.

## 2. Backend Foundations
- [ ] Add database schema (medal_ceremonies, divisions, medalists, global settings) with migrations.
- [ ] Implement Rust models/services for CRUD, playlist preparation, playback state, asset discovery.
- [ ] Expose Tauri commands for Medal Ceremony CRUD, playlist prepare/save, playback controls, asset listings.
- [ ] Manage external screen window lifecycle and data pipeline for ceremony playback.

## 3. Frontend OVR Module
- [ ] Create Medal Ceremony tab replicating OVR Integration styling with Background/Break selectors.
- [ ] Implement expandable division fieldsets with medalist selectors, order editing, validation, CRUD.
- [ ] Add global settings controls (animation duration, speed, photo time) and action buttons (Prepare, Show external, Play Next).
- [ ] Build Zustand store for ceremony state, playlist, and playback status.

## 4. Asset Management UI
- [ ] Introduce OVR → Animation Management (Lottie flag assets) with CRUD/upload.
- [ ] Introduce OVR → Anthem Management with CRUD/upload.
- [ ] Auto-link flag/anthem fields to asset libraries (filter by IOC code).

## 5. External Screen Playback
- [ ] Create full-screen window rendering backgrounds, animations, anthem playback, transitions.
- [ ] Implement playlist execution (Play Next), handle show/hide toggle, and item state (played/unplayed).
- [ ] Support global settings for animation duration/speed and photo countdown.

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
