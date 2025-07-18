# AI Agent Windows Development Guide

## Overview
This guide provides comprehensive instructions for AI agents working on the reStrike VTA project, a Windows-only native desktop application built with Tauri v2 and React.

## Current Status ✅

### Tauri v2 Migration Complete
- **Native Windows Mode**: Successfully running as native Windows desktop application
- **Project Structure**: Reorganized to follow Tauri v2 conventions with `src-tauri/` directory
- **Environment Detection**: Automatic detection of Tauri API availability
- **Hot Reload**: Development mode with live reload for both frontend and backend
- **Build System**: Integrated build process working correctly

## Project Architecture

### Technology Stack
- **Backend**: Rust with Tauri v2 for native Windows integration
- **Frontend**: React 18 with TypeScript and Tailwind CSS
- **Design System**: Atomic design with reusable components
- **State Management**: React hooks and context
- **Build System**: Integrated Tauri build process

### Directory Structure
```
reStrike_VTA_Cursor/
├── src-tauri/                    # Tauri v2 backend (Rust)
│   ├── src/                      # Rust source code
│   │   ├── main.rs              # Tauri app entry point
│   │   ├── tauri_commands.rs    # Tauri command definitions
│   │   ├── plugins/             # Plugin modules (obs, playback, store, udp)
│   │   ├── obs/                 # OBS WebSocket integration
│   │   ├── pss/                 # PSS protocol handling
│   │   └── video/               # Video player integration
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
├── ui/                          # React frontend
│   ├── src/components/          # Atomic design components
│   │   ├── atoms/               # Basic UI components
│   │   ├── molecules/           # Composite components
│   │   ├── organisms/           # Complex UI sections
│   │   └── layouts/             # Page and section layouts
│   ├── src/hooks/               # Custom React hooks
│   ├── src/utils/               # Utility functions
│   └── public/assets/flags/     # Country flag images
├── docs/                        # Project documentation
└── scripts/                     # Development scripts
```

## Development Workflow

### Starting Development
```bash
# From project root - starts both frontend and backend
cd src-tauri
cargo tauri dev
```

This single command:
1. Starts React development server (port 3000)
2. Builds Rust backend
3. Launches native Windows application
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

### Build Commands
```bash
# Development build
cd ui
npm run build

# Production build with Tauri
cd src-tauri
cargo tauri build
```

## Environment Detection

### Tauri API Detection
The application automatically detects whether it's running in native Windows mode or web mode:

```typescript
// ui/src/hooks/useEnvironment.ts
export const useEnvironment = () => {
  const [tauriAvailable, setTauriAvailable] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const checkTauriAvailability = async () => {
      try {
        if (typeof window !== 'undefined' && window.__TAURI__) {
          await invoke('get_app_status');
          setTauriAvailable(true);
        } else {
          setTauriAvailable(false);
        }
      } catch (error) {
        console.warn('Tauri API not available:', error);
        setTauriAvailable(false);
      } finally {
        setIsLoading(false);
      }
    };

    checkTauriAvailability();
  }, []);

  return {
    tauriAvailable,
    isLoading,
    isNative: tauriAvailable,
    isWeb: !tauriAvailable && !isLoading
  };
};
```

### Environment Modes
- **Native Mode**: Tauri API available, full desktop functionality
- **Web Mode**: Running in browser, limited functionality for development/testing

## Component Architecture

### Atomic Design Implementation
The frontend follows atomic design principles:

#### Atoms (Basic Components)
- **Button**: Reusable button component with variants
- **Input**: Form input component
- **Checkbox**: Checkbox component
- **Label**: Form label component
- **StatusDot**: Status indicator component
- **Icon**: Icon component

#### Molecules (Composite Components)
- **EventTableSection**: Event table with filtering
- **LiveDataPanel**: Real-time data display
- **LogDownloadList**: Log download management
- **LogToggleGroup**: Log toggle controls

#### Organisms (Complex Sections)
- **EventTable**: Main event table
- **MatchInfoSection**: Match information display
- **ObsWebSocketManager**: OBS connection management
- **PlayerInfoSection**: Player information display

#### Layouts (Page Structure)
- **DockBar**: Main sidebar layout
- **AdvancedPanel**: Advanced settings panel
- **StatusbarAdvanced**: Advanced status bar
- **StatusbarDock**: Status bar for dock

## Tauri Integration

