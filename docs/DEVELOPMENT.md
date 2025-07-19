# reStrike VTA Development Guide

## 🚀 Development Environment Setup

### Prerequisites

#### System Requirements
- **Operating System**: Windows 10/11 (x64)
- **Node.js**: v24 or higher (LTS recommended)
- **Rust**: Stable toolchain with MSVC
- **Git**: Latest version
- **OBS Studio**: v28+ with WebSocket v5 plugin

#### Development Tools
- **IDE**: Visual Studio Code with Rust and TypeScript extensions
- **Tauri CLI**: Latest version
- **Cargo**: Rust package manager
- **npm**: Node.js package manager

### Installation Steps

#### 1. Clone Repository
```bash
git clone <repository-url>
cd reStrike_VTA_Cursor
```

#### 2. Install Dependencies
```bash
# Install root dependencies
npm install

# Install frontend dependencies
cd ui
npm install
cd ..
```

#### 3. Verify Installation
```bash
# Check Rust toolchain
rustc --version
cargo --version

# Check Node.js
node --version
npm --version

# Check Tauri CLI
cargo tauri --version
```

## 🛠️ Development Workflow

### Starting Development

#### 1. Clean Environment
```bash
# Clean ports and processes
./scripts/development/cleanup-dev-environment.sh --cleanup
```

#### 2. Start Development Server
```bash
# Start Tauri development
cargo tauri dev
```

This command will:
- Start the React development server on port 3000
- Compile the Rust backend
- Launch the Tauri application
- Enable hot reload for both frontend and backend

### Development Commands

#### Frontend Development
```bash
cd ui

# Start development server only
npm start

# Build for production
npm run build

# Type checking
npm run type-check

# Linting
npm run lint
```

#### Backend Development
```bash
cd src-tauri

# Compile only
cargo build

# Run tests
cargo test

# Check for issues
cargo check

# Format code
cargo fmt

# Clippy linting
cargo clippy
```

#### Full Application
```bash
# Development build
cargo tauri dev

# Production build
cargo tauri build

# Preview production build
cargo tauri preview
```

## 📝 Coding Standards

### Rust Backend Standards

#### Code Organization
```
src-tauri/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library exports
│   ├── tauri_commands.rs    # Tauri command definitions
│   ├── core/
│   │   └── app.rs           # Core application logic
│   ├── plugins/
│   │   ├── plugin_obs.rs    # OBS integration
│   │   ├── plugin_udp.rs    # UDP listener
│   │   ├── plugin_playback.rs # Video playback
│   │   └── plugin_store.rs  # Data persistence
│   ├── config/
│   │   ├── manager.rs       # Configuration management
│   │   ├── types.rs         # Configuration types
│   │   └── mod.rs           # Module organization
│   └── types/
│       └── mod.rs           # Shared types
```

#### Naming Conventions
- **Files**: `snake_case.rs`
- **Modules**: `snake_case`
- **Structs/Enums**: `PascalCase`
- **Functions/Variables**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`

#### Error Handling
```rust
// Use AppResult<T> for all plugin methods
pub async fn some_method(&self) -> AppResult<()> {
    // Implementation
    Ok(())
}

