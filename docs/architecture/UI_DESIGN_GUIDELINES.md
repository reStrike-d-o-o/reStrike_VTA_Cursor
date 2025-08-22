# UI Design Guidelines & Component System

## Overview

The reStrike VTA UI system is built on a comprehensive design system with atomic components, consistent styling patterns, and modern UX principles. This document provides complete guidelines for UI development, component usage, and design patterns.

## 🎨 Design System Foundation

### **Technology Stack**
- **Framework**: React 18 with TypeScript
- **Styling**: Tailwind CSS with custom design tokens
- **Component System**: Atomic Design (Atoms, Molecules, Organisms, Layouts)
- **State Management**: Zustand for global state
- **Icons**: Custom icon system with consistent sizing
- **Typography**: Custom font stack with semantic hierarchy

### **Design Principles**
- **Consistency**: Unified design language across all components
- **Accessibility**: WCAG 2.1 AA compliance
- **Performance**: Optimized rendering and minimal re-renders
- **Responsiveness**: Adaptive layouts for different screen sizes
- **Modern Aesthetics**: Clean, professional appearance with subtle animations

---

## 🎨 Color System

### **Primary Color Palette**
```css
/* Main Background Colors */
bg-gray-900          /* Main app background */
bg-black/60          /* DockBar background */
bg-gray-800/80       /* Advanced panel background */
bg-gray-700/90       /* Sidebar background */

/* Modern Gradient Style (Primary) */
bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg

/* Text Colors */
text-blue-300        /* Primary headings */
text-gray-300        /* Body text and labels */
text-gray-400        /* Secondary text */
text-gray-500        /* Disabled text */

/* Status Colors */
bg-green-900 text-green-300    /* Success states */
bg-yellow-900 text-yellow-300  /* Warning states */
bg-red-900 text-red-300        /* Error states */
bg-blue-900 text-blue-300      /* Info states */

/* Interactive Elements */
bg-gray-700/30 rounded border border-gray-600/20  /* Interactive containers */
hover:bg-gray-600/40                              /* Hover states */
focus:ring-2 focus:ring-blue-500/50               /* Focus states */
```

### **Semantic Color Usage**
```css
/* Success/Positive Actions */
bg-green-900 text-green-300 border-green-700
hover:bg-green-800

/* Warning/Caution Actions */
bg-yellow-900 text-yellow-300 border-yellow-700
hover:bg-yellow-800

/* Error/Danger Actions */
bg-red-900 text-red-300 border-red-700
hover:bg-red-800

/* Primary Actions */
bg-blue-900 text-blue-300 border-blue-700
hover:bg-blue-800

/* Secondary Actions */
bg-gray-700 text-gray-300 border-gray-600
hover:bg-gray-600
```

---

## 📝 Typography System

### **Font Hierarchy**
```css
/* Headings */
text-4xl font-bold text-blue-300    /* H1 - Page titles */
text-3xl font-semibold text-blue-300 /* H2 - Section titles */
text-2xl font-semibold text-blue-300 /* H3 - Subsection titles */
text-xl font-semibold text-blue-300  /* H4 - Component titles */
text-lg font-medium text-blue-300    /* H5 - Small titles */
text-base font-medium text-gray-300  /* H6 - Micro titles */

/* Body Text */
text-base text-gray-300             /* Default body text */
text-sm text-gray-300               /* Small body text */
text-xs text-gray-400               /* Caption text */

/* Labels and UI Text */
text-sm font-medium text-gray-300   /* Form labels */
text-sm text-gray-400               /* Helper text */
text-xs text-gray-500               /* Disabled text */
```

### **Font Stack**
```css
/* Primary Font Stack */
font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;

/* Monospace Font Stack */
font-mono: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
```

---

## 🧩 Component System

### **Atomic Design Hierarchy**

