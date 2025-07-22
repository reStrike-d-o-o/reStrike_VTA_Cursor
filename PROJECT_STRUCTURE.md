# Project Structure

## Overview (Updated: 2025-01-28)

This document provides a comprehensive overview of the reStrike VTA project structure, including the Tauri v2 backend, React frontend, and development infrastructure.

## Current Status ✅

### **Real-Time Event System - COMPLETE**
- **Push-Based PSS Events**: Implemented `window.__TAURI__.event.listen` for real-time PSS event handling
- **Live Data Streaming**: Real-time log streaming with auto-scroll controls and "End" button
- **OBS Status Monitoring**: Real-time OBS connection status and recording/streaming state
- **CPU Monitoring**: Real-time system resource monitoring with push-based updates
- **Event Table Integration**: Real-time event display with proper filtering and centering

### **Window Management System - COMPLETE**
- **Dynamic Window Sizing**: Compact mode (350x1080) and fullscreen mode with custom dimensions
- **Advanced Mode Toggle**: Fullscreen + show Advanced panel when enabled, compact + hide when disabled
- **Manual Mode Toggle**: Separate dialog with "el Manuel" code validation
- **Window Persistence**: Settings saved and loaded across sessions
- **Resize Protection**: Manual window resizing disabled, only Advanced button controls

### **Authentication System - COMPLETE**
- **Password Dialog**: Modal popup for Advanced mode with "reStrike" password validation
- **Manual Mode Dialog**: Separate dialog asking for "el Manuel" with exact text match
- **Error Handling**: Clear error messages for wrong passwords/codes with cancel option
- **State Management**: Authentication state managed in Zustand store
- **Security**: Proper authentication flow with session management

### **Project Organization - COMPLETE**
- **Tauri v2 Migration**: Successfully migrated to Tauri v2 architecture with proper capabilities
- **Atomic Design**: Complete frontend component hierarchy with reusable components
- **Plugin Architecture**: Modular backend with clear separation and comprehensive error handling
- **Tab System**: Reusable tab components with flat styling and consistent design
- **Flag Management**: Complete flag management system with 253+ IOC flags and PSS mapping

### **Code Quality & Build Optimization - COMPLETE**
- **Rust Backend**: Clean compilation with no warnings or unused imports
- **React Frontend**: Production-ready build (74.14 kB gzipped) with no errors
- **Development Logs**: Console.logs commented out for production readiness
- **Import Optimization**: All unused imports removed from both frontend and backend
- **Build Pipeline**: Both frontend and backend compile cleanly

### **Recent Major Updates (2025-01-28)**
- **Real-Time Events**: Implemented push-based event system using Tauri v2
- **Window Management**: Complete window sizing and persistence system
- **Authentication**: Password-protected Advanced mode and Manual mode dialogs
- **UI Improvements**: Centered Event Table with precise title positioning
- **Code Cleanup**: Removed unused imports and development console.logs
- **Build Optimization**: Achieved spotless builds for both frontend and backend
- **Tab System Infrastructure**: Reusable Tab and TabGroup components
- **OBS Drawer Organization**: WebSocket and Integration tabs
- **PSS Drawer Organization**: UDP Server & Protocol and Flag Management tabs
- **Flag Management System**: Complete implementation with upload, search, and mapping
- **PSS Code Mapping**: Simplified system where PSS codes = IOC codes
- **Documentation Consolidation**: Streamlined and updated documentation

## Directory Structure

