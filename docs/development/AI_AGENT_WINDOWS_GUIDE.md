# AI Agent Windows Development Quick Reference

> **Note:** All architecture, onboarding, and coding conventions are defined in .cursor/rules/context.mdc (single source of truth). Project is Windows-only; Docker/devcontainer is fully removed. All onboarding, build, and documentation reference Windows-native setup only.

## 🎯 **Quick Setup for AI Agents**

This guide provides essential information for AI agents working on the reStrike VTA project in a Windows environment.

---

## 📋 **Project Overview**

### **Technology Stack**
- **Frontend**: React 18 + TypeScript + Tailwind CSS + Framer Motion
- **Backend**: Rust + Tauri framework
- **State Management**: Zustand
- **Video**: mpv with hardware acceleration
- **Real-time**: WebSocket (OBS), UDP (PSS)
- **Flags**: 253 IOC flags with React integration

### **Key Components**
- **6 React Components**: SidebarTest, Overlay, VideoClips, ObsWebSocketManager, Settings, EnvironmentTest
- **3 Rust Plugins**: plugin_udp.rs, plugin_obs.rs, plugin_playback.rs
- **Environment System**: Dual environment (Web/Windows) with automatic detection

---

## 🚀 **Essential Commands**

### **Development Start**
```powershell
# Full Tauri app (recommended for Windows development)
npm run start:tauri

# React frontend only
cd ui && npm start

# Rust backend only
cargo run
```

### **Build Commands**
```powershell
# Build for Windows
npm run build:windows

# Build for web
npm run build:web

# Production build
cargo tauri build
```

### **Testing**
```powershell
# Frontend tests
cd ui && npm test

# Backend tests
cargo test

# Type checking
cargo check
```

---

## 🔧 **Critical Configuration**

### **Environment Variables**
```powershell
# Set for Windows development
$env:REACT_APP_ENVIRONMENT = "windows"

# Set for web development
$env:REACT_APP_ENVIRONMENT = "web"
```

### **OBS WebSocket Configuration**
⚠️ **CRITICAL**: OBS WebSocket authentication must be disabled
- Open OBS Studio
- Go to Tools > WebSocket Server Settings
- **Uncheck "Enable Authentication"**
- Set port to 4455 (default)

### **Port Configuration**
- **3000**: React development server
- **1420**: Tauri backend
- **4455**: OBS WebSocket
- **6000**: UDP PSS protocol

---

## 📁 **Key Files and Directories**

### **Frontend (React)**
```
ui/src/
├── components/
│   ├── SidebarTest.tsx      # Main sidebar with event table
│   ├── ObsWebSocketManager.tsx  # OBS integration
│   ├── Overlay.tsx          # Video overlay system
│   ├── VideoClips.tsx       # Clip management
│   ├── Settings.tsx         # Configuration
│   └── EnvironmentTest.tsx  # Environment testing
├── hooks/
│   └── useEnvironment.ts    # Environment-aware hooks
├── config/
│   └── environment.ts       # Environment detection
├── stores/
│   └── index.ts             # Zustand state management
└── utils/
    ├── logger.ts            # Comprehensive logging
    └── flagUtils.tsx        # Flag management (253 IOC flags)
```

### **Backend (Rust)**
```
src/
├── plugins/
│   ├── plugin_udp.rs        # PSS protocol (640 lines)
│   ├── plugin_obs.rs        # OBS WebSocket (455 lines)
│   └── plugin_playback.rs   # mpv integration (568 lines)
├── commands/
│   └── tauri_commands.rs    # Tauri command handlers
├── utils/
│   └── logger.rs            # Rust logging system
└── main.rs                  # Application entry point
```

---

## 🌐 **Environment System**

### **Dual Environment Support**
The project supports both **Web** and **Windows** environments with automatic detection:

```typescript
// Environment detection
const { environment, isWindows, isWeb } = useEnvironment();

// Environment-aware API calls
const { apiCall } = useEnvironmentApi();
await apiCall('obs/status');

// Conditional rendering
<WindowsOnly><NativeFeature /></WindowsOnly>
<WebOnly><WebFeature /></WebOnly>
```

### **Environment-Specific Features**
- **Windows**: Tauri commands, native file system, system tray, OBS integration
- **Web**: Direct WebSocket, HTTP APIs, browser features, hot reload

---

## 🔍 **Common Issues and Solutions**

### **React Development Server Issues**
```powershell
# Clear cache and restart
cd ui
npm cache clean --force
rm -rf node_modules
npm install
npm start
```

### **Rust Compilation Issues**
```powershell
# Update Rust and rebuild
rustup update
cargo clean
cargo build
```

