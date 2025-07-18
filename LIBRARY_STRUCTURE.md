# Library Structure

## Overview
This document describes the technical architecture and library structure of the reStrike VTA project, a Windows-only native desktop application built with Tauri v2 (Rust backend) and React (frontend).

## Current Status ✅

### Tauri v2 Migration Complete
- **Native Windows Mode**: Successfully running as native Windows desktop application
- **Project Structure**: Reorganized to follow Tauri v2 conventions with `src-tauri/` directory
- **Environment Detection**: Automatic detection of Tauri API availability
- **Hot Reload**: Development mode with live reload for both frontend and backend
- **Build System**: Integrated build process working correctly

## Backend Architecture (Rust/Tauri)

### Project Structure
```
src-tauri/
├── src/
│   ├── main.rs                 # Tauri app entry point
│   ├── lib.rs                  # Library exports and module declarations
│   ├── tauri_commands.rs       # Tauri command definitions
│   ├── core/                   # Core application logic
│   │   ├── mod.rs             # Core module exports
│   │   ├── app.rs             # Application state management
│   │   ├── config.rs          # Configuration management
│   │   └── state.rs           # Global state management
│   ├── plugins/               # Plugin modules
│   │   ├── mod.rs             # Plugin module exports
│   │   ├── plugin_obs.rs      # OBS WebSocket integration
│   │   ├── plugin_playback.rs # Video playback management
│   │   ├── plugin_store.rs    # Data storage and persistence
│   │   └── plugin_udp.rs      # UDP protocol handling
│   ├── obs/                   # OBS WebSocket integration
│   │   ├── mod.rs             # OBS module exports
│   │   ├── manager.rs         # OBS connection management
│   │   ├── protocol.rs        # WebSocket protocol implementation
│   │   └── commands.rs        # OBS command definitions
│   ├── pss/                   # PSS protocol handling
│   │   ├── mod.rs             # PSS module exports
│   │   ├── listener.rs        # UDP listener implementation
│   │   ├── protocol.rs        # PSS protocol parsing
│   │   └── events.rs          # Event type definitions
│   ├── video/                 # Video player integration
│   │   ├── mod.rs             # Video module exports
│   │   ├── player.rs          # Video player management
│   │   ├── clips.rs           # Video clip management
│   │   └── overlay.rs         # Video overlay system
│   ├── types/                 # Type definitions
│   │   └── mod.rs             # Type exports
│   ├── utils/                 # Utility functions
│   │   └── logger.rs          # Logging utilities
│   └── commands/              # Command implementations
│       └── mod.rs             # Command module exports
├── Cargo.toml                 # Rust dependencies and configuration
├── tauri.conf.json            # Tauri application configuration
├── build.rs                   # Build script
├── icons/                     # Application icons
└── gen/                       # Generated files
```

### Core Modules

#### Main Application (`main.rs`)
```rust
use tauri::Manager;
use re_strike_vta_app::tauri_commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            tauri_commands::get_app_status,
            tauri_commands::obs_get_status,
            tauri_commands::system_get_info,
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### Tauri Commands (`tauri_commands.rs`)
```rust
use crate::types::AppResult;

#[tauri::command]
pub async fn get_app_status() -> AppResult<String> {
    Ok("Application is running".to_string())
}

#[tauri::command]
pub async fn obs_get_status() -> AppResult<String> {
    // OBS status implementation
    Ok("OBS status".to_string())
}

#[tauri::command]
pub async fn system_get_info() -> AppResult<SystemInfo> {
    // System information implementation
    Ok(SystemInfo::default())
}
```

#### Plugin System (`plugins/mod.rs`)
```rust
pub mod plugin_obs;
pub mod plugin_playback;
pub mod plugin_store;
pub mod plugin_udp;

