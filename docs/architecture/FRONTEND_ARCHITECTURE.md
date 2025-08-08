# Frontend Architecture

## Overview

The reStrike VTA frontend is built with React 18, TypeScript, and Tailwind CSS, providing a modern, responsive user interface for the Windows desktop application. The frontend follows atomic design principles and integrates seamlessly with the Tauri backend through a comprehensive command and event system.

## Product Requirements

### Elevator Pitch
A cross-platform Instant Video Replay Manager designed for taekwondo referees, enabling rapid video review and AI-assisted data analysis. The app integrates with taekwondo PSS systems via UDP, controls OBS Studio over WebSocket, and manages local video playback using mpv. With built-in automation, an intuitive manual mode, AI-ready architecture, and a licensing system tied to local hardware, it empowers referees to resolve match challenges within seconds.

### Target Users
- **Primary Users**: Taekwondo referees during live competitions
- **Secondary Users**: Tournament organizers and technical assistants
- **Usage Context**: Fast-paced match environments where decisions must be made within 20 seconds or less

### Core Features
- **UDP Server**: Listens to and parses datagrams from PSS systems
- **OBS Studio Control**: WebSocket control for multiple OBS instances
- **Video Playback**: Launch clips in `mpv` player with UI auto-hide/restore
- **Recording Archive**: Auto-named recordings with match metadata and visual timeline
- **Manual Mode**: Bypass UDP to directly control OBS and playback via UI
- **Local Storage**: SQLite DB stores parsed UDP events, metadata, and settings
- **AI Data Analyzer**: Parses and interprets structured event data from UDP
- **Licensing System**: One-time online activation tied to hardware
- **Logging System**: All logs saved in dedicated 'log' folder
- **Hardware Simulator**: Integrated PSS v2.3 protocol simulator for testing and development

### User Stories
- **As a referee**, I want the most recent recording to play instantly when a challenge is raised
- **As a referee**, I want a searchable list of previous clips organized by match
- **As a referee**, I want visual timelines on each recording showing significant moments
- **As a referee**, I want a simple interface with minimal input to avoid distractions
- **As a technical assistant**, I want to configure protocol formats and logging
- **As an organizer**, I want the license to be secure and tied to the machine

## Event Table System

### Overview
The Event Table system provides real-time display of PSS events with intelligent time and round tracking, manual override detection, and automatic event management.

### Recent Improvements (2025-01-29)

#### **Event Table Time & Round Display Fixes**
- **Persistent "2:00:00" Issue**: Resolved by removing hardcoded values from backend JSON event creation
- **Round Display**: Added 'RND' to importantEventCodes array to display round events
- **Time Accuracy**: Event Table now displays actual PSS event times instead of fallback values
- **Files Modified**:
  - `ui/src/components/molecules/EventTableSection.tsx` - Enhanced event filtering
  - `ui/src/hooks/useLiveDataEvents.ts` - Improved event processing
  - `ui/src/utils/pssEventHandler.ts` - Added fight_ready event handling

#### **Event Table Management**
- **Automatic Clearing**: Event Table clears automatically on `rdy;FightReady` events
- **Manual Button Removal**: Removed Clear Events buttons from UI components
- **Counter Behavior**: Verified correct behavior (Round/Time preserved, Total/Table reset)
- **Clean UI**: Simplified interface with automatic event management

#### **Manual Override Detection System**
- **Event Sequence Tracking**: Replaced time-based threshold with event sequence analysis
- **Break Event Exception**: Round changes after `brk;0:00;stopEnd` are NOT manual override
- **Normal Pattern**: `brk;0:00;stopEnd` → `rnd;3` → `clk;02:00;start`
- **Detection Logic**: Manual override only when other events occur between break stopEnd and round

### Technical Implementation

#### **Event Table Component**
- **File**: `ui/src/components/molecules/EventTableSection.tsx`
- **Features**: Real-time event display, filtering, auto-scroll
- **Counters**: Round, Time, Total, Table counters with proper state management
- **Filtering**: Color and event type filtering with important event codes

#### **Live Data Store**
- **File**: `ui/src/stores/liveDataStore.ts`
- **State Management**: Events array, current round/time, connection status
- **Actions**: addEvent, clearEvents, setCurrentRound, setCurrentRoundTime
- **Computed**: getFilteredEvents, getEventsByRound

#### **WebSocket Integration**
- **File**: `ui/src/hooks/useLiveDataEvents.ts`
- **Connection**: Singleton WebSocket connection to backend
- **Event Processing**: Real-time PSS event processing and store updates
- **Manual Mode**: Automatic disconnection when manual mode is enabled

### Scoreboard Overlay Integration

#### **Manual Override Detection**
- **File**: `ui/public/scoreboard-overlay.html`
- **State Tracking**: Clock state, manual override mode, event sequence
- **Exception Handling**: Break event exception for normal inter-round changes
- **Debug Logging**: Comprehensive logging for manual override detection

#### **Event Processing**
- **Clock Events**: Proper state management for start/stop actions
- **Round Events**: Enhanced round change detection with exception handling
- **Break Events**: Tracking of break stopEnd events for manual override exception

## Flag Management System

### Overview
The Flag Management System provides comprehensive IOC (International Olympic Committee) flag support with 253 flags covering current NOCs, historic NOCs, and special Olympic/Paralympic codes.

### System Status
- **Database Migration**: Successfully migrated to database-backed flag management system
- **253 IOC Entries**: Automatically populated into `flag_mappings` table
- **Frontend Integration**: Complete React UI integration with database toggle
- **Real-time Updates**: PSS code synchronization and live flag management

### Flag Collection Statistics

#### Total Flags: 253
- **Current NOCs (Table 1)**: 206 flags - Main Olympic countries
- **Additional Territories (Table 2)**: 2 flags - Faroe Islands, Macau
- **Historic NOCs (Table 3)**: 12 flags - Soviet Union, Yugoslavia, East/West Germany
- **Historic Country Names (Table 4)**: 18 flags - Burma, Ceylon, Zaire, etc.
- **Special Olympic Codes (Table 5)**: 10 flags - Refugee Olympic Team, Independent Athletes
- **Special Paralympic Codes (Table 6)**: 5 flags - Refugee Paralympic Team, etc.

#### By Region
- **Africa**: 47 flags
- **Asia**: 13 flags  
- **Europe**: 48 flags
- **North America**: 21 flags
- **Oceania**: 8 flags
- **South America**: 16 flags
- **Historic/Special**: 100 flags

### Technical Implementation

#### Database Schema
- **`flags`**: Flag file metadata (id, filename, file_size, upload_date, recognition_status, etc.)
- **`flag_mappings`**: IOC to PSS code mappings (253+ entries populated automatically)
- **`recognition_history`**: Historical flag recognition data
- **`settings`**: Flag management system settings

#### Tauri Commands
- **`get_flag_mappings_data`**: Retrieve flag mapping statistics
- **`scan_and_populate_flags`**: Scan filesystem and populate flags table
- **`get_flags_data`**: Retrieve flag metadata from database
- **`clear_flags_table`**: Clear flags table for re-scanning

#### Frontend Integration
- **Database Toggle**: Switch between file-based and database-backed flag loading
- **Real-time Statistics**: Live display of flag counts and recognition status
- **PSS Code Synchronization**: Proper update of PSS codes when selecting flags
- **File Management**: Scan, populate, and clear flag database operations