```
reStrike_VTA_Cursor/
├── src-tauri/                    # Tauri v2 backend (Rust)
│   ├── src/                      # Rust source code
│   │   ├── main.rs              # Tauri app entry point
│   │   ├── lib.rs               # Library exports and plugin registration
│   │   ├── tauri_commands.rs    # Tauri command definitions (1815 lines)
│   │   ├── core/                # Core application functionality
│   │   │   ├── app.rs           # Application state and lifecycle
│   │   │   ├── config.rs        # Configuration management
│   │   │   └── state.rs         # Global state management
│   │   ├── config/              # Configuration system
│   │   │   ├── manager.rs       # Configuration manager
│   │   │   ├── types.rs         # Configuration types
│   │   │   └── mod.rs           # Configuration module
│   │   ├── logging/             # Logging system
│   │   │   ├── logger.rs        # Logging implementation
│   │   │   ├── rotation.rs      # Log rotation
│   │   │   ├── archival.rs      # Log archival
│   │   │   └── mod.rs           # Logging module
│   │   ├── plugins/             # Plugin modules
│   │   │   ├── mod.rs           # Plugin module registration
│   │   │   ├── plugin_obs.rs    # OBS WebSocket integration
│   │   │   ├── plugin_udp.rs    # UDP protocol handling
│   │   │   ├── plugin_pss.rs    # PSS protocol implementation
│   │   │   ├── plugin_playback.rs # Video playback management
│   │   │   ├── plugin_store.rs  # Data storage and persistence
│   │   │   ├── plugin_cpu_monitor.rs # System monitoring
│   │   │   └── plugin_license.rs # License management
│   │   ├── obs/                 # OBS WebSocket integration
│   │   │   ├── manager.rs       # OBS connection manager
│   │   │   ├── protocol.rs      # WebSocket protocol handling
│   │   │   └── commands.rs      # OBS command definitions
│   │   ├── pss/                 # PSS protocol implementation
│   │   │   ├── listener.rs      # UDP listener
│   │   │   ├── protocol.rs      # PSS protocol parsing
│   │   │   └── events.rs        # Event handling
│   │   ├── video/               # Video management
│   │   │   ├── player.rs        # Video player integration
│   │   │   ├── clips.rs         # Clip management
│   │   │   └── overlay.rs       # Video overlay system
│   │   ├── types/               # Shared types
│   │   │   └── mod.rs           # Type definitions
│   │   └── utils/               # Utility functions
│   │       └── logger.rs        # Logging utilities
│   ├── Cargo.toml               # Rust dependencies
│   ├── tauri.conf.json          # Tauri configuration
│   ├── capabilities.json        # Tauri capabilities
│   ├── build.rs                 # Build script
│   ├── config/                  # Application configuration
│   │   ├── app_config.json      # Main configuration file
│   │   └── app_config.backup.json # Configuration backup
│   ├── logs/                    # Log files and archives
│   │   ├── app.log              # Application logs
│   │   ├── obs.log              # OBS WebSocket logs
│   │   ├── pss.log              # PSS protocol logs
│   │   ├── udp.log              # UDP server logs
│   │   └── archives/            # Compressed log archives
│   └── icons/                   # Application icons
│       └── icon.ico             # Windows icon
├── ui/                          # React frontend
│   ├── src/                     # React source code
│   │   ├── App.tsx              # Main application component
│   │   ├── index.tsx            # React entry point
│   │   ├── index.css            # Global styles
│   │   ├── components/          # Atomic design components
│   │   │   ├── atoms/           # Basic UI elements
│   │   │   │   ├── Button.tsx   # Button component
│   │   │   │   ├── Input.tsx    # Input component
│   │   │   │   ├── Checkbox.tsx # Checkbox component
│   │   │   │   ├── Label.tsx    # Label component
│   │   │   │   ├── StatusDot.tsx # Status indicator
│   │   │   │   ├── Icon.tsx     # Icon component
│   │   │   │   ├── Tab.tsx      # Tab component
│   │   │   │   └── TabGroup.tsx # Tab group component
│   │   │   ├── molecules/       # Compound components
│   │   │   │   ├── EventTableSection.tsx # Event table section
│   │   │   │   ├── LiveDataPanel.tsx # Live data display
│   │   │   │   ├── CpuMonitoringSection.tsx # CPU monitoring
│   │   │   │   ├── LogDownloadList.tsx # Log download interface
│   │   │   │   ├── FlagManagementPanel.tsx # Flag management interface
│   │   │   │   ├── PasswordDialog.tsx # Authentication dialog
│   │   │   │   ├── ManualModeDialog.tsx # Manual mode dialog
│   │   │   │   ├── PssDrawer.tsx # PSS drawer with tabs
│   │   │   │   └── ObsDrawer.tsx # OBS drawer with tabs
│   │   │   ├── organisms/       # Complex components
│   │   │   │   ├── EventTable.tsx # Event table organism
│   │   │   │   ├── MatchInfoSection.tsx # Match information
│   │   │   │   ├── ObsWebSocketManager.tsx # OBS manager
│   │   │   │   ├── SidebarSmall.tsx # Small sidebar
│   │   │   │   └── SidebarBig.tsx # Large sidebar
│   │   │   └── layouts/         # Layout components
│   │   │       ├── DockBar.tsx  # Main sidebar layout
│   │   │       ├── AdvancedPanel.tsx # Advanced panel layout
│   │   │       └── StatusbarAdvanced.tsx # Status bar layout
│   │   ├── hooks/               # Custom React hooks
│   │   │   ├── useEnvironment.ts # Environment detection
│   │   │   ├── useEnvironmentApi.ts # API environment
│   │   │   ├── useEnvironmentObs.ts # OBS environment
│   │   │   ├── usePssEvents.ts  # Real-time PSS event handling
│   │   │   └── useLiveDataEvents.ts # Live data streaming
│   │   ├── stores/              # State management
│   │   │   ├── index.ts         # Store exports
│   │   │   ├── liveDataStore.ts # Live data state
│   │   │   ├── obsStore.ts      # OBS state management
│   │   │   └── pssMatchStore.ts # PSS match state
│   │   ├── types/               # TypeScript types
│   │   │   ├── index.ts         # Type exports
│   │   │   └── tauri.d.ts       # Tauri type definitions
│   │   ├── utils/               # Utility functions
│   │   │   ├── flagUtils.tsx    # Flag utility functions
│   │   │   ├── obsUtils.ts      # OBS utility functions
│   │   │   ├── tauriCommands.ts # Tauri command utilities
│   │   │   ├── videoUtils.ts    # Video utility functions
│   │   │   ├── pssEventHandler.ts # PSS event handling
│   │   │   └── countryCodeMapping.ts # PSS to IOC mapping
│   │   ├── config/              # Frontend configuration
│   │   │   └── environments/    # Environment configurations
│   │   │       ├── web.ts       # Web environment
│   │   │       └── windows.ts   # Windows environment
│   │   └── lib/                 # Library utilities
│   │       └── index.ts         # Library exports
│   ├── public/                  # Static assets
│   │   ├── index.html           # HTML template
│   │   └── assets/              # Static assets
│   │       ├── flags/           # 253+ IOC country flag images
│   │       │   ├── AFG.png      # Afghanistan flag
│   │       │   ├── AUS.png      # Australia flag
│   │       │   ├── USA.png      # United States flag
│   │       │   └── ...          # 250+ more flag images
│   │       └── img/             # Other images
│   │           └── logo.png     # Application logo
│   ├── package.json             # Node.js dependencies
│   ├── package-lock.json        # Dependency lock file
│   ├── tsconfig.json            # TypeScript configuration
│   ├── tailwind.config.js       # Tailwind CSS configuration
│   ├── postcss.config.js        # PostCSS configuration
│   └── eslint.config.js         # ESLint configuration
├── docs/                        # Project documentation
│   ├── README.md                # Documentation overview
│   ├── ARCHITECTURE.md          # System architecture
│   ├── DEVELOPMENT.md           # Development guidelines
│   ├── OBS_INTEGRATION.md       # OBS integration guide
│   ├── FLAG_MANAGEMENT_SYSTEM.md # Flag management system
│   ├── api/                     # API documentation
│   │   └── obs-websocket.md     # OBS WebSocket API
│   ├── development/             # Development guides
│   │   ├── AI_AGENT_WINDOWS_GUIDE.md # AI agent guide
│   │   ├── WINDOWS_VSCODE_SETUP_GUIDE.md # VS Code setup
│   │   ├── development-management.md # Development management
│   │   ├── documentation-maintenance-guide.md # Documentation guide
│   │   ├── flag-images-guide.md # Flag images guide
│   │   ├── sidebar-filter-implementation.md # Sidebar implementation
│   │   └── checklists/          # Development checklists
│   ├── integration/             # Integration guides
│   │   ├── obs-dual-protocol.md # OBS protocol guide
│   │   └── obs-websocket-config.md # OBS WebSocket config
│   ├── project/                 # Project management
│   │   ├── project-management-summary.md # Project management
│   │   ├── github-integration-status.md # GitHub integration
│   │   ├── github-integration-guide.md # GitHub guide
│   │   ├── github-automation-setup.md # GitHub automation
│   │   ├── github-board-setup-instructions.md # GitHub board setup
│   │   ├── automation-quick-setup.md # Automation setup
│   │   ├── FLAG_MANAGEMENT_SPECIFICATION.md # Flag management spec
│   │   └── FLAG_MANAGEMENT_MODULE_PLAN.md # Flag management plan
│   ├── requirements/            # Requirements documentation
│   │   ├── instant-video-replay-prd.md # Video replay PRD
│   │   ├── ui-design-document.md # UI design document
│   │   ├── software-requirements.md # Software requirements
│   │   └── FLAG_MANAGEMENT_MODULE.md # Flag management module
│   └── testing/                 # Testing documentation
├── scripts/                     # Development scripts
│   ├── README.md                # Scripts overview
│   ├── development/             # Development scripts
│   │   ├── cleanup-dev-environment.sh # Environment cleanup
│   │   ├── dev.sh               # Development server
│   │   ├── fast-dev.sh          # Fast development server
│   │   ├── install-mpv-latest.sh # MPV installation
│   │   ├── manage-dev-resources.py # Resource management
│   │   ├── update-frameworks.sh # Framework updates
│   │   ├── verify-ports.sh      # Port verification
│   │   └── windows-fast-setup.ps1 # Windows setup
│   ├── github/                  # GitHub automation
│   │   ├── README.md            # GitHub scripts overview
│   │   ├── create-issues.py     # Issue creation
│   │   └── setup-project-board.py # Project board setup
│   ├── media/                   # Media processing scripts
│   │   ├── download_official_ioc_flags.py # IOC flag download
│   │   ├── download-flags.py    # Flag download
│   │   ├── enhanced-flag-recognition.py # Flag recognition
│   │   ├── flag-recognition.py  # Flag recognition
│   │   ├── generate-clip.sh     # Clip generation
│   │   ├── ioc_flag_database.json # IOC flag database
│   │   ├── ioc-flag-database.py # Flag database script
│   │   ├── europe_html_sample.html # Sample HTML
│   │   ├── north_america_html_sample.html # Sample HTML
│   │   ├── sovereign_states_html.html # Sample HTML
│   │   └── simple-enhanced-recognition.py # Enhanced recognition
│   ├── obs/                     # OBS integration scripts
│   │   └── setup-obs-websocket.sh # OBS WebSocket setup
│   ├── project/                 # Project management scripts
│   │   ├── project-tracker.py   # Project tracking
│   │   └── update-issues-after-checkpoint.sh # Issue updates
│   └── workflows/               # CI/CD workflows
│       └── ci.yml               # Continuous integration
├── protocol/                    # Protocol definitions
│   ├── pss_schema.txt           # PSS protocol schema
│   └── pss_v2.3.json            # PSS protocol v2.3 specification
├── config/                      # Global configuration
│   └── dev_resources.json       # Development resources
├── log/                         # Global logs
├── target/                      # Build artifacts
├── PROJECT_CONTEXT.md           # Project context and overview
├── PROJECT_STRUCTURE.md         # This file
├── FRONTEND_DEVELOPMENT_SUMMARY.md # Frontend development summary
├── LIBRARY_STRUCTURE.md         # Backend library structure
├── ui-design-document.md        # UI design specifications
├── package.json                 # Root package.json
├── package-lock.json            # Root package lock
├── LICENSE                      # Project license
└── README.md                    # Project README
```