#### **Atoms (Basic UI Elements)**
```tsx
// Button Component
interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger' | 'success';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  icon?: React.ReactNode;
  onClick?: () => void;
  children: React.ReactNode;
}

// Input Component
interface InputProps {
  type?: 'text' | 'password' | 'number' | 'email';
  placeholder?: string;
  value: string;
  onChange: (value: string) => void;
  error?: string;
  disabled?: boolean;
  required?: boolean;
  label?: string;
}

// Status Indicator
interface StatusDotProps {
  status: 'online' | 'offline' | 'warning' | 'error' | 'loading';
  size?: 'sm' | 'md' | 'lg';
  label?: string;
  animated?: boolean;
}

// Tab Components
interface TabProps {
  icon: React.ReactNode;
  label: string;
  isActive?: boolean;
  onClick?: () => void;
  disabled?: boolean;
}

interface TabGroupProps {
  children: React.ReactNode;
  className?: string;
  orientation?: 'horizontal' | 'vertical';
}
```

#### **Molecules (Compound Components)**
```tsx
// Form Field
interface FormFieldProps {
  label: string;
  error?: string;
  required?: boolean;
  children: React.ReactNode;
}

// Card Component
interface CardProps {
  title?: string;
  subtitle?: string;
  children: React.ReactNode;
  actions?: React.ReactNode;
  className?: string;
}

// Modal/Dialog
interface DialogProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  actions?: React.ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl';
}
```

#### **Organisms (Complex Components)**
```tsx
// Event Table
interface EventTableProps {
  events: PssEvent[];
  filters: EventFilters;
  onFilterChange: (filters: EventFilters) => void;
  onEventSelect: (event: PssEvent) => void;
  loading?: boolean;
}

// OBS Manager
interface ObsManagerProps {
  connections: ObsConnection[];
  onConnect: (config: ObsConnectionConfig) => void;
  onDisconnect: (connectionId: string) => void;
  onStartRecording: (connectionId: string) => void;
  onStopRecording: (connectionId: string) => void;
}

// Flag Management Panel
interface FlagManagementPanelProps {
  flags: FlagMapping[];
  onFlagSelect: (flag: FlagMapping) => void;
  onUpload: (file: File) => void;
  onSearch: (query: string) => void;
  loading?: boolean;
}
```

#### **Layouts (Page and Section Layouts)**
```tsx
// Main Layout
interface MainLayoutProps {
  sidebar: React.ReactNode;
  mainContent: React.ReactNode;
  statusBar?: React.ReactNode;
}

// Advanced Panel Layout
interface AdvancedPanelProps {
  sidebar: React.ReactNode;
  mainContent: React.ReactNode;
  statusBar: React.ReactNode;
  isAdvancedMode: boolean;
}
```

---

## 🎯 Component Styling Patterns

### **Panel Components**
```tsx
// Modern gradient panel (Primary pattern)
<div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg p-4">
  {children}
</div>

// Interactive container
<div className="bg-gray-700/30 rounded border border-gray-600/20 p-3 hover:bg-gray-600/40 transition-colors">
  {children}
</div>

// Card component
<div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg p-6">
  <div className="flex items-center justify-between mb-4">
    <h3 className="text-lg font-semibold text-blue-300">{title}</h3>
    {actions && <div className="flex space-x-2">{actions}</div>}
  </div>
  {children}
</div>
```

### **Table Components**
```tsx
// Scrollable table with sticky headers
<div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
  <table className="w-full">
    <thead className="sticky top-0 z-10 bg-[#101820]">
      <tr>
        <th className="text-gray-200 px-4 py-2 text-left font-medium">Header</th>
        {/* More headers */}
      </tr>
    </thead>
    <tbody>
      <tr className="hover:bg-blue-900 transition-colors">
        <td className="text-gray-300 px-4 py-2">Content</td>
        {/* More cells */}
      </tr>
    </tbody>
  </table>
</div>

// Table row with status indicators
<tr className="hover:bg-blue-900 transition-colors border-b border-gray-700/50">
  <td className="text-gray-300 px-4 py-2">
    <div className="flex items-center space-x-2">
      <StatusDot status="online" size="sm" />
      <span>Event Name</span>
    </div>
  </td>
  {/* More cells */}
</tr>
```

