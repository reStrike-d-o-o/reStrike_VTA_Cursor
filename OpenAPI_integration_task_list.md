# OpenAPI Integration Task List

## Overview
This document tracks the implementation progress for the OpenAPI communication interface integration. Each section lists actionable tasks, their status, and notes about implementation details, blockers, or follow-up work. Update this file whenever you complete a task or discover new work.

## 1. Discovery & Planning
- [x] Review OpenAPI 3.1 specification updates, REST best practices, and encoding requirements.
- [x] Outline high-level implementation plan and obtain approval.
- [x] Document validation libraries, serialization tools, and conversion approach for Rust and frontend stack. *(Using `serde_json`/`serde_yaml`, `openapiv3`, `serde_path_to_error`, and custom structural checks for semantic validation.)*

## 2. Backend (Rust / Tauri)
- [x] Create `openapi_manager` module responsible for schema storage, conversion (YAML ↔ JSON), validation, and UTF-8 enforcement. *(Implemented with canonical JSON/YAML persistence, metadata tracking, and validation pipeline.)*
- [x] Implement persistent storage for active schema (config path or database) with versioning and rollback handling. *(Schemas stored under `config/openapi/` with synchronized YAML/JSON files and metadata timestamps.)*
- [x] Add Tauri commands providing CRUD operations (load, save, validate, upload, export, format switch) and structured error reporting (path, line, column). *(New `tauri_commands_openapi` module exposes full lifecycle commands with detailed diagnostics.)*
- [x] Integrate schema application with external communication layer so published schemas drive the REST interface. *(Background Axum server serves `/openapi.json|yaml` and health endpoint, refreshed on every save.)*
- [ ] Write unit/integration tests covering validation, persistence, and external interface refresh.

## 3. Frontend (Settings UI)
- [x] Refactor App Settings tab into sub-tabs: move existing content into **Visual & Localization**, add new **System** sub-tab. *(Settings drawer now renders two-tier layout with preserved visual styling.)*
- [x] Implement schema editor console in **System** tab with YAML/JSON toggle, upload, download/export, validation, and save actions (styling aligned with Diagnostic & Logs manager). *(Console editor, action bar, and endpoints summary added using existing theme tokens.)*
- [x] Display validation results with line/column references and user feedback (toasts, error list). *(System tab surfaces validation status, error list with pointer/line/column, and success messaging.)*
- [x] Ensure UTF-8-only enforcement for uploads and edits, with clear error messages. *(Backend enforces UTF-8, frontend surfaces friendly errors for incompatible files.)*

## 4. Documentation & Developer Experience
- [x] Update task list (this file) as milestones complete with code references and notable decisions. *(Ongoing updates added after each milestone.)*
- [x] Produce `OpenAPI integration.md` describing architecture, modules, workflows, and usage guide. *(Document added alongside this task list for developer onboarding.)*
- [ ] Add examples / sample schemas for QA and provide guidance for external integrators.

## 5. QA & Release Preparation
- [ ] Execute regression tests (backend, frontend) and verify schema application against external communication endpoints.
- [ ] Confirm export/import workflows on Windows/macOS/Linux environments.
- [ ] Prepare release notes and migration instructions.

---

### Notes
- Only UTF-8 encoding is allowed for schema files; validation routines must reject other encodings.
- Adjust tasks as new requirements emerge, keeping this list aligned with project status.
