# Integration Guide: tkStrikeHardwareSimulator with reStrikeVTA

## 🎯 Overview

This guide explains how to use the `tkStrikeHardwareSimulator` to test your `reStrikeVTA` project. The simulator will send PSS v2.3 protocol messages to your application, allowing you to test real-time event processing, database storage, and UI updates without needing actual hardware.

## 🚀 Quick Setup

### 1. Verify Simulator Installation

First, ensure the simulator is working correctly:

```bash
cd hardware_simulator
python tkstrike_hardware_simulator.py --help
```

### 2. Port Configuration

**Important**: Your `reStrikeVTA` project listens on UDP port **8888**, but the simulator defaults to port **6000**. You need to configure the simulator to send to the correct port.

#### Option A: Use Command Line Arguments
```bash
python tkstrike_hardware_simulator.py --host 127.0.0.1 --port 8888
```

#### Option B: Modify Configuration
Edit `hardware_simulator/config.json`:
```json
{
  "simulator": {
    "default_host": "127.0.0.1",
    "default_port": 8888
  }
}
```

### 3. Start reStrikeVTA

Start your `reStrikeVTA` application:

```bash
# From project root
cd src-tauri
cargo tauri dev
```

This will:
- Start the React frontend (port 3000)
- Start the Tauri backend (port 1420)
- Start the UDP server listening on port 8888
- Start the WebSocket server (port 3001)

## 🧪 Testing Scenarios

### Scenario 1: Basic Match Testing

1. **Start reStrikeVTA** (as shown above)

2. **Run Basic Demo**:
```bash
cd hardware_simulator
python tkstrike_hardware_simulator.py --host 127.0.0.1 --port 8888 --demo basic
```

This will:
- Load a basic match scenario
- Send athlete information
- Send match configuration
- Simulate a 3-round match with points, warnings, and clock events
- Automatically end the match

### Scenario 2: Interactive Testing

1. **Start reStrikeVTA** (as shown above)

2. **Start Interactive Mode**:
```bash
cd hardware_simulator
python tkstrike_hardware_simulator.py --host 127.0.0.1 --port 8888 --interactive
```

3. **Use Interactive Commands**:
```
Available commands:
  load <scenario>     - Load match scenario (basic/championship/training)
  start              - Start the match
  point <athlete> <type> - Add point (1=blue, 2=red, type=1-5)
  warning <athlete>  - Add warning (1=blue, 2=red)
  injury <athlete>   - Start injury time
  break              - Start break
  clock              - Toggle clock
  stop               - Stop the match
  quit               - Exit simulator
```

### Scenario 3: Championship Match

```bash
cd hardware_simulator
python tkstrike_hardware_simulator.py --host 127.0.0.1 --port 8888 --demo championship
```

This simulates a high-level championship match with:
- Professional athletes
- Multiple rounds
- Complex scoring patterns
- Injury and break scenarios

### Scenario 4: Random Event Testing

```bash
cd hardware_simulator
python tkstrike_hardware_simulator.py --host 127.0.0.1 --port 8888 --random 120
```

This generates random events for 120 seconds, useful for stress testing.

## 📊 What to Monitor in reStrikeVTA

### 1. Event Table in DockBar
- Watch the "Event Table" section in the sidebar
- Events should appear in real-time with color-coded dots
- Verify event types: K (Kick), P (Punch), H (Head), TH (Technical Head), TB (Technical Body), R (Referee)

### 2. Scoreboard Overlay
- Open `ui/public/scoreboard-overlay.html` in a browser
- Verify real-time score updates
- Check injury time display/hide functionality
- Monitor clock synchronization

### 3. Database Storage
- Check the database for stored events
- Verify event relationships with matches
- Monitor performance under high event volume

### 4. WebSocket Broadcasting
- Monitor WebSocket connections (port 3001)
- Verify message broadcasting to all connected clients
- Check message structure and timing

## 🔧 Advanced Testing

### Custom Event Sequences

Create custom test scenarios by modifying the simulator:

```python
# In hardware_simulator/example_usage.py
def custom_test_scenario():
    simulator = tkStrikeHardwareSimulator("127.0.0.1", 8888)
    simulator.connect()
    
    # Load match
    simulator.load_match(MatchScenario.BASIC)
    
    # Custom event sequence
    simulator.add_point(1, 1)  # Blue athlete, punch
    time.sleep(2)
    simulator.add_point(2, 3)  # Red athlete, head kick
    time.sleep(1)
    simulator.add_warning(1)   # Blue athlete warning
    
    simulator.disconnect()
```

### Performance Testing

Test high-volume scenarios:

```bash
# Generate 1000 events rapidly
python tkstrike_hardware_simulator.py --host 127.0.0.1 --port 8888 --random 300 --fast
```

### Protocol Compliance Testing

Run the test suite to verify protocol compliance:

```bash
cd hardware_simulator
python test_simulator.py
```

## 🐛 Troubleshooting

### Common Issues

1. **"Connection refused"**
   - Ensure reStrikeVTA is running
   - Verify UDP port 8888 is open
   - Check firewall settings

2. **"No events appearing in UI"**
   - Check WebSocket connection (port 3001)
   - Verify event parsing in backend
   - Check browser console for errors

3. **"Database not storing events"**
   - Verify database connection
   - Check `store_pss_event` Tauri command
   - Monitor backend logs

4. **"Scoreboard overlay not updating"**
   - Verify WebSocket message structure
   - Check overlay HTML file connection
   - Monitor browser console

### Debug Commands

```bash
# Check if reStrikeVTA is listening on UDP port 8888
netstat -an | findstr 8888

# Check WebSocket server (port 3001)
netstat -an | findstr 3001

# Monitor UDP traffic
# (Use Wireshark or similar tool)

# Check reStrikeVTA logs
# Look in browser console and backend terminal
```

## 📈 Testing Checklist

### Basic Functionality
- [ ] Simulator connects to reStrikeVTA
- [ ] Events appear in Event Table
- [ ] Scoreboard overlay updates
- [ ] Database stores events
- [ ] WebSocket broadcasts work

### Advanced Features
- [ ] Manual mode toggle works
- [ ] New match creation stores previous events
- [ ] Event filtering works correctly
- [ ] Real-time clock synchronization
- [ ] Injury/break time handling

### Performance
- [ ] High-volume event processing
- [ ] Memory usage under load
- [ ] Database performance
- [ ] WebSocket connection stability

### Error Handling
- [ ] Malformed message handling
- [ ] Connection loss recovery
- [ ] Database error handling
- [ ] UI error states

## 🎯 Integration Benefits

Using the `tkStrikeHardwareSimulator` with `reStrikeVTA` provides:

1. **Realistic Testing**: Simulates actual PSS hardware behavior
2. **Protocol Validation**: Ensures PSS v2.3 compliance
3. **Performance Testing**: High-volume event generation
4. **Development Efficiency**: No hardware dependencies
5. **Automated Testing**: Scriptable test scenarios
6. **Debugging**: Controlled event sequences

## 📚 Next Steps

1. **Run Basic Tests**: Start with the basic demo scenario
2. **Explore Interactive Mode**: Test manual event generation
3. **Create Custom Scenarios**: Develop specific test cases
4. **Performance Testing**: Test under high load
5. **Integration Testing**: Test with OBS and other components

---

**Happy Testing! 🥋**

For more information, see:
- `hardware_simulator/README.md` - Complete simulator documentation
- `hardware_simulator/QUICKSTART.md` - Quick start guide
- `hardware_simulator/example_usage.py` - Programmatic examples 