# Windows + VSCode Development Setup Guide

## 🎯 **Complete Step-by-Step Guide for Windows Development**

This guide provides a comprehensive setup for developing the reStrike VTA project directly on Windows 10/11 using VSCode as the IDE. This is the **recommended approach** for production development and testing.

---

## 📋 **Prerequisites Checklist**

### **System Requirements**
- ✅ **Windows 10/11** (64-bit)
- ✅ **8GB RAM minimum** (16GB recommended)
- ✅ **10GB free disk space**
- ✅ **Administrator privileges** (for installation)
- ✅ **Internet connection** (for downloads)

### **Required Software**
- ✅ **VSCode** (Latest version)
- ✅ **Node.js** (v24+ LTS)
- ✅ **Rust** (Latest stable)
- ✅ **Git** (Latest version)
- ✅ **Python** (v3.8+ for scripts)
- ✅ **OBS Studio** (v29+ for testing)
- ✅ **mpv** (Windows build for video playback)

---

## 🚀 **Step 1: Install Core Development Tools**

### **1.1 Install VSCode**
```powershell
# Download and install VSCode from:
# https://code.visualstudio.com/download
# Choose "System Installer" for Windows
```

**VSCode Extensions to Install:**
- **Rust Analyzer** (rust-lang.rust-analyzer)
- **TypeScript and JavaScript Language Features** (built-in)
- **ES7+ React/Redux/React-Native snippets** (dsznajder.es7-react-js-snippets)
- **Tailwind CSS IntelliSense** (bradlc.vscode-tailwindcss)
- **GitLens** (eamodio.gitlens)
- **Thunder Client** (rangav.vscode-thunder-client)
- **Error Lens** (usernamehw.errorlens)
- **Auto Rename Tag** (formulahendry.auto-rename-tag)

### **1.2 Install Node.js**
```powershell
# Download Node.js LTS from:
# https://nodejs.org/en/download/
# Choose "Windows Installer (.msi)" 64-bit

# Verify installation
node --version  # Should show v24.x.x
npm --version   # Should show 10.x.x
```

### **1.3 Install Rust**
```powershell
# Download rustup-init.exe from:
# https://rustup.rs/

# Run the installer
.\rustup-init.exe

# Choose option 1 (default installation)
# Restart your terminal/PowerShell after installation

# Verify installation
rustc --version  # Should show rustc 1.75.x
cargo --version  # Should show cargo 1.75.x
```

### **1.4 Install Git**
```powershell
# Download Git from:
# https://git-scm.com/download/win

# Use default settings during installation
# Verify installation
git --version
```

### **1.5 Install Python**
```powershell
# Download Python from:
# https://www.python.org/downloads/
# Choose "Windows installer (64-bit)"

# IMPORTANT: Check "Add Python to PATH" during installation
# Verify installation
python --version  # Should show Python 3.8+
pip --version
```

---

## 🛠️ **Step 2: Install Tauri CLI and Dependencies**

### **2.1 Install Tauri CLI**
```powershell
# Open PowerShell as Administrator
cargo install tauri-cli

# Verify installation
tauri --version
```

### **2.2 Install Windows Build Tools**
```powershell
# Install Visual Studio Build Tools (required for Rust compilation)
# Download from: https://visualstudio.microsoft.com/downloads/
# Choose "Build Tools for Visual Studio 2022"

# During installation, select:
# - MSVC v143 - VS 2022 C++ x64/x86 build tools
# - Windows 10/11 SDK
# - CMake tools for Visual Studio

# Alternative: Install via winget
winget install Microsoft.VisualStudio.2022.BuildTools
```

### **2.3 Install Additional Dependencies**
```powershell
# Install Chocolatey (Windows package manager)
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Install additional tools
choco install 7zip
choco install wget
```

---

## 📁 **Step 3: Clone and Setup Project**

### **3.1 Clone Repository**
```powershell
# Create development directory
mkdir C:\dev
cd C:\dev

# Clone the repository
git clone https://github.com/reStrike-d-o-o/reStrike_VTA_Cursor.git
cd reStrike_VTA_Cursor

# Verify repository
git status
```

