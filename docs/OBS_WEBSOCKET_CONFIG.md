# OBS WebSocket Configuration Guide - Dual Protocol Support

## Overview

reStrike VTA supports both OBS WebSocket v4 and v5 protocols simultaneously, allowing you to connect to multiple OBS instances running different protocol versions. This is particularly useful in tournament environments where you might have:

- **Main OBS Instance**: Running OBS WebSocket v5 (latest)
- **Backup OBS Instance**: Running OBS WebSocket v4 (legacy)
- **Replay OBS Instance**: Running either version

## Protocol Differences

### OBS WebSocket v4
- **Authentication**: Simple password-based
- **Message Format**: JSON with `request-type` field
- **Response Format**: Direct JSON responses
- **Field Names**: kebab-case (e.g., `scene-name`, `is-recording`)
- **Compatibility**: OBS Studio 26+ with obs-websocket v4 plugin

### OBS WebSocket v5
- **Authentication**: SHA256 challenge-response
- **Message Format**: JSON with `op` (opcode) and `d` (data) structure
- **Response Format**: Structured with `requestStatus` and `responseData`
- **Field Names**: camelCase (e.g., `sceneName`, `outputActive`)
- **Compatibility**: OBS Studio 28+ with obs-websocket v5 plugin

## Configuration

### Connection Configuration Structure

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
    },
    {
      "name": "Replay OBS",
      "host": "localhost",
      "port": 4456,
      "password": null,
      "protocol_version": "v5",
      "enabled": false
    }
  ]
}
```

### Configuration Fields

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `name` | String | Unique identifier for the connection | Yes |
| `host` | String | OBS WebSocket server hostname/IP | Yes |
| `port` | Number | OBS WebSocket server port | Yes |
| `password` | String/null | Authentication password (null if disabled) | No |
| `protocol_version` | String | "v4" or "v5" | Yes |
| `enabled` | Boolean | Whether to connect automatically | Yes |

## OBS Studio Setup

### Installing OBS WebSocket Plugins

#### For OBS WebSocket v4:
1. Download obs-websocket v4 from: https://github.com/obsproject/obs-websocket/releases
2. Install the plugin in OBS Studio
3. Configure in Tools → WebSocket Server Settings
4. Set port (default: 4444)
5. Enable/disable authentication

#### For OBS WebSocket v5:
1. Download obs-websocket v5 from: https://github.com/obsproject/obs-websocket/releases
2. Install the plugin in OBS Studio
3. Configure in Tools → WebSocket Server Settings
4. Set port (default: 4455)
5. Enable/disable authentication

### Recommended Port Configuration

| Instance | Protocol | Default Port | Recommended Port |
|----------|----------|--------------|------------------|
| Main OBS | v5 | 4455 | 4455 |
| Backup OBS | v4 | 4444 | 4444 |
| Replay OBS | v5 | 4455 | 4456 |

## Usage Examples

### Adding Connections Programmatically

```rust
use crate::plugin_obs::{ObsPlugin, ObsConnectionConfig, ObsWebSocketVersion};

// Create OBS plugin instance
let (event_tx, event_rx) = mpsc::unbounded_channel();
let obs_plugin = ObsPlugin::new(event_tx);

// Add v5 connection
let v5_config = ObsConnectionConfig {
    name: "Main OBS".to_string(),
    host: "localhost".to_string(),
    port: 4455,
    password: Some("password123".to_string()),
    protocol_version: ObsWebSocketVersion::V5,
    enabled: true,
};
obs_plugin.add_connection(v5_config).await?;

// Add v4 connection
let v4_config = ObsConnectionConfig {
    name: "Backup OBS".to_string(),
    host: "192.168.1.100".to_string(),
    port: 4444,
    password: Some("backup_pass".to_string()),
    protocol_version: ObsWebSocketVersion::V4,
    enabled: true,
};
obs_plugin.add_connection(v4_config).await?;
```

### Protocol-Agnostic Operations

The plugin automatically handles protocol differences:

```rust
// Get current scene (works with both v4 and v5)
let scene = obs_plugin.get_current_scene("Main OBS").await?;

