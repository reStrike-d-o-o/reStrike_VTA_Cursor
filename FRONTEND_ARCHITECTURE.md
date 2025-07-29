# Frontend Architecture & UI System

## Overview

The reStrike VTA frontend is built with React 18, TypeScript, and Tailwind CSS, featuring an atomic design system with real-time event handling, comprehensive state management, and a sophisticated UI component hierarchy. The frontend provides a modern, responsive interface for sports broadcasting and event management.

## 🏗️ Frontend Architecture

### **Technology Stack**
- **Framework**: React 18 with TypeScript
- **Styling**: Tailwind CSS with custom design system
- **State Management**: Zustand for global state
- **Build Tool**: Vite with Tauri integration
- **Desktop Integration**: Tauri v2 with native capabilities
- **Real-time Events**: Tauri event system for live updates

### **Core Principles**
- **Atomic Design**: Organized component hierarchy (atoms, molecules, organisms, layouts)
- **Type Safety**: Full TypeScript integration
- **Real-time Updates**: Event-driven architecture
- **Responsive Design**: Adaptive layouts for different screen sizes
- **Performance**: Optimized rendering and state management

---

## 📁 Directory Structure

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
│   │   │   ├── CpuMonitoringSection.tsx # CPU monitoring
│   │   │   ├── LogDownloadList.tsx # Log download interface
│   │   │   ├── FlagManagementPanel.tsx # Flag management interface
│   │   │   ├── PasswordDialog.tsx # Authentication dialog
│   │   │   ├── ManualModeDialog.tsx # Manual mode dialog
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

---

## 🎨 Component Architecture

### **Atomic Design System**

The frontend follows atomic design principles with a clear hierarchy:

#### **Atoms (Basic UI Elements)**
```tsx
// Button component with variants
interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  onClick?: () => void;
  children: React.ReactNode;
}

// Input component with validation
interface InputProps {
  type?: 'text' | 'password' | 'number';
  placeholder?: string;
  value: string;
  onChange: (value: string) => void;
  error?: string;
  disabled?: boolean;
}

// Status indicator
interface StatusDotProps {
  status: 'online' | 'offline' | 'warning' | 'error';
  size?: 'sm' | 'md' | 'lg';
  label?: string;
}

// Tab system components
interface TabProps {
  icon: React.ReactNode;
  label: string;
  isActive?: boolean;
  onClick?: () => void;
}

interface TabGroupProps {
  children: React.ReactNode;
  className?: string;
}
```

#### **Molecules (Compound Components)**
```tsx
// Event table section
interface EventTableSectionProps {
  events: PssEvent[];
  onEventSelect: (event: PssEvent) => void;
  filters: EventFilters;
}

// Live data panel
interface LiveDataPanelProps {
  dataType: LiveDataType;
  isEnabled: boolean;
  onToggle: (enabled: boolean) => void;
  data: LiveDataItem[];
}

// Flag management panel
interface FlagManagementPanelProps {
  flags: FlagMapping[];
  onFlagSelect: (flag: FlagMapping) => void;
  onUpload: (file: File) => void;
}

// Authentication dialogs
interface PasswordDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onAuthenticate: (password: string) => boolean;
  title?: string;
  message?: string;
}

interface ManualModeDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onToggle: (code: string) => boolean;
  isEnabled: boolean;
}
```

#### **Organisms (Complex Components)**
```tsx
// Event table organism
interface EventTableProps {
  events: PssEvent[];
  filters: EventFilters;
  onFilterChange: (filters: EventFilters) => void;
  onEventSelect: (event: PssEvent) => void;
}

// OBS WebSocket manager
interface ObsWebSocketManagerProps {
  connections: ObsConnection[];
  onConnect: (config: ObsConnectionConfig) => void;
  onDisconnect: (connectionId: string) => void;
  onStartRecording: (connectionId: string) => void;
  onStopRecording: (connectionId: string) => void;
}

// Sidebar components
interface SidebarSmallProps {
  isAdvancedMode: boolean;
  onToggleAdvanced: () => void;
  onToggleManual: () => void;
  playerInfo: PlayerInfo;
}

interface SidebarBigProps {
  matchInfo: MatchInfo;
  athleteInfo: AthleteInfo[];
  controls: ControlPanel;
}
```

