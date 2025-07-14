# Dev Container Restart Guide

## Overview

This guide explains how to restart the dev container to apply framework updates, including Node.js v24, latest mpv, and other dependency updates.

## When to Restart the Container

Restart the container when:
- ✅ Framework updates are applied (Node.js, mpv, etc.)
- ✅ Dev container configuration changes
- ✅ Dockerfile modifications
- ✅ Port forwarding updates
- ✅ New development tools added
- 🔄 After major dependency updates

## Restart Methods

### Method 1: VS Code Command Palette (Recommended)

1. **Open Command Palette**
   - Press `Ctrl+Shift+P` (Windows/Linux) or `Cmd+Shift+P` (Mac)
   - Or go to `View → Command Palette`

2. **Select Rebuild Command**
   - Type: `Dev Containers: Rebuild and Reopen in Container`
   - Select the command from the dropdown

3. **Wait for Rebuild**
   - VS Code will show progress in the bottom-right corner
   - This may take 5-10 minutes depending on your system

### Method 2: VS Code UI

1. **Click Dev Container Icon**
   - Look for the container icon in the bottom-left corner
   - It shows the current container status

2. **Select Rebuild Option**
   - Click the container icon
   - Select **"Rebuild Container"** from the menu

3. **Confirm Rebuild**
   - Click **"Rebuild"** when prompted
   - Wait for the process to complete

### Method 3: Command Line (Advanced)

```bash
# From VS Code terminal
code --command "devcontainers.rebuild"

# Or using Docker directly (not recommended)
docker-compose down
docker-compose up --build
```

## What Happens During Rebuild

### Framework Updates Applied
- ✅ **Node.js**: v18.20.8 → v24.4.0 (latest LTS)
- ✅ **mpv**: v0.32.0 → latest stable version
- ✅ **React**: v18.3.1 (latest stable)
- ✅ **TypeScript**: v5.4.3 (latest stable)
- ✅ **All npm packages**: Updated to latest compatible versions

### Configuration Updates
- ✅ **Port Forwarding**: All ports (3000, 1420, 6000, 4455, 8080) configured
- ✅ **Dev Container**: Updated Node.js feature configuration
- ✅ **Dockerfile**: Added mpv PPA repository
- ✅ **Dependencies**: All package.json files updated

### Environment Setup
- ✅ **Development Tools**: All tools reinstalled with latest versions
- ✅ **Cache Cleared**: Fresh start with updated dependencies
- ✅ **Permissions**: All file permissions reset
- ✅ **Network**: Port forwarding reconfigured

## Pre-Restart Checklist

Before restarting, ensure you have:

- [ ] **Saved all work** - Any unsaved changes will be lost
- [ ] **Committed changes** - Git commits are preserved
- [ ] **Backed up configs** - Any custom configurations
- [ ] **Noted current state** - Running services, open files, etc.

## Post-Restart Verification

After the container rebuilds, verify:

### 1. Check Framework Versions
```bash
# Check Node.js version
node --version  # Should show v24.4.0

# Check mpv version
mpv --version   # Should show latest version

# Check Rust version
rustc --version # Should show 1.88.0
```

### 2. Verify Dependencies
```bash
# Check npm packages
npm outdated    # Should show minimal outdated packages

# Check UI dependencies
cd ui && npm outdated && cd ..
```

### 3. Test Core Functionality
```bash
# Start React frontend
cd ui && npm start

# In another terminal, start Rust backend
npm start
```

### 4. Check Port Forwarding
```bash
# Use the verification script
./scripts/verify_ports.sh

# Or check manually
netstat -tulpn | grep -E ":(3000|1420|6000|4455|8080)"
```

## Troubleshooting

### Common Issues

#### 1. Rebuild Fails
```bash
# Check Docker status
docker ps
docker system prune -f

# Check dev container logs
# In VS Code: View → Output → Dev Containers
```

