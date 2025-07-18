# Project Structure Documentation

## Overview
reStrike VTA is a Windows-native desktop application built with Tauri v2, featuring a Rust backend and React frontend. The application provides comprehensive OBS integration, video replay management, and real-time event processing for taekwondo competitions.

## Technology Stack

### Backend (Rust + Tauri v2)
- **Framework**: Tauri v2 for native Windows integration
- **Language**: Rust with async/await support
- **Architecture**: Plugin-based microkernel architecture
- **Database**: SQLite for event storage and configuration
- **WebSocket**: tokio-tungstenite for OBS integration
- **Logging**: Structured logging with file rotation

### Frontend (React + TypeScript)
- **Framework**: React 18 with TypeScript
- **Styling**: Tailwind CSS with atomic design
- **State Management**: Zustand for global state
- **Build System**: Vite with Tauri integration
- **Components**: Atomic design pattern (atoms, molecules, organisms, layouts)

## Directory Structure

```
reStrike_VTA_Cursor/
├── src-tauri/                    # Tauri v2 backend (Rust)
│   ├── src/
│   │   ├── main.rs              # Tauri app entry point
│   │   ├── lib.rs               # Library exports and initialization
│   │   ├── tauri_commands.rs    # Tauri command definitions
│   │   ├── core/                # Core application logic
│   │   │   ├── app.rs           # Main application class
│   │   │   └── mod.rs           # Core module exports
│   │   ├── config/              # Configuration management system
│   │   │   ├── mod.rs           # Configuration module exports
│   │   │   ├── types.rs         # Configuration data structures
│   │   │   └── manager.rs       # Configuration manager implementation
│   │   ├── plugins/             # Plugin modules
│   │   │   ├── mod.rs           # Plugin module exports
│   │   │   ├── plugin_obs.rs    # OBS WebSocket integration
│   │   │   ├── plugin_playback.rs # Video playback management
│   │   │   ├── plugin_udp.rs    # UDP/PSS protocol handling
│   │   │   ├── plugin_store.rs  # Event storage and database
│   │   │   └── plugin_license.rs # License management
│   │   ├── logging/             # Logging system
│   │   │   ├── mod.rs           # Logging module exports
│   │   │   └── manager.rs       # Log manager implementation
│   │   ├── types/               # Shared types and error handling
│   │   │   ├── mod.rs           # Types module exports
│   │   │   └── errors.rs        # Error types and handling
│   │   └── commands/            # Legacy command handlers
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
├── ui/                          # React frontend
│   ├── src/
│   │   ├── components/          # Atomic design components
│   │   │   ├── atoms/           # Basic UI components
│   │   │   │   ├── Button.tsx   # Reusable button component
│   │   │   │   ├── Input.tsx    # Form input component
│   │   │   │   ├── Checkbox.tsx # Checkbox component
│   │   │   │   ├── Label.tsx    # Form label component
│   │   │   │   ├── StatusDot.tsx # Status indicator component
│   │   │   │   ├── Icon.tsx     # Icon component
│   │   │   │   └── README.md    # Atoms documentation
│   │   │   ├── molecules/       # Composite components
│   │   │   │   ├── EventTableSection.tsx # Event table section
│   │   │   │   ├── LiveDataPanel.tsx    # Live data display
│   │   │   │   ├── LogDownloadList.tsx  # Log download management
│   │   │   │   ├── LogToggleGroup.tsx   # Log toggle controls
│   │   │   │   └── WebSocketManager.tsx # OBS connection management
│   │   │   ├── organisms/       # Complex UI sections
│   │   │   │   ├── EventTable.tsx       # Main event table
│   │   │   │   ├── MatchInfoSection.tsx # Match information display
│   │   │   │   ├── ObsWebSocketManager.tsx # OBS connection management
│   │   │   │   ├── PlayerInfoSection.tsx # Player information display
│   │   │   │   ├── Settings.tsx         # Settings panel
│   │   │   │   ├── SidebarBig.tsx       # Main sidebar
│   │   │   │   ├── SidebarSmall.tsx     # Compact sidebar
│   │   │   │   ├── StatusBar.tsx        # Status bar
│   │   │   │   └── VideoClips.tsx       # Video clip management
│   │   │   └── layouts/         # Page and section layouts
│   │   │       ├── AdvancedPanel.tsx    # Advanced settings panel
│   │   │       ├── DockBar.tsx          # Main sidebar layout
│   │   │       ├── StatusbarAdvanced.tsx # Advanced status bar
│   │   │       ├── StatusbarDock.tsx    # Status bar for dock
│   │   │       └── TaskBar.tsx          # Task bar layout
│   │   ├── hooks/               # Custom React hooks
│   │   │   ├── useEnvironment.ts # Environment detection
│   │   │   ├── useEnvironmentApi.ts # API environment hooks
│   │   │   └── useEnvironmentObs.ts # OBS environment hooks
│   │   ├── stores/              # State management
│   │   │   └── index.ts         # Zustand store definitions
│   │   ├── utils/               # Utility functions
│   │   │   ├── flagUtils.tsx    # Flag management utilities
│   │   │   ├── obsUtils.ts      # OBS integration utilities
│   │   │   ├── tauriCommands.ts # Tauri command wrappers
│   │   │   └── videoUtils.ts    # Video processing utilities
│   │   ├── types/               # TypeScript type definitions
│   │   │   ├── index.ts         # Main type definitions
│   │   │   └── tauri.d.ts       # Tauri-specific types
│   │   ├── config/              # Environment configuration
│   │   │   └── environments/    # Environment-specific configs
│   │   │       ├── web.ts       # Web environment config
│   │   │       └── windows.ts   # Windows environment config
│   │   ├── lib/                 # Library exports
│   │   │   └── index.ts         # Main library exports
│   │   ├── App.tsx              # Main application component
│   │   ├── index.tsx            # Application entry point
│   │   └── index.css            # Global styles
│   ├── public/                  # Static assets
│   │   ├── assets/
│   │   │   └── flags/           # IOC flag images (253 PNGs)
│   │   └── index.html           # HTML template
│   ├── package.json             # Frontend dependencies
│   ├── tailwind.config.js       # Tailwind CSS configuration
│   ├── tsconfig.json            # TypeScript configuration
│   └── vite.config.ts           # Vite build configuration
├── docs/                        # Project documentation
│   ├── api/                     # API documentation
│   │   └── obs-websocket.md     # OBS WebSocket API docs
│   ├── development/             # Development guides
│   │   ├── AI_AGENT_WINDOWS_GUIDE.md # AI agent development guide
│   │   ├── checklists/          # Development checklists
│   │   ├── container-restart.md # Container restart guide
│   │   ├── development-management.md # Development management
│   │   ├── documentation-maintenance-guide.md # Doc maintenance
│   │   ├── flag-images-guide.md # Flag image management
│   │   ├── framework-update-summary.md # Framework updates
│   │   ├── framework-updates.md # Framework update details
│   │   ├── port-forwarding.md   # Port forwarding guide
│   │   ├── sidebar-filter-implementation.md # Sidebar filters
│   │   └── WINDOWS_VSCODE_SETUP_GUIDE.md # VS Code setup
│   ├── integration/             # Integration guides
│   │   ├── obs-dual-protocol.md # OBS dual protocol support
│   │   └── obs-websocket-config.md # OBS WebSocket configuration
│   ├── project/                 # Project management
│   │   ├── automation-quick-setup.md # Automation setup
│   │   ├── FLAG_MANAGEMENT_MODULE_PLAN.md # Flag management plan
│   │   ├── FLAG_MANAGEMENT_SPECIFICATION.md # Flag management spec
│   │   ├── github-automation-setup.md # GitHub automation
│   │   ├── github-board-setup-instructions.md # GitHub board setup
│   │   ├── github-integration-guide.md # GitHub integration
│   │   ├── github-integration-status.md # GitHub integration status
│   │   ├── project-management-summary.md # Project management
│   │   └── tracker-quick-reference.md # Tracker reference
│   ├── requirements/            # Requirements documentation
│   │   ├── FLAG_MANAGEMENT_MODULE.md # Flag management requirements
│   │   ├── instant-video-replay-prd.md # Video replay PRD
│   │   ├── software-requirements.md # Software requirements
│   │   └── ui-design-document.md # UI design document
│   ├── testing/                 # Testing documentation
│   │   └── core-testing-report.md # Core testing report
│   ├── README.md                # Documentation navigation
│   ├── PROJECT_CONTEXT.md       # Project context and overview
│   ├── PROJECT_STRUCTURE.md     # This file
│   ├── LIBRARY_STRUCTURE.md     # Library architecture
│   ├── FRONTEND_DEVELOPMENT_SUMMARY.md # Frontend development summary
│   ├── FLAG_MANAGEMENT_SYSTEM.md # Flag management system
│   ├── PERFORMANCE_OPTIMIZATION.md # Performance optimization
│   └── DOCKER_HOT_RELOAD_SETUP.md # Docker hot reload setup
├── scripts/                     # Development and utility scripts
│   ├── development/             # Development scripts
│   │   ├── cleanup-dev-environment.sh # Environment cleanup
│   │   ├── dev.sh               # Development server
│   │   ├── fast-dev.sh          # Fast development server
│   │   ├── install-mpv-latest.sh # MPV installation
│   │   ├── manage-dev-resources.py # Dev resource management
│   │   ├── update-frameworks.sh # Framework updates
│   │   ├── verify-ports.sh      # Port verification
│   │   └── windows-fast-setup.ps1 # Windows fast setup
│   ├── github/                  # GitHub automation scripts
│   │   ├── create-issues.py     # Issue creation
│   │   ├── README.md            # GitHub scripts README
│   │   └── setup-project-board.py # Project board setup
│   ├── media/                   # Media processing scripts
│   │   ├── download_official_ioc_flags.py # IOC flag download
│   │   ├── download-flags.py    # Flag download utility
│   │   ├── enhanced-flag-recognition.py # Enhanced flag recognition
│   │   ├── flag-recognition.py  # Flag recognition
│   │   ├── generate-clip.sh     # Clip generation
│   │   ├── ioc_flag_database.json # IOC flag database
│   │   ├── ioc-flag-database.py # IOC flag database script
│   │   ├── simple-enhanced-recognition.py # Simple recognition
│   │   └── various HTML samples # Flag recognition samples
│   ├── obs/                     # OBS integration scripts
│   │   └── setup-obs-websocket.sh # OBS WebSocket setup
│   ├── project/                 # Project management scripts
│   │   ├── project-tracker.py   # Project tracking
│   │   └── update-issues-after-checkpoint.sh # Issue updates
│   ├── workflows/               # CI/CD workflows
│   │   └── ci.yml               # Continuous integration
│   └── README.md                # Scripts documentation
├── config/                      # Configuration files
│   ├── dev_resources.json       # Development resources config
│   ├── app_config.json          # Main application configuration
│   └── app_config.backup.json   # Configuration backup
├── protocol/                    # Protocol definitions
│   └── pss_schema.txt           # PSS protocol schema
├── log/                         # Application logs
├── .cursor/                     # Cursor IDE configuration
│   └── rules/                   # Cursor rules
│       └── context.mdc          # Project context and conventions
├── Cargo.toml                   # Root Rust configuration
├── package.json                 # Root package configuration
├── README.md                    # Main project README
├── LICENSE                      # Project license
└── reStrike_VTA_Cursor.code-workspace # VS Code workspace

```