#### React Components
- **File**: `ui/src/utils/flagUtils.tsx`
- **Components**: `FlagImage`, `getFlagConfig`, `getFlagUrl`
- **Fallbacks**: Emoji flags for all 253 IOC codes
- **Error Handling**: Automatic fallback to emoji on image load failure

### Simulation Integration

#### Overview
The frontend includes comprehensive simulation support through a dedicated Simulation tab in the PSS drawer, providing one-click access to the tkStrike Hardware Simulator.

#### Simulation Panel Component
- **File**: `ui/src/components/molecules/SimulationPanel.tsx`
- **Integration**: PSS drawer tab with robot animation icon
- **Features**: Real-time status monitoring, scenario selection, manual event control, enhanced error handling

#### Key Features
- **Status Monitoring**: Real-time display of simulation status and connection state
- **Scenario Control**: Dropdown selection for basic, championship, and training matches
- **Mode Selection**: Demo, random events, and interactive modes
- **Manual Events**: One-click buttons for points, warnings, and injury time
- **Duration Control**: Configurable simulation duration (10-600 seconds)
- **Enhanced Error Handling**: User-friendly error messages with actionable solutions
- **Dependency Management**: Retry and install dependency buttons for Python issues
- **Progress Indicators**: Loading states during dependency installation

#### Tauri Integration
```typescript
// Simulation commands
const startSimulation = async () => {
  const result = await invoke('simulation_start', {
    mode: selectedMode,
    scenario: selectedScenario,
    duration: duration
  });
};

const stopSimulation = async () => {
  const result = await invoke('simulation_stop');
};

const sendManualEvent = async (eventType: string, params: any) => {
  const result = await invoke('simulation_send_event', {
    eventType,
    params
  });
};
```

#### UI Components
- **Status Indicators**: Green/red dots for running/stopped states
- **Control Buttons**: Start/stop simulation with loading states
- **Event Buttons**: Manual event generation (Blue Punch, Red Head Kick, etc.)
- **Real-time Updates**: 2-second polling for status updates
- **Error Handling**: Comprehensive error display and success messages
- **Simulation Environment Errors**: Special handling for Python/dependency issues
- **Actionable Error Messages**: Clear instructions with retry and install buttons
- **Loading States**: Progress indicators during dependency installation

#### Integration Points
- **PSS Drawer**: Simulation tab with robot animation icon
- **Event Table**: Real-time event display from simulator
- **Scoreboard Overlay**: Live score updates from simulation events
- **WebSocket**: Real-time event broadcasting to connected clients

#### Enhanced Error Handling
The simulation components now include robust error handling for environment issues:

```typescript
// Error detection and user-friendly messages
const isSimulationEnvError = (errorMsg: string): boolean => {
  return errorMsg.includes('Simulation environment error') || 
         errorMsg.includes('PythonNotFound') ||
         errorMsg.includes('PythonVersionTooLow') ||
         errorMsg.includes('PipInstallFailed') ||
         errorMsg.includes('DependencyCheckFailed') ||
         errorMsg.includes('SimulationPathNotFound');
};

// Actionable error messages with retry/install buttons
const getFriendlyErrorMessage = (errorMsg: string): string => {
  if (errorMsg.includes('PythonNotFound')) {
    return 'Python is not installed or not found in PATH. Please install Python 3.8 or higher.';
  }
  // ... other error mappings
};
```

**Error Handling Features:**
- **Automatic Detection**: Environment issues detected before simulation starts
- **User-Friendly Messages**: Clear instructions instead of technical error codes
- **Retry Buttons**: One-click retry for connection issues
- **Install Dependencies**: Automatic Python package installation
- **Progress Feedback**: Loading indicators during installation
- **Fallback Categories**: Self-test categories available even if backend fails

#### Flag Storage
- **Directory**: `ui/public/assets/flags/`
- **Format**: PNG images
- **Naming**: `{IOC}.png` (e.g., `USA.png`, `GBR.png`)
- **Size**: Optimized 40px width from Wikipedia

## Architecture

### Core Architecture
- **React 18**: Modern React with hooks and functional components
- **TypeScript**: Full type safety throughout the application
- **Tailwind CSS**: Utility-first CSS framework for styling
- **Tauri API**: Native desktop integration with event system
- **Atomic Design**: Organized component architecture
- **Zustand**: State management for UI components

### Technology Stack
- **Framework**: React 18 with TypeScript
- **Styling**: Tailwind CSS with custom design system
- **State Management**: Zustand for global state
- **Build System**: Vite with Tauri integration
- **Development**: Hot reload with development server
- **Testing**: Jest and React Testing Library

## Directory Structure

```
ui/
├── src/
│   ├── App.tsx              # Main application component
│   ├── index.tsx            # React entry point
│   ├── index.css            # Global styles
│   ├── components/          # Atomic design components
│   │   ├── atoms/           # Basic UI elements
│   │   │   ├── Button.tsx   # Button component
│   │   │   ├── Input.tsx    # Input component
│   │   │   ├── Checkbox.tsx # Checkbox component
│   │   │   ├── Label.tsx    # Label component
│   │   │   ├── StatusDot.tsx # Status indicator
│   │   │   ├── Icon.tsx     # Icon component
│   │   │   ├── Tab.tsx      # Tab component
│   │   │   └── TabGroup.tsx # Tab group component
│   │   ├── molecules/       # Compound components
│   │   │   ├── EventTableSection.tsx # Event table section
│   │   │   ├── LiveDataPanel.tsx # Live data display
│   │   │   │   ├── CpuMonitoringSection.tsx # CPU monitoring
│   │   │   ├── LogDownloadList.tsx # Log download interface
│   │   │   ├── FlagManagementPanel.tsx # Flag management interface
│   │   │   ├── PasswordDialog.tsx # Authentication dialog
│   │   │   ├── ManualModeDialog.tsx # Manual mode dialog
│   │   │   ├── WebSocketManager.tsx # OBS WebSocket connection management
│   │   │   ├── ControlRoom.tsx # Control Room STR management interface
│   │   │   ├── PssDrawer.tsx # PSS drawer with tabs
│   │   │   └── ObsDrawer.tsx # OBS drawer with tabs
│   │   ├── organisms/       # Complex components
│   │   │   ├── EventTable.tsx # Event table organism
│   │   │   ├── MatchInfoSection.tsx # Match information
│   │   │   ├── ObsWebSocketManager.tsx # OBS manager
│   │   │   ├── SidebarSmall.tsx # Small sidebar
│   │   │   └── SidebarBig.tsx # Large sidebar
│   │   └── layouts/         # Layout components
│   │       ├── DockBar.tsx  # Main sidebar layout
│   │       ├── AdvancedPanel.tsx # Advanced panel layout
│   │       └── StatusbarAdvanced.tsx # Status bar layout
│   ├── hooks/               # Custom React hooks
│   │   ├── useEnvironment.ts # Environment detection
│   │   ├── useEnvironmentApi.ts # API environment
│   │   ├── useEnvironmentObs.ts # OBS environment
│   │   ├── usePssEvents.ts  # Real-time PSS event handling
│   │   └── useLiveDataEvents.ts # Live data streaming
│   ├── stores/              # State management
│   │   ├── index.ts         # Store exports
│   │   ├── liveDataStore.ts # Live data state
│   │   ├── obsStore.ts      # OBS state management
│   │   └── pssMatchStore.ts # PSS match state
│   ├── types/               # TypeScript types
│   │   ├── index.ts         # Type exports
│   │   └── tauri.d.ts       # Tauri type definitions
│   ├── utils/               # Utility functions
│   │   ├── flagUtils.tsx    # Flag utility functions
│   │   ├── obsUtils.ts      # OBS utility functions
│   │   ├── tauriCommands.ts # Tauri command utilities
│   │   ├── videoUtils.ts    # Video utility functions
│   │   ├── pssEventHandler.ts # PSS event handling
│   │   └── countryCodeMapping.ts # PSS to IOC mapping
│   ├── config/              # Frontend configuration
│   │   └── environments/    # Environment configurations
│   │       ├── web.ts       # Web environment
│   │       └── windows.ts   # Windows environment
│   └── lib/                 # Library utilities
│       └── index.ts         # Library exports
├── public/                  # Static assets
│   ├── index.html           # HTML template
│   └── assets/              # Static assets
│       ├── flags/           # 253+ IOC country flag images
│       │   ├── AFG.png      # Afghanistan flag
│       │   ├── AUS.png      # Australia flag
│       │   ├── USA.png      # United States flag
│       │   └── ...          # 250+ more flag images
│       └── img/             # Other images
│           └── logo.png     # Application logo
├── package.json             # Node.js dependencies
├── package-lock.json        # Dependency lock file
├── tsconfig.json            # TypeScript configuration
├── tailwind.config.js       # Tailwind CSS configuration
├── postcss.config.js        # PostCSS configuration
└── eslint.config.js         # ESLint configuration
```

