# Library Structure Documentation

## Overview (Updated: 2025-01-28)

The reStrike VTA backend library provides a comprehensive Rust-based architecture for Windows desktop application development. The library features a modular plugin system, custom logging infrastructure, and robust error handling patterns.

## 🏗️ **Core Architecture**

### **Application Structure**
```rust
// Main application entry point
src-tauri/src/main.rs
├── Application initialization
├── Tauri command registration
├── Plugin system startup
└── Logging system initialization

// Core application logic
src-tauri/src/core/
├── app.rs           // Main application class
├── config.rs        // Configuration management
└── state.rs         // Application state management
```

### **Plugin System**
```rust
// Plugin modules
src-tauri/src/plugins/
├── mod.rs              // Plugin module exports
├── plugin_obs.rs       // OBS WebSocket integration
├── plugin_cpu_monitor.rs // CPU monitoring system
├── plugin_udp.rs       // UDP server implementation
├── plugin_playback.rs  // Video playback management
├── plugin_store.rs     // Data storage and persistence
└── plugin_license.rs   // License management
```

## 🔌 **Plugin Implementations**

### **OBS Plugin** (`plugin_obs.rs`)
```rust
pub struct ObsPlugin {
    connections: Arc<Mutex<HashMap<String, ObsConnection>>>,
    event_tx: mpsc::UnboundedSender<ObsEvent>,
    debug_ws_messages: Arc<Mutex<bool>>,
    show_full_events: Arc<Mutex<bool>>,
    recent_events: Arc<Mutex<Vec<RecentEvent>>>,
    log_manager: Arc<Mutex<LogManager>>,  // Custom logging integration
}

impl ObsPlugin {
    // Custom logging method
    async fn log_to_file(&self, level: &str, message: &str) {
        let log_manager = self.log_manager.lock().await;
        if let Err(e) = log_manager.log("obs", level, message) {
            eprintln!("Failed to log to obs.log: {}", e);
        }
    }
}
```

**Features**:
- ✅ **Real-time WebSocket communication** with OBS Studio
- ✅ **Custom LogManager integration** for event logging
- ✅ **Scene management** and recording control
- ✅ **Event streaming** to frontend
- ✅ **Connection status monitoring**

### **CPU Monitor Plugin** (`plugin_cpu_monitor.rs`)
```rust
pub struct CpuMonitorPlugin {
    config: CpuMonitorConfig,
    process_data: Arc<Mutex<Vec<CpuProcessData>>>,
    system_data: Arc<Mutex<SystemCpuData>>,
    is_monitoring: Arc<AtomicBool>,
}

pub struct CpuProcessData {
    pub process_name: String,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}
```

**Features**:
- ✅ **Windows `wmic` command integration**
- ✅ **Real-time process monitoring**
- ✅ **System CPU usage tracking**
- ✅ **Background task management**
- ⏳ **Awaiting `wmic` installation for testing**

### **UDP Plugin** (`plugin_udp.rs`)
```rust
pub struct UdpPlugin {
    config: UdpServerConfig,
    server: Option<UdpServer>,
    stats: Arc<Mutex<UdpStats>>,
}
```

**Features**:
- ✅ **UDP server implementation**
- ✅ **PSS protocol parsing**
- ✅ **Real-time packet processing**
- ✅ **Statistics tracking**

## 📝 **Logging System**

### **Custom LogManager** (`logging/mod.rs`)
```rust
pub struct LogManager {
    config: Arc<Mutex<LogConfig>>,
    loggers: Arc<Mutex<HashMap<String, Logger>>>,
    rotator: LogRotator,
    archiver: LogArchiver,
}

impl LogManager {
    pub fn log(&self, subsystem: &str, level: &str, message: &str) -> io::Result<()> {
        // All subsystems are always enabled
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let entry = LogEntry {
            timestamp,
            level: level.to_string(),
            subsystem: subsystem.to_string(),
            message: message.to_string(),
        };
        
        // Write to subsystem-specific log file
        let mut loggers = self.loggers.lock().unwrap();
        let logger = loggers.entry(subsystem.to_string()).or_insert_with(|| {
            Logger::new(&config.log_dir, subsystem).unwrap_or_else(|e| {
                log::error!("Failed to create logger for subsystem {}: {}", subsystem, e);
                Logger::new("log", "fallback").unwrap()
            })
        });
        
        logger.write_entry(&entry)?;
        Ok(())
    }
}
```

### **Logging Components**
- **Logger** (`logging/logger.rs`): Individual subsystem loggers
- **Rotation** (`logging/rotation.rs`): Log file rotation logic
- **Archival** (`logging/archival.rs`): Log compression and archival

### **Log Files Structure**
```
src-tauri/logs/
├── app.log              # Application-level events
├── obs.log              # OBS WebSocket events (REAL-TIME)
├── pss.log              # PSS protocol events
├── udp.log              # UDP server events
└── archives/            # Compressed log archives
    ├── obs_20250128_archive.zip
    ├── pss_20250128_archive.zip
    └── udp_20250128_archive.zip
```

