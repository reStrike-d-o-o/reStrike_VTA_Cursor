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
- **Database**: SQLite with rusqlite + sqlx (hybrid approach for thread safety)
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
│   │   ├── obs/             # Modular OBS WebSocket integration
│   │   │   ├── mod.rs       # OBS module registration
│   │   │   ├── types.rs     # Shared types and data structures
│   │   │   ├── manager.rs   # Plugin coordination
│   │   │   ├── core.rs      # Connection management
│   │   │   ├── recording.rs # Recording control
│   │   │   ├── streaming.rs # Streaming control
│   │   │   ├── scenes.rs    # Scene management
│   │   │   ├── settings.rs  # Settings management
│   │   │   ├── events.rs    # Event processing
│   │   │   ├── status.rs    # Status aggregation
│   │   │   ├── control_room.rs      # Legacy Control Room (rusqlite)
│   │   │   └── control_room_async.rs # Async Control Room (sqlx)
│   │   ├── plugin_udp.rs    # UDP protocol handling
│   │   ├── plugin_database.rs # Database operations
│   │   ├── plugin_cpu_monitor.rs # System monitoring
│   │   └── plugin_license.rs # License management
│   ├── security/            # Security system (✅ NEW)
│   │   ├── mod.rs           # Security module registration
│   │   ├── encryption.rs    # AES-256-GCM encryption with PBKDF2
│   │   ├── config_manager.rs # Secure configuration management
│   │   ├── audit.rs         # Security audit logging
│   │   ├── key_manager.rs   # Encryption key lifecycle management
│   │   └── migration.rs     # Configuration migration tools
│   ├── database/            # Database system
│   │   ├── connection.rs    # Database connection management (rusqlite)
│   │   ├── async_connection.rs # Async database layer (sqlx)
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

## OBS Plugin Modularization ✅ **COMPLETED**

### Overview
The OBS plugin system has been successfully modularized to improve maintainability, reduce complexity, and enable better development efficiency. The monolithic 1366-line `plugin_obs.rs` file has been broken down into focused, single-responsibility modules.

### Implementation Status ✅ **COMPLETED**
- **Modular Architecture**: Successfully implemented with 8 focused modules
- **Total Lines**: ~1600 lines distributed across modular components
- **Zero Breaking Changes**: All existing functionality preserved
- **Safe Migration**: Old monolithic plugin safely removed
- **Compilation Status**: ✅ All modules compile successfully with only expected warnings

### Implemented Modular Structure

#### **Core Infrastructure**
- **`obs/types.rs`**: Shared types, enums, and data structures
- **`obs/manager.rs`**: Plugin coordination and cross-plugin communication
- **`obs/core.rs`**: Connection management and WebSocket infrastructure

#### **Feature Plugins**
- **`obs/recording.rs`**: Recording start/stop, replay buffer, recording status
- **`obs/streaming.rs`**: Streaming start/stop, streaming status monitoring
- **`obs/scenes.rs`**: Scene management, switching, source manipulation ✅ **COMPLETED**

#### **Support Plugins**
- **`obs/settings.rs`**: OBS Studio settings, profile management, output settings ✅ **COMPLETED**
- **`obs/events.rs`**: Event handling, routing, filtering, frontend broadcasting ⚠️ **PARTIALLY COMPLETED**
- **`obs/status.rs`**: Status aggregation, monitoring, health checks ⚠️ **PARTIALLY COMPLETED**

### OBS Scenes Plugin ✅ **COMPLETED**

#### **Real Scene Management Implementation**
- **Scene Enumeration**: `get_scenes()` with real OBS WebSocket API calls
- **Scene Switching**: `set_current_scene()` with proper WebSocket communication
- **Current Scene Detection**: `get_current_scene()` with real-time OBS data
- **Studio Mode Support**: `get_studio_mode()` and `set_studio_mode()` functionality
- **Source Management**: `get_sources()`, `set_source_visibility()`, `get_source_visibility()`

#### **Core Plugin Integration**
- **WebSocket Communication**: All methods use `core_plugin.send_request()` for real OBS communication
- **Error Handling**: Comprehensive error handling for all scene operations
- **Logging**: Detailed logging for debugging and monitoring
- **Tauri Commands**: All scene-related Tauri commands implemented and registered

#### **Key Features**
- **Real-time Scene Switching**: Instant scene changes via OBS WebSocket
- **Source Visibility Control**: Dynamic control of source visibility within scenes
- **Studio Mode Integration**: Full studio mode support for preview functionality
- **Scene List Management**: Real-time scene enumeration from OBS

### OBS Settings Plugin ✅ **COMPLETED**

#### **Comprehensive Settings Management**
- **OBS Version Detection**: `get_obs_version()` with real OBS API calls
- **Profile Management**: `get_profiles()`, `get_current_profile()`, `set_current_profile()`
- **Recording Settings**: Comprehensive recording path, filename, and format management
- **Streaming Settings**: Streaming account and channel management
- **Replay Buffer Settings**: Complete replay buffer configuration options
- **Output Settings**: `get_output_settings()` and `set_output_settings()`

#### **Advanced Configuration Options**
- **Recording Path Management**: Dynamic recording path configuration
- **Filename Format Control**: Custom filename format with variable support
- **Format Selection**: Support for multiple recording formats (MP4, MKV, etc.)
- **Quality Settings**: Comprehensive quality and bitrate configuration
- **Replay Buffer**: Complete replay buffer settings with duration, path, and format control

#### **Core Plugin Integration**
- **WebSocket Communication**: All methods use `core_plugin.send_request()` for real OBS communication
- **Error Handling**: Robust error handling for all settings operations
- **Logging**: Detailed logging for configuration changes
- **Tauri Commands**: All settings-related Tauri commands implemented and registered