## UI Design Guidelines and Requirements

### Design System Foundation

The frontend follows a comprehensive design system optimized for taekwondo competition management:

#### Visual Style
- **Dark Theme**: Optimized for arenas and low-light environments
- **High Contrast**: Ensures readability in various lighting conditions
- **Touch-Friendly**: Large UI elements for fast input during competitions
- **Professional Appearance**: Clean, modern interface for tournament environments

#### Color Palette
```css
/* Primary Colors */
--primary-red: #dc2626;      /* Recording, alerts */
--primary-blue: #2563eb;     /* Navigation, playback */
--primary-green: #16a34a;    /* Connected/Ready */
--primary-gray: #6b7280;     /* Inactive items */

/* Background Colors */
--bg-main: #111827;          /* Main app background */
--bg-dockbar: rgba(0, 0, 0, 0.6); /* DockBar background */
--bg-panel: rgba(31, 41, 55, 0.8); /* Advanced panel background */
--bg-sidebar: rgba(55, 65, 81, 0.9); /* Sidebar background */

/* Modern Gradient Style */
--gradient-panel: linear-gradient(135deg, rgba(31, 41, 55, 0.8) 0%, rgba(17, 24, 39, 0.9) 100%);
--gradient-border: rgba(75, 85, 99, 0.3);
```

#### Typography
```css
/* Font Stack */
font-family: 'Inter', 'Segoe UI', 'Roboto', system-ui, sans-serif;

/* Font Sizes */
--text-xs: 0.75rem;    /* 12px */
--text-sm: 0.875rem;   /* 14px */
--text-base: 1rem;     /* 16px */
--text-lg: 1.125rem;   /* 18px */
--text-xl: 1.25rem;    /* 20px */
--text-2xl: 1.5rem;    /* 24px */

/* Font Weights */
--font-normal: 400;
--font-medium: 500;
--font-semibold: 600;
--font-bold: 700;
```

### Layout Structure

#### Main Application Layout
```tsx
// Main app layout with responsive design
<div className="h-screen flex flex-col bg-gray-900">
  <div className="flex flex-1 min-h-0">
    <div className="w-[350px] flex-shrink-0"> {/* DockBar */}
      <DockBar />
    </div>
    <div className="flex-1 min-h-0"> {/* Advanced Panel */}
      <AdvancedPanel />
    </div>
  </div>
</div>
```

#### DockBar Layout (Two-Column Design)
```tsx
// DockBar with SidebarSmall and SidebarBig
<div className="flex flex-row h-full bg-black/60">
  <div className="w-24 flex-shrink-0"> {/* SidebarSmall */}
    <SidebarSmall />
  </div>
  <div className="flex-1 min-h-0"> {/* SidebarBig */}
    <SidebarBig />
  </div>
</div>
```

#### Advanced Panel Layout
```tsx
// Advanced panel with sidebar and main content
<div className="flex flex-row h-full bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm">
  <div className="w-64 bg-gray-700/90"> {/* Sidebar */}
    <AdvancedSidebar />
  </div>
  <div className="flex-1 bg-gray-800/60"> {/* Main content */}
    <AdvancedContent />
  </div>
</div>
```

### Component System

#### Atomic Design Implementation

**Atoms (Basic Components)**
```tsx
// Button component with variants
interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  children: React.ReactNode;
  onClick?: () => void;
}

// StatusDot component for status indicators
interface StatusDotProps {
  status: 'connected' | 'disconnected' | 'error' | 'warning';
  size?: 'sm' | 'md' | 'lg';
  label?: string;
}

// Tab component for navigation
interface TabProps {
  icon: React.ReactNode;
  label: string;
  isActive?: boolean;
  onClick?: () => void;
}
```

**Molecules (Compound Components)**
```tsx
// EventTableSection with filtering and display
interface EventTableSectionProps {
  events: PssEvent[];
  onEventSelect?: (event: PssEvent) => void;
  filters?: EventFilters;
}

// FlagManagementPanel with database integration
interface FlagManagementPanelProps {
  useDatabase: boolean;
  onToggleDatabase: (useDatabase: boolean) => void;
  flagMappingsData?: FlagMappingsData;
}

// PasswordDialog for authentication
interface PasswordDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onAuthenticate: (password: string) => boolean;
  title?: string;
  message?: string;
}
```

**Organisms (Complex Components)**
```tsx
// EventTable with real-time updates
interface EventTableProps {
  events: PssEvent[];
  onEventClick?: (event: PssEvent) => void;
  onFilterChange?: (filters: EventFilters) => void;
}

// ObsWebSocketManager with connection management
interface ObsWebSocketManagerProps {
  connections: ObsConnection[];
  onConnect: (connection: ObsConnection) => void;
  onDisconnect: (connectionName: string) => void;
}
```

### Interactive Elements

#### Red Blinking Record Button
```tsx
// Pulsating record button for OBS recording status
const RecordButton: React.FC<{ isRecording: boolean }> = ({ isRecording }) => (
  <button
    className={`
      w-16 h-16 rounded-full border-4 border-red-500
      ${isRecording 
        ? 'bg-red-500 animate-pulse shadow-lg shadow-red-500/50' 
        : 'bg-transparent hover:bg-red-500/20'
      }
      transition-all duration-300 ease-in-out
    `}
    onClick={handleRecordToggle}
  >
    <Icon name="record" className="w-8 h-8 text-white" />
  </button>
);
```

