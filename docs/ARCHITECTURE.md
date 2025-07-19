# reStrike VTA Architecture Guide

## 🏗️ System Architecture Overview

reStrike VTA follows a **plugin-based microkernel architecture** with clear separation of concerns between frontend, backend, and infrastructure layers. The system is designed for modularity, maintainability, and extensibility.

## 🎯 Design Principles

### 1. Plugin-Based Architecture
- **Microkernel Pattern**: Core functionality provided by independent plugins
- **Loose Coupling**: Plugins communicate through well-defined interfaces
- **High Cohesion**: Related functionality grouped within plugins
- **Extensibility**: New features can be added as plugins

### 2. Configuration-Driven Design
- **Persistent Settings**: All settings survive application restarts
- **Cross-Session Sync**: Frontend and backend stay synchronized
- **Backup/Restore**: Automatic configuration backup and restore
- **Import/Export**: Full configuration management capabilities

### 3. Real-Time Processing
- **Event-Driven**: Asynchronous event processing
- **WebSocket Integration**: Real-time OBS communication
- **UDP Listener**: PSS protocol event collection
- **Live Data Streaming**: Real-time data streaming capabilities

## 🏛️ Architecture Layers

### 1. Tauri Application Layer
```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Application Layer                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   Window    │ │   System    │ │   IPC       │ │  Shell  │ │
│  │  Management │ │  Integration│ │  Commands   │ │  API    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Responsibilities**:
- Native Windows integration
- Window management and UI rendering
- System tray and notifications
- IPC (Inter-Process Communication)
- Shell integration

### 2. Core Application Layer
```
┌─────────────────────────────────────────────────────────────┐
│                    Core Application Layer                   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   Config    │ │   Logging   │ │    Types    │ │  Core   │ │
│  │  Manager    │ │   Manager   │ │             │ │  App    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Components**:

#### Config Manager
- **Purpose**: Centralized configuration management
- **Features**: JSON persistence, backup/restore, import/export
- **Location**: `src-tauri/src/config/`
- **Key Files**:
  - `manager.rs`: Configuration loading/saving logic
  - `types.rs`: Configuration data structures
  - `mod.rs`: Module organization

#### Logging Manager
- **Purpose**: Structured logging across all subsystems
- **Features**: File rotation, archiving, live streaming
- **Location**: `src-tauri/src/plugins/plugin_logging.rs`

#### Types
- **Purpose**: Shared type definitions
- **Features**: Error types, data structures, interfaces
- **Location**: `src-tauri/src/types/`

#### Core App
- **Purpose**: Application lifecycle management
- **Features**: Plugin initialization, event routing, state management
- **Location**: `src-tauri/src/core/app.rs`

### 3. Plugin Layer
```
┌─────────────────────────────────────────────────────────────┐
│                    Plugin Layer                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │    OBS      │ │     UDP     │ │  Playback   │ │  Store  │ │
│  │   Plugin    │ │   Plugin    │ │   Plugin    │ │ Plugin  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Plugins**:

#### OBS Plugin
- **Purpose**: OBS Studio WebSocket integration
- **Features**: Connection management, scene control, recording/streaming
- **Location**: `src-tauri/src/plugins/plugin_obs.rs`
- **Key Capabilities**:
  - WebSocket v5 protocol support
  - Multiple connection management
  - Real-time status monitoring
  - Authentication handling
  - Scene and recording control

#### UDP Plugin
- **Purpose**: PSS protocol event collection
- **Features**: UDP listener, event processing, real-time streaming
- **Location**: `src-tauri/src/plugins/plugin_udp.rs`

#### Playback Plugin
- **Purpose**: Video playback and clip management
- **Features**: MPV integration, clip extraction, metadata handling
- **Location**: `src-tauri/src/plugins/plugin_playback.rs`

#### Store Plugin
- **Purpose**: Data persistence and event storage
- **Features**: SQLite database, event management, data export
- **Location**: `src-tauri/src/plugins/plugin_store.rs`

### 4. Infrastructure Layer
```
┌─────────────────────────────────────────────────────────────┐
│                    Infrastructure Layer                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │  WebSocket  │ │    SQLite   │ │   File I/O  │ │ Network │ │
│  │             │ │             │ │             │ │         │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Infrastructure Components**:
- **WebSocket**: tokio-tungstenite for OBS communication
- **SQLite**: Event storage and configuration persistence
- **File I/O**: Log files, configuration files, video files
- **Network**: UDP listener, HTTP client, network utilities

## 🔧 Frontend Architecture

