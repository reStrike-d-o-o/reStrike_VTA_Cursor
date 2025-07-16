# Project Structure Guide

> **Note:** All architecture, onboarding, and coding conventions are defined in .cursor/rules/context.mdc (single source of truth). Project is Windows-only; Docker/devcontainer is fully removed. All onboarding, build, and documentation reference Windows-native setup only.

## Crate and Library Naming
- The crate and library name is `re_strike_vta` (snake_case, as required by Rust best practices).
- Avoid output filename collisions by ensuring the lib and bin targets have unique names in Cargo.toml.

## Plugin Module Integration
- All plugin modules are declared only in `src/plugins/mod.rs`.
- Plugins are re-exported via `pub mod plugins;` in `src/lib.rs`.
- Do not declare `mod` or `pub mod` for plugins anywhere else.

## Importing Types in Plugins
- All plugin files must import types using:
  ```rust
  use crate::types::{AppError, AppResult};
  ```
- Do not use relative imports like `super::super::types`.

## Avoiding Double Declarations
- Ensure there are no duplicate or conflicting `mod` or `pub mod` statements for plugins or types.
- Only `lib.rs` should declare `pub mod types;` and `pub mod plugins;`.

## Output Filename Collisions
- The library and binary targets must have unique names in Cargo.toml to avoid build errors.

## Directory Structure

