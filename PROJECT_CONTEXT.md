# Project Context

## Overview
reStrike VTA is a Windows-only native desktop application designed for instant video replay and analysis in sports broadcasting. Built with Tauri v2 (Rust backend) and React (frontend), the application provides real-time event tracking, OBS Studio integration, and advanced video playback capabilities.

## Current Status ✅ (Updated: 2025-01-28)

### Tauri v2 Migration Complete
- **Native Windows Mode**: Successfully running as native Windows desktop application
- **Project Structure**: Reorganized to follow Tauri v2 conventions with `src-tauri/` directory
- **Environment Detection**: Automatic detection of Tauri API availability
- **Hot Reload**: Development mode with live reload for both frontend and backend
- **Command System**: Complete Tauri command registration and invocation working

### Frontend Architecture - COMPLETE
- **Atomic Design**: Fully implemented component hierarchy (atoms, molecules, organisms, layouts)
- **TypeScript**: Complete type safety and IntelliSense support
- **Tailwind CSS**: Utility-first styling with custom design system
- **Responsive Design**: Adaptive layouts for different screen sizes
- **Environment Detection**: Smart detection of Tauri vs Web mode
- **Tab System**: Reusable Tab and TabGroup components with flat styling
- **Flag Management**: Complete flag management system with 253+ IOC flags

### Backend Architecture
- **Plugin System**: Modular architecture with separate plugins for different features
- **OBS Integration**: WebSocket protocol support for OBS Studio
- **PSS Protocol**: UDP-based event handling system
- **Video Integration**: mpv-based video player support
- **Error Handling**: Comprehensive error handling with AppResult<T> and AppError types

### Recent Major Updates (2025-01-28)
- **Tab System Infrastructure**: Reusable Tab and TabGroup components with flat styling
- **OBS Drawer Organization**: WebSocket and Integration tabs with consistent design
- **PSS Drawer Organization**: UDP Server & Protocol and Flag Management tabs
- **Complete Flag Management**: Full implementation with upload, search, and mapping features
- **PSS Code Mapping**: Simplified system where PSS codes = IOC codes
- **Flag Integration**: All systems using flags from `ui/public/assets/flags/`
- **User Experience**: Clear, intuitive interface for flag management

## Technical Stack

### Frontend
- **React 18**: Modern React with hooks and functional components
- **TypeScript**: Full type safety
- **Tailwind CSS**: Utility-first CSS framework
- **Tauri API**: Native desktop integration
- **Atomic Design**: Organized component architecture
- **Zustand**: State management for UI components

### Backend
- **Rust**: Systems programming language for performance and safety
- **Tauri v2**: Modern desktop application framework
- **WebSocket**: Real-time communication with OBS Studio
- **UDP**: PSS protocol implementation
- **mpv**: Video player integration

## Development Workflow

### Starting Development
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

### Alternative Manual Start
```bash
# Terminal 1: Start React dev server
cd ui
npm run start:fast

# Terminal 2: Start Tauri app
cd src-tauri
cargo tauri dev
```

### Build for Production
```bash
cd src-tauri
cargo tauri build
```

## Key Features

### Core Functionality
- **Instant Video Replay**: Quick access to recent video clips
- **Event Tracking**: Real-time event capture and analysis
- **OBS Integration**: Seamless connection with OBS Studio
- **Flag Management**: Country flag recognition and display with 253+ IOC flags
- **Advanced Panel**: Comprehensive settings and diagnostics with tabbed interface

### UI Components
- **DockBar**: Main sidebar with player info and controls
- **Event Table**: Real-time event display with filtering
- **Advanced Panel**: Settings, diagnostics, and configuration with organized tabs
- **Status Indicators**: Real-time system status display
- **Tab System**: Reusable tab components with flat styling
- **Flag Management Panel**: Complete flag management interface

### Technical Features
- **Environment Detection**: Automatic Tauri vs Web mode detection
- **Plugin Architecture**: Modular backend design
- **Error Handling**: Comprehensive error management
- **Hot Reload**: Development efficiency with live updates
- **Type Safety**: Full TypeScript and Rust type safety
- **Flag System**: IOC flag integration with PSS code mapping

