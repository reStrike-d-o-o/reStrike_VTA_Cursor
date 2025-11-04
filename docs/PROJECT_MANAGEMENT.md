# Project Management Plan - reStrike_TKS

## Objective
Deliver a native Rust desktop application that replicates the reStrike VTA functionality set using Tauri v2 for packaging/system integrations and Relm4/GTK for the primary control surface. The new stack must stay within the official Tauri plugin ecosystem, embrace GTK/Relm4 idioms, and preserve all automation, replay, and licensing behaviours from the legacy app.

## Functional Scope
### UDP Ingestion & Match Data
- Stand up a Tokio-based UDP listener that honours the editable PSS TXT schema.
- Detect network interfaces, allow manual interface binding, and surface connection health metrics.
- Parse datagrams into strongly typed domain events, enrich them with match context, and publish them onto the internal event bus.
- Persist raw payloads for diagnostics and expose a schema reloader that respects user-provided formats.

### OBS Session Control
- Manage multiple OBS WebSocket sessions via `obws` with reconnect, authentication, and TLS options.
- Provide buffered replay commands, scene switching, and manual overrides; enforce confirmation flows for recording actions.
- Surface telemetry (connection state, dropped frames, buffer fill percentage) to the UI.
- Use `tauri-plugin-global-shortcut` to trigger replay commands from hardware keys.

### Video Playback & Clip Pipeline
- Consume replay requests, orchestrate buffer extraction, and launch `mpv` with the correct offsets (10-second pre-roll by default).
- Auto-hide the GTK window while playback runs, restore focus on exit, and track playback history.
- Catalogue generated clips on disk with metadata matching the legacy naming format.

### Recording Archive & Timeline
- Store clip metadata, match logs, and annotations in SQLite via `sqlx` (runtime mode) fronted by `crates/storage`.
- Provide indexed queries for filters (match, athlete, weight class, event tags) and maintain derived data for fast timeline rendering.
- Implement a timeline visualiser that aligns UDP events, manual markers, and AI tags.

### AI Data Analyzer & Insights
- Rebuild the rule-based tagging engine for strike/penalty events, preparing hooks for future video AI.
- Generate insight panels (event summaries, heatmaps) populated by `crates/analytics`.
- Keep processing asynchronous; surface progress and errors to the UI toast system.

### Licensing & Activation
- Hardware fingerprinting using `tauri-plugin-stronghold` for secure storage and `machine_uid` for identifiers.
- Activation and periodic validation over HTTPS (via `tauri-plugin-http`), with offline grace periods.
- Lock down functionality on license failure and surface the status prominently in the UI.

### Flag Management System
- Ship IOC PNG assets, expose retrieval APIs, and support emoji fallbacks.
- Provide nightly update hooks (manual plus scheduled) using official Tauri scheduler APIs when available.

### Diagnostics & Logging
- Unified logging pipeline with `tracing` and `tauri-plugin-log`; expose log viewer in-app.
- Health dashboard summarising UDP/OBS/licensing status, including last successful operations.

## External Dependencies & Environment
- OBS Studio 29+ with WebSocket v5 plugin enabled.
- mpv CLI accessible via PATH.
- SQLite 3.x (bundled through the Rust crate backend).
- GTK4 runtime for Windows/macOS/Linux per official GTK-RS documentation.
- Network access for license validation and flag updates.

## Dependency Graph (High-Level)
1. Core event bus (`crates/core`) underpins all services.
2. Ingestion feeds events into storage and analytics.
3. OBS and playback rely on core for command dispatch and telemetry reporting.
4. UI (Relm4) subscribes to snapshots emitted by core; it never owns the long-lived async tasks.
5. Licensing gates command dispatch; all top-level commands perform a license check before execution.
6. Flag system and settings provide assets/config utilised by both UI and services.

## Phase Breakdown
### Phase 0 - Discovery & Environment
- Architecture & Platform: Finalise workspace layout, Tauri v2 configuration, and GTK bootstrap strategy.
- Environment Setup: Define Rust toolchain versions, GTK dependencies, mpv/OBS prerequisites, and VS Code settings.
- Protocol Audit: Validate the existing PSS TXT schema, catalogue custom fields, and confirm UDP source behaviour.
- Risk Register: Document integration risks (for example, OBS reconnect, mpv availability) and mitigation tactics.

