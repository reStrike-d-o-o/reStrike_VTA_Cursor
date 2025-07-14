# Framework Updates - reStrike VTA

## Overview
This document tracks the framework and dependency updates made to the reStrike VTA project to ensure we're using the latest stable versions.

## Recent Updates (Latest)

### Node.js Update
- **From:** v18.20.8
- **To:** v24.4.0 (latest LTS)
- **Reason:** Node.js 18 is approaching end-of-life, v24 provides better performance and security
- **Impact:** All Node.js dependencies and tooling will use the latest features

### mpv Media Player Update
- **From:** v0.32.0 (2020 build)
- **To:** Latest stable version from PPA
- **Reason:** Significant improvements in video playback, codec support, and performance
- **Source:** [mpv.io/installation](https://mpv.io/installation/)

### React and Frontend Dependencies
- **React:** Updated to v18.3.1 (latest stable)
- **TypeScript:** Updated to v5.4.3 (latest stable)
- **Tailwind CSS:** Updated to v3.4.1 (latest stable)
- **Framer Motion:** Updated to v11.10.16 (latest stable)
- **ESLint:** Updated to v8.57.0 (latest stable)

### Backend Dependencies
- **Rust:** Already using latest stable (1.88.0)
- **Tauri CLI:** Will be updated when container rebuilds
- **OBS WebSocket:** Updated to v5.0.3 (latest stable)

## Configuration Changes

### Dev Container Updates
```json
// .devcontainer/devcontainer.json
{
  "features": {
    "ghcr.io/devcontainers/features/node:1": {
      "version": "24"  // Updated from "18"
    }
  }
}
```

### Dockerfile Updates
```dockerfile
# .devcontainer/Dockerfile
# Added PPA repository for latest mpv
RUN add-apt-repository ppa:mpv-player/mpv-stable
```

### Package.json Updates
```json
// ui/package.json
{
  "dependencies": {
    "react": "^18.3.1",           // Updated from "^18.0.0"
    "react-dom": "^18.3.1",       // Updated from "^18.0.0"
    "tailwindcss": "^3.4.1",      // Updated from "^4.1.11"
    "framer-motion": "^11.10.16"  // Updated from "^12.23.3"
  },
  "devDependencies": {
    "typescript": "^5.4.3",       // Updated from "^4.9.5"
    "react-scripts": "^5.0.1"     // Fixed from "^0.0.0"
  }
}
```

## Benefits of Updates

### Node.js v24 Benefits
- **Performance:** Up to 20% faster startup and execution
- **Security:** Latest security patches and improvements
- **Features:** Latest ECMAScript features and Node.js APIs
- **Long-term Support:** Supported until April 2027

### mpv Latest Version Benefits
- **Codec Support:** Latest video and audio codec support
- **Performance:** Improved playback performance
- **Features:** Latest mpv features and improvements
- **Bug Fixes:** Critical bug fixes and stability improvements

### React 18.3.1 Benefits
- **Stability:** Latest stable release with bug fixes
- **Performance:** Improved rendering performance
- **Compatibility:** Better TypeScript integration

## Migration Steps

### For Developers
1. **Rebuild Dev Container:**
   ```bash
   # VS Code Command Palette → "Dev Containers: Rebuild and Reopen in Container"
   ```

2. **Update Dependencies:**
   ```bash
   npm install
   cd ui && npm install
   ```

3. **Verify Updates:**
   ```bash
   node --version  # Should show v24+
   mpv --version   # Should show latest version
   npm outdated    # Should show minimal outdated packages
   ```

### For CI/CD
- GitHub Actions already updated to use Node.js 24
- All CI workflows will automatically use the new versions

## Testing Checklist

After applying updates, verify:

- [ ] React frontend starts without errors
- [ ] Tauri backend compiles and runs
- [ ] mpv video playback works correctly
- [ ] All existing functionality still works
- [ ] No new security vulnerabilities introduced
- [ ] Performance is maintained or improved

## Rollback Plan

If issues arise:

1. **Node.js:** Revert to v18 in `.devcontainer/devcontainer.json`
2. **mpv:** Remove PPA and reinstall system version
3. **Dependencies:** Use `npm install --save-exact` with previous versions
4. **Container:** Rebuild with previous configuration

## Future Update Schedule

- **Node.js:** Update to next LTS version when available
- **mpv:** Update when new stable releases are available
- **React:** Update to React 19 when stable (expected 2024)
- **Dependencies:** Monthly security updates, quarterly feature updates

## Resources

- [Node.js Downloads](https://nodejs.org/)
- [mpv Installation Guide](https://mpv.io/installation/)
- [React Release Notes](https://react.dev/blog)
- [Tauri Documentation](https://tauri.app/docs/)

---

*Last Updated: $(date)*
*Updated by: AI Assistant*
*Environment: Dev Container* 