```
reStrike_VTA/
├── 📁 .devcontainer/           # Development container configuration
│   ├── devcontainer.json      # Container settings
│   ├── Dockerfile             # Container image definition
│   └── README-devcontainer.md # Container usage guide
│
├── 📁 .github/                 # GitHub configuration
│   ├── ISSUE_TEMPLATE/        # Issue templates
│   │   ├── bug_report.md      # Bug report template
│   │   └── feature_request.md # Feature request & project tracker
│   ├── workflows/             # CI/CD workflows
│   │   └── ci.yml            # Continuous integration
│   └── dependabot.yml        # Dependency updates
│
├── 📁 .vscode/                 # VS Code configuration
│   ├── launch.json           # Debug configurations
│   ├── settings.json         # Editor settings
│   └── tasks.json            # Build tasks
│
├── 📁 config/                  # Configuration files
│   └── dev_resources.json    # Development environment config
│
├── 📁 docs/                    # Documentation
│   ├── 📁 api/                # API documentation
│   │   ├── obs-websocket.md  # OBS WebSocket API reference
│   │   └── udp-protocol.md   # UDP protocol specification
│   │
│   ├── 📁 development/        # Development guides
│   │   ├── container-restart.md # Container restart guide
│   │   ├── development-management.md # Dev environment management
│   │   ├── framework-updates.md # Framework update procedures
│   │   └── port-forwarding.md # Port configuration guide
│   │
│   ├── 📁 project/            # Project management
│   │   ├── project-tracker-guide.md # Project tracking system
│   │   ├── project-management-summary.md # Management overview
│   │   └── tracker-quick-reference.md # Quick reference guide
│   │
│   ├── 📁 requirements/       # Requirements and specifications
│   │   ├── instant-video-replay-prd.md # Product requirements
│   │   ├── software-requirements.md # Technical requirements
│   │   ├── ui-design-document.md # UI/UX specifications
│   │   └── FLAG_MANAGEMENT_MODULE.md # Flag management specifications
│   │
│   ├── 📁 integration/        # Integration guides
│   │   ├── obs-dual-protocol.md # OBS WebSocket implementation
│   │   └── obs-websocket-config.md # OBS configuration guide
│   │
│   ├── FLAG_MANAGEMENT_SYSTEM.md # Complete flag management documentation
│   ├── PROJECT_STRUCTURE.md   # This file
│   └── README.md              # Main documentation index
│
├── 📁 protocol/                # Protocol definitions
│   └── pss_schema.txt        # PSS protocol schema
│
├── 📁 scripts/                 # Automation scripts
│   ├── 📁 development/        # Development scripts
│   │   ├── cleanup-dev-environment.sh # Environment cleanup
│   │   ├── dev.sh             # Main development wrapper
│   │   ├── install-mpv-latest.sh # mpv installation
│   │   ├── manage-dev-resources.py # Resource management
│   │   ├── update-frameworks.sh # Framework updates
│   │   └── verify-ports.sh    # Port verification
│   │
│   ├── 📁 obs/                # OBS integration scripts
│   │   └── setup-obs-websocket.sh # OBS WebSocket setup
│   │
│   ├── 📁 project/            # Project management scripts
│   │   └── project-tracker.py # GitHub issue management
│   │
│   ├── 📁 media/              # Media processing scripts
│   │   ├── generate-clip.sh   # Video clip generation
│   │   └── download_ioc_flags.py # IOC flag download script
│   │
│   └── 📁 workflows/          # CI/CD workflows
│       └── ci.yml            # Continuous integration
│
├── 📁 src/                     # Rust backend source code
│   ├── 📁 plugins/            # Plugin modules
│   │   ├── license.rs         # License management
│   │   ├── obs.rs             # OBS WebSocket integration
│   │   ├── playback.rs        # Video playback
│   │   ├── store.rs           # Data storage
│   │   └── udp.rs             # UDP protocol handling
│   │
│   ├── 📁 commands/           # Tauri command handlers
│   │   └── tauri-commands.rs  # Frontend-backend bridge
│   │
│   └── main.rs                # Application entry point
│
├── 📁 ui/                      # React frontend
│   ├── 📁 public/             # Static assets
│   │   ├── index.html         # HTML template
│   │   └── 📁 assets/         # Static assets
│   │       └── 📁 flags/      # IOC flag images (253 flags)
│   │           ├── {IOC}.png  # Individual flag files
│   │           └── README.md  # Flag management documentation
│   │
│   ├── 📁 src/                # React source code
│   │   ├── 📁 components/     # React components
│   │   │   ├── ObsWebSocketManager.tsx # OBS connection manager
│   │   │   ├── Overlay.tsx    # Main overlay component
│   │   │   └── SidebarTest.tsx # Event table and filtering
│   │   │
│   │   ├── 📁 stores/         # State management
│   │   │   └── index.ts       # Zustand stores
│   │   │
│   │   ├── 📁 utils/          # Utility functions
│   │   │   └── flagUtils.tsx  # Flag management utilities
│   │   │
│   │   ├── App.tsx            # Main application component
│   │   └── index.tsx          # React entry point
│   │
│   ├── package.json           # Frontend dependencies
│   ├── tsconfig.json          # TypeScript configuration
│   └── eslint.config.js       # ESLint configuration
│
├── 📁 target/                  # Rust build artifacts (gitignored)
├── 📁 node_modules/            # Node.js dependencies (gitignored)
├── 📁 src-tauri/              # Tauri configuration (gitignored)
│
├── .cursor/                    # Cursor IDE configuration
├── .gitignore                  # Git ignore rules
├── Cargo.lock                  # Rust dependency lock
├── Cargo.toml                  # Rust project configuration
├── LICENSE                     # Project license
├── package-lock.json           # Node.js dependency lock
├── package.json                # Root project configuration
├── PROJECT_CONTEXT.md          # Project context and status
└── README.md                   # Project overview
```

## Organization Principles

### 1. **Separation of Concerns**
- **Backend**: Rust code in `src/`
- **Frontend**: React code in `ui/`
- **Documentation**: Organized by category in `docs/`
- **Scripts**: Categorized by purpose in `scripts/`
- **Assets**: Static files in `ui/public/assets/`

### 2. **Logical Grouping**
- **Development**: Container, VS Code, scripts
- **Documentation**: API, guides, requirements, integration
- **Source Code**: Backend plugins, frontend components
- **Configuration**: Environment and project settings
- **Media Assets**: Flags, images, and static content

### 3. **Scalability**
- **Modular Structure**: Easy to add new components
- **Clear Hierarchy**: Intuitive navigation
- **Consistent Naming**: Predictable file locations

### 4. **Maintainability**
- **Single Responsibility**: Each directory has a clear purpose
- **Easy Navigation**: Developers can quickly find files
- **Reduced Clutter**: No files scattered in root directory

## File Naming Conventions

