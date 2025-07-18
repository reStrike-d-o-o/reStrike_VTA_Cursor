# Project Context

## Overview
reStrike VTA is a Windows-native desktop application designed for taekwondo competition management, featuring advanced OBS Studio integration, real-time event processing, and comprehensive video replay capabilities. Built with Tauri v2, React, and Rust, the application provides a robust platform for tournament organizers and referees.

## Current Status ✅

### Core Systems Complete
- **Tauri v2 Integration**: Native Windows desktop application
- **Configuration Management**: Comprehensive settings persistence system
- **OBS WebSocket Integration**: Full OBS Studio v5 protocol support
- **Atomic Design System**: Complete frontend component architecture
- **Plugin Architecture**: Modular backend with clear separation of concerns

### Recent Major Updates (2025-01-28)
- **Configuration System**: Complete settings persistence across sessions
- **OBS Connection Management**: WebSocket connections with configuration integration
- **WebSocket Manager**: Full CRUD operations with status monitoring
- **Settings Persistence**: All app settings survive restarts
- **Backup System**: Automatic configuration backup and restore

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

## Key Features

### Configuration Management System
- **Persistent Settings**: All settings survive app restarts
- **OBS Connections**: WebSocket connections with password preservation
- **Cross-Session Sync**: Frontend and backend stay synchronized
- **Backup/Restore**: Automatic backup with manual restore
- **Import/Export**: Full configuration backup and restore
- **Statistics**: Configuration health monitoring

### OBS Integration
- **WebSocket v5**: Full OBS WebSocket v5 protocol support
- **Multiple Connections**: Support for multiple OBS instances
- **Real-time Status**: Live connection status monitoring
- **Authentication**: Secure password handling
- **Scene Control**: Scene switching and management
- **Recording Control**: Start/stop recording functionality

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
│  │             │ │             │ │             │ │         │ │
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

## OBS Integration

### WebSocket Management
The application provides comprehensive OBS WebSocket management:

- **Connection Management**: Add, edit, delete OBS connections
- **Status Monitoring**: Real-time connection status updates
- **Authentication**: Secure password handling and preservation
- **Protocol Support**: OBS WebSocket v5 protocol
- **Configuration Integration**: Connections persist across sessions

### OBS Commands
- **Scene Management**: Get/set current scene
- **Recording Control**: Start/stop recording
- **Replay Buffer**: Start/stop/save replay buffer
- **Status Monitoring**: Real-time OBS status
- **Connection Management**: Multiple connection support

## Event Processing

### PSS Protocol
- **UDP Listener**: Configurable UDP listener
- **Protocol Parsing**: PSS protocol schema parsing
- **Event Filtering**: Configurable event filtering
- **Real-time Processing**: Live event processing
- **Data Storage**: SQLite-based event storage

### Event Management
- **Event Storage**: Persistent event storage
- **Event Filtering**: Advanced filtering capabilities
- **Data Export**: Event data export
- **Live Streaming**: Real-time event streaming
- **Statistics**: Event statistics and analytics

## Video Management

### Clip Management
- **Automatic Extraction**: Automatic clip extraction from OBS
- **Metadata Handling**: Video metadata management
- **Organization**: Automatic clip organization
- **Playback**: High-performance video playback
- **Storage**: Efficient clip storage management

### MPV Integration
- **High Performance**: Hardware-accelerated playback
- **Format Support**: Wide format support
- **Custom Controls**: Custom playback controls
- **Integration**: Seamless OBS integration
- **Optimization**: Performance optimization

## Logging and Diagnostics

### Logging System
- **Multi-subsystem**: Comprehensive logging across all subsystems
- **File Rotation**: Automatic log file rotation
- **Compression**: Log compression for storage efficiency
- **Archiving**: Automatic log archiving
- **Live Streaming**: Real-time log streaming

### Diagnostic Tools
- **Health Monitoring**: System health monitoring
- **Performance Metrics**: Performance tracking
- **Error Tracking**: Comprehensive error tracking
- **Debug Tools**: Built-in debugging utilities
- **Reporting**: Diagnostic reporting

