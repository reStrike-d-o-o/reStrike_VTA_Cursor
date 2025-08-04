# Backend Architecture

## Overview

The reStrike VTA backend is built with Rust and Tauri v2, providing a native Windows desktop application with a modular plugin architecture. The system is designed for high performance, real-time event processing, and seamless integration with external systems like OBS Studio.

## Architecture

### Core Architecture
- **Tauri v2**: Modern desktop application framework
- **Plugin System**: Modular architecture with clear separation of concerns
- **Async/Await**: Tokio-based asynchronous programming
- **Error Handling**: Comprehensive error management with AppResult<T>
- **State Management**: Thread-safe shared state with Arc<Mutex<T>>

### Technology Stack
- **Language**: Rust (latest stable)
- **Framework**: Tauri v2 for native Windows integration
- **Async Runtime**: Tokio for asynchronous operations
- **Database**: SQLite with rusqlite
- **WebSocket**: tokio-tungstenite for OBS integration
- **Serialization**: Serde for JSON handling
- **Logging**: Structured logging with file rotation

## Directory Structure

```
src-tauri/
├── src/
│   ├── main.rs              # Tauri app entry point
│   ├── lib.rs               # Library exports and plugin registration
│   ├── tauri_commands.rs    # Tauri command definitions (4200+ lines)
│   ├── core/                # Core application functionality
│   │   ├── app.rs           # Application state and lifecycle
│   │   ├── config.rs        # Configuration management
│   │   └── state.rs         # Global state management
│   ├── config/              # Configuration system
│   │   ├── manager.rs       # Configuration manager
│   │   ├── types.rs         # Configuration types
│   │   └── mod.rs           # Configuration module
│   ├── logging/             # Logging system
│   │   ├── logger.rs        # Logging implementation
│   │   ├── rotation.rs      # Log rotation
│   │   ├── archival.rs      # Log archival
│   │   └── mod.rs           # Logging module
│   ├── plugins/             # Plugin modules
│   │   ├── mod.rs           # Plugin module registration
│   │   ├── plugin_obs.rs    # OBS WebSocket integration
│   │   ├── plugin_udp.rs    # UDP protocol handling
│   │   ├── plugin_database.rs # Database operations
│   │   ├── plugin_cpu_monitor.rs # System monitoring
│   │   └── plugin_license.rs # License management
│   ├── database/            # Database system
│   │   ├── connection.rs    # Database connection management
│   │   ├── migrations.rs    # Database migrations
│   │   ├── models.rs        # Data models
│   │   ├── operations.rs    # Database operations
│   │   └── mod.rs           # Database module
│   ├── types/               # Shared types
│   │   └── mod.rs           # Type definitions
│   └── utils/               # Utility functions
│       ├── logger.rs        # Logging utilities
│       └── network.rs       # Network interface detection
├── Cargo.toml               # Rust dependencies
├── tauri.conf.json          # Tauri configuration
├── capabilities.json        # Tauri capabilities
└── build.rs                 # Build script

simulation/                   # Hardware Simulator Module
├── core/                    # Core simulator implementation
│   └── tkstrike_hardware_simulator.py
├── config/                  # Configuration files
│   └── config.json
├── tests/                   # Test scripts
│   ├── test_simulator.py
│   ├── test_integration.py
│   └── quick_test.py
├── examples/                # Example usage scripts
│   └── example_usage.py
├── docs/                    # Documentation
│   ├── README.md
│   ├── INTEGRATION_GUIDE.md
│   ├── QUICKSTART.md
│   ├── INTEGRATION_SUMMARY.md
│   └── SIMULATOR_SUMMARY.md
├── main.py                  # Main entry point
└── requirements.txt         # Python dependencies
```

## Plugin System

### Plugin Architecture

The backend uses a modular plugin architecture where each plugin is responsible for specific functionality:

```rust
// Plugin trait for common interface
pub trait Plugin {
    fn name(&self) -> &str;
    fn initialize(&mut self) -> AppResult<()>;
    fn shutdown(&mut self) -> AppResult<()>;
    fn get_status(&self) -> PluginStatus;
}

// Plugin status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    Initialized,
    Running,
    Error(String),
    Stopped,
}
```

### Core Plugins

#### Simulation Integration
The backend includes comprehensive simulation support through Tauri commands that interface with the Python-based tkStrike Hardware Simulator:

```rust
// Simulation commands in tauri_commands.rs
#[tauri::command]
pub async fn simulation_start(
    mode: String,
    scenario: String,
    duration: u32,
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError>

#[tauri::command]
pub async fn simulation_stop(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError>

#[tauri::command]
pub async fn simulation_get_status(_app: State<'_, Arc<App>>) -> Result<serde_json::Value, TauriError>

#[tauri::command]
pub async fn simulation_send_event(
    event_type: String,
    params: serde_json::Value,
    _app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, TauriError>
```

**Features:**
- **One-click Simulation**: Start/stop simulation from PSS drawer
- **Multiple Scenarios**: Basic, championship, and training matches
- **Real-time Control**: Manual event generation and monitoring
- **Protocol Compliance**: Full PSS v2.3 protocol implementation
- **Integration**: Seamless integration with existing UDP and WebSocket systems
- **Robust Dependency Management**: Cross-platform Python detection and auto-installation
- **Enhanced Error Handling**: User-friendly error messages with actionable solutions

