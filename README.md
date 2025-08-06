# reStrike VTA - Taekwondo Competition Management System

## 🏆 Overview

reStrike VTA is a Windows-native desktop application designed for taekwondo competition management, featuring advanced OBS Studio integration, real-time event processing, and comprehensive video replay capabilities. Built with Tauri v2, React, and Rust, the application provides a robust platform for tournament organizers and referees.

## 🚀 Current Status (2025-01-06)

### ✅ Complete Systems
- **Tauri v2 Integration**: Native Windows desktop application with secure Control Room access
- **Production Security**: Enterprise-grade bcrypt authentication with 12-hour tournament sessions
- **Control Room**: Complete STR management system with dedicated connection management
- **OBS WebSocket Integration**: Full OBS Studio v5 protocol support with advanced connection management
- **Atomic Design System**: Complete frontend component architecture with security integration
- **Plugin Architecture**: Modular backend with thread-safe async database layer
- **Database Security**: Comprehensive audit logging and encrypted configuration storage
- **Session Management**: Tournament-optimized session timeouts with refresh and logout capabilities
- **Flag Management Database**: Complete database-backed flag management with 253+ IOC flags
- **Real-time Events**: Push-based event system for live updates with full monitoring

### 🔧 Latest Major Updates (2025-01-06)
- **🔐 Control Room Security Enhancement**: Complete production-grade security implementation
  - **bcrypt Authentication**: Enterprise-level password hashing with DEFAULT_COST security
  - **Tournament Sessions**: 12-hour session timeouts optimized for competition day operations
  - **Audit Logging**: Comprehensive authentication attempt tracking and security event logging
  - **Database Security**: Three dedicated security tables with encrypted storage
  - **API Security**: 9 production-ready Tauri commands with full authentication control
- **🏗️ Hybrid Database Architecture**: Seamless integration of rusqlite and sqlx for async operations
- **📊 Zero Technical Debt**: Clean compilation with no warnings, production deployment ready
- **📚 Documentation Update**: Complete security architecture documentation and implementation guides

## 🛠️ Technology Stack

### Backend (Rust + Tauri v2)
- **Framework**: Tauri v2 for native Windows integration
- **Language**: Rust with async/await support
- **Architecture**: Plugin-based microkernel architecture
- **WebSocket**: tokio-tungstenite for OBS integration (v5 protocol only)
- **Configuration**: JSON-based settings with automatic persistence
- **Logging**: Structured logging with file rotation

### Frontend (React + TypeScript)
- **Framework**: React 18 with TypeScript
- **Styling**: Tailwind CSS with atomic design
- **State Management**: Zustand for global state
- **Build System**: Vite with Tauri integration
- **Components**: Atomic design pattern (atoms, molecules, organisms, layouts)

## 🎯 Key Features

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

## 🚀 Quick Start

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

# Start development (Windows-native)
cd src-tauri
cargo tauri dev
```

### Build Commands
```bash
# Development build
cd ui && npm run build

# Production build
cd src-tauri && cargo tauri build

# Frontend development server
cd ui && npm start
```

## 📚 Documentation

### Core Documentation
The project documentation is organized in a consolidated structure with the following main documents:

#### **Main Architecture Documents** (in `docs/architecture/`)
- **[Database Integration Guide](docs/architecture/DATABASE_INTEGRATION_GUIDE.md)**: Complete database architecture, models, and integration patterns
- **[Backend Architecture](docs/architecture/BACKEND_ARCHITECTURE.md)**: Backend structure, plugin system, and Rust implementation
- **[Frontend Architecture](docs/architecture/FRONTEND_ARCHITECTURE.md)**: Frontend structure, React components, and UI patterns
- **[Data Flow Architecture](docs/architecture/DATA_FLOW_ARCHITECTURE.md)**: System data flow, event processing, and integration patterns
- **[Layer Wiring Diagram](docs/architecture/LAYER_WIRING_DIAGRAM.md)**: Layer connections, dependencies, and system architecture
- **[UI Design Guidelines](docs/architecture/UI_DESIGN_GUIDELINES.md)**: UI development guidelines, design system, and component patterns

#### **Development Guides** (in `docs/development/`)
- **[Windows Setup Guide](docs/development/WINDOWS_SETUP_GUIDE.md)**: Windows development environment setup
- **[AI Agent Windows Guide](docs/development/AI_AGENT_WINDOWS_GUIDE.md)**: AI assistant setup and configuration
- **[Development Management](docs/development/development-management.md)**: Development workflows and project management

#### **Integration Guides** (in `docs/integration/`)
- **[OBS Integration](docs/integration/OBS_INTEGRATION.md)**: OBS WebSocket integration and connection management
- **[Flag Management](docs/integration/FLAG_MANAGEMENT.md)**: IOC flag system and management

#### **Requirements** (in `docs/requirements/`)
- **[Product Requirements](docs/requirements/PRODUCT_REQUIREMENTS.md)**: Product specifications and requirements
- **[Software Requirements](docs/requirements/SOFTWARE_REQUIREMENTS.md)**: Technical requirements and dependencies

### Documentation Structure
```
docs/
├── architecture/                       # Main consolidated docs (authoritative)
│   ├── DATABASE_INTEGRATION_GUIDE.md
│   ├── BACKEND_ARCHITECTURE.md
│   ├── FRONTEND_ARCHITECTURE.md
│   ├── DATA_FLOW_ARCHITECTURE.md
│   ├── LAYER_WIRING_DIAGRAM.md
│   └── UI_DESIGN_GUIDELINES.md
├── development/                        # Development guides
├── integration/                        # Integration guides
└── requirements/                       # Requirements docs
```

**Note**: The 6 main architecture documents in `docs/architecture/` serve as the single source of truth for all project documentation. All other documentation references and builds upon these core documents.

## 🔧 Configuration System

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

## 🎥 OBS Integration

### WebSocket Management
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

## 📝 Recent Changes

### 2025-01-28: OBS Connection Management Improvements
- **Protocol Simplification**: Removed OBS WebSocket v4 support, streamlined to v5 only
- **Parameter Fixes**: Resolved TypeScript parameter mismatches between frontend and backend
- **Disconnect Functionality**: Added proper WebSocket disconnection that preserves configuration
- **Settings Separation**: Clear separation between "Save Connection Settings" and "Connect" actions
- **Type Safety**: Fixed all TypeScript compilation errors
- **Documentation**: Consolidated and updated all documentation

### Key Technical Improvements
- **Backend**: Added `disconnect_obs()` method for proper WebSocket disconnection
- **Frontend**: Updated WebSocketManager with proper button labels and functionality
- **Configuration**: Enhanced settings persistence and synchronization
- **Error Handling**: Improved error messages and user feedback

## 🤝 Contributing

Please read the [Development Guide](docs/DEVELOPMENT.md) for coding standards and contribution guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Last Updated**: 2025-01-28  
**Version**: 0.1.0  
**Status**: Active Development  
**OBS Protocol**: WebSocket v5 only