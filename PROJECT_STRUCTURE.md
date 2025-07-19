# Project Structure Documentation

## Overview

This document outlines the structure and organization of the reStrike VTA project, a Windows-native Tauri application with React frontend and Rust backend.

## Project Architecture

### Frontend (React/TypeScript)
- **Location**: `ui/` directory
- **Framework**: React 18 with TypeScript
- **Styling**: Tailwind CSS
- **State Management**: Zustand
- **Build Tool**: Vite (via React Scripts)

### Backend (Rust/Tauri)
- **Location**: `src-tauri/` directory
- **Framework**: Tauri v2
- **Language**: Rust
- **Architecture**: Plugin-based modular system

## Directory Structure

```
reStrike_VTA_Cursor/
├── ui/                          # React frontend
│   ├── src/
│   │   ├── components/          # React components (atomic design)
│   │   │   ├── atoms/           # Basic UI elements
│   │   │   ├── molecules/       # Compound components
│   │   │   ├── organisms/       # Complex components
│   │   │   └── layouts/         # Layout components
│   │   ├── stores/              # Zustand state management
│   │   ├── hooks/               # Custom React hooks
│   │   ├── utils/               # Utility functions
│   │   └── types/               # TypeScript type definitions
│   └── public/                  # Static assets
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── plugins/             # Plugin modules
│   │   ├── core/                # Core application logic
│   │   ├── commands/            # Tauri commands
│   │   └── logging/             # Logging system
│   ├── gen/schemas/             # Generated schemas
│   └── config/                  # Configuration files
├── docs/                        # Documentation
├── scripts/                     # Development scripts
└── config/                      # Project configuration
```

## Component Architecture

### Atomic Design System
- **Atoms**: Basic building blocks (Button, Input, Icon, StatusDot)
- **Molecules**: Simple combinations (EventTableSection, LiveDataPanel)
- **Organisms**: Complex components (EventTable, ObsWebSocketManager)
- **Layouts**: Page-level components (DockBar, AdvancedPanel)

### Component Hierarchy
```
App.tsx
├── DockBar.tsx
│   ├── SidebarSmall.tsx
│   └── SidebarBig.tsx
└── AdvancedPanel.tsx
    ├── ObsWebSocketManager.tsx
    ├── LiveDataPanel.tsx
    └── CpuMonitoringSection.tsx
```

## Development Guidelines

### 🚨 **Critical UI Development Rules**

#### **UI Work Boundaries**
- **ONLY modify**: React components and UI styling
- **NEVER touch**: Backend code, Tauri configuration, or permissions
- **Focus on**: Visual appearance, layout, and user experience
- **Preserve**: All existing functionality exactly as is

#### **Safe to Modify**
- `ui/src/components/` - All React components
- `ui/src/App.tsx` - Main application layout
- `ui/src/stores/` - UI state management
- Tailwind CSS classes and styling
- Component props and UI logic

#### **Never Touch During UI Work**
- `src-tauri/` - Any Rust code or backend files
- `capabilities.json` - Tauri permissions
- `tauri.conf.json` - Tauri configuration
- Event listeners and API calls
- Backend plugins and commands

### Backend Architecture

#### Plugin System
- **plugin_obs.rs**: OBS WebSocket integration
- **plugin_cpu_monitor.rs**: System monitoring
- **plugin_store.rs**: Data persistence
- **plugin_udp.rs**: UDP protocol handling

#### Core Modules
- **app.rs**: Application initialization and state
- **config.rs**: Configuration management
- **logging/**: Custom logging system with archival

#### Tauri Integration
- **commands/**: Tauri command definitions
- **tauri_commands.rs**: Frontend-backend communication
- **gen/schemas/**: Generated API schemas

## State Management

### Frontend State (Zustand)
```typescript
// Main application state
interface AppState {
  isAdvancedPanelOpen: boolean;
  obsConnections: ObsConnection[];
  currentView: string;
  // ... other UI state
}
```

### Backend State (Rust)
```rust
// Application state with Arc<Mutex<>>
pub struct App {
    log_manager: Arc<Mutex<LogManager>>,
    obs_plugin: Arc<Mutex<ObsPlugin>>,
    // ... other state
}
```

## Development Workflow

### Frontend Development
1. **Start dev server**: `cd ui && npm start`
2. **Make UI changes**: Only React components and styling
3. **Test functionality**: Ensure existing features work
4. **No backend changes**: Never modify Rust code during UI work

### Backend Development
1. **Start Tauri**: `cargo tauri dev`
2. **Modify Rust code**: Only when working on backend features
3. **Test integration**: Verify frontend-backend communication
4. **Update permissions**: Only when adding new Tauri capabilities

### UI Design Work
1. **Identify scope**: Only visual/styling changes
2. **Modify UI files**: React components and Tailwind CSS
3. **Preserve functionality**: All backend features must work
4. **Test appearance**: Verify visual changes work correctly

## Configuration Files

### Frontend Configuration
- `ui/package.json`: Dependencies and scripts
- `ui/tailwind.config.js`: Tailwind CSS configuration
- `ui/tsconfig.json`: TypeScript configuration

### Backend Configuration
- `src-tauri/Cargo.toml`: Rust dependencies
- `src-tauri/tauri.conf.json`: Tauri application configuration
- `src-tauri/gen/schemas/capabilities.json`: Tauri permissions

## Documentation Structure

### Core Documentation
- `CONTINUATION_PROMPT.md`: Current project status and next steps
- `FRONTEND_DEVELOPMENT_SUMMARY.md`: Frontend development details
- `PROJECT_STRUCTURE.md`: This file - project organization
- `LIBRARY_STRUCTURE.md`: Backend library structure

### Feature Documentation
- `docs/FLAG_MANAGEMENT_SYSTEM.md`: IOC flag system
- `docs/OBS_INTEGRATION.md`: OBS WebSocket integration
- `docs/requirements/`: Software requirements and specifications

## Best Practices

### Code Organization
- **Separation of concerns**: UI logic separate from business logic
- **Atomic design**: Consistent component hierarchy
- **Type safety**: TypeScript for frontend, Rust for backend
- **Error handling**: Proper error boundaries and fallbacks

### Development Process
- **UI work isolation**: Never touch backend during UI development
- **Feature branches**: Separate UI and backend development
- **Testing**: Verify functionality after any changes
- **Documentation**: Update docs when adding new features

### Performance Considerations
- **Lazy loading**: Load components on demand
- **State optimization**: Minimize unnecessary re-renders
- **Bundle size**: Keep frontend bundle optimized
- **Memory usage**: Efficient backend resource management

---

**Last Updated**: 2025-01-28  
**Status**: Project structure documented with clear development guidelines  
**Focus**: Maintain separation between UI and backend development 