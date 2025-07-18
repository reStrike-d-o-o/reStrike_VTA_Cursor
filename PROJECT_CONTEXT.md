# Project Context

## Overview
reStrike VTA is a Windows-only native desktop application designed for instant video replay and analysis in sports broadcasting. Built with Tauri v2 (Rust backend) and React (frontend), the application provides real-time event tracking, OBS Studio integration, and advanced video playback capabilities.

## Current Status ✅

### Tauri v2 Migration Complete
- **Native Windows Mode**: Successfully running as native Windows desktop application
- **Project Structure**: Reorganized to follow Tauri v2 conventions with `src-tauri/` directory
- **Environment Detection**: Automatic detection of Tauri API availability
- **Hot Reload**: Development mode with live reload for both frontend and backend
- **Command System**: Complete Tauri command registration and invocation working

### Frontend Architecture
- **Atomic Design**: Fully implemented component hierarchy (atoms, molecules, organisms, layouts)
- **TypeScript**: Complete type safety and IntelliSense support
- **Tailwind CSS**: Utility-first styling with custom design system
- **Responsive Design**: Adaptive layouts for different screen sizes
- **Environment Detection**: Smart detection of Tauri vs Web mode

### Backend Architecture
- **Plugin System**: Modular architecture with separate plugins for different features
- **OBS Integration**: WebSocket protocol support for OBS Studio
- **PSS Protocol**: UDP-based event handling system
- **Video Integration**: mpv-based video player support
- **Error Handling**: Comprehensive error handling with AppResult<T> and AppError types

## Technical Stack

### Frontend
- **React 18**: Modern React with hooks and functional components
- **TypeScript**: Full type safety
- **Tailwind CSS**: Utility-first CSS framework
- **Tauri API**: Native desktop integration
- **Atomic Design**: Organized component architecture

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
- **Flag Management**: Country flag recognition and display
- **Advanced Panel**: Comprehensive settings and diagnostics

### UI Components
- **DockBar**: Main sidebar with player info and controls
- **Event Table**: Real-time event display with filtering
- **Advanced Panel**: Settings, diagnostics, and configuration
- **Status Indicators**: Real-time system status display

### Technical Features
- **Environment Detection**: Automatic Tauri vs Web mode detection
- **Plugin Architecture**: Modular backend design
- **Error Handling**: Comprehensive error management
- **Hot Reload**: Development efficiency with live updates
- **Type Safety**: Full TypeScript and Rust type safety

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
│   ├── src/hooks/               # Custom React hooks
│   ├── src/utils/               # Utility functions
│   └── public/assets/flags/     # Country flag images
├── docs/                        # Project documentation
└── scripts/                     # Development scripts
```

### Component Architecture
- **Atoms**: Basic UI components (Button, Input, Checkbox, etc.)
- **Molecules**: Composite components (EventTable, LogToggleGroup, etc.)
- **Organisms**: Complex UI sections (DockBar, AdvancedPanel, etc.)
- **Layouts**: Page and section layouts

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
- `PROJECT_REORGANIZATION_SUMMARY.md`: Migration history
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
4. **Flag Management**: Complete flag recognition system

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
- **Docker**: Optional containerized development
- **VS Code**: Recommended IDE with extensions
- **Git**: Version control and collaboration 