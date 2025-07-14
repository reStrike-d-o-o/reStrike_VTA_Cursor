# Development Environment Management

This document describes the tools and scripts available for managing the reStrike VTA development environment, including port management, service monitoring, and cleanup utilities.

## Overview

The development environment includes several tools to help manage:

- **Port Management**: Track and monitor all development ports
- **Service Management**: Start, stop, and monitor development services
- **Cleanup Utilities**: Clean up processes, cache, and temporary files
- **Resource Monitoring**: Health checks and status reporting

## Quick Start

### Using the Main Wrapper Script

The easiest way to manage your development environment is using the main wrapper script:

```bash
# Show all available commands
./scripts/dev.sh help

# Check current status
./scripts/dev.sh status

# Start all services
./scripts/dev.sh start-all

# Stop all services
./scripts/dev.sh stop-all

# Clean up environment
./scripts/dev.sh cleanup
```

## Available Tools

### 1. Development Wrapper (`scripts/dev.sh`)

A comprehensive wrapper script that provides easy access to all development commands.

#### Commands

**Development Commands:**
- `start-frontend` - Start React frontend
- `start-backend` - Start Rust backend
- `start-all` - Start both frontend and backend
- `stop-all` - Stop all development servers

**Management Commands:**
- `status` - Show development environment status
- `ports` - List all ports and their status
- `services` - List all services and their status
- `cleanup` - Full cleanup (stop processes, clear cache)
- `quick-cleanup` - Quick cleanup (stop processes only)
- `health` - Run health checks

**Utility Commands:**
- `install-deps` - Install all dependencies
- `build` - Build the project
- `test` - Run tests
- `update-config` - Update configuration status

#### Examples

```bash
# Start just the frontend
./scripts/dev.sh start-frontend

# Check what's running
./scripts/dev.sh status

# Clean up everything
./scripts/dev.sh cleanup

# Install dependencies
./scripts/dev.sh install-deps
```

### 2. Cleanup Script (`scripts/cleanup_dev_environment.sh`)

A comprehensive cleanup script that stops processes, clears cache, and checks ports.

#### Commands

- `--cleanup, -c` - Full cleanup (stop processes, clear cache, check ports)
- `--quick, -q` - Quick cleanup (stop processes only)
- `--status, -s` - Show current status
- `--help, -h` - Show help message

#### Examples

```bash
# Full cleanup
./scripts/cleanup_dev_environment.sh --cleanup

# Quick cleanup (just stop processes)
./scripts/cleanup_dev_environment.sh --quick

# Check current status
./scripts/cleanup_dev_environment.sh --status
```

### 3. Resource Manager (`scripts/manage_dev_resources.py`)

A Python script that manages the development resources configuration and provides detailed status information.

#### Commands

- `status` - Show current status summary
- `ports` - List all ports and their status
- `services` - List all services and their status
- `update` - Update all statuses
- `health` - Run health checks
- `export [file]` - Export status report (default: dev_status_report.json)
- `help` - Show help message

#### Examples

```bash
# Check status
python3 scripts/manage_dev_resources.py status

# List ports
python3 scripts/manage_dev_resources.py ports

# Update configuration
python3 scripts/manage_dev_resources.py update

# Export status report
python3 scripts/manage_dev_resources.py export my_report.json
```

## Configuration Database

### Overview

The development environment configuration is stored in `config/dev_resources.json`. This file contains:

- **Port Configuration**: All development ports and their status
- **Service Configuration**: All development services and their status
- **Environment Information**: Node.js, Rust, and other tool versions
- **Cleanup Configuration**: What to clean up and how
- **Monitoring Configuration**: Health checks and status indicators

### Structure

```json
{
  "development_environment": {
    "name": "reStrike VTA Development Environment",
    "version": "1.0.0",
    "last_updated": "2025-01-27"
  },
  "ports": {
    "frontend": {
      "port": 3000,
      "service": "React Frontend",
      "protocol": "HTTP",
      "status": "available",
      "forwarded": true
    }
  },
  "services": {
    "react_frontend": {
      "name": "React Frontend",
      "command": "npm start",
      "status": "stopped",
      "auto_restart": true
    }
  }
}
```

### Updating Configuration

The configuration is automatically updated when you run:

```bash
# Update all statuses
python3 scripts/manage_dev_resources.py update

# Or use the wrapper
./scripts/dev.sh update-config
```

## Port Management

### Default Ports

| Port | Service | Description | Auto-start |
|------|---------|-------------|------------|
| 3000 | React Frontend | React development server | ✅ Yes |
| 1420 | Tauri Backend | Tauri development server | ❌ No |
| 6000 | UDP PSS Protocol | WT competition data collection | ❌ No |
| 4455 | OBS WebSocket | OBS Studio WebSocket control | ❌ No |
| 8080 | Development Server | Additional development services | ❌ No |

### Port Status

Ports can have the following statuses:

- **available** - Port is free and ready to use
- **in_use** - Port is currently being used by a process
- **blocked** - Port is blocked by firewall or other system

### Checking Port Status

```bash
# Check all ports
./scripts/dev.sh ports

# Or use the Python script
python3 scripts/manage_dev_resources.py ports
```

## Service Management

### Available Services

