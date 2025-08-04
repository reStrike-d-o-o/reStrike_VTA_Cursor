# tkStrike Hardware Simulator

A comprehensive hardware simulator for tkStrikeGen3 that implements the PSS v2.3 protocol specification for WT (World Taekwondo) competition data collection.

## 🎯 Purpose

This simulator provides realistic PSS protocol events for testing reStrikeVTA functionality, including:
- **Manual Simulation**: Interactive control for testing specific scenarios
- **Automated Simulation**: Multi-match scenarios with realistic event generation
- **Protocol Compliance**: Full PSS v2.3 protocol implementation
- **Real-time Integration**: Seamless integration with reStrikeVTA Event Table and Scoreboard Overlay

## 🚀 Features

### Manual Simulation
- **Interactive Mode**: Real-time manual event generation
- **Demo Mode**: Predefined match scenarios (basic, championship, training)
- **Random Mode**: Random event generation for stress testing
- **Manual Events**: One-click buttons for points, warnings, injuries, etc.

### Automated Simulation ✨ **NEW**
- **Multi-Match Scenarios**: Run multiple matches automatically
- **Realistic Event Generation**: Probability-based event sequences
- **Random Athletes**: Generate realistic athlete data from multiple countries
- **Dynamic Match Configs**: Random match configurations and categories
- **Progress Tracking**: Real-time progress monitoring
- **Scenario Types**:
  - **Quick Test**: Single match for testing (30-60 seconds)
  - **Training Session**: 5 matches for training (1-3 minutes each)
  - **Tournament Day**: 20 matches for full tournament simulation (2-5 minutes each)
  - **Championship**: 8 high-intensity matches (3-6 minutes each)

## 📋 Requirements

- Python 3.8+ (automatically detected and verified)
- Network access to target host/port
- reStrikeVTA application running (for integration testing)

## 🛠️ Installation

### Automatic Installation (Recommended)

The reStrikeVTA application now includes **robust dependency management** that automatically:

1. **Detects Python installation** (supports `python3`, `python`, `py` on Windows)
2. **Verifies Python version** (requires 3.8 or higher)
3. **Checks required packages** (`requests`, `websocket-client`, `psutil`, etc.)
4. **Auto-installs missing dependencies** via `pip install -r requirements.txt`
5. **Provides clear error messages** and retry options in the UI

### Manual Installation (Alternative)

If you prefer manual installation:

1. **Clone or download** the simulation files to your project
2. **Install dependencies**:
   ```bash
   pip install -r requirements.txt
   ```
3. **Verify installation**:
   ```bash
   python test_automated.py
   ```

### Troubleshooting

If you encounter dependency issues:

1. **Check Python installation**: Ensure Python 3.8+ is installed and in PATH
2. **Use the UI retry buttons**: Click "Retry" or "Install Dependencies" in the Simulation tab
3. **Manual pip install**: Run `pip install -r requirements.txt` manually
4. **Check internet connection**: Required for downloading packages

## 🎮 Usage

### Integration with reStrikeVTA

This simulator is designed to work with the reStrikeVTA project. To test the integration:

1. **Start reStrikeVTA**: `cd src-tauri && cargo tauri dev`
2. **Run Quick Test**: `python test_automated.py`
3. **Run Integration Tests**: `python test_integration.py`
4. **Use Interactive Mode**: `python main.py --interactive`

For detailed integration instructions, see `INTEGRATION_GUIDE.md`.

### Command Line Usage

#### Manual Simulation
```bash
# Interactive mode
python main.py --mode interactive

# Demo mode with basic scenario
python main.py --mode demo --scenario basic --duration 60

# Random events for 2 minutes
python main.py --mode random --duration 120
```

#### Automated Simulation ✨ **NEW**
```bash
# List available automated scenarios
python main.py --list-scenarios

# Run quick test scenario
python main.py --mode automated --scenario quick_test

# Run training session (5 matches)
python main.py --mode automated --scenario training_session

# Run full tournament day (20 matches)
python main.py --mode automated --scenario tournament_day

# Run championship matches (8 high-intensity matches)
python main.py --mode automated --scenario championship
```

### Frontend Integration

The simulator integrates with reStrikeVTA's frontend through the Simulation Panel:

1. **Open PSS Drawer** → **Simulation Tab**
2. **Enhanced Error Handling**: The UI now provides:
   - **Clear error messages** for Python/dependency issues
   - **Retry buttons** to attempt reconnection
   - **Install Dependencies** button for automatic package installation
   - **Progress indicators** during dependency installation
   - **User-friendly messages** instead of technical error codes