## Component Architecture

### **Frontend Components (Atomic Design)**

#### **Atoms (Basic UI Elements)**
- **Button**: Primary, secondary, and icon buttons with consistent styling
- **Input**: Text inputs with validation and error states
- **Checkbox**: Boolean selection with proper accessibility
- **Label**: Form labels and text display components
- **StatusDot**: Status indicators with color-coded states
- **Icon**: SVG icon system with consistent sizing
- **Tab**: Individual tab component with flat styling
- **TabGroup**: Tab container component with consistent design

#### **Molecules (Compound Components)**
- **EventTableSection**: Event table with filtering and display
- **LiveDataPanel**: Real-time data display panels with streaming controls
- **CpuMonitoringSection**: System monitoring interface
- **LogDownloadList**: Log file download and management
- **FlagManagementPanel**: Complete flag management interface
- **PasswordDialog**: Authentication dialog for Advanced mode
- **ManualModeDialog**: Manual mode dialog with code validation
- **PssDrawer**: PSS drawer with UDP Server & Protocol and Flag Management tabs
- **ObsDrawer**: OBS drawer with WebSocket and Integration tabs

#### **Organisms (Complex Components)**
- **EventTable**: Complex event table with real-time updates
- **MatchInfoSection**: Match information display with flag integration
- **ObsWebSocketManager**: OBS connection management interface
- **SidebarSmall**: Small sidebar with controls and status
- **SidebarBig**: Large sidebar with player info and match details

