# Library Structure Documentation

## Overview
The reStrike VTA library provides a comprehensive foundation for building Windows-native desktop applications with advanced OBS integration, real-time event processing, and video replay management. The architecture follows a plugin-based microkernel pattern with robust configuration management.

## Core Architecture

### Plugin-Based Microkernel
The application uses a microkernel architecture where core functionality is provided by independent plugins that communicate through well-defined interfaces.

```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Application Layer                  │
├─────────────────────────────────────────────────────────────┤
│                    Core Application Layer                   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   Config    │ │   Logging   │ │    Types    │ │  Core   │ │
│  │  Manager    │ │   Manager   │ │             │ │  App    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Plugin Layer                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │    OBS      │ │     UDP     │ │  Playback   │ │  Store  │ │
│  │   Plugin    │ │   Plugin    │ │   Plugin    │ │ Plugin  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │  WebSocket  │ │    SQLite   │ │   File I/O  │ │ Network │ │
│  │             │ │             │ │             │ │         │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Module Structure

### Core Modules (`src-tauri/src/core/`)

#### Application Core (`core/app.rs`)
- **Purpose**: Main application orchestration and lifecycle management
- **Key Components**:
  - `App`: Main application class
  - `AppState`: Application state management
  - Plugin coordination and event routing
  - Configuration manager integration
  - Logging system integration

#### Configuration Management (`config/`)
- **Purpose**: Comprehensive settings persistence and management
- **Key Components**:
  - `ConfigManager`: Configuration persistence and sync
  - `AppConfig`: Complete configuration data structures
  - Automatic backup and restore functionality
  - Cross-session settings persistence

**Configuration Segments**:
```rust
pub struct AppConfig {
    pub app: AppSettings,           // Application metadata and core settings
    pub obs: ObsSettings,           // OBS WebSocket connection settings
    pub udp: UdpSettings,           // UDP/PSS protocol settings
    pub logging: LoggingSettings,   // Logging and diagnostics settings
    pub ui: UiSettings,             // UI and overlay settings
    pub video: VideoSettings,       // Video playback settings
    pub license: LicenseSettings,   // License and activation settings
    pub flags: FlagSettings,        // Flag management settings
    pub advanced: AdvancedSettings, // Advanced settings and features
}
```

### Plugin Modules (`src-tauri/src/plugins/`)

#### OBS Plugin (`plugin_obs.rs`)
- **Purpose**: OBS Studio WebSocket integration
- **Key Features**:
  - Dual protocol support (v4/v5)
  - Multiple connection management
  - Real-time status monitoring
  - Authentication handling
  - Scene and recording control

**Key Structures**:
```rust
pub struct ObsPlugin {
    connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    event_tx: mpsc::UnboundedSender<ObsEvent>,
    debug_ws_messages: Arc<Mutex<bool>>,
}

pub struct ObsConnection {
    pub config: ObsConnectionConfig,
    pub status: ObsConnectionStatus,
    pub websocket: Option<WebSocketStream>,
    pub request_id_counter: u64,
    pub pending_requests: HashMap<String, oneshot::Sender<Value>>,
}
```

#### UDP Plugin (`plugin_udp.rs`)
- **Purpose**: PSS protocol handling and event processing
- **Key Features**:
  - UDP listener management
  - PSS protocol parsing
  - Event filtering and processing
  - Real-time data streaming

#### Playback Plugin (`plugin_playback.rs`)
- **Purpose**: Video clip management and playback
- **Key Features**:
  - MPV integration
  - Clip extraction and management
  - Video metadata handling
  - Playback controls

#### Store Plugin (`plugin_store.rs`)
- **Purpose**: Event storage and database operations
- **Key Features**:
  - SQLite database management
  - Event persistence
  - Query and filtering
  - Data export/import

#### License Plugin (`plugin_license.rs`)
- **Purpose**: License validation and management
- **Key Features**:
  - Hardware-locked activation
  - Offline grace period
  - Periodic validation
  - License status tracking

### Logging System (`src-tauri/src/logging/`)

#### Log Manager (`logging/manager.rs`)
- **Purpose**: Structured logging with file management
- **Key Features**:
  - Multi-subsystem logging
  - File rotation and compression
  - Log archiving
  - Live data streaming
  - Subsystem enable/disable

**Key Structures**:
```rust
pub struct LogManager {
    config: LogConfig,
    log_dir: PathBuf,
    subsystems: Arc<RwLock<HashMap<String, bool>>>,
}

