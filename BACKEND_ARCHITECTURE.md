# Backend Architecture & Plugin System

## Overview

The reStrike VTA backend is built with Rust using Tauri v2, featuring a modular plugin architecture that provides real-time event processing, OBS integration, UDP server management, and comprehensive system monitoring. The backend is designed for high performance, reliability, and extensibility.

## 🏗️ Backend Architecture

### **Technology Stack**
- **Framework**: Tauri v2 with Rust backend
- **Language**: Rust with async/await support
- **Architecture**: Plugin-based microkernel architecture
- **Database**: SQLite with custom migration system
- **Networking**: UDP for PSS events, WebSocket for OBS integration
- **Error Handling**: Custom `AppError` and `AppResult` types

### **Core Principles**
- **Modularity**: Each subsystem is a separate plugin
- **Error Safety**: Comprehensive error handling with custom types
- **Async First**: All I/O operations are asynchronous
- **Type Safety**: Full Rust type safety throughout
- **Performance**: Optimized for real-time operations

---

## 📁 Directory Structure

```
src-tauri/
├── src/
│   ├── main.rs              # Tauri app entry point
│   ├── lib.rs               # Library exports and plugin registration
│   ├── tauri_commands.rs    # Tauri command definitions (3112 lines)
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
│   │   ├── models.rs        # Data models
│   │   ├── operations.rs    # Database operations
│   │   ├── migrations.rs    # Migration system
│   │   └── connection.rs    # Connection management
│   ├── types/               # Shared types
│   │   └── mod.rs           # Type definitions
│   └── utils/               # Utility functions
│       ├── logger.rs        # Logging utilities
│       └── network.rs       # Network interface detection
├── Cargo.toml               # Rust dependencies
├── tauri.conf.json          # Tauri configuration
├── capabilities.json        # Tauri capabilities
└── build.rs                 # Build script
```

---

## 🔌 Plugin System

### **Plugin Architecture Overview**

The backend uses a plugin-based architecture where each major subsystem is implemented as a separate plugin. This provides:

- **Modularity**: Each plugin can be developed and tested independently
- **Extensibility**: New plugins can be added without modifying existing code
- **Maintainability**: Clear separation of concerns
- **Testability**: Each plugin can be unit tested in isolation

### **Core Plugins**

#### **1. Database Plugin (`plugin_database.rs`)**
```rust
pub struct DatabasePlugin {
    connection: Arc<DatabaseConnection>,
    migration_strategy: MigrationStrategy,
    hybrid_provider: Arc<Mutex<HybridSettingsProvider>>,
}
```

**Responsibilities:**
- Database connection management
- Migration system
- Settings persistence
- Data operations for all subsystems

**Key Features:**
- Async connection pooling
- Automatic migration system
- Settings backup and restore
- Google Drive integration

#### **2. UDP Plugin (`plugin_udp.rs`)**
```rust
pub struct UdpServer {
    config: UdpServerConfig,
    database: Arc<DatabasePlugin>,
    current_session_id: Arc<Mutex<Option<i64>>>,
    current_match_id: Arc<Mutex<Option<i64>>>,
    athlete_cache: Arc<Mutex<HashMap<String, i64>>>,
    event_type_cache: Arc<Mutex<HashMap<String, i64>>>,
    // ... other fields
}
```

**Responsibilities:**
- PSS protocol UDP server
- Real-time event processing
- Network interface detection
- Event storage and caching

**Key Features:**
- Real-time PSS event parsing
- Network interface optimization
- Event caching for performance
- Session tracking and statistics

#### **3. OBS Plugin (`plugin_obs.rs`)**
```rust
pub struct ObsPlugin {
    connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    event_tx: mpsc::Sender<ObsEvent>,
    database: Arc<DatabasePlugin>,
}
```

**Responsibilities:**
- OBS Studio WebSocket integration
- Recording and streaming control
- Scene management
- Status monitoring

**Key Features:**
- WebSocket v5 protocol support
- Connection management
- Real-time status updates
- Recording control

#### **4. CPU Monitor Plugin (`plugin_cpu_monitor.rs`)**
```rust
pub struct CpuMonitorPlugin {
    config: CpuMonitorConfig,
    monitoring_enabled: Arc<AtomicBool>,
    stats_tx: mpsc::Sender<CpuStats>,
}
```

**Responsibilities:**
- System resource monitoring
- Performance metrics collection
- Real-time statistics reporting