#### Big Action Buttons
```tsx
// Large, touch-friendly action buttons
const ActionButton: React.FC<ActionButtonProps> = ({ 
  label, 
  icon, 
  onClick, 
  variant = 'primary' 
}) => (
  <button
    className={`
      px-6 py-4 rounded-lg text-lg font-semibold
      flex items-center justify-center space-x-3
      min-h-[60px] min-w-[120px]
      transition-all duration-200 ease-in-out
      ${getVariantStyles(variant)}
    `}
    onClick={onClick}
  >
    <Icon name={icon} className="w-6 h-6" />
    <span>{label}</span>
  </button>
);
```

#### Status Panel
```tsx
// Real-time status display
const StatusPanel: React.FC<StatusPanelProps> = ({ 
  matchInfo, 
  connectionStatus, 
  licenseStatus 
}) => (
  <div className="bg-gray-800/80 rounded-lg p-4 space-y-3">
    <div className="flex items-center justify-between">
      <span className="text-gray-300">Match #{matchInfo.matchNumber}</span>
      <StatusDot status={connectionStatus} />
    </div>
    <div className="flex items-center space-x-4">
      <FlagImage countryCode={matchInfo.athlete1Country} />
      <span className="text-white font-medium">{matchInfo.athlete1Name}</span>
      <span className="text-gray-400">vs</span>
      <span className="text-white font-medium">{matchInfo.athlete2Name}</span>
      <FlagImage countryCode={matchInfo.athlete2Country} />
    </div>
  </div>
);
```

### Responsive Design

#### Breakpoint System
```css
/* Tailwind breakpoints */
--breakpoint-sm: 640px;   /* Small devices */
--breakpoint-md: 768px;   /* Medium devices */
--breakpoint-lg: 1024px;  /* Large devices */
--breakpoint-xl: 1280px;  /* Extra large devices */
--breakpoint-2xl: 1536px; /* 2X large devices */
```

#### Adaptive Layouts
```tsx
// Responsive layout with adaptive sizing
const ResponsiveLayout: React.FC = ({ children }) => (
  <div className="
    w-full h-full
    flex flex-col lg:flex-row
    space-y-4 lg:space-y-0 lg:space-x-4
    p-4 lg:p-6
  ">
    {children}
  </div>
);
```

### Animation and Transitions

#### Smooth Transitions
```css
/* Global transition classes */
.transition-all {
  transition: all 0.2s ease-in-out;
}

.transition-colors {
  transition: color 0.2s ease-in-out, background-color 0.2s ease-in-out;
}

.transition-transform {
  transition: transform 0.2s ease-in-out;
}
```

#### Component Animations
```tsx
// Animated component with framer-motion
import { motion } from 'framer-motion';

const AnimatedPanel: React.FC = ({ children, isVisible }) => (
  <motion.div
    initial={{ opacity: 0, y: 20 }}
    animate={{ 
      opacity: isVisible ? 1 : 0, 
      y: isVisible ? 0 : 20 
    }}
    transition={{ duration: 0.3, ease: "easeOut" }}
    className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg"
  >
    {children}
  </motion.div>
);
```

### Accessibility

#### Keyboard Navigation
```tsx
// Keyboard-accessible components
const KeyboardAccessibleButton: React.FC<ButtonProps> = ({ 
  children, 
  onClick, 
  ...props 
}) => (
  <button
    onClick={onClick}
    onKeyDown={(e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        onClick?.();
      }
    }}
    className="focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
    {...props}
  >
    {children}
  </button>
);
```

#### Screen Reader Support
```tsx
// Screen reader-friendly components
const AccessibleStatusIndicator: React.FC<StatusProps> = ({ 
  status, 
  label 
}) => (
  <div
    role="status"
    aria-live="polite"
    aria-label={`${label}: ${status}`}
    className="flex items-center space-x-2"
  >
    <StatusDot status={status} />
    <span className="sr-only">{label}: {status}</span>
  </div>
);
```

### Flag System Integration

#### Flag Display Components
```tsx
// Flag image component with fallback
const FlagImage: React.FC<FlagImageProps> = ({ 
  countryCode, 
  className = "w-8 h-6 object-cover rounded-sm shadow-sm" 
}) => {
  const flagConfig = getFlagConfig(countryCode);
  
  return (
    <img
      src={getFlagUrl(countryCode)}
      alt={`Flag of ${flagConfig.countryName}`}
      className={className}
      onError={(e) => {
        // Fallback to emoji flag
        e.currentTarget.style.display = 'none';
        const emojiSpan = document.createElement('span');
        emojiSpan.textContent = flagConfig.fallbackEmoji;
        emojiSpan.className = className;
        e.currentTarget.parentNode?.appendChild(emojiSpan);
      }}
    />
  );
};
```

#### Flag Management Interface
```tsx
// Complete flag management panel
const FlagManagementPanel: React.FC = () => {
  const [useDatabase, setUseDatabase] = useState(false);
  const [flagMappingsData, setFlagMappingsData] = useState<FlagMappingsData>();
  
  return (
    <div className="space-y-6">
      {/* Database Toggle */}
      <div className="flex items-center justify-between">
        <Label>Use Database for Flags</Label>
        <Toggle
          checked={useDatabase}
          onChange={setUseDatabase}
        />
      </div>
      
      {/* Statistics */}
      {flagMappingsData && (
        <div className="grid grid-cols-2 gap-4">
          <StatCard
            title="Total Flags"
            value={flagMappingsData.totalFlags}
          />
          <StatCard
            title="Active Mappings"
            value={flagMappingsData.activeMappings}
          />
        </div>
      )}
      
      {/* Flag Operations */}
      <div className="space-y-3">
        <Button onClick={handleScanFlags}>
          Scan and Populate Flags
        </Button>
        <Button onClick={handleClearFlags} variant="danger">
          Clear Flags Database
        </Button>
      </div>
    </div>
  );
};
```

### Real-Time Features

#### Live Data Streaming
```tsx
// Real-time data display with auto-scroll
const LiveDataPanel: React.FC<LiveDataPanelProps> = ({ 
  data, 
  autoScroll = true 
}) => {
  const scrollRef = useRef<HTMLDivElement>(null);
  
  useEffect(() => {
    if (autoScroll && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [data, autoScroll]);
  
  return (
    <div className="bg-gray-900 rounded-lg border border-gray-700">
      <div className="p-3 border-b border-gray-700">
        <h3 className="text-lg font-semibold text-white">Live Data</h3>
      </div>
      <div
        ref={scrollRef}
        className="h-64 overflow-y-auto p-3 font-mono text-sm"
      >
        {data.map((entry, index) => (
          <div key={index} className="text-gray-300 mb-1">
            {entry}
          </div>
        ))}
      </div>
    </div>
  );
};
```

#### Event Table with Real-Time Updates
```tsx
// Real-time event table with filtering
const EventTable: React.FC<EventTableProps> = ({ 
  events, 
  onEventClick 
}) => {
  const [filteredEvents, setFilteredEvents] = useState(events);
  const [filters, setFilters] = useState<EventFilters>({});
  
  return (
    <div className="bg-gray-900 rounded-lg border border-gray-700">
      {/* Header */}
      <div className="p-4 border-b border-gray-700">
        <h3 className="text-lg font-semibold text-white">Events</h3>
      </div>
      
      {/* Filters */}
      <div className="p-4 border-b border-gray-700">
        <EventFilters filters={filters} onChange={setFilters} />
      </div>
      
      {/* Table */}
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead className="bg-gray-800 sticky top-0 z-10">
            <tr>
              <th className="px-4 py-2 text-left text-gray-300">Time</th>
              <th className="px-4 py-2 text-left text-gray-300">Event</th>
              <th className="px-4 py-2 text-left text-gray-300">Details</th>
            </tr>
          </thead>
          <tbody>
            {filteredEvents.map((event) => (
              <tr
                key={event.id}
                className="border-b border-gray-700 hover:bg-gray-800 transition-colors cursor-pointer"
                onClick={() => onEventClick?.(event)}
              >
                <td className="px-4 py-2 text-gray-300">
                  {formatTime(event.timestamp)}
                </td>
                <td className="px-4 py-2 text-white">
                  {event.eventName}
                </td>
                <td className="px-4 py-2 text-gray-400">
                  {event.details}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
```

