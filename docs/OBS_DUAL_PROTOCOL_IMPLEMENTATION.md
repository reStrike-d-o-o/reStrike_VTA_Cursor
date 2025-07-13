# OBS WebSocket Dual-Protocol Implementation

## Overview

reStrike VTA now supports both OBS WebSocket v4 and v5 protocols simultaneously, allowing you to connect to multiple OBS instances running different protocol versions. This implementation provides a unified interface that automatically handles protocol differences.

## Architecture

### Backend (Rust/Tauri)

#### Core Components

1. **`src/plugin_obs.rs`** - Main OBS WebSocket plugin
   - `ObsPlugin` - Manages multiple OBS connections
   - `ObsConnectionConfig` - Configuration for each connection
   - `ObsWebSocketVersion` - Protocol version enum (V4/V5)
   - Protocol-agnostic API methods

2. **`src/tauri_commands.rs`** - Tauri command handlers
   - Bridges frontend with backend OBS plugin
   - Handles all OBS operations (connect, disconnect, scene control, etc.)
   - Provides unified response format

3. **`src/main.rs`** - Application entry point
   - Initializes OBS plugin
   - Registers Tauri commands
   - Manages plugin lifecycle

#### Key Features

- **Multiple Connections**: Support for unlimited OBS instances
- **Protocol Detection**: Automatic protocol version handling
- **Event System**: Real-time status updates and events
- **Error Handling**: Comprehensive error management
- **Thread Safety**: Arc<Mutex> for concurrent access

### Frontend (React/TypeScript)

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

## Protocol Differences Handled

### OBS WebSocket v4
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

### OBS WebSocket v5
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

## Implementation Details

### Connection Management

```rust
// Add a new OBS connection
let config = ObsConnectionConfig {
    name: "Main OBS".to_string(),
    host: "localhost".to_string(),
    port: 4455,
    password: Some("password123".to_string()),
    protocol_version: ObsWebSocketVersion::V5,
    enabled: true,
};

obs_plugin.add_connection(config).await?;
```

### Protocol-Agnostic Operations

```rust
// Get current scene (works with both v4 and v5)
let scene = obs_plugin.get_current_scene("Main OBS").await?;

// Set current scene (works with both v4 and v5)
obs_plugin.set_current_scene("Main OBS", "Replay Scene").await?;

// Start recording (works with both v4 and v5)
obs_plugin.start_recording("Main OBS").await?;
```

### Event Handling

```rust
// Handle events from both protocol versions
while let Some(event) = event_rx.recv().await {
    match event {
        ObsEvent::ConnectionStatusChanged { connection_name, status } => {
            println!("{}: {:?}", connection_name, status);
        }
        ObsEvent::SceneChanged { connection_name, scene_name } => {
            println!("{} switched to scene: {}", connection_name, scene_name);
        }
        ObsEvent::RecordingStateChanged { connection_name, is_recording } => {
            println!("{} recording: {}", connection_name, is_recording);
        }
        ObsEvent::Error { connection_name, error } => {
            eprintln!("{} error: {}", connection_name, error);
        }
    }
}
```

## Configuration

### Connection Configuration

```json
{
  "obs_connections": [
    {
      "name": "Main OBS",
      "host": "localhost",
      "port": 4455,
      "password": "your_password_here",
      "protocol_version": "v5",
      "enabled": true
    },
    {
      "name": "Backup OBS",
      "host": "192.168.1.100",
      "port": 4444,
      "password": "backup_password",
      "protocol_version": "v4",
      "enabled": true
    }
  ]
}
```

### Environment Variables

```bash
# OBS WebSocket v5 default port
OBS_WS_V5_PORT=4455

# OBS WebSocket v4 default port
OBS_WS_V4_PORT=4444

# Default connection timeout (seconds)
OBS_CONNECTION_TIMEOUT=30

# Reconnection attempts
OBS_MAX_RECONNECT_ATTEMPTS=5
```

## API Reference

### Backend API

#### Connection Management
- `add_connection(config)` - Add new OBS connection
- `remove_connection(name)` - Remove OBS connection
- `connect_obs(name)` - Connect to OBS instance
- `get_connection_status(name)` - Get connection status
- `get_connection_names()` - List all connections

#### Scene Operations
- `get_current_scene(name)` - Get current scene
- `set_current_scene(name, scene)` - Set current scene
- `get_scenes(name)` - Get all scenes

#### Recording Operations
- `start_recording(name)` - Start recording
- `stop_recording(name)` - Stop recording
- `get_recording_status(name)` - Get recording status

#### Replay Buffer Operations
- `start_replay_buffer(name)` - Start replay buffer
- `stop_replay_buffer(name)` - Stop replay buffer
- `save_replay_buffer(name)` - Save replay buffer
- `get_replay_buffer_status(name)` - Get replay buffer status

### Frontend API

#### Tauri Commands
```typescript
// Add connection
await invoke('obs_add_connection', {
  name: 'Main OBS',
  host: 'localhost',
  port: 4455,
  password: 'password123',
  protocol_version: 'v5',
  enabled: true
});

// Get current scene
const response = await invoke('obs_get_current_scene', {
  connection_name: 'Main OBS'
});

// Start recording
await invoke('obs_start_recording', {
  connection_name: 'Main OBS'
});
```

## Usage Examples

### Tournament Setup

