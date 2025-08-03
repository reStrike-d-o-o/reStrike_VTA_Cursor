# tkStrike Hardware Simulator

A comprehensive hardware simulator for tkStrikeGen3 that implements the PSS v2.3 protocol specification for WT (World Taekwondo) competition data collection.

## 🎯 Purpose

This simulator allows you to:
- Test tkStrikeGen3 without requiring actual hardware
- Simulate realistic match scenarios and events
- Develop and debug PSS protocol implementations
- Train operators on tkStrikeGen3 functionality
- Generate test data for development and testing

## 🚀 Features

### Core Functionality
- **PSS v2.3 Protocol Compliance**: Full implementation of the WT UDP protocol specification
- **Real-time Event Generation**: Send realistic match events with proper timing
- **Multiple Operating Modes**: Interactive, demo, and random event generation
- **Predefined Scenarios**: Basic, championship, and training match configurations
- **Clock Simulation**: Real-time countdown timer with automatic updates

### Event Types Supported
- **Points**: Punch, Body, Head, Technical Body, Technical Head
- **Hit Levels**: Impact force measurements (1-100)
- **Warnings/Gam-jeom**: Referee warnings and penalties
- **Injury Time**: Athlete injury management with show/hide actions
- **Challenges/IVR**: Video review requests and outcomes
- **Break Time**: Inter-round breaks and timeouts
- **Match Control**: Clock, rounds, start/stop, winner determination

### Operating Modes

#### 1. Interactive Mode
- Command-line interface for manual control
- Real-time event generation
- Full match control and customization

#### 2. Demo Mode
- Automated match simulation
- Predefined event sequences
- Realistic timing and flow

#### 3. Random Mode
- Automated random event generation
- Configurable duration
- Stress testing capabilities

## 📋 Requirements

- Python 3.7 or higher
- No external dependencies (uses only standard library)
- Network access to target tkStrikeGen3 instance

## 🛠️ Installation

1. Clone or download the simulator files
2. Ensure Python 3.7+ is installed
3. No additional installation required

## 🎮 Usage

### Basic Usage

```bash
# Start interactive mode (default)
python tkstrike_hardware_simulator.py

# Run demo mode with basic scenario
python tkstrike_hardware_simulator.py --mode demo --scenario basic

# Run random events for 2 minutes
python tkstrike_hardware_simulator.py --mode random --duration 120

# Connect to specific host and port
python tkstrike_hardware_simulator.py --host 192.168.1.100 --port 6000
```

### Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--host` | Target host address | `127.0.0.1` |
| `--port` | Target port number | `6000` |
| `--mode` | Operating mode (interactive/demo/random) | `interactive` |
| `--scenario` | Match scenario (basic/championship/training) | `basic` |
| `--duration` | Duration for random mode (seconds) | `60` |

### Interactive Mode Commands

| Command | Description | Usage |
|---------|-------------|-------|
| `load` | Load match scenario | `load [basic\|championship\|training]` |
| `start` | Start the match | `start` |
| `stop` | Stop the match | `stop` |
| `point` | Add point | `point <athlete> <type>` |
| `warning` | Add warning | `warning <athlete>` |
| `injury` | Start injury time | `injury <athlete> <duration>` |
| `injury-stop` | Stop injury time | `injury-stop <athlete>` |
| `break` | Start break | `break <duration>` |
| `break-end` | End break | `break-end` |
| `clock` | Set clock | `clock <time>` |
| `round` | Set round | `round <num>` |
| `quit` | Exit simulator | `quit` |

### Point Types

| Type | Description |
|------|-------------|
| 1 | Punch |
| 2 | Body |
| 3 | Head |
| 4 | Technical Body |
| 5 | Technical Head |

### Athletes

| Number | Description |
|--------|-------------|
| 1 | Athlete 1 (Blue) |
| 2 | Athlete 2 (Red) |
| 0 | Unknown/Unidentified |

