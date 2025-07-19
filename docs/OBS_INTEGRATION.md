# OBS Integration Guide

## 🎥 Overview

reStrike VTA provides comprehensive OBS Studio integration through WebSocket v5 protocol, enabling real-time control and monitoring of OBS instances for taekwondo competition management.

## 🔌 WebSocket Management

### Connection Management

The application provides a complete WebSocket connection management system with the following capabilities:

#### Connection Operations
- **Add Connection**: Create new OBS WebSocket connections
- **Edit Connection**: Modify existing connection settings
- **Delete Connection**: Remove connection configurations
- **Connect**: Establish WebSocket connection to OBS
- **Disconnect**: Close WebSocket connection (preserves configuration)
- **Status Monitoring**: Real-time connection status updates

#### Connection Configuration
```typescript
interface ObsConnection {
  name: string;           // Connection name (e.g., "OBS_REC", "OBS_STR")
  host: string;           // OBS host (default: "localhost")
  port: number;           // WebSocket port (default: 4455)
  password?: string;      // OBS WebSocket password (optional)
  enabled: boolean;       // Connection enabled/disabled
  status: 'Disconnected' | 'Connecting' | 'Connected' | 
          'Authenticating' | 'Authenticated' | 'Error';
  error?: string;         // Error message if connection failed
}
```

### WebSocket Manager Interface

#### Connection List
- **Status Indicators**: Visual status dots for each connection
- **Connection Details**: Host, port, password status
- **Action Buttons**: Connect, Disconnect, Edit, Delete
- **Active Connection**: Highlighted active connection

#### Add/Edit Form
- **Connection Name**: Unique identifier for the connection
- **Host**: OBS WebSocket host address
- **Port**: WebSocket port number
- **Password**: Optional authentication password
- **Enabled**: Enable/disable connection
- **Save Settings**: Save configuration without connecting

#### Global Reconnection Settings
- **Auto-reconnect**: Enable automatic reconnection on connection loss
- **Reconnection Delay**: Delay between reconnection attempts (seconds)
- **Maximum Attempts**: Maximum number of reconnection attempts
- **Status Monitoring**: Enable continuous connection status monitoring
- **Status Interval**: Interval for status checks (seconds)

## 🔐 Authentication

### OBS WebSocket v5 Authentication

The application supports OBS WebSocket v5 protocol with SHA256 authentication:

#### Authentication Flow
1. **Hello Message**: Receive OBS Studio version and WebSocket version
2. **Identify Message**: Send RPC version and authentication data
3. **Identified Message**: Receive authentication confirmation
4. **Connection Established**: WebSocket connection ready for commands

#### Password Handling
- **Secure Storage**: Passwords stored securely in configuration
- **Password Preservation**: Passwords preserved during connection updates
- **Optional Authentication**: Connections work without passwords
- **Password Masking**: Passwords displayed as dots in UI

## 📡 Real-time Communication

### Connection Status Monitoring

#### Status States
- **Disconnected**: No WebSocket connection
- **Connecting**: Attempting to establish connection
- **Connected**: WebSocket connection established
- **Authenticating**: Performing authentication
- **Authenticated**: Successfully authenticated with OBS
- **Error**: Connection error with error message

#### Status Updates
- **Real-time Updates**: Status changes reflected immediately
- **Event-driven**: Status updates triggered by WebSocket events
- **Error Handling**: Detailed error messages for troubleshooting
- **Visual Indicators**: Color-coded status dots

### Event Processing

#### OBS Events
- **Scene Changes**: Current program scene changes
- **Recording State**: Recording start/stop events
- **Streaming State**: Streaming start/stop events
- **Replay Buffer**: Replay buffer state changes
- **Error Events**: Connection and authentication errors

#### Event Handling
```rust
pub enum ObsEvent {
    ConnectionStatusChanged {
        connection_name: String,
        status: ObsConnectionStatus,
    },
    SceneChanged {
        connection_name: String,
        scene_name: String,
    },
    RecordingStateChanged {
        connection_name: String,
        is_recording: bool,
    },
    StreamStateChanged {
        connection_name: String,
        is_streaming: bool,
    },
    ReplayBufferStateChanged {
        connection_name: String,
        is_active: bool,
    },
    Error {
        connection_name: String,
        error: String,
    },
}
```

## 🎮 OBS Control Commands

### Scene Management
```rust
// Get current scene
pub async fn get_current_scene(&self, connection_name: &str) -> AppResult<String>

// Set current scene
pub async fn set_current_scene(&self, connection_name: &str, scene_name: &str) -> AppResult<()>

// Get all scenes
pub async fn get_scenes(&self, connection_name: &str) -> AppResult<Vec<String>>
```

### Recording Control
```rust
// Start recording
pub async fn start_recording(&self, connection_name: &str) -> AppResult<()>

// Stop recording
pub async fn stop_recording(&self, connection_name: &str) -> AppResult<()>

// Get recording status
pub async fn get_recording_status(&self, connection_name: &str) -> AppResult<bool>
```