```typescript
// Configure main OBS (v5)
await invoke('obs_add_connection', {
  name: 'Main OBS',
  host: '192.168.1.10',
  port: 4455,
  password: 'main_password',
  protocol_version: 'v5',
  enabled: true
});

// Configure backup OBS (v4)
await invoke('obs_add_connection', {
  name: 'Backup OBS',
  host: '192.168.1.11',
  port: 4444,
  password: 'backup_password',
  protocol_version: 'v4',
  enabled: true
});

// Configure replay OBS (v5)
await invoke('obs_add_connection', {
  name: 'Replay OBS',
  host: '192.168.1.12',
  port: 4456,
  password: 'replay_password',
  protocol_version: 'v5',
  enabled: false
});
```

### Scene Management

```typescript
// Switch to replay scene on main OBS
await invoke('obs_set_current_scene', {
  connection_name: 'Main OBS',
  scene_name: 'Replay Scene'
});

// Get current scene from backup OBS
const response = await invoke('obs_get_current_scene', {
  connection_name: 'Backup OBS'
});
console.log('Current scene:', response.data.scene_name);
```

### Recording Control

```typescript
// Start recording on main OBS
await invoke('obs_start_recording', {
  connection_name: 'Main OBS'
});

// Check recording status on backup OBS
const status = await invoke('obs_get_recording_status', {
  connection_name: 'Backup OBS'
});
console.log('Recording:', status.data.is_recording);
```

### Replay Buffer Operations

```typescript
// Start replay buffer
await invoke('obs_start_replay_buffer', {
  connection_name: 'Main OBS'
});

// Save replay buffer
await invoke('obs_save_replay_buffer', {
  connection_name: 'Main OBS'
});

// Stop replay buffer
await invoke('obs_stop_replay_buffer', {
  connection_name: 'Main OBS'
});
```

## Error Handling

### Common Errors

1. **Connection Refused**
   - Cause: OBS WebSocket server not running
   - Solution: Check OBS WebSocket settings

2. **Authentication Failed**
   - Cause: Wrong password or authentication disabled
   - Solution: Verify password in OBS settings

3. **Protocol Mismatch**
   - Cause: Client using wrong protocol version
   - Solution: Check OBS WebSocket plugin version

4. **Connection Timeout**
   - Cause: Network issues or OBS not responding
   - Solution: Check network connectivity

### Error Response Format

```json
{
  "success": false,
  "data": null,
  "error": "Connection refused: OBS WebSocket server not running"
}
```

## Performance Considerations

### Connection Limits
- **Recommended**: 2-3 OBS instances per application
- **Maximum**: 10 concurrent connections
- **Memory Usage**: ~2MB per connection

### Network Optimization
- Use local network for OBS instances
- Configure firewall to allow WebSocket ports
- Monitor connection health regularly

### Event Handling
- Subscribe only to required events
- Implement event filtering for high-frequency events
- Use debouncing for rapid state changes

## Security

### Authentication
- Always use strong passwords
- Enable authentication in production
- Use unique passwords per connection

### Network Security
- Restrict WebSocket access to local network
- Configure firewall rules
- Use VPN for remote connections

### Data Protection
- Encrypt sensitive configuration data
- Implement connection logging
- Regular security audits

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_connection() {
        let (event_tx, _) = mpsc::unbounded_channel();
        let plugin = ObsPlugin::new(event_tx);
        
        let config = ObsConnectionConfig {
            name: "test".to_string(),
            host: "localhost".to_string(),
            port: 4455,
            password: None,
            protocol_version: ObsWebSocketVersion::V5,
            enabled: false,
        };
        
        assert!(plugin.add_connection(config).await.is_ok());
    }
}
```

### Integration Tests
- Test with real OBS instances
- Verify protocol compatibility
- Test error scenarios
- Performance testing

## Deployment

### Production Setup
1. Configure OBS WebSocket plugins
2. Set up network connectivity
3. Configure authentication
4. Test all connections
5. Monitor performance

### Monitoring
- Connection health checks
- Error rate monitoring
- Performance metrics
- Log analysis

## Future Enhancements

### Planned Features
- WebSocket over SSL/TLS
- Connection pooling
- Automatic failover
- Advanced event filtering
- Plugin marketplace integration

### Protocol Support
- OBS WebSocket v6 (when available)
- Custom protocol extensions
- Third-party plugin support

## Troubleshooting

### Debug Mode
Enable debug logging to see protocol-specific details:

```rust
// Enable debug logging
env_logger::init();
log::set_max_level(log::LevelFilter::Debug);
```

### Common Issues

1. **Port Already in Use**
   ```bash
   # Check what's using the port
   netstat -tulpn | grep :4455
   
   # Kill the process
   sudo kill -9 <PID>
   ```

2. **Authentication Issues**
   - Verify password in OBS WebSocket settings
   - Check if authentication is enabled
   - Try connecting without password first

3. **Protocol Version Mismatch**
   - Check OBS WebSocket plugin version
   - Verify protocol version in configuration
   - Update OBS WebSocket plugin if needed

## Support

### Documentation
- [OBS WebSocket v4 Documentation](https://github.com/obsproject/obs-websocket/blob/4.x-current/docs/generated/protocol.md)
- [OBS WebSocket v5 Documentation](https://github.com/obsproject/obs-websocket/blob/master/docs/generated/protocol.md)

### Community
- OBS Project Discord
- GitHub Issues
- Stack Overflow

### Professional Support
- Custom implementations
- Training and consulting
- Enterprise deployments 