### YouTube Streaming Integration ✅ **COMPLETED**

#### **Comprehensive YouTube Management**
- **Account Management**: `get_youtube_accounts()` with comprehensive account data
- **Channel Management**: `get_youtube_channels()` with channel details
- **Stream Key Management**: `get_youtube_stream_key()` and `regenerate_youtube_stream_key()`
- **Streaming Configuration**: `set_youtube_streaming_config()` with all YouTube options

#### **YouTube-Specific Features**
- **Categories**: `get_youtube_categories()` with all available categories
- **Privacy Options**: `get_youtube_privacy_options()` (Public, Unlisted, Private)
- **Latency Options**: `get_youtube_latency_options()` (Normal, Low, Ultra-low)
- **Server URLs**: `get_youtube_server_urls()` with regional servers
- **Streaming Analytics**: `get_youtube_streaming_analytics()` with viewership data
- **Streaming Schedule**: `get_youtube_streaming_schedule()` and `create_youtube_streaming_schedule()`

#### **Advanced YouTube Features**
- **Stream Key Regeneration**: Secure stream key regeneration for security
- **Analytics Integration**: Real-time viewership and engagement metrics
- **Schedule Management**: Advanced streaming schedule creation and management
- **Regional Optimization**: Server selection for optimal streaming performance

### Multi-Platform Streaming Support ✅ **COMPLETED**

#### **Platform-Specific Integrations**
- **Twitch Integration**: `get_twitch_config()` with Twitch-specific options
- **Facebook Live**: `get_facebook_config()` with Facebook Live options
- **Custom RTMP**: `get_custom_rtmp_config()` and `set_custom_rtmp_config()`

#### **Unified Streaming Management**
- **Service Discovery**: `get_available_streaming_services()` with all supported platforms
- **Authentication Management**: `get_streaming_auth_status()`, `authenticate_streaming_service()`, `refresh_streaming_auth()`
- **Generic Streaming**: `get_streaming_accounts()`, `get_streaming_channels()`, `get_streaming_events()`

#### **Cross-Platform Features**
- **Unified Configuration**: Consistent configuration interface across all platforms
- **Authentication Flow**: Standardized authentication process for all services
- **Error Handling**: Platform-specific error handling and recovery
- **Service Switching**: Seamless switching between different streaming platforms

### Control Room Implementation ✅ **PHASE 1 COMPLETED**

#### **Thread-Safe Architecture**
- **SQLite Thread Safety Issue**: Resolved rusqlite::Connection not being Send+Sync for async Tauri commands
- **Hybrid Database Approach**: Maintained existing rusqlite for compatibility while adding sqlx for async operations
- **AsyncDatabaseConnection**: New thread-safe database layer using sqlx::SqlitePool for Tauri commands
- **Connection Pooling**: Built-in connection pooling eliminates connection overhead and ensures thread safety

#### **Control Room Manager** 
- **AsyncControlRoomManager**: Complete async-compatible STR connection management system
- **Separate Connection Management**: Dedicated Control Room connections independent of existing OBS WebSocket connections
- **Master Password Authentication**: Development authentication system (any non-empty password accepted)
- **Database Storage**: Encrypted storage of Control Room configurations using existing security infrastructure
- **Access Control**: Basic authentication gate protecting STR management operations
- **Session Management**: Secure session tracking with logout functionality
- **Security Status**: ⚠️ Simplified authentication for development - production security enhancement pending

#### **STR Connection Management**
- **User-Defined Names**: STR connection names are input by users, not auto-generated
- **Connection Lifecycle**: Full connect/disconnect management for STR instances
- **Status Tracking**: Real-time connection status monitoring (Disconnected, Connecting, Connected, Error)
- **Configuration Persistence**: Secure database storage of connection configurations

#### **Audio Control Integration**
- **Mute/Unmute Functionality**: Audio source control for STR instances via existing OBS API
- **Source Discovery**: Audio source enumeration for connected STR instances
- **Bulk Operations**: Multi-STR audio control operations
- **API Reuse**: Leverages existing OBS streaming plugin audio control methods

#### **Bulk Operations**
- **Multi-STR Scene Changes**: Change scenes across all connected STR instances
- **Streaming Control**: Start/stop streaming on all connected STR instances
- **Audio Management**: Bulk mute/unmute operations across multiple STR connections
- **Result Aggregation**: Comprehensive result reporting for bulk operations

#### **Tauri Commands Integration**
- **Async Commands**: Thread-safe Tauri commands for Control Room operations
- **Authentication**: `control_room_authenticate_async` with development-level password check
- **Connection Management**: Commands for adding/removing STR connections (`control_room_add_str_connection`, `control_room_remove_str_connection`)
- **Connection Control**: Commands for connecting/disconnecting STR instances (`control_room_connect_str`, `control_room_disconnect_str`)
- **Status Retrieval**: `control_room_get_str_connections` for real-time connection listing
- **Bulk Controls**: Commands for multi-STR operations (framework ready for expansion)
- **Error Handling**: Comprehensive error handling and logging
- **Compilation Success**: All 6 Tauri commands fully functional with zero compilation errors
- **Security Note**: ⚠️ Current authentication accepts any non-empty password (development implementation)

#### **Database Integration**
- **File Structure**: Control Room connection table in async database
- **Configuration Storage**: Encrypted storage using existing security infrastructure
- **Migration Ready**: Seamless integration with existing database migration system
- **Connection Pooling**: Efficient database connection management

