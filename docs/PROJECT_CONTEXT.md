# Project Context

## Overview
reStrike VTA is a Windows-native desktop application designed for taekwondo competition management, featuring advanced OBS Studio integration, real-time event processing, and comprehensive video replay capabilities. Built with Tauri v2, React, and Rust, the application provides a robust platform for tournament organizers and referees.

## Current Status ✅

### Core Systems Complete
- **Tauri v2 Integration**: Native Windows desktop application
- **Configuration Management**: Comprehensive settings persistence system
- **OBS WebSocket Integration**: Full OBS Studio v5 protocol support with connection management
- **Atomic Design System**: Complete frontend component architecture
- **Plugin Architecture**: Modular backend with clear separation of concerns

### Recent Major Updates (2025-01-28)
- **OBS Connection Management**: Complete WebSocket connection management system
- **Protocol Simplification**: Removed OBS WebSocket v4 support, streamlined to v5 only
- **Disconnect Functionality**: Proper WebSocket disconnection that preserves configuration
- **Settings Separation**: Clear separation between "Save Connection Settings" and "Connect" actions
- **TypeScript Error Fixes**: Resolved all parameter and type issues
- **Documentation Consolidation**: Comprehensive documentation system

## Technology Stack

### Backend (Rust + Tauri v2)
- **Framework**: Tauri v2 for native Windows integration
- **Language**: Rust with async/await support
- **Architecture**: Plugin-based microkernel architecture
- **Database**: SQLite for event storage and configuration
- **WebSocket**: tokio-tungstenite for OBS integration (v5 protocol only)
- **Logging**: Structured logging with file rotation

### Frontend (React + TypeScript)
- **Framework**: React 18 with TypeScript
- **Styling**: Tailwind CSS with atomic design
- **State Management**: Zustand for global state
- **Build System**: Vite with Tauri integration
- **Components**: Atomic design pattern (atoms, molecules, organisms, layouts)

## Key Features

### Configuration Management System
- **Persistent Settings**: All settings survive app restarts
- **OBS Connections**: WebSocket connections with password preservation
- **Cross-Session Sync**: Frontend and backend stay synchronized
- **Backup/Restore**: Automatic backup with manual restore
- **Import/Export**: Full configuration backup and restore
- **Statistics**: Configuration health monitoring

### OBS Integration (v5 Protocol Only)
- **WebSocket v5**: Full OBS WebSocket v5 protocol support
- **Multiple Connections**: Support for multiple OBS instances
- **Real-time Status**: Live connection status monitoring
- **Authentication**: Secure password handling and preservation
- **Connection Management**: Add, edit, delete, connect, disconnect
- **Settings Persistence**: Connections persist across sessions
- **Disconnect Functionality**: Proper disconnection without losing configuration

### Event Processing
- **UDP Listener**: PSS protocol event collection
- **Real-time Processing**: Live event processing and filtering
- **Event Storage**: SQLite-based event persistence
- **Data Export**: Event data export capabilities
- **Live Streaming**: Real-time data streaming

### Video Management
- **Clip Extraction**: Automatic clip extraction from OBS
- **MPV Integration**: High-performance video playback
- **Clip Organization**: Automatic clip organization
- **Metadata Management**: Video metadata handling
- **Replay Buffer**: OBS replay buffer integration

### Logging and Diagnostics
- **Multi-subsystem Logging**: Comprehensive logging system
- **File Rotation**: Automatic log file rotation
- **Archive Management**: Log archiving and compression
- **Live Data Streaming**: Real-time log streaming
- **Diagnostic Tools**: Built-in diagnostic utilities

## Architecture Overview

### Plugin-Based Microkernel
The application uses a microkernel architecture where core functionality is provided by independent plugins:

```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Application Layer                  │
├─────────────────────────────────────────────────────────────┤
│                    Core Application Layer                   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   Config    │ │   Logging   │ │    Types    │ │  Core   │ │
│  │  Manager    │ │   Manager   │ │             │ │  App    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Plugin Layer                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │    OBS      │ │     UDP     │ │  Playback   │ │  Store  │ │
│  │   Plugin    │ │   Plugin    │ │   Plugin    │ │ Plugin  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │  WebSocket  │ │    SQLite   │ │   File I/O  │ │ Network │ │
│  │   (v5)      │ │             │ │             │ │         │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Configuration System Architecture
The configuration system provides comprehensive settings management:

```
┌─────────────────────────────────────────────────────────────┐
│                    Configuration Manager                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │    App      │ │     OBS     │ │     UDP     │ │ Logging │ │
│  │  Settings   │ │  Settings   │ │  Settings   │ │Settings │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │     UI      │ │   Video     │ │  License    │ │ Flags   │ │
│  │  Settings   │ │  Settings   │ │  Settings   │ │Settings │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Persistence Layer                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   JSON      │ │   Backup    │ │   Import    │ │ Export  │ │
│  │   Store     │ │   System    │ │   System    │ │ System  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Development Environment