#### **Layouts (Page and Section Layouts)**
```tsx
// Main dock bar layout
interface DockBarProps {
  children: React.ReactNode;
  isAdvancedMode: boolean;
  onToggleAdvanced: () => void;
}

// Advanced panel layout
interface AdvancedPanelProps {
  sidebar: React.ReactNode;
  mainContent: React.ReactNode;
  statusBar: React.ReactNode;
}

// Status bar layout
interface StatusbarAdvancedProps {
  status: SystemStatus;
  notifications: Notification[];
  onNotificationDismiss: (id: string) => void;
}
```

---

## 🔄 State Management

### **Zustand Stores**

#### **Live Data Store**
```typescript
interface LiveDataStore {
  // State
  enabledSubsystems: Set<string>;
  liveData: Record<string, LiveDataItem[]>;
  streamingStatus: Record<string, boolean>;
  
  // Actions
  toggleSubsystem: (subsystem: string) => void;
  updateLiveData: (subsystem: string, data: LiveDataItem[]) => void;
  setStreamingStatus: (subsystem: string, status: boolean) => void;
  clearLiveData: (subsystem: string) => void;
}

const useLiveDataStore = create<LiveDataStore>((set, get) => ({
  enabledSubsystems: new Set(),
  liveData: {},
  streamingStatus: {},
  
  toggleSubsystem: (subsystem) => {
    const { enabledSubsystems } = get();
    const newSet = new Set(enabledSubsystems);
    if (newSet.has(subsystem)) {
      newSet.delete(subsystem);
    } else {
      newSet.add(subsystem);
    }
    set({ enabledSubsystems: newSet });
  },
  
  updateLiveData: (subsystem, data) => {
    set((state) => ({
      liveData: { ...state.liveData, [subsystem]: data }
    }));
  },
  
  setStreamingStatus: (subsystem, status) => {
    set((state) => ({
      streamingStatus: { ...state.streamingStatus, [subsystem]: status }
    }));
  },
  
  clearLiveData: (subsystem) => {
    set((state) => {
      const newLiveData = { ...state.liveData };
      delete newLiveData[subsystem];
      return { liveData: newLiveData };
    });
  }
}));
```

#### **OBS Store**
```typescript
interface ObsStore {
  // State
  connections: ObsConnection[];
  activeConnection: string | null;
  recordingStatus: RecordingStatus;
  streamingStatus: StreamingStatus;
  
  // Actions
  addConnection: (connection: ObsConnection) => void;
  removeConnection: (connectionId: string) => void;
  setActiveConnection: (connectionId: string | null) => void;
  updateRecordingStatus: (status: RecordingStatus) => void;
  updateStreamingStatus: (status: StreamingStatus) => void;
}
```

#### **PSS Match Store**
```typescript
interface PssMatchStore {
  // State
  currentMatch: PssMatch | null;
  events: PssEvent[];
  athletes: PssAthlete[];
  scores: PssScore[];
  
  // Actions
  setCurrentMatch: (match: PssMatch | null) => void;
  addEvent: (event: PssEvent) => void;
  updateAthletes: (athletes: PssAthlete[]) => void;
  updateScores: (scores: PssScore[]) => void;
  clearEvents: () => void;
}
```

---

## 🎣 Custom Hooks

### **Real-time Event Hooks**

#### **PSS Events Hook**
```typescript
export const usePssEvents = () => {
  const [events, setEvents] = useState<PssEvent[]>([]);
  const [isListening, setIsListening] = useState(false);
  
  const setupEventListener = async () => {
    try {
      await pssCommands.setupEventListener();
      const unlisten = await window.__TAURI__.event.listen('pss_event', (event: any) => {
        handlePssEvent(event.payload);
      });
      setIsListening(true);
      return unlisten;
    } catch (error) {
      console.error('Failed to setup PSS event listener:', error);
    }
  };
  
  const handlePssEvent = (eventData: any) => {
    setEvents(prev => [...prev, eventData]);
  };
  
  const clearEvents = () => {
    setEvents([]);
  };
  
  return {
    events,
    isListening,
    setupEventListener,
    clearEvents
  };
};
```

