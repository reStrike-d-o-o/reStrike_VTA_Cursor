# UI Design Document

## Overview
This document describes the UI/UX design system and component architecture for the reStrike VTA project, a Windows-only native desktop application built with Tauri v2 and React.

## Current Status ✅

### Tauri v2 Integration Complete
- **Native Windows Mode**: Successfully running as native Windows desktop application
- **Environment Detection**: Automatic detection of Tauri API availability
- **Hot Reload**: Development mode with live reload for both frontend and backend
- **Build System**: Integrated build process working correctly

### Atomic Design Implementation Complete
- **Atoms**: All basic UI components extracted and implemented
- **Molecules**: Composite components fully functional
- **Organisms**: Complex UI sections implemented
- **Layouts**: Page and section layouts complete

## Design System

### Color Palette
- **Primary**: `#1e40af` (Blue) - Main brand color
- **Secondary**: `#64748b` (Gray) - Supporting elements
- **Accent**: `#f59e0b` (Orange) - Highlights and actions
- **Background**: `#101820` (Dark) - Main background
- **Surface**: `#181F26` (Darker gray) - Card and panel backgrounds
- **Text**: `#ffffff` (White) - Primary text
- **Text Secondary**: `#9ca3af` (Light gray) - Secondary text

### Typography
- **Font Family**: System fonts (Segoe UI on Windows)
- **Font Sizes**: Tailwind CSS scale (text-xs to text-6xl)
- **Font Weights**: 400 (normal), 500 (medium), 600 (semibold), 700 (bold)
- **Line Heights**: 1.2 (tight), 1.5 (normal), 1.75 (relaxed)

### Spacing
- **Base Unit**: 4px (0.25rem)
- **Spacing Scale**: Tailwind CSS spacing scale
- **Component Spacing**: Consistent 16px (1rem) and 24px (1.5rem) margins
- **Layout Spacing**: 32px (2rem) between major sections

## Component Architecture

### Atomic Design Hierarchy

#### Atoms (Basic Components)
Basic building blocks that cannot be broken down further:

**Button Component**
```typescript
interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  children: React.ReactNode;
  onClick?: () => void;
}
```

**Input Component**
```typescript
interface InputProps {
  type?: 'text' | 'email' | 'password' | 'number';
  placeholder?: string;
  value?: string;
  onChange?: (value: string) => void;
  disabled?: boolean;
  error?: string;
}
```

**StatusDot Component**
```typescript
interface StatusDotProps {
  status: 'online' | 'offline' | 'warning' | 'error';
  size?: 'sm' | 'md' | 'lg';
  label?: string;
}
```

**Icon Component**
```typescript
interface IconProps {
  name: string;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}
```

#### Molecules (Composite Components)
Components that combine atoms to create functional UI sections:

**EventTableSection**
- Combines EventTable organism with filtering controls
- Handles event data display and interaction
- Integrates with Tauri commands for data retrieval

**LiveDataPanel**
- Real-time data display component
- Integrates with OBS WebSocket for live data
- Status indicators and connection management

**LogToggleGroup**
- Log level toggle controls
- Combines Checkbox atoms with Label atoms
- Manages log filtering state

**LogDownloadList**
- Log file download management
- Combines Button atoms with status indicators
- Handles file download operations

#### Organisms (Complex Sections)
Major UI sections that combine molecules and atoms:

**DockBar**
- Main sidebar with player info and controls
- Two-column layout: SidebarSmall and SidebarBig
- Status indicators and navigation controls
- Player information display with flags

**EventTable**
- Main event table with real-time data
- Sorting and filtering capabilities
- Pagination and search functionality
- Event type indicators and timestamps

**ObsWebSocketManager**
- OBS connection management interface
- Connection status indicators
- Protocol selection and configuration
- Real-time connection monitoring

**MatchInfoSection**
- Match information display
- Player details with flag integration
- Match statistics and timing
- Score and status information

#### Layouts (Page Structure)
Page-level components that define overall structure:

**AdvancedPanel**
- Settings and configuration panel
- Tabbed interface for different settings categories
- Diagnostics and log management
- Drawer-based navigation system

**StatusbarAdvanced**
- Advanced status bar with detailed information
- System status indicators
- Performance metrics display
- Connection status monitoring

**StatusbarDock**
- Status bar for dock layout
- Compact status indicators
- Quick access controls
- System information display

## Layout System

### Main Application Layout
```
┌─────────────────────────────────────────────────────────┐
│                    Header                                │
├─────────────┬───────────────────────────────────────────┤
│             │                                           │
│   DockBar   │              Main Content                 │
│  (Sidebar)  │                                           │
│             │                                           │
├─────────────┴───────────────────────────────────────────┤
│                  Status Bar                             │
└─────────────────────────────────────────────────────────┘
```

### DockBar Layout
```
┌─────────────┬───────────────────────────────────────────┐
│ SidebarSmall│              SidebarBig                   │
│             │                                           │
│ [Replay]    │  Player Info (Flags, Names)              │
│ [Manual]    │  Match Details (Category, Stage)         │
│ [Advanced]  │  ─────────────────────────────────────── │
│             │  Event Table                             │
│             │  [↑ Scroll to Top]                       │
│             │                                           │
│ REC ●       │                                           │
│ STR ●       │                                           │
│ CPU ●       │                                           │
└─────────────┴───────────────────────────────────────────┘
```