### Prerequisites
- **Operating System**: Windows 10/11
- **Node.js**: v24 or higher
- **Rust**: Stable toolchain
- **Tauri CLI**: Latest version
- **OBS Studio**: v28+ with WebSocket v5 plugin

### Development Setup
```bash
# Clone repository
git clone <repository-url>
cd reStrike_VTA_Cursor

# Install dependencies
npm install
cd ui && npm install

# Start development
cd src-tauri
cargo tauri dev
```

### Build Commands
```bash
# Development build
cd ui && npm run build

# Production build
cd src-tauri && cargo tauri build
```

## Configuration System

### Configuration Segments
The application manages settings across multiple segments:

1. **App Settings**: Version, startup behavior, performance
2. **OBS Settings**: Connections, defaults, behavior, reconnection settings
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

## OBS Integration

### WebSocket Management
The application provides comprehensive OBS WebSocket management:

- **Connection Management**: Add, edit, delete OBS connections
- **Status Monitoring**: Real-time connection status updates
- **Authentication**: Secure password handling and preservation
- **Protocol Support**: OBS WebSocket v5 protocol only (v4 removed)
- **Configuration Integration**: Connections persist across sessions
- **Disconnect Functionality**: Proper disconnection without losing configuration

### OBS Commands
- **Scene Management**: Get/set current scene
- **Recording Control**: Start/stop recording
- **Streaming Control**: Start/stop streaming
- **Replay Buffer**: Start/stop/save replay buffer
- **Status Monitoring**: Real-time status updates

### Connection Workflow
1. **Save Connection Settings**: Configure connection without connecting
2. **Connect**: Establish WebSocket connection to OBS
3. **Monitor Status**: Real-time connection status monitoring
4. **Disconnect**: Close connection while preserving configuration
5. **Reconnect**: Reconnect using saved configuration

## Recent Technical Improvements

### 2025-01-28: OBS Connection Management
- **Protocol Simplification**: Removed OBS WebSocket v4 support, streamlined to v5 only
- **Parameter Fixes**: Resolved TypeScript parameter mismatches between frontend and backend
- **Disconnect Functionality**: Added proper WebSocket disconnection that preserves configuration
- **Settings Separation**: Clear separation between "Save Connection Settings" and "Connect" actions
- **Type Safety**: Fixed all TypeScript compilation errors
- **Documentation**: Consolidated and updated all documentation

### Key Technical Changes
- **Backend**: Added `disconnect_obs()` method for proper WebSocket disconnection
- **Frontend**: Updated WebSocketManager with proper button labels and functionality
- **Configuration**: Enhanced settings persistence and synchronization
- **Error Handling**: Improved error messages and user feedback
- **Type Safety**: Resolved all TypeScript and Rust compilation issues

## Documentation Structure

### Core Documentation
- **[README.md](README.md)**: Main project overview and quick start
- **[ARCHITECTURE.md](ARCHITECTURE.md)**: Detailed system architecture and design patterns
- **[DEVELOPMENT.md](DEVELOPMENT.md)**: Development setup, coding standards, and workflows
- **[OBS_INTEGRATION.md](OBS_INTEGRATION.md)**: OBS WebSocket integration and connection management

### Legacy Documentation
- **FRONTEND_DEVELOPMENT_SUMMARY.md**: Frontend development details
- **LIBRARY_STRUCTURE.md**: Backend library structure
- **PROJECT_STRUCTURE.md**: Project file organization
- **FLAG_MANAGEMENT_SYSTEM.md**: Flag management system details

## Future Roadmap

### Planned Features
- **Multiple OBS Instances**: Support for multiple OBS instances on different machines
- **Advanced Authentication**: Support for additional authentication methods
- **Custom Commands**: Support for custom OBS WebSocket commands
- **Event Filtering**: Advanced event filtering and processing
- **Performance Analytics**: Detailed performance analytics and reporting

### Integration Opportunities
- **Streaming Platforms**: Integration with streaming platforms
- **Video Processing**: Advanced video processing capabilities
- **Automation**: Automated scene switching and recording
- **Analytics**: Competition analytics and reporting

---

**Last Updated**: 2025-01-28  
**Version**: 0.1.0  
**Status**: Active Development  
**OBS Protocol**: WebSocket v5 only 