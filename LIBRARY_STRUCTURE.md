# Library Structure Documentation

## Overview

This document describes the Rust backend library structure for reStrike VTA, including the plugin system, core modules, and data flow patterns.

## Library Architecture

```
src-tauri/src/
├── lib.rs                    # Library entry point and exports
├── main.rs                   # Application entry point
├── tauri_commands.rs         # Tauri command definitions
├── core/                     # Core application logic
│   ├── app.rs               # Application state and lifecycle
│   ├── config.rs            # Configuration management
│   ├── state.rs             # Global state management
│   └── mod.rs
├── plugins/                  # Plugin system
│   ├── mod.rs               # Plugin registry and management
│   ├── plugin_obs.rs        # OBS WebSocket integration
│   ├── plugin_playback.rs   # Video playback management
│   ├── plugin_store.rs      # Data persistence
│   ├── plugin_udp.rs        # UDP communication
│   ├── plugin_cpu_monitor.rs # NEW: CPU monitoring system
│   └── plugin_license.rs    # License management
├── obs/                      # OBS integration modules
├── pss/                      # PSS protocol handling
├── video/                    # Video processing
├── config/                   # Configuration management
├── logging/                  # Logging system
├── types/                    # Shared type definitions
├── utils/                    # Utility functions
└── commands/                 # Command implementations
```

## Plugin System

### Plugin Architecture

The plugin system provides a modular approach to different functionalities:

```rust
// src-tauri/src/plugins/mod.rs
pub mod plugin_obs;
pub mod plugin_playback;
pub mod plugin_store;
pub mod plugin_udp;
pub mod plugin_cpu_monitor; // NEW
pub mod plugin_license;

// Plugin trait for common interface
pub trait Plugin {
    fn name(&self) -> &str;
    fn init(&self) -> AppResult<()>;
    fn shutdown(&self) -> AppResult<()>;
}
```

### CPU Monitoring Plugin (NEW - 2025-01-28)

#### **File**: `src-tauri/src/plugins/plugin_cpu_monitor.rs`

**Purpose**: Real-time CPU and memory monitoring for system processes

**Key Features**:
- System CPU usage tracking
- Individual process monitoring
- Memory usage tracking
- Background monitoring with configurable intervals
- Process filtering (>0.1% CPU or >10MB memory)

**Data Structures**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuProcessData {
    pub process_name: String,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCpuData {
    pub total_cpu_percent: f64,
    pub cores: Vec<f64>,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct CpuMonitorConfig {
    pub enabled: bool,
    pub update_interval_seconds: u64,
    pub monitored_processes: Vec<String>,
    pub include_system_cpu: bool,
}
```

**Core Implementation**:
```rust
pub struct CpuMonitorPlugin {
    config: Arc<Mutex<CpuMonitorConfig>>,
    process_data: Arc<Mutex<HashMap<String, CpuProcessData>>>,
    system_data: Arc<Mutex<Option<SystemCpuData>>>,
    monitoring_active: Arc<Mutex<bool>>,
}
```

**Key Methods**:
- `update_all_processes()` - Collects process data using `wmic` commands
- `update_system_cpu()` - Updates system CPU information
- `get_process_cpu_data()` - Returns current process data
- `get_system_cpu_data()` - Returns system CPU data
- `start_monitoring()` - Starts background monitoring task
- `stop_monitoring()` - Stops monitoring

**Status**: ✅ Implemented, awaiting `wmic` installation for testing

### OBS Plugin

#### **File**: `src-tauri/src/plugins/plugin_obs.rs`

**Purpose**: OBS Studio WebSocket integration

**Key Features**:
- WebSocket connection management
- Scene switching and control
- Source management
- Connection status monitoring

### Playback Plugin

#### **File**: `src-tauri/src/plugins/plugin_playback.rs`

**Purpose**: Video playback and replay management

**Key Features**:
- Video player control
- Clip management
- Replay functionality
- Video file handling

### Store Plugin

#### **File**: `src-tauri/src/plugins/plugin_store.rs`

**Purpose**: Data persistence and storage

**Key Features**:
- SQLite database management
- Configuration storage
- Event logging
- Data archival

### UDP Plugin

#### **File**: `src-tauri/src/plugins/plugin_udp.rs`

**Purpose**: UDP communication for real-time data

**Key Features**:
- Network communication
- Data streaming
- Protocol handling

## Core Modules

### Application Core

#### **File**: `src-tauri/src/core/app.rs`

**Purpose**: Application state and lifecycle management

**Key Features**:
- Plugin initialization and management
- Global state coordination
- Application lifecycle events
- Error handling

```rust
pub struct App {
    pub cpu_monitor_plugin: CpuMonitorPlugin,
    pub obs_plugin: ObsPlugin,
    pub playback_plugin: PlaybackPlugin,
    pub store_plugin: StorePlugin,
    pub udp_plugin: UdpPlugin,
    // ... other plugins
}
```

### Configuration Management

#### **File**: `src-tauri/src/config/manager.rs`

**Purpose**: Configuration loading and management

**Key Features**:
- JSON configuration files
- Environment-specific settings
- Runtime configuration updates
- Validation and error handling

### Logging System

#### **File**: `src-tauri/src/logging/logger.rs`

**Purpose**: Structured logging and archival

**Key Features**:
- Multiple log levels (debug, info, warn, error)
- Log rotation and archival
- File and console output
- Performance monitoring

## Tauri Commands

### Command Definitions

#### **File**: `src-tauri/src/tauri_commands.rs`

**Purpose**: Tauri command definitions for frontend-backend communication

**CPU Monitoring Commands** (NEW):
```rust
#[tauri::command]
pub async fn cpu_get_process_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    let process_data = app.cpu_monitor_plugin().get_process_cpu_data().await;
    Ok(serde_json::json!({
        "success": true,
        "processes": process_data
    }))
}

#[tauri::command]
pub async fn cpu_get_system_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String> {
    let system_data = app.cpu_monitor_plugin().get_system_cpu_data().await;
    Ok(serde_json::json!({
        "success": true,
        "system": system_data
    }))
}
```

**Command Registration**:
```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            cpu_get_process_data,
            cpu_get_system_data,
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Error Handling

### Error Types

#### **File**: `src-tauri/src/types/mod.rs`

**Purpose**: Shared error types and result handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    // ... other error types
}