**Robust Environment Management:**
```rust
// utils/simulation_env.rs - Cross-platform environment detection
pub fn ensure_simulation_env() -> Result<(String, PathBuf), SimulationEnvError> {
    // 1. Detect Python command (python3, python, py)
    // 2. Verify Python version (>= 3.8)
    // 3. Check required packages
    // 4. Auto-install missing dependencies
    // 5. Resolve simulation paths relative to executable
}
```

**Error Handling:**
- **PythonNotFound**: Automatic detection of Python installation
- **PythonVersionTooLow**: Version verification with clear upgrade instructions
- **DependencyCheckFailed**: Package verification with auto-installation
- **PipInstallFailed**: Network error handling with retry options
- **SimulationPathNotFound**: Dynamic path resolution for bundled simulation files

**Usage:**
```bash
# Start simulation from command line
python simulation/main.py --mode demo --scenario basic --duration 30

# Or use the integrated UI in PSS drawer > Simulation tab
# The UI now provides retry and install dependency buttons
```

#### Database Plugin
```rust
#[derive(Clone)]
pub struct DatabasePlugin {
```

### Analytics System

#### Overview
The analytics system provides comprehensive performance metrics and insights for athletes, matches, tournaments, and daily operations. It processes real-time PSS events to generate statistics and trends.

#### Key Components

##### Event Processing Pipeline
- **Real-time Event Processing**: Automatic recalculation when new PSS events arrive
- **Efficient Data Filtering**: Memory-optimized statistics calculation
- **Event Timeline Analysis**: Comprehensive event tracking and analysis

##### Performance Metrics
- **Win Rates**: Match statistics and performance tracking
- **Points Scoring**: Performance trends and scoring analysis
- **Warning Tracking**: Discipline and rule violation monitoring
- **Match Intensity**: Efficiency metrics and completion rates

##### Database Operations
- **Connection Pooling**: Timeout mechanisms with retry logic
- **Robust Error Handling**: Comprehensive error management
- **Schema Version Management**: Proper migration system
- **Performance Optimization**: Efficient queries and indexing

#### Analytics Components

##### AthleteAnalytics
- Individual athlete performance metrics
- Win/loss statistics and win rate calculation
- Points scored and average points per match
- Warning and injury tracking
- Best performance tracking
- Performance trends analysis

##### MatchAnalytics
- Detailed match statistics and duration tracking
- Individual athlete performance within matches
- Event distribution analysis (points, warnings, injuries, other)
- Match intensity calculation (events per minute)
- Winner determination and match completion status
- Timeline analysis

##### TournamentAnalytics
- Overall tournament statistics and metrics
- Top 10 athletes by points with win rates
- Top 10 countries by performance
- Match completion rates and efficiency
- Event distribution across tournament
- Average match duration and intensity

##### DayAnalytics
- Daily performance metrics and statistics
- Hourly activity timeline with peak hour identification
- Top athletes of the day
- Day efficiency and completion rates
- Event distribution by type
- Match intensity and performance metrics

#### Technical Implementation

##### Backend Optimizations
- Connection pooling with timeout mechanisms
- Efficient database queries and indexing
- Robust error handling and recovery
- Memory-optimized event processing

##### Frontend Integration
- Real-time event processing from PSS data
- Efficient state management with React hooks
- Proper TypeScript typing for all components
- Responsive design patterns

##### Performance Features
- Efficient React rendering with proper dependency arrays
- Memoized calculations for expensive operations
- Lazy loading of analytics components
- Optimized re-rendering patterns

## Development Workflow

### Starting Development
```bash
# From project root - starts both frontend and backend
cd src-tauri
cargo tauri dev
```

This single command:
1. Starts React development server (port 3000)
2. Builds Rust backend
3. Launches native Windows application
4. Enables hot reload for both frontend and backend

### Alternative Manual Start
```bash
# Terminal 1: Start React dev server
cd ui
npm run start:fast

# Terminal 2: Start Tauri app
cd src-tauri
cargo tauri dev
```

### Build Commands
```bash
# Development build
cd ui
npm run build

# Production build with Tauri
cd src-tauri
cargo tauri build
```

### Environment Detection

#### Tauri API Detection
The application automatically detects whether it's running in native Windows mode or web mode:

```typescript
// ui/src/hooks/useEnvironment.ts
export const useEnvironment = () => {
  const [tauriAvailable, setTauriAvailable] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const checkTauriAvailability = async () => {
      try {
        // Check if Tauri API is available
        if (typeof window !== 'undefined' && window.__TAURI__) {
          setTauriAvailable(true);
        }
      } catch (error) {
        console.warn('Tauri API not available:', error);
      } finally {
        setIsLoading(false);
      }
    };

    checkTauriAvailability();
  }, []);

  return { tauriAvailable, isLoading };
};
```

### Performance Best Practices

#### Backend (Rust)
- Use optimized dev profile: `opt-level=1`, `codegen-units=256`, `incremental=true`, `lto=false`
- Enable incremental compilation and debug assertions
- Use `./scripts/development/fast-dev.sh` for fast dev cycles
- Clean build artifacts regularly (`cargo clean`)
- Monitor build times and optimize as needed