### Error Handling & Troubleshooting

The application now provides robust error handling:

#### Common Issues & Solutions

| Issue | Error Message | Solution |
|-------|---------------|----------|
| Python not found | "Python is not installed or not found in PATH" | Install Python 3.8+ and add to PATH |
| Python version too low | "Python version is too low" | Upgrade to Python 3.8+ |
| Missing packages | "Required Python packages are missing" | Click "Install Dependencies" button |
| Network issues | "Failed to install Python dependencies" | Check internet connection and retry |
| Simulation files missing | "Simulation files not found" | Reinstall the application |

#### UI Error Handling Features

- **Automatic Detection**: Environment issues are detected before simulation starts
- **Actionable Messages**: Clear instructions on how to fix each issue
- **One-Click Fixes**: Install dependencies with a single button click
- **Progress Feedback**: Loading indicators during installation
- **Retry Options**: Easy retry without restarting the application
2. **Toggle "Automated Simulation"** to enable automated mode
3. **Select Scenario** from dropdown (Quick Test, Training Session, etc.)
4. **Click "Start Automated"** to begin simulation
5. **Monitor Progress** with real-time status updates

## 📊 Protocol Implementation

### PSS v2.3 Events Supported
- **Match Setup**: Fight loaded, athletes, match config, fight ready
- **Scoring**: Points (punch, kick, head kick, technical body/head)
- **Penalties**: Warnings, disqualifications
- **Time Management**: Clock start/stop, round changes
- **Medical**: Injury time, breaks
- **Challenges**: Coach challenges, video reviews
- **Results**: Winners, final scores

### Event Generation
- **Realistic Timing**: Events occur at realistic intervals
- **Probability-Based**: Different event types have configurable probabilities
- **Athlete-Specific**: Events are assigned to specific athletes (blue/red)
- **Round-Aware**: Events respect round structure and timing

## 🔧 Configuration

### Automated Scenario Configuration
Scenarios are defined in `core/automated_simulator.py`:

```python
"quick_test": AutomatedScenario(
    name="Quick Test",
    description="Fast single match for testing",
    match_count=1,
    duration_range=(30, 60),
    event_frequency=0.5,
    point_probability=0.3,
    warning_probability=0.1,
    injury_probability=0.05,
    break_probability=0.02,
    challenge_probability=0.03
)
```

### Custom Scenarios
You can create custom scenarios by modifying the `scenarios` dictionary in `AutomatedSimulator`:

```python
"custom_scenario": AutomatedScenario(
    name="Custom Scenario",
    description="Your custom scenario description",
    match_count=10,
    duration_range=(90, 180),
    event_frequency=1.0,
    point_probability=0.4,
    warning_probability=0.15,
    injury_probability=0.08,
    break_probability=0.05,
    challenge_probability=0.05
)
```

## 🧪 Testing Scenarios

### Manual Testing
- **Basic Functionality**: Test individual event types
- **Protocol Compliance**: Verify PSS v2.3 message format
- **Integration**: Test with reStrikeVTA Event Table and Scoreboard

### Automated Testing
- **Quick Test**: Fast validation of basic functionality
- **Training Session**: Medium-duration testing with multiple matches
- **Tournament Day**: Extended testing with high event volume
- **Championship**: High-intensity testing with complex scenarios

### Test Commands
```bash
# Run automated test suite
python test_automated.py

# Test specific scenario
python main.py --mode automated --scenario quick_test

# Test with custom duration
python main.py --mode automated --scenario training_session

# Run comprehensive self-test
python main.py --self-test

# Test self-test system
python tests/test_self_test.py
```

### Self-Test System

The simulator includes a comprehensive self-test system that monitors all system integrations:

**Test Categories:**
- **Backend Services**: UDP Server, WebSocket Server, Database Connection, Tauri Commands
- **Frontend Integration**: React App Status, WebSocket Client, Event Table Updates, Scoreboard Overlay
- **Simulation System**: Python Simulator, PSS Protocol, Event Generation, Automated Scenarios
- **Data Flow**: UDP to WebSocket, Event Parsing, Real-time Updates, Data Persistence
- **UI Components**: Manual Mode Toggle, Event Table Rendering, Simulation Panel, PSS Drawer
- **Performance**: Event Processing Speed, Memory Usage, Network Latency, Concurrent Connections

