# Library Structure

## Overview (Updated: 2025-01-28)

This document describes the backend architecture and plugin system of reStrike VTA, a Windows-native desktop application built with Tauri v2 and Rust.

## Current Status ✅

### **Window Management & Network Interface System - COMPLETE**
- **Window Positioning**: Fixed app startup position to screen coordinates x=1, y=1
- **Network Interface Detection**: Complete UDP/PSS network interface system with ⭐ Recommended status
- **Window Management Commands**: `set_window_position`, `set_window_startup_position` for consistent positioning
- **Network Interface Commands**: `get_best_network_interface` with optimal interface selection
- **Startup Behavior**: Consistent window positioning and sizing on every app launch

### **Real-Time Event System - COMPLETE**
- **Push-Based Events**: Implemented `window.emit` for real-time event streaming to frontend
- **PSS Event Listener**: `pss_setup_event_listener` command for real-time PSS event handling
- **OBS Status Listener**: `obs_setup_status_listener` for real-time OBS status updates
- **CPU Stats Listener**: `cpu_setup_stats_listener` for real-time system monitoring
- **Live Data Streaming**: Real-time log streaming with subsystem filtering
- **Event Emission**: Proper event emission using Tauri v2 event system

### **Window Management System - COMPLETE**
- **Dynamic Window Sizing**: `set_window_fullscreen`, `set_window_compact`, `set_window_custom_size`
- **Window Positioning**: `set_window_position`, `set_window_startup_position` for consistent positioning
- **Window Persistence**: `save_window_settings`, `load_window_settings` for cross-session persistence
- **Screen Size Detection**: `get_screen_size` for adaptive window sizing
- **Compact Mode**: Default 350x1080 dimensions with resizable option
- **Fullscreen Mode**: Custom dimensions with Advanced panel integration

### **Plugin Architecture - COMPLETE**
- **Modular Design**: All functionality organized into plugins
- **Clear Separation**: Each plugin handles specific domain
- **Error Handling**: Comprehensive AppResult<T> and AppError system
- **Configuration**: Centralized configuration management
- **Logging**: Structured logging with rotation and archival

### **Code Quality & Build Optimization - COMPLETE**
- **Clean Compilation**: Backend compiles without warnings or unused imports
- **Import Optimization**: Removed unused `Manager` import from `tauri_commands.rs`
- **Error Handling**: Consistent AppResult<T> usage across all plugins
- **Build Pipeline**: Clean cargo check and build process
- **Production Ready**: Backend ready for production deployment

### **Recent Major Updates (2025-01-28)**
- **Window Positioning**: Fixed app startup position to x=1, y=1 with consistent behavior
- **Network Interface Detection**: Complete UDP/PSS network interface system with optimal selection
- **Real-Time Events**: Implemented push-based event system using Tauri v2
- **Window Management**: Complete window sizing and persistence system with positioning
- **Code Cleanup**: Removed unused imports and optimized build process
- **Build Optimization**: Achieved clean compilation for backend
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
├── tauri_commands.rs       # Tauri command definitions (1835 lines)
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
    ├── logger.rs          # Logging utilities
    └── network.rs         # Network interface detection
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

## Tauri Commands (1835 lines)

### **Core App Commands**
- `get_app_status`: Get application status
- `shutdown_app`: Graceful application shutdown

### **UDP Commands**
- `start_udp_server`: Start UDP server for PSS events
- `stop_udp_server`: Stop UDP server
- `get_udp_status`: Get UDP server status

### **OBS Commands**
- `obs_connect`: Connect to OBS WebSocket
- `obs_add_connection`: Add OBS connection configuration
- `obs_connect_to_connection`: Connect to specific OBS instance
- `obs_get_connection_status`: Get connection status
- `obs_get_connections`: List all OBS connections
- `obs_disconnect`: Disconnect from OBS
- `obs_remove_connection`: Remove OBS connection
- `obs_get_status`: Get OBS recording/streaming status
- `obs_start_recording`: Start OBS recording
- `obs_stop_recording`: Stop OBS recording
- `obs_setup_status_listener`: Set up real-time OBS status monitoring

