# Software Requirements Specification

> **Note:** All architecture, onboarding, and coding conventions are defined in .cursor/rules/context.mdc (single source of truth). Project is Windows-only; Docker/devcontainer is fully removed. All onboarding, build, and documentation reference Windows-native setup only.

## Prerequisites
- **Operating System:** Windows 10/11 (Windows recommended for full feature support)
- **Node.js:** v24 or higher (latest LTS recommended)
- **Rust:** Stable (install via [rustup.rs](https://rustup.rs/))
- **Tauri CLI:** Install with `cargo install tauri-cli`
- **Frontend:** React 18, TypeScript, Zustand, Tailwind CSS, framer-motion
- **Bundler:** Tauri
- **react-scripts:** 5.0.1 (required for React 18 compatibility)

## System Design
- **Modules & Responsibilities**  
  - **Core Bus (Microkernel)**  
    - Central event router; loads and manages plugins.  
  - **UDP Plugin**  
    - Rust-based listener on configurable IPv4 interface; parses PSS datagrams against TXT schema.  
  - **OBS Plugin**  
    - Manages one or more OBS Studio instances via WebSocket; handles buffer clipping on demand.  
  - **Playback Plugin**  
    - Shell-invokes `mpv` with `--start=10`; hides/restores UI.  
  - **Event Store Plugin**  
    - Persists events in SQLite; superfast bulk writes; exposes query API.  
  - **AI Analysis Plugin**  
    - Tags incoming events; prepares for future video-content AI modules.  
  - **Flag Management Plugin**  
    - Manages IOC flag recognition and display; handles flag downloads and updates.
  - **UI Overlay**  
    - Tauri + React front-end; docks left/right; global shortcuts; collapsed/expanded modes.  
  - **License Plugin**  
    - Hardware-locked activation via REST; periodic background validation with offline grace.  
  - **Settings & Diagnostics**  
    - Single tabbed panel; network, protocol file, OBS creds, shortcuts, log-viewer.

## Architecture pattern
- **Microkernel (Plugin) Architecture**  
  - Lightweight core managing lifecycle and inter-plugin events.  
  - Plugins are independently testable, updatable, and deployable.  
- **Layered within Plugins**  
  1. **Infrastructure** (Rust/Node I/O, WebSocket, SQLite)  
  2. **Domain Logic** (parsing, OBS commands, licensing rules, flag management)  
  3. **Application API** (commands/events published to bus)  
  4. **Presentation** (UI plugin subscribes to events, issues commands)

## State management
- **Frontend (React)**  
  - **Zustand** for simple, scalable stores:  
    - `useUdpEventsStore`, `useObsStatusStore`, `useUiStore`, `useLicenseStore`, `useFlagStore`  
  - Plugins expose commands via Tauri; UI subscribes to bus events.  
- **Backend (Rust)**  
  - **tokio::sync::broadcast** channel for inter-plugin events.  
  - Each plugin maintains minimal internal state, responds to messages via the bus.

## Data flow
1. **UDP datagram** → UDP Plugin parses → emits `EventParsed` on bus.  
2. **EventParsed** → Event Store persists → emits `EventStored` → UI subscribes → updates table.  
3. **User clicks "Replay"** → UI invokes Tauri command → Core Bus → OBS Plugin extracts buffer clip → emits `ClipReady` → Playback Plugin launches `mpv`.  
4. **OBS status change** → OBS Plugin emits `ObsStatus` → UI store updates record button animation.  
5. **Manual Mode toggle** → UI confirms → emits `ManualModeToggled` → Core Bus → UI enters editable mode.
6. **Flag display** → UI requests flag → Flag Plugin provides flag URL → UI displays flag image.

## Technical Stack
- **Shell & IPC**: Tauri (Rust backend + Node.js runtime)  
- **UI**: React + Tailwind CSS + framer-motion  
- **State**: Zustand (frontend) + tokio broadcast (backend)  
- **Protocol Parsing**: Rust module loading TXT schema at runtime  
- **Database**: SQLite via `rusqlite` (backend)  
- **OBS Integration**: `obs-websocket-rs` plugin  
- **Playback**: `mpv` via Tauri's `shell` API  
- **Licensing**: Rust HTTP client (`reqwest`) for REST; fingerprint via `sysinfo` + `machine_uid`  
- **Hotkeys**: `tauri-plugin-global-shortcut`
- **Flag Management**: IOC flag recognition and display system

## Authentication Process
- **Activation Flow**  
  1. UI prompts for license key → Tauri → License Plugin POST `/api/activate` with fingerprint  
  2. Server returns JWT + expiry → stored encrypted in filesystem  
- **Validation Flow**  
  - On startup & daily: License Plugin POST `/api/validate`  
  - If offline: track days since last success; warn after 5 days; disable after 7  
- **Revocation**  
  - Server can revoke keys; on validation failure UI locks down and prompts reactivation

## Route Design
- **Internal (Tauri Commands)**  
  - `udp:start(iface,port)`, `obs:cmd(action,params)`, `replay:play(recId)`, `license:activate(key)`, `settings:update(opts)`, `flag:get(iocCode)`  
- **Event Topics**  
  - `EventParsed`, `EventStored`, `ObsStatus`, `ClipReady`, `LicenseStatus`, `UiStateChange`, `FlagUpdated`  
- **External REST**  
  - `POST /api/activate`  
  - `POST /api/validate`  
  - `GET /api/license-info`

## API Design
- **Tauri Command Handlers** (Rust)  
  ```rust
  #[tauri::command]
  fn obs_cmd(action: String, params: JsonValue) -> Result<(), Error> { /* … */ }

  #[tauri::command]
  fn replay_play(recording_id: i64) -> Result<(), Error> { /* … */ }

  #[tauri::command]
  fn get_flag_url(ioc_code: String) -> Result<String, Error> { /* … */ }
```

## Known Issues & Compatibility
- **'cargo' is not recognized:** Rust is not installed or not in PATH. Install from [https://rustup.rs/](https://rustup.rs/), restart terminal.
- **'react-scripts' is not recognized:** Run `npm install react-scripts@5.0.1 --save-dev` in `ui`, then `npm install`.
- **TypeScript/JSX errors:** Install `@types/react` and `@types/react-dom` as dev dependencies in `ui`.
- **Directory confusion:** Always run commands from the correct project root (`reStrike_VTA_Cursor`).
- **Windows is recommended** for OBS and mpv integration; some features may not work on Mac/Linux.

## AdvancedPanel (Frontend)
The AdvancedPanel is now a drawer-based UI:
- Left sidebar with icons for: PSS, OBS, Video, AI Analyzer, Settings
- Right content area for selected drawer
- Each drawer is a modular section for advanced features

See [FRONTEND_DEVELOPMENT_SUMMARY.md] and [ui-design-document.md] for details.