### Streaming Control
```rust
// Start streaming
pub async fn start_streaming(&self, connection_name: &str) -> AppResult<()>

// Stop streaming
pub async fn stop_streaming(&self, connection_name: &str) -> AppResult<()>

// Get streaming status
pub async fn get_streaming_status(&self, connection_name: &str) -> AppResult<bool>
```

### Replay Buffer Control
```rust
// Start replay buffer
pub async fn start_replay_buffer(&self, connection_name: &str) -> AppResult<()>

// Stop replay buffer
pub async fn stop_replay_buffer(&self, connection_name: &str) -> AppResult<()>

// Save replay buffer
pub async fn save_replay_buffer(&self, connection_name: &str) -> AppResult<()>

// Get replay buffer status
pub async fn get_replay_buffer_status(&self, connection_name: &str) -> AppResult<bool>
```

### System Information
```rust
// Get OBS CPU usage
pub async fn get_obs_cpu_usage(&self, connection_name: &str) -> AppResult<f64>

// Get comprehensive OBS status
pub async fn get_obs_status(&self) -> AppResult<ObsStatusInfo>
```

## 🔧 Configuration Integration

### Settings Persistence

#### Configuration Storage
- **JSON Configuration**: All OBS settings stored in `config/app_config.json`
- **Cross-session Persistence**: Settings survive application restarts
- **Backup System**: Automatic backup of configuration files
- **Import/Export**: Full configuration backup and restore

#### Configuration Segments
```json
{
  "obs": {
    "connections": [
      {
        "name": "OBS_REC",
        "host": "localhost",
        "port": 4455,
        "password": "encrypted_password",
        "protocol_version": "v5",
        "enabled": true,
        "timeout_seconds": 30,
        "auto_reconnect": true,
        "max_reconnect_attempts": 5
      }
    ],
    "behavior": {
      "auto_reconnect": true,
      "reconnect_delay": 5,
      "max_attempts": 5,
      "status_monitoring": true,
      "status_interval": 30
    }
  }
}
```

### Synchronization

#### Frontend-Backend Sync
- **Configuration Loading**: Frontend loads settings from backend on startup
- **Real-time Updates**: Settings changes immediately reflected in UI
- **Status Synchronization**: Connection status synchronized between frontend and backend
- **Error Handling**: Configuration errors handled gracefully

## 🚨 Troubleshooting

### Common Issues

#### Connection Refused
- **Check OBS WebSocket Plugin**: Ensure WebSocket v5 plugin is installed and enabled
- **Verify Port**: Check that OBS WebSocket is listening on the correct port
- **Firewall Settings**: Ensure firewall allows connections to OBS WebSocket port
- **OBS Studio Version**: Ensure OBS Studio v28+ is installed

#### Authentication Failed
- **Check Password**: Verify WebSocket password in OBS settings
- **Password Encoding**: Ensure password is correctly encoded
- **OBS Settings**: Check OBS WebSocket plugin settings
- **Protocol Version**: Ensure using WebSocket v5 protocol

#### Connection Lost
- **Network Issues**: Check network connectivity
- **OBS Restart**: Restart OBS Studio if connection becomes unstable
- **Auto-reconnect**: Enable auto-reconnect in settings
- **Status Monitoring**: Enable status monitoring for automatic detection

### Debug Information

#### Backend Logging
```rust
// Enable debug logging
log::info!("[PLUGIN_OBS] Connection attempt: {}@{}:{}", name, host, port);
log::debug!("[AUTH-DEBUG] Authentication step: {}", step);
log::warn!("[PLUGIN_OBS] Connection warning: {}", warning);
log::error!("[PLUGIN_OBS] Connection error: {}", error);
```

#### Frontend Debugging
```typescript
// Enable connection debugging
console.log('Connection status:', connection.status);
console.log('Connection error:', connection.error);
console.log('WebSocket URL:', `ws://${connection.host}:${connection.port}`);
```

## 📊 Performance Optimization

### Connection Management
- **Connection Pooling**: Efficient management of multiple connections
- **Status Caching**: Cache connection status to reduce API calls
- **Event Batching**: Batch events to reduce processing overhead
- **Memory Management**: Efficient memory usage for WebSocket connections

### Monitoring
- **Performance Metrics**: Monitor connection performance and latency
- **Error Tracking**: Track and analyze connection errors
- **Status Monitoring**: Continuous monitoring of connection health
- **Resource Usage**: Monitor CPU and memory usage

## 🔮 Future Enhancements

### Planned Features
- **Multiple OBS Instances**: Support for multiple OBS instances on different machines
- **Advanced Authentication**: Support for additional authentication methods
- **Custom Commands**: Support for custom OBS WebSocket commands
- **Event Filtering**: Advanced event filtering and processing
- **Performance Analytics**: Detailed performance analytics and reporting

### Integration Opportunities
- **Streaming Platforms**: Integration with streaming platforms
- **Video Processing**: Advanced video processing capabilities
- **Automation**: Automated scene switching and recording
- **Analytics**: Competition analytics and reporting

---

**Last Updated**: 2025-01-28  
**OBS Integration Version**: 2.0  
**Protocol Support**: WebSocket v5  
**Status**: Production Ready 