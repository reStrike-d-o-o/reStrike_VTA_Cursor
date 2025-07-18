# reStrike VTA - Taekwondo Video Replay Manager

> **Windows-only native desktop application** for taekwondo referees using Tauri and React

## 🎯 **Project Overview**

reStrike VTA is a **native Windows desktop application** designed specifically for taekwondo referees and competition officials. It provides real-time video replay capabilities, OBS integration, and competition data management through a modern, professional interface.

### **Key Features**
- 🎥 **Video Replay System** - Instant video replay with mpv integration
- 📡 **OBS Integration** - Dual WebSocket protocol (v4/v5) support
- 📊 **PSS Protocol** - Real-time UDP competition data parsing
- 🏁 **Flag Management** - 253 IOC flags with React integration
- 🖥️ **Native Windows UI** - Professional desktop interface
- 🔄 **Live Data Streaming** - Real-time competition data
- 📝 **Event Logging** - Comprehensive logging and diagnostics

## 🏗️ **Architecture**

### **Technology Stack**
- **Frontend**: React 18 + TypeScript + Tailwind CSS + Framer Motion
- **Backend**: Rust + Tauri framework
- **Video**: mpv with hardware acceleration
- **Real-time**: WebSocket (OBS), UDP (PSS)
- **Flags**: 253 IOC flags with React integration

### **Project Structure**
```
reStrike_VTA_Cursor/
├── 📁 src-tauri/              # Tauri v2 application (Rust backend)
│   ├── 📁 src/                # Rust source code
│   │   ├── 📁 plugins/        # Plugin modules (OBS, PSS, Video)
│   │   ├── 📁 core/           # Core application logic
│   │   ├── 📁 types/          # Type definitions
│   │   └── 📁 utils/          # Utility functions
│   ├── 📁 icons/              # Application icons
│   ├── 📁 gen/                # Generated schemas
│   ├── Cargo.toml             # Rust dependencies
│   └── tauri.conf.json        # Tauri configuration
├── 📁 ui/                     # React frontend
│   ├── 📁 src/                # React source code
│   │   ├── 📁 components/     # React components (atomic design)
│   │   ├── 📁 hooks/          # React hooks
│   │   ├── 📁 utils/          # Utility functions
│   │   └── 📁 types/          # TypeScript types
│   ├── 📁 public/             # Static assets
│   │   └── 📁 assets/flags/   # 253 IOC flag images
│   └── package.json           # Node.js dependencies
├── 📁 docs/                   # Project documentation
├── 📁 scripts/                # Build and utility scripts
└── package.json               # Project-level scripts
```

## 🚀 **Quick Start**

### **Prerequisites**
- **Windows 10/11** (x64)
- **Node.js 24+** (LTS)
- **Rust** (stable, MSVC toolchain)
- **OBS Studio** (with WebSocket enabled, no auth)
- **mpv player** (Windows build)

### **Installation**
```bash
# Clone repository
git clone https://github.com/reStrike-d-o-o/reStrike_VTA_Cursor.git
cd reStrike_VTA_Cursor

# Install dependencies
npm install
cd ui && npm install && cd ..

# Start development
npm start
```

### **Build Commands**
```bash
# Development
npm start                    # Start Tauri development server
npm run dev                  # Alternative development command
npm run dev:fast            # Fast development mode

# Production
npm run build               # Build for Windows
npm run build:fast          # Fast production build

# Testing
npm test                    # Frontend tests
npm run test:backend        # Backend tests

# Maintenance
npm run clean               # Clean build artifacts
npm run clean:all           # Deep clean
npm run format              # Format code
npm run lint                # Lint code
```

## 🌐 **Environment System**

The project supports **dual environment operation** - **Web** and **Windows** modes with automatic detection and environment-specific features.

### **Environment Detection**
- **Automatic**: Detects Tauri availability and environment variables
- **Manual Override**: Set `REACT_APP_ENVIRONMENT=web` or `REACT_APP_ENVIRONMENT=windows`
- **Build Scripts**: Separate scripts for each environment

### **Environment-Specific Features**

#### **Windows Environment** 🪟
- ✅ **Tauri Commands**: Native Windows API access
- ✅ **Native File System**: Direct file system operations
- ✅ **System Tray**: Windows system tray integration
- ✅ **Auto Updates**: Automatic application updates
- ✅ **OBS Integration**: Direct OBS WebSocket via Tauri
- ✅ **Hardware Access**: Direct hardware control

#### **Web Environment** 🌐
- ✅ **Direct WebSocket**: Browser-based WebSocket connections
- ✅ **HTTP API**: RESTful API communication
- ✅ **Browser APIs**: File upload/download via browser
- ✅ **Hot Reload**: Development hot reload support
- ✅ **Cross-Platform**: Works on any platform with a browser

### **Usage Examples**

```bash
# Start in web mode
npm run start:web

# Start in Windows mode  
npm run start:windows

# Build for web
npm run build:web

# Build for Windows
npm run build:windows
```

### **Environment-Aware Components**

