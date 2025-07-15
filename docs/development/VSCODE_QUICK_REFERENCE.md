# VSCode Quick Reference - Windows-Only Development

Essential commands and workflows for developing the reStrike VTA Windows desktop application.

> **Note**: This quick reference is for the Windows-only version of reStrike VTA, converted from the dual environment system at commit `4d222ceed0cd698b7e3ba0d7037f51388d553803`.

## 🚀 **Essential VSCode Extensions**

### **Install All Extensions (PowerShell)**
```powershell
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

## ⚡ **Quick Development Commands**

### **VSCode Tasks (Ctrl+Shift+P → "Tasks: Run Task")**
- `dev-tauri` - Start full Tauri development (React + Rust)
- `start-react` - Start React frontend only (port 3000)
- `build-tauri` - Build Rust backend
- `test-rust` - Run Rust tests
- `clean-tauri` - Clean Rust build artifacts
- `build-react` - Build React frontend

### **Terminal Commands**
```bash
# Full development
npm run dev

# Individual components
npm run start:react    # React frontend only
npm run start:tauri    # Full Tauri app

# Rust commands
cargo build           # Build Rust backend
cargo test            # Run Rust tests
cargo clean           # Clean build artifacts
cargo tauri dev       # Start Tauri development
```

## 🐛 **Debugging Workflows**

### **Frontend Debugging**
1. **React DevTools**: Install browser extension
2. **Console Logging**: Use comprehensive logging system
3. **State Inspection**: Use Zustand DevTools
4. **Network Tab**: Monitor Tauri command calls

### **Backend Debugging**
1. **Set Breakpoints**: Click line numbers in Rust files
2. **Start Debugging**: F5 or Ctrl+Shift+D → "Debug Tauri App"
3. **Logging**: Use `RUST_LOG=debug` environment variable
4. **Backtrace**: Use `RUST_BACKTRACE=1` for stack traces

### **Integration Debugging**
1. **Tauri DevTools**: Built-in debugging for Tauri commands
2. **Process Monitoring**: Task Manager → Monitor app processes
3. **File System**: Monitor file operations and permissions
4. **Network**: Monitor WebSocket and UDP connections

## ⌨️ **Essential Keyboard Shortcuts**

### **Navigation**
- `Ctrl+P` - Quick open files
- `Ctrl+T` - Search symbols
- `Ctrl+Shift+E` - Explorer
- `Ctrl+Shift+G` - Git
- `Ctrl+Shift+D` - Debug

### **Editing**
- `Ctrl+Shift+P` - Command palette
- `F2` - Rename symbol
- `Alt+Click` - Multi-cursor
- `Ctrl+Shift+[` - Fold code block
- `Ctrl+Shift+]` - Unfold code block

### **Terminal**
- `Ctrl+`` - Toggle terminal
- `Ctrl+Shift+`` - New terminal

## 🔧 **Common Development Workflows**

### **1. Daily Development Start**
```bash
# 1. Open VSCode in project root
code .

# 2. Install extensions (if first time)
# VSCode will prompt for recommended extensions

# 3. Start development
Ctrl+Shift+P → "Tasks: Run Task" → "dev-tauri"
```

### **2. Frontend-Only Development**
```bash
# For React-only development
Ctrl+Shift+P → "Tasks: Run Task" → "start-react"
# Or terminal: npm run start:react
```

### **3. Backend-Only Development**
```bash
# For Rust-only development
Ctrl+Shift+P → "Tasks: Run Task" → "build-tauri"
# Or terminal: cargo build
```

### **4. Testing**
```bash
# Run Rust tests
Ctrl+Shift+P → "Tasks: Run Task" → "test-rust"
# Or terminal: cargo test

# Run React tests
cd ui && npm test
```

### **5. Debugging**
```bash
# 1. Set breakpoints in code
# 2. F5 or Ctrl+Shift+D → "Debug Tauri App"
# 3. Use debug console for inspection
# 4. Use call stack and variables panels
```

## 🐛 **Troubleshooting**

### **Rust Analyzer Issues**
```bash
# Restart Rust Analyzer
Ctrl+Shift+P → "Rust Analyzer: Restart Server"
```

### **TypeScript Issues**
```bash
# Restart TypeScript server
Ctrl+Shift+P → "TypeScript: Restart TS Server"
```

### **Build Issues**
```bash
# Clean and rebuild
Ctrl+Shift+P → "Tasks: Run Task" → "clean-tauri"
Ctrl+Shift+P → "Tasks: Run Task" → "build-tauri"
```

### **VSCode Performance**
- Disable unnecessary extensions
- Use workspace-specific settings
- Monitor memory usage
- Restart VSCode if needed

## 📁 **File Organization**

### **Key Files for Development**
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

## 🎯 **Productivity Tips**

### **1. Use VSCode Features**
- **IntelliSense**: Auto-completion for Rust and TypeScript
- **Error Detection**: Real-time error checking
- **Refactoring**: F2 to rename symbols across files
- **Multi-cursor**: Alt+Click for multiple cursors

### **2. Git Integration**
- **Source Control**: Built-in Git support
- **GitLens**: Enhanced Git features
- **GitHub**: Direct GitHub integration
- **Branch management**: Easy branch switching

### **3. Performance Monitoring**
- **Task Manager**: Monitor process CPU and memory
- **Resource Monitor**: Detailed system resource usage
- **Performance Monitor**: Windows performance counters

## 📚 **Additional Resources**

### **Documentation**
- [VSCode Windows Setup Guide](./VSCODE_WINDOWS_SETUP.md)
- [Windows-Only Conversion Guide](./WINDOWS_ONLY_CONVERSION_GUIDE.md)
- [Project Context](../PROJECT_CONTEXT.md)

### **External Links**
- [VSCode User Guide](https://code.visualstudio.com/docs)
- [Rust in VSCode](https://code.visualstudio.com/docs/languages/rust)
- [Tauri VSCode Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)

---

**🎯 This quick reference provides all essential commands and workflows for efficient Windows-only development of the reStrike VTA application using VSCode.** 