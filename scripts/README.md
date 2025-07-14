# reStrike VTA Scripts

This directory contains all automation scripts for the reStrike VTA project, organized by category for easy navigation and maintenance.

## 📁 Script Categories

### 🚀 [Development](./development/)
Scripts for managing the development environment, dependencies, and build processes.

#### Core Development Scripts
- **[dev.sh](./development/dev.sh)** - Main development wrapper script
  ```bash
  ./scripts/development/dev.sh help          # Show all commands
  ./scripts/development/dev.sh start-all     # Start all services
  ./scripts/development/dev.sh status        # Check environment status
  ./scripts/development/dev.sh cleanup       # Full environment cleanup
  ```

#### Environment Management
- **[cleanup-dev-environment.sh](./development/cleanup-dev-environment.sh)** - Comprehensive environment cleanup
- **[manage-dev-resources.py](./development/manage-dev-resources.py)** - Development resource management
- **[verify-ports.sh](./development/verify-ports.sh)** - Port availability checking

#### Framework Management
- **[update-frameworks.sh](./development/update-frameworks.sh)** - Framework update automation
- **[install-mpv-latest.sh](./development/install-mpv-latest.sh)** - mpv media player installation

### 🔌 [OBS Integration](./obs/)
Scripts for OBS Studio integration and WebSocket setup.

- **[setup-obs-websocket.sh](./obs/setup-obs-websocket.sh)** - OBS WebSocket plugin setup
  ```bash
  ./scripts/obs/setup-obs-websocket.sh       # Interactive setup
  ./scripts/obs/setup-obs-websocket.sh --help # Show options
  ```

### 📋 [Project Management](./project/)
Scripts for project tracking, issue management, and reporting.

- **[project-tracker.py](./project/project-tracker.py)** - GitHub issue management
  ```bash
  python3 scripts/project/project-tracker.py summary    # Show project summary
  python3 scripts/project/project-tracker.py create     # Create new issue
  python3 scripts/project/project-tracker.py report     # Generate report
  ```

### 🎬 [Media Processing](./media/)
Scripts for video processing and media management.

- **[generate-clip.sh](./media/generate-clip.sh)** - Video clip generation
  ```bash
  ./scripts/media/generate-clip.sh [input] [output] [start] [duration]
  ```

### 🔄 [Workflows](./workflows/)
CI/CD workflows and automation.

- **[ci.yml](./workflows/ci.yml)** - Continuous integration workflow

## 🚀 Quick Start

### Main Development Commands
```bash
# Start development environment
./scripts/development/dev.sh start-all

# Check status
./scripts/development/dev.sh status

# Clean up
./scripts/development/dev.sh cleanup

# Get help
./scripts/development/dev.sh help
```

### Project Management
```bash
# View project summary
python3 scripts/project/project-tracker.py summary

# Create new feature
python3 scripts/project/project-tracker.py create "Feature Name" category priority "Description"

# Generate report
python3 scripts/project/project-tracker.py report my_report.md
```

### OBS Setup
```bash
# Interactive OBS WebSocket setup
./scripts/obs/setup-obs-websocket.sh
```

## 📝 Script Standards

### Naming Conventions
- **Development Scripts**: `kebab-case.sh` or `kebab-case.py`
- **Categories**: Grouped in subdirectories by purpose
- **Descriptive Names**: Clear indication of script purpose

### Script Requirements
- **Shebang**: All scripts should have proper shebang (`#!/bin/bash` or `#!/usr/bin/env python3`)
- **Help**: All scripts should provide `--help` or `-h` option
- **Error Handling**: Proper error handling and exit codes
- **Documentation**: Clear comments and usage examples

### Execution Permissions
```bash
# Make scripts executable
chmod +x scripts/development/*.sh
chmod +x scripts/obs/*.sh
chmod +x scripts/media/*.sh
```

## 🔧 Configuration

### Script Paths
Script paths are configured in `config/dev_resources.json`:
```json
{
  "scripts": {
    "cleanup": "scripts/development/cleanup-dev-environment.sh",
    "verify_ports": "scripts/development/verify-ports.sh",
    "update_frameworks": "scripts/development/update-frameworks.sh",
    "install_mpv": "scripts/development/install-mpv-latest.sh",
    "setup_obs": "scripts/obs/setup-obs-websocket.sh",
    "project_tracker": "scripts/project/project-tracker.py",
    "dev_wrapper": "scripts/development/dev.sh"
  }
}
```

### Environment Variables
Scripts use environment variables for configuration:
- `PROJECT_ROOT`: Project root directory
- `NODE_VERSION`: Node.js version
- `RUST_VERSION`: Rust version
- `MPV_VERSION`: mpv version

## 🛠️ Adding New Scripts

### 1. Choose Category
Place scripts in the appropriate category directory:
- **Development**: Environment and build scripts
- **OBS**: OBS Studio integration
- **Project**: Project management and tracking
- **Media**: Video and media processing
- **Workflows**: CI/CD and automation

### 2. Follow Standards
- Use proper naming convention
- Include help documentation
- Add error handling
- Update this README

### 3. Update Configuration
- Add script path to `config/dev_resources.json`
- Update any references in other scripts
- Test the script thoroughly

## 🔍 Troubleshooting

### Common Issues
- **Permission Denied**: Run `chmod +x script-name.sh`
- **Path Not Found**: Check script paths in configuration
- **Python Dependencies**: Install required Python packages
- **GitHub CLI**: Ensure `gh` CLI is installed and authenticated

### Getting Help
```bash
# Script help
./script-name.sh --help

# Development wrapper help
./scripts/development/dev.sh help

# Project tracker help
python3 scripts/project/project-tracker.py --help
```

## 📊 Script Status

| Category | Scripts | Status | Last Updated |
|----------|---------|--------|--------------|
| Development | 6 | ✅ Active | 2025-01-27 |
| OBS | 1 | ✅ Active | 2025-01-27 |
| Project | 1 | ✅ Active | 2025-01-27 |
| Media | 1 | ✅ Active | 2025-01-27 |
| Workflows | 1 | ✅ Active | 2025-01-27 |

---

**📝 Note**: All scripts are maintained by the development team and should be tested before use in production environments.

**🔄 Last Updated**: $(date)
**👤 Maintained by**: Development Team 