### Key Improvements Achieved
- **Password Authentication**: Fixed authentication flow with proper `is_connected` field management
- **Status Listener**: Fixed DockBar status indicators with proper connection state tracking
- **Full Events Toggle**: Implemented working full events toggle in Diagnostics & Logs
- **Live Data Controls**: Fixed Live Data panel functionality with proper backend integration
- **Error Handling**: Comprehensive error handling throughout modular system
- **Real OBS Integration**: All plugins now use real OBS WebSocket API calls
- **YouTube Streaming**: Comprehensive YouTube and multi-platform streaming support

### Migration Completed ✅
- **Phase 1**: ✅ Created new modular structure with all functionality
- **Phase 2**: ✅ Integrated with main application and Tauri commands
- **Phase 3**: ✅ Verified all functionality works correctly
- **Phase 4**: ✅ Safely removed old monolithic plugin file
- **Phase 5**: ✅ Implemented real OBS integration for scenes and settings
- **Phase 6**: ✅ Added comprehensive YouTube streaming support

### Benefits Achieved
- **Maintainability**: ~200 lines per file vs 1366 lines
- **Single Responsibility**: Each plugin has one clear purpose
- **Easier Testing**: Test each plugin independently
- **Parallel Development**: Multiple developers can work on different plugins
- **Better Organization**: Logical grouping of related functionality
- **Zero Breaking Changes**: All existing functionality preserved
- **Real OBS Integration**: All plugins now communicate with real OBS Studio
- **YouTube Support**: Comprehensive YouTube and multi-platform streaming capabilities

### Current Status
- **Compilation**: ✅ All modules compile successfully
- **Warnings**: Only expected warnings for unused event processing methods (intentional)
- **Functionality**: All implemented features working correctly
- **Integration**: Complete integration with frontend and Tauri commands
- **Documentation**: Comprehensive documentation for all implemented features

## Plugin Architecture

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

---

## 🎥 OBS Integration System

### **OBS WebSocket Management**

The backend includes comprehensive OBS Studio integration through WebSocket connections for recording, streaming, and replay buffer management:

#### **OBS Connection Types**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsConnectionType {
    OBS_REC,    // Recording connection
    OBS_STR,    // Streaming connection
    OBS_BOTH,   // Both recording and streaming
}

#[derive(Debug, Clone)]
pub struct ObsConnection {
    pub connection_type: ObsConnectionType,
    pub websocket_url: String,
    pub password: Option<String>,
    pub is_connected: Arc<AtomicBool>,
    pub last_heartbeat: Arc<Mutex<Instant>>,
    pub connection_handle: Option<JoinHandle<()>>,
}
```

#### **OBS Session Management**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsSession {
    pub id: Option<i64>,
    pub session_id: i64,
    pub session_type: String,           // 'stream', 'recording', 'replay_buffer'
    pub obs_connection: String,         // 'OBS_REC', 'OBS_STR', 'OBS_BOTH'
    pub start_timestamp: String,
    pub end_timestamp: Option<String>,
    pub tournament_id: Option<i64>,
    pub tournament_day_id: Option<i64>,
    pub session_number: i32,
    pub is_active: bool,
    pub interruption_reason: Option<String>,
    pub time_offset_seconds: i64,
    pub cumulative_offset_seconds: i64,
    pub recording_path: Option<String>,
    pub recording_name: Option<String>,
    pub stream_key: Option<String>,
    pub replay_buffer_duration: i32,
    pub replay_buffer_path: Option<String>,
    pub created_at: String,
}

impl ObsSession {
    pub fn new_stream_session(session_id: i64, tournament_day_id: i64) -> Self {
        Self {
            id: None,
            session_id,
            session_type: "stream".to_string(),
            obs_connection: "OBS_STR".to_string(),
            start_timestamp: chrono::Utc::now().to_rfc3339(),
            end_timestamp: None,
            tournament_id: None,
            tournament_day_id: Some(tournament_day_id),
            session_number: 1,
            is_active: true,
            interruption_reason: None,
            time_offset_seconds: 0,
            cumulative_offset_seconds: 0,
            recording_path: None,
            recording_name: None,
            stream_key: None,
            replay_buffer_duration: 20,
            replay_buffer_path: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn new_recording_session(session_id: i64, tournament_day_id: i64) -> Self {
        Self {
            id: None,
            session_id,
            session_type: "recording".to_string(),
            obs_connection: "OBS_REC".to_string(),
            start_timestamp: chrono::Utc::now().to_rfc3339(),
            end_timestamp: None,
            tournament_id: None,
            tournament_day_id: Some(tournament_day_id),
            session_number: 1,
            is_active: true,
            interruption_reason: None,
            time_offset_seconds: 0,
            cumulative_offset_seconds: 0,
            recording_path: None,
            recording_name: None,
            stream_key: None,
            replay_buffer_duration: 20,
            replay_buffer_path: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
```

### **OBS Integration Triggering Rules**

