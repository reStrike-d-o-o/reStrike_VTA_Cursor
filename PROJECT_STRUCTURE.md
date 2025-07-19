# Project Structure Documentation

## Overview (Updated: 2025-01-28)

The reStrike VTA project is a Windows-native desktop application built with Rust (Tauri) backend and React/TypeScript frontend. The project follows a modular, plugin-based architecture with comprehensive logging and monitoring capabilities.

## 🏗️ **Architecture Overview**

### **Backend (Rust/Tauri)**
- **Plugin System**: Modular architecture with specialized plugins for different subsystems
- **Logging System**: Custom LogManager with subsystem-based logging (app, pss, obs, udp)
- **State Management**: Thread-safe shared state with Arc and Mutex
- **Error Handling**: AppResult<T> pattern with proper error propagation

### **Frontend (React/TypeScript)**
- **Atomic Design**: Component hierarchy from atoms to organisms
- **State Management**: Zustand for global state, React hooks for local state
- **Real-time Updates**: WebSocket events and polling for live data
- **UI/UX**: Professional dark theme with Tailwind CSS

## 📁 **Directory Structure**

```
reStrike_VTA_Cursor/
├── src-tauri/                    # Rust backend (Tauri)
│   ├── src/
│   │   ├── plugins/              # Plugin modules
│   │   │   ├── plugin_obs.rs     # OBS WebSocket integration
│   │   │   ├── plugin_cpu_monitor.rs  # CPU monitoring
│   │   │   ├── plugin_udp.rs     # UDP server
│   │   │   ├── plugin_playback.rs # Video playback
│   │   │   ├── plugin_store.rs   # Data storage
│   │   │   └── plugin_license.rs # License management
│   │   ├── logging/              # Custom logging system
│   │   │   ├── mod.rs           # LogManager implementation
│   │   │   ├── logger.rs        # Logger components
│   │   │   ├── rotation.rs      # Log rotation
│   │   │   └── archival.rs      # Log archival
│   │   ├── core/                # Core application logic
│   │   │   ├── app.rs           # Main application class
│   │   │   ├── config.rs        # Configuration management
│   │   │   └── state.rs         # Application state
│   │   ├── commands/            # Tauri command handlers
│   │   ├── config/              # Configuration management
│   │   ├── types/               # Shared type definitions
│   │   ├── utils/               # Utility functions
│   │   └── main.rs              # Application entry point
│   ├── logs/                    # Log files directory
│   │   ├── app.log              # Application logs
│   │   ├── obs.log              # OBS WebSocket events
│   │   ├── pss.log              # PSS protocol events
│   │   ├── udp.log              # UDP server events
│   │   └── archives/            # Archived log files
│   └── config/                  # Configuration files
├── ui/                          # React frontend
│   ├── src/
│   │   ├── components/          # React components (Atomic Design)
│   │   │   ├── atoms/           # Basic UI components
│   │   │   ├── molecules/       # Component combinations
│   │   │   ├── organisms/       # Complex UI sections
│   │   │   └── layouts/         # Page-level layouts
│   │   ├── hooks/               # Custom React hooks
│   │   ├── stores/              # Zustand state stores
│   │   ├── utils/               # Utility functions
│   │   ├── types/               # TypeScript type definitions
│   │   └── config/              # Frontend configuration
│   └── public/                  # Static assets
│       └── assets/
│           └── flags/           # IOC flag images (253 PNGs)
├── docs/                        # Project documentation
├── scripts/                     # Development and utility scripts
└── config/                      # Global configuration
```

## 🔌 **Plugin System**

### **OBS Plugin** (`plugin_obs.rs`)
- **Purpose**: OBS Studio WebSocket integration
- **Features**: 
  - Real-time WebSocket communication
  - Scene management and recording control
  - Event logging to `obs.log` file
  - Connection status monitoring
- **Integration**: Custom LogManager for event logging
- **Status**: ✅ **COMPLETE** - Fully integrated with logging system

### **CPU Monitor Plugin** (`plugin_cpu_monitor.rs`)
- **Purpose**: System and process CPU monitoring
- **Features**:
  - Windows `wmic` command integration
  - Real-time process monitoring
  - System CPU usage tracking
  - Background task management
- **Status**: ✅ **COMPLETE** - Awaiting `wmic` installation for testing

### **UDP Plugin** (`plugin_udp.rs`)
- **Purpose**: UDP server for PSS protocol
- **Features**:
  - Real-time UDP packet processing
  - PSS protocol parsing
  - Event streaming to frontend
- **Status**: ✅ **COMPLETE**

### **Playback Plugin** (`plugin_playback.rs`)
- **Purpose**: Video playback and clip management
- **Features**:
  - MPV integration
  - Video clip extraction
  - Hardware acceleration