### **Form Components**
```tsx
// Form field with label and error
<div className="space-y-2">
  <label className="block text-sm font-medium text-gray-300">
    {label}
    {required && <span className="text-red-400 ml-1">*</span>}
  </label>
  <Input
    type={type}
    value={value}
    onChange={onChange}
    placeholder={placeholder}
    error={error}
    disabled={disabled}
  />
  {error && (
    <p className="text-sm text-red-400">{error}</p>
  )}
</div>

// Form group
<div className="space-y-4">
  <FormField label="Username" required>
    <Input
      type="text"
      value={username}
      onChange={setUsername}
      placeholder="Enter username"
    />
  </FormField>
  <FormField label="Password" required>
    <Input
      type="password"
      value={password}
      onChange={setPassword}
      placeholder="Enter password"
    />
  </FormField>
</div>
```

---

## 🎨 Interactive Elements

### **Button System**
```tsx
// Primary button
<Button
  variant="primary"
  size="md"
  onClick={handleClick}
  disabled={loading}
>
  {loading ? <Spinner size="sm" /> : 'Save Changes'}
</Button>

// Secondary button
<Button
  variant="secondary"
  size="md"
  onClick={handleCancel}
>
  Cancel
</Button>

// Danger button
<Button
  variant="danger"
  size="md"
  onClick={handleDelete}
>
  Delete
</Button>

// Icon button
<Button
  variant="secondary"
  size="sm"
  icon={<Icon name="settings" />}
  onClick={handleSettings}
>
  Settings
</Button>
```

### **Input System**
```tsx
// Text input
<Input
  type="text"
  value={value}
  onChange={setValue}
  placeholder="Enter text..."
  error={error}
/>

// Password input
<Input
  type="password"
  value={password}
  onChange={setPassword}
  placeholder="Enter password"
  error={passwordError}
/>

// Number input
<Input
  type="number"
  value={number}
  onChange={setNumber}
  placeholder="0"
  min={0}
  max={100}
/>
```

### **Status Indicators**
```tsx
// Status dot
<StatusDot status="online" size="md" label="Connected" />

// Status badge
<div className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-900 text-green-300">
  <StatusDot status="online" size="sm" className="mr-1" />
  Online
</div>

// Progress indicator
<div className="w-full bg-gray-700 rounded-full h-2">
  <div className="bg-blue-500 h-2 rounded-full transition-all duration-300" style={{ width: `${progress}%` }} />
</div>
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

### **Responsive Patterns**
```tsx
// Responsive sidebar
<div className={`
  ${isAdvancedMode ? 'w-64' : 'w-[350px]'}
  flex-shrink-0
  bg-gray-700/90
  transition-all duration-300
  ${isCollapsed ? 'w-16' : ''}
`}>
  {isAdvancedMode ? <SidebarBig /> : <SidebarSmall />}
</div>

// Responsive grid
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
  {items.map(item => (
    <Card key={item.id}>
      {item.content}
    </Card>
  ))}
</div>

// Responsive table
<div className="overflow-x-auto">
  <table className="w-full min-w-[600px]">
    {/* Table content */}
  </table>
</div>

// Responsive navigation
<nav className="flex flex-col md:flex-row space-y-2 md:space-y-0 md:space-x-4">
  {navItems.map(item => (
    <NavLink key={item.id} to={item.path}>
      {item.label}
    </NavLink>
  ))}
</nav>
```

---

## 🎭 Animation & Transitions

### **Transition Classes**
```css
/* Standard transitions */
transition-all duration-300 ease-in-out
transition-colors duration-200 ease-in-out
transition-transform duration-200 ease-in-out

/* Hover effects */
hover:scale-105 transition-transform duration-200
hover:bg-gray-600/40 transition-colors duration-200
hover:shadow-lg transition-shadow duration-200

/* Focus effects */
focus:ring-2 focus:ring-blue-500/50 focus:outline-none
focus:border-blue-500 transition-colors duration-200