### State Management

#### Zustand Stores
```tsx
// Live data store for real-time updates
interface LiveDataStore {
  data: string[];
  isConnected: boolean;
  addData: (data: string) => void;
  clearData: () => void;
  setConnectionStatus: (status: boolean) => void;
}

const useLiveDataStore = create<LiveDataStore>((set) => ({
  data: [],
  isConnected: false,
  addData: (newData) => set((state) => ({
    data: [...state.data, newData].slice(-1000) // Keep last 1000 entries
  })),
  clearData: () => set({ data: [] }),
  setConnectionStatus: (status) => set({ isConnected: status }),
}));

// OBS store for connection management
interface ObsStore {
  connections: ObsConnection[];
  activeConnection: string | null;
  addConnection: (connection: ObsConnection) => void;
  removeConnection: (name: string) => void;
  setActiveConnection: (name: string | null) => void;
}
```

### Environment Detection

#### Tauri vs Web Mode
```tsx
// Environment detection hook
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

### Performance Optimization

#### Component Optimization
```tsx
// Memoized components for performance
const OptimizedEventTable = React.memo<EventTableProps>(({ 
  events, 
  onEventClick 
}) => {
  // Component implementation
});

// Custom hooks for performance
const useDebouncedCallback = (callback: Function, delay: number) => {
  const timeoutRef = useRef<NodeJS.Timeout>();
  
  return useCallback((...args: any[]) => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    timeoutRef.current = setTimeout(() => callback(...args), delay);
  }, [callback, delay]);
};
```

#### Bundle Optimization
```json
// package.json optimization scripts
{
  "scripts": {
    "build:fast": "GENERATE_SOURCEMAP=false react-scripts build",
    "start:fast": "GENERATE_SOURCEMAP=false react-scripts start",
    "analyze": "source-map-explorer 'build/static/js/*.js'",
    "clean:all": "rm -rf node_modules/.cache build"
  }
}
```

### Testing Strategy

#### Component Testing
```tsx
// Component test example
import { render, screen, fireEvent } from '@testing-library/react';
import { EventTable } from './EventTable';

describe('EventTable', () => {
  it('renders events correctly', () => {
    const mockEvents = [
      { id: 1, eventName: 'Test Event', timestamp: new Date(), details: 'Test Details' }
    ];
    
    render(<EventTable events={mockEvents} />);
    
    expect(screen.getByText('Test Event')).toBeInTheDocument();
    expect(screen.getByText('Test Details')).toBeInTheDocument();
  });
  
  it('calls onEventClick when event is clicked', () => {
    const mockEvents = [{ id: 1, eventName: 'Test Event', timestamp: new Date(), details: 'Test Details' }];
    const mockOnClick = jest.fn();
    
    render(<EventTable events={mockEvents} onEventClick={mockOnClick} />);
    
    fireEvent.click(screen.getByText('Test Event'));
    expect(mockOnClick).toHaveBeenCalledWith(mockEvents[0]);
  });
});
```

### Development Guidelines

#### Code Quality Standards
- **TypeScript**: Full type safety throughout
- **ESLint**: Consistent code style and best practices
- **Prettier**: Automatic code formatting
- **Testing**: Comprehensive unit and integration tests

#### Component Development
- **Atomic Design**: Follow atomic design principles
- **Props Interface**: Define clear prop interfaces
- **Error Boundaries**: Implement error boundaries for robustness
- **Accessibility**: Ensure accessibility compliance

#### Performance Guidelines
- **Memoization**: Use React.memo and useMemo appropriately
- **Lazy Loading**: Implement lazy loading for large components
- **Bundle Size**: Monitor and optimize bundle size
- **Caching**: Implement appropriate caching strategies

## 🎯 Scoreboard Overlay System

### Overview
The scoreboard overlay system provides real-time display of match information for broadcasting and arena displays. The overlay is designed as a standalone HTML application that receives WebSocket events and updates the scoreboard display in real-time.

### Architecture

#### **File Structure**
```
ui/public/
├── scoreboard-overlay.html          # Main scoreboard overlay
├── player-introduction-overlay.html # Player introduction overlay
├── websocket-debug.html            # WebSocket debugging tool
└── assets/scoreboard/
    ├── scoreboard-overlay.svg      # Scoreboard SVG template
    ├── scoreboard-utils.js         # Scoreboard utility functions
    └── scoreboard-name-utils.js    # Name management utilities
```

#### **Core Components**

**1. ScoreboardOverlay Class**
```javascript
class ScoreboardOverlay {
    constructor(svgElement) {
        this.svg = svgElement;
        this.initializeElements();
    }
    
    // Core update methods
    updateScore(player, score) { /* Update player scores */ }
    updateRoundWins(player, wins) { /* Update round wins */ }
    updateRound(round) { /* Update current round */ }
    updateTimer(minutes, seconds) { /* Update match timer */ }
    updatePenalties(player, penalties) { /* Update warnings */ }
    updatePlayerName(player, name) { /* Update player names */ }
    updateInjuryTime(time) { /* Update injury time display */ }
}
```

**2. Manual Override Detection System**
```javascript
// Robust manual override detection with panic prevention
const manualOverrideState = {
    clockState: 'stopped',
    recentEvents: [],
    lastBlueScore: 0,
    lastRedScore: 0,
    lastBlueWarnings: 0,
    lastRedWarnings: 0,
    timeCorrectionThreshold: 5000
};