#### General
- Always use fast scripts for development
- Clean caches and artifacts weekly or when performance degrades
- Review and optimize imports and dependencies regularly

## Development Environment Management

### Overview
The development environment includes several tools to help manage port management, service monitoring, and cleanup utilities.

### Quick Start

#### Using the Main Wrapper Script
```bash
# Show all available commands
./scripts/dev.sh help

# Check current status
./scripts/dev.sh status

# Start all services
./scripts/dev.sh start-all

# Stop all services
./scripts/dev.sh stop-all

# Clean up environment
./scripts/dev.sh cleanup
```

### Available Tools

#### 1. Development Wrapper (`scripts/dev.sh`)

**Development Commands:**
- `start-frontend` - Start React frontend
- `start-backend` - Start Rust backend
- `start-all` - Start both frontend and backend
- `stop-all` - Stop all development servers

**Management Commands:**
- `status` - Show development environment status
- `ports` - List all ports and their status
- `services` - List all services and their status
- `cleanup` - Full cleanup (stop processes, clear cache)
- `quick-cleanup` - Quick cleanup (stop processes only)
- `health` - Run health checks

**Utility Commands:**
- `install-deps` - Install all dependencies
- `build` - Build the project
- `test` - Run tests
- `update-config` - Update configuration status

#### 2. Cleanup Script (`scripts/cleanup_dev_environment.sh`)

**Commands:**
- `--cleanup, -c` - Full cleanup (stop processes, clear cache, check ports)
- `--quick, -q` - Quick cleanup (stop processes only)
- `--status, -s` - Show current status
- `--help, -h` - Show help message

**Examples:**
```bash
# Full cleanup
./scripts/cleanup_dev_environment.sh --cleanup

# Quick cleanup (just stop processes)
./scripts/cleanup_dev_environment.sh --quick
```

## Windows Development Setup

### Prerequisites Checklist

#### System Requirements
- ✅ **Windows 10/11** (64-bit)
- ✅ **8GB RAM minimum** (16GB recommended)
- ✅ **10GB free disk space**
- ✅ **Administrator privileges** (for installation)
- ✅ **Internet connection** (for downloads)

#### Required Software
- ✅ **VSCode** (Latest version)
- ✅ **Node.js** (v24+ LTS)
- ✅ **Rust** (Latest stable)
- ✅ **Git** (Latest version)
- ✅ **Python** (v3.8+ for scripts)
- ✅ **OBS Studio** (v29+ for testing)
- ✅ **mpv** (Windows build for video playback)

### Installation Steps

#### 1. Install Core Development Tools

**VSCode Extensions to Install:**
- **Rust Analyzer** (rust-lang.rust-analyzer)
- **TypeScript and JavaScript Language Features** (built-in)
- **ES7+ React/Redux/React-Native snippets** (dsznajder.es7-react-js-snippets)
- **Tailwind CSS IntelliSense** (bradlc.vscode-tailwindcss)
- **GitLens** (eamodio.gitlens)
- **Thunder Client** (rangav.vscode-thunder-client)
- **Error Lens** (usernamehw.errorlens)
- **Auto Rename Tag** (formulahendry.auto-rename-tag)

**Installation Commands:**
```powershell
# Node.js
# Download from: https://nodejs.org/en/download/
node --version  # Should show v24.x.x
npm --version   # Should show 10.x.x

# Rust
# Download rustup-init.exe from: https://rustup.rs/
rustc --version  # Should show rustc 1.75.x
cargo --version  # Should show cargo 1.75.x

# Git
# Download from: https://git-scm.com/download/win
git --version

# Python
# Download from: https://www.python.org/downloads/
# IMPORTANT: Check "Add Python to PATH" during installation
python --version  # Should show Python 3.8+
pip --version
```

#### 2. Install Tauri CLI and Dependencies

```powershell
# Install Tauri CLI
cargo install tauri-cli

# Install frontend dependencies
cd ui
npm install

# Install backend dependencies
cd ../src-tauri
cargo build
```

#### 3. Verify Installation

```powershell
# Check all tools are available
node --version
npm --version
rustc --version
cargo --version
git --version
python --version

# Test Tauri development
cd src-tauri
cargo tauri dev
```

## Software Requirements Specification