pub use plugin_obs::*;
pub use plugin_playback::*;
pub use plugin_store::*;
pub use plugin_udp::*;
```

### Error Handling

#### AppResult Type
```rust
// types/mod.rs
pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("OBS error: {0}")]
    ObsError(String),
    
    #[error("Video error: {0}")]
    VideoError(String),
    
    #[error("PSS error: {0}")]
    PssError(String),
}
```

#### Error Conversion Guidelines
- Use `AppError::IoError(e)` when converting `std::io::Error` to `AppError`
- Use `AppError::ConfigError(e.to_string())` when converting `std::io::Error` to `AppError::ConfigError`
- Use `e.to_string()` when returning errors to API responses expecting `Option<String>`

## Frontend Architecture (React/TypeScript)

### Project Structure
```
ui/
├── src/
│   ├── components/            # Atomic design components
│   │   ├── atoms/            # Basic UI components
│   │   │   ├── Button.tsx    # Reusable button component
│   │   │   ├── Input.tsx     # Form input component
│   │   │   ├── Checkbox.tsx  # Checkbox component
│   │   │   ├── Label.tsx     # Form label component
│   │   │   ├── StatusDot.tsx # Status indicator component
│   │   │   └── Icon.tsx      # Icon component
│   │   ├── molecules/        # Composite components
│   │   │   ├── EventTableSection.tsx # Event table section
│   │   │   ├── LiveDataPanel.tsx    # Live data display
│   │   │   ├── LogDownloadList.tsx  # Log download management
│   │   │   └── LogToggleGroup.tsx   # Log toggle controls
│   │   ├── organisms/        # Complex UI sections
│   │   │   ├── EventTable.tsx       # Main event table
│   │   │   ├── MatchInfoSection.tsx # Match information display
│   │   │   ├── ObsWebSocketManager.tsx # OBS connection management
│   │   │   └── PlayerInfoSection.tsx # Player information display
│   │   └── layouts/          # Page and section layouts
│   │       ├── DockBar.tsx          # Main sidebar layout
│   │       ├── AdvancedPanel.tsx    # Advanced settings panel
│   │       ├── StatusbarAdvanced.tsx # Advanced status bar
│   │       └── StatusbarDock.tsx    # Status bar for dock
│   ├── hooks/                # Custom React hooks
│   │   ├── useEnvironment.ts # Environment detection
│   │   ├── useEnvironmentApi.ts # Tauri API integration
│   │   └── useEnvironmentObs.ts # OBS WebSocket integration
│   ├── utils/                # Utility functions
│   │   ├── tauriCommands.ts  # Tauri command wrappers
│   │   ├── flagUtils.tsx     # Flag management utilities
│   │   ├── obsUtils.ts       # OBS utility functions
│   │   └── videoUtils.ts     # Video utility functions
│   ├── types/                # TypeScript type definitions
│   │   ├── index.ts          # Type exports
│   │   └── tauri.d.ts        # Tauri type definitions
│   ├── config/               # Environment configuration
│   │   └── environments/     # Environment-specific configs
│   │       ├── web.ts        # Web environment config
│   │       └── windows.ts    # Windows environment config
│   ├── stores/               # State management
│   │   └── index.ts          # Store exports
│   ├── lib/                  # Library utilities
│   │   └── index.ts          # Library exports
│   ├── App.tsx               # Main application component
│   ├── index.tsx             # React entry point
│   └── index.css             # Global styles
├── public/                   # Static assets
│   ├── assets/
│   │   └── flags/            # Country flag images
│   └── index.html            # HTML template
├── package.json              # Frontend dependencies
├── tailwind.config.js        # Tailwind CSS configuration
├── tsconfig.json             # TypeScript configuration
└── eslint.config.js          # ESLint configuration
```

### Component Architecture

#### Atomic Design Implementation
The frontend follows atomic design principles with clear separation of concerns:

**Atoms** (Basic Components)
```typescript
// ui/src/components/atoms/Button.tsx
interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  children: React.ReactNode;
  onClick?: () => void;
}

export const Button: React.FC<ButtonProps> = ({ 
  variant = 'primary', 
  size = 'md', 
  disabled = false, 
  children, 
  onClick 
}) => {
  // Component implementation
};
```

**Molecules** (Composite Components)
```typescript
// ui/src/components/molecules/EventTableSection.tsx
export const EventTableSection: React.FC = () => {
  // Combines EventTable organism with filtering controls
  // Handles event data display and interaction
  // Integrates with Tauri commands for data retrieval
};
```

**Organisms** (Complex Sections)
```typescript
// ui/src/components/organisms/DockBar.tsx
export const DockBar: React.FC = () => {
  // Main sidebar with player info and controls
  // Two-column layout: SidebarSmall and SidebarBig
  // Status indicators and navigation controls
};
```

**Layouts** (Page Structure)
```typescript
// ui/src/components/layouts/AdvancedPanel.tsx
export const AdvancedPanel: React.FC = () => {
  // Settings and configuration panel
  // Tabbed interface for different settings categories
  // Diagnostics and log management
};
```

### Environment Detection

#### Environment Hook
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

#### Tauri API Integration
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

### State Management

#### React Hooks
- **useEnvironment**: Tauri API detection
- **useEnvironmentApi**: Tauri command invocation
- **useEnvironmentObs**: OBS WebSocket integration

#### Component State
- Local state management with useState
- Context for shared state when needed
- Props for component communication

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

## Configuration

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

### Technical Features
- **Environment Detection**: Automatic Tauri vs Web mode detection
- **Plugin Architecture**: Modular backend design
- **Error Handling**: Comprehensive error management
- **Hot Reload**: Development efficiency with live updates
- **Type Safety**: Full TypeScript and Rust type safety

## Performance Optimization

### Backend Optimization
- **Async/Await**: Non-blocking I/O operations
- **Memory Management**: Efficient memory usage with Rust
- **Error Handling**: Comprehensive error management
- **Plugin System**: Modular architecture for scalability

### Frontend Optimization
- **Code Splitting**: Lazy loading for large components
- **Memoization**: React.memo for expensive components
- **Bundle Optimization**: Efficient bundling and tree shaking
- **Performance Monitoring**: Real-time performance metrics

## Testing Strategy

### Backend Testing
- **Unit Tests**: Individual function testing
- **Integration Tests**: Module interaction testing
- **Error Testing**: Comprehensive error scenario testing
- **Performance Testing**: Load and stress testing

### Frontend Testing
- **Component Testing**: Individual component testing
- **Integration Testing**: Component interaction testing
- **E2E Testing**: Complete workflow testing
- **Accessibility Testing**: WCAG compliance testing

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

## Future Enhancements

### Immediate Priorities
1. **OBS Integration**: Complete WebSocket protocol implementation
2. **Event System**: Implement PSS protocol event handling
3. **Video Player**: Integrate mpv video player
4. **Flag Management**: Complete flag recognition system

### Technical Improvements
1. **Performance**: Further optimization for large datasets
2. **Testing**: Comprehensive test coverage
3. **Documentation**: Enhanced developer documentation
4. **Accessibility**: Improved accessibility features

---

**Last Updated**: December 2024  
**Status**: ✅ Native Windows Mode - Ready for Development  
**Next Phase**: Feature Development and Enhancement 