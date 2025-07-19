# Project Structure Documentation

## Overview

reStrike VTA is a Tauri v2 desktop application for Taekwondo video replay management, built with Rust backend and React frontend using atomic design principles.

## Architecture

```
reStrike_VTA_Cursor/
├── src-tauri/                 # Rust backend (Tauri v2)
│   ├── src/
│   │   ├── plugins/           # Plugin system
│   │   │   ├── plugin_obs.rs      # OBS WebSocket integration
│   │   │   ├── plugin_playback.rs # Video playback management
│   │   │   ├── plugin_store.rs    # Data persistence
│   │   │   ├── plugin_udp.rs      # UDP communication
│   │   │   ├── plugin_cpu_monitor.rs # NEW: CPU monitoring system
│   │   │   └── mod.rs
│   │   ├── core/              # Core application logic
│   │   ├── commands/          # Tauri commands
│   │   ├── config/            # Configuration management
│   │   ├── logging/           # Logging system
│   │   ├── obs/               # OBS integration
│   │   ├── pss/               # PSS protocol handling
│   │   ├── video/             # Video processing
│   │   ├── types/             # Shared types
│   │   ├── utils/             # Utility functions
│   │   ├── tauri_commands.rs  # Tauri command definitions
│   │   ├── lib.rs             # Library entry point
│   │   └── main.rs            # Application entry point
│   ├── Cargo.toml             # Rust dependencies
│   └── tauri.conf.json        # Tauri configuration
├── ui/                        # React frontend
│   ├── src/
│   │   ├── components/        # Atomic design components
│   │   │   ├── atoms/             # Basic UI components
│   │   │   ├── molecules/         # Composite components
│   │   │   ├── organisms/         # Complex UI sections
│   │   │   └── layouts/           # Page layouts
│   │   ├── hooks/             # React hooks
│   │   ├── stores/            # State management
│   │   ├── types/             # TypeScript types
│   │   ├── utils/             # Utility functions
│   │   └── config/            # Environment configuration
│   ├── package.json           # Node.js dependencies
│   └── tailwind.config.js     # Tailwind CSS configuration
├── docs/                      # Documentation
├── scripts/                   # Build and utility scripts
└── config/                    # Configuration files
```

## Backend Architecture (Rust/Tauri)

### Plugin System

The backend uses a modular plugin architecture for different functionalities:

#### **plugin_cpu_monitor.rs** (NEW - 2025-01-28)
- **Purpose**: Real-time CPU and memory monitoring
- **Implementation**: Uses Windows `wmic` commands for process monitoring
- **Features**:
  - System CPU usage tracking
  - Individual process monitoring
  - Memory usage tracking
  - Background monitoring with configurable intervals
  - Process filtering (>0.1% CPU or >10MB memory)
- **Data Structures**:
  ```rust
  pub struct CpuProcessData {
      pub process_name: String,
      pub cpu_percent: f64,
      pub memory_mb: f64,
      pub last_update: chrono::DateTime<chrono::Utc>,
  }

  pub struct SystemCpuData {
      pub total_cpu_percent: f64,
      pub cores: Vec<f64>,
      pub last_update: chrono::DateTime<chrono::Utc>,
  }
  ```
- **Commands**: `cpu_get_process_data`, `cpu_get_system_data`
- **Status**: ✅ Implemented, awaiting `wmic` installation for testing

#### **plugin_obs.rs**
- **Purpose**: OBS Studio WebSocket integration
- **Features**: Connection management, scene switching, source control
- **Status**: ✅ Fully implemented

#### **plugin_playback.rs**
- **Purpose**: Video playback and replay management
- **Features**: Video player control, clip management, replay functionality
- **Status**: ✅ Implemented

#### **plugin_store.rs**
- **Purpose**: Data persistence and storage
- **Features**: SQLite database, configuration storage, event logging
- **Status**: ✅ Implemented

#### **plugin_udp.rs**
- **Purpose**: UDP communication for real-time data
- **Features**: Network communication, data streaming
- **Status**: ✅ Implemented

### Core Modules

#### **core/app.rs**
- Application state management
- Plugin initialization and lifecycle
- Global configuration

#### **tauri_commands.rs**
- Tauri command definitions
- Frontend-backend communication
- Error handling and response formatting

#### **logging/**
- Structured logging system
- Log rotation and archival
- Debug and error tracking

## Frontend Architecture (React)

### Atomic Design Implementation

#### **Atoms** (Basic Components)
- `Button.tsx` - Reusable button with variants
- `Input.tsx` - Form input component
- `Checkbox.tsx` - Checkbox component
- `Label.tsx` - Form label component
- `StatusDot.tsx` - Status indicator
- `Icon.tsx` - Icon component