### Prerequisites
- **Operating System:** Windows 10/11 (Windows recommended for full feature support)
- **Node.js:** v24 or higher (latest LTS recommended)
- **Rust:** Stable (install via [rustup.rs](https://rustup.rs/))
- **Tauri CLI:** Install with `cargo install tauri-cli`
- **Frontend:** React 18, TypeScript, Zustand, Tailwind CSS, framer-motion
- **Bundler:** Tauri
- **react-scripts:** 5.0.1 (required for React 18 compatibility)

### System Design
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

### Architecture Pattern
- **Microkernel (Plugin) Architecture**  
  - Lightweight core managing lifecycle and inter-plugin events.  
  - Plugins are independently testable, updatable, and deployable.  
- **Layered within Plugins**  
  1. **Infrastructure** (Rust/Node I/O, WebSocket, SQLite)  
  2. **Domain Logic** (parsing, OBS commands, licensing rules, flag management)  
  3. **Application API** (commands/events published to bus)  
  4. **Presentation** (UI plugin subscribes to events, issues commands)

### State Management
- **Frontend (React)**  
  - **Zustand** for simple, scalable stores:  
    - `useUdpEventsStore`, `useObsStatusStore`, `useUiStore`, `useLicenseStore`, `useFlagStore`  
  - Plugins expose commands via Tauri; UI subscribes to bus events.  
- **Backend (Rust)**  
  - **tokio::sync::broadcast** channel for inter-plugin events.  
  - Each plugin maintains minimal internal state, responds to messages via the bus.

### Data Flow
1. **UDP datagram** → UDP Plugin parses → emits `EventParsed` on bus.  
2. **EventParsed** → Event Store persists → emits `EventStored` → UI subscribes → updates table.  
3. **User clicks "Replay"** → UI invokes Tauri command → Core Bus → OBS Plugin extracts buffer clip → emits `ClipReady` → Playback Plugin launches `mpv`.  
4. **OBS status change** → OBS Plugin emits `ObsStatus` → UI store updates record button animation.  
5. **Manual Mode toggle** → UI confirms → emits `ManualModeToggled` → Core Bus → UI enters editable mode.
6. **Flag display** → UI requests flag → Flag Plugin provides flag URL → UI displays flag image.

### Technical Stack
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

### Authentication Process
- **Activation Flow**  
  1. UI prompts for license key → Tauri → License Plugin POST `/api/activate` with fingerprint  
  2. Server returns JWT + expiry → stored encrypted in filesystem  
- **Validation Flow**  
  - On startup & daily: License Plugin POST `/api/validate`  
  - If offline: track days since last success; warn after 5 days; disable after 7  
- **Revocation**  
  - Server can revoke keys; on validation failure UI locks down and prompts reactivation

### Route Design
- **Internal (Tauri Commands)**  
  - `udp:start(iface,port)`, `obs:cmd(action,params)`, `replay:play(recId)`, `license:activate(key)`, `settings:update(opts)`, `flag:get(iocCode)`  
- **Event Topics**  
  - `EventParsed`, `EventStored`, `ObsStatus`, `ClipReady`, `LicenseStatus`, `UiStateChange`, `FlagUpdated`  
- **External REST**  
  - `POST /api/activate`  
  - `POST /api/validate`  
  - `GET /api/license-info`

### API Design
- **Tauri Command Handlers** (Rust)  
  ```rust
  #[tauri::command]
  fn obs_cmd(action: String, params: JsonValue) -> Result<(), Error> { /* … */ }
  ```

## OBS WebSocket Integration

### Overview
reStrike VTA supports both OBS WebSocket v4 and v5 protocols simultaneously, allowing connection to multiple OBS instances running different protocol versions with a unified interface.

### Architecture

#### Backend Components
1. **`src-tauri/src/plugins/plugin_obs.rs`** - Main OBS WebSocket plugin
   - `ObsPlugin` - Manages multiple OBS connections
   - `ObsConnectionConfig` - Configuration for each connection
   - `ObsWebSocketVersion` - Protocol version enum (V4/V5)
   - Protocol-agnostic API methods

2. **`src-tauri/src/tauri_commands.rs`** - Tauri command handlers
   - Bridges frontend with backend OBS plugin
   - Handles all OBS operations (connect, disconnect, scene control, etc.)
   - Provides unified response format

#### Key Features
- **Multiple Connections**: Support for unlimited OBS instances
- **Protocol Detection**: Automatic protocol version handling
- **Event System**: Real-time status updates and events
- **Error Handling**: Comprehensive error management
- **Thread Safety**: Arc<Mutex> for concurrent access

### Protocol Differences Handled

#### OBS WebSocket v4
```json
// Request Format
{
  "request-type": "GetCurrentScene",
  "message-id": "uuid-here"
}

// Response Format
{
  "scene-name": "Scene Name",
  "is-recording": true
}
```

#### OBS WebSocket v5
```json
// Request Format
{
  "op": 6,
  "d": {
    "requestType": "GetCurrentProgramScene",
    "requestId": "uuid-here"
  }
}

// Response Format
{
  "requestStatus": {
    "result": true,
    "code": 100
  },
  "responseData": {
    "sceneName": "Scene Name",
    "outputActive": true
  }
}
```

### Frontend Integration

#### Core Components
1. **`ui/src/components/ObsWebSocketManager.tsx`** - Main UI component
   - Connection management interface
   - Protocol version selection
   - Real-time status display
   - Connection controls (connect/disconnect)

2. **Tauri Integration** - Frontend-backend communication
   - Type-safe command invocations
   - Real-time event handling
   - Error handling and user feedback

### Connection Management

```rust
// Add a new OBS connection
let config = ObsConnectionConfig {
    name: "Main OBS".to_string(),
    host: "localhost".to_string(),
    port: 4444,
    password: Some("password".to_string()),
    protocol_version: ObsWebSocketVersion::V4,
};

// Connect to OBS instance
obs_plugin.add_connection(config).await?;
```
    connection: Arc<DatabaseConnection>,
    migration_strategy: MigrationStrategy,
    hybrid_provider: Arc<Mutex<HybridSettingsProvider>>,
}

impl DatabasePlugin {
    // Flag Management
    pub async fn get_flag_mappings_data(&self) -> AppResult<FlagMappingsData>
    pub async fn scan_and_populate_flags(&self) -> AppResult<FlagScanResult>
    pub async fn get_flags_data(&self) -> AppResult<Vec<Flag>>
    pub async fn clear_flags_table(&self) -> AppResult<()>

    // PSS Event Management
    pub async fn get_pss_events(&self, limit: Option<i64>) -> AppResult<Vec<PssEventV2>>
    pub async fn get_pss_event_types(&self) -> AppResult<Vec<PssEventType>>
    pub async fn create_pss_event(&self, event: PssEventV2) -> AppResult<i64>

    // UDP Management
    pub async fn get_udp_server_configs(&self) -> AppResult<Vec<UdpServerConfig>>
    pub async fn create_udp_server_config(&self, config: UdpServerConfig) -> AppResult<i64>
    pub async fn update_udp_server_config(&self, id: i64, config: UdpServerConfig) -> AppResult<()>
}
```

#### UDP Plugin
```rust
pub struct UdpPlugin {
    config: UdpServerConfig,
    event_tx: UnboundedSender<PssEvent>,
    protocol_manager: Arc<ProtocolManager>,
    database: Arc<DatabasePlugin>,
    server: Arc<Mutex<Option<UdpServer>>>,
}

impl UdpPlugin {
    pub fn new(
        config: UdpServerConfig,
        event_tx: UnboundedSender<PssEvent>,
        protocol_manager: Arc<ProtocolManager>,
        database: Arc<DatabasePlugin>,
    ) -> Self {
        // Initialize UDP plugin with database integration
    }

    pub async fn start(&self) -> AppResult<()> {
        // Start UDP server with database session tracking
    }

    pub async fn stop(&self) -> AppResult<()> {
        // Stop UDP server and update database session
    }
}
```

##### **Panic Prevention and Defensive Programming**

The UDP Plugin implements comprehensive panic prevention and defensive programming patterns to ensure robust operation:

**1. JSON Serialization Error Handling**
```rust
// Safely serialize JSON with error handling
match serde_json::to_string(&event_json) {
    Ok(json_string) => {
        log::info!("📤 Emitting event JSON: {}", json_string);
        
        // Emit to Tauri frontend
        if let Err(e) = event_tx.send(event.clone()) {
            log::warn!("⚠️ Failed to send PSS event to internal channel: {}", e);
        }
        
        // Emit to frontend via core app's unified event emission
        crate::core::app::App::emit_pss_event(event_json);
        
        // Stream log to frontend for Live Data panel
        let log_message = format!("🎯 UDP-EVENT: {:?}", event);
        crate::core::app::App::emit_log_event(log_message);
    }
    Err(e) => {
        log::error!("❌ Failed to serialize PSS event to JSON: {}", e);
        log::error!("❌ Event that failed: {:?}", event);
        
        // Still try to send the event to internal channel
        if let Err(e) = event_tx.send(event.clone()) {
            log::warn!("⚠️ Failed to send PSS event to internal channel: {}", e);
        }
    }
}
```

**2. Defensive Programming in Event Conversion**
```rust
fn convert_pss_event_to_json(event: &PssEvent) -> serde_json::Value {
    // Add defensive programming to handle any potential issues
    match event {
        PssEvent::Clock { time, action } => {
            // Defensive programming for clock events
            let safe_time = time.as_str();
            let safe_action = action.as_ref().map(|a| a.as_str()).unwrap_or("");
            let description = format!("Clock: {} {:?}", safe_time, safe_action);
            
            serde_json::json!({
                "type": "clock",
                "time": safe_time,
                "action": safe_action,
                "description": description,
                "timestamp": chrono::Utc::now().timestamp_millis()
            })
        }
        PssEvent::Raw(message) => {
            // Defensive programming for raw messages
            let safe_message = message.as_str();
            let description = format!("Raw message: {}", safe_message);
            
            serde_json::json!({
                "type": "raw",
                "message": safe_message,
                "description": description,
                "timestamp": chrono::Utc::now().timestamp_millis()
            })
        }
        // ... other event types with similar defensive patterns
    }
}
```

**3. Error Recovery Strategies**
- **Graceful Degradation**: Continue operation when serialization fails
- **Event Preservation**: Ensure events reach internal channels even if JSON serialization fails
- **Comprehensive Logging**: Detailed error logging for debugging
- **Safe String Handling**: Use `as_str()` and `unwrap_or("")` for potentially problematic strings

**4. Benefits of Defensive Programming**
- **Panic Prevention**: Eliminates application crashes from serialization errors
- **Data Integrity**: Ensures events are processed even when JSON conversion fails
- **Debugging Support**: Comprehensive error logging for troubleshooting
- **System Reliability**: Robust operation under various error conditions

#### OBS Plugin
```rust
pub struct ObsPlugin {
    connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    event_tx: UnboundedSender<ObsEvent>,
}

impl ObsPlugin {
    // Connection Management
    pub async fn add_connection(&mut self, config: ObsConnectionConfig) -> AppResult<()>
    pub async fn remove_connection(&mut self, name: &str) -> AppResult<()>
    pub async fn connect_obs(&mut self, name: &str) -> AppResult<()>
    pub async fn disconnect_obs(&mut self, name: &str) -> AppResult<()>

    // Scene Operations
    pub async fn get_current_scene(&self, name: &str) -> AppResult<String>
    pub async fn set_current_scene(&self, name: &str, scene: &str) -> AppResult<()>
    pub async fn get_scenes(&self, name: &str) -> AppResult<Vec<String>>

    // Recording Operations
    pub async fn start_recording(&self, name: &str) -> AppResult<()>
    pub async fn stop_recording(&self, name: &str) -> AppResult<()>
    pub async fn get_recording_status(&self, name: &str) -> AppResult<bool>

    // Replay Buffer Operations
    pub async fn start_replay_buffer(&self, name: &str) -> AppResult<()>
    pub async fn stop_replay_buffer(&self, name: &str) -> AppResult<()>
    pub async fn save_replay_buffer(&self, name: &str) -> AppResult<()>
}
```

#### CPU Monitor Plugin
```rust
pub struct CpuMonitorPlugin {
    interval: Duration,
    event_tx: UnboundedSender<CpuEvent>,
    running: Arc<AtomicBool>,
}

impl CpuMonitorPlugin {
    pub async fn start_monitoring(&self) -> AppResult<()> {
        // Start CPU monitoring with configurable interval
    }

    pub async fn stop_monitoring(&self) -> AppResult<()> {
        // Stop CPU monitoring
    }

    pub async fn get_system_info(&self) -> AppResult<SystemInfo> {
        // Get current system information
    }
}
```

#### License Plugin
```rust
pub struct LicensePlugin {
    license_key: Option<String>,
    hardware_id: String,
    validation_url: String,
    event_tx: UnboundedSender<LicenseEvent>,
}

impl LicensePlugin {
    pub async fn activate_license(&mut self, key: &str) -> AppResult<LicenseStatus>
    pub async fn validate_license(&self) -> AppResult<LicenseStatus>
    pub async fn get_license_info(&self) -> AppResult<LicenseInfo>
    pub async fn check_hardware_id(&self) -> AppResult<String>
}
```

## Development Environment Management

### Windows Development Setup

The project is optimized for Windows development with comprehensive tooling:

#### Prerequisites
- **Windows 10/11** (64-bit)
- **Node.js** v24+ LTS
- **Rust** (latest stable via rustup)
- **Tauri CLI** (`cargo install tauri-cli`)
- **VSCode** with recommended extensions
- **Git** (latest version)
- **Python** v3.8+ (for scripts)

#### VSCode Extensions
- **Rust Analyzer** (rust-lang.rust-analyzer)
- **TypeScript and JavaScript Language Features** (built-in)
- **ES7+ React/Redux/React-Native snippets** (dsznajder.es7-react-js-snippets)
- **Tailwind CSS IntelliSense** (bradlc.vscode-tailwindcss)
- **GitLens** (eamodio.gitlens)
- **Thunder Client** (rangav.vscode-thunder-client)
- **Error Lens** (usernamehw.errorlens)
- **Auto Rename Tag** (formulahendry.auto-rename-tag)

#### Development Scripts
```bash
# Main development wrapper
./scripts/dev.sh help                    # Show all commands
./scripts/dev.sh status                  # Check current status
./scripts/dev.sh start-all               # Start all services
./scripts/dev.sh stop-all                # Stop all services
./scripts/dev.sh cleanup                 # Full cleanup

# Cleanup script
./scripts/development/cleanup-dev-environment.sh --cleanup
./scripts/development/cleanup-dev-environment.sh --quick
./scripts/development/cleanup-dev-environment.sh --status

# Fast development
./scripts/development/fast-dev.sh        # Fast development server
./scripts/development/dev.sh             # Standard development server
```

#### Environment Configuration
```json
// .vscode/settings.json
{
  "typescript.preferences.importModuleSpecifier": "relative",
  "typescript.suggest.autoImports": true,
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "files.associations": {
    "*.rs": "rust"
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.buildScripts.enable": true,
  "tailwindCSS.includeLanguages": {
    "typescript": "javascript",
    "typescriptreact": "javascript"
  }
}
```

#### Launch Configuration
```json
// .vscode/launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Launch Tauri App",
      "type": "node",
      "request": "launch",
      "program": "${workspaceFolder}/ui/node_modules/.bin/react-scripts",
      "args": ["start"],
      "cwd": "${workspaceFolder}/ui",
      "env": {
        "REACT_APP_ENVIRONMENT": "windows"
      },
      "console": "integratedTerminal"
    },
    {
      "name": "Debug Rust Backend",
      "type": "lldb",
      "request": "launch",
      "program": "${workspaceFolder}/target/debug/restrike-vta",
      "args": [],
      "cwd": "${workspaceFolder}",
      "console": "integratedTerminal"
    }
  ]
}
```

### Development Workflow

#### Starting Development
```bash
# Single command (recommended)
cd src-tauri
cargo tauri dev