// Detection functions with comprehensive error handling
function isManualRoundChange(event) {
    try {
        if (isClockStopped() && event.type === 'round') {
            return true;
        }
        // Additional detection logic...
        return false;
    } catch (error) {
        console.warn('⚠️ Error detecting manual round change:', error);
        return false;
    }
}
```

### Manual Override Detection

#### **Detection Methods**

**1. Manual Round Change Detection**
- **Round changes during stopped clock**: Indicates manual intervention
- **Rapid round changes**: Multiple round changes within 5 seconds
- **Pattern analysis**: Unusual round change patterns

**2. Manual Score Change Detection**
- **Score changes during stopped clock**: Manual intervention during paused time
- **Score changes during time correction**: Changes made during `clk;XX:XX;corr` events
- **Large score jumps**: Unusual score increases (3+ points at once)
- **Rapid point messages**: Multiple point events within 2 seconds

**3. Manual Time Change Detection**
- **Time correction events**: `clk;XX:XX;corr` messages indicate manual time adjustments
- **Time changes during stopped periods**: Unusual time modifications

**4. Manual Warning Change Detection**
- **Warning changes during stopped clock**: Manual intervention during paused time
- **Warning changes during time correction**: Changes made during time correction events
- **Rapid warning changes**: Multiple warning events within 3 seconds

#### **Handling Strategies**

**1. Manual Round Change Handling**
```javascript
function handleManualRoundChange(event) {
    // Update round number but preserve all other data
    scoreboardInstance.updateRound(event.current_round);
    
    // DO NOT reset scores, warnings, or other data
    // This is the key difference from normal round changes
}
```

**2. Manual Score Change Handling**
```javascript
function handleManualScoreChange(event, newBlueScore, newRedScore) {
    // Update scores immediately
    scoreboardInstance.updateScores(newBlueScore, newRedScore);
    
    // Update tracking state
    manualOverrideState.lastBlueScore = newBlueScore;
    manualOverrideState.lastRedScore = newRedScore;
}
```

**3. Manual Time Change Handling**
```javascript
function handleManualTimeChange(event) {
    // Update time immediately
    const timeParts = event.time.split(':');
    const minutes = parseInt(timeParts[0]) || 0;
    const seconds = parseInt(timeParts[1]) || 0;
    scoreboardInstance.updateTimer(minutes, seconds);
}
```

**4. Manual Warning Change Handling**
```javascript
function handleManualWarningChange(event, newBlueWarnings, newRedWarnings) {
    // Update warnings immediately
    scoreboardInstance.updatePenalties(newBlueWarnings, newRedWarnings);
    
    // Update tracking state
    manualOverrideState.lastBlueWarnings = newBlueWarnings;
    manualOverrideState.lastRedWarnings = newRedWarnings;
}
```

### Error Handling and Safety

#### **Panic Prevention**
- **Try-catch blocks**: All detection functions wrapped in error handling
- **Safe utility functions**: Fallback values for all operations
- **Defensive programming**: Safe data access patterns
- **Graceful degradation**: Continue operation when errors occur

#### **Robust Error Handling**
```javascript
// Safely get current timestamp
function getCurrentTimestamp() {
    try {
        return Date.now();
    } catch (error) {
        console.warn('⚠️ Error getting timestamp:', error);
        return 0;
    }
}

// Safely add event to recent events
function addToRecentEvents(event) {
    try {
        manualOverrideState.recentEvents.push({
            event: event,
            timestamp: getCurrentTimestamp()
        });
        
        // Keep only the last N events
        if (manualOverrideState.recentEvents.length > maxRecentEvents) {
            manualOverrideState.recentEvents.shift();
        }
    } catch (error) {
        console.warn('⚠️ Error adding to recent events:', error);
    }
}
```

### Event Processing Integration

#### **Main Event Handler**
```javascript
function handlePssEvent(event) {
    // Add event to recent events for pattern detection
    addToRecentEvents(event);
    
    // Update clock state tracking
    if (event.type === 'clock') {
        updateClockState(event);
    }
    
    // Check for manual overrides BEFORE normal processing
    if (isManualTimeChange(event)) {
        handleManualTimeChange(event);
        return;
    }
    
    if (isManualRoundChange(event)) {
        handleManualRoundChange(event);
        return;
    }
    
    // Normal event processing
    processNormalEvent(event);
}
```

#### **Scoreboard Integration**
```javascript
function handleScoresEvent(event) {
    // Check for manual score change
    if (isManualScoreChange(event, blueScore, redScore)) {
        handleManualScoreChange(event, blueScore, redScore);
    } else {
        // Normal score update
        scoreboardInstance.updateScores(blueScore, redScore);
    }
}
```

### Benefits and Impact

#### **1. Accurate Manual Change Detection**
- **Reliable detection**: Multiple detection methods for accuracy
- **Pattern recognition**: Intelligent pattern analysis
- **Context awareness**: Clock state and timing awareness

#### **2. Proper Data Handling**
- **Data preservation**: Manual round changes preserve all data
- **Immediate acceptance**: Manual changes accepted immediately
- **Real-time updates**: Instant scoreboard updates

#### **3. System Reliability**
- **Panic-free operation**: Comprehensive error handling
- **Graceful degradation**: Continue operation during errors
- **Robust state tracking**: Reliable state management

#### **4. User Experience**
- **Seamless operation**: Manual changes work as expected
- **No data loss**: All data preserved during manual changes
- **Real-time feedback**: Immediate visual updates

### WebSocket Integration

#### **Event Reception**
```javascript
// WebSocket connection for real-time events
const websocket = new WebSocket('ws://localhost:8080');

websocket.onmessage = function(event) {
    try {
        const data = JSON.parse(event.data);
        handlePssEvent(data);
    } catch (error) {
        console.error('❌ Error parsing WebSocket message:', error);
    }
};
```

#### **Connection Management**
```javascript
// Connection status tracking
function updateConnectionStatus(connected) {
    const statusElement = document.getElementById('connection-status');
    if (connected) {
        statusElement.classList.add('connected');
    } else {
        statusElement.classList.remove('connected');
    }
}
```

### Styling and Design

#### **Visual Design**
- **High contrast**: Optimized for arena displays
- **Large text**: Readable from distance
- **Professional appearance**: Clean, modern interface
- **Responsive layout**: Adapts to different screen sizes

#### **Animation System**
```css
/* Score update animations */
.score-update {
    animation: scorePulse 0.5s ease-in-out;
}

@keyframes scorePulse {
    0% { transform: scale(1); }
    50% { transform: scale(1.1); }
    100% { transform: scale(1); }
}
```

---



---



---

## 🎛️ Control Room Frontend Implementation ✅ **NEW**

### **Control Room Frontend Architecture**

The Control Room provides centralized management of multiple STR (streaming) OBS instances with secure authentication and real-time status monitoring.

#### **Control Room Tab Integration**
```typescript
// OBS Drawer Tab Structure
const obsDrawerTabs = [
  {
    id: 'websocket',
    label: 'WebSocket',
    content: <WebSocketManager />
  },
  {
    id: 'control-room',
    label: 'Control Room',
    content: <ControlRoom />
  },
  {
    id: 'integration',
    label: 'Integration',
    content: <ObsIntegrationSettings />
  }
];
```

### **Control Room Frontend Component Structure**

#### **ControlRoom.tsx - Main Interface**
```typescript
interface ControlRoomState {
  isAuthenticated: boolean;
  isLoading: boolean;
  sessionId: string | null;
  password: string;
  connections: StrConnection[];
  showAddConnection: boolean;
  newConnection: {
    name: string;
    host: string;
    port: string;
    password: string;
    notes: string;
  };
}