### Documentation
- **Guides**: `kebab-case.md` (e.g., `container-restart.md`)
- **References**: `kebab-case.md` (e.g., `obs-websocket.md`)
- **Specifications**: `kebab-case.md` (e.g., `software-requirements.md`)

### Scripts
- **Development**: `kebab-case.sh` or `kebab-case.py`
- **Categories**: Grouped in subdirectories
- **Descriptive Names**: Clear purpose indication

### Source Code
- **Rust**: `snake_case.rs`
- **TypeScript/React**: `PascalCase.tsx` for components, `camelCase.ts` for utilities
- **Configuration**: `kebab-case.json` or `kebab-case.toml`

### Assets
- **Flags**: `{IOC}.png` (3-letter IOC country codes)
- **Images**: `kebab-case.png` or `kebab-case.jpg`
- **Documents**: `kebab-case.md` or `kebab-case.pdf`

## Import/Reference Paths

### Frontend Imports
```typescript
// Components
import ObsWebSocketManager from './components/ObsWebSocketManager';

// Stores
import { useAppStore } from './stores/index';

// Utilities
import { getFlagUrl } from './utils/flagUtils';

// Types
import { ObsConnectionConfig } from './types/obs';
```

### Backend Imports
```rust
// Plugins
mod plugins;
use plugins::obs::ObsPlugin;
use plugins::udp::UdpServer;

// Commands
mod commands;
use commands::tauri_commands::*;
```

### Documentation References
```markdown
<!-- Cross-references -->
See [Container Restart Guide](../development/container-restart.md)
See [OBS Configuration](../integration/obs-websocket-config.md)
See [Flag Management System](../FLAG_MANAGEMENT_SYSTEM.md)
```

### Asset References
```typescript
// Flag images
const flagUrl = '/assets/flags/USA.png';

// Static assets
const imageUrl = '/assets/images/logo.png';
```

## Error Handling Conventions

- All plugin and core methods must use `AppResult<T>` (from `crate::types`).
- Errors must be propagated as `AppError`.
- **AppError::IoError** is only for actual `std::io::Error` values.
- **Custom error messages** (including those created from strings or formatted text) must use **AppError::ConfigError** or another appropriate variant (e.g., `AppError::VideoError`, `AppError::ObsError`, etc.).
- Never use `AppError::IoError` with a `String` or formatted message.
- When returning errors to API responses or structs expecting Option<String>, always use e.to_string() to convert AppError to String.
- When converting std::io::Error to AppError, use AppError::IoError(e).
- When converting std::io::Error to AppError::ConfigError, use AppError::ConfigError(e.to_string()).

**Examples:**
```rust
// Convert AppError to String for API response:
error: Some(e.to_string())

// Convert std::io::Error to AppError:
.map_err(AppError::IoError)?

// Convert std::io::Error to AppError::ConfigError:
.map_err(|e| AppError::ConfigError(e.to_string()))?
```

## Maintenance Guidelines

### 1. **Adding New Files**
- Place in appropriate category directory
- Follow naming conventions
- Update this document if adding new categories

### 2. **Moving Files**
- Update all import/reference paths
- Update documentation links
- Update configuration files
- Test all functionality

### 3. **Regular Reviews**
- Monthly structure review
- Remove obsolete files
- Consolidate similar files
- Update documentation

### 4. **Automation**
- Use scripts for common operations
- Automated path updates where possible
- CI/CD integration for structure validation

## Benefits of This Structure

### For Developers
- **Quick Navigation**: Find files easily
- **Clear Purpose**: Understand file roles
- **Consistent Patterns**: Predictable organization
- **Reduced Confusion**: No scattered files

### For Project Management
- **Easy Onboarding**: New developers understand structure
- **Clear Documentation**: Organized by purpose
- **Maintainable**: Easy to update and extend
- **Scalable**: Grows with project needs

### For Maintenance
- **Logical Grouping**: Related files together
- **Clear Dependencies**: Easy to trace relationships
- **Consistent Updates**: Predictable change locations
- **Reduced Errors**: Less chance of broken references

---

**📝 Note**: This structure should be maintained and updated as the project evolves. All team members should follow these guidelines when adding or modifying files.

**🔄 Last Updated**: Current session - Flag management system implementation complete
**👤 Maintained by**: Development Team 