# Alternative manual start
# Terminal 1: Start React dev server
cd ui
npm run start:fast

# Terminal 2: Start Tauri app
cd src-tauri
cargo tauri dev
```

#### Build Commands
```bash
# Development build
cd ui && npm run build

# Production build
cd src-tauri && cargo tauri build

# Clean build
cargo clean
npm run build
```

#### Testing Commands
```bash
# Frontend tests
cd ui && npm test

# Backend tests
cargo test

# Integration tests
cargo test --test integration

# Code quality
cargo clippy
cargo fmt
npm run lint
```

### Performance Optimization

#### Rust Backend Optimization
```toml
# Cargo.toml optimization settings
[profile.dev]
opt-level = 1
codegen-units = 256
incremental = true
lto = false

[profile.release]
opt-level = 3
codegen-units = 1
lto = true
panic = "abort"
```

#### Development Scripts
```bash
# Fast development scripts
npm run start:fast          # Fast React development
npm run build:fast          # Fast build
npm run clean:all           # Clean all caches
npm run analyze             # Bundle analysis
```

### Troubleshooting

#### Common Issues

**Rust Compilation Errors**
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build

# Check for missing dependencies
cargo check
```

**Node.js/npm Issues**
```bash
# Clear npm cache
npm cache clean --force

# Delete node_modules and reinstall
rm -rf node_modules
npm install

# Update npm
npm install -g npm@latest
```