/* Loading states */
animate-pulse
animate-spin
animate-bounce
```

### **Animation Patterns**
```tsx
// Fade in animation
<div className="animate-in fade-in duration-300">
  {content}
</div>

// Slide in animation
<div className="animate-in slide-in-from-right duration-300">
  {content}
</div>

// Scale animation
<div className="animate-in zoom-in duration-300">
  {content}
</div>

// Loading spinner
<div className="animate-spin rounded-full h-4 w-4 border-2 border-gray-300 border-t-blue-500" />
```

---

## ♿ Accessibility Guidelines

### **WCAG 2.1 AA Compliance**
```tsx
// Proper heading hierarchy
<h1>Page Title</h1>
<h2>Section Title</h2>
<h3>Subsection Title</h3>

// Form labels
<label htmlFor="username" className="block text-sm font-medium text-gray-300">
  Username
</label>
<input
  id="username"
  type="text"
  aria-describedby="username-error"
  aria-required="true"
/>

// Error messages
<div id="username-error" className="text-sm text-red-400" role="alert">
  Username is required
</div>

// Button accessibility
<button
  aria-label="Close dialog"
  aria-pressed={isOpen}
  onClick={onClose}
>
  <Icon name="close" />
</button>

// Status indicators
<div role="status" aria-live="polite">
  <StatusDot status="online" />
  <span className="sr-only">System is online</span>
</div>
```

### **Keyboard Navigation**
```tsx
// Focus management
const focusTrapRef = useRef<HTMLDivElement>(null);

useEffect(() => {
  if (isOpen && focusTrapRef.current) {
    const focusableElements = focusTrapRef.current.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (focusableElements.length > 0) {
      (focusableElements[0] as HTMLElement).focus();
    }
  }
}, [isOpen]);

// Tab navigation
<div ref={focusTrapRef} className="space-y-4">
  <Button tabIndex={0}>First Button</Button>
  <Button tabIndex={0}>Second Button</Button>
  <Button tabIndex={0}>Third Button</Button>
</div>
```

---

## 🎨 Icon System

### **Icon Guidelines**
```tsx
// Icon component
interface IconProps {
  name: string;
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  className?: string;
  color?: string;
}

// Icon sizes
const iconSizes = {
  xs: 'w-3 h-3',
  sm: 'w-4 h-4',
  md: 'w-5 h-5',
  lg: 'w-6 h-6',
  xl: 'w-8 h-8'
};

// Icon usage
<Icon name="settings" size="md" className="text-gray-300" />
<Icon name="check" size="sm" className="text-green-400" />
<Icon name="warning" size="lg" className="text-yellow-400" />
```

### **Icon Categories**
```tsx
// Navigation icons
const navigationIcons = [
  'home', 'settings', 'profile', 'dashboard', 'analytics'
];

// Action icons
const actionIcons = [
  'play', 'pause', 'stop', 'record', 'save', 'delete', 'edit'
];

// Status icons
const statusIcons = [
  'check', 'warning', 'error', 'info', 'loading'
];

// Communication icons
const communicationIcons = [
  'message', 'notification', 'email', 'phone'
];
```

---

## 🎯 Component Best Practices

### **Performance Optimization**
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

// Lazy loading
const AdvancedPanel = lazy(() => import('./components/layouts/AdvancedPanel'));
```

### **Error Boundaries**
```tsx
// Error boundary component
class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-4 bg-red-900/20 border border-red-700 rounded-lg">
          <h2 className="text-lg font-semibold text-red-300 mb-2">Something went wrong</h2>
          <p className="text-red-400 mb-4">An error occurred while rendering this component.</p>
          <Button variant="secondary" onClick={() => this.setState({ hasError: false })}>
            Try Again
          </Button>
        </div>
      );
    }

    return this.props.children;
  }
}
```