### **PSS Commands**
- `pss_start_listener`: Start PSS UDP listener
- `pss_stop_listener`: Stop PSS listener
- `pss_get_events`: Get PSS events
- `pss_emit_event`: Emit PSS event to frontend
- `pss_emit_pending_events`: Emit pending events
- `pss_setup_event_listener`: Set up real-time PSS event listener

### **Window Management Commands**
- `set_window_fullscreen`: Set window to fullscreen mode
- `set_window_compact`: Set window to compact mode (350x1080)
- `set_window_custom_size`: Set custom window dimensions
- `set_window_position`: Set window position to specific coordinates
- `set_window_startup_position`: Set window to startup position (x=1, y=1) with compact size
- `save_window_settings`: Save window settings to persistent storage
- `load_window_settings`: Load window settings from storage
- `get_screen_size`: Get screen dimensions

### **System Commands**
- `system_get_info`: Get system information
- `system_open_file_dialog`: Open file dialog
- `get_network_interfaces`: List network interfaces with detailed information
- `get_best_network_interface`: Get optimal network interface with ⭐ Recommended status

### **Logging Commands**
- `list_log_files`: List available log files
- `download_log_file`: Download specific log file
- `list_archives`: List log archives
- `extract_archive`: Extract log archive
- `download_archive`: Download log archive

### **Live Data Commands**
- `set_live_data_streaming`: Enable/disable live data streaming
- `start_live_data`: Start live data for subsystem
- `stop_live_data`: Stop live data for subsystem
- `get_live_data`: Get live data for subsystem

### **CPU Monitoring Commands**
- `cpu_get_process_data`: Get process CPU usage
- `cpu_get_system_data`: Get system CPU usage
- `cpu_get_obs_usage`: Get OBS CPU usage
- `cpu_update_config`: Update CPU monitoring configuration
- `cpu_test_plugin`: Test CPU monitoring plugin
- `cpu_enable_monitoring`: Enable CPU monitoring
- `cpu_disable_monitoring`: Disable CPU monitoring
- `cpu_get_monitoring_status`: Get monitoring status
- `cpu_setup_stats_listener`: Set up real-time CPU stats

### **Protocol Commands**
- `protocol_get_versions`: Get available protocol versions
- `protocol_set_active_version`: Set active protocol version
- `protocol_upload_file`: Upload protocol file
- `protocol_delete_version`: Delete protocol version
- `protocol_export_file`: Export protocol file
- `protocol_get_current`: Get current protocol

### **Configuration Commands**
- `get_settings`: Get application settings
- `update_settings`: Update application settings
- `get_config_stats`: Get configuration statistics
- `reset_settings`: Reset to default settings
- `export_settings`: Export settings to file
- `import_settings`: Import settings from file
- `restore_settings_backup`: Restore from backup

### **Video Commands**
- `video_play`: Play video file
- `video_stop`: Stop video playback
- `video_get_info`: Get video information
- `extract_clip`: Extract video clip

### **License Commands**
- `activate_license`: Activate license key
- `validate_license`: Validate license
- `get_license_status`: Get license status

### **Flag Commands**
- `get_flag_url`: Get flag URL for IOC code
- `download_flags`: Download flag images

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
  - Real-time status listener for frontend updates
- **Error Handling**: AppResult<T> with AppError variants
- **Configuration**: Persistent connection settings

### **UDP Plugin (`plugin_udp.rs`)**
- **Purpose**: UDP protocol handling for PSS events
- **Features**:
  - UDP listener for PSS protocol
  - Real-time event processing
  - Event filtering and validation
  - Connection status monitoring
  - Live data streaming to frontend
- **Integration**: Works with PSS plugin for event handling

### **PSS Plugin (`plugin_pss.rs`)**
- **Purpose**: PSS protocol v2.3 implementation
- **Features**:
  - PSS protocol parsing and validation
  - Event type detection and categorization
  - Country code mapping (PSS to IOC codes)
  - Real-time event streaming to frontend
  - Event storage and retrieval
  - Event listener setup for real-time updates
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
  - Real-time stats listener for frontend

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
    pub window_settings: WindowSettings,
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
- **Live Data Streaming**: Real-time log streaming to frontend
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
- **Window Management**: Dynamic window sizing and persistence with positioning

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
**Status**: Complete plugin architecture with real-time event system, window management, and network interface detection  
**Focus**: Maintainable, scalable backend architecture with comprehensive Tauri integration 