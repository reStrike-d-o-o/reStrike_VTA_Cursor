# Library Structure

## Overview (Updated: 2025-01-28)

This document describes the backend architecture and plugin system of reStrike VTA, a Windows-native desktop application built with Tauri v2 and Rust.

## Current Status ✅

### **Plugin Architecture - COMPLETE**
- **Modular Design**: All functionality organized into plugins
- **Clear Separation**: Each plugin handles specific domain
- **Error Handling**: Comprehensive AppResult<T> and AppError system
- **Configuration**: Centralized configuration management
- **Logging**: Structured logging with rotation and archival

### **Recent Major Updates (2025-01-28)**
- **Tab System Integration**: Frontend tab system working with backend plugins
- **Flag Management**: Complete flag management system with 253+ IOC flags
- **PSS Code Mapping**: Simplified mapping where PSS codes = IOC codes
- **UI Integration**: All frontend components properly integrated with backend
- **Error Handling**: Consistent error handling across all plugins

## Backend Architecture

### **Core Application Layer**
```
src-tauri/src/
├── main.rs                 # Tauri app entry point
├── lib.rs                  # Library exports and plugin registration
├── tauri_commands.rs       # Tauri command definitions
├── core/                   # Core application functionality
│   ├── app.rs             # Application state and lifecycle
│   ├── config.rs          # Configuration management
│   └── state.rs           # Global state management
├── config/                 # Configuration system
│   ├── manager.rs         # Configuration manager
│   ├── types.rs           # Configuration types
│   └── mod.rs             # Configuration module
├── logging/                # Logging system
│   ├── logger.rs          # Logging implementation
│   ├── rotation.rs        # Log rotation
│   ├── archival.rs        # Log archival
│   └── mod.rs             # Logging module
├── types/                  # Shared types
│   └── mod.rs             # Type definitions
└── utils/                  # Utility functions
    └── logger.rs          # Logging utilities
```

### **Plugin System**
```
src-tauri/src/plugins/
├── mod.rs                 # Plugin module registration
├── plugin_obs.rs          # OBS WebSocket integration
├── plugin_udp.rs          # UDP protocol handling
├── plugin_pss.rs          # PSS protocol implementation
├── plugin_playback.rs     # Video playback management
├── plugin_store.rs        # Data storage and persistence
├── plugin_cpu_monitor.rs  # System monitoring
└── plugin_license.rs      # License management
```

### **Protocol Implementations**
```
src-tauri/src/
├── obs/                   # OBS WebSocket integration
│   ├── manager.rs         # OBS connection manager
│   ├── protocol.rs        # WebSocket protocol handling
│   └── commands.rs        # OBS command definitions
├── pss/                   # PSS protocol implementation
│   ├── listener.rs        # UDP listener
│   ├── protocol.rs        # PSS protocol parsing
│   └── events.rs          # Event handling
└── video/                 # Video management
    ├── player.rs          # Video player integration
    ├── clips.rs           # Clip management
    └── overlay.rs         # Video overlay system
```

## Plugin Details

### **OBS Plugin (`plugin_obs.rs`)**
- **Purpose**: OBS Studio WebSocket integration
- **Protocol**: WebSocket v5 only (v4 support removed)
- **Features**:
  - Connection management (add, edit, delete, connect, disconnect)
  - Settings persistence across sessions
  - Real-time status monitoring
  - Secure password handling
  - Multiple OBS instance support
- **Error Handling**: AppResult<T> with AppError variants
- **Configuration**: Persistent connection settings

### **UDP Plugin (`plugin_udp.rs`)**
- **Purpose**: UDP protocol handling for PSS events
- **Features**:
  - UDP listener for PSS protocol
  - Real-time event processing
  - Event filtering and validation
  - Connection status monitoring
- **Integration**: Works with PSS plugin for event handling

### **PSS Plugin (`plugin_pss.rs`)**
- **Purpose**: PSS protocol v2.3 implementation
- **Features**:
  - PSS protocol parsing and validation
  - Event type detection and categorization
  - Country code mapping (PSS to IOC codes)
  - Real-time event streaming
  - Event storage and retrieval
- **Flag Integration**: Uses IOC flag codes for country identification

### **Playback Plugin (`plugin_playback.rs`)**
- **Purpose**: Video playback and clip management
- **Features**:
  - MPV video player integration
  - Clip extraction from OBS
  - Video metadata management
  - Replay buffer integration
  - Clip organization and storage

### **Store Plugin (`plugin_store.rs`)**
- **Purpose**: Data persistence and storage
- **Features**:
  - SQLite database management
  - Event data storage
  - Configuration persistence
  - Data export capabilities
  - Backup and restore functionality

### **CPU Monitor Plugin (`plugin_cpu_monitor.rs`)**
- **Purpose**: System resource monitoring
- **Features**:
  - CPU usage monitoring
  - Memory usage tracking
  - System performance metrics
  - Real-time status reporting
  - Resource alerting

### **License Plugin (`plugin_license.rs`)**
- **Purpose**: License management and validation
- **Features**:
  - License key validation
  - Feature access control
  - License status monitoring
  - Expiration handling

## Error Handling System

### **AppResult<T> Type**
```rust
pub type AppResult<T> = Result<T, AppError>;
```