#### **Live Data Events Hook**
```typescript
export const useLiveDataEvents = (enabled: boolean, selectedType: LiveDataType) => {
  const [data, setData] = useState<LiveDataItem[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const listenerRef = useRef<Promise<() => void> | null>(null);
  
  useEffect(() => {
    if (enabled && !isStreaming) {
      startLiveData();
    } else if (!enabled && isStreaming) {
      stopLiveData();
    }
  }, [enabled, selectedType]);
  
  const startLiveData = async () => {
    try {
      await liveDataCommands.startLiveData(selectedType);
      const unlisten = await window.__TAURI__.event.listen('live_data', (event: any) => {
        handleLiveDataEvent(event.payload);
      });
      listenerRef.current = Promise.resolve(unlisten);
      setIsStreaming(true);
    } catch (error) {
      console.error('Failed to start live data:', error);
    }
  };
  
  const stopLiveData = async () => {
    try {
      await liveDataCommands.stopLiveData(selectedType);
      if (listenerRef.current) {
        const unlisten = await listenerRef.current;
        unlisten();
        listenerRef.current = null;
      }
      setIsStreaming(false);
    } catch (error) {
      console.error('Failed to stop live data:', error);
    }
  };
  
  const handleLiveDataEvent = (eventData: any) => {
    setData(prev => [...prev, eventData]);
  };
  
  return {
    data,
    isStreaming,
    startLiveData,
    stopLiveData
  };
};
```

### **Environment Detection Hooks**

#### **Environment Hook**
```typescript
export const useEnvironment = () => {
  const [isTauri, setIsTauri] = useState(false);
  const [isWeb, setIsWeb] = useState(false);
  
  useEffect(() => {
    const checkEnvironment = () => {
      const tauriAvailable = typeof window !== 'undefined' && window.__TAURI__;
      setIsTauri(tauriAvailable);
      setIsWeb(!tauriAvailable);
    };
    
    checkEnvironment();
  }, []);
  
  return { isTauri, isWeb };
};
```

---

## 🎨 UI Design System

### **Color Scheme**
```css
/* Main backgrounds */
bg-gray-900          /* Main app background */
bg-black/60          /* DockBar background */
bg-gray-800/80       /* Advanced panel background */
bg-gray-700/90       /* Sidebar background */

/* Modern gradient style */
bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg

/* Text colors */
text-blue-300        /* Headings */
text-gray-300        /* Labels */
text-gray-400        /* Secondary text */

/* Status indicators */
bg-green-900 text-green-300    /* Success */
bg-yellow-900 text-yellow-300  /* Warnings */
bg-red-900 text-red-300        /* Errors */

/* Interactive elements */
bg-gray-700/30 rounded border border-gray-600/20
```

### **Typography**
```css
/* Headings */
text-xl font-semibold text-blue-300
text-lg font-medium text-blue-300
text-base font-medium text-gray-300

/* Body text */
text-sm text-gray-300
text-xs text-gray-400

/* Labels */
text-sm font-medium text-gray-300
```

### **Spacing System**
```css
/* Consistent spacing */
space-y-4            /* Vertical spacing */
space-x-4            /* Horizontal spacing */
p-4                  /* Padding */
m-2                  /* Margin */
gap-4                /* Grid/Flex gap */
```

### **Component Styling Patterns**

#### **Panel Components**
```tsx
// Modern gradient panel
<div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg p-4">
  {children}
</div>

// Interactive container
<div className="bg-gray-700/30 rounded border border-gray-600/20 p-3">
  {children}
</div>
```

#### **Table Components**
```tsx
// Scrollable table with sticky headers
<div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
  <table className="w-full">
    <thead className="sticky top-0 z-10 bg-[#101820]">
      <tr>
        <th className="text-gray-200 px-4 py-2 text-left">Header</th>
      </tr>
    </thead>
    <tbody>
      <tr className="hover:bg-blue-900 transition-colors">
        <td className="text-gray-300 px-4 py-2">Content</td>
      </tr>
    </tbody>
  </table>
</div>
```

---

## 🚀 Real-time Features

### **Event-Driven Architecture**