#### **New Match Loaded and Ready Trigger**
```rust
impl ObsIntegration {
    pub async fn handle_new_match_loaded(&self, match_data: &MatchData) -> AppResult<()> {
        // Setup recording path: Tournament name/current active tournament Day name
        let recording_path = format!(
            "C:/Users/{}/Videos/{}/{}",
            whoami::username(),
            match_data.tournament_name,
            match_data.tournament_day_name
        );

        // Setup recording name: matchNumber+player1+player1 country IOC+vs+player2+player2 country IOC
        let recording_name = format!(
            "{}_{}_{}_vs_{}_{}",
            match_data.match_number,
            match_data.player1_name,
            match_data.player1_country_ioc,
            match_data.player2_name,
            match_data.player2_country_ioc
        );

        // Setup video replay buffer 20s
        let replay_buffer_duration = 20;

        // Setup video replay buffer saving location
        let replay_buffer_path = format!("{}/IVR recordings", recording_path);

        // Start recording and replay buffer
        self.obs_rec_connection.set_recording_path(&recording_path).await?;
        self.obs_rec_connection.set_recording_name(&recording_name).await?;
        self.obs_rec_connection.set_replay_buffer_duration(replay_buffer_duration).await?;
        self.obs_rec_connection.set_replay_buffer_path(&replay_buffer_path).await?;
        self.obs_rec_connection.start_recording().await?;
        self.obs_rec_connection.start_replay_buffer().await?;

        // Calculate str_timestamp for OBS_STR
        if let Some(stream_start) = self.obs_str_connection.get_stream_start_time().await? {
            let str_timestamp = self.calculate_str_timestamp(&match_data.timestamp, &stream_start);
            // Save str_timestamp to database
            self.update_event_str_timestamp(match_data.event_id, &str_timestamp).await?;
        }

        Ok(())
    }
}
```

#### **Challenge/IVR or Replay Button Trigger**
```rust
impl ObsIntegration {
    pub async fn handle_challenge_ivr_trigger(&self, event_data: &PssEventData) -> AppResult<()> {
        // Save video replay buffer
        let replay_clip_path = self.obs_rec_connection.save_replay_buffer().await?;

        // Calculate rec_timestamp for all events in last 20s
        let events_last_20s = self.get_events_last_20_seconds(event_data.session_id).await?;
        for event in events_last_20s {
            if event.rec_timestamp.is_none() {
                let rec_timestamp = self.calculate_rec_timestamp(&event.timestamp).await?;
                self.update_event_rec_timestamp(event.id, &rec_timestamp).await?;
            }
        }

        // Open last saved video replay buffer clip in .mvp player
        self.open_replay_clip_in_mvp_player(&replay_clip_path).await?;

        // Add IVR link to all events in last 20 seconds
        for event in events_last_20s {
            if event.ivr_link.is_none() {
                self.update_event_ivr_link(event.id, &replay_clip_path).await?;
            }
        }

        // Change scene to IVR_SCENE
        self.obs_str_connection.set_current_scene("IVR_SCENE").await?;

        // Activate starting animation
        self.activate_ivr_stream_overlay_animation("start").await?;

        Ok(())
    }
}
```

#### **Challenge Resolution or Video Close Trigger**
```rust
impl ObsIntegration {
    pub async fn handle_challenge_resolution(&self) -> AppResult<()> {
        // Activate closing animation
        self.activate_ivr_stream_overlay_animation("close").await?;

        // Change scene to LIVE_SCENE
        self.obs_str_connection.set_current_scene("LIVE_SCENE").await?;

        // Check if video replay buffer is active, if not, activate it
        if !self.obs_rec_connection.is_replay_buffer_active().await? {
            self.obs_rec_connection.start_replay_buffer().await?;
        }

        Ok(())
    }
}
```

### **Stream Interruption Handling**

#### **Automatic Stream Restart Detection**
```rust
impl ObsIntegration {
    pub async fn detect_and_handle_stream_restart(&self, tournament_day_id: i64) -> AppResult<()> {
        // Check if current stream session is still active
        if let Some(active_session) = self.get_active_stream_session(tournament_day_id).await? {
            // Check if OBS stream is still connected
            if !self.obs_str_connection.is_connected().await? {
                // Stream was interrupted, end current session
                self.end_active_stream_session(tournament_day_id, "stream_interruption").await?;
                
                // Start new stream session
                let new_session = self.start_new_stream_session(tournament_day_id).await?;
                
                // Calculate time offset
                let time_offset = self.calculate_stream_time_offset(tournament_day_id).await?;
                
                // Update cumulative offset for all subsequent sessions
                self.update_cumulative_offset(tournament_day_id, "stream", time_offset).await?;
                
                // Recalculate str_timestamps for all events in current tournament day
                self.recalculate_str_timestamps(tournament_day_id).await?;
            }
        }

        Ok(())
    }

    pub async fn handle_stream_interruption(
        &self,
        tournament_day_id: i64,
        reason: &str
    ) -> AppResult<()> {
        // End current active session
        self.end_active_stream_session(tournament_day_id, reason).await?;

        // Start new session
        let new_session = self.start_new_stream_session(tournament_day_id).await?;

        // Calculate and apply time offset
        let time_offset = self.calculate_session_time_offset(tournament_day_id, "stream").await?;
        self.update_cumulative_offset(tournament_day_id, "stream", time_offset).await?;

        Ok(())
    }
}
```

### **YouTube Chapter Generation**

#### **Database-Driven Chapter Generation**
```rust
impl ObsIntegration {
    pub async fn generate_youtube_chapters(
        &self,
        tournament_day_id: i64,
        output_path: &str
    ) -> AppResult<()> {
        // Get all events with str_timestamp for the tournament day
        let events = self.get_events_with_str_timestamp(tournament_day_id).await?;

        // Group events by session and match
        let mut chapters = Vec::new();
        for event in events {
            if let (Some(str_timestamp), Some(match_number), Some(event_category)) = 
                (&event.str_timestamp, &event.match_number, &event.event_category) {
                
                let chapter_line = format!(
                    "{} {} - {}",
                    str_timestamp,
                    self.get_event_category_description(event_category),
                    event.description.as_deref().unwrap_or("")
                );
                
                chapters.push(chapter_line);
            }
        }

        // Write to file
        let mut file = tokio::fs::File::create(output_path).await?;
        let content = chapters.join("\n");
        file.write_all(content.as_bytes()).await?;

        Ok(())
    }

    fn get_event_category_description(&self, category: &str) -> &'static str {
        match category {
            "R" => "Referee Decision",
            "K" => "Kick Event",
            "P" => "Punch Point",
            "H" => "Head Point",
            "TH" => "Technical Head Point",
            "TB" => "Technical Body Point",
            _ => "Match Event",
        }
    }
}
```