### **Tauri Build Issues**
```powershell
# Check requirements
cargo tauri doctor

# Update Tauri CLI
cargo install tauri-cli --force
```

### **OBS Connection Issues**
1. Verify OBS WebSocket authentication is disabled
2. Check port 4455 is not blocked
3. Ensure OBS Studio is running
4. Check firewall settings

---

## 📊 **Testing and Verification**

### **Component Testing**
```typescript
// Test environment detection
const { environment, isWindows, isWeb } = useEnvironment();
console.log('Environment:', environment);

// Test OBS connection
// Use ObsWebSocketManager component
// Check connection status indicators

// Test flag system
// Verify 253 IOC flags are loading
// Check fallback emoji flags
```

### **Integration Testing**
```powershell
# Test full application
npm run start:tauri

# Verify all components load
# Check OBS WebSocket connection
# Test video playback
# Verify flag display
```

---

## 🎯 **Development Workflow**

### **1. Start Development**
```powershell
# Clone repository
git clone https://github.com/reStrike-d-o-o/reStrike_VTA_Cursor.git
cd reStrike_VTA_Cursor

# Install dependencies
npm install
cd ui && npm install && cd ..

# Start development
npm run start:tauri
```

### **2. Make Changes**
- Edit React components in `ui/src/components/`
- Edit Rust plugins in `src-tauri/src/plugins/`
- Use environment-aware hooks and components
- Follow the dual environment system

### **3. Test Changes**
```powershell
# Test frontend
cd ui && npm test

# Test backend
cargo test

# Build for verification
npm run build:windows
```

### **4. Commit and Push**
```powershell
git add .
git commit -m "Description of changes"
git push origin main
```

---

## 📚 **Key Documentation**

### **Project Documentation**
- **PROJECT_CONTEXT.md**: Complete project overview and status
- **README.md**: Quick start and basic information
- **docs/development/environment-system.md**: Environment system details
- **docs/FLAG_MANAGEMENT_SYSTEM.md**: Flag system documentation

### **External Resources**
- **Tauri**: https://tauri.app/docs/
- **React**: https://react.dev/
- **Rust**: https://doc.rust-lang.org/
- **TypeScript**: https://www.typescriptlang.org/docs/

---

## ✅ **Verification Checklist**

### **Before Starting Development**
- [ ] Windows 10/11 with latest updates
- [ ] Node.js v24+ installed
- [ ] Rust stable installed
- [ ] Tauri CLI installed
- [ ] OBS Studio installed with WebSocket enabled (no auth)
- [ ] mpv player installed
- [ ] Project cloned and dependencies installed

### **Development Verification**
- [ ] `npm run start:tauri` launches successfully
- [ ] All 6 React components render without errors
- [ ] OBS WebSocket connection works
- [ ] Video playback is functional
- [ ] Flag system displays 253 IOC flags
- [ ] Environment switching works correctly
- [ ] All tests pass

### **Production Verification**
- [ ] `npm run build:windows` completes successfully
- [ ] `cargo tauri build` creates executable
- [ ] Application runs on clean Windows system
- [ ] All features work in production build

---

## 🚨 **Critical Notes for AI Agents**

### **Environment System Compliance**
- **ALWAYS** use environment-aware hooks (`useEnvironment`, `useEnvironmentApi`, `useEnvironmentObs`)
- **NEVER** use direct `console.log` - use the logging system
- **ALWAYS** guard Tauri/native code with environment checks
- **ALWAYS** use component wrappers for conditional rendering

### **OBS WebSocket Protocol**
- **V4**: Standard WebSocket with authentication handling
- **V5**: Proper Identify request without authentication field
- **CRITICAL**: Authentication must be disabled in OBS settings

### **Flag System**
- 253 IOC flags are already downloaded and integrated
- Use `FlagImage` component with automatic fallbacks
- Flags are stored in `ui/public/assets/flags/`

### **Logging System**
- Use `createComponentLogger` for component-specific logging
- Logs are written to file and console
- Automatic cleanup on startup

---

## 🎯 **Quick Commands Reference**

```powershell
# Development
npm run start:tauri          # Full Tauri app
npm run start:windows        # Windows environment
npm run start:web           # Web environment

# Building
npm run build:windows       # Windows build
npm run build:web          # Web build
cargo tauri build          # Production build

# Testing
npm test                   # Frontend tests
cargo test                 # Backend tests
cargo check                # Type checking

# Cleanup
cargo clean                # Clean Rust build
npm run clean              # Clean npm build
```

**Remember**: This is a production-ready Windows desktop application with comprehensive flag support for international competitions. Follow the environment system and use the provided logging system for all development work.