#### **PSS Event System**
```typescript
// Event listener setup
const setupPssEventListener = async () => {
  await window.__TAURI__.core.invoke('pss_setup_event_listener');
  
  const unlisten = await window.__TAURI__.event.listen('pss_event', (event: any) => {
    const { event_type, data, timestamp } = event.payload;
    
    // Update UI based on event type
    switch (event_type) {
      case 'match_config':
        handleMatchConfig(data);
        break;
      case 'athletes':
        handleAthletes(data);
        break;
      case 'current_scores':
        handleScores(data);
        break;
      case 'warnings':
        handleWarnings(data);
        break;
    }
  });
  
  return unlisten;
};
```

#### **OBS Status Monitoring**
```typescript
// OBS status listener
const setupObsStatusListener = async () => {
  await window.__TAURI__.core.invoke('obs_setup_status_listener');
  
  const unlisten = await window.__TAURI__.event.listen('obs_status', (event: any) => {
    const { connection_status, recording_status, streaming_status } = event.payload;
    
    // Update OBS store
    obsStore.updateConnectionStatus(connection_status);
    obsStore.updateRecordingStatus(recording_status);
    obsStore.updateStreamingStatus(streaming_status);
  });
  
  return unlisten;
};
```

#### **CPU Monitoring**
```typescript
// CPU stats listener
const setupCpuStatsListener = async () => {
  await window.__TAURI__.core.invoke('cpu_setup_stats_listener');
  
  const unlisten = await window.__TAURI__.event.listen('cpu_stats', (event: any) => {
    const { cpu_usage, memory_usage, disk_usage, obs_usage } = event.payload;
    
    // Update CPU monitoring UI
    updateCpuDisplay(cpu_usage, memory_usage, disk_usage, obs_usage);
  });
  
  return unlisten;
};
```

---

## 🔧 Utility Functions

### **Tauri Commands**
```typescript
// Window management commands
export const windowCommands = {
  setFullscreen: () => invoke('set_window_fullscreen'),
  setCompact: (width?: number, height?: number) => invoke('set_window_compact', { width, height }),
  setCustomSize: (width: number, height: number) => invoke('set_window_custom_size', { width, height }),
  setPosition: (x: number, y: number) => invoke('set_window_position', { x, y }),
  saveSettings: (settings: any) => invoke('save_window_settings', { settings }),
  loadSettings: () => invoke('load_window_settings'),
};

// OBS commands
export const obsCommands = {
  connect: (url: string) => invoke('obs_connect', { url }),
  startRecording: () => invoke('obs_start_recording'),
  stopRecording: () => invoke('obs_stop_recording'),
  getStatus: () => invoke('obs_get_status'),
};

// Database commands
export const dbCommands = {
  getSetting: (key: string) => invoke('db_get_ui_setting', { key }),
  setSetting: (key: string, value: string, changedBy: string, changeReason?: string) => 
    invoke('db_set_ui_setting', { key, value, changed_by: changedBy, change_reason: changeReason }),
  getAllSettings: () => invoke('db_get_all_ui_settings'),
};
```

### **Flag Utilities**
```typescript
// Flag management utilities
export const flagUtils = {
  getFlagUrl: (iocCode: string) => `/assets/flags/${iocCode}.png`,
  getFlagFallback: (iocCode: string) => getCountryEmoji(iocCode),
  validateIocCode: (code: string) => /^[A-Z]{3}$/.test(code),
  searchFlags: (query: string, flags: FlagMapping[]) => 
    flags.filter(flag => 
      flag.country_name.toLowerCase().includes(query.toLowerCase()) ||
      flag.ioc_code.toLowerCase().includes(query.toLowerCase())
    ),
};
```

---

## 🎯 Performance Optimization

### **React Optimization**

#### **Memoization**
```tsx
// Memoized components
const EventTable = React.memo<EventTableProps>(({ events, filters, onFilterChange, onEventSelect }) => {
  // Component implementation
});

// Memoized callbacks
const handleEventSelect = useCallback((event: PssEvent) => {
  onEventSelect(event);
}, [onEventSelect]);

// Memoized values
const filteredEvents = useMemo(() => {
  return events.filter(event => {
    // Filtering logic
  });
}, [events, filters]);
```