### **3.2 Open in VSCode**
```powershell
# Open project in VSCode
code .

# Or from VSCode: File > Open Folder > C:\dev\reStrike_VTA_Cursor
```

### **3.3 Install Project Dependencies**
```powershell
# Install root dependencies
npm install

# Install UI dependencies
cd ui
npm install

# Install specific React Scripts version (if needed)
npm install react-scripts@5.0.1 --save-dev

# Return to root
cd ..
```

---

## ⚙️ **Step 4: Configure Development Environment**

### **4.1 Set Environment Variables**
```powershell
# Set environment variable for Windows development
$env:REACT_APP_ENVIRONMENT = "windows"

# Or add to Windows Environment Variables permanently:
# 1. Open System Properties > Advanced > Environment Variables
# 2. Add new User Variable:
#    Name: REACT_APP_ENVIRONMENT
#    Value: windows
```

### **4.2 Configure VSCode Settings**
Create `.vscode/settings.json` in the project root:
```json
{
  "typescript.preferences.importModuleSpecifier": "relative",
  "typescript.suggest.autoImports": true,
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "files.associations": {
    "*.rs": "rust"
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.buildScripts.enable": true,
  "tailwindCSS.includeLanguages": {
    "typescript": "javascript",
    "typescriptreact": "javascript"
  }
}
```

### **4.3 Configure VSCode Launch Configuration**
Create `.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Launch Tauri App",
      "type": "node",
      "request": "launch",
      "program": "${workspaceFolder}/ui/node_modules/.bin/react-scripts",
      "args": ["start"],
      "cwd": "${workspaceFolder}/ui",
      "env": {
        "REACT_APP_ENVIRONMENT": "windows"
      },
      "console": "integratedTerminal"
    },
    {
      "name": "Debug Rust Backend",
      "type": "lldb",
      "request": "launch",
      "program": "${workspaceFolder}/target/debug/restrike-vta",
      "args": [],
      "cwd": "${workspaceFolder}",
      "console": "integratedTerminal"
    }
  ]
}
```

---

## 🧪 **Step 5: Install Testing Dependencies**

### **5.1 Install OBS Studio**
```powershell
# Download OBS Studio from:
# https://obsproject.com/download

# Install with default settings
# IMPORTANT: Disable WebSocket authentication in OBS settings
# Tools > WebSocket Server Settings > Uncheck "Enable Authentication"
```

### **5.2 Install mpv Player**
```powershell
# Download mpv for Windows from:
# https://mpv.io/installation/

# Extract to C:\mpv
# Add C:\mpv to PATH environment variable
```

### **5.3 Install Additional Testing Tools**
```powershell
# Install Postman (for API testing)
# Download from: https://www.postman.com/downloads/

# Install Wireshark (for network debugging)
choco install wireshark
```

---

## 🚀 **Step 6: Development Workflow**

### **6.1 Start Development Environment**
```powershell
# Method 1: Using npm scripts
npm run start:windows

# Method 2: Using Tauri CLI
npm run start:tauri

# Method 3: Manual start
cd ui
npm start
# In another terminal:
cargo tauri dev
```

### **6.2 Development Commands**
```powershell
# Frontend only (React)
cd ui
npm start

# Backend only (Rust)
cargo run

# Full Tauri app
npm run start:tauri

# Build for production
npm run build:windows

# Run tests
npm test
cargo test

# Clean build
cargo clean
npm run build
```

### **6.3 Environment-Specific Development**
```powershell
# Web environment (browser only)
npm run start:web

# Windows environment (Tauri app)
npm run start:windows

# Build for web
npm run build:web

# Build for Windows
npm run build:windows
```

---

## 🔧 **Step 7: Debugging and Troubleshooting**

### **7.1 Common Issues and Solutions**

#### **Rust Compilation Errors**
```powershell
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build

# Check for missing dependencies
cargo check
```

#### **Node.js/npm Issues**
```powershell
# Clear npm cache
npm cache clean --force

# Delete node_modules and reinstall
rm -rf node_modules
npm install

# Update npm
npm install -g npm@latest
```