```typescript
import { useEnvironment, EnvironmentWrapper } from './hooks/useEnvironment';

// Environment detection
const { environment, isWindows, isWeb } = useEnvironment();

// Environment-aware API calls
const { apiCall } = useEnvironmentApi();
await apiCall('obs/status');

// Conditional rendering
<WindowsOnly><NativeFeature /></WindowsOnly>
<WebOnly><WebFeature /></WebOnly>
```

## 🏁 **Flag Management System**

### **Status**: ✅ **COMPLETE**
- **253 IOC Flags**: All official International Olympic Committee flags
- **Emoji Fallback**: Automatic fallback for missing flags
- **React Integration**: `ui/src/utils/flagUtils.tsx` updated with all 253 IOC codes
- **Download Script**: Python-based Wikipedia scraper with BeautifulSoup
- **Storage**: `ui/public/assets/flags/` with 253 PNG files
- **Documentation**: Complete system documentation in `docs/FLAG_MANAGEMENT_SYSTEM.md`

### **Technical Implementation:**
- **Download Script**: Python-based Wikipedia scraper with BeautifulSoup
- **Strategy**: Prioritized Current NOCs, then downloaded from other tables only if IOC code not already present
- **Reports**: JSON and Markdown reports generated automatically
- **React Integration**: `ui/src/utils/flagUtils.tsx` updated with all 253 IOC codes
- **Fallbacks**: Emoji flags for all codes with automatic error handling
- **Storage**: `ui/public/assets/flags/` with 253 PNG files
- **Documentation**: Complete system documentation in `docs/FLAG_MANAGEMENT_SYSTEM.md`

## 🎥 **Video, OBS, and PSS Protocols**

### **Video System**
- **mpv Integration**: Hardware acceleration, advanced controls
- **Clip Management**: Extract and save video clips
- **Overlay System**: Video overlay with competition data

### **OBS Integration**
- **Dual Protocol**: WebSocket v4/v5 support
- **Connection Management**: Multiple OBS connections
- **Scene Control**: Scene switching and management
- **Recording/Streaming**: Start/stop recording and streaming

### **PSS Protocol**
- **Real-time UDP**: Competition data parsing
- **Event Types**: Match events, scoring, timing
- **Live Streaming**: Real-time data to UI components

## 🛠️ **Development**

### **Key Components**
- **6 React Components**: SidebarTest, Overlay, VideoClips, ObsWebSocketManager, Settings, EnvironmentTest
- **3 Rust Plugins**: plugin_udp.rs, plugin_obs.rs, plugin_playback.rs
- **Environment System**: Dual environment (Web/Windows) with automatic detection

### **Critical Configuration**
- **OBS WebSocket**: Authentication must be disabled, port 4455
- **Port Configuration**: 3000 (React), 1420 (Tauri), 4455 (OBS), 6000 (UDP PSS)
- **Environment Variables**: Set for Windows/web development

### **Development Workflow**
1. **Start Development**: `npm start` (runs Tauri dev server)
2. **Make Changes**: Edit React components in `ui/src/components/`
3. **Test Changes**: Use environment-aware hooks and components
4. **Build for Production**: `npm run build` (creates Windows executable)

## 📚 **Documentation**

### **Key Documentation Files**
- **PROJECT_CONTEXT.md**: Complete project overview and status
- **README.md**: Quick start and basic information
- **docs/development/environment-system.md**: Environment system details
- **docs/FLAG_MANAGEMENT_SYSTEM.md**: Flag system documentation
- **docs/LIBRARY_STRUCTURE.md**: Technical architecture details

### **External Resources**
- **Tauri**: https://tauri.app/docs/
- **React**: https://react.dev/
- **Rust**: https://doc.rust-lang.org/
- **TypeScript**: https://www.typescriptlang.org/docs/

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
- [ ] `npm start` launches successfully
- [ ] All 6 React components render without errors
- [ ] OBS WebSocket connection works
- [ ] Video playback is functional
- [ ] Flag system displays 253 IOC flags
- [ ] Environment switching works correctly
- [ ] All tests pass

### **Production Verification**
- [ ] `npm run build` completes successfully
- [ ] `cargo tauri build` creates executable
- [ ] Application runs on clean Windows system
- [ ] All features work in production build

## 🚨 **Critical Notes**

### **Windows-Only Application**
- This project is a **native Windows desktop application** (no Docker/devcontainer)
- All development, build, and deployment targets Windows 10/11 (x64)
- Production deployment is via Windows .exe and MSI installer (no containerization)

### **Environment System Compliance**
- All new components must use environment-aware hooks
- Follow the dual environment system for all features
- Test in both web and Windows environments
- Use environment wrappers for conditional rendering

### **Performance Best Practices**
- Use fast scripts for development (`npm run dev:fast`)
- Clean caches regularly (`npm run clean:all`)
- Monitor bundle size and build times
- Optimize imports and dependencies

---

**License**: MIT  
**Author**: damjanZGB  
**Repository**: https://github.com/reStrike-d-o-o/reStrike_VTA_Cursor