**Tauri Build Issues**
```bash
# Check Tauri requirements
cargo tauri info

# Update Tauri CLI
cargo install tauri-cli --force

# Check system requirements
cargo tauri doctor
```

**Port Conflicts**
```bash
# Clean up ports before starting
./scripts/development/cleanup-dev-environment.sh --cleanup
```

#### Debug Information

**Enable Debug Logging**
```rust
// Enable debug logging
env_logger::init();
log::set_max_level(log::LevelFilter::Debug);
```

**Check System Status**
```bash
# Check development environment status
./scripts/dev.sh status

# Check port usage
./scripts/dev.sh ports

# Run health checks
./scripts/dev.sh health
```

## Core Systems

### Application State Management
```rust
pub struct AppState {
    pub plugins: Arc<Mutex<HashMap<String, Box<dyn Plugin>>>>,
    pub event_tx: UnboundedSender<AppEvent>,
    pub config: Arc<ConfigManager>,
    pub database: Arc<DatabasePlugin>,
}

impl AppState {
    pub async fn initialize(&mut self) -> AppResult<()> {
        // Initialize all plugins in correct order
        self.initialize_database_plugin().await?;
        self.initialize_udp_plugin().await?;
        self.initialize_obs_plugin().await?;
        self.initialize_cpu_monitor_plugin().await?;
        self.initialize_license_plugin().await?;
        Ok(())
    }
}
```

