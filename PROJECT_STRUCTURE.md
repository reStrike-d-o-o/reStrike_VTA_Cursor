# Project Structure

## AdvancedPanel (ui/src/components/layouts/AdvancedPanel.tsx)

The project follows a clean Tauri v2 structure with clear separation between frontend and backend:

```
reStrike_VTA_Cursor/
├── 📁 src-tauri/              # Tauri v2 application (Rust backend)
│   ├── 📁 src/                # Rust source code
│   │   ├── 📁 plugins/        # Plugin modules (OBS, PSS, Video)
│   │   ├── 📁 core/           # Core application logic
│   │   ├── 📁 types/          # Type definitions
│   │   └── 📁 utils/          # Utility functions
│   ├── 📁 icons/              # Application icons
│   ├── 📁 gen/                # Generated schemas
│   ├── Cargo.toml             # Rust dependencies
│   └── tauri.conf.json        # Tauri configuration
├── 📁 ui/                     # React frontend
│   ├── 📁 src/                # React source code
│   │   ├── 📁 components/     # React components (atomic design)
│   │   ├── 📁 hooks/          # React hooks
│   │   ├── 📁 utils/          # Utility functions
│   │   └── 📁 types/          # TypeScript types
│   ├── 📁 public/             # Static assets
│   │   └── 📁 assets/flags/   # 253 IOC flag images
│   └── package.json           # Node.js dependencies
├── 📁 docs/                   # Project documentation
├── 📁 scripts/                # Build and utility scripts
└── package.json               # Project-level scripts
``` 