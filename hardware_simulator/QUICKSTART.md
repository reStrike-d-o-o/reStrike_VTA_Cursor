# Quick Start Guide - tkStrike Hardware Simulator

Get up and running with the tkStrike Hardware Simulator in minutes!

## 🚀 Quick Setup

### 1. Prerequisites
- Python 3.7 or higher
- tkStrikeGen3 running and listening on port 6000

### 2. Download/Clone
```bash
# If you have the files locally:
cd hardware_simulator/

# Or clone from repository:
git clone <repository-url>
cd hardware_simulator/
```

### 3. Test the Installation
```bash
python test_simulator.py
```

You should see:
```
🧪 tkStrike Hardware Simulator Test Suite
==================================================
✅ All tests passed! Simulator is ready for use.
```

## 🎮 Quick Usage

### Interactive Mode (Recommended for beginners)
```bash
python tkstrike_hardware_simulator.py
```

This starts the interactive command-line interface where you can:
- Load matches
- Add points and events
- Control the match flow
- Test different scenarios

### Demo Mode (Automated match)
```bash
python tkstrike_hardware_simulator.py --mode demo --scenario basic
```

This runs a complete automated match with realistic events.

### Random Mode (Stress testing)
```bash
python tkstrike_hardware_simulator.py --mode random --duration 60
```

This generates random events for 60 seconds.

## 📋 Essential Commands (Interactive Mode)

Once in interactive mode, use these commands:

| Command | What it does | Example |
|---------|-------------|---------|
| `load basic` | Load a basic match | `load basic` |
| `start` | Start the match | `start` |
| `point 1 3` | Add head kick for athlete 1 | `point 1 3` |
| `warning 2` | Add warning for athlete 2 | `warning 2` |
| `injury 1 30` | Start 30s injury time for athlete 1 | `injury 1 30` |
| `stop` | Stop the match | `stop` |
| `quit` | Exit simulator | `quit` |

## 🥊 Point Types

| Type | Description | Example |
|------|-------------|---------|
| 1 | Punch | `point 1 1` |
| 2 | Body | `point 2 2` |
| 3 | Head | `point 1 3` |
| 4 | Technical Body | `point 2 4` |
| 5 | Technical Head | `point 1 5` |

## 🎯 Quick Test Scenario

1. **Start tkStrikeGen3** and ensure it's listening on port 6000

2. **Run the simulator**:
   ```bash
   python tkstrike_hardware_simulator.py
   ```

3. **Load and start a match**:
   ```
   > load basic
   > start
   ```

4. **Add some events**:
   ```
   > point 1 3    # Head kick for blue athlete
   > point 2 2    # Body kick for red athlete
   > warning 1    # Warning for blue athlete
   > point 1 1    # Punch for blue athlete
   ```

5. **Stop the match**:
   ```
   > stop
   > quit
   ```

## 🔧 Troubleshooting

### Connection Issues
```
❌ Failed to connect: [Errno 111] Connection refused
```
**Solution**: Make sure tkStrikeGen3 is running and listening on port 6000

### Command Not Found
```
❌ Unknown command
```
**Solution**: Check the command syntax. Use `help` or refer to the command list above.

### No Events Showing in tkStrikeGen3
**Solution**: 
1. Verify tkStrikeGen3 UDP input is enabled
2. Check that port 6000 is configured correctly
3. Ensure firewall isn't blocking UDP traffic

## 📚 Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Try the [examples](example_usage.py) for programmatic usage
- Explore different scenarios (basic, championship, training)
- Test with your specific tkStrikeGen3 configuration

## 🆘 Need Help?

1. Check the [troubleshooting section](README.md#troubleshooting)
2. Run the test suite: `python test_simulator.py`
3. Review the [protocol specification](../protocol/pss_v2.3.txt)
4. Check tkStrikeGen3 documentation

---

**Happy simulating! 🥋** 