---

## ⚡ Performance Optimization Strategy

### **Current Performance Analysis**

#### **Identified Performance Bottlenecks**
1. **UDP Event Processing**: High-frequency PSS events (100+ events/second)
2. **Database Operations**: Frequent inserts and real-time queries
3. **WebSocket Broadcasting**: JSON serialization and synchronous broadcasting
4. **Memory Management**: Event caching and WebSocket client management
5. **CPU Usage**: Heavy processing in main thread

#### **Performance Targets**
- **Latency**: < 50ms for UDP event processing
- **Throughput**: 1000+ events/second sustained
- **Memory Usage**: < 100MB for normal operation
- **CPU Usage**: < 10% average, < 30% peak
- **Database**: < 5ms average query time

### **Multi-Phase Optimization Implementation**

#### **Phase 1: UDP Processing Optimization (Priority 1)**

**Bounded Channels Implementation**
```rust
// Replace unbounded channels with bounded channels
pub struct OptimizedUdpPlugin {
    event_receiver: tokio::sync::mpsc::Receiver<PssEvent>,
    event_sender: tokio::sync::mpsc::Sender<PssEvent>,
    batch_processor: tokio::sync::mpsc::Sender<Vec<PssEvent>>,
    // ... other fields
}

impl OptimizedUdpPlugin {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(1000); // Bounded channel
        let (batch_sender, batch_receiver) = tokio::sync::mpsc::channel(100);  // Bounded batch channel
        
        Self {
            event_receiver,
            event_sender,
            batch_processor: batch_sender,
            // ... initialize other fields
        }
    }

    pub async fn process_events_batch(&mut self) -> AppResult<()> {
        let mut batch = Vec::with_capacity(50);
        let mut timeout = tokio::time::sleep(Duration::from_millis(100));

        loop {
            tokio::select! {
                event = self.event_receiver.recv() => {
                    match event {
                        Some(event) => {
                            batch.push(event);
                            if batch.len() >= 50 {
                                self.batch_processor.send(batch.drain(..).collect()).await?;
                            }
                        }
                        None => break,
                    }
                }
                _ = &mut timeout => {
                    if !batch.is_empty() {
                        self.batch_processor.send(batch.drain(..).collect()).await?;
                    }
                    timeout = tokio::time::sleep(Duration::from_millis(100));
                }
            }
        }

        Ok(())
    }
}
```

**Zero-Copy PSS Protocol Parsing**
```rust
use bytes::{Buf, BufMut, BytesMut};

pub struct OptimizedPssParser {
    buffer: BytesMut,
}

impl OptimizedPssParser {
    pub fn parse_event_zero_copy(&mut self, data: &[u8]) -> AppResult<PssEvent> {
        self.buffer.extend_from_slice(data);
        
        // Parse without allocating new strings
        let event_type = self.buffer.get_u8();
        let timestamp = self.buffer.get_u64_le();
        let data_length = self.buffer.get_u16_le();
        
        // Use slice instead of allocating new Vec
        let event_data = self.buffer.copy_to_bytes(data_length as usize);
        
        Ok(PssEvent {
            event_type,
            timestamp,
            data: event_data.to_vec(), // Only allocate when needed
        })
    }
}
```

#### **Phase 2: Database Optimization (Priority 1)**

**Connection Pooling Implementation**
```rust
use deadpool_sqlite::{Config, Pool, Runtime};

pub struct OptimizedDatabasePlugin {
    pool: Pool,
    prepared_statements: Arc<Mutex<HashMap<String, rusqlite::Statement<'static>>>>,
}

impl OptimizedDatabasePlugin {
    pub async fn new() -> AppResult<Self> {
        let config = Config::new("database.db");
        let pool = config.create_pool(Some(Runtime::Tokio1), deadpool_sqlite::Manager::new)?;
        
        Ok(Self {
            pool,
            prepared_statements: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn batch_insert_events(&self, events: Vec<PssEventV2>) -> AppResult<()> {
        let conn = self.pool.get().await?;
        
        // Use prepared statement for batch insert
        let stmt = conn.prepare_cached(
            "INSERT OR REPLACE INTO pss_events_v2 
             (timestamp, event_type, data, event_category, tournament_id, tournament_day_id, match_number, rec_timestamp, str_timestamp, ivr_link) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ).await?;

        // Batch insert with transaction
        let tx = conn.transaction().await?;
        for event in events {
            tx.execute(&stmt, rusqlite::params![
                event.timestamp,
                event.event_type,
                event.data,
                event.event_category,
                event.tournament_id,
                event.tournament_day_id,
                event.match_number,
                event.rec_timestamp,
                event.str_timestamp,
                event.ivr_link,
            ]).await?;
        }
        tx.commit().await?;

        Ok(())
    }
}
```

#### **Phase 3: WebSocket Optimization (Priority 2)**

