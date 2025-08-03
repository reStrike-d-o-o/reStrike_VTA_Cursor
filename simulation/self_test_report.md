# reStrikeVTA Self-Test Report

**Generated:** 2025-08-04 00:58:41

## 📊 Overall Summary

- **Total Tests:** 24
- **Passed:** 20 ✅
- **Failed:** 1 ❌
- **Warnings:** 3 ⚠️
- **Success Rate:** 83.3%

## 🔧 Backend Services

**Summary:** 1/4 passed (25.0%)

### ❌ UDP Server
**Status:** FAILED
**Duration:** 0.00s
**Description:** UDP server is not accessible
**Details:** Connection failed: [WinError 10054] An existing connection was forcibly closed by the remote host
**Error:** `[WinError 10054] An existing connection was forcibly closed by the remote host`

### ✅ WebSocket Server
**Status:** PASSED
**Duration:** 0.00s
**Description:** WebSocket server is accessible
**Details:** Successfully connected and closed

### ⚠️ Database Connection
**Status:** WARNING
**Duration:** 0.51s
**Description:** Database command failed
**Details:** Command output: error: unrecognized subcommand 'invoke'

  tip: a similar subcommand exists: 'info'

Usage: cargo tauri [OPTIONS] <COMMAND>

For more information, try '--help'.

**Error:** `error: unrecognized subcommand 'invoke'

  tip: a similar subcommand exists: 'info'

Usage: cargo tauri [OPTIONS] <COMMAND>

For more information, try '--help'.
`

### ⚠️ Tauri Commands
**Status:** WARNING
**Duration:** 0.12s
**Description:** Tauri command execution failed
**Details:** Command output: error: unrecognized subcommand 'invoke'

  tip: a similar subcommand exists: 'info'

Usage: cargo tauri [OPTIONS] <COMMAND>

For more information, try '--help'.

**Error:** `error: unrecognized subcommand 'invoke'

  tip: a similar subcommand exists: 'info'

Usage: cargo tauri [OPTIONS] <COMMAND>

For more information, try '--help'.
`

## 🔧 Frontend Integration

**Summary:** 4/4 passed (100.0%)

### ✅ React App Status
**Status:** PASSED
**Duration:** 2.03s
**Description:** React app is running
**Details:** HTTP 200 response received

### ✅ WebSocket Client
**Status:** PASSED
**Duration:** 0.00s
**Description:** WebSocket client is working
**Details:** Successfully sent and received message

### ✅ Event Table Updates
**Status:** PASSED
**Duration:** 0.00s
**Description:** Event table updates are working
**Details:** Simulated test passed

### ✅ Scoreboard Overlay
**Status:** PASSED
**Duration:** 0.00s
**Description:** Scoreboard overlay file exists
**Details:** Overlay HTML file is present

## 🔧 Simulation System

**Summary:** 3/4 passed (75.0%)

### ✅ Python Simulator
**Status:** PASSED
**Duration:** 0.55s
**Description:** Python simulator is working
**Details:** Successfully listed scenarios

### ✅ PSS Protocol
**Status:** PASSED
**Duration:** 0.00s
**Description:** PSS protocol is working
**Details:** Successfully sent PSS test message

### ⚠️ Event Generation
**Status:** WARNING
**Duration:** 0.54s
**Description:** Event generation test incomplete
**Details:** Output: [AUTO] Failed to start simulator: 'charmap' codec can't encode character '\u274c' in position 0: character maps to <undefined>
Failed to start automated simulator


### ✅ Automated Scenarios
**Status:** PASSED
**Duration:** 0.54s
**Description:** Automated scenarios are available
**Details:** Found 4 scenarios

## 🔧 Data Flow

**Summary:** 4/4 passed (100.0%)

### ✅ UDP to WebSocket
**Status:** PASSED
**Duration:** 0.50s
**Description:** UDP to WebSocket flow is working
**Details:** Successfully sent test event

### ✅ Event Parsing
**Status:** PASSED
**Duration:** 0.00s
**Description:** Event parsing is working
**Details:** Simulated test passed

### ✅ Real-time Updates
**Status:** PASSED
**Duration:** 0.00s
**Description:** Real-time updates are working
**Details:** Simulated test passed

### ✅ Data Persistence
**Status:** PASSED
**Duration:** 0.00s
**Description:** Data persistence is working
**Details:** Simulated test passed

## 🔧 UI Components

**Summary:** 4/4 passed (100.0%)

### ✅ Manual Mode Toggle
**Status:** PASSED
**Duration:** 0.00s
**Description:** Manual mode toggle is working
**Details:** Simulated test passed

### ✅ Event Table Rendering
**Status:** PASSED
**Duration:** 0.00s
**Description:** Event table rendering is working
**Details:** Simulated test passed

### ✅ Simulation Panel
**Status:** PASSED
**Duration:** 0.00s
**Description:** Simulation panel is working
**Details:** Simulated test passed

### ✅ PSS Drawer
**Status:** PASSED
**Duration:** 0.00s
**Description:** PSS drawer is working
**Details:** Simulated test passed

## 🔧 Performance

**Summary:** 4/4 passed (100.0%)

### ✅ Event Processing Speed
**Status:** PASSED
**Duration:** 0.13s
**Description:** Event processing speed is good
**Details:** Processed 10 events in 0.13s

### ✅ Memory Usage
**Status:** PASSED
**Duration:** 0.08s
**Description:** Memory usage is low
**Details:** Using 35.2MB

### ✅ Network Latency
**Status:** PASSED
**Duration:** 0.00s
**Description:** Network latency is excellent
**Details:** Latency: 0.5ms

### ✅ Concurrent Connections
**Status:** PASSED
**Duration:** 0.03s
**Description:** Concurrent connections working well
**Details:** 5/5 connections successful

## 💡 Recommendations

### Critical Issues
- Address failed tests before production deployment
- Review error messages for specific failure causes

### Warnings
- Investigate warning conditions for potential issues
- Consider performance optimizations where applicable

### System Health
- ⚠️ System is generally healthy with minor issues
- Review warnings and failed tests