### Configuration Management
```rust
pub struct ConfigManager {
    config_path: PathBuf,
    config: Arc<RwLock<AppConfig>>,
    backup_path: PathBuf,
}

impl ConfigManager {
    pub async fn load_config(&self) -> AppResult<AppConfig>
    pub async fn save_config(&self, config: &AppConfig) -> AppResult<()>
    pub async fn backup_config(&self) -> AppResult<()>
    pub async fn restore_config(&self) -> AppResult<()>
    pub async fn get_config_statistics(&self) -> AppResult<ConfigStats>
}
```

### Logging System
```rust
pub struct Logger {
    log_path: PathBuf,
    rotation_config: RotationConfig,
    archival_config: ArchivalConfig,
}

impl Logger {
    pub fn initialize(&self) -> AppResult<()>
    pub fn log(&self, level: Level, message: &str) -> AppResult<()>
    pub async fn rotate_logs(&self) -> AppResult<()>
    pub async fn archive_logs(&self) -> AppResult<()>
    pub async fn get_log_statistics(&self) -> AppResult<LogStats>
}
```

## Tauri Integration

### Command Registration
```rust
// tauri_commands.rs
#[tauri::command]
async fn get_app_status() -> AppResult<AppStatus> {
    // Return application status
}

#[tauri::command]
async fn obs_get_status(connection_name: String) -> AppResult<ObsStatus> {
    // Return OBS connection status
}

#[tauri::command]
async fn system_get_info() -> AppResult<SystemInfo> {
    // Return system information
}

#[tauri::command]
async fn get_flag_mappings_data() -> AppResult<FlagMappingsData> {
    // Return flag mapping data
}

#[tauri::command]
async fn scan_and_populate_flags() -> AppResult<FlagScanResult> {
    // Scan and populate flags
}
```

### Event System
```rust
// Event types for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    PssEvent(PssEvent),
    ObsEvent(ObsEvent),
    CpuEvent(CpuEvent),
    LicenseEvent(LicenseEvent),
    DatabaseEvent(DatabaseEvent),
}

// Event emission
pub async fn emit_event(event: AppEvent) -> AppResult<()> {
    if let Some(app_handle) = APP_HANDLE.get() {
        app_handle.emit_all("app_event", event)?;
    }
    Ok(())
}
```

## Error Handling

### AppResult and AppError
```rust
pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    #[error("Plugin error: {0}")]
    PluginError(String),
}
```