pub type AppResult<T> = Result<T, AppError>;
```

**Error Handling Patterns**:
- Use `AppResult<T>` for all plugin and core methods
- Convert `std::io::Error` to `AppError::IoError(e)`
- Use `AppError::ConfigError(e.to_string())` for custom error messages
- Never use `AppError::IoError` with String or formatted messages

## Data Flow Patterns

### CPU Monitoring Flow (NEW)

```
1. Background Task (plugin_cpu_monitor.rs)
   ↓ update_all_processes()
2. WMIC Command Execution
   ↓ wmic process get name,processid,workingsetsize,percentprocessortime /format:csv
3. Data Parsing and Filtering
   ↓ Process CSV output, filter significant processes
4. State Update
   ↓ Update process_data HashMap
5. Frontend Request (tauri_commands.rs)
   ↓ cpu_get_process_data command
6. JSON Serialization
   ↓ Convert to serde_json::Value
7. Frontend Display (CpuMonitoringSection.tsx)
   ↓ React component rendering
```

### General Data Flow

```
Frontend Request → Tauri Command → Plugin Method → System Call → Data Processing → Response
```

## Type System

### Shared Types

#### **File**: `src-tauri/src/types/mod.rs`

**Purpose**: Common type definitions used across modules

```rust
// CPU monitoring types
pub type ProcessData = HashMap<String, CpuProcessData>;
pub type SystemData = Option<SystemCpuData>;

// Plugin types
pub type PluginResult<T> = Result<T, Box<dyn std::error::Error>>;

// Configuration types
pub type ConfigValue = serde_json::Value;
```

## Utility Functions

### Common Utilities

#### **File**: `src-tauri/src/utils/logger.rs`

**Purpose**: Logging utilities and helpers

```rust
pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Logging setup implementation
}

pub fn log_error(context: &str, error: &dyn std::error::Error) {
    log::error!("[{}] Error: {}", context, error);
}
```

## Testing Strategy

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_monitor_plugin_creation() {
        let config = CpuMonitorConfig::default();
        let plugin = CpuMonitorPlugin::new(config);
        assert_eq!(plugin.name(), "cpu_monitor");
    }

    #[tokio::test]
    async fn test_process_data_collection() {
        // Test process data collection
  }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_cpu_monitoring_workflow() {
        // Test complete CPU monitoring workflow
    }
}
```

## Performance Considerations

### Memory Management

- Use `Arc<Mutex<T>>` for shared state across async tasks
- Implement proper cleanup in plugin shutdown methods
- Monitor memory usage in CPU monitoring plugin

### Async Operations

- Use `tokio` for async runtime
- Implement proper error handling in async functions
- Use background tasks for continuous monitoring

### Error Handling

- Implement comprehensive error handling
- Use structured logging for debugging
- Provide meaningful error messages to frontend

## Security Considerations

### Input Validation

- Validate all input data from frontend
- Sanitize process names and data
- Implement proper error handling without information disclosure

### System Access

- Limit system access to necessary operations
- Implement proper permissions for `wmic` commands
- Handle command execution failures gracefully

## Current Status (2025-01-28)

### ✅ **Completed**
- Plugin system architecture
- CPU monitoring plugin implementation
- Tauri command integration
- Error handling patterns
- Logging system
- Configuration management

### 🚧 **In Progress**
- CPU monitoring testing with `wmic`
- Performance optimization
- Error handling improvements

### 📋 **Planned**
- Enhanced error handling
- Performance monitoring
- Additional plugin features
- Comprehensive testing

---

**Last Updated**: 2025-01-28
**Version**: 0.1.0
**Status**: CPU monitoring implementation complete, awaiting testing 