### Phase 1 - Workspace Assembly
- Workspace Scaffolding: Generate `Cargo.toml` workspace, stub crates, and baseline CI (fmt/clippy/build).
- Tauri Shell Configuration: Configure `tks-shell` with official plugins (log, fs, global-shortcut, http, updater, store, single-instance).
- Relm4 UI Shell: Boot minimal GTK window in `tks-ui`; wire to Tauri event loop via channel bridge.
- Settings Infrastructure: Implement configuration loading and saving using `tauri-plugin-store` and typed accessors in `crates/storage`.

### Phase 2 - Core Services
- UDP Service: Implement listener, schema parsing, error recovery, and integration tests with captured datagrams.
- OBS Service: Build connection manager, command API, telemetry publisher, and reconnection policies.
- Event Bus: Establish `tokio::sync` channels, command/event enums, and service registration macros.
- Logging & Diagnostics: Configure tracing layers, log persistence, and health snapshot generation.

### Phase 3 - Video & Data Pipeline
- Playback Engine: Integrate mpv launcher, focus management, and fallback strategies when mpv is unavailable.
- Clip Catalogue: Design storage schema, implement migrations, and create repository methods for filtering and sorting.
- Timeline Builder: Map UDP events to timeline markers, compute derived statistics, and expose snapshot models.
- Flag Management: Import IOC assets, expose lookups, and integrate fallback emoji logic.

### Phase 4 - UI & Interaction Layer
- Screen Composition: Assemble status bar, control rail, replay canvas, insight column, and modal stack in Relm4.
- State Synchronisation: Bind event bus snapshots to Relm4 models, ensuring thread-safe message passing.
- Control Workflows: Implement manual mode, command confirmations, and error toasts.
- Accessibility & Shortcuts: Map keyboard shortcuts, high-contrast theming, and focus traversal.

### Phase 5 - AI Insights & Advanced Features
- Analytics Engine: Port legacy tagging rules, implement insight summaries, and expose data to UI components.
- Export & Reporting: Provide CSV/JSON exports for events and clip metadata using official Tauri filesystem APIs.
- Automation Hooks: Surface scheduled tasks (flag refresh, log rotation) respecting user preferences.

### Phase 6 - Quality, Packaging & Release
- Testing Strategy: Stand up unit tests, service integration tests, and smoke tests via `cargo tauri test` once available.
- CI/CD: Configure GitHub Actions for Windows/macOS/Linux builds, code quality gates, and artifact upload.
- Installer Pipeline: Use Tauri bundler to produce MSI/DMG/AppImage artefacts; document signing requirements.
- Release Checklist: Validate licensing, run regression scripts, update documentation, and provide upgrade notes.

## Deliverables per Functionality
- UDP: Functional listener, config UI, diagnostics.
- OBS: Multi-profile control, telemetry, shortcuts.
- Playback: Reliable mpv orchestration, clip archive.
- Archive: Filterable catalogue, timeline, export.
- AI Insights: Tagging rules, insight panels, future hook APIs.
- Licensing: Activation UI, offline grace, enforcement.
- Flags: Asset bundle, lookup API, update routine.
- Diagnostics: In-app log viewer, system status, crash capture.

## Definition of Done
- All critical flows (UDP ingest -> replay, manual command, clip review, licensing) demonstrated on Windows with OBS and mpv on the same machine.
- UI meets performance budget (replay command to playback under one second under normal conditions).
- Official plugin usage documented; no unofficial crates introduced without sign-off.
- Documentation (README, AGENTS, setup guides) reflects the implemented architecture.
- Test suite automated in CI and green.

## Open Questions
- Confirm whether Relm4 UI should support touch optimisations beyond GTK defaults.
- Decide on analytics storage granularity (per event vs aggregated bins).
- Determine hosting for the license validation service and whether mutual TLS is required.
- Clarify requirements for web fallback modes, if any, now that the stack is fully native.