**Selective Testing** ✨ **NEW**
- **Category Selection**: Choose specific test categories to run
- **Toggle Controls**: Enable/disable individual categories
- **Bulk Operations**: Select all or deselect all categories
- **Visual Feedback**: Real-time status updates and progress monitoring
- **Comprehensive Reports**: Detailed markdown reports with recommendations

**Usage:**
```bash
# List available test categories
python main.py --list-test-categories

# Run comprehensive self-test (all categories)
python main.py --self-test

# Run selective self-test for specific categories
python main.py --self-test --test-categories "Backend Services" "Frontend Integration"

# Run multiple categories
python main.py --self-test --test-categories "Backend Services" "Simulation System" "Performance"
```

**Features:**
- **Comprehensive Testing**: 24 individual tests across 6 categories
- **Real-time Monitoring**: Progress tracking and status updates
- **Detailed Reporting**: Markdown format reports with recommendations
- **Performance Metrics**: Memory usage, network latency, processing speed
- **Integration Validation**: End-to-end system connectivity testing

## 🔍 Troubleshooting

### Common Issues
1. **Connection Failed**: Ensure reStrikeVTA is running and listening on port 8888
2. **Python Not Found**: Ensure Python 3.8+ is installed and in PATH
3. **Import Errors**: Run `pip install -r requirements.txt`
4. **Permission Errors**: Run as administrator if needed

### Debug Mode
Enable debug logging by setting environment variable:
```bash
export PYTHONPATH=.
python main.py --mode automated --scenario quick_test
```

## 📈 Performance

### Manual Simulation
- **Event Rate**: Up to 10 events per second
- **Latency**: < 10ms per event
- **Memory Usage**: < 50MB

### Automated Simulation
- **Match Rate**: 1-3 matches per minute (depending on scenario)
- **Event Generation**: 0.5-1.5 events per second
- **Memory Usage**: < 100MB for extended runs
- **CPU Usage**: < 5% on modern systems

## 🔒 Security

- **Local Only**: Simulator only connects to localhost
- **No External Data**: All data is generated locally
- **Protocol Compliant**: Uses standard PSS v2.3 protocol
- **Safe for Testing**: No production data or systems affected

## 📝 Logging

### Log Levels
- **INFO**: General operation messages
- **DEBUG**: Detailed event generation
- **WARNING**: Non-critical issues
- **ERROR**: Critical failures

### Log Output
- **Console**: Real-time status updates
- **File**: Detailed logs (if configured)
- **Frontend**: Status updates via Tauri commands

## 🤝 Integration

### reStrikeVTA Integration
- **UDP Communication**: Sends events to port 8888
- **WebSocket Broadcasting**: Events appear in real-time
- **Event Table**: Events displayed in DockBar
- **Scoreboard Overlay**: Live score updates
- **Database Storage**: Events stored when implemented

### Tauri Commands
```rust
// Get available scenarios
simulation_get_scenarios()

// Run automated simulation
simulation_run_automated(scenario_name, custom_config)

// Get detailed status
simulation_get_detailed_status()
```

### Warning Limit Rule
The simulator implements a 5-warning limit per athlete per round:
- **Automatic Enforcement**: Athletes cannot receive more than 5 warnings per round
- **Round Loss**: When an athlete reaches 5 warnings, the round automatically ends
- **Winner Determination**: The opposite athlete wins the round
- **Round Reset**: Warning counts reset when a new round begins
- **Match End**: The match ends immediately when a round is lost due to warnings

This rule applies to both manual and automated simulation modes.

## 📚 References

- **PSS Protocol**: World Taekwondo PSS v2.3 Specification
- **reStrikeVTA**: Main application documentation
- **Tauri**: Cross-platform framework documentation
- **Python Socket**: Network programming documentation

## 🆘 Support

### Getting Help
1. **Check Documentation**: Review this README and related docs
2. **Run Tests**: Execute `python test_automated.py`
3. **Check Logs**: Review console output for error messages
4. **Verify Setup**: Ensure all dependencies are installed

### Common Commands
```bash
# Test basic functionality
python test_automated.py

# List available scenarios
python main.py --list-scenarios

# Run quick test
python main.py --mode automated --scenario quick_test

# Interactive mode for debugging
python main.py --mode interactive
```

## 📄 License

This simulator is part of the reStrikeVTA project and follows the same licensing terms.

---

**Version**: 2.0.0  
**Protocol**: PSS v2.3  
**Compatibility**: tkStrikeGen3, reStrikeVTA  
**Last Updated**: 2025-01-29  
**Features**: Manual + Automated Simulation + Warning Limit Rule ✨ 