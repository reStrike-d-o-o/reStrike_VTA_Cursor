# Project Structure

## Overview
reStrike VTA is a Windows-only native desktop application built with Tauri v2 (Rust backend) and React (frontend). The project has been reorganized to follow Tauri v2 conventions with proper native Windows mode operation.

## Directory Structure

```
reStrike_VTA_Cursor/
├── src-tauri/                    # Tauri v2 backend (Rust)
│   ├── src/                      # Rust source code
│   │   ├── main.rs              # Tauri app entry point
│   │   ├── lib.rs               # Library exports
│   │   ├── tauri_commands.rs    # Tauri command definitions
│   │   ├── core/                # Core application logic
│   │   ├── plugins/             # Plugin modules (obs, playback, store, udp)
│   │   ├── obs/                 # OBS WebSocket integration
│   │   ├── pss/                 # PSS protocol handling
│   │   ├── video/               # Video player integration
│   │   ├── types/               # Type definitions
│   │   ├── utils/               # Utility functions
│   │   └── commands/            # Command implementations
│   ├── Cargo.toml               # Rust dependencies and configuration
│   ├── tauri.conf.json          # Tauri application configuration
│   ├── build.rs                 # Build script
│   ├── icons/                   # Application icons
│   └── gen/                     # Generated files
├── ui/                          # React frontend
│   ├── src/
│   │   ├── components/          # Atomic design components
│   │   │   ├── atoms/           # Basic UI components
│   │   │   ├── molecules/       # Composite components
│   │   │   ├── organisms/       # Complex UI sections
│   │   │   └── layouts/         # Layout components
│   │   ├── hooks/               # Custom React hooks
│   │   ├── stores/              # State management
│   │   ├── types/               # TypeScript type definitions
│   │   ├── utils/               # Utility functions
│   │   └── config/              # Environment configuration
│   ├── public/                  # Static assets
│   │   └── assets/
│   │       └── flags/           # Country flag images
│   ├── package.json             # Frontend dependencies
│   └── tailwind.config.js       # Tailwind CSS configuration
├── docs/                        # Project documentation
├── scripts/                     # Development and automation scripts
├── config/                      # Configuration files
├── protocol/                    # Protocol specifications
└── package.json                 # Root package.json for npm scripts
```

## Key Features

### Native Windows Mode
- **Tauri v2 Integration**: Full native Windows desktop application
- **Hot Reload**: Development mode with live reload for both frontend and backend
- **Environment Detection**: Automatic detection of Tauri API availability
- **Command System**: Complete Tauri command registration and invocation

### Frontend Architecture
- **Atomic Design**: Organized component hierarchy (atoms, molecules, organisms, layouts)
- **TypeScript**: Full type safety and IntelliSense support
- **Tailwind CSS**: Utility-first styling with custom design system
- **Responsive Design**: Adaptive layouts for different screen sizes

### Backend Architecture
- **Plugin System**: Modular architecture with separate plugins for different features
- **OBS Integration**: WebSocket protocol support for OBS Studio
- **PSS Protocol**: UDP-based event handling system
- **Video Integration**: mpv-based video player support

## Development Workflow

### Starting Development
```bash
# From project root
cd src-tauri
cargo tauri dev
```

This command:
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

## Environment Detection

The application automatically detects whether it's running in:
- **Native Mode**: Tauri API available (`window.__TAURI__` exists)
- **Web Mode**: Running in browser without Tauri

This allows the same codebase to run in both development and production environments.

## Configuration Files

- `src-tauri/tauri.conf.json`: Tauri application configuration
- `src-tauri/Cargo.toml`: Rust dependencies and features
- `ui/package.json`: Frontend dependencies and scripts
- `ui/tailwind.config.js`: Tailwind CSS configuration
- `ui/src/config/`: Environment-specific configurations

## Documentation

- `docs/`: Comprehensive project documentation
- `PROJECT_CONTEXT.md`: High-level project overview
- `FRONTEND_DEVELOPMENT_SUMMARY.md`: Frontend architecture details
- `PROJECT_REORGANIZATION_SUMMARY.md`: Migration history and changes 