#### 2. Port Conflicts
```bash
# Check what's using the ports
./scripts/dev.sh ports

# Clean up processes
./scripts/dev.sh cleanup
```

#### 3. Dependency Issues
```bash
# Reinstall dependencies
npm install
cd ui && npm install && cd ..

# Clear npm cache
npm cache clean --force
```

#### 4. Permission Issues
```bash
# Fix file permissions
sudo chown -R $USER:$USER .

# Fix npm permissions
sudo chown -R $USER:$USER ~/.npm
```

### Recovery Steps

If the rebuild fails:

1. **Check Error Messages**
   - Look at the VS Code output panel
   - Check Docker logs

2. **Try Alternative Method**
   - If Command Palette fails, try UI method
   - If both fail, try command line

3. **Manual Recovery**
   ```bash
   # Stop all containers
   docker stop $(docker ps -q)
   
   # Remove dev container
   docker rmi $(docker images -q)
   
   # Rebuild from scratch
   # VS Code → Rebuild Container
   ```

## Performance Tips

### Faster Rebuilds
- **Use Docker BuildKit**: Enable for faster builds
- **Optimize Dockerfile**: Minimize layers and cache effectively
- **Use .dockerignore**: Exclude unnecessary files

### Resource Management
- **Allocate More Memory**: Give Docker more RAM if rebuilds are slow
- **Use SSD Storage**: Faster disk I/O improves rebuild times
- **Close Other Apps**: Free up system resources

## Automation

### Automated Restart Script
```bash
#!/bin/bash
# scripts/restart_container.sh

echo "🔄 Restarting dev container..."

# Save current state
git add .
git commit -m "Auto-save before container restart" || true

# Trigger rebuild
code --command "devcontainers.rebuild"

echo "✅ Container restart initiated"
echo "📋 Check VS Code for progress"
```

### Git Hooks
```bash
# .git/hooks/pre-commit
#!/bin/bash
# Auto-save before commits
./scripts/dev.sh quick-cleanup
```

## Best Practices

### Before Restart
1. **Commit Changes**: Always commit or stash changes
2. **Document State**: Note any running services or configurations
3. **Backup Configs**: Save any custom configurations
4. **Check Dependencies**: Ensure all updates are ready

### After Restart
1. **Verify Versions**: Check all framework versions
2. **Test Functionality**: Run core application features
3. **Update Documentation**: Update any version-specific docs
4. **Share Changes**: Inform team of any breaking changes

### Regular Maintenance
1. **Monthly Updates**: Regular framework updates
2. **Security Audits**: Check for security vulnerabilities
3. **Performance Monitoring**: Monitor rebuild times
4. **Documentation Updates**: Keep guides current

## Integration with Development Workflow

### CI/CD Integration
```yaml
# .github/workflows/container-test.yml
name: Container Test
on: [push, pull_request]

jobs:
  test-container:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Test Container Build
        uses: devcontainers/ci@v0.3
        with:
          runCmd: |
            node --version
            mpv --version
            npm test
```

### Development Scripts
```bash
# scripts/dev.sh restart
restart() {
    echo "🔄 Restarting development environment..."
    ./scripts/dev.sh stop-all
    ./scripts/dev.sh cleanup
    echo "📋 Please restart container manually:"
    echo "   VS Code → Command Palette → 'Dev Containers: Rebuild and Reopen in Container'"
}
```

## Support

### Getting Help
1. **Check Logs**: VS Code → View → Output → Dev Containers
2. **Community**: GitHub Issues, Discord, Stack Overflow
3. **Documentation**: This guide and related docs
4. **Team**: Ask team members for assistance

### Useful Commands
```bash
# Check container status
docker ps

# View container logs
docker logs <container_id>

# Check dev container extension
code --list-extensions | grep devcontainer

# Reset VS Code settings
rm -rf ~/.vscode-server
```

---

**📝 Note**: This guide should be updated whenever the container configuration changes or new restart procedures are added.

**🔄 Last Updated**: $(date)
**👤 Maintained by**: Development Team 