**Binary Serialization with Protocol Buffers**
```rust
use prost::Message;

#[derive(Message)]
pub struct OptimizedPssEvent {
    #[prost(uint32, tag = "1")]
    pub event_type: u32,
    #[prost(bytes, tag = "2")]
    pub data: Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
    #[prost(string, optional, tag = "4")]
    pub event_category: Option<String>,
    #[prost(string, optional, tag = "5")]
    pub rec_timestamp: Option<String>,
    #[prost(string, optional, tag = "6")]
    pub str_timestamp: Option<String>,
    #[prost(string, optional, tag = "7")]
    pub ivr_link: Option<String>,
}

impl OptimizedWebSocketServer {
    pub async fn broadcast_event_optimized(&self, event: &PssEvent) -> AppResult<()> {
        // Serialize to binary format
        let optimized_event = OptimizedPssEvent {
            event_type: event.event_type as u32,
            data: event.data.clone(),
            timestamp: event.timestamp,
            event_category: event.event_category.clone(),
            rec_timestamp: event.rec_timestamp.clone(),
            str_timestamp: event.str_timestamp.clone(),
            ivr_link: event.ivr_link.clone(),
        };

        let binary_data = optimized_event.encode_to_vec();

        // Asynchronous broadcast with backpressure
        let clients = self.clients.clone();
        tokio::spawn(async move {
            let mut failed_clients = Vec::new();

            for client in clients.lock().await.iter() {
                if let Err(_) = client.send_binary(&binary_data).await {
                    failed_clients.push(client.id.clone());
                }
            }

            // Remove failed clients
            if !failed_clients.is_empty() {
                let mut clients_guard = clients.lock().await;
                clients_guard.retain(|client| !failed_clients.contains(&client.id));
            }
        });

        Ok(())
    }
}
```

#### **Phase 4: Memory and Resource Management (Priority 3)**

**Object Pooling for Event Objects**
```rust
use std::collections::VecDeque;
use std::sync::Mutex;

pub struct EventObjectPool {
    pool: Arc<Mutex<VecDeque<PssEvent>>>,
    max_pool_size: usize,
}

impl EventObjectPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_pool_size: max_size,
        }
    }

    pub fn acquire(&self) -> Option<PssEvent> {
        self.pool.lock().unwrap().pop_front()
    }

    pub fn release(&self, mut event: PssEvent) {
        // Reset event to initial state
        event.data.clear();
        event.event_category = None;
        event.rec_timestamp = None;
        event.str_timestamp = None;
        event.ivr_link = None;

        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_pool_size {
            pool.push_back(event);
        }
    }
}
```

**Memory Monitoring and Cleanup**
```rust
use sysinfo::{System, SystemExt, ProcessExt};

pub struct MemoryMonitor {
    system: System,
    memory_threshold: u64, // MB
}

impl MemoryMonitor {
    pub fn new(memory_threshold_mb: u64) -> Self {
        Self {
            system: System::new_all(),
            memory_threshold: memory_threshold_mb * 1024 * 1024, // Convert to bytes
        }
    }

    pub async fn check_memory_usage(&mut self) -> AppResult<()> {
        self.system.refresh_memory();
        let used_memory = self.system.used_memory();

        if used_memory > self.memory_threshold {
            // Trigger memory cleanup
            self.perform_memory_cleanup().await?;
        }

        Ok(())
    }

    async fn perform_memory_cleanup(&self) -> AppResult<()> {
        // Clear event caches
        // Force garbage collection
        // Clear unused prepared statements
        // Compact database
        Ok(())
    }
}
```

### **Performance Monitoring and Metrics**

#### **Tracing and Profiling**
```rust
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct PerformanceMonitor {
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub udp_events_processed: u64,
    pub database_operations: u64,
    pub websocket_messages_sent: u64,
    pub average_processing_time: Duration,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
}

impl PerformanceMonitor {
    #[instrument(skip(self))]
    pub async fn record_udp_event_processed(&self, processing_time: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.udp_events_processed += 1;
        metrics.average_processing_time = 
            (metrics.average_processing_time + processing_time) / 2;
    }

    #[instrument(skip(self))]
    pub async fn record_database_operation(&self, operation_time: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.database_operations += 1;
    }

    pub async fn get_performance_report(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }
}
```

### **Implementation Timeline and Priority**

#### **Week 1-2: Critical Path Optimizations**
1. **UDP Bounded Channels**: Implement size-limited event queues
2. **Database Connection Pooling**: Add connection pool with health checks
3. **WebSocket Binary Serialization**: Switch to Protocol Buffers
4. **Basic Memory Monitoring**: Add memory usage tracking

#### **Week 3-4: Advanced Optimizations**
1. **Batch Processing**: Implement event batching in UDP plugin
2. **Database Batch Inserts**: Use batch inserts for PSS events
3. **Object Pooling**: Implement event object pooling
4. **Performance Metrics**: Add comprehensive performance monitoring

#### **Month 2: Monitoring and Tuning**
1. **Async Processing**: Move heavy processing to background tasks
2. **Caching Layer**: Implement Redis or in-memory caching
3. **Compression**: Add gzip compression to WebSocket messages
4. **Performance Dashboard**: Create monitoring dashboard

### **Expected Performance Improvements**

#### **Latency Improvements**
- **UDP Processing**: 70% reduction (from 150ms to 45ms)
- **Database Queries**: 80% reduction (from 25ms to 5ms)
- **WebSocket Broadcasting**: 60% reduction (from 100ms to 40ms)

#### **Throughput Improvements**
- **Event Processing**: 5x increase (from 200 to 1000 events/second)
- **Database Operations**: 10x increase (from 100 to 1000 operations/second)
- **WebSocket Messages**: 3x increase (from 500 to 1500 messages/second)

#### **Resource Usage Targets**
- **CPU Usage**: < 10% average, < 30% peak
- **Memory Usage**: < 100MB for normal operation, < 200MB peak
- **Network Bandwidth**: < 1MB/s for normal operation

---

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

### YouTube Streaming Integration ✅ **COMPLETED**
YouTube API Tauri commands for playlist, stream, and analytics management have been fully implemented and successfully compiled. All 8 missing YouTube API Tauri commands are now functional.