### Atomic Design System
```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend Architecture                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   Atoms     │ │ Molecules   │ │ Organisms   │ │ Layouts │ │
│  │             │ │             │ │             │ │         │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    State Management                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   Zustand   │ │   Hooks     │ │   Utils     │ │ Types   │ │
│  │   Store     │ │             │ │             │ │         │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Component Hierarchy

#### Atoms (Basic Components)
- **Location**: `ui/src/components/atoms/`
- **Components**:
  - `Button.tsx`: Reusable button component
  - `Input.tsx`: Form input component
  - `Label.tsx`: Form label component
  - `StatusDot.tsx`: Status indicator component
  - `Icon.tsx`: Icon component
  - `Checkbox.tsx`: Checkbox component

#### Molecules (Composite Components)
- **Location**: `ui/src/components/molecules/`
- **Components**:
  - `WebSocketManager.tsx`: OBS connection management
  - `EventTableSection.tsx`: Event table display
  - `PlayerInfoSection.tsx`: Player information display
  - `MatchDetailsSection.tsx`: Match details display
  - `LiveDataPanel.tsx`: Live data streaming panel
  - `LogDownloadList.tsx`: Log file management
  - `LogToggleGroup.tsx`: Logging controls

#### Organisms (Complex Components)
- **Location**: `ui/src/components/organisms/`
- **Components**:
  - `EventTable.tsx`: Complete event table
  - `MatchInfoSection.tsx`: Complete match information
  - `ObsWebSocketManager.tsx`: OBS integration manager
  - `Overlay.tsx`: Overlay display
  - `Settings.tsx`: Settings management
  - `SidebarBig.tsx`: Main sidebar
  - `SidebarSmall.tsx`: Compact sidebar
  - `StatusBar.tsx`: Status bar
  - `VideoClips.tsx`: Video clip management

#### Layouts (Page Layouts)
- **Location**: `ui/src/components/layouts/`
- **Components**:
  - `DockBar.tsx`: Main application layout
  - `AdvancedPanel.tsx`: Advanced settings panel
  - `StatusbarDock.tsx`: Status bar layout
  - `StatusbarAdvanced.tsx`: Advanced status layout
  - `TaskBar.tsx`: Task bar layout

### State Management

#### Zustand Store
- **Location**: `ui/src/stores/index.ts`
- **Features**:
  - Global state management
  - OBS connection state
  - UI state management
  - Video clip state
  - Settings state

#### Custom Hooks
- **Location**: `ui/src/hooks/`
- **Hooks**:
  - `useEnvironment.ts`: Environment detection
  - `useEnvironmentApi.ts`: API environment hooks
  - `useEnvironmentObs.ts`: OBS environment hooks

## 🔄 Data Flow

### Configuration Flow
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Frontend  │───▶│   Tauri     │───▶│   Config    │
│   Settings  │    │   Commands  │    │  Manager    │
└─────────────┘    └─────────────┘    └─────────────┘
       ▲                   │                   │
       │                   ▼                   ▼
└─────────────┐    ┌─────────────┐    ┌─────────────┐
│   React     │◀───│   JSON      │◀───│   File      │
│   State     │    │   Response  │    │   System    │
└─────────────┘    └─────────────┘    └─────────────┘
```

### OBS Communication Flow
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Frontend  │───▶│   Tauri     │───▶│   OBS       │
│   WebSocket │    │   Commands  │    │   Plugin    │
│   Manager   │    │             │    │             │
└─────────────┘    └─────────────┘    └─────────────┘
       ▲                   │                   │
       │                   ▼                   ▼
└─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Status    │◀───│   Status    │◀───│   OBS       │
│   Updates   │    │   Events    │    │   Studio    │
└─────────────┘    └─────────────┘    └─────────────┘
```

### Event Processing Flow
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   PSS       │───▶│   UDP       │───▶│   Event     │
│   Protocol  │    │   Plugin    │    │  Processing │
└─────────────┘    └─────────────┘    └─────────────┘
                           │                   │
                           ▼                   ▼
                   ┌─────────────┐    ┌─────────────┐
                   │   Store     │    │   Frontend  │
                   │   Plugin    │    │   Display   │
                   └─────────────┘    └─────────────┘
```

## 🔐 Security Architecture

### Authentication
- **OBS WebSocket**: SHA256 authentication for v5 protocol
- **Password Handling**: Secure password storage and transmission
- **Configuration**: Encrypted sensitive data storage

### Data Protection
- **Configuration Backup**: Automatic backup with encryption
- **Log Security**: Secure log file handling
- **Network Security**: Secure WebSocket and UDP communication

## 📊 Performance Architecture

### Optimization Strategies
- **Async/Await**: Non-blocking I/O operations
- **Plugin Isolation**: Independent plugin execution
- **Memory Management**: Efficient memory usage patterns
- **Caching**: Configuration and data caching

### Monitoring
- **Performance Metrics**: CPU usage, memory consumption
- **Connection Monitoring**: WebSocket connection health
- **Error Tracking**: Comprehensive error logging
- **Status Monitoring**: Real-time system status

## 🔧 Development Architecture

### Build System
- **Frontend**: Vite with React and TypeScript
- **Backend**: Cargo with Rust
- **Integration**: Tauri CLI for application bundling

### Development Workflow
- **Hot Reload**: Frontend hot reload with Tauri integration
- **Type Safety**: Full TypeScript and Rust type checking
- **Error Handling**: Comprehensive error handling and reporting
- **Testing**: Unit and integration testing support

## 📈 Scalability

### Horizontal Scaling
- **Multiple OBS Instances**: Support for multiple OBS connections
- **Event Processing**: Scalable event processing pipeline
- **Configuration Management**: Scalable configuration system

### Vertical Scaling
- **Plugin Architecture**: Modular plugin system for feature expansion
- **Configuration Segments**: Extensible configuration system
- **Component System**: Extensible frontend component system

---

**Last Updated**: 2025-01-28  
**Architecture Version**: 2.0  
**Status**: Production Ready 