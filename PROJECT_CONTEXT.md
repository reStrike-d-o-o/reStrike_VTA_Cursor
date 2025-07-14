# reStrike VTA Project - Master Context Document

> **Last Updated**: January 27, 2025  
> **Version**: 2.2  
> **Purpose**: Complete project context for AI assistance and development continuity

## 📋 Project Overview

### Mission Statement
**reStrike VTA** (Video Tracking Assistant) is a **native Windows desktop application** designed specifically for taekwondo referees, enabling rapid video review and AI-assisted data analysis during live competitions. The system integrates with taekwondo PSS (Point Scoring System) via UDP, controls OBS Studio over WebSocket, and manages local video playback using mpv.

### Primary Use Case
- **Users**: Taekwondo referees during live competitions
- **Platform**: **Windows 10/11 (Primary Target)**
- **Context**: Fast-paced match environments requiring decisions within 20 seconds
- **Goal**: Resolve match challenges instantly with automated video replay systems

### **🖥️ Windows-First Architecture**
- **Primary Platform**: Windows 10/11 (64-bit)
- **Framework**: Tauri (Rust + React) for native Windows performance
- **Distribution**: Windows executable (.exe) with installer
- **Integration**: Native Windows APIs, OBS Studio, mpv media player
- **Development**: Cross-platform capable but Windows-optimized

## 🏗️ Architecture Overview

### Technology Stack
- **Platform**: **Windows 10/11 Desktop Application**
- **Backend**: Rust with Tauri framework (native Windows performance)
- **Frontend**: React 18 with TypeScript 5.4.3 (embedded WebView)
- **State Management**: Zustand
- **Styling**: Tailwind CSS v3.4.17
- **Animations**: Framer Motion v11.10.16
- **Media Playback**: mpv (Windows build)
- **Development**: Node.js v24.4.0, Docker dev containers

### Core Components
1. **Windows Native App** - Tauri-based desktop application
2. **UDP Server** - Listens to PSS system datagrams (Windows networking)
3. **OBS WebSocket Client** - Controls OBS Studio (Windows version)
4. **Video Playback Engine** - mpv integration for instant replay (Windows build)
5. **React Frontend** - Professional UI with overlay system (embedded)
6. **Licensing System** - Hardware-tied activation system (Windows-specific)

### Port Configuration
- **3000**: React development server (embedded in Windows app)
- **1420**: Tauri backend server (Windows executable)
- **6000**: UDP PSS protocol listener (Windows networking)
- **4455**: OBS WebSocket connection (Windows OBS)
- **8080**: Development/debug server (Docker dev container)

## 🎯 Current Development Status

### ✅ Completed Components

#### Windows Desktop Application Framework
- **Tauri Configuration**: Windows-optimized build settings
- **Native Performance**: Rust backend with React frontend
- **Windows Integration**: Ready for Windows-specific features
- **Executable Generation**: .exe build pipeline configured

#### Frontend (React/TypeScript - Embedded in Windows App)
- **Modern UI Framework**: Complete React 18 implementation with TypeScript
- **State Management**: Zustand store with comprehensive type safety
- **Overlay System**: 5-position overlay with themes (dark, light, transparent)
- **Video Clips Manager**: Grid-based clip management with search/filter
- **OBS WebSocket Manager**: Dual protocol support (v4/v5) with real-time status
- **Settings Panel**: Tabbed interface for all configuration options
- **Responsive Design**: Optimized for Windows desktop screens
- **Keyboard Shortcuts**: Power user navigation (Ctrl+1-4, Space, F, arrows)

#### Backend (Rust/Tauri - Windows Native)
- **Plugin Architecture**: Modular plugin system
- **OBS Integration**: Comprehensive WebSocket implementation (Windows OBS)
- **UDP Protocol**: Basic server structure (Windows networking)
- **License System**: Hardware-based activation framework (Windows hardware IDs)
- **Video Playback**: mpv integration hooks (Windows mpv build)
- **Command Handlers**: Frontend-backend communication bridge