#### **Layouts (Page and Section Layouts)**
- **DockBar**: Main sidebar layout with two-column design
- **AdvancedPanel**: Advanced panel layout with sidebar and main content
- **StatusbarAdvanced**: Status bar layout with real-time indicators

### **Backend Architecture (Plugin System)**

#### **Core Application Layer**
- **App**: Main application state and lifecycle management
- **Config**: Configuration management and persistence
- **State**: Global state management across plugins

#### **Plugin Modules**
- **OBS Plugin**: WebSocket integration with OBS Studio
- **UDP Plugin**: UDP protocol handling for PSS events
- **PSS Plugin**: PSS protocol v2.3 implementation
- **Playback Plugin**: Video playback and clip management
- **Store Plugin**: Data persistence and storage
- **CPU Monitor Plugin**: System resource monitoring
- **License Plugin**: License management and validation

#### **Protocol Implementations**
- **OBS Manager**: WebSocket connection management
- **PSS Listener**: UDP listener for PSS protocol
- **Video Player**: MPV-based video player integration

## Development Workflow

### **Starting Development**
```bash
# From project root
cd src-tauri
cargo tauri dev
```

This single command:
1. Starts the React development server (port 3000)
2. Builds the Rust backend
3. Launches the native Windows application
4. Enables hot reload for both frontend and backend