## Key Architectural Components

### Configuration Management System
- **Location**: `src-tauri/src/config/`
- **Purpose**: Comprehensive settings persistence across app sessions
- **Features**:
  - Automatic backup and restore
  - Cross-session persistence
  - Import/export functionality
  - Configuration statistics
  - Thread-safe operations

### Plugin Architecture
- **Location**: `src-tauri/src/plugins/`
- **Purpose**: Modular functionality with clear separation of concerns
- **Plugins**:
  - **OBS Plugin**: WebSocket integration for OBS Studio
  - **UDP Plugin**: PSS protocol handling and event processing
  - **Playback Plugin**: Video clip management and playback
  - **Store Plugin**: Event storage and database operations
  - **License Plugin**: License validation and management

### Atomic Design System
- **Location**: `ui/src/components/`
- **Purpose**: Scalable and maintainable UI component architecture
- **Levels**:
  - **Atoms**: Basic UI components (Button, Input, etc.)
  - **Molecules**: Composite components (EventTableSection, etc.)
  - **Organisms**: Complex UI sections (EventTable, Settings, etc.)
  - **Layouts**: Page and section layouts (DockBar, AdvancedPanel, etc.)

### State Management
- **Frontend**: Zustand for global state management
- **Backend**: tokio broadcast channels for inter-plugin communication
- **Configuration**: Persistent configuration with automatic sync