| Service | Command | Directory | Auto-restart |
|---------|---------|-----------|--------------|
| React Frontend | `npm start` | `ui/` | ✅ Yes |
| Rust Backend | `cargo run` | `.` | ❌ No |
| mpv Player | `mpv` | N/A | ❌ No |
| OBS Studio | External | N/A | N/A |

### Service Status

Services can have the following statuses:

- **running** - Service is currently running
- **stopped** - Service is not running
- **external** - Service is external (like OBS Studio)

### Managing Services

```bash
# Start specific service
./scripts/dev.sh start-frontend
./scripts/dev.sh start-backend

# Start all services
./scripts/dev.sh start-all

# Stop all services
./scripts/dev.sh stop-all

# Check service status
./scripts/dev.sh services
```

## Cleanup Operations

### What Gets Cleaned Up

**Processes:**
- npm start (React frontend)
- cargo run (Rust backend)
- tauri dev (Tauri development)
- mpv (media player)

**Files and Directories:**
- `target/` (Rust build artifacts)
- `ui/node_modules/.cache/` (React cache)
- `*.tmp` files
- `*.log` files

**Cache:**
- npm cache
- cargo cache (optional)

### Cleanup Types

**Quick Cleanup:**
- Stops all development processes
- Does not clear cache or temporary files

**Full Cleanup:**
- Stops all development processes
- Clears all cache and temporary files
- Checks port status
- Shows final status report

### Running Cleanup

```bash
# Quick cleanup
./scripts/dev.sh quick-cleanup

# Full cleanup
./scripts/dev.sh cleanup

# Or use the cleanup script directly
./scripts/cleanup_dev_environment.sh --quick
./scripts/cleanup_dev_environment.sh --cleanup
```

## Health Checks

### Available Health Checks

- **Frontend**: `curl -f http://localhost:3000`
- **Backend**: `curl -f http://localhost:1420`
- **mpv**: `mpv --version`

### Running Health Checks

```bash
# Run all health checks
./scripts/dev.sh health

# Or use the Python script
python3 scripts/manage_dev_resources.py health
```

## Status Reporting

### Status Summary

The status summary shows:

- Environment information (Node.js, Rust versions)
- Port availability (X/Y available)
- Service status (X/Y running)
- Quick status for auto-start services

### Exporting Reports

You can export detailed status reports to JSON:

```bash
# Export to default file (dev_status_report.json)
python3 scripts/manage_dev_resources.py export

# Export to custom file
python3 scripts/manage_dev_resources.py export my_report.json
```

## Best Practices

### Daily Development Workflow

1. **Start of day:**
   ```bash
   ./scripts/dev.sh status          # Check current state
   ./scripts/dev.sh cleanup         # Clean up from previous session
   ./scripts/dev.sh start-all       # Start all services
   ```

2. **During development:**
   ```bash
   ./scripts/dev.sh status          # Check status when needed
   ./scripts/dev.sh health          # Run health checks if issues
   ```

3. **End of day:**
   ```bash
   ./scripts/dev.sh stop-all        # Stop all services
   ./scripts/dev.sh cleanup         # Clean up
   ```

### Troubleshooting

**Port conflicts:**
```bash
./scripts/dev.sh ports              # Check what's using ports
./scripts/dev.sh cleanup            # Clean up processes
```

**Service issues:**
```bash
./scripts/dev.sh health             # Run health checks
./scripts/dev.sh services           # Check service status
```

**Cache issues:**
```bash
./scripts/dev.sh cleanup            # Full cleanup clears cache
```

## Integration with IDE

### VS Code Tasks

You can add these tasks to your `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Start All Services",
      "type": "shell",
      "command": "./scripts/dev.sh",
      "args": ["start-all"],
      "group": "build"
    },
    {
      "label": "Stop All Services",
      "type": "shell",
      "command": "./scripts/dev.sh",
      "args": ["stop-all"],
      "group": "build"
    },
    {
      "label": "Check Status",
      "type": "shell",
      "command": "./scripts/dev.sh",
      "args": ["status"],
      "group": "test"
    },
    {
      "label": "Cleanup",
      "type": "shell",
      "command": "./scripts/dev.sh",
      "args": ["cleanup"],
      "group": "build"
    }
  ]
}
```

### Git Hooks

You can add a pre-commit hook to clean up before committing:

```bash
#!/bin/bash
# .git/hooks/pre-commit
./scripts/dev.sh quick-cleanup
```

## Future Enhancements

### Planned Features

- **Auto-restart**: Automatically restart services that crash
- **Port conflict resolution**: Automatically find alternative ports
- **Dependency checking**: Verify all required tools are installed
- **Performance monitoring**: Track resource usage
- **Integration with CI/CD**: Automated environment setup

### Configuration Extensions

- **Custom ports**: Allow custom port configurations
- **Service dependencies**: Define service startup order
- **Environment-specific configs**: Different configs for dev/staging/prod
- **Plugin system**: Allow custom cleanup and monitoring plugins

---

## Support

If you encounter issues with the development management tools:

1. Check the help documentation: `./scripts/dev.sh help`
2. Run status checks: `./scripts/dev.sh status`
3. Try cleanup: `./scripts/dev.sh cleanup`
4. Check the configuration: `config/dev_resources.json`

For more detailed information, see the individual script documentation in the `scripts/` directory. 