#### Development Infrastructure
- **Project Organization**: Complete reorganization with logical structure
- **Documentation**: Comprehensive docs organized by category
- **Scripts**: Automated development, build, and deployment scripts
- **Project Tracking**: GitHub Issues-based tracker with templates
- **Container Setup**: Dev container with all dependencies

### 🚧 In Progress / Next Steps

#### Immediate Priorities (High) - Windows Focus
1. **Windows Build Pipeline**: Complete .exe generation and testing
2. **Complete UDP Plugin**: Finish PSS protocol implementation (Windows networking)
3. **Video Playback Integration**: Complete mpv integration with Windows-specific optimizations
4. **OBS Scene Control**: Implement scene switching and recording commands (Windows OBS)
5. **Windows Installer**: Create MSI installer for distribution
6. **Testing Framework**: Unit and integration tests on Windows

#### Medium Term (1-2 months) - Windows Platform
1. **Windows-Specific Features**: Utilize Windows APIs for enhanced functionality
2. **Performance Optimization**: Windows-specific optimizations for video processing
3. **Hardware Integration**: Windows hardware detection and management
4. **Auto-updater**: Windows application update system
5. **Security Hardening**: Windows security best practices

#### Long Term (3+ months) - Platform Expansion
1. **Cross-Platform Support**: Extend to macOS and Linux (secondary priority)
2. **Advanced Analytics**: Machine learning analysis features
3. **Cloud Integration**: Remote clip storage and sharing
4. **Mobile Companion**: Windows Phone/Android companion app
5. **Tournament Management**: Full tournament workflow support

## 📁 Project Structure

```
reStrike_VTA_Cursor/
├── 📁 .devcontainer/           # Container configuration
│   ├── devcontainer.json      # Node.js v24 config
│   ├── Dockerfile            # mpv PPA setup
│   └── README-devcontainer.md
├── 📁 .github/                # GitHub configuration
│   ├── ISSUE_TEMPLATE/        # Feature/bug templates
│   ├── workflows/ci.yml       # CI/CD pipeline
│   └── dependabot.yml         # Dependency updates
├── 📁 docs/                   # Organized documentation
│   ├── 📁 api/               # API documentation
│   ├── 📁 development/       # Dev guides & checklists
│   ├── 📁 integration/       # OBS & external systems
│   ├── 📁 project/           # Management & tracking
│   ├── 📁 requirements/      # PRD & specifications
│   └── README.md             # Documentation index
├── 📁 scripts/               # Automation scripts
│   ├── 📁 development/       # Dev environment management
│   ├── 📁 obs/              # OBS integration scripts
│   ├── 📁 project/          # Project tracking tools
│   ├── 📁 media/            # Video processing
│   └── 📁 workflows/        # CI/CD workflows
├── 📁 src/                  # Rust backend
│   ├── 📁 plugins/          # Modular plugin system
│   │   ├── plugin_obs.rs    # OBS WebSocket implementation
│   │   ├── plugin_udp.rs    # PSS protocol handler
│   │   ├── plugin_playback.rs # Video playback engine
│   │   ├── plugin_license.rs # Licensing system
│   │   └── plugin_store.rs  # Data storage
│   ├── 📁 commands/         # Tauri command handlers
│   └── main.rs              # Application entry point
├── 📁 ui/                   # React frontend
│   ├── 📁 src/              # React source code
│   │   ├── 📁 components/   # UI components
│   │   │   ├── ObsWebSocketManager.tsx
│   │   │   ├── Overlay.tsx
│   │   │   ├── VideoClips.tsx
│   │   │   └── Settings.tsx
│   │   ├── 📁 stores/       # Zustand state management
│   │   ├── App.tsx          # Main application
│   │   └── index.tsx        # React entry point
│   ├── package.json         # Frontend dependencies
│   ├── postcss.config.js    # Tailwind CSS v3 config
│   └── tailwind.config.js   # Tailwind configuration
├── 📁 config/               # Configuration files
├── 📁 protocol/             # PSS protocol definitions
├── package.json             # Root project configuration
├── Cargo.toml               # Rust dependencies
└── README.md                # Project overview
```

