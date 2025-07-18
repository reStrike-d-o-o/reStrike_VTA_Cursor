# reStrike VTA

A Windows-only native desktop application for instant video replay and analysis in sports broadcasting. Built with Tauri v2 (Rust backend) and React (frontend).

## 🚀 Current Status

✅ **Native Windows Mode**: Successfully running as native Windows desktop application  
✅ **Tauri v2 Integration**: Complete migration with all features working  
✅ **Hot Reload**: Development mode with live reload for both frontend and backend  
✅ **Environment Detection**: Automatic detection of Tauri API availability  

## 🏗️ Architecture

- **Backend**: Rust with Tauri v2 for native Windows integration
- **Frontend**: React 18 with TypeScript and Tailwind CSS
- **Design System**: Atomic design with reusable components
- **State Management**: React hooks and context
- **Build System**: Integrated Tauri build process

## 📁 Project Structure

```
reStrike_VTA_Cursor/
├── src-tauri/                    # Tauri v2 backend (Rust)
│   ├── src/                      # Rust source code
│   │   ├── main.rs              # Tauri app entry point
│   │   ├── tauri_commands.rs    # Tauri command definitions
│   │   ├── plugins/             # Plugin modules (obs, playback, store, udp)
│   │   ├── obs/                 # OBS WebSocket integration
│   │   ├── pss/                 # PSS protocol handling
│   │   └── video/               # Video player integration
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
├── ui/                          # React frontend
│   ├── src/components/          # Atomic design components
│   │   ├── atoms/               # Basic UI components
│   │   ├── molecules/           # Composite components
│   │   ├── organisms/           # Complex UI sections
│   │   └── layouts/             # Page and section layouts
│   ├── src/hooks/               # Custom React hooks
│   ├── src/utils/               # Utility functions
│   └── public/assets/flags/     # Country flag images
├── docs/                        # Project documentation
└── scripts/                     # Development scripts
```

## 🛠️ Development Setup

### Prerequisites

- **Windows 10/11**: Primary development platform
- **Rust**: Latest stable version
- **Node.js**: Version 18 or higher
- **Git**: Version control

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd reStrike_VTA_Cursor
   ```

2. **Install frontend dependencies**
   ```bash
   cd ui
   npm install
   ```

3. **Install Rust dependencies**
   ```bash
   cd ../src-tauri
   cargo build
   ```

### Development Workflow

#### Quick Start (Recommended)
```bash
# From project root - starts both frontend and backend
cd src-tauri
cargo tauri dev
```

This single command:
1. Starts React development server (port 3000)
2. Builds Rust backend
3. Launches native Windows application
4. Enables hot reload for both frontend and backend

#### Manual Development
```bash
# Terminal 1: Start React dev server
cd ui
npm run start:fast

# Terminal 2: Start Tauri app
cd src-tauri
cargo tauri dev
```

### Build Commands

```bash
# Development build
cd ui
npm run build

# Production build with Tauri
cd src-tauri
cargo tauri build
```

## 🎯 Key Features

### Core Functionality
- **Instant Video Replay**: Quick access to recent video clips
- **Event Tracking**: Real-time event capture and analysis
- **OBS Integration**: Seamless connection with OBS Studio
- **Flag Management**: Country flag recognition and display
- **Advanced Panel**: Comprehensive settings and diagnostics

### UI Components
- **DockBar**: Main sidebar with player info and controls
- **Event Table**: Real-time event display with filtering
- **Advanced Panel**: Settings, diagnostics, and configuration
- **Status Indicators**: Real-time system status display

### Technical Features
- **Environment Detection**: Automatic Tauri vs Web mode detection
- **Plugin Architecture**: Modular backend design
- **Error Handling**: Comprehensive error management
- **Hot Reload**: Development efficiency with live updates
- **Type Safety**: Full TypeScript and Rust type safety

## 🔧 Configuration

### Environment Detection
The application automatically detects whether it's running in:
- **Native Mode**: Tauri API available (`window.__TAURI__` exists)
- **Web Mode**: Running in browser without Tauri

### Tauri Configuration
- **Global Tauri API**: Enabled for frontend access
- **Development Server**: React dev server integration
- **Build Configuration**: Optimized for Windows
- **Security**: Proper allowlist configuration

## 📚 Documentation

### Key Documents
- [Project Structure](PROJECT_STRUCTURE.md): Detailed project organization
- [Project Context](PROJECT_CONTEXT.md): High-level project overview
- [Frontend Development Summary](FRONTEND_DEVELOPMENT_SUMMARY.md): Frontend architecture details
- [Project Reorganization Summary](PROJECT_REORGANIZATION_SUMMARY.md): Migration history

### Development Guides
- [Development Setup](docs/development/): Development environment setup
- [API Documentation](docs/api/): API reference
- [Integration Guides](docs/integration/): Integration documentation

## 🚨 Troubleshooting

### Common Issues

#### Port Conflicts
```bash
# Clean up ports before starting
./scripts/development/cleanup-dev-environment.sh --cleanup
```

#### Build Errors
```bash
# Clean and rebuild
cd src-tauri
cargo clean
cargo build
```

#### Tauri API Issues
- Verify environment detection in browser console
- Check that `window.__TAURI__` is available
- Ensure Tauri commands are properly registered

#### Hot Reload Issues
- Verify React development server is running on port 3000
- Check Tauri configuration for correct dev path
- Restart both frontend and backend servers

### Development Tips
- Use React DevTools for frontend debugging
- Monitor Tauri console for backend issues
- Check browser console for frontend errors
- Verify environment detection in development

## 🎨 UI/UX Features

### Design System
- **Atomic Design**: Organized component hierarchy
- **Dark Theme**: Professional dark theme with blue accents
- **Responsive Design**: Works on different screen sizes
- **Accessibility**: WCAG AA compliance

### Component Architecture
- **Atoms**: Basic UI components (Button, Input, Checkbox, etc.)
- **Molecules**: Composite components (EventTable, LogToggleGroup, etc.)
- **Organisms**: Complex UI sections (DockBar, AdvancedPanel, etc.)
- **Layouts**: Page and section layouts

## 🔮 Future Enhancements

### Immediate Priorities
1. **OBS Integration**: Complete WebSocket protocol implementation
2. **Event System**: Implement PSS protocol event handling
3. **Video Player**: Integrate mpv video player
4. **Flag Management**: Complete flag recognition system

### Future Features
1. **AI Integration**: Automated event analysis
2. **Advanced Analytics**: Statistical analysis and reporting
3. **Multi-language Support**: Internationalization
4. **Plugin System**: Extensible plugin architecture

## 🤝 Contributing

### Development Guidelines
- Follow atomic design principles
- Maintain type safety with TypeScript
- Use proper error handling
- Write comprehensive documentation

### Code Quality
- Run linting and type checking
- Follow project coding conventions
- Test functionality thoroughly
- Update documentation as needed

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

For support and questions:
- Check the [documentation](docs/)
- Review [troubleshooting guide](#troubleshooting)
- Open an issue for bugs or feature requests

---

**Last Updated**: December 2024  
**Status**: ✅ Native Windows Mode - Ready for Development