#### **Virtual Scrolling**
```tsx
// For large event lists
const VirtualizedEventList = ({ events }: { events: PssEvent[] }) => {
  return (
    <FixedSizeList
      height={400}
      itemCount={events.length}
      itemSize={50}
      itemData={events}
    >
      {EventRow}
    </FixedSizeList>
  );
};
```

### **Bundle Optimization**
```typescript
// Dynamic imports for code splitting
const AdvancedPanel = lazy(() => import('./components/layouts/AdvancedPanel'));
const FlagManagementPanel = lazy(() => import('./components/molecules/FlagManagementPanel'));

// Tree shaking friendly imports
import { Button } from './components/atoms/Button';
import { Input } from './components/atoms/Input';
```

---

## 🔒 Security & Authentication

### **Authentication System**
```typescript
// Password dialog for Advanced mode
const PasswordDialog: React.FC<PasswordDialogProps> = ({
  isOpen,
  onClose,
  onAuthenticate,
  title = "Enter Password",
  message = "Please enter the password to access Advanced mode"
}) => {
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  
  const handleSubmit = () => {
    if (onAuthenticate(password)) {
      onClose();
      setPassword('');
      setError('');
    } else {
      setError('Invalid password');
    }
  };
  
  return (
    <Dialog isOpen={isOpen} onClose={onClose}>
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-medium text-blue-300 mb-2">{title}</h3>
        <p className="text-gray-300 mb-4">{message}</p>
        <Input
          type="password"
          value={password}
          onChange={setPassword}
          placeholder="Enter password"
          error={error}
        />
        <div className="flex justify-end space-x-2 mt-4">
          <Button variant="secondary" onClick={onClose}>Cancel</Button>
          <Button variant="primary" onClick={handleSubmit}>Submit</Button>
        </div>
      </div>
    </Dialog>
  );
};
```

---

## 📱 Responsive Design

### **Breakpoint System**
```css
/* Tailwind breakpoints */
sm: 640px   /* Small screens */
md: 768px   /* Medium screens */
lg: 1024px  /* Large screens */
xl: 1280px  /* Extra large screens */
2xl: 1536px /* 2X large screens */
```

### **Adaptive Layouts**
```tsx
// Responsive sidebar
const Sidebar = ({ isAdvancedMode }: { isAdvancedMode: boolean }) => {
  return (
    <div className={`
      ${isAdvancedMode ? 'w-64' : 'w-[350px]'}
      flex-shrink-0
      bg-gray-700/90
      transition-all duration-300
    `}>
      {isAdvancedMode ? <SidebarBig /> : <SidebarSmall />}
    </div>
  );
};

// Responsive event table
const EventTable = ({ events }: { events: PssEvent[] }) => {
  return (
    <div className="
      w-full
      overflow-x-auto
      md:overflow-x-visible
    ">
      <table className="
        w-full
        min-w-[600px]
        md:min-w-0
      ">
        {/* Table content */}
      </table>
    </div>
  );
};
```

---

## 🔮 Future Enhancements

### **Planned Features**

#### **1. Advanced UI Components**
- Virtual scrolling for large datasets
- Advanced filtering and search
- Drag and drop functionality
- Keyboard shortcuts

#### **2. Performance Improvements**
- Service worker for offline support
- Advanced caching strategies
- Bundle optimization
- Lazy loading improvements

#### **3. Accessibility Enhancements**
- Screen reader support
- Keyboard navigation
- High contrast mode
- Focus management

#### **4. Real-time Features**
- WebSocket fallbacks
- Offline event queuing
- Real-time collaboration
- Push notifications

---

## 📞 Troubleshooting

### **Common Issues**

#### **1. Tauri Integration**
- Verify environment detection
- Check Tauri API availability
- Review command definitions
- Test event listeners

#### **2. Performance Issues**
- Monitor bundle size
- Check component re-renders
- Analyze memory usage
- Review state management

#### **3. Styling Issues**
- Verify Tailwind configuration
- Check CSS specificity
- Review responsive breakpoints
- Test dark mode compatibility

### **Debugging Tools**
- React DevTools
- Performance profiling
- Bundle analyzer
- Network monitoring

---

**Last Updated**: 2025-01-29  
**Architecture Version**: 2.0  
**Status**: Production Ready with Real-time Features