pub struct LogConfig {
    pub directory: String,
    pub max_size_mb: u64,
    pub max_files: usize,
    pub retention_days: u64,
    pub compression: bool,
    pub archive_enabled: bool,
}
```

### Type System (`src-tauri/src/types/`)

#### Error Handling (`types/errors.rs`)
- **Purpose**: Centralized error handling and propagation
- **Key Features**:
  - `AppError` enum for all application errors
  - `AppResult<T>` type alias for consistent error handling
  - Error conversion and propagation utilities

**Error Types**:
```rust
pub enum AppError {
    IoError(std::io::Error),
    ConfigError(String),
    ObsError(String),
    UdpError(String),
    PlaybackError(String),
    StoreError(String),
    LicenseError(String),
    ValidationError(String),
}
```

## Configuration System Architecture

### Configuration Manager (`config/manager.rs`)
The configuration manager provides comprehensive settings persistence with the following features:

#### Core Functionality
- **Persistence**: Automatic saving to JSON files
- **Backup**: Automatic backup creation and restoration
- **Thread Safety**: RwLock for concurrent access
- **Validation**: Configuration validation and error handling
- **Statistics**: Configuration health monitoring

#### Key Methods
```rust
impl ConfigManager {
    pub async fn get_config(&self) -> AppConfig
    pub async fn update_config(&self, new_config: AppConfig) -> AppResult<()>
    pub async fn update_section<F, T>(&self, section_updater: F) -> AppResult<()>
    pub async fn reset_to_defaults(&self) -> AppResult<()>
    pub async fn export_config(&self, export_path: &Path) -> AppResult<()>
    pub async fn import_config(&self, import_path: &Path) -> AppResult<()>
    pub async fn restore_from_backup(&self) -> AppResult<()>
    pub async fn get_config_stats(&self) -> AppResult<ConfigStats>
}
```

### Configuration Segments

#### App Settings
```rust
pub struct AppSettings {
    pub version: String,
    pub last_save: String,
    pub startup: StartupSettings,
    pub performance: PerformanceSettings,
}
```

#### OBS Settings
```rust
pub struct ObsSettings {
    pub connections: Vec<ObsConnectionConfig>,
    pub defaults: ObsDefaultSettings,
    pub behavior: ObsBehaviorSettings,
}
```

#### Logging Settings
```rust
pub struct LoggingSettings {
    pub global: GlobalLoggingSettings,
    pub subsystems: HashMap<String, SubsystemLoggingSettings>,
    pub files: LogFileSettings,
    pub live_data: LiveDataSettings,
}
```

## Tauri Integration

### Command Layer (`tauri_commands.rs`)
The Tauri command layer provides the bridge between the frontend and backend:

#### OBS Commands
- `obs_add_connection`: Add new OBS connection
- `obs_remove_connection`: Remove OBS connection
- `obs_get_connections`: Get all OBS connections
- `obs_connect_to_connection`: Connect to specific OBS instance
- `obs_get_connection_status`: Get connection status
- `obs_get_status`: Get comprehensive OBS status

#### Configuration Commands
- `get_settings`: Get all application settings
- `update_settings`: Update application settings
- `get_config_stats`: Get configuration statistics
- `reset_settings`: Reset to defaults
- `export_settings`: Export configuration
- `import_settings`: Import configuration
- `restore_settings_backup`: Restore from backup

#### Logging Commands
- `set_logging_enabled`: Enable/disable subsystem logging
- `list_log_files`: List available log files
- `download_log_file`: Download specific log file
- `list_archives`: List log archives
- `extract_archive`: Extract log archive
- `download_archive`: Download log archive

## Frontend Integration

### State Management (`ui/src/stores/`)
The frontend uses Zustand for state management with the following stores:

#### App Store
```typescript
export interface AppState {
  obsConnections: ObsConnection[];
  activeObsConnection: string | null;
  obsStatus: ObsStatusInfo | null;
  overlaySettings: OverlaySettings;
  videoClips: VideoClip[];
  currentClip: VideoClip | null;
  isPlaying: boolean;
  currentView: AppView;
  isLoading: boolean;
  error: string | null;
  isAdvancedPanelOpen: boolean;
}
```

### Tauri Commands (`ui/src/utils/tauriCommands.ts`)
Frontend wrappers for Tauri commands:

#### OBS Commands
```typescript
export const obsCommands = {
  async addConnection(config: ObsConnectionConfig): Promise<TauriResult>
  async removeConnection(connectionName: string): Promise<TauriResult>
  async getConnections(): Promise<TauriResult>
  async connectToConnection(connectionName: string): Promise<TauriResult>
  async getConnectionStatus(connectionName: string): Promise<TauriResult>
  async disconnect(connectionName: string): Promise<TauriResult>
}
```

#### Configuration Commands
```typescript
export const configCommands = {
  async getSettings(): Promise<TauriResult>
  async updateSettings(settings: any): Promise<TauriResult>
  async getConfigStats(): Promise<TauriResult>
  async resetSettings(): Promise<TauriResult>
  async exportSettings(exportPath: string): Promise<TauriResult>
  async importSettings(importPath: string): Promise<TauriResult>
  async restoreSettingsBackup(): Promise<TauriResult>
}
```

## Error Handling Strategy

### Backend Error Handling
1. **AppError Enum**: Centralized error types
2. **AppResult<T>**: Consistent error propagation
3. **Error Conversion**: Proper error type conversion
4. **Logging**: Comprehensive error logging

### Frontend Error Handling
1. **TauriResult**: Standardized result format
2. **Error Boundaries**: React error boundaries
3. **User Feedback**: User-friendly error messages
4. **Retry Logic**: Automatic retry for transient errors

## Performance Considerations

### Backend Performance
- **Async Operations**: All I/O operations are async
- **Thread Safety**: RwLock for concurrent access
- **Memory Management**: Efficient memory usage
- **Resource Cleanup**: Proper resource disposal

### Frontend Performance
- **React Optimization**: React.memo and useMemo
- **Bundle Optimization**: Tree shaking and code splitting
- **State Management**: Efficient Zustand usage
- **Component Design**: Atomic design for reusability

## Security Considerations

### Configuration Security
- **Password Handling**: Secure password storage
- **File Permissions**: Proper file permissions
- **Backup Security**: Secure backup storage
- **Validation**: Input validation and sanitization

### Network Security
- **WebSocket Security**: Secure WebSocket connections
- **Authentication**: Proper authentication handling
- **Data Validation**: Network data validation
- **Error Handling**: Secure error handling

## Testing Strategy

### Backend Testing
- **Unit Tests**: Individual component testing
- **Integration Tests**: Plugin interaction testing
- **Configuration Tests**: Configuration system testing
- **Error Tests**: Error handling testing

### Frontend Testing
- **Component Tests**: React component testing
- **Integration Tests**: Tauri command testing
- **E2E Tests**: End-to-end workflow testing
- **Performance Tests**: Performance benchmarking

## Maintenance and Updates

### Configuration Migration
- **Version Management**: Configuration version tracking
- **Migration Scripts**: Automatic migration scripts
- **Backward Compatibility**: Backward compatibility support
- **Validation**: Configuration validation

### Plugin Updates
- **Plugin Lifecycle**: Plugin initialization and cleanup
- **Event System**: Plugin event communication
- **Error Handling**: Plugin error isolation
- **Performance Monitoring**: Plugin performance tracking

---

*Last updated: 2025-01-28*
*Configuration system: Complete*
*OBS WebSocket management: Complete*
*Error handling: Comprehensive*
*Performance optimization: Implemented* 