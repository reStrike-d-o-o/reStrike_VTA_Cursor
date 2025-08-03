# reStrikeVTA Self-Test Report

**Generated:** 2025-08-04 01:07:18

## 📊 Overall Summary

- **Total Tests:** 4
- **Passed:** 1 ✅
- **Failed:** 1 ❌
- **Warnings:** 2 ⚠️
- **Success Rate:** 25.0%

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
**Duration:** 0.19s
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
**Duration:** 0.22s
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

## 💡 Recommendations

### Critical Issues
- Address failed tests before production deployment
- Review error messages for specific failure causes

### Warnings
- Investigate warning conditions for potential issues
- Consider performance optimizations where applicable

### System Health
- ❌ System requires attention
- Multiple critical failures detected