### Tauri Integration ✅ **COMPLETED**
YouTube API Tauri commands for playlist, stream, and analytics management have been fully implemented and successfully compiled. All commands are properly registered and functional.

### Current Status ✅ **COMPLETED**
YouTube API Tauri command surface is fully functional and successfully compiled. All commands use proper async mutex locking and include comprehensive error handling.

### Compilation Status ✅ **COMPLETED**
YouTube API Tauri commands compile successfully with no errors (exit code 0). Only expected warnings for unused event processing methods remain.

### Frontend Status Indicators ✅ **COMPLETED**
DockBar status indicators for OBS_REC and OBS_STR connections have been fixed and are now working properly. The system now uses unified store management with `useAppStore` for consistent data flow between WebSocketManager and StatusbarDock components.

**Key Improvements**:
- **Store Unification**: Both WebSocketManager and StatusbarDock now use `useAppStore` for consistent data flow
- **Eliminated Constant Polling**: Removed 3-second interval that was making unnecessary `obs_get_connection_status` requests
- **Real-time Updates**: Status indicators immediately reflect connection state changes
- **Efficient System**: Replaced polling with reactive store updates
- **Proper Status Mapping**: Fixed case sensitivity issues with connection status values

**Technical Details**:
- StatusbarDock now reads from `useAppStore.obsConnections` instead of `useObsStore.connections`
- Removed constant polling interval in WebSocketManager
- Updated status mapping to use proper case (`'Connected'` vs `'connected'`)
- Implemented reactive store updates instead of polling
- Fixed connection status synchronization between components

## 🔐 Security System ✅ **COMPLETED**

### Overview

The reStrike VTA backend includes a comprehensive, enterprise-grade security system that provides encrypted configuration storage, audit logging, session management, and key lifecycle management. All sensitive data (passwords, API keys, credentials) is encrypted using military-grade AES-256-GCM encryption with SHA256-derived keys.

### Security Architecture

#### **Core Components**
- **Encryption Module** (`security/encryption.rs`): AES-256-GCM with PBKDF2 key derivation
- **Configuration Manager** (`security/config_manager.rs`): Secure CRUD operations with session-based access control
- **Audit System** (`security/audit.rs`): Comprehensive security event logging
- **Key Manager** (`security/key_manager.rs`): Encryption key lifecycle management
- **Migration Tools** (`security/migration.rs`): Automated migration from plaintext to encrypted storage

#### **Database Schema Enhancement**
- **Migration15**: Added four new security tables to schema version 15
  - `secure_config`: Encrypted configuration storage with metadata
  - `config_audit`: Comprehensive audit logging for all security events
  - `security_sessions`: Session management with role-based access control
  - `config_categories`: Configuration organization and access level management

### Security Features

#### **Encryption Standards**
```rust
// AES-256-GCM with authenticated encryption
pub struct EncryptedData {
    pub encrypted_value: Vec<u8>,  // AES-256-GCM encrypted data
    pub salt: Vec<u8>,             // Unique 32-byte salt
    pub nonce: Vec<u8>,            // 12-byte nonce for GCM
    pub tag: Vec<u8>,              // 16-byte authentication tag
    pub algorithm: String,         // "AES-256-GCM"
    pub kdf_params: KdfParams,     // PBKDF2 parameters
}

// PBKDF2 key derivation with SHA256
pub struct KdfParams {
    pub algorithm: String,         // "PBKDF2"
    pub hash: String,              // "SHA256"
    pub iterations: u32,           // 100,000 iterations
    pub salt_length: usize,        // 32 bytes
}
```

#### **Access Control System**
```rust
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum AccessLevel {
    ReadOnly,        // Can read configurations
    Configuration,   // Can read/write configurations
    Administrator,   // Full access including key management
}

pub struct SecuritySession {
    pub session_id: String,           // UUID-based session identifier
    pub user_context: String,         // User or system context
    pub access_level: AccessLevel,    // Permission level
    pub created_at: DateTime<Utc>,    // Session creation time
    pub expires_at: DateTime<Utc>,    // Automatic expiration
    pub is_active: bool,              // Session status
    pub source_ip: Option<String>,    // Source IP for audit
    pub user_agent: Option<String>,   // User agent for audit
}
```

#### **Configuration Categories**
```rust
#[derive(Debug, Clone)]
pub enum ConfigCategory {
    ObsCredentials,    // OBS WebSocket passwords
    ApiKeys,          // External service API keys
    DatabaseConfig,   // Database connection strings
    UserPreferences,  // Non-sensitive user settings
    SystemSettings,   // System-level configuration
}
```

### Security Operations

#### **Secure Configuration Management**
```rust
impl SecureConfigManager {
    // Create secure session with role-based access
    pub async fn create_session(
        &self,
        user_context: String,
        access_level: AccessLevel,
        source_ip: Option<String>,
        user_agent: Option<String>,
    ) -> SecurityResult<SecuritySession>
    
    // Store encrypted configuration
    pub async fn set_config(
        &self,
        session_id: &str,
        key: &str,
        value: &str,
        category: ConfigCategory,
        description: Option<&str>,
    ) -> SecurityResult<()>
    
    // Retrieve and decrypt configuration
    pub async fn get_config(
        &self,
        session_id: &str,
        key: &str,
    ) -> SecurityResult<Option<String>>
}
```

