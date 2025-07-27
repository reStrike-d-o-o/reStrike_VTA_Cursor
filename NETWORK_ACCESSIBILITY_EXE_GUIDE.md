# Network Accessibility Guide for Compiled .exe

## Overview

When you compile the reStrike VTA application to an .exe file, the HTML overlays need to be accessible from other computers on the same LAN. This guide explains the setup and configuration required.

## Architecture Changes for .exe

### 1. Built-in File Server

The compiled .exe now includes a built-in HTTP file server that:
- **Port**: 3000 (configurable)
- **Binding**: `0.0.0.0` (network accessible)
- **Serves**: HTML overlays and assets from within the .exe

### 2. WebSocket Server

The WebSocket server for real-time PSS events:
- **Port**: 3001 (configurable)
- **Binding**: `0.0.0.0` (network accessible)
- **Purpose**: Real-time PSS event broadcasting

## Network Setup

### Step 1: Find Your Computer's IP Address

**Windows:**
```cmd
ipconfig
```
Look for your local IP address (usually `192.168.x.x` or `10.x.x.x`)

**Mac/Linux:**
```bash
ifconfig
# or
ip addr
```

### Step 2: Configure Firewall

**Windows Firewall:**
1. Open Windows Defender Firewall
2. Click "Allow an app or feature through Windows Defender Firewall"
3. Click "Change settings" and "Allow another app"
4. Browse to your .exe file
5. Ensure both "Private" and "Public" are checked

**Alternative (Command Line):**
```cmd
netsh advfirewall firewall add rule name="reStrike VTA" dir=in action=allow program="C:\path\to\your\app.exe"
```

### Step 3: Access from Other Computers

**File Server URLs:**
- Scoreboard Overlay: `http://[YOUR-IP]:3000/scoreboard-overlay.html`
- Player Introduction: `http://[YOUR-IP]:3000/player-introduction-overlay.html`
- Test Page: `http://[YOUR-IP]:3000/test-scoreboard-fixes.html`

**WebSocket URL:**
- WebSocket Server: `ws://[YOUR-IP]:3001`

## OBS Integration

### Browser Source Setup

1. **Add Browser Source** in OBS
2. **URL**: `http://[YOUR-IP]:3000/scoreboard-overlay.html`
3. **Width**: 1920
4. **Height**: 1080
5. **Custom CSS**: (if needed for transparency)

### Multiple Overlays

You can add multiple browser sources:
- Scoreboard: `http://[YOUR-IP]:3000/scoreboard-overlay.html`
- Player Intro: `http://[YOUR-IP]:3000/player-introduction-overlay.html`

## Testing Network Accessibility

### 1. Test File Server
```bash
# From another computer
curl http://[YOUR-IP]:3000/scoreboard-overlay.html
```

### 2. Test WebSocket Connection
```javascript
// From browser console on another computer
const ws = new WebSocket('ws://[YOUR-IP]:3001');
ws.onopen = () => console.log('Connected!');
ws.onmessage = (event) => console.log('Received:', event.data);
```

### 3. Test Complete Setup
1. Start the .exe on your main computer
2. Open `http://[YOUR-IP]:3000/test-scoreboard-fixes.html` from another computer
3. Click test buttons to verify PSS events work
4. Open overlay URLs to see real-time updates

## Troubleshooting

### Common Issues

**1. Connection Refused**
- Check if .exe is running
- Verify firewall settings
- Ensure ports 3000 and 3001 are not blocked

**2. Can't Find IP Address**
- Ensure both computers are on same network
- Try `ping [YOUR-IP]` from other computer

**3. Overlays Not Updating**
- Check WebSocket connection in browser console
- Verify PSS events are being received
- Check network connectivity

**4. Assets Not Loading**
- Verify assets directory is copied correctly
- Check file paths in HTML files
- Ensure all SVG and JS files are present

### Debug Commands

**Check Server Status:**
```bash
# Check if ports are listening
netstat -an | findstr :3000
netstat -an | findstr :3001
```

**Test Local Access:**
```bash
# Test from same computer
curl http://localhost:3000/scoreboard-overlay.html
```

**Check Firewall Rules:**
```cmd
netsh advfirewall firewall show rule name="reStrike VTA"
```

## Configuration Options

### Custom Ports

You can modify the ports in the source code:

**File Server Port** (`src-tauri/src/core/app.rs`):
```rust
let file_server_plugin = Arc::new(Mutex::new(FileServerPlugin::new(3000)));
```

**WebSocket Port** (`src-tauri/src/core/app.rs`):
```rust
let websocket_plugin = Arc::new(Mutex::new(WebSocketPlugin::new(3001)));
```

### Network Interface Binding

The servers bind to `0.0.0.0` by default, which means they listen on all network interfaces. This is required for network accessibility.

## Security Considerations

### Production Deployment

1. **Firewall Rules**: Only allow necessary ports
2. **Network Isolation**: Use dedicated network segment if possible
3. **Access Control**: Consider implementing authentication for production use
4. **HTTPS**: For production, consider implementing HTTPS

### Development vs Production

- **Development**: Open access for testing
- **Production**: Restrict access to specific IP ranges
- **Monitoring**: Log access attempts and connections

## Performance Optimization

### Network Performance

1. **Compression**: Assets are served with appropriate content types
2. **Caching**: Browser caching is enabled for static assets
3. **Connection Pooling**: WebSocket connections are managed efficiently

### Resource Usage

- **Memory**: File server uses minimal memory
- **CPU**: WebSocket server is lightweight
- **Network**: Only sends data when PSS events occur

## Monitoring and Logging

### Built-in Logging

The application logs:
- File server startup and connections
- WebSocket connections and events
- PSS event broadcasting
- Error conditions

### Log Locations

- **Application Logs**: `logs/app.log`
- **Console Output**: Check terminal/console where .exe is running

## Support and Maintenance

### Regular Tasks

1. **Port Monitoring**: Ensure ports 3000 and 3001 are available
2. **Network Testing**: Regularly test network accessibility
3. **Log Review**: Monitor logs for connection issues
4. **Firewall Updates**: Update firewall rules as needed

### Updates

When updating the application:
1. Stop the current .exe
2. Deploy new version
3. Test network accessibility
4. Update firewall rules if ports changed

## Summary

The compiled .exe provides full network accessibility through:
- **Built-in HTTP file server** (port 3000)
- **WebSocket server** (port 3001)
- **Network binding** (`0.0.0.0`)
- **Automatic asset copying** during build

This enables seamless integration with OBS and real-time PSS event updates across your local network. 