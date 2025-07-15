# VSCode Windows-Only Development Setup

Complete guide for developing and debugging the reStrike VTA Windows desktop application using VSCode.

## 🚀 **Essential VSCode Extensions**

### **Core Development Extensions**
```json
{
  "recommendations": [
    // Rust Development
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    
    // TypeScript/React Development
    "ms-vscode.vscode-typescript-next",
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode",
    "ms-vscode.vscode-eslint",
    
    // Tauri Development
    "tauri-apps.tauri-vscode",
    
    // Git & Project Management
    "eamodio.gitlens",
    "ms-vscode.vscode-github",
    
    // Debugging & Testing
    "ms-vscode.vscode-json",
    "ms-vscode.test-adapter-converter",
    
    // Productivity
    "ms-vscode.vscode-json",
    "formulahendry.auto-rename-tag",
    "christian-kohler.path-intellisense",
    "ms-vscode.vscode-npm-script"
  ]
}
```

### **Installation Command**
```bash
# Install all recommended extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension serayuzgur.crates
code --install-extension ms-vscode.vscode-typescript-next
code --install-extension bradlc.vscode-tailwindcss
code --install-extension esbenp.prettier-vscode
code --install-extension ms-vscode.vscode-eslint
code --install-extension tauri-apps.tauri-vscode
code --install-extension eamodio.gitlens
code --install-extension ms-vscode.vscode-github
```

## ⚙️ **VSCode Workspace Configuration**

### **`.vscode/settings.json`**
```json
{
  // Rust Configuration
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.buildScripts.enable": true,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.lens.enable": true,
  "rust-analyzer.lens.implementations.enable": true,
  "rust-analyzer.lens.references.adt.enable": true,
  "rust-analyzer.lens.references.trait.enable": true,
  "rust-analyzer.lens.references.enumVariant.enable": true,
  "rust-analyzer.lens.references.method.enable": true,
  
  // TypeScript Configuration
  "typescript.preferences.includePackageJsonAutoImports": "on",
  "typescript.suggest.autoImports": true,
  "typescript.updateImportsOnFileMove.enabled": "always",
  
  // React Configuration
  "emmet.includeLanguages": {
    "typescript": "html",
    "typescriptreact": "html"
  },
  
  // Tailwind CSS
  "tailwindCSS.includeLanguages": {
    "typescript": "html",
    "typescriptreact": "html"
  },
  "tailwindCSS.experimental.classRegex": [
    ["cva\\(([^)]*)\\)", "[\"'`]([^\"'`]*).*?[\"'`]"],
    ["cx\\(([^)]*)\\)", "(?:'|\"|`)([^']*)(?:'|\"|`)"]
  ],
  
  // Formatting
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[typescriptreact]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  
  // File Associations
  "files.associations": {
    "*.toml": "toml"
  },
  
  // Search Configuration
  "search.exclude": {
    "**/target": true,
    "**/node_modules": true,
    "**/dist": true,
    "**/build": true
  },
  
  // Terminal Configuration
  "terminal.integrated.defaultProfile.windows": "PowerShell",
  "terminal.integrated.profiles.windows": {
    "PowerShell": {
      "source": "PowerShell",
      "args": ["-NoExit", "-Command", "cd '${workspaceFolder}'"]
    }
  }
}
```

### **`.vscode/launch.json`**
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug Tauri App",
      "type": "lldb",
      "request": "launch",
      "program": "${workspaceFolder}/src-tauri/target/debug/reStrike_VTA.exe",
      "args": [],
      "cwd": "${workspaceFolder}/src-tauri",
      "preLaunchTask": "build-tauri",
      "env": {
        "RUST_BACKTRACE": "1",
        "RUST_LOG": "debug"
      }
    },
    {
      "name": "Debug Rust Backend",
      "type": "lldb",
      "request": "launch",
      "program": "${workspaceFolder}/src-tauri/target/debug/reStrike_VTA.exe",
      "args": [],
      "cwd": "${workspaceFolder}/src-tauri",
      "env": {
        "RUST_BACKTRACE": "1",
        "RUST_LOG": "debug"
      }
    },
    {
      "name": "Attach to Tauri Process",
      "type": "lldb",
      "request": "attach",
      "program": "${workspaceFolder}/src-tauri/target/debug/reStrike_VTA.exe",
      "processId": "${command:pickProcess}"
    }
  ]
}
```