## Frontend Architecture

### Atomic Design System
The frontend follows atomic design principles:

- **Atoms**: Basic UI components (Button, Input, etc.)
- **Molecules**: Composite components (EventTableSection, etc.)
- **Organisms**: Complex UI sections (EventTable, Settings, etc.)
- **Layouts**: Page and section layouts (DockBar, AdvancedPanel, etc.)

### State Management
- **Zustand**: Lightweight state management
- **TypeScript**: Full type safety
- **Reactive Updates**: Reactive state updates
- **Persistence**: State persistence where needed
- **Performance**: Optimized state management

## Performance Considerations

### Backend Performance
- **Async Operations**: All I/O operations are async
- **Thread Safety**: RwLock for concurrent access
- **Memory Management**: Efficient memory usage
- **Resource Cleanup**: Proper resource disposal
- **Optimization**: Performance optimization throughout

### Frontend Performance
- **React Optimization**: React.memo and useMemo
- **Bundle Optimization**: Tree shaking and code splitting
- **State Management**: Efficient Zustand usage
- **Component Design**: Atomic design for reusability
- **Caching**: Strategic caching implementation

## Security Considerations

### Configuration Security
- **Password Handling**: Secure password storage
- **File Permissions**: Proper file permissions
- **Backup Security**: Secure backup storage
- **Validation**: Input validation and sanitization
- **Encryption**: Sensitive data encryption

### Network Security
- **WebSocket Security**: Secure WebSocket connections
- **Authentication**: Proper authentication handling
- **Data Validation**: Network data validation
- **Error Handling**: Secure error handling
- **Protocol Security**: Protocol-level security

## Testing Strategy

### Backend Testing
- **Unit Tests**: Individual component testing
- **Integration Tests**: Plugin interaction testing
- **Configuration Tests**: Configuration system testing
- **Error Tests**: Error handling testing
- **Performance Tests**: Performance benchmarking

### Frontend Testing
- **Component Tests**: React component testing
- **Integration Tests**: Tauri command testing
- **E2E Tests**: End-to-end workflow testing
- **Performance Tests**: Performance benchmarking
- **Accessibility Tests**: Accessibility compliance testing

## Deployment

### Windows Distribution
- **Native Windows .exe**: Standalone executable
- **MSI Package**: Enterprise deployment package
- **Portable Executable**: Portable application option
- **Auto-update System**: Automatic update mechanism
- **Installation**: User-friendly installation process

### Development Distribution
- **Development Builds**: Development builds with hot reload
- **Debug Builds**: Debug builds with full logging
- **Release Builds**: Optimized release builds
- **Testing Builds**: Testing-specific builds
- **CI/CD Integration**: Continuous integration and deployment

## Maintenance and Updates

### Regular Maintenance
- **Monthly Reviews**: Monthly structure and performance reviews
- **Dependency Updates**: Regular dependency updates
- **Performance Monitoring**: Continuous performance monitoring
- **Security Updates**: Regular security updates
- **Documentation Updates**: Regular documentation maintenance

### Configuration Migration
- **Version Management**: Configuration version tracking
- **Migration Scripts**: Automatic migration scripts
- **Backward Compatibility**: Backward compatibility support
- **Validation**: Configuration validation
- **Rollback**: Configuration rollback capabilities

## Future Roadmap

### Planned Features
- **Real-time Collaboration**: Multi-user collaboration features
- **Advanced Analytics**: Advanced event analytics
- **Cloud Integration**: Cloud-based data synchronization
- **Mobile Support**: Mobile application support
- **API Integration**: Third-party API integrations

### Performance Improvements
- **Advanced Caching**: Advanced caching strategies
- **Optimization**: Further performance optimization
- **Scalability**: Improved scalability features
- **Memory Management**: Advanced memory management
- **Resource Optimization**: Resource usage optimization

---

*Last updated: 2025-01-28*
*Configuration system: Complete*
*OBS WebSocket management: Complete*
*Atomic design system: Complete*
*Performance optimization: Implemented*
*Documentation: Comprehensive* 