## Development Workflow

### Starting Development
```bash
# From project root - starts both frontend and backend
cd src-tauri
cargo tauri dev
```

### Manual Development
```bash
# Terminal 1: Start React dev server
cd ui
npm run start:fast

# Terminal 2: Start Tauri app
cd src-tauri
cargo tauri dev
```

### Build Commands
```bash
# Development build
cd ui
npm run build

# Production build with Tauri
cd src-tauri
cargo tauri build
```

## Configuration System

### Configuration Segments
1. **App Settings**: Version, startup behavior, performance
2. **OBS Settings**: Connections, defaults, behavior
3. **UDP Settings**: Listener config, PSS protocol, events
4. **Logging Settings**: Global, subsystems, files, live data
5. **UI Settings**: Overlay, theme, layout, animations
6. **Video Settings**: Player, replay, clips
7. **License Settings**: Keys, validation, expiration
8. **Flag Settings**: Storage, recognition, display
9. **Advanced Settings**: Development, network, security, experimental

### Configuration Features
- **Auto-save**: Settings automatically saved to `config/app_config.json`
- **Backup system**: Automatic backup to `config/app_config.backup.json`
- **Cross-session**: All settings persist between app restarts
- **Sync**: Frontend and backend stay synchronized
- **Statistics**: File sizes, connection counts, last save time
- **Import/Export**: Full config backup and restore
- **Backup/Restore**: Automatic backup with manual restore