### **Alternative Manual Start**
```bash
# Terminal 1: Start React dev server
cd ui
npm run start:fast

# Terminal 2: Start Tauri app
cd src-tauri
cargo tauri dev
```

### **Build for Production**
```bash
cd src-tauri
cargo tauri build
```

## Key Features

### **Core Functionality**
- **Instant Video Replay**: Quick access to recent video clips
- **Real-Time Event Tracking**: Push-based PSS event capture and analysis
- **OBS Integration**: Seamless connection with OBS Studio and status monitoring
- **Flag Management**: Country flag recognition and display with 253+ IOC flags
- **Advanced Panel**: Comprehensive settings and diagnostics with tabbed interface
- **Window Management**: Dynamic window sizing with authentication protection

### **UI Components**
- **DockBar**: Main sidebar with player info, controls, and authentication
- **Event Table**: Real-time event display with filtering and centered layout
- **Advanced Panel**: Settings, diagnostics, and configuration with organized tabs
- **Status Indicators**: Real-time system status display
- **Tab System**: Reusable tab components with flat styling
- **Flag Management Panel**: Complete flag management interface
- **Authentication Dialogs**: Password and manual mode protection

### **Technical Features**
- **Environment Detection**: Automatic Tauri vs Web mode detection
- **Plugin Architecture**: Modular backend design
- **Error Handling**: Comprehensive error management
- **Hot Reload**: Development efficiency with live updates
- **Type Safety**: Full TypeScript and Rust type safety
- **Flag System**: IOC flag integration with PSS code mapping
- **Real-Time Events**: Push-based event system using Tauri v2
- **Window Management**: Dynamic sizing with authentication