## Project Organization

### Directory Structure
```
reStrike_VTA_Cursor/
├── src-tauri/                    # Tauri v2 backend (Rust)
│   ├── src/                      # Rust source code
│   │   ├── main.rs              # Tauri app entry point
│   │   ├── tauri_commands.rs    # Tauri command definitions
│   │   ├── plugins/             # Plugin modules
│   │   ├── obs/                 # OBS WebSocket integration
│   │   ├── pss/                 # PSS protocol handling
│   │   └── video/               # Video player integration
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
├── ui/                          # React frontend
│   ├── src/components/          # Atomic design components
│   │   ├── atoms/               # Basic UI elements (Button, Input, etc.)
│   │   ├── molecules/           # Compound components
│   │   ├── organisms/           # Complex components
│   │   └── layouts/             # Layout components
│   ├── src/hooks/               # Custom React hooks
│   ├── src/utils/               # Utility functions
│   └── public/assets/flags/     # 253+ IOC country flag images
├── docs/                        # Project documentation
└── scripts/                     # Development scripts
```

### Component Architecture
- **Atoms**: Basic UI components (Button, Input, Checkbox, Icon, StatusDot, etc.)
- **Molecules**: Composite components (EventTable, LogToggleGroup, etc.)
- **Organisms**: Complex UI sections (DockBar, AdvancedPanel, etc.)
- **Layouts**: Page and section layouts with tabbed interfaces

## Environment Detection

The application automatically detects its running environment:

### Native Mode (Tauri)
- `window.__TAURI__` is available
- Full access to Tauri commands
- Native Windows desktop application
- File system access and system integration

### Web Mode (Browser)
- Running in web browser
- Limited functionality (no file system access)
- Fallback UI for development/testing

## Configuration

### Tauri Configuration
- **Global Tauri API**: Enabled for frontend access
- **Development Server**: React dev server integration
- **Build Configuration**: Optimized for Windows
- **Security**: Proper allowlist configuration

### Frontend Configuration
- **Environment Detection**: Smart Tauri API detection
- **Development Scripts**: Optimized for Windows development
- **Build Process**: Integrated with Tauri build system

## Development Guidelines

### Code Quality
- **Type Safety**: Full TypeScript and Rust type safety
- **Error Handling**: Comprehensive error management
- **Documentation**: Inline documentation and external docs
- **Testing**: Unit and integration testing

### Architecture Principles
- **Modularity**: Plugin-based backend architecture
- **Atomic Design**: Organized frontend component hierarchy
- **Separation of Concerns**: Clear frontend/backend separation
- **Performance**: Optimized for real-time operations

## Documentation

### Key Documents
- `PROJECT_STRUCTURE.md`: Detailed project organization
- `FRONTEND_DEVELOPMENT_SUMMARY.md`: Frontend architecture details
- `LIBRARY_STRUCTURE.md`: Backend architecture and plugin system
- `ui-design-document.md`: UI design specifications
- `docs/`: Comprehensive project documentation

### Development Guides
- `docs/development/`: Development setup and guidelines
- `docs/api/`: API documentation
- `docs/integration/`: Integration guides

## Next Steps

### Immediate Priorities
1. **OBS Integration**: Complete WebSocket protocol implementation
2. **Event System**: Implement PSS protocol event handling
3. **Video Player**: Integrate mpv video player
4. **Flag Management**: Complete flag recognition system ✅

### Future Enhancements
1. **AI Integration**: Automated event analysis
2. **Advanced Analytics**: Statistical analysis and reporting
3. **Multi-language Support**: Internationalization
4. **Plugin System**: Extensible plugin architecture

## Troubleshooting

### Common Issues
- **Port Conflicts**: Use cleanup scripts to free ports
- **Build Errors**: Clean build artifacts and rebuild
- **Tauri API Issues**: Verify environment detection
- **Hot Reload**: Ensure proper development server setup

### Development Environment
- **Windows 10/11**: Primary development platform
- **Windows Native**: Direct Windows development environment
- **VS Code**: Recommended IDE with extensions
- **Git**: Version control and collaboration 