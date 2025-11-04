# AGENTS.md - Orchestrator workspace guide for AI collaborators

## Project at a glance
- Rust workspace delivering a native GTK4 control center for OBS.
- Crates: `crates/app` (Relm4 application shell), `crates/core` (domain models), `crates/obs` (obs-websocket integration and session manager runtime), `crates/storage` (JSON persistence), `crates/ui-widgets` (reusable GTK components).
- Key crates: `relm4`, `relm4-components`, `relm4-icons`, `tokio`, `obws`, `serde`, `tracing`.

## Build / test / run
- Build everything: `cargo build --all`.
- Faster feedback: `cargo check --all`.
- Run the GTK shell: `cargo run -p orchestrator-app`.
- Format before pushing: `cargo fmt`.
- (Tests will arrive later) `cargo test --all`.

## UI patterns and conventions
- Main workspace: top command staging bar (red accent), right recording deck (blue accent), remaining streaming deck surface. Treat all three as drop targets and keep their helper copy in sync with interactions.
- The command staging rail is a `gtk::Overlay` with a pastel highlight reveal; keep the overlay height generous so the entire panel accepts drops and hide the helper label when empty.
- Drag & drop: command toggles emit `cmd:<id>` payloads, subscription toggles emit `sub:<id>`. Commands only land on the staging rail or directly onto a specific connection card; deck surfaces reject command drops and should steer users back toward the staging rail. Subscriptions land on recording/streaming decks when at least one matching connection is enabled. Route handling through `AppMsg::ProcessDrop` so hints and highlights stay current.
- Recording deck safety: command drops on the recording deck must trigger `AppMsg::PromptRecConfirmation` (see `show_rec_confirmation`) before the staged command is accepted.
- Deck cards pull state from the `telemetry: HashMap<u32, ObsTelemetry>` cache. `AppMsg::TelemetrySnapshot` replaces this map and then calls `rebuild_decks`; any new runtime stats must flow through the session manager snapshot.
- VU meters live in `connection_vu_meters`; reuse the existing handle per connection, call `set_levels` with fresh telemetry, and `widget.unparent()` before appending so GTK can safely reparent the drawing area.
- The settings popover drives `AppMsg::ToggleAutoReconnect`, `AppMsg::UpdateStatsInterval`, and `AppMsg::UpdateStatusInterval`. Keep the adjustments (`stats_adjustment`, `status_adjustment`) in sync with persisted `GlobalSettings`.
- Harness Relm4 patterns. New custom widgets must live in `crates/ui-widgets` and be reusable components. Use factories/components over ad-hoc widget construction in the app crate.
- Connection editing rows must use `orchestrator_ui_widgets::ConnectionRow`. The row layout is a single horizontal strip of entry fields (`Name`, `Address`, `Port`, `Password`), the STR/REC toggle, the enabled switch, and the trailing removal button. Keep widgets at uniform height; no extra labels.
- Newly created connections start disabled; require an explicit toggle before becoming active.
- Drop zone highlights use `trigger_highlight` to flash a translucent overlay instead of CSS hacks.
- Toast notifications originate from `enqueue_toast`; they stack at the lower-left and newest entries appear last.
- Command staging buttons live in `staged_commands`; use `AppMsg::RemoveStagedCommand` to drop entries and always rebuild via `rebuild_command_stage` after mutations so the flow box mirrors state.
- Persisting connection edits must call `session_manager.upsert_connection` (or `remove_connection`) so the obs runtime restarts immediately with new credentials.
- Recording deck sits on the right and must remain roughly one-third of the main width. Use accent bars (`destructive-action` for REC, `suggested-action` for STR) instead of container borders, and rely on `dashed_separator` helpers for section dividers.
- Connection deck tiles are plain GTK buttons (no `card` class) with top-right status pills and metric rows. Reuse the Fluent monochrome icons bundled via `relm4-icons` plus the custom Wi-Fi SVG stored at `crates/app/assets/icons/wifi-1-24-regular.svg`.
- Subscriptions assigned per-connection are tracked in `connection_subscriptions`. Always update them via `add_subscription_to_connection` and refresh the decks so FlowBox chips stay in sync.
- The connection role toggle defaults to STR (Streaming). When toggled it switches between STR (blue, suggested-action) and REC (red, destructive-action) states and must be persisted via `ConnectionPrimaryRole`.
- When the sidebar collapses the divider disappears and the main panel stretches to the window edge. Keep the empty-state copy uppercase and dimmed.
- The theme controls rely on `gtk::Settings::set_gtk_application_prefer_dark_theme`. The header toggle mirrors the OS preference on startup and flips between the mono sun/moon icons.
- Icons come exclusively from `relm4-icons`. Call `relm4_icons::initialize_icons(icon_names::GRESOURCE_BYTES, icon_names::RESOURCE_PREFIX)` during startup, keep choices monochrome, and rely on the bundled Fluent glyphs (e.g., `icon_names::ADD_SQUARE_REGULAR` / `icon_names::SUBTRACT_SQUARE_REGULAR`) for connection management controls.
- The Wi-Fi status glyph is brought in via `MetricIcon::Svg(WIFI_ICON_BYTES)`; keep the asset in sync with `crates/app/assets/icons/wifi-1-24-regular.svg` if we refresh icons.
- Avoid custom CSS. Prefer GTK/Adwaita classes (`pill`, `suggested-action`, `destructive-action`) and structural accents (e.g., slim coloured boxes) to signal state.

## Policy
- Rust 2021, async via `tokio`. Never block the async runtime.
- OBS traffic flows only through `obws`; do not open other sockets.
- UI work stays on the main thread; heavy work moves to async tasks.
- Configuration resides under the platform config directory through `orchestrator-storage`.
- Logging uses `tracing`; configure the subscriber in `orchestrator-app` only.
- Documentation for UI behaviour belongs under `docs/` (see `docs/gtk-desktop-control-center.md`).

## Asset sources
- Primary icon set: https://github.com/Relm4/icons (mirrored by the `relm4-icons` crate). When you need an icon, prefer constants from `relm4_icons::icon_names`.
- If an asset is missing, document the gap and align with OBS branding before introducing external art.

## File access rules
- Allowed: `docs/**`, `crates/**`, workspace root configs (`Cargo.toml`, `.gitignore`, `AGENTS.md`).
- Avoid modifying: future `LICENSE`, build outputs (`target/**`).

## Workflow expectations
- Keep changes scoped and reviewable; ask questions when direction is unclear.
- Run `cargo fmt` and (when relevant) `cargo check` before handing work back.
- Respect existing uncommitted changes—never revert user work unless instructed.
- Align terminology with the OBS WebSocket protocol.

## Rust installation (for humans)
1. Install Rustup from https://rustup.rs/ (Windows users can run the installer `.exe`).
2. Accept the stable toolchain and ensure PATH updates during setup.
3. Open a new terminal and verify with `rustc --version` and `cargo --version`.
4. Follow the GTK4 Windows guide before building GTK UI code: https://gtk-rs.org/gtk4-rs/git/book/installation_windows.html#set-rust-toolchain-to-msvc.
