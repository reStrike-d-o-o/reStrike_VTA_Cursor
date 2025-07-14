# Port Forwarding Configuration for reStrike VTA

## Overview
This document describes the port forwarding configuration for the reStrike VTA development environment and production deployment.

## Port Configuration

### Development Ports

| Port | Service | Description | Status |
|------|---------|-------------|--------|
| 3000 | React Frontend | React development server | ✅ Forwarded |
| 1420 | Tauri Backend | Tauri development server | ✅ Forwarded |
| 8080 | Development Server | Additional development services | ✅ Forwarded |

### Production Ports

| Port | Service | Description | Status |
|------|---------|-------------|--------|
| 6000 | UDP PSS Protocol | WT competition data collection | ✅ Forwarded |
| 4455 | OBS WebSocket | OBS Studio WebSocket control | ✅ Forwarded |

## Dev Container Configuration

The `.devcontainer/devcontainer.json` file includes automatic port forwarding:

```json
{
  "forwardPorts": [3000, 1420, 6000, 4455, 8080],
  "portsAttributes": {
    "3000": {
      "label": "React Frontend",
      "onAutoForward": "notify"
    },
    "1420": {
      "label": "Tauri Backend", 
      "onAutoForward": "notify"
    },
    "6000": {
      "label": "UDP PSS Protocol",
      "onAutoForward": "notify"
    },
    "4455": {
      "label": "OBS WebSocket",
      "onAutoForward": "notify"
    },
    "8080": {
      "label": "Development Server",
      "onAutoForward": "notify"
    }
  }
}
```

## Manual Port Forwarding

If you need to manually configure port forwarding:

### VS Code Dev Container
1. Open Command Palette (`Ctrl+Shift+P`)
2. Select "Dev Containers: Forward a Port"
3. Enter the port number
4. Choose "Always Allow" for persistent forwarding

### Docker Compose (Alternative)
```yaml
version: '3.8'
services:
  restrike-vta:
    ports:
      - "3000:3000"  # React Frontend
      - "1420:1420"  # Tauri Backend
      - "6000:6000"  # UDP PSS Protocol
      - "4455:4455"  # OBS WebSocket
      - "8080:8080"  # Development Server
```

## Port Usage Details

### Port 3000 - React Frontend
- **Service**: React development server
- **Protocol**: HTTP
- **Access**: http://localhost:3000
- **Purpose**: UI development and testing

### Port 1420 - Tauri Backend
- **Service**: Tauri development server
- **Protocol**: HTTP/WebSocket
- **Access**: http://localhost:1420
- **Purpose**: Backend API and Tauri integration

### Port 6000 - UDP PSS Protocol
- **Service**: WT competition data collection
- **Protocol**: UDP
- **Purpose**: Receiving competition data from PSS systems
- **Configuration**: Configured in `protocol/pss_schema.txt`

### Port 4455 - OBS WebSocket
- **Service**: OBS Studio WebSocket control
- **Protocol**: WebSocket
- **Purpose**: Controlling OBS Studio for video recording and scene switching
- **Configuration**: OBS WebSocket plugin must be enabled in OBS Studio

### Port 8080 - Development Server
- **Service**: Additional development services
- **Protocol**: HTTP
- **Purpose**: Future development features, API testing, etc.

## Troubleshooting

### Port Already in Use
If a port is already in use:
1. Check what's using the port: `netstat -tulpn | grep :<PORT>`
2. Stop the conflicting service
3. Or change the port in the configuration

### UDP Port 6000 Issues
- Ensure firewall allows UDP traffic on port 6000
- Check that PSS system is configured to send to correct IP
- Verify network interface binding in UDP plugin

### OBS WebSocket Connection Issues
- Ensure OBS WebSocket plugin is installed and enabled
- Check OBS WebSocket settings (default port: 4455)
- Verify authentication if enabled

## Security Considerations

- **Development ports** (3000, 1420, 8080) should only be accessible locally
- **Production ports** (6000, 4455) may need network access for external systems
- Consider using environment variables for port configuration in production
- Implement proper authentication for OBS WebSocket connections

## Environment Variables

You can configure ports using environment variables:

```bash
# Development
REACT_DEV_PORT=3000
TAURI_DEV_PORT=1420
DEV_SERVER_PORT=8080

# Production
UDP_PSS_PORT=6000
OBS_WEBSOCKET_PORT=4455
```

## Verification Commands

Test port availability:
```bash
# Check if ports are listening
netstat -tulpn | grep -E ':(3000|1420|6000|4455|8080)'

# Test UDP port 6000
nc -u -z localhost 6000

# Test HTTP ports
curl -I http://localhost:3000
curl -I http://localhost:1420
```

## Updates and Maintenance

- Review this document when adding new services
- Update port configurations in all relevant files
- Test port forwarding after container rebuilds
- Document any port conflicts or special requirements 