### AdvancedPanel Layout
```
┌─────────────────────────────────────────────────────────┐
│ [PSS] [OBS] [Video] [AI] [Settings]                    │
├─────────────────────────────────────────────────────────┤
│                                                         │
│                    Content Area                         │
│                                                         │
│  • PSS: UDP server, PSS protocol, event DB             │
│  • OBS: Connection management and options               │
│  • Video: mpv integration and controls                  │
│  • AI: Report creation and data analyzer                │
│  • Settings: All settings, including Diagnostics       │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Responsive Design

### Breakpoints
- **Mobile**: 320px - 768px (not applicable for desktop app)
- **Tablet**: 768px - 1024px (not applicable for desktop app)
- **Desktop**: 1024px+ (primary target)

### Adaptive Layouts
- **Fixed Sidebar**: DockBar maintains consistent width
- **Flexible Content**: Main content area adapts to available space
- **Status Bar**: Always positioned at bottom
- **Overlays**: Positioned relative to viewport

## Accessibility

### ARIA Support
- **Labels**: Proper ARIA labels for all interactive elements
- **Descriptions**: ARIA descriptions for complex components
- **Roles**: Semantic HTML roles for screen readers
- **States**: ARIA states for dynamic content

### Keyboard Navigation
- **Tab Order**: Logical tab order through interface
- **Shortcuts**: Keyboard shortcuts for common actions
- **Focus Management**: Clear focus indicators
- **Skip Links**: Skip to main content functionality

### Color Contrast
- **WCAG AA**: Minimum 4.5:1 contrast ratio
- **High Contrast**: Support for high contrast mode
- **Color Blind**: Color-blind friendly design
- **Status Colors**: Status indicators use both color and shape

## Animation and Transitions

### Micro-interactions
- **Hover States**: Subtle hover effects on interactive elements
- **Focus States**: Clear focus indicators
- **Loading States**: Loading spinners and progress indicators
- **Status Changes**: Smooth transitions for status updates

### Page Transitions
- **Tab Switching**: Smooth transitions between tabs
- **Panel Opening**: Animated panel reveals
- **Modal Dialogs**: Fade in/out animations
- **Content Loading**: Progressive content loading

## Icon System

### Icon Types
- **System Icons**: Standard system icons for common actions
- **Status Icons**: Status indicators and notifications
- **Navigation Icons**: Navigation and menu icons
- **Action Icons**: Action buttons and controls

### Icon Sizes
- **Small**: 16px (1rem) - Inline icons
- **Medium**: 24px (1.5rem) - Button icons
- **Large**: 32px (2rem) - Header icons
- **Extra Large**: 48px (3rem) - Feature icons

## Status Indicators

### Connection Status
- **Online**: Green dot with "Connected" label
- **Offline**: Red dot with "Disconnected" label
- **Warning**: Yellow dot with "Warning" label
- **Error**: Red dot with "Error" label

### System Status
- **REC**: Recording status indicator
- **STR**: Streaming status indicator
- **CPU**: System performance indicator

### Event Status
- **New**: Blue indicator for new events
- **Processing**: Yellow indicator for processing events
- **Complete**: Green indicator for completed events
- **Error**: Red indicator for error events

## Form Design

### Input Fields
- **Text Inputs**: Clean, minimal design with focus states
- **Select Dropdowns**: Custom styled dropdowns
- **Checkboxes**: Custom checkbox design
- **Radio Buttons**: Custom radio button design

### Validation
- **Real-time Validation**: Immediate feedback on input
- **Error Messages**: Clear, helpful error messages
- **Success States**: Positive feedback for valid input
- **Loading States**: Loading indicators for async validation

## Data Visualization

### Tables
- **Event Table**: Sortable, filterable event data
- **Status Table**: System status and metrics
- **Log Table**: Log entries with filtering

### Charts and Graphs
- **Performance Charts**: System performance metrics
- **Event Timeline**: Chronological event display
- **Status Dashboard**: Real-time status overview

## Dark Theme

### Color Scheme
- **Background**: Dark backgrounds for reduced eye strain
- **Text**: High contrast white text
- **Accents**: Blue and orange accents for highlights
- **Borders**: Subtle gray borders for separation

### Benefits
- **Eye Strain**: Reduced eye strain in low-light environments
- **Professional**: Professional appearance for broadcast environments
- **Focus**: Better focus on content and data
- **Consistency**: Consistent with modern desktop applications

## Future Enhancements

### Planned Features
1. **Custom Themes**: User-configurable themes
2. **High Contrast Mode**: Enhanced accessibility
3. **Animation Library**: Comprehensive animation system
4. **Icon Library**: Extended icon set

### Technical Improvements
1. **Performance**: Optimized rendering and animations
2. **Accessibility**: Enhanced accessibility features
3. **Internationalization**: Multi-language support
4. **Responsive**: Enhanced responsive design

---

**Last Updated**: December 2024  
**Status**: ✅ Complete - Ready for Development  
**Design System**: Atomic Design with Dark Theme 