- **Status**: ✅ **COMPLETE**

### **Store Plugin** (`plugin_store.rs`)
- **Purpose**: Data persistence and storage
- **Features**:
  - Event storage
  - Configuration persistence
  - Data export/import
- **Status**: ✅ **COMPLETE**

### **License Plugin** (`plugin_license.rs`)
- **Purpose**: License management and validation
- **Features**:
  - License key validation
  - Feature access control
  - License status monitoring
- **Status**: ✅ **COMPLETE**

## 📝 **Logging System**

### **Custom LogManager**
- **Architecture**: Subsystem-based logging with file rotation
- **Subsystems**: app, pss, obs, udp
- **Features**:
  - Automatic log file creation
  - Log rotation based on file size
  - Log archival with retention policies
  - Thread-safe concurrent access
- **Integration**: All plugins use LogManager for structured logging

### **Log Files**
- **app.log**: Application-level events and errors
- **obs.log**: OBS WebSocket events and responses
- **pss.log**: PSS protocol events and data
- **udp.log**: UDP server events and packet processing

### **Log Management**
- **Rotation**: Automatic rotation at 10MB file size
- **Archival**: Compressed archives with 30-day retention
- **Access**: Tauri commands for log file management
- **Real-time**: Live log monitoring capabilities

## 🎨 **Frontend Architecture**

### **Atomic Design System**
- **Atoms**: Basic UI components (Button, Input, Icon, etc.)
- **Molecules**: Simple component combinations
- **Organisms**: Complex UI sections
- **Layouts**: Page-level structure components

### **Component Hierarchy**
```
App.tsx
├── DockBar (Sidebar)
│   ├── SidebarSmall (Controls)
│   └── SidebarBig (Info + Events)
└── AdvancedPanel (Main Content)
    ├── MatchInfoSection (Athlete/Match Details)
    ├── EventTable (Event Rows)
    ├── LiveDataPanel (Real-time Data)
    ├── CpuMonitoringSection (CPU Metrics)
    └── StatusBar (System Status)
```

### **State Management**
- **Zustand**: Global state management
- **React Hooks**: Component-level state
- **Tauri Commands**: Backend communication
- **Real-time Updates**: WebSocket events and polling

## 🔧 **Development Workflow**

### **Backend Development**
- **Build**: `cargo build` in `src-tauri/`
- **Run**: `cargo tauri dev` for development
- **Testing**: Unit tests with `cargo test`
- **Logging**: Structured logging with custom LogManager

### **Frontend Development**
- **Development**: `npm run start:docker` in `ui/`
- **Build**: `npm run build` for production
- **Testing**: Jest and React Testing Library
- **Linting**: ESLint with TypeScript rules

### **Integration**
- **Tauri Commands**: Type-safe backend-frontend communication
- **WebSocket Events**: Real-time data streaming
- **File System**: Log file access and management
- **System Integration**: OBS, CPU monitoring, UDP/PSS

## 📊 **Current Status**

### **✅ Completed Features**
- **OBS Integration**: Complete WebSocket integration with event logging
- **CPU Monitoring**: Real-time system and process monitoring
- **Logging System**: Comprehensive subsystem-based logging
- **Frontend UI**: Atomic design system with real-time updates
- **Plugin Architecture**: Modular, extensible plugin system

### **🚧 In Progress**
- **WMIC Installation**: Awaiting `wmic` command installation for CPU monitoring
- **Performance Optimization**: Ongoing optimization of real-time updates
- **Error Handling**: Enhanced error boundaries and user feedback

### **📋 Next Steps**
1. **Complete CPU Monitoring**: Install `wmic` and test real process data
2. **Performance Testing**: Optimize data flow and UI updates
3. **Error Handling**: Implement comprehensive error handling
4. **Documentation**: Update all documentation with latest changes

## 🔍 **Troubleshooting**

### **Common Issues**
- **Build Errors**: Check TypeScript types and Rust compilation
- **Runtime Errors**: Verify Tauri command availability
- **Logging Issues**: Check file permissions and LogManager initialization
- **Performance Issues**: Monitor bundle size and component re-renders

### **Development Tips**
- **Hot Reload**: Use `npm run start:docker` for frontend development
- **Logging**: Check `src-tauri/logs/` for detailed backend logs
- **Type Safety**: Leverage TypeScript for catching errors early
- **Plugin Development**: Follow the established plugin pattern

---

**Last Updated**: 2025-01-28  
**Status**: OBS logging integration complete, CPU monitoring awaiting `wmic` installation  
**Next Action**: Install `wmic` and test real process data display 