#### **Tauri Build Issues**
```powershell
# Check Tauri requirements
cargo tauri info

# Update Tauri CLI
cargo install tauri-cli --force

# Check system requirements
cargo tauri doctor
```

### **7.2 Debugging Tools**

#### **VSCode Debugging**
- **Frontend**: Use Chrome DevTools (F12 in browser)
- **Backend**: Use VSCode debugger with launch.json
- **Tauri**: Use integrated debugging in VSCode

#### **Logging and Monitoring**
```powershell
# View application logs
# Check console output in VSCode terminal
# Use browser DevTools for frontend debugging
# Use Rust logging for backend debugging
```

---

## 📊 **Step 8: Testing and Quality Assurance**

### **8.1 Running Tests**
```powershell
# Frontend tests
cd ui
npm test

# Backend tests
cargo test

# Integration tests
cargo test --test integration

# E2E tests (if configured)
npm run test:e2e
```

### **8.2 Code Quality Tools**
```powershell
# ESLint (Frontend)
cd ui
npm run lint

# Clippy (Rust)
cargo clippy

# Format code
cargo fmt
npm run format
```

### **8.3 Performance Testing**
```powershell
# Build performance
npm run build:windows -- --analyze

# Runtime performance
# Use browser DevTools Performance tab
# Use Windows Performance Monitor
```

---

## 🚀 **Step 9: Production Build and Deployment**

### **9.1 Build Production Executable**
```powershell
# Build Windows executable
npm run build:windows

# Build with installer
cargo tauri build

# Output location: src-tauri/target/release/
```

### **9.2 Create Installer**
```powershell
# Build MSI installer
cargo tauri build --target x86_64-pc-windows-msvc

# Installer will be in: src-tauri/target/release/bundle/msi/
```

### **9.3 Distribution**
```powershell
# Create distribution package
# Copy executable and dependencies
# Create installer package
# Test on clean Windows system
```

---

## 📚 **Step 10: Documentation and Resources**

### **10.1 Project Documentation**
- **PROJECT_CONTEXT.md**: Complete project overview
- **README.md**: Quick start and basic information
- **docs/development/**: Development guides
- **docs/api/**: API documentation
- **docs/integration/**: Integration guides

### **10.2 External Resources**
- **Tauri Documentation**: https://tauri.app/docs/
- **React Documentation**: https://react.dev/
- **Rust Documentation**: https://doc.rust-lang.org/
- **TypeScript Documentation**: https://www.typescriptlang.org/docs/

### **10.3 Community and Support**
- **GitHub Issues**: https://github.com/reStrike-d-o-o/reStrike_VTA_Cursor/issues
- **Project Board**: https://github.com/orgs/reStrike-d-o-o/projects/3
- **Tauri Discord**: https://discord.gg/tauri
- **Rust Community**: https://users.rust-lang.org/

---

## ✅ **Verification Checklist**

### **Installation Verification**
- [ ] VSCode opens project without errors
- [ ] Node.js and npm are working
- [ ] Rust and Cargo are working
- [ ] Tauri CLI is installed
- [ ] All dependencies are installed
- [ ] Environment variables are set

### **Development Verification**
- [ ] React frontend starts on port 3000
- [ ] Rust backend compiles without errors
- [ ] Tauri app launches successfully
- [ ] Hot reload is working
- [ ] Debugging is functional
- [ ] Tests are passing

### **Integration Verification**
- [ ] OBS WebSocket connection works
- [ ] Video playback is functional
- [ ] Flag system displays correctly
- [ ] Environment switching works
- [ ] All components render properly
- [ ] Error handling is working

---

## 🎯 **Next Steps**

1. **Start Development**: Begin with `npm run start:windows`
2. **Explore Components**: Review the 6 main React components
3. **Test Features**: Verify OBS integration and video playback
4. **Run Tests**: Ensure all tests are passing
5. **Build Production**: Create Windows executable
6. **Deploy**: Test on target Windows systems

---

## 📞 **Support and Troubleshooting**

If you encounter issues:

1. **Check the troubleshooting section above**
2. **Review project documentation**
3. **Check GitHub issues for similar problems**
4. **Create a new issue with detailed information**
5. **Contact the development team**

**Happy coding! 🚀**