### Command Invocation
```typescript
// ui/src/utils/tauriCommands.ts
import { invoke } from '@tauri-apps/api/core';

export const tauriCommands = {
  getAppStatus: () => invoke('get_app_status'),
  obsGetStatus: () => invoke('obs_get_status'),
  systemGetInfo: () => invoke('system_get_info'),
  // ... other commands
};
```

### Environment Hooks
```typescript
// ui/src/hooks/useEnvironmentApi.ts
export const useEnvironmentApi = () => {
  const { tauriAvailable } = useEnvironment();
  
  const invokeCommand = useCallback(async (command: string, args?: any) => {
    if (!tauriAvailable) {
      throw new Error('Tauri API not available');
    }
    return await invoke(command, args);
  }, [tauriAvailable]);

  return { invokeCommand, tauriAvailable };
};
```

## Development Guidelines

### Code Quality Standards
- **TypeScript**: Full type safety throughout the application
- **Error Handling**: Comprehensive error management with AppResult<T> and AppError
- **Documentation**: Inline documentation and external docs
- **Testing**: Unit and integration testing

### Architecture Principles
- **Modularity**: Plugin-based backend architecture
- **Atomic Design**: Organized frontend component hierarchy
- **Separation of Concerns**: Clear frontend/backend separation
- **Performance**: Optimized for real-time operations

### File Organization
- **Components**: Follow atomic design hierarchy
- **Hooks**: Custom React hooks for shared logic
- **Utils**: Utility functions and helpers
- **Types**: TypeScript type definitions
- **Config**: Environment-specific configurations

## Troubleshooting

### Common Issues

#### Port Conflicts
```bash
# Clean up ports before starting
./scripts/development/cleanup-dev-environment.sh --cleanup
```

#### Build Errors
```bash
# Clean and rebuild
cd src-tauri
cargo clean
cargo build
```

#### Tauri API Issues
- Verify environment detection in browser console
- Check that `window.__TAURI__` is available
- Ensure Tauri commands are properly registered

#### Hot Reload Issues
- Verify React development server is running on port 3000
- Check Tauri configuration for correct dev path
- Restart both frontend and backend servers

### Development Tips
- Use React DevTools for frontend debugging
- Monitor Tauri console for backend issues
- Check browser console for frontend errors
- Verify environment detection in development

## Configuration Files

### Tauri Configuration
```json
// src-tauri/tauri.conf.json
{
  "build": {
    "beforeDevCommand": "cd ui && npm run start:fast",
    "beforeBuildCommand": "cd ui && npm run build",
    "devPath": "http://localhost:3000",
    "distDir": "../ui/dist"
  },
  "app": {
    "withGlobalTauri": true
  }
}
```

### Rust Dependencies
```toml
// src-tauri/Cargo.toml
[package]
name = "re-strike-vta-app"
version = "2.0.0"

[dependencies]
tauri = { version = "2.0.0", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[[bin]]
name = "re-strike-vta-app"
path = "src/main.rs"
```

### Frontend Configuration
```javascript
// ui/tailwind.config.js
module.exports = {
  content: ['./src/**/*.{js,jsx,ts,tsx}'],
  theme: {
    extend: {
      colors: {
        primary: '#1e40af',
        secondary: '#64748b',
        accent: '#f59e0b',
      }
    }
  },
  plugins: []
};
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

## Documentation

### Key Documents
- `PROJECT_STRUCTURE.md`: Detailed project organization
- `PROJECT_CONTEXT.md`: High-level project overview
- `FRONTEND_DEVELOPMENT_SUMMARY.md`: Frontend architecture details
- `PROJECT_REORGANIZATION_SUMMARY.md`: Migration history

### Development Guides
- `docs/development/`: Development setup and guidelines
- `docs/api/`: API documentation
- `docs/integration/`: Integration guides

## Success Metrics

### Development Metrics
- **Native Windows Mode**: ✅ Successfully running
- **Tauri v2 Integration**: ✅ Complete migration
- **Hot Reload**: ✅ Working for both frontend and backend
- **Environment Detection**: ✅ Automatic detection working
- **Build System**: ✅ Integrated build process working

### User Experience Metrics
- **Performance**: Native Windows performance
- **Responsiveness**: Smooth UI interactions
- **Accessibility**: Keyboard navigation support
- **Error Handling**: Comprehensive error management

---

**Last Updated**: December 2024  
**Status**: ✅ Native Windows Mode - Ready for Development  
**Next Phase**: Feature Development and Enhancement