interface StrConnection {
  name: string;
  host: string;
  port: number;
  status: 'Disconnected' | 'Connecting' | 'Connected' | 'Error';
  notes?: string;
}
```

#### **Authentication Section**
```typescript
const AuthenticationSection: React.FC = () => {
  return (
    <div className="p-6 bg-gradient-to-br from-indigo-900/80 to-purple-900/90">
      <h3 className="text-lg font-semibold mb-4 text-gray-100">
        🔐 Control Room Access
      </h3>
      <Input
        type="password"
        placeholder="Enter Control Room password"
        value={password}
        onChange={handlePasswordChange}
      />
      <Button
        onClick={handleAuthenticate}
        disabled={isLoading || !password.trim()}
      >
        Enter Control Room
      </Button>
    </div>
  );
};
```

#### **STR Connection Management**
```typescript
const ConnectionManagement: React.FC = () => {
  return (
    <div className="p-6 bg-gradient-to-br from-gray-800/80 to-gray-900/90">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-100">STR Connections</h3>
        <Button onClick={toggleAddConnection}>
          Add STR Connection
        </Button>
      </div>
      
      {/* Connection List */}
      {connections.map((connection) => (
        <div key={connection.name} className="connection-item">
          <StatusDot color={getStatusColor(connection.status)} />
          <div className="connection-info">
            <h4>{connection.name}</h4>
            <p>{connection.host}:{connection.port}</p>
            <p>{getStatusText(connection.status)}</p>
          </div>
          <div className="connection-actions">
            <Button onClick={() => handleConnect(connection.name)}>
              {connection.status === 'Connected' ? 'Disconnect' : 'Connect'}
            </Button>
            <Button onClick={() => handleRemove(connection.name)}>
              Remove
            </Button>
          </div>
        </div>
      ))}
    </div>
  );
};
```

#### **Bulk Operations Interface**
```typescript
const BulkOperations: React.FC = () => {
  return (
    <div className="p-6 bg-gradient-to-br from-orange-900/80 to-red-900/90">
      <h3 className="text-lg font-semibold mb-4 text-gray-100">
        Bulk Operations
      </h3>
      <div className="grid grid-cols-2 gap-4">
        <Button 
          onClick={handleMuteAllAudio}
          disabled={connectedConnections.length === 0}
        >
          🔇 Mute All STR Audio
        </Button>
        <Button 
          onClick={handleUnmuteAllAudio}
          disabled={connectedConnections.length === 0}
        >
          🔊 Unmute All STR Audio
        </Button>
        <Button 
          onClick={handleChangeAllScenes}
          disabled={connectedConnections.length === 0}
        >
          🎬 Change All Scenes
        </Button>
        <Button 
          onClick={handleStartAllStreaming}
          disabled={connectedConnections.length === 0}
        >
          📺 Start All Streaming
        </Button>
        <Button 
          onClick={handleStopAllStreaming}
          disabled={connectedConnections.length === 0}
        >
          ⏹️ Stop All Streaming
        </Button>
      </div>
    </div>
  );
};
```

### **Control Room Frontend Integration**

#### **Tauri Commands**
```typescript
// Authentication
await invoke('control_room_authenticate_async', { password });

// Connection Management
await invoke('control_room_add_str_connection', {
  sessionId,
  name,
  host,
  port,
  password,
  notes
});

await invoke('control_room_connect_str', { sessionId, strName });
await invoke('control_room_disconnect_str', { sessionId, strName });
await invoke('control_room_remove_str_connection', { sessionId, strName });

// Get Connections
await invoke('control_room_get_str_connections', { sessionId });
```

#### **Real-time Status Updates**
```typescript
const updateConnectionStatus = (name: string, status: StrConnection['status']) => {
  setState(prev => ({
    ...prev,
    connections: prev.connections.map(conn =>
      conn.name === name ? { ...conn, status } : conn
    )
  }));
};