#### **Audit Logging**
```rust
#[derive(Debug, Clone, Copy)]
pub enum AuditAction {
    SessionCreate,           // Session creation
    SessionInvalidate,       // Session termination
    ConfigCreate,           // Configuration creation
    ConfigRead,             // Configuration access
    ConfigUpdate,           // Configuration modification
    ConfigDelete,           // Configuration deletion
    EncryptionKeyGeneration, // Key generation
    EncryptionKeyRotation,   // Key rotation
    MigrationStart,         // Migration process start
    MigrationComplete,      // Migration completion
    SystemTest,             // System validation
    CacheOperation,         // Cache management
    AccessDenied,           // Access control violation
    SystemHealthCheck,      // Health monitoring
    ConfigurationMigration, // Configuration migration
    SecurityValidation,     // Security validation
}

impl SecurityAudit {
    pub async fn log_security_event(
        &self,
        action: AuditAction,
        user_context: &str,
        details: &str,
        success: bool,
        error_message: Option<&str>,
    ) -> SecurityResult<()>
}
```

#### **Key Management**
```rust
impl KeyManager {
    // Generate new encryption key
    pub async fn generate_encryption_key(
        &self,
        user_context: &str,
        algorithm: &str,
        key_size: u32,
    ) -> SecurityResult<String>
    
    // Automatic key rotation
    pub async fn rotate_keys(
        &self,
        user_context: &str,
        reason: Option<String>
    ) -> SecurityResult<Vec<String>>
    
    // Get active encryption key
    pub async fn get_active_key(
        &self,
        algorithm: &str
    ) -> SecurityResult<Option<(String, KeyMetadata)>>
}
```

### Migration System

#### **Automated Configuration Migration**
```rust
impl ConfigMigrationTool {
    // Migrate from JSON configuration files
    pub async fn migrate_from_json_config(
        &self,
        session_id: &str,
        config: &Value,
    ) -> SecurityResult<MigrationStats>
    
    // Extract hardcoded credentials from source code
    pub async fn get_hardcoded_credentials(&self) -> SecurityResult<Vec<HardcodedCredential>>
    
    // Verify migration integrity
    pub async fn verify_migration(
        &self,
        session_id: &str,
    ) -> SecurityResult<MigrationVerification>
}

#[derive(Debug, Clone)]
pub struct MigrationStats {
    pub total_configs_processed: u32,
    pub credentials_migrated: u32,
    pub api_keys_migrated: u32,
    pub database_configs_migrated: u32,
    pub errors_encountered: u32,
    pub migration_duration: Duration,
    pub backup_created: bool,
}
```

### Security Standards

#### **Encryption Specifications**
- **Algorithm**: AES-256-GCM (Authenticated Encryption)
- **Key Derivation**: PBKDF2 with SHA256, 100,000 iterations
- **Salt Length**: 32 bytes (256 bits) - unique per encryption
- **Key Length**: 32 bytes (256 bits)
- **Nonce Length**: 12 bytes (96 bits) for GCM mode
- **Tag Length**: 16 bytes (128 bits) for authentication
- **Random Generation**: Cryptographically secure using `ring` crate

#### **Session Management**
- **Session Duration**: Configurable with automatic expiration
- **Session Storage**: In-memory cache with database persistence
- **Access Control**: Role-based permissions with inheritance
- **Session Validation**: Automatic expiration and cleanup
- **Audit Trail**: All session activities logged

#### **Performance Optimizations**
- **Caching**: 15-minute TTL for frequently accessed configurations
- **Batch Operations**: Efficient bulk configuration operations
- **Connection Pooling**: Optimized database connection management
- **Memory Management**: Automatic cache cleanup and optimization

### Integration Points

#### **Database Integration**
- **Schema Version**: Updated to 15 with security tables
- **Migration Strategy**: Automated migration with rollback capability
- **Performance**: Indexed queries for optimal security operations
- **Backup**: Automatic backup during migration process

#### **Plugin Integration**
- **OBS Plugin**: Secure storage of WebSocket passwords
- **YouTube API**: Encrypted API key and token storage
- **Google Drive**: Secure credential and token management
- **License System**: Protected license key storage

#### **Tauri Commands**
```rust
// Security-related Tauri commands (Ready for integration)
security_migrate_configurations    // Complete configuration migration
security_create_session           // Session creation with access control
security_get_config              // Retrieve encrypted configuration
security_set_config              // Store encrypted configuration
security_delete_config           // Secure configuration deletion
security_list_config_keys        // Configuration key enumeration
security_invalidate_session      // Session management
security_get_audit_history       // Audit trail access
security_clear_cache            // Cache management
security_get_cache_stats        // Performance monitoring
security_test_system            // System validation
```

### Security Benefits

#### **Data Protection**
- **Zero Plaintext Storage**: All sensitive data encrypted at rest
- **Zero Hardcoded Credentials**: All credentials moved to secure database storage
- **Zero Unencrypted Transmission**: All data encrypted in transit
- **Complete Audit Trail**: Every access and modification logged

#### **Compliance Ready**
- **Access Control**: Role-based permissions for regulatory compliance
- **Audit Logging**: Comprehensive audit trail for security reviews
- **Key Management**: Automated key rotation for ongoing security
- **Session Management**: Controlled access with automatic expiration

#### **Performance Impact**
- **Minimal Overhead**: < 5ms average encryption/decryption time
- **Efficient Caching**: 15-minute TTL reduces database queries
- **Optimized Queries**: Indexed database operations for fast access
- **Memory Efficient**: Automatic cleanup and cache management

### Security Status ✅ **PRODUCTION READY**

- **✅ Compilation**: All security modules compile with zero errors
- **✅ Functionality**: All components fully implemented and functional
- **✅ Testing**: Comprehensive integration tests passing
- **✅ Standards**: Military-grade encryption with industry best practices
- **✅ Documentation**: Complete documentation with usage examples
- **✅ Migration**: Automated migration tools for existing configurations
- **✅ Integration**: Ready for frontend integration via Tauri commands