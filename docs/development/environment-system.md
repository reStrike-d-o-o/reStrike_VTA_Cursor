# Environment System Documentation

## Overview

The reStrike VTA project implements a comprehensive environment system that allows seamless switching between **Web** and **Windows** environments. This system provides environment-aware components, API calls, and feature availability.

## Architecture

### Core Components

1. **Environment Configuration** (`ui/src/config/environment.ts`)
   - Global environment detection and configuration
   - Singleton pattern for consistent state
   - Environment-specific settings and feature flags

2. **Environment Hooks** (`ui/src/hooks/useEnvironment.ts`)
   - React hooks for environment-aware components
   - Environment-specific API calls
   - Conditional rendering utilities

3. **Environment Wrappers** (`ui/src/components/EnvironmentWrapper.tsx`)
   - Component wrappers for environment-specific rendering
   - Feature availability checking
   - Conditional component rendering

## Environment Detection

### Automatic Detection

The system automatically detects the environment based on:

1. **Tauri Availability**: Checks for `window.__TAURI__` presence
2. **Environment Variables**: `REACT_APP_ENVIRONMENT` setting
3. **Build Configuration**: Different build scripts for each environment

### Manual Override

You can manually set the environment:

```typescript
import { env } from '../config/environment';

// Override environment (for testing)
env.setEnvironment('web'); // or 'windows'
```

## Environment-Specific Features

### Windows Environment Features

- ✅ **Tauri Commands**: Native Windows API access
- ✅ **Native File System**: Direct file system access
- ✅ **System Tray**: Windows system tray integration
- ✅ **Auto Updates**: Automatic application updates
- ✅ **OBS Integration**: Direct OBS WebSocket via Tauri
- ✅ **Hardware Access**: Direct hardware control

### Web Environment Features

- ✅ **Direct WebSocket**: Browser-based WebSocket connections
- ✅ **HTTP API**: RESTful API communication
- ✅ **Browser APIs**: File upload/download via browser
- ✅ **Hot Reload**: Development hot reload support
- ✅ **Cross-Platform**: Works on any platform with a browser

## Usage Examples

### Basic Environment Detection

```typescript
import { useEnvironment } from '../hooks/useEnvironment';

function MyComponent() {
  const { environment, isWindows, isWeb, config } = useEnvironment();
  
  return (
    <div>
      <p>Current Environment: {environment}</p>
      {isWindows && <p>Windows-specific content</p>}
      {isWeb && <p>Web-specific content</p>}
    </div>
  );
}
```

### Environment-Aware API Calls

```typescript
import { useEnvironmentApi } from '../hooks/useEnvironment';

function MyComponent() {
  const { apiCall } = useEnvironmentApi();
  
  const handleApiCall = async () => {
    try {
      const result = await apiCall('obs/status');
      console.log(result);
    } catch (error) {
      console.error('API call failed:', error);
    }
  };
  
  return <button onClick={handleApiCall}>Get Status</button>;
}
```

### Conditional Component Rendering

```typescript
import { EnvironmentWrapper, WindowsOnly, WebOnly } from '../components/EnvironmentWrapper';

function MyComponent() {
  return (
    <div>
      {/* Render in all environments */}
      <EnvironmentWrapper>
        <p>This renders everywhere</p>
      </EnvironmentWrapper>
      
      {/* Windows-only component */}
      <WindowsOnly>
        <p>This only renders in Windows</p>
      </WindowsOnly>
      
      {/* Web-only component */}
      <WebOnly>
        <p>This only renders in web</p>
      </WebOnly>
      
      {/* With fallback */}
      <WindowsOnly fallback={<p>Web fallback content</p>}>
        <p>Windows-specific content</p>
      </WindowsOnly>
    </div>
  );
}
```

### Feature-Based Rendering

```typescript
import { FeatureWrapper } from '../components/EnvironmentWrapper';

function MyComponent() {
  return (
    <div>
      <FeatureWrapper feature="tauriCommands">
        <button>Use Tauri Command</button>
      </FeatureWrapper>
      
      <FeatureWrapper feature="webSocketDirect">
        <button>Use WebSocket</button>
      </FeatureWrapper>
    </div>
  );
}
```

## Build Scripts

### Package.json Scripts

```json
{
  "scripts": {
    "start": "react-scripts start",
    "start:web": "REACT_APP_ENVIRONMENT=web react-scripts start",
    "start:windows": "REACT_APP_ENVIRONMENT=windows react-scripts start",
    "build": "react-scripts build",
    "build:web": "REACT_APP_ENVIRONMENT=web react-scripts build",
    "build:windows": "REACT_APP_ENVIRONMENT=windows react-scripts build"
  }
}
```

### Usage

```bash
# Start in web mode
npm run start:web

# Start in Windows mode
npm run start:windows

# Build for web
npm run build:web

# Build for Windows
npm run build:windows
```

## Configuration

### Environment-Specific Settings

```typescript
// Web Environment
{
  api: {
    baseUrl: 'http://localhost:1420',
    timeout: 10000,
  },
  obs: {
    useTauriCommands: false,
    useWebSocketDirect: true,
  },
  features: {
    tauriCommands: false,
    webSocketDirect: true,
    nativeFileSystem: false,
  }
}

// Windows Environment
{
  api: {
    baseUrl: 'tauri://localhost',
    timeout: 30000,
  },
  obs: {
    useTauriCommands: true,
    useWebSocketDirect: false,
  },
  features: {
    tauriCommands: true,
    webSocketDirect: false,
    nativeFileSystem: true,
  }
}
```