### **`.vscode/tasks.json`**
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "build-tauri",
      "type": "shell",
      "command": "cargo",
      "args": ["build"],
      "group": "build",
      "options": {
        "cwd": "${workspaceFolder}/src-tauri"
      },
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared",
        "showReuseMessage": true,
        "clear": false
      },
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "dev-tauri",
      "type": "shell",
      "command": "cargo",
      "args": ["tauri", "dev"],
      "group": "build",
      "options": {
        "cwd": "${workspaceFolder}"
      },
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared",
        "showReuseMessage": true,
        "clear": false
      },
      "isBackground": true,
      "problemMatcher": {
        "pattern": [
          {
            "regexp": ".",
            "file": 1,
            "location": 2,
            "message": 3
          }
        ],
        "background": {
          "activeOnStart": true,
          "beginsPattern": ".*",
          "endsPattern": ".*"
        }
      }
    },
    {
      "label": "clean-tauri",
      "type": "shell",
      "command": "cargo",
      "args": ["clean"],
      "group": "build",
      "options": {
        "cwd": "${workspaceFolder}/src-tauri"
      }
    },
    {
      "label": "test-rust",
      "type": "shell",
      "command": "cargo",
      "args": ["test"],
      "group": "test",
      "options": {
        "cwd": "${workspaceFolder}/src-tauri"
      },
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      },
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "start-react",
      "type": "shell",
      "command": "npm",
      "args": ["start"],
      "group": "build",
      "options": {
        "cwd": "${workspaceFolder}/ui"
      },
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      },
      "isBackground": true,
      "problemMatcher": {
        "pattern": [
          {
            "regexp": ".",
            "file": 1,
            "location": 2,
            "message": 3
          }
        ],
        "background": {
          "activeOnStart": true,
          "beginsPattern": ".*",
          "endsPattern": ".*"
        }
      }
    },
    {
      "label": "build-react",
      "type": "shell",
      "command": "npm",
      "args": ["run", "build"],
      "group": "build",
      "options": {
        "cwd": "${workspaceFolder}/ui"
      },
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    }
  ]
}
```

## 🔧 **Development Workflow**

### **1. Daily Development Commands**

#### **Quick Start (Terminal)**
```bash
# Start development environment
npm run dev

# Or individual components
npm run start:react    # React frontend only
npm run start:tauri    # Full Tauri app
```

#### **VSCode Tasks (Ctrl+Shift+P → "Tasks: Run Task")**
- `dev-tauri` - Start full Tauri development
- `start-react` - Start React frontend only
- `build-tauri` - Build Rust backend
- `test-rust` - Run Rust tests
- `clean-tauri` - Clean Rust build artifacts

### **2. Debugging Workflows**

#### **Frontend Debugging**
1. **React DevTools**: Install React Developer Tools browser extension
2. **Console Logging**: Use the comprehensive logging system
3. **State Inspection**: Use Zustand DevTools for state debugging
4. **Network Tab**: Monitor Tauri command calls

#### **Backend Debugging**
1. **Rust Analyzer**: Real-time error checking and code completion
2. **LLDB Debugger**: Set breakpoints in Rust code
3. **Logging**: Use `RUST_LOG=debug` for detailed logging
4. **Backtrace**: Use `RUST_BACKTRACE=1` for stack traces

#### **Integration Debugging**
1. **Tauri DevTools**: Built-in debugging for Tauri commands
2. **Process Monitoring**: Use Task Manager to monitor app processes
3. **File System**: Monitor file operations and permissions
4. **Network**: Monitor WebSocket and UDP connections

### **3. Performance Monitoring**

#### **VSCode Extensions for Performance**
- **Performance Monitor**: Built-in VSCode performance tools
- **Memory Usage**: Monitor memory consumption
- **CPU Profiling**: Profile CPU usage during development

#### **Windows Tools**
- **Task Manager**: Monitor process CPU and memory
- **Resource Monitor**: Detailed system resource usage
- **Performance Monitor**: Windows performance counters

## 🐛 **Debugging Strategies**

### **1. Frontend Issues**
```typescript
// Use comprehensive logging system
import { logger } from './utils/logger';