// Set current scene (works with both v4 and v5)
obs_plugin.set_current_scene("Main OBS", "Replay Scene").await?;

// Start recording (works with both v4 and v5)
obs_plugin.start_recording("Main OBS").await?;

// Get recording status (works with both v4 and v5)
let is_recording = obs_plugin.get_recording_status("Main OBS").await?;
```

## Event Handling

The plugin emits events for both protocol versions:

```rust
// Handle events from both v4 and v5 connections
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

## Troubleshooting

### Common Issues

#### Connection Refused
- **Cause**: OBS WebSocket server not running or wrong port
- **Solution**: Check OBS WebSocket settings and ensure server is enabled

#### Authentication Failed
- **Cause**: Wrong password or authentication disabled
- **Solution**: Verify password in OBS WebSocket settings

#### Protocol Mismatch
- **Cause**: Client trying to use v5 protocol with v4 server
- **Solution**: Ensure protocol version matches OBS WebSocket plugin version

#### Multiple Connections
- **Cause**: Trying to connect to same OBS instance multiple times
- **Solution**: Use unique connection names and ensure only one connection per OBS instance

### Debug Information

Enable debug logging to see protocol-specific details:

```rust
// Check connection status
let status = obs_plugin.get_connection_status("Main OBS");
println!("Status: {:?}", status);

// List all connections
let connections = obs_plugin.get_connection_names();
println!("Active connections: {:?}", connections);
```

## Best Practices

### Tournament Setup
1. **Main OBS**: Use v5 protocol for latest features
2. **Backup OBS**: Use v4 protocol for compatibility
3. **Replay OBS**: Use v5 protocol for buffer operations
4. **Network**: Ensure all OBS instances are on same network
5. **Ports**: Use different ports for each instance

### Security
1. **Passwords**: Use strong, unique passwords for each connection
2. **Network**: Restrict WebSocket access to local network
3. **Firewall**: Configure firewall to allow only necessary ports
4. **Authentication**: Always enable authentication in production

### Performance
1. **Connections**: Limit to necessary connections only
2. **Events**: Subscribe only to required events
3. **Reconnection**: Implement automatic reconnection logic
4. **Monitoring**: Monitor connection health regularly

## Migration Guide

### From Single Protocol to Dual Protocol

1. **Backup Configuration**: Save current OBS WebSocket settings
2. **Install Both Plugins**: Install both v4 and v5 plugins in OBS
3. **Configure Ports**: Set different ports for each protocol
4. **Update Client**: Use new dual-protocol client
5. **Test Connections**: Verify both protocols work
6. **Deploy**: Gradually migrate to dual-protocol setup

### Protocol-Specific Features

| Feature | v4 Support | v5 Support | Notes |
|---------|------------|------------|-------|
| Scene Switching | ✅ | ✅ | Both protocols support |
| Recording Control | ✅ | ✅ | Both protocols support |
| Replay Buffer | ✅ | ✅ | Both protocols support |
| Authentication | ✅ | ✅ | v5 has stronger auth |
| Batch Requests | ❌ | ✅ | v5 only |
| Event Subscriptions | ❌ | ✅ | v5 only |
| Vendor Requests | ❌ | ✅ | v5 only |

## API Reference

### Connection Management
- `add_connection(config)` - Add new OBS connection
- `remove_connection(name)` - Remove OBS connection
- `get_connection_status(name)` - Get connection status
- `get_connection_names()` - List all connections

### Scene Operations
- `get_current_scene(name)` - Get current scene
- `set_current_scene(name, scene)` - Set current scene
- `get_scenes(name)` - Get all scenes

### Recording Operations
- `start_recording(name)` - Start recording
- `stop_recording(name)` - Stop recording
- `get_recording_status(name)` - Get recording status

### Replay Buffer Operations
- `start_replay_buffer(name)` - Start replay buffer
- `stop_replay_buffer(name)` - Stop replay buffer
- `save_replay_buffer(name)` - Save replay buffer
- `get_replay_buffer_status(name)` - Get replay buffer status

### Generic Operations
- `send_request(name, type, data)` - Send custom request
- Protocol differences are handled automatically 