## OBS WebSocket Integration

### Environment-Aware OBS Connection

```typescript
import { useEnvironmentObs } from '../hooks/useEnvironment';

function ObsComponent() {
  const { obsOperation } = useEnvironmentObs();
  
  const connectToObs = async () => {
    try {
      if (isWindows()) {
        // Uses Tauri commands
        await obsOperation('connect', { host: 'localhost', port: 4455 });
      } else {
        // Uses direct WebSocket
        await obsOperation('connect', { host: 'localhost', port: 4455 });
      }
    } catch (error) {
      console.error('OBS connection failed:', error);
    }
  };
  
  return <button onClick={connectToObs}>Connect to OBS</button>;
}
```

## File System Operations

### Environment-Aware File Operations

```typescript
import { useEnvironmentFileSystem } from '../hooks/useEnvironment';

function FileComponent() {
  const { fileOperation } = useEnvironmentFileSystem();
  
  const handleFileRead = async () => {
    try {
      const content = await fileOperation('read');
      console.log('File content:', content);
    } catch (error) {
      console.error('File read failed:', error);
    }
  };
  
  const handleFileSave = async () => {
    try {
      await fileOperation('save', {
        data: 'Hello World',
        filename: 'test.txt'
      });
    } catch (error) {
      console.error('File save failed:', error);
    }
  };
  
  return (
    <div>
      <button onClick={handleFileRead}>Read File</button>
      <button onClick={handleFileSave}>Save File</button>
    </div>
  );
}
```

## Development Workflow

### 1. Environment Setup

```bash
# Clone repository
git clone <repository-url>
cd reStrike_VTA

# Install dependencies
npm install
cd ui && npm install

# Start in web mode for development
npm run start:web

# Start in Windows mode for testing
npm run start:windows
```

### 2. Environment Testing

```typescript
// Test environment detection
import { env } from '../config/environment';

console.log('Environment Info:', env.getInfo());
console.log('Current Environment:', env.environment);
console.log('Is Windows:', env.isWindows);
console.log('Is Web:', env.isWeb);
```

### 3. Feature Testing

```typescript
// Test feature availability
import { config } from '../config/environment';

console.log('Available Features:', config.features);
console.log('Can use Tauri:', config.features.tauriCommands);
console.log('Can use WebSocket:', config.features.webSocketDirect);
```

## Best Practices

### 1. Always Use Environment Hooks

```typescript
// ✅ Good
const { isWindows, isWeb } = useEnvironment();

// ❌ Bad
const isWindows = typeof window !== 'undefined' && window.__TAURI__;
```

### 2. Use Environment Wrappers

```typescript
// ✅ Good
<WindowsOnly>
  <NativeFeature />
</WindowsOnly>

// ❌ Bad
{isWindows && <NativeFeature />}
```

### 3. Handle Environment-Specific Errors

```typescript
try {
  await apiCall('obs/connect');
} catch (error) {
  if (isWindows()) {
    // Handle Windows-specific error
    console.error('Tauri error:', error);
  } else {
    // Handle web-specific error
    console.error('WebSocket error:', error);
  }
}
```

### 4. Test Both Environments

```bash
# Test web environment
npm run start:web
# Test functionality in browser

# Test Windows environment
npm run start:windows
# Test functionality in Tauri app
```

## Troubleshooting

### Common Issues

1. **Environment Not Detected Correctly**
   - Check `REACT_APP_ENVIRONMENT` environment variable
   - Verify Tauri availability in Windows mode
   - Check browser console for detection logs

2. **Feature Not Available**
   - Verify feature is enabled in current environment
   - Check `config.features` object
   - Use `FeatureWrapper` for conditional rendering

3. **API Calls Failing**
   - Check environment-specific API configuration
   - Verify correct base URL for environment
   - Check network connectivity and CORS settings

### Debug Information

```typescript
import { env } from '../config/environment';

// Get detailed environment info
console.log('Environment Debug Info:', env.getInfo());

// Check configuration
console.log('Current Config:', env.config);

// Test feature availability
console.log('Features:', env.config.features);
```

## Migration Guide

### From Single Environment to Multi-Environment

1. **Update Imports**
   ```typescript
   // Old
   import { someFunction } from './utils';
   
   // New
   import { useEnvironment } from './hooks/useEnvironment';
   ```

2. **Wrap Components**
   ```typescript
   // Old
   <MyComponent />
   
   // New
   <EnvironmentWrapper>
     <MyComponent />
   </EnvironmentWrapper>
   ```

3. **Update API Calls**
   ```typescript
   // Old
   fetch('/api/endpoint')
   
   // New
   const { apiCall } = useEnvironmentApi();
   await apiCall('endpoint');
   ```

## Future Enhancements

### Planned Features

1. **Environment-Specific Builds**
   - Separate build outputs for web and Windows
   - Optimized bundles for each environment

2. **Environment-Specific Testing**
   - Automated tests for both environments
   - Environment-specific test suites

3. **Dynamic Environment Switching**
   - Runtime environment switching
   - Hot reload for environment changes

4. **Environment-Specific Styling**
   - CSS-in-JS with environment variables
   - Environment-specific themes

### Configuration Management

1. **Environment-Specific Config Files**
   - Separate config files for each environment
   - Dynamic config loading

2. **Feature Flags**
   - Runtime feature flag management
   - A/B testing capabilities

3. **Performance Monitoring**
   - Environment-specific performance metrics
   - Automated performance testing 