## Configuration Management

### **Tauri Configuration**
- **Global Tauri API**: Enabled for frontend access
- **Development Server**: React dev server integration
- **Build Configuration**: Optimized for Windows
- **Security**: Proper allowlist configuration with capabilities
- **Event System**: Real-time event listening and emission

### **Frontend Configuration**
- **Environment Detection**: Smart Tauri API detection
- **Development Scripts**: Optimized for Windows development
- **Build Process**: Integrated with Tauri build system
- **State Management**: Zustand stores for UI state

## Development Guidelines

### **Code Quality**
- **Type Safety**: Full TypeScript and Rust type safety
- **Error Handling**: Comprehensive error management
- **Documentation**: Inline documentation and external docs
- **Testing**: Unit and integration testing

### **Architecture Principles**
- **Modularity**: Plugin-based backend architecture
- **Atomic Design**: Organized frontend component hierarchy
- **Separation of Concerns**: Clear frontend/backend separation
- **Performance**: Optimized for real-time operations
- **Real-Time Events**: Push-based event system

## Documentation

### **Key Documents**
- `PROJECT_CONTEXT.md`: Project context and overview
- `FRONTEND_DEVELOPMENT_SUMMARY.md`: Frontend architecture details
- `LIBRARY_STRUCTURE.md`: Backend architecture and plugin system
- `ui-design-document.md`: UI design specifications
- `docs/`: Comprehensive project documentation

### **Development Guides**
- `docs/development/`: Development setup and guidelines
- `docs/api/`: API documentation
- `docs/integration/`: Integration guides

## Next Steps

### **Immediate Priorities**
1. **OBS Integration**: Complete WebSocket protocol implementation ✅
2. **Event System**: Implement PSS protocol event handling ✅
3. **Video Player**: Integrate mpv video player
4. **Flag Management**: Complete flag recognition system ✅

### **Future Enhancements**
1. **AI Integration**: Automated event analysis
2. **Advanced Analytics**: Statistical analysis and reporting
3. **Multi-language Support**: Internationalization
4. **Plugin System**: Extensible plugin architecture

## Troubleshooting

### **Common Issues**
- **Port Conflicts**: Use cleanup scripts to free ports
- **Build Errors**: Clean build artifacts and rebuild
- **Tauri API Issues**: Verify environment detection
- **Hot Reload**: Ensure proper development server setup
- **Event System**: Check Tauri capabilities configuration

### **Development Environment**
- **Windows 10/11**: Primary development platform
- **Windows Native**: Direct Windows development environment
- **VS Code**: Recommended IDE with extensions
- **Git**: Version control and collaboration

---

**Last Updated**: 2025-01-28  
**Status**: Complete project structure with real-time event system and comprehensive UI features  
**Focus**: Maintainable, scalable architecture with clear organization and real-time capabilities 