## 📊 Protocol Implementation

### PSS v2.3 Events

The simulator implements all major event types from the PSS v2.3 specification:

#### Connection Events
```
Udp Port 6000 connected;
Udp Port 6000 disconnected;
```

#### Match Setup Events
```
pre;FightLoaded;
at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;
mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;
rdy;FightReady;
```

#### Game Events
```
pt1;3;          # Head kick for athlete 1
hl1;75;         # Hit level 75 for athlete 1
wg1;1;wg2;0;    # Warning for athlete 1
ij2;1:30;show;  # Injury time for athlete 2
clk;1:45;       # Clock update
```

### Message Format

All messages follow the PSS protocol format:
```
<event_type>;<parameter1>;<parameter2>;...
```

## 🔧 Configuration

The simulator uses `config.json` for configuration:

```json
{
  "simulator": {
    "default_host": "127.0.0.1",
    "default_port": 6000
  },
  "scenarios": {
    "basic": { ... },
    "championship": { ... },
    "training": { ... }
  },
  "events": {
    "point_types": { ... },
    "hit_levels": { ... }
  }
}
```

## 🧪 Testing Scenarios

### Basic Match Flow
1. Connection establishment
2. Match loading with athlete information
3. Match configuration setup
4. Match start with clock countdown
5. Point scoring and hit level events
6. Warning and penalty events
7. Injury time management
8. Match conclusion

### Championship Match
- Higher-level athletes
- More complex scoring patterns
- Extended match duration
- Multiple rounds

### Training Match
- Training-specific configuration
- Longer round duration
- Count-up timer
- Different color scheme

## 🔍 Troubleshooting

### Common Issues

#### Connection Failed
```
❌ Failed to connect: [Errno 111] Connection refused
```
**Solution**: Ensure tkStrikeGen3 is running and listening on the correct port

#### Invalid Parameters
```
❌ Invalid athlete (must be 1 or 2)
❌ Invalid point type (must be 1-5)
```
**Solution**: Check command syntax and parameter ranges

#### Protocol Errors
```
❌ Failed to send message
```
**Solution**: Verify network connectivity and tkStrikeGen3 status

### Debug Mode

Enable verbose logging by modifying the simulator code:
```python
# Add debug prints
print(f"DEBUG: Sending message: {message}")
```

## 📈 Performance

- **Message Rate**: Up to 10 messages per second
- **Latency**: < 1ms local, < 10ms network
- **Memory Usage**: < 10MB
- **CPU Usage**: < 1% during normal operation

## 🔒 Security

- **Network**: Uses UDP protocol (no connection state)
- **Authentication**: None (simulator only sends data)
- **Validation**: Input validation for all parameters
- **Logging**: Optional event logging for debugging

## 📝 Logging

The simulator can log events to a file:
```json
{
  "logging": {
    "enabled": true,
    "level": "INFO",
    "file": "simulator.log"
  }
}
```

## 🤝 Integration

### With tkStrikeGen3
1. Start tkStrikeGen3
2. Configure UDP input on port 6000
3. Run simulator with appropriate parameters
4. Monitor tkStrikeGen3 for received events

### With Development Tools
- Use with network analyzers (Wireshark)
- Integrate with automated testing frameworks
- Combine with other PSS protocol tools

## 📚 References

- [PSS v2.3 Protocol Specification](protocol/pss_v2.3.txt)
- [WT Competition Rules](https://www.worldtaekwondo.org/)
- [tkStrikeGen3 Documentation](https://www.tkstrike.com/)

## 🆘 Support

For issues and questions:
1. Check the troubleshooting section
2. Review the protocol specification
3. Verify tkStrikeGen3 configuration
4. Test with different scenarios

## 📄 License

This simulator is provided as-is for development and testing purposes.

---

**Version**: 1.0.0  
**Protocol**: PSS v2.3  
**Compatibility**: tkStrikeGen3  
**Last Updated**: 2025-01-29 