### **AppError Variants**
```rust
pub enum AppError {
    IoError(std::io::Error),
    ConfigError(String),
    ObsError(String),
    UdpError(String),
    PssError(String),
    PlaybackError(String),
    StoreError(String),
    ValidationError(String),
    NetworkError(String),
}
```

### **Error Handling Guidelines**
- **Always use AppResult<T>**: For all plugin and core methods
- **Convert std::io::Error**: Use `AppError::IoError(e)` for actual IO errors
- **Use ConfigError**: For custom error messages or formatted strings
- **Never use IoError with String**: Only for actual std::io::Error values
- **Return e.to_string()**: When converting AppError to String for API responses

## Configuration Management

### **Configuration Structure**
```rust
pub struct AppConfig {
    pub obs_connections: Vec<ObsConnection>,
    pub udp_settings: UdpSettings,
    pub pss_settings: PssSettings,
    pub playback_settings: PlaybackSettings,
    pub logging_settings: LoggingSettings,
}
```

### **Configuration Features**
- **Persistent Storage**: All settings survive app restarts
- **Cross-Session Sync**: Frontend and backend stay synchronized
- **Backup/Restore**: Automatic backup with manual restore
- **Import/Export**: Full configuration backup and restore
- **Validation**: Configuration validation and error handling

## Logging System

### **Logging Features**
- **Multi-subsystem Logging**: Separate loggers for each plugin
- **File Rotation**: Automatic log file rotation
- **Archive Management**: Log archiving and compression
- **Live Data Streaming**: Real-time log streaming
- **Diagnostic Tools**: Built-in diagnostic utilities

### **Log Levels**
- **Error**: Critical errors that affect functionality
- **Warn**: Warning conditions that may need attention
- **Info**: General information about application state
- **Debug**: Detailed debugging information
- **Trace**: Very detailed tracing information

## Frontend Integration

### **Tauri Commands**
```rust
// Example Tauri command
#[tauri::command]
pub async fn get_obs_connections() -> AppResult<Vec<ObsConnection>> {
    // Implementation
}
```

### **State Management**
- **Global State**: Shared state across plugins
- **Configuration Sync**: Real-time configuration updates
- **Event Streaming**: Real-time event updates to frontend
- **Status Monitoring**: Live status updates

### **UI Integration Points**
- **OBS Manager**: WebSocket connection management
- **Event Table**: Real-time PSS event display
- **Flag Management**: IOC flag integration with PSS codes
- **Advanced Panel**: System diagnostics and configuration
- **Status Indicators**: Real-time system status

## Development Guidelines

### **Plugin Development**
1. **Use AppResult<T>**: For all return types
2. **Implement Error Handling**: Comprehensive error management
3. **Follow Naming Conventions**: Consistent naming across plugins
4. **Add Documentation**: Inline documentation for all public methods
5. **Test Integration**: Ensure proper frontend integration

### **Error Handling Best Practices**
- **Propagate Errors**: Use `?` operator for error propagation
- **Convert Errors**: Use appropriate AppError variants
- **Provide Context**: Include meaningful error messages
- **Handle Gracefully**: Graceful degradation when possible

### **Configuration Management**
- **Validate Input**: Always validate configuration data
- **Provide Defaults**: Sensible defaults for all settings
- **Handle Migration**: Support configuration format changes
- **Backup Before Changes**: Automatic backup before modifications

## Testing and Validation

### **Unit Testing**
- **Plugin Tests**: Individual plugin functionality
- **Integration Tests**: Plugin interaction testing
- **Error Handling**: Comprehensive error scenario testing
- **Configuration Tests**: Configuration validation testing

### **Integration Testing**
- **Frontend Integration**: UI component integration
- **Protocol Testing**: PSS and OBS protocol testing
- **Performance Testing**: System performance validation
- **Error Recovery**: Error recovery and resilience testing

## Performance Considerations

### **Optimization Strategies**
- **Async Operations**: Use async/await for I/O operations
- **Memory Management**: Efficient memory usage patterns
- **Resource Pooling**: Reuse expensive resources
- **Caching**: Intelligent caching strategies

### **Monitoring**
- **CPU Usage**: Real-time CPU monitoring
- **Memory Usage**: Memory consumption tracking
- **Network Performance**: Network operation monitoring
- **Error Rates**: Error frequency monitoring

## Security Considerations

### **Data Protection**
- **Secure Storage**: Encrypted configuration storage
- **Network Security**: Secure WebSocket connections
- **Input Validation**: Comprehensive input validation
- **Error Information**: Limited error information exposure

### **Access Control**
- **Feature Flags**: License-based feature access
- **Permission System**: Granular permission control
- **Audit Logging**: Security event logging
- **Secure Communication**: Encrypted communication channels

## Future Enhancements

### **Planned Features**
- **Plugin Extensions**: Extensible plugin architecture
- **Advanced Analytics**: Statistical analysis and reporting
- **AI Integration**: Automated event analysis
- **Multi-language Support**: Internationalization

### **Architecture Improvements**
- **Microservices**: Potential microservice architecture
- **Distributed Processing**: Distributed event processing
- **Advanced Caching**: Intelligent caching strategies
- **Performance Optimization**: Advanced performance tuning

---

**Last Updated**: 2025-01-28  
**Status**: Complete plugin architecture with comprehensive error handling  
**Focus**: Maintainable, scalable backend architecture 