## 🔧 Framework Updates & Dependencies

### Windows Development Environment
- **Primary OS**: Windows 10/11 for development and testing
- **Dev Container**: Docker-based development (WSL2 on Windows)
- **Target Platform**: Windows 10/11 (64-bit)
- **Build Output**: Native Windows executable (.exe)

### Recently Updated (January 2025)
- **Node.js**: v18.20.8 → v24.4.0 (LTS, Windows optimized)
- **React**: v18.3.1 (latest stable, embedded in Windows app)
- **TypeScript**: v5.4.3 (from v4.9.5)
- **Tailwind CSS**: v3.4.17 (fixed PostCSS configuration)
- **Framer Motion**: v11.10.16 (latest)
- **mpv**: v0.32.0 → latest (Windows build configured)
- **Tauri**: v1.x (Windows desktop application framework)

### Current Versions
- **Rust**: 1.88.0 (stable)
- **Cargo**: 1.88.0
- **OBS WebSocket**: v5.0.3 support
- **Zustand**: Latest (state management)
- **ESLint**: v8.57.0

### Container Status
- **Current**: Node.js v18.20.8, mpv v0.32.0
- **Pending Rebuild**: Node.js v24.4.0, latest mpv
- **Required Action**: VS Code → Command Palette → "Dev Containers: Rebuild and Reopen in Container"

## 📋 Recent Development History

### Phase 1: Project Foundation (Completed)
- Created Tauri + React architecture
- Implemented basic UI framework
- Set up development environment
- Created project documentation structure

### Phase 2: Frontend Development (Completed)
- Built comprehensive React UI with TypeScript
- Implemented Zustand state management
- Created responsive overlay system
- Added video clips management
- Built OBS WebSocket manager component
- Added settings panel with live preview

### Phase 3: Backend Foundation (Completed)
- Created modular plugin architecture
- Implemented OBS WebSocket plugin (dual protocol)
- Set up Tauri command handlers
- Created license system framework
- Established UDP server structure

### Phase 4: Infrastructure & Organization (Completed)
- Complete project reorganization
- Comprehensive documentation system
- Automated development scripts
- GitHub Issues project tracking
- Container optimization

### Phase 5: Framework Updates (Completed)
- Node.js v24 upgrade configuration
- Frontend dependency updates
- Tailwind CSS v3 compatibility fix
- mpv installation automation
- Security audit and fixes

### Phase 6: Core Implementation (Current)
- **In Progress**: UDP protocol completion
- **In Progress**: Video playback integration
- **Next**: OBS scene control implementation
- **Next**: Testing framework setup

## 🛠️ Development Workflow

### Windows Development
```bash
# Development on Windows or WSL2
./scripts/development/dev.sh status

# Start Windows-targeted development
cd ui && npm start  # React on port 3000 (embedded preview)
npm run tauri dev   # Windows desktop app development

# Build Windows executable
npm run tauri build # Generates .exe for Windows
```

### Windows Testing
```bash
# Test Windows-specific features
./scripts/development/test-windows-features.sh

# Test OBS integration (requires Windows OBS)
./scripts/obs/test-obs-integration.sh

# Verify Windows hardware detection
./scripts/development/test-hardware-detection.sh
```

## 🎯 Feature Categories & Priorities

### Windows Desktop Core (High Priority)
- **Native Windows App**: Tauri-based desktop application
- **Windows APIs**: Integration with Windows-specific features
- **Performance**: Native Windows performance optimization
- **Hardware Detection**: Windows hardware identification for licensing

### Windows-Specific Integration (High Priority)
- **OBS Studio Windows**: Windows version integration
- **mpv Windows Build**: Windows-optimized video playback
- **PSS Protocol**: UDP server optimized for Windows networking
- **File System**: Windows file system integration

### Cross-Platform Considerations (Medium Priority)
- **Future macOS Support**: Potential macOS version
- **Future Linux Support**: Potential Linux version
- **Code Portability**: Maintain cross-platform compatibility where possible

## 🚨 Known Issues & Resolutions