## 🔧 **Error Handling Patterns**

### **AppResult<T> Pattern**
```rust
pub type AppResult<T> = Result<T, AppError>;

pub enum AppError {
    IoError(std::io::Error),
    ConfigError(String),
    PluginError(String),
    NetworkError(String),
    ValidationError(String),
}
```

### **Error Conversion Patterns**
```rust
// Converting std::io::Error to AppError
.map_err(|e| AppError::IoError(e))

// Converting to AppError::ConfigError for custom messages
.map_err(|e| AppError::ConfigError(e.to_string()))

// Using AppResult<T> for all plugin methods
pub async fn connect_obs(&self, connection_name: &str) -> AppResult<()> {
    // Implementation with proper error handling
}
```

## 📡 **Tauri Commands**

### **Command Structure**
```rust
#[tauri::command]
pub async fn list_log_files(
    subsystem: Option<String>,
    app: State<'_, Arc<App>>,
) -> Result<serde_json::Value, String> {
    let log_manager = app.log_manager().lock().await;
    match log_manager.list_log_files(subsystem.as_deref()) {
        Ok(files) => Ok(serde_json::json!({
            "success": true,
            "data": files
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": format!("Failed to list log files: {}", e)
        }))
    }
}
```

### **Command Categories**
- **Logging Commands**: Log file management and archival
- **OBS Commands**: WebSocket connection and control
- **CPU Commands**: System and process monitoring
- **UDP Commands**: Server management and statistics
- **System Commands**: General system information

## 🔄 **Data Flow Patterns**

### **OBS Event Flow**
```
OBS Studio → WebSocket → ObsPlugin → LogManager → obs.log
                ↓
            Frontend UI ← Tauri Commands ← Event Channel
```

### **CPU Monitoring Flow**
```
Windows System → wmic Commands → CpuMonitorPlugin → Tauri Commands → Frontend UI
```

### **Logging Flow**
```
Any Plugin → LogManager → Subsystem Logger → Log File → Rotation/Archival
```

## 🛠️ **Development Patterns**

### **Plugin Development Pattern**
```rust
// 1. Define plugin struct with necessary fields
pub struct MyPlugin {
    config: MyConfig,
    state: Arc<Mutex<MyState>>,
    log_manager: Arc<Mutex<LogManager>>,  // For logging
}

// 2. Implement constructor
impl MyPlugin {
    pub fn new(config: MyConfig, log_manager: Arc<Mutex<LogManager>>) -> Self {
        // Initialize plugin
    }
}

// 3. Implement methods with AppResult<T>
impl MyPlugin {
    pub async fn do_something(&self) -> AppResult<()> {
        // Use custom logging
        let log_manager = self.log_manager.lock().await;
        log_manager.log("my_plugin", "INFO", "Doing something")?;
        
        // Implementation
        Ok(())
    }
}
```

### **Async Mutex Pattern**
```rust
// Proper async mutex handling
let log_manager = self.log_manager.lock().await;
if let Err(e) = log_manager.log("subsystem", "level", "message") {
    eprintln!("Logging error: {}", e);
}
```

## 📊 **Current Status**

### **✅ Completed Features**
- **OBS Integration**: Complete WebSocket integration with custom logging
- **CPU Monitoring**: Real-time system monitoring implementation
- **Logging System**: Comprehensive subsystem-based logging
- **Plugin Architecture**: Modular, extensible plugin system
- **Error Handling**: Robust AppResult<T> pattern implementation

### **🚧 In Progress**
- **WMIC Integration**: Awaiting `wmic` command installation
- **Performance Optimization**: Ongoing optimization efforts
- **Error Handling Enhancement**: Improved error boundaries

### **📋 Next Steps**
1. **Complete CPU Monitoring**: Install `wmic` and test real data
2. **Performance Testing**: Optimize data flow and memory usage
3. **Documentation**: Update all documentation with latest patterns
4. **Testing**: Comprehensive unit and integration testing

## 🔍 **Troubleshooting**

### **Common Issues**
- **Compilation Errors**: Check type mismatches and imports
- **Runtime Errors**: Verify Tauri command registration
- **Logging Issues**: Check file permissions and LogManager initialization
- **Performance Issues**: Monitor memory usage and async patterns

### **Development Tips**
- **Use AppResult<T>**: Always use AppResult<T> for plugin methods
- **Proper Logging**: Use custom LogManager for structured logging
- **Async Patterns**: Use proper async mutex handling
- **Error Handling**: Convert errors appropriately (IoError vs ConfigError)

---

**Last Updated**: 2025-01-28  
**Status**: OBS logging integration complete, CPU monitoring awaiting `wmic` installation  
**Next Action**: Install `wmic` and test real process data display 