// Connection status flow: Disconnected → Connecting → Connected/Error
```

### **Control Room Frontend Features**

#### **🔐 Security Features**
- **Master Password Authentication**: Basic password gate for Control Room access (development implementation)
- **Session Management**: Session-based authentication with logout functionality
- **Encrypted Storage**: All configurations stored securely in database
- **Access Control**: Password-gated access to STR connection management
- **Role Separation**: Control Room administrators vs regular OBS users
- **Security Gates**: Password barrier for STR management operations
- **✅ Security Status**: Production-grade bcrypt authentication with 12-hour tournament sessions implemented

#### **🔗 Connection Management**
- **Separate Management**: Independent from regular OBS WebSocket connections
- **User-defined Names**: Custom names for STR connections
- **Real-time Status**: Live connection status monitoring with color indicators
- **Configuration Persistence**: Host, port, credentials, and notes stored securely

#### **⚡ Real-time Operations**
- **Connect/Disconnect**: Individual STR connection management
- **Status Monitoring**: Live status updates (Disconnected, Connecting, Connected, Error)
- **Error Handling**: Comprehensive error messages and user feedback
- **Loading States**: Visual feedback during operations

#### **🎛️ Bulk Operations Framework**
- **Multi-STR Control**: Operate on all connected STR instances simultaneously
- **Audio Management**: Bulk mute/unmute functionality
- **Scene Control**: Change scenes across all STR instances
- **Streaming Control**: Start/stop streaming on all connections
- **Result Aggregation**: Comprehensive feedback for bulk operations

### **Frontend Technical Implementation**

#### **State Management**
- **Local Component State**: React useState for UI state management
- **Real-time Updates**: Immediate UI feedback for user actions
- **Error Boundaries**: Comprehensive error handling and user feedback
- **Loading States**: Visual indicators for all async operations
- **TypeScript Compliance**: Proper event handler typing for React.ChangeEvent and React.KeyboardEvent
- **Import Resolution**: Correct default imports for Button and Input atomic components

#### **UI/UX Design**
- **Consistent Styling**: Follows existing design patterns and color schemes
- **Responsive Layout**: Grid-based layout with proper spacing
- **Visual Feedback**: Color-coded status indicators and success/error messages
- **Accessibility**: Proper ARIA labels and keyboard navigation support

#### **Integration Points**
- **OBS Drawer**: Seamlessly integrated as third tab in OBS drawer
- **Existing APIs**: Reuses established OBS WebSocket infrastructure
- **Security Layer**: Integrates with existing security and database systems
- **Error Handling**: Consistent with application-wide error handling patterns

#### **Compilation Status**
- **Frontend**: ✅ Zero TypeScript errors, all import issues resolved
- **Backend**: ✅ All Rust code compiles successfully with no warnings
- **Integration**: ✅ All 6 Tauri commands functional and tested
- **Linting**: ✅ No ESLint warnings or errors
- **Functional Ready**: ✅ Full compilation success, ready for testing

#### **✅ Production Security Implementation**
- **Current State**: Production-grade bcrypt authentication with enterprise security
- **Access Method**: Set master password on first use, then authenticate with your chosen password
- **✅ Completed Security Features**:
  - **bcrypt Password Hashing**: DEFAULT_COST (12 rounds) enterprise-grade security
  - **Tournament Sessions**: 12-hour session timeouts optimized for competition days
  - **First-time Setup**: Seamless master password configuration on initial authentication
  - **Password Management**: Secure password change API with verification
  - **Session Architecture**: Comprehensive session tracking with refresh and logout
  - **Audit Logging**: Full authentication attempt tracking and security event logging

## OBS Integration System

### Overview
The OBS integration system provides WebSocket-based control of OBS Studio instances, enabling recording, streaming, scene management, and real-time status monitoring.

### Frontend Components

#### **WebSocketManager (Molecules)**
- **File**: `ui/src/components/molecules/WebSocketManager.tsx`
- **Purpose**: General OBS WebSocket connection management
- **Features**: Full CRUD operations (Create, Read, Update, Delete), reconnection settings, mode-based filtering
- **Store**: Uses `useAppStore` (main application store)
- **API**: Uses `obsObwsCommands` for backend communication
- **Props**: `mode?: 'local' | 'remote'` for connection filtering
- **Status**: ✅ Complete implementation with edit functionality and mode support

#### **ObsWebSocketManager (Organisms)**
- **File**: `ui/src/components/organisms/ObsWebSocketManager.tsx`
- **Purpose**: Specialized OBS management for Control Room scenarios
- **Features**: Mode-based filtering (local/remote), event handling, status monitoring, full CRUD operations
- **Store**: Uses `useObsStore` (dedicated OBS store)
- **API**: Uses `obsObwsCommands` for backend communication
- **Status**: ✅ Complete implementation with edit functionality and mode support

### Frontend Architecture Issues

#### **OBS Drawer Tab Configuration**
- **WebSocket Tab**: ✅ Now uses `WebSocketManager` (has full edit functionality)
- **Control Room Tab**: Uses `ObsWebSocketManager` (missing edit functionality)
- **Integration Tab**: Settings panel for OBS integration preferences

#### **Edit Functionality Status**
- **ObsWebSocketManager**: ✅ Now has complete CRUD functionality including Edit
- **WebSocketManager**: ✅ Has complete CRUD functionality including Edit
- **Status**: Both managers now support full connection management

#### **Store Inconsistencies**
- **WebSocketManager**: Uses `useAppStore` with `obsConnections` array
- **ObsWebSocketManager**: Uses `useObsStore` with `connections` array
- **Type Differences**: Different ObsConnection interfaces between stores

### Frontend Implementation Plan

#### **Option 1: Separate Managers with Shared API (RECOMMENDED)**
- **WebSocket Tab**: Use `WebSocketManager` with local mode filtering
- **Control Room Tab**: Use `ObsWebSocketManager` with enhanced edit functionality
- **Benefits**: Immediate edit functionality, code reuse, separation of concerns

#### **Frontend Implementation Steps**
1. **✅ Add Mode Support**: Add mode prop to WebSocketManager for local filtering
2. **✅ Update AdvancedPanel.tsx**: Switch WebSocket tab to use WebSocketManager
3. **✅ Enhance ObsWebSocketManager**: Add edit functionality from WebSocketManager
4. **✅ Fix Frontend Issues**: Resolve command registration and type mismatches
5. **✅ Fix Tauri IPC Arguments**: Correct argument passing for `obs_obws_add_connection` command
6. **✅ Fix Parameter Naming**: Fixed Tauri IPC parameter naming conventions for all OBS WebSocket commands
7. **✅ Fix Missing Enabled Field**: Fixed missing `enabled` field in `updateConnection` function to resolve "missing field enabled" error
8. **✅ Add Test Recording Button**: Added test button to WebSocketManager for testing OBS start recording command

#### **OBS Recording Integration Plan** ✅ **COMPLETED**
9. **✅ Phase 1**: Database Schema & Models - Create recording configuration tables
10. **✅ Phase 2**: Backend OBS Commands - Add replay buffer and path configuration commands
11. **✅ Phase 3**: Frontend Integration Tab - Enhanced Integration tab with recording configuration and connection selection
12. **✅ Phase 4**: Path Generation Logic - Implement Windows Videos folder detection and tournament path logic
13. **✅ Phase 5**: PSS Event Integration - Integrate with UDP/PSS event system for automatic recording

#### **Current Status** ✅ **OBS RECORDING INTEGRATION FULLY COMPLETED**
- **Frontend**: All TypeScript errors resolved, build successful
- **Backend**: All Rust compilation errors fixed, warnings only (non-critical)
- **Integration**: OBS WebSocket management fully functional with database persistence
- **UI Components**: Both WebSocket tab (local) and Control Room tab (remote) have full CRUD functionality
- **API**: All `obws` commands properly registered and functional
- **Database**: Connection persistence working correctly
- **Tauri IPC**: Fixed argument passing for all OBS WebSocket commands
- **Database Schema**: OBS recording configuration and session tables created and ready
- **Backend Commands**: Replay buffer and path configuration commands implemented and registered
- **Frontend Integration**: Enhanced Integration tab with recording configuration and connection selection
- **Path Generation**: Windows Videos folder detection and tournament path logic fully implemented
- **Database Integration**: Dynamic data retrieval from tournament, match, and player tables implemented
- **Frontend Testing**: Both sample data and database-driven path generation testing available
- **PSS Event Integration**: Automatic recording control based on UDP/PSS events fully implemented
- **Recording Event Handler**: Complete event handling system for FightLoaded, FightReady, Clock, Winner events
- **Automatic Recording**: Configuration UI and manual controls for recording management
- **Session Management**: Real-time recording session tracking and state management
- **Testing**: Development environment ready for full integration testing

#### **Frontend Integration**
- **Shared API**: Both managers use `obsObwsCommands` for backend communication
- **Database Integration**: Connections saved to SQLite database via obws plugin
- **Command Registration**: Fix missing obws commands in main.rs
- **Type Consistency**: Align frontend and backend type definitions

#### **Frontend Integration Status**
- **✅ OBS WebSocket Management**: Full connection management with database persistence
- **✅ OBS Recording Integration**: Phase 1 completed - Database Schema & Models ready
- **✅ OBS Recording Integration**: Phase 2 completed - Backend OBS Commands ready
- **✅ OBS Recording Integration**: Phase 3 completed - Frontend Integration Tab ready
- **✅ OBS Recording Integration**: Phase 4 completed - Path Generation Logic with real folder creation
- **✅ OBS Recording Integration**: Phase 5 completed - PSS Event Integration with automatic recording
- **🔄 Control Room Phase 1**: Real OBS Integration - Replace mock data with real OBS audio source and scene enumeration
- **🔄 Control Room Phase 2**: Bulk Operations Implementation - Implement actual bulk mute/unmute and scene switching

---

## Future Enhancements

### Planned Features
1. **Advanced Animations**: Enhanced animation system
2. **Theme System**: Multiple theme support
3. **Internationalization**: Multi-language support
4. **Advanced Analytics**: Real-time analytics dashboard
5. **Mobile Responsiveness**: Enhanced mobile support

### Technical Improvements
1. **Performance Optimization**: Advanced performance tuning
2. **Accessibility**: Enhanced accessibility features
3. **Testing**: Comprehensive testing coverage
4. **Documentation**: Enhanced component documentation
5. **Developer Experience**: Improved development tools

## Support and Resources

### Documentation
- **React Documentation**: https://react.dev/
- **TypeScript Documentation**: https://www.typescriptlang.org/docs/
- **Tailwind CSS Documentation**: https://tailwindcss.com/docs/

### Community
- **React Community**: https://reactjs.org/community/
- **TypeScript Community**: https://www.typescriptlang.org/community/
- **GitHub Issues**: Project-specific issues and discussions

### Professional Support
- **Custom Development**: Tailored UI/UX solutions
- **Training and Consulting**: Frontend development training
- **Enterprise Support**: Enterprise-level support and maintenance

## Current Status

- **✅ OBS Integration**: Fully implemented with `obws` crate integration
- **✅ Database Integration**: All OBS connections saved to SQLite database
- **✅ Frontend Components**: Complete CRUD operations for OBS WebSocket connections
- **✅ Tauri IPC**: Fixed argument passing for all OBS WebSocket commands
- **✅ Connection Management**: Proper update functionality implemented (no more "Connection already exists" errors)
- **✅ Control Room**: Separate component for remote OBS instance management
- **✅ Status Monitoring**: Real-time connection status updates
- **✅ Error Handling**: Comprehensive error handling and user feedback
- **✅ OBS Recording Integration**: Complete automatic OBS recording system with PSS event integration, path generation, and real folder creation