### Windows-Specific Considerations
1. **Windows Build Pipeline**: Need to complete Windows executable generation
2. **OBS Studio Integration**: Requires Windows version of OBS Studio
3. **mpv Integration**: Need Windows-specific mpv build integration
4. **Hardware Licensing**: Windows hardware ID detection implementation

### Recently Resolved
1. **Tailwind CSS PostCSS Error**: Fixed by downgrading to v3.4.17 and updating PostCSS config
2. **TypeScript Version Conflicts**: Resolved by using compatible versions with react-scripts
3. **Container Port Forwarding**: Fixed with comprehensive port configuration
4. **Project Organization**: Completed full reorganization for maintainability

### Current Issues
1. **Tauri CLI Missing**: Needs `cargo install tauri-cli`
2. **Security Vulnerabilities**: 9 npm vulnerabilities need fixing
3. **Container Rebuild Needed**: Node.js v24 and mpv updates pending

### Prevention Strategies
- Regular dependency audits
- Automated testing pipeline
- Documentation maintenance
- Version compatibility checking

## 📚 Documentation System

### Organization
- **API**: OBS WebSocket API reference
- **Development**: Guides, checklists, framework updates
- **Integration**: External system integration guides
- **Project**: Management, tracking, and coordination
- **Requirements**: PRD, specifications, design documents

### Key Documents
- **[PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md)**: Detailed structure guide
- **[INSTANT_VIDEO_REPLAY_PRD.md](docs/requirements/instant-video-replay-prd.md)**: Product requirements
- **[PROJECT_TRACKER_GUIDE.md](docs/project/project-tracker-guide.md)**: GitHub Issues tracking
- **[DEV-CONTAINER-CHECKLIST.md](docs/development/checklists/DEV-CONTAINER-CHECKLIST.md)**: Environment verification

### Maintenance
- Monthly documentation reviews
- Automated link checking
- Version updates tracking
- Cross-reference maintenance

## 🔮 Future Vision

### Short Term (1-3 months) - Windows Focus
- Complete Windows desktop application
- Native Windows performance optimization
- Windows-specific hardware integration
- Professional Windows installer

### Medium Term (3-6 months) - Platform Enhancement
- Windows-specific advanced features
- Enhanced Windows OBS integration
- Windows hardware-based licensing
- Windows auto-update system

### Long Term (6+ months) - Platform Expansion
- Cross-platform support (macOS, Linux)
- Mobile companion apps
- Cloud-based tournament management
- Global deployment and distribution

## 🎉 Success Metrics

### Windows Application Metrics
- **Installation Success**: > 95% successful Windows installations
- **Startup Time**: < 3 seconds on Windows 10/11
- **Memory Usage**: < 500MB RAM on Windows
- **Performance**: 60fps UI, < 100ms latency on Windows

### Platform-Specific Metrics
- **Windows Compatibility**: 100% Windows 10/11 compatibility
- **OBS Integration**: Seamless Windows OBS Studio integration
- **Hardware Detection**: 99%+ Windows hardware identification
- **Video Performance**: Smooth playback on Windows systems

---

## 📞 Getting Help

### Documentation
- Start with [docs/README.md](docs/README.md) for navigation
- Quick reference in [docs/project/tracker-quick-reference.md](docs/project/tracker-quick-reference.md)
- Detailed guides in respective documentation categories

### Scripts & Tools
- All scripts have `--help` flags
- Main wrapper: `./scripts/development/dev.sh help`
- Project tracker: `python3 scripts/project/project-tracker.py help`

### Community & Support
- GitHub Issues for bug reports and features
- GitHub Discussions for questions and ideas
- Documentation for comprehensive guides

---

**📋 This document serves as the single source of truth for the reStrike VTA Windows desktop application project context and should be referenced by all AI assistants and team members for consistent understanding of project status, architecture, and goals.**

**🖥️ Platform Focus**: This is primarily a Windows desktop application with potential future cross-platform expansion.**

**🔄 Maintenance**: Update this document when major milestones are reached, architecture changes, or new phases begin.**

**👥 Audience**: AI assistants, developers, project managers, and stakeholders. 