// Use AppError for error propagation
match some_operation() {
    Ok(result) => Ok(result),
    Err(e) => Err(AppError::ConfigError(e.to_string())),
}
```

#### Async/Await Patterns
```rust
// Use tokio for async operations
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct SomeStruct {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl SomeStruct {
    pub async fn update_data(&self, key: String, value: String) -> AppResult<()> {
        let mut data = self.data.lock().await;
        data.insert(key, value);
        Ok(())
    }
}
```

### TypeScript Frontend Standards

#### Code Organization
```
ui/src/
├── components/
│   ├── atoms/               # Basic components
│   ├── molecules/           # Composite components
│   ├── organisms/           # Complex components
│   └── layouts/             # Page layouts
├── hooks/                   # Custom React hooks
├── stores/                  # Zustand state management
├── types/                   # TypeScript type definitions
├── utils/                   # Utility functions
└── config/                  # Configuration files
```

#### Naming Conventions
- **Files**: `PascalCase.tsx` for components, `camelCase.ts` for utilities
- **Components**: `PascalCase`
- **Functions/Variables**: `camelCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Types/Interfaces**: `PascalCase`

#### Component Structure
```typescript
import React, { useState, useEffect } from 'react';
import { useAppStore } from '../../stores';
import { someUtil } from '../../utils/someUtil';

interface ComponentProps {
  title: string;
  onAction?: () => void;
}

const ComponentName: React.FC<ComponentProps> = ({ title, onAction }) => {
  const [state, setState] = useState<string>('');
  const { someStoreValue, someAction } = useAppStore();

  useEffect(() => {
    // Effect logic
  }, [someStoreValue]);

  const handleClick = () => {
    // Event handler logic
    onAction?.();
  };

  return (
    <div className="component-class">
      <h1>{title}</h1>
      {/* Component JSX */}
    </div>
  );
};

export default ComponentName;
```

#### State Management
```typescript
// Use Zustand for global state
import { create } from 'zustand';

interface AppState {
  data: string[];
  addData: (item: string) => void;
  removeData: (index: number) => void;
}

export const useAppStore = create<AppState>((set) => ({
  data: [],
  addData: (item) => set((state) => ({ 
    data: [...state.data, item] 
  })),
  removeData: (index) => set((state) => ({ 
    data: state.data.filter((_, i) => i !== index) 
  })),
}));
```

## 🔧 Configuration Management

### Configuration Structure
```typescript
interface AppConfig {
  app: AppSettings;
  obs: ObsSettings;
  udp: UdpSettings;
  logging: LoggingSettings;
  ui: UiSettings;
  video: VideoSettings;
  license: LicenseSettings;
  flags: FlagSettings;
  advanced: AdvancedSettings;
}
```

### Configuration Operations
```typescript
// Load configuration
const config = await configCommands.getSettings();

// Update configuration
await configCommands.updateSettings(updatedConfig);

// Export configuration
await configCommands.exportSettings(exportPath);

// Import configuration
await configCommands.importSettings(importPath);
```

## 🧪 Testing

### Backend Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_some_function() {
        // Test implementation
        assert_eq!(result, expected);
    }
}
```

### Frontend Testing
```typescript
import { render, screen } from '@testing-library/react';
import ComponentName from './ComponentName';

describe('ComponentName', () => {
  it('renders correctly', () => {
    render(<ComponentName title="Test" />);
    expect(screen.getByText('Test')).toBeInTheDocument();
  });
});
```

## 🐛 Debugging

### Backend Debugging
```rust
// Use log macros for debugging
log::info!("Debug message: {}", value);
log::warn!("Warning message");
log::error!("Error message: {}", error);

// Use println! for immediate output
println!("[DEBUG] Immediate debug output");
```

### Frontend Debugging
```typescript
// Use console methods
console.log('Debug message:', value);
console.warn('Warning message');
console.error('Error message:', error);

// Use React DevTools for component debugging
```

### Tauri Debugging
```bash
# Enable Tauri debug logging
RUST_LOG=debug cargo tauri dev

# Enable WebSocket debug messages
# Set debug_ws_messages to true in OBS plugin
```

## 📦 Building and Deployment

### Development Build
```bash
# Frontend build
cd ui && npm run build

# Backend build
cd src-tauri && cargo build

# Full development build
cargo tauri dev
```

### Production Build
```bash
# Production build
cargo tauri build

# Build artifacts will be in src-tauri/target/release/
```

### Distribution
```bash
# Create installer
cargo tauri build --release

# Installer will be in src-tauri/target/release/bundle/
```

## 🔄 Version Control

### Git Workflow
```bash
# Create feature branch
git checkout -b feature/new-feature

# Make changes and commit
git add .
git commit -m "feat: add new feature"

# Push to remote
git push origin feature/new-feature

# Create pull request
# Merge after review
```

### Commit Message Format
```
type(scope): description

feat: new feature
fix: bug fix
docs: documentation changes
style: formatting changes
refactor: code refactoring
test: adding tests
chore: maintenance tasks
```

## 🚨 Common Issues and Solutions

### Port Conflicts
```bash
# Clean ports before starting
./scripts/development/cleanup-dev-environment.sh --cleanup
```

### TypeScript Errors
```bash
# Check for type errors
cd ui && npm run type-check

# Fix import issues
# Ensure all imports are correct
```

### Rust Compilation Errors
```bash
# Check for compilation errors
cargo check

# Fix unused imports
cargo fix

# Run clippy for suggestions
cargo clippy
```

### OBS Connection Issues
```bash
# Check OBS WebSocket plugin
# Ensure v5 protocol is enabled
# Verify port and password settings
```

## 📚 Resources

### Documentation
- [Tauri Documentation](https://tauri.app/docs/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [React Documentation](https://react.dev/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)

### Tools
- [Rust Analyzer](https://rust-analyzer.github.io/)
- [React DevTools](https://react.dev/learn/react-developer-tools)
- [Tauri DevTools](https://tauri.app/docs/guides/debugging/)

---

**Last Updated**: 2025-01-28  
**Development Guide Version**: 2.0  
**Status**: Active Development 