### **Loading States**
```tsx
// Loading component
const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({ size = 'md', text = 'Loading...' }) => {
  return (
    <div className="flex items-center justify-center space-x-2">
      <div className={`animate-spin rounded-full border-2 border-gray-300 border-t-blue-500 ${sizeClasses[size]}`} />
      <span className="text-gray-400">{text}</span>
    </div>
  );
};

// Skeleton loading
const SkeletonCard: React.FC = () => {
  return (
    <div className="bg-gray-800/50 rounded-lg p-4 animate-pulse">
      <div className="h-4 bg-gray-700 rounded mb-2" />
      <div className="h-3 bg-gray-700 rounded mb-1" />
      <div className="h-3 bg-gray-700 rounded w-2/3" />
    </div>
  );
};
```

---

## 🎨 Theme System

### **Dark Theme (Default)**
```css
/* Dark theme colors */
--color-bg-primary: #111827;    /* gray-900 */
--color-bg-secondary: #1f2937;  /* gray-800 */
--color-bg-tertiary: #374151;   /* gray-700 */
--color-text-primary: #d1d5db;  /* gray-300 */
--color-text-secondary: #9ca3af; /* gray-400 */
--color-accent: #3b82f6;        /* blue-500 */
--color-success: #10b981;       /* green-500 */
--color-warning: #f59e0b;       /* yellow-500 */
--color-error: #ef4444;         /* red-500 */
```

### **Theme Switching**
```tsx
// Theme context
interface ThemeContextType {
  theme: 'dark' | 'light';
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

// Theme provider
const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');

  const toggleTheme = () => {
    setTheme(prev => prev === 'dark' ? 'light' : 'dark');
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      <div className={theme}>
        {children}
      </div>
    </ThemeContext.Provider>
  );
};
```

---

## 🔧 Development Guidelines

### **Component Development**
1. **Start with Atoms**: Build basic UI elements first
2. **Compose Molecules**: Combine atoms into functional components
3. **Create Organisms**: Build complex components from molecules
4. **Design Layouts**: Arrange organisms into page layouts
5. **Test Responsively**: Ensure components work on all screen sizes
6. **Document Props**: Provide clear TypeScript interfaces
7. **Add Accessibility**: Include proper ARIA attributes and keyboard navigation

### **Styling Guidelines**
1. **Use Tailwind Classes**: Prefer utility classes over custom CSS
2. **Follow Design System**: Use established color and spacing tokens
3. **Maintain Consistency**: Use consistent patterns across components
4. **Optimize Performance**: Minimize CSS bundle size
5. **Test Interactions**: Ensure hover, focus, and active states work correctly

### **Code Quality**
1. **TypeScript**: Use strict typing for all components
2. **Props Validation**: Validate required and optional props
3. **Error Handling**: Include proper error boundaries
4. **Testing**: Write unit tests for component behavior
5. **Documentation**: Provide clear usage examples

---

## 🎯 Future Enhancements

### **Planned Features**
1. **Advanced Animations**: Micro-interactions and page transitions
2. **Custom Themes**: User-configurable color schemes
3. **Component Library**: Storybook integration for component documentation
4. **Design Tokens**: CSS custom properties for consistent theming
5. **Accessibility Tools**: Automated accessibility testing and validation

### **Performance Improvements**
1. **Code Splitting**: Lazy load components and routes
2. **Bundle Optimization**: Reduce JavaScript and CSS bundle sizes
3. **Image Optimization**: Implement responsive images and lazy loading
4. **Caching Strategies**: Implement effective caching for static assets

---

**Last Updated**: 2025-01-29  
**Design System Version**: 2.0  
**Status**: Production Ready with Comprehensive Guidelines

---

## 📺 Overlay Design Addendum (Olympic Scoreboard)

- See `ui/public/assets/scoreboard/README-scoreboard-overlays.md` for implementation details.
- Fonts are injected into embedded SVGs; do not override in HTML/CSS.
- Text centering is authored in the SVG using `text-anchor="middle"` and `dominant-baseline="central"` to avoid JS drift.
- Flags are `<image>` elements with `preserveAspectRatio="xMidYMid meet"` and clipPaths to avoid stretching; render flags beneath frame elements.
- Logo positions are authored in SVG; JS only toggles visibility based on injury time.