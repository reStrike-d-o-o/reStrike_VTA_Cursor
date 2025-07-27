# Network Accessibility Guide

## Overview
The reStrike VTA application now supports network accessibility for HTML overlays and real-time PSS event communication. This allows other computers on the network to access the scoreboard overlays and receive real-time updates.

## Changes Made

### 1. WebSocket Server Network Binding
- **File**: `src-tauri/src/plugins/plugin_websocket.rs`
- **Change**: WebSocket server now binds to `0.0.0.0:3001` instead of `127.0.0.1:3001`
- **Result**: WebSocket server is accessible from any network interface

### 2. HTML Overlay WebSocket Connection
- **Files**: 
  - `ui/public/scoreboard-overlay.html`
  - `ui/public/player-introduction-overlay.html`
- **Change**: Dynamic WebSocket URL detection based on current hostname
- **Logic**: 
  - If accessing from `localhost` or `127.0.0.1` → connects to `ws://127.0.0.1:3001`
  - If accessing from network IP → connects to `ws://[network-ip]:3001`

### 3. Development Server Network Access
- **File**: `ui/package.json`
- **Change**: Added `"dev:network": "set HOST=0.0.0.0 && react-scripts start"`
- **Result**: Development server accessible from network when using `npm run dev:network`

### 4. Updated UI Instructions
- **File**: `ui/src/components/molecules/ScoreboardManager.tsx`
- **Changes**:
  - Updated Network Access section with correct instructions
  - Removed deprecated `eventBroadcaster` references
  - Added network setup instructions

## How to Use Network Accessibility

### For Local Development
```bash
# Standard local development
npm run dev:fast
# Access at: http://localhost:3000
# WebSocket at: ws://127.0.0.1:3001
```

### For Network Access
```bash
# Network-accessible development
npm run dev:network
# Access at: http://[your-ip]:3000
# WebSocket at: ws://[your-ip]:3001
```

### Finding Your IP Address
- **Windows**: Run `ipconfig` in Command Prompt
- **Mac/Linux**: Run `ifconfig` in Terminal
- Look for your local network IP (usually starts with `192.168.` or `10.0.`)

## OBS Browser Source URLs

### Local Access
```
http://localhost:3000/scoreboard-overlay.html
http://localhost:3000/player-introduction-overlay.html
```

### Network Access
```
http://[your-ip]:3000/scoreboard-overlay.html
http://[your-ip]:3000/player-introduction-overlay.html
```

## Testing Network Accessibility

1. **Start the application with network access**:
   ```bash
   npm run dev:network
   ```

2. **Find your IP address**:
   ```bash
   ipconfig  # Windows
   ```

3. **Test from another computer**:
   - Open browser on another computer
   - Navigate to `http://[your-ip]:3000/scoreboard-overlay.html`
   - Check browser console for WebSocket connection logs

4. **Verify real-time updates**:
   - Load a match in the main application
   - Watch for PSS event updates in the overlay
   - Check console for WebSocket message logs

## Security Considerations

- The WebSocket server is currently unauthenticated
- Only use on trusted networks
- Consider adding authentication for production use
- Firewall may need to allow port 3001 for WebSocket connections

## Troubleshooting

### WebSocket Connection Issues
- Check if port 3001 is open in firewall
- Verify the Tauri app is running
- Check browser console for connection errors
- Ensure both computers are on the same network

### Development Server Issues
- Make sure to use `npm run dev:network` for network access
- Check if port 3000 is open in firewall
- Verify the development server started successfully

### HTML Overlay Not Updating
- Check WebSocket connection status in browser console
- Verify PSS data is loading in the main application
- Look for PSS event logs in the overlay console
- Test with a simple ping/pong message

## Technical Details

### WebSocket Message Format
```json
{
  "type": "pss_event",
  "data": {
    "type": "match_config",
    "message": "mch;1;Men's 73kg;73kg;Division A"
  },
  "timestamp": 1703123456789
}
```

### Connection Status
The WebSocket server provides status information:
```json
{
  "running": true,
  "port": 3001,
  "clients": 2,
  "address": "ws://0.0.0.0:3001",
  "network_accessible": true
}
```

### Automatic Reconnection
- HTML overlays automatically attempt to reconnect if connection is lost
- Exponential backoff with maximum 5 attempts
- Falls back to localStorage events if WebSocket fails
- Ping/pong keep-alive every 30 seconds 