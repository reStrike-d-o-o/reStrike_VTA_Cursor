# OpenAPI Integration

## Overview

The application now exposes an OpenAPI-driven external communication surface that can be fully managed from the desktop UI.  
Key capabilities:

- Unified storage for the canonical OpenAPI schema (`config/openapi/schema.yaml` & `schema.json`).
- Round-trip conversion between JSON and YAML with UTF-8 enforcement.
- Structured validation (parse errors, semantic checks, OpenAPI 3.x deserialisation) with detailed diagnostics.
- A local Axum-based REST service serving the active schema and health probe.
- Desktop settings UI (Settings → App Settings → **System**) featuring a console editor, validation tooling, file upload/export, and format switching.

## Backend Architecture

### `openapi::manager::OpenApiManager`
- Loads and persists schemas under `config/openapi/`.
- Maintains canonical JSON tree to simplify conversions and exports.
- Synchronises YAML & JSON artifacts and stores metadata (`schema_state.json`) with timestamps and preferred format.
- Provides helpers for rendering documents in either format, exporting to arbitrary paths, and validating arbitrary payloads.

### Validation Pipeline (`openapi::validation`)
- Parses content with `serde_json` / `serde_yaml`, returning precise line/column data on syntax errors.
- Applies structural rules (presence of `openapi`, `info.title`, `info.version`, valid HTTP methods, responses, leading slash on path keys).
- Re-deserialises via `openapiv3::OpenAPI` using `serde_path_to_error` to capture JSON pointers for deeper semantic issues.
- Maps JSON pointers back to human-readable line/column by scanning the original payload to aid UI highlighting.

### Runtime server (`openapi::runtime::OpenApiRuntime`)
- Spins up an Axum server (default `127.0.0.1:8787`) on application start.
- Routes:
  - `GET /openapi.json` → canonical JSON spec.
  - `GET /openapi.yaml` → YAML rendering.
  - `GET /api/health` → `{"status":"ok","timestamp":"..."}`.
- Server state is updated after every successful save via `App::apply_openapi_schema`.

### Tauri Commands (`tauri_commands_openapi`)
Commands expose CRUD workflow to the frontend:

| Command | Purpose |
|---------|---------|
| `openapi_get_state` | Fetch current schema, validation snapshot, endpoints, metadata. |
| `openapi_save_schema` | Validate & persist schema; refresh runtime on success. |
| `openapi_validate_schema` | Run validation without persistence. |
| `openapi_upload_schema` | Read UTF-8 file, infer format, return content + validation. |
| `openapi_export_schema` | Write schema to user-selected path/format. |

Request/response structs live alongside the command definitions.

## Frontend UI & Commands

### `openApiCommands` (TypeScript)
- Wrapper functions over the new Tauri commands: `getState`, `saveSchema`, `validateSchema`, `uploadSchema`, `exportSchema`.
- Guard against non-Tauri contexts; all functions throw when Tauri is unavailable.

### Settings UI (`AppSettingsSection`)
- App Settings drawer now features sub-tabs: **Visual & Localization** (existing content) and **System** (OpenAPI management).
- The System tab presents:
  - Active schema endpoints (base/json/yaml/health) with consistent theming (Diagnostic & Logs style).
  - YAML/JSON toggle (loads fresh copy from backend; warns about unsaved changes).
  - Monospaced editor with dirty-state tracking.
  - Actions: Validate, Save (apply), Export, Upload (via `@tauri-apps/plugin-dialog`).
  - Feedback banners for success/error and detailed validation list (message, pointer, line, column).

## Schema Storage & Encoding

- Canonical files:  
  - `config/openapi/schema.yaml` (default format and authoritative source).  
  - `config/openapi/schema.json` (kept in sync).  
  - `config/openapi/schema_state.json` (format + `updated_at` metadata).
- Default schema seeded from `resources/openapi/default.yaml`.
- All I/O paths enforce UTF-8 (uploads reject non UTF-8 bytes; writes always encoded as UTF-8).

## External REST Interface

- Listening address is configured via `RE_STRIKE_OPENAPI_ADDR` env var (defaults to `127.0.0.1:8787`).
- Exposed endpoints respond immediately to schema updates (no restart required).
- Health endpoint can be used by monitoring or integration tests to confirm connectivity.

## Validation Notes

- Successful `openapi_save_schema` requires a fully valid schema. Partial/invalid schemas remain unsaved but are returned to the UI with diagnostics.
- Validation output contains:
  - `message`: human-readable summary.
  - `pointer`: JSON pointer compatible path (when available).
  - `line` / `column`: 1-based offsets into the submitted document.
- Semantic checks ensure OpenAPI version 3.x, populated `info` section, non-empty `paths`, supported HTTP verbs, and response definitions.

## Typical Workflow

1. Open Settings → App Settings → **System**.
2. Edit schema directly in the console or upload an existing OpenAPI file.
3. Use **Validate** to review errors with precise locations.
4. When validation passes, **Save** applies the schema:  
   - Files are rewritten in both JSON and YAML forms.  
   - Runtime server is refreshed.  
   - UI updates timestamps/endpoints.
5. Use **Export** to generate a copy for downstream systems (choose JSON or YAML).

## Reset & Recovery

- To revert to defaults, delete `config/openapi/schema.*` files; the app will reseed from `resources/openapi/default.yaml` on next launch.
- Validation guards prevent corrupt schemas from being persisted—users can fix issues in-place via the System tab.

## Further Enhancements (Follow-up Ideas)

- Automated integration tests hitting the Axum endpoints with multiple schema revisions.
- Optional Git-based snapshotting for schema history.
- Syntax highlighting for the editor (e.g., Monaco fork + tauri plugin).
- Extended validation using upstream OpenAPI JSON Schema once available offline. 