#### **Molecules** (Composite Components)
- `EventTableSection.tsx` - Event table with filtering
- `LiveDataPanel.tsx` - Live data streaming controls
- `CpuMonitoringSection.tsx` - **NEW: CPU monitoring display**
  - Real-time process monitoring
  - System CPU usage display
  - Process list with CPU/memory data
  - Start/Stop monitoring controls
- `LogDownloadList.tsx` - Log file management
- `MatchInfoSection.tsx` - Match information display
- `ObsWebSocketManager.tsx` - OBS connection management

#### **Organisms** (Complex Sections)
- `EventTable.tsx` - Main event table with sorting/filtering
- `MatchInfoSection.tsx` - Match details organism
- `ObsWebSocketManager.tsx` - OBS integration organism

#### **Layouts** (Page Structure)
- `DockBar.tsx` - Main sidebar with two-column layout
- `AdvancedPanel.tsx` - Advanced settings panel
  - Live Data section
  - **CPU Monitoring section** (positioned underneath Live Data)
- `StatusbarAdvanced.tsx` - Status bar component

### State Management

#### **Zustand Stores**
- Global state management
- Real-time data synchronization
- Component communication

#### **React Hooks**
- `useEnvironment.ts` - Tauri API detection
- `useEnvironmentApi.ts` - Tauri command invocation
- `useEnvironmentObs.ts` - OBS WebSocket integration

### Styling System

#### **Tailwind CSS**
- Utility-first CSS framework
- Custom color palette for sports broadcasting
- Responsive design patterns
- Accessibility features

## Data Flow

### CPU Monitoring Flow (NEW)
```
1. Rust Plugin (plugin_cpu_monitor.rs)
   ↓ Uses wmic commands
2. Process Data Collection
   ↓ Background task (every 2 seconds)
3. Tauri Commands (tauri_commands.rs)
   ↓ JSON serialization
4. React Frontend (CpuMonitoringSection.tsx)
   ↓ Real-time updates
5. UI Display (process list, CPU usage, memory)
```

### General Data Flow
```
Rust Backend → Tauri Commands → React Frontend → UI Components
     ↓              ↓                ↓              ↓
  Plugin Logic → Command API → State Management → User Interface
```

## Development Workflow

### Environment Setup
1. **Rust Backend**: `cd src-tauri && cargo tauri dev`
2. **React Frontend**: `cd ui && npm run start:docker`
3. **Hot Reload**: Both frontend and backend support live reload

### Build Process
1. **Development**: `cargo tauri dev` (includes frontend build)
2. **Production**: `cargo tauri build` (optimized builds)

### Testing Strategy
- **Unit Tests**: Rust backend tests
- **Integration Tests**: Tauri command testing
- **E2E Tests**: Complete workflow testing
- **Component Tests**: React component testing

## Configuration Management

### Backend Configuration
- `src-tauri/config/app_config.json` - Application settings
- `src-tauri/tauri.conf.json` - Tauri framework configuration
- Environment-specific configurations

### Frontend Configuration
- `ui/src/config/environments/` - Environment-specific settings
- `ui/tailwind.config.js` - Styling configuration
- `ui/package.json` - Dependencies and scripts

## Logging and Monitoring

### Backend Logging
- Structured logging with different levels
- Log rotation and archival
- Debug information for development

### Frontend Monitoring
- Console logging for debugging
- Error boundaries for React components
- Performance monitoring

## Security Considerations

### Backend Security
- Input validation and sanitization
- Error handling without information disclosure
- Secure configuration management

### Frontend Security
- XSS prevention
- Input validation
- Secure API communication

## Performance Optimization

### Backend Optimization
- Async/await for I/O operations
- Efficient data structures
- Background task management

### Frontend Optimization
- Code splitting and lazy loading
- React.memo for expensive components
- Efficient re-rendering strategies

## Deployment

### Development
- Hot reload enabled
- Debug builds
- Development server

### Production
- Optimized builds
- Asset compression
- Error tracking

## Documentation

### Code Documentation
- Rust doc comments
- TypeScript JSDoc comments
- Component documentation

### Architecture Documentation
- This file (PROJECT_STRUCTURE.md)
- LIBRARY_STRUCTURE.md
- FRONTEND_DEVELOPMENT_SUMMARY.md
- ui-design-document.md

## Current Status (2025-01-28)

### ✅ **Completed Features**
- Atomic design system implementation
- Tauri v2 integration
- OBS WebSocket integration
- Event management system
- **CPU monitoring system** (NEW)
- Logging and archival system
- Configuration management

### 🚧 **In Progress**
- CPU monitoring testing with `wmic`
- Performance optimization
- Error handling improvements

### 📋 **Planned Features**
- Advanced filtering capabilities
- Real-time data streaming
- Custom themes
- Internationalization

---

**Last Updated**: 2025-01-28
**Version**: 0.1.0
**Status**: CPU monitoring implementation complete, awaiting testing 