logger.info('Component mounted', { component: 'VideoClips' });
logger.error('API call failed', { error, endpoint: '/api/status' });
logger.debug('State update', { prevState, newState });
```

### **2. Backend Issues**
```rust
// Use Rust logging
use log::{info, error, debug};

info!("Plugin initialized: {}", plugin_name);
error!("Failed to parse message: {:?}", error);
debug!("Processing event: {:?}", event);
```

### **3. Integration Issues**
```typescript
// Monitor Tauri command calls
const result = await invoke('obs_connect', { url: 'ws://localhost:4455' });
console.log('Tauri command result:', result);
```

## 📁 **File Organization for VSCode**

### **Recommended VSCode Workspace Layout**
```
reStrike_VTA/
├── .vscode/                    # VSCode configuration
│   ├── settings.json          # Workspace settings
│   ├── launch.json            # Debug configurations
│   ├── tasks.json             # Build tasks
│   └── extensions.json        # Recommended extensions
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── main.rs           # Main entry point
│   │   ├── plugins/          # Plugin modules
│   │   └── commands/         # Tauri commands
│   └── Cargo.toml           # Rust dependencies
├── ui/                       # React frontend
│   ├── src/
│   │   ├── components/       # React components
│   │   ├── hooks/           # Custom hooks
│   │   ├── utils/           # Utility functions
│   │   └── App.tsx          # Main app component
│   └── package.json         # Node.js dependencies
└── README.md                # Project documentation
```

## 🚀 **Productivity Tips**

### **1. Keyboard Shortcuts**
- `Ctrl+Shift+P` - Command palette
- `Ctrl+Shift+E` - Explorer
- `Ctrl+Shift+G` - Git
- `Ctrl+Shift+D` - Debug
- `Ctrl+Shift+X` - Extensions
- `Ctrl+`` - Terminal

### **2. VSCode Features**
- **Multi-cursor**: `Alt+Click` for multiple cursors
- **Code folding**: `Ctrl+Shift+[` to fold code blocks
- **Quick open**: `Ctrl+P` to quickly open files
- **Symbol search**: `Ctrl+T` to search for symbols
- **Refactoring**: `F2` to rename symbols

### **3. Git Integration**
- **Source Control**: Built-in Git support
- **GitLens**: Enhanced Git features
- **GitHub**: Direct GitHub integration
- **Branch management**: Easy branch switching and creation

## 🔍 **Troubleshooting**

### **Common Issues**

#### **Rust Analyzer Not Working**
```bash
# Restart Rust Analyzer
Ctrl+Shift+P → "Rust Analyzer: Restart Server"
```

#### **TypeScript Errors**
```bash
# Restart TypeScript server
Ctrl+Shift+P → "TypeScript: Restart TS Server"
```

#### **Build Issues**
```bash
# Clean and rebuild
cargo clean
cargo build
```

#### **VSCode Performance**
- Disable unnecessary extensions
- Use workspace-specific settings
- Monitor memory usage
- Restart VSCode if needed

## 📚 **Additional Resources**

### **VSCode Documentation**
- [VSCode User Guide](https://code.visualstudio.com/docs)
- [Rust in VSCode](https://code.visualstudio.com/docs/languages/rust)
- [TypeScript in VSCode](https://code.visualstudio.com/docs/languages/typescript)

### **Tauri Documentation**
- [Tauri VSCode Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [Tauri Development Guide](https://tauri.app/v1/guides/getting-started/setup/)

### **Project-Specific**
- [Windows-Only Conversion Guide](./WINDOWS_ONLY_CONVERSION_GUIDE.md)
- [Development Environment Setup](./WINDOWS_VSCODE_SETUP_GUIDE.md)
- [Project Context](../PROJECT_CONTEXT.md)

---

**🎯 This setup provides optimal development and debugging experience for the Windows-only reStrike VTA application using VSCode.** 