## Environment Detection

The application automatically detects whether it's running in native Windows mode or web mode:

- **Native Mode**: Tauri API available, full desktop functionality
- **Web Mode**: Running in browser, limited functionality for development/testing

## Performance Optimizations

### Frontend
- Disable source maps in development
- Use Fast Refresh and React.memo
- Disable StrictMode in development
- Optimize imports and bundle size
- Use fast build/dev scripts

### Backend (Rust)
- Use optimized dev profile
- Enable incremental compilation
- Use fast dev scripts
- Clean build artifacts regularly

## Testing and Quality Assurance

### Testing Strategy
- Unit tests for Rust backend components
- Integration tests for plugin interactions
- Frontend component testing
- End-to-end testing for critical workflows

### Code Quality
- Rust clippy for code quality
- TypeScript strict mode
- ESLint for JavaScript/TypeScript
- Prettier for code formatting

## Deployment

### Windows Distribution
- Native Windows .exe installer
- MSI package for enterprise deployment
- Portable executable option
- Auto-update system

### Development Distribution
- Development builds with hot reload
- Debug builds with full logging
- Release builds optimized for performance

## Maintenance and Updates

### Regular Maintenance
- Monthly structure reviews
- Dependency updates
- Performance monitoring
- Security updates

### Documentation Updates
- Keep navigation indexes current
- Update after major changes
- Maintain single source of truth
- Regular documentation reviews

---

*Last updated: 2025-01-28*
*Configuration system implementation: Complete*
*OBS WebSocket management: Complete*
*Atomic design system: Complete* 