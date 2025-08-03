# Simulation Module

This module contains the tkStrike Hardware Simulator for testing reStrikeVTA with realistic PSS v2.3 protocol data.

## 📁 Folder Structure

```
simulation/
├── core/                    # Core simulator implementation
│   └── tkstrike_hardware_simulator.py
├── config/                  # Configuration files
│   └── config.json
├── tests/                   # Test scripts
│   ├── test_simulator.py
│   ├── test_integration.py
│   └── quick_test.py
├── examples/                # Example usage scripts
│   └── example_usage.py
├── docs/                    # Documentation
│   ├── README.md
│   ├── INTEGRATION_GUIDE.md
│   ├── QUICKSTART.md
│   ├── INTEGRATION_SUMMARY.md
│   └── SIMULATOR_SUMMARY.md
├── main.py                  # Main entry point
└── requirements.txt         # Python dependencies
```

## 🚀 Quick Start

### Command Line Usage
```bash
# Run basic demo
python simulation/main.py --mode demo --scenario basic

# Run random events
python simulation/main.py --mode random --duration 60

# Interactive mode
python simulation/main.py --mode interactive
```

### Integration with reStrikeVTA
The simulator is integrated into the reStrikeVTA PSS drawer with a dedicated Simulation tab for easy one-click operation.

## 🔧 Configuration

Edit `config/config.json` to customize:
- Default host and port
- Match scenarios
- Event parameters
- Timing settings

## 🧪 Testing

Run tests to verify functionality:
```bash
cd simulation/tests
python test_simulator.py
python test_integration.py
python quick_test.py
```

## 📚 Documentation

- `docs/INTEGRATION_GUIDE.md` - Detailed integration instructions
- `docs/QUICKSTART.md` - Quick start guide
- `docs/SIMULATOR_SUMMARY.md` - Technical implementation details

## 🎯 Features

- **PSS v2.3 Protocol Compliance**: Full implementation of WT UDP protocol
- **Multiple Scenarios**: Basic, championship, and training matches
- **Real-time Events**: Points, warnings, injuries, clock management
- **Interactive Control**: Manual event generation
- **Automated Testing**: Demo and random event modes
- **reStrikeVTA Integration**: Seamless integration with main application

## 🔗 Integration

The simulator is fully integrated with reStrikeVTA through:
- Frontend Simulation tab in PSS drawer
- Backend Tauri commands for simulation control
- Real-time event transmission to UDP port 8888
- WebSocket broadcasting for UI updates

---

**Version**: 1.0.0  
**Protocol**: PSS v2.3  
**Integration**: reStrikeVTA  
**Last Updated**: 2025-01-29 