**Key Features:**
- CPU, memory, and disk monitoring
- OBS-specific resource tracking
- Real-time statistics streaming
- Configurable monitoring intervals

#### **5. License Plugin (`plugin_license.rs`)**
```rust
pub struct LicensePlugin {
    license_key: Arc<Mutex<Option<String>>>,
    license_status: Arc<Mutex<LicenseStatus>>,
    http_client: reqwest::Client,
}
```

**Responsibilities:**
- License validation and management
- Online activation
- Offline validation
- License status tracking

**Key Features:**
- Online license activation
- Offline validation with grace period
- License status monitoring
- Secure license storage

---

## 🔧 Core Systems

### **Application Core (`core/app.rs`)**
```rust
pub struct App {
    config_manager: ConfigManager,
    database_plugin: DatabasePlugin,
    udp_plugin: UdpPlugin,
    obs_plugin: ObsPlugin,
    cpu_monitor_plugin: CpuMonitorPlugin,
    license_plugin: LicensePlugin,
    event_bus: tokio::sync::broadcast::Sender<AppEvent>,
}
```

**Responsibilities:**
- Plugin lifecycle management
- Event bus coordination
- Application state management
- Startup and shutdown orchestration

### **Configuration System (`config/`)**
```rust
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    backup_path: PathBuf,
}
```

**Features:**
- JSON-based configuration
- Automatic backup and restore
- Configuration validation
- Hot-reload support

### **Logging System (`logging/`)**
```rust
pub struct Logger {
    log_file: Arc<Mutex<File>>,
    rotation_config: RotationConfig,
    archival_config: ArchivalConfig,
}
```

**Features:**
- Multi-level logging
- Automatic log rotation
- Log archival system
- Subsystem-specific logging

---

## 🚀 Tauri Integration

### **Command System (`tauri_commands.rs`)**

The backend exposes functionality to the frontend through Tauri commands:

#### **UDP Commands**
```rust
#[tauri::command]
pub async fn start_udp_server(app: State<'_, Arc<App>>) -> Result<(), String>

#[tauri::command]
pub async fn stop_udp_server(app: State<'_, Arc<App>>) -> Result<(), String>

#[tauri::command]
pub async fn get_udp_status(app: State<'_, Arc<App>>) -> Result<String, String>
```

#### **OBS Commands**
```rust
#[tauri::command]
pub async fn obs_connect(url: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

#[tauri::command]
pub async fn obs_start_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

#[tauri::command]
pub async fn obs_stop_recording(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>
```

#### **Database Commands**
```rust
#[tauri::command]
pub async fn db_get_ui_setting(key: String, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

#[tauri::command]
pub async fn db_set_ui_setting(key: String, value: String, changed_by: String, change_reason: Option<String>, app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>
```

#### **Flag Management Commands**
```rust
#[tauri::command]
pub async fn get_flag_mappings_data(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>

#[tauri::command]
pub async fn scan_and_populate_flags(app: State<'_, Arc<App>>) -> Result<serde_json::Value, String>
```

### **Event System**

The backend uses Tauri's event system for real-time communication:

```rust
// Emit events to frontend
window.emit("pss_event", event_data)?;

// Listen for frontend events
window.listen("frontend_event", move |event| {
    // Handle frontend events
});
```

---

## 🔄 Data Flow

### **PSS Event Processing Flow**

1. **UDP Reception**: UDP server receives PSS datagrams
2. **Event Parsing**: Parse PSS protocol events
3. **Database Storage**: Store events in database
4. **Cache Update**: Update in-memory caches
5. **Frontend Notification**: Emit events to frontend
6. **UI Update**: Frontend updates in real-time

### **OBS Integration Flow**

1. **WebSocket Connection**: Connect to OBS Studio
2. **Status Monitoring**: Monitor OBS status
3. **Command Execution**: Execute OBS commands
4. **Event Handling**: Handle OBS events
5. **Frontend Updates**: Update UI with OBS status

### **Database Operations Flow**

1. **Connection Pool**: Get database connection
2. **Transaction Management**: Begin transaction
3. **Data Operations**: Execute database operations
4. **Error Handling**: Handle database errors
5. **Commit/Rollback**: Commit or rollback transaction

---

## 🛡️ Error Handling

### **Error Types**
```rust
pub enum AppError {
    ConfigError(String),
    IoError(std::io::Error),
    DatabaseError(String),
    NetworkError(String),
    ValidationError(String),
    LicenseError(String),
}
```