### Error Propagation
```rust
// Proper error handling in plugins
impl DatabasePlugin {
    pub async fn create_pss_event(&self, event: PssEventV2) -> AppResult<i64> {
        let conn = self.connection.lock()
            .map_err(|e| AppError::DatabaseError(rusqlite::Error::InvalidPath(e.to_string())))?;
        
        // Database operation
        let event_id = conn.execute(
            "INSERT INTO pss_events_v2 (...) VALUES (...)",
            params![...]
        )?;
        
        Ok(event_id)
    }
}
```

## Performance Monitoring

### System Monitoring
```rust
pub struct SystemMonitor {
    cpu_usage: Arc<AtomicU64>,
    memory_usage: Arc<AtomicU64>,
    disk_usage: Arc<AtomicU64>,
    network_stats: Arc<Mutex<NetworkStats>>,
}

impl SystemMonitor {
    pub async fn start_monitoring(&self) -> AppResult<()>
    pub async fn get_system_stats(&self) -> AppResult<SystemStats>
    pub async fn get_performance_metrics(&self) -> AppResult<PerformanceMetrics>
}
```

### Database Performance
```rust
pub struct DatabaseMonitor {
    query_times: Arc<Mutex<Vec<Duration>>>,
    connection_pool: Arc<Mutex<ConnectionPool>>,
    cache_stats: Arc<Mutex<CacheStats>>,
}

impl DatabaseMonitor {
    pub async fn track_query_time(&self, duration: Duration)
    pub async fn get_performance_stats(&self) -> AppResult<DatabaseStats>
    pub async fn optimize_queries(&self) -> AppResult<()>
}
```

## Security

### License Validation
```rust
pub struct LicenseValidator {
    hardware_id: String,
    license_key: Option<String>,
    validation_url: String,
    offline_grace_period: Duration,
}

impl LicenseValidator {
    pub async fn validate_license(&self) -> AppResult<LicenseStatus>
    pub async fn activate_license(&mut self, key: &str) -> AppResult<LicenseStatus>
    pub async fn check_hardware_id(&self) -> AppResult<String>
    pub async fn get_offline_status(&self) -> AppResult<OfflineStatus>
}
```

### Data Protection
```rust
pub struct DataProtector {
    encryption_key: Vec<u8>,
    sensitive_fields: HashSet<String>,
}

impl DataProtector {
    pub fn encrypt_sensitive_data(&self, data: &str) -> AppResult<String>
    pub fn decrypt_sensitive_data(&self, encrypted_data: &str) -> AppResult<String>
    pub fn mask_sensitive_fields(&self, data: &mut serde_json::Value) -> AppResult<()>
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_plugin_creation() {
        let plugin = DatabasePlugin::new().await.unwrap();
        assert_eq!(plugin.name(), "database");
    }

    #[tokio::test]
    async fn test_udp_plugin_initialization() {
        let (event_tx, _) = mpsc::unbounded_channel();
        let config = UdpServerConfig::default();
        let plugin = UdpPlugin::new(config, event_tx, Arc::new(ProtocolManager::new()), Arc::new(DatabasePlugin::new().await.unwrap()));
        assert_eq!(plugin.name(), "udp");
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        // Test complete workflow from UDP event to database storage
        let app_state = AppState::new().await.unwrap();
        app_state.initialize().await.unwrap();
        
        // Simulate PSS event
        let event = PssEvent::new_test_event();
        app_state.process_pss_event(event).await.unwrap();
        
        // Verify database storage
        let events = app_state.database.get_pss_events(Some(1)).await.unwrap();
        assert_eq!(events.len(), 1);
    }
}
```

## Deployment

### Production Build
```bash
# Build for production
cargo tauri build --release

# Create installer
cargo tauri build --target x86_64-pc-windows-msvc

# Output location: src-tauri/target/release/bundle/
```

### Distribution
```bash
# Create distribution package
# - Windows executable (.exe)
# - MSI installer
# - Portable version
# - Documentation and licenses
```

## Future Enhancements

### Planned Features
1. **Plugin Marketplace**: Extensible plugin system
2. **Advanced Analytics**: Real-time analytics and reporting
3. **Cloud Integration**: Cloud backup and synchronization
4. **Multi-language Support**: Internationalization
5. **Advanced Security**: Enhanced security features

### Technical Improvements
1. **Performance Optimization**: Advanced performance tuning
2. **Memory Management**: Improved memory usage
3. **Error Recovery**: Enhanced error recovery mechanisms
4. **Monitoring**: Advanced monitoring and alerting
5. **Scalability**: Improved scalability for large deployments

## Support and Resources

### Documentation
- **Tauri Documentation**: https://tauri.app/docs/
- **Rust Documentation**: https://doc.rust-lang.org/
- **Tokio Documentation**: https://tokio.rs/

### Community
- **Tauri Discord**: https://discord.gg/tauri
- **Rust Community**: https://users.rust-lang.org/
- **GitHub Issues**: Project-specific issues and discussions

### Professional Support
- **Custom Development**: Tailored solutions and features
- **Training and Consulting**: Development team training
- **Enterprise Support**: Enterprise-level support and maintenance