### **Error Handling Patterns**
```rust
// Convert database errors
.map_err(|e| AppError::DatabaseError(format!("Failed to get data: {}", e)))

// Convert IO errors
.map_err(|e| AppError::IoError(e))

// Convert network errors
.map_err(|e| AppError::NetworkError(format!("Network error: {}", e)))
```

### **Result Types**
```rust
pub type AppResult<T> = Result<T, AppError>;
pub type DatabaseResult<T> = Result<T, rusqlite::Error>;
```

---

## 🔍 Performance Optimization

### **Caching Strategy**
```rust
// In-memory caches for frequently accessed data
pub struct UdpServer {
    athlete_cache: Arc<Mutex<HashMap<String, i64>>>,
    event_type_cache: Arc<Mutex<HashMap<String, i64>>>,
}
```

### **Async Operations**
```rust
// Async database operations
pub async fn get_network_interfaces(&self) -> AppResult<Vec<NetworkInterface>> {
    let conn = self.connection.get_connection().await?;
    PssUdpOperations::get_network_interfaces(&*conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to get network interfaces: {}", e)))
}
```

### **Connection Pooling**
```rust
// Database connection pooling
pub struct DatabaseConnection {
    connection: Arc<Mutex<rusqlite::Connection>>,
}
```

---

## 🔧 Development Guidelines

### **Plugin Development**

#### **1. Plugin Structure**
```rust
pub struct MyPlugin {
    // Plugin-specific fields
    config: MyPluginConfig,
    database: Arc<DatabasePlugin>,
}

impl MyPlugin {
    pub fn new(config: MyPluginConfig, database: Arc<DatabasePlugin>) -> Self {
        Self { config, database }
    }
    
    pub async fn start(&self) -> AppResult<()> {
        // Plugin startup logic
    }
    
    pub async fn stop(&self) -> AppResult<()> {
        // Plugin shutdown logic
    }
}
```

#### **2. Error Handling**
- Always use `AppResult<T>` for public methods
- Convert specific errors to `AppError` variants
- Provide meaningful error messages
- Log errors with context

#### **3. Async Operations**
- Use `async/await` for all I/O operations
- Avoid blocking operations in async contexts
- Use proper error propagation
- Handle cancellation gracefully

#### **4. State Management**
- Use `Arc<Mutex<T>>` for shared state
- Minimize lock contention
- Use atomic types when possible
- Document thread safety guarantees

### **Testing Guidelines**

#### **1. Unit Testing**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_operation() {
        // Test implementation
    }
}
```

#### **2. Integration Testing**
- Test plugin interactions
- Test error scenarios
- Test performance under load
- Test real-world usage patterns

---

## 📊 Monitoring and Diagnostics

### **Logging System**
```rust
// Structured logging
log::info!("UDP server started on port {}", port);
log::error!("Failed to parse PSS event: {}", error);
log::debug!("Processing event: {:?}", event);
```

### **Performance Metrics**
- Database query performance
- Network interface statistics
- Memory usage monitoring
- CPU utilization tracking

### **Health Checks**
```rust
pub async fn health_check(&self) -> AppResult<HealthStatus> {
    // Check all subsystems
    // Return comprehensive health status
}
```

---

## 🔮 Future Enhancements

### **Planned Features**

#### **1. Advanced Plugin System**
- Dynamic plugin loading
- Plugin hot-swapping
- Plugin dependency management
- Plugin marketplace

#### **2. Performance Improvements**
- Connection pooling optimization
- Query optimization
- Memory management improvements
- Async operation optimization

#### **3. Monitoring Enhancements**
- Real-time performance dashboards
- Automated alerting
- Performance trend analysis
- Resource usage optimization

#### **4. Security Enhancements**
- Enhanced license validation
- Secure communication protocols
- Data encryption
- Access control improvements

---

## 📞 Troubleshooting

### **Common Issues**

#### **1. Plugin Startup Failures**
- Check plugin dependencies
- Verify configuration
- Review error logs
- Test plugin isolation

#### **2. Performance Issues**
- Monitor resource usage
- Check database performance
- Analyze network usage
- Review caching effectiveness

#### **3. Integration Issues**
- Verify Tauri configuration
- Check event system
- Review command definitions
- Test frontend communication

### **Debugging Tools**
- Comprehensive logging
- Performance profiling
- Memory leak detection
- Network traffic analysis

---

**Last Updated**: 2025-01-29  
**Architecture Version**: 2.0  
**Status**: Production Ready with Plugin System