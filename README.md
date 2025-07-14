# reStrike VTA

Overlay and automation toolkit for VTA using Tauri and React.

## Project Overview
reStrike VTA is designed to provide a modern overlay and automation solution for VTA, integrating UDP ingestion, OBS control, and license management.

## Directory Structure
```
reStrike_VTA/
├── src/            # Application source code
├── tests/          # Unit and integration tests
├── public/         # Static assets (if applicable)
├── scripts/        # Automation scripts (build, deploy)
├── .github/
│   ├── workflows/  # CI/CD workflows
│   └── ISSUE_TEMPLATE/  # GitHub issue templates
├── docs/           # Design docs and API specs
├── LICENSE
├── package.json    # Dependencies and scripts (Node.js/Tauri)
└── README.md
```

## Development Environment
- **OS:** Windows 10/11, Mac, Linux (Windows recommended for OBS/mpv integration)
- **Node.js:** v24+ (latest LTS recommended)
- **Rust:** Stable (install via [rustup.rs](https://rustup.rs/))
- **Tauri CLI:** Install with `cargo install tauri-cli`
- **Frontend:** React 18 + TypeScript + Zustand + Tailwind CSS + framer-motion
- **Bundler:** Tauri (for native desktop app)
- **Linting:** ESLint (with TypeScript and React plugin)

## Quick Start
1. **Clone the repository:**
   ```bash
   git clone https://github.com/reStrike-d-o-o/reStrike_VTA_Cursor
   cd reStrike_VTA
   ```
2. **Install Rust and Cargo:**
   - Download and run the installer from [https://rustup.rs/](https://rustup.rs/)
   - Or in PowerShell:
     ```powershell
     Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/init.exe -OutFile rustup-init.exe
     .\rustup-init.exe
     ```
   - Restart your terminal after installation.
3. **Install Tauri CLI:**
   ```bash
   cargo install tauri-cli
   ```
4. **Install Node.js dependencies:**
   ```bash
   npm install
   cd ui
   npm install
   # If you see errors about react-scripts, run:
   npm install react-scripts@5.0.1 --save-dev
   npm install
   ```
5. **Start the development server:**
   ```bash
   cd ui
   npm run start
   ```
6. **Run backend (Tauri):**
   ```bash
   cd ..
   npm run start
   ```

## Project Structure

The project follows a well-organized structure for maintainability and clarity:

```
reStrike_VTA/
├── 📁 docs/                    # Documentation (organized by category)
│   ├── 📁 api/                # API documentation
│   ├── 📁 development/        # Development guides and checklists
│   ├── 📁 project/            # Project management
│   ├── 📁 requirements/       # Requirements and specifications
│   └── 📁 integration/        # Integration guides
├── 📁 scripts/                 # Automation scripts (categorized)
│   ├── 📁 development/        # Development environment scripts
│   ├── 📁 obs/                # OBS integration scripts
│   ├── 📁 project/            # Project management scripts
│   └── 📁 media/              # Media processing scripts
├── 📁 src/                     # Rust backend (organized modules)
│   ├── 📁 plugins/            # Plugin modules
│   └── 📁 commands/           # Tauri command handlers
└── 📁 ui/                      # React frontend
```

For detailed structure information, see [Project Structure Guide](./docs/PROJECT_STRUCTURE.md).

## Development Environment

### Dev Container Verification & Automation

- **Checklists**: See [Development Checklists](./docs/development/checklists/) for verification steps
- **Container Restart**: See [Container Restart Guide](./docs/development/container-restart.md) for framework updates
- **Environment Management**: See [Development Management](./docs/development/development-management.md) for tools and scripts

### Quick Start Commands

```bash
# Main development wrapper
./scripts/development/dev.sh help

# Start all services
./scripts/development/dev.sh start-all

# Check status
./scripts/development/dev.sh status

# Clean up environment
./scripts/development/dev.sh cleanup
```

## Project Management & Tracking

- **Project Tracker**: Use the comprehensive [Feature Request Template](./.github/ISSUE_TEMPLATE/feature_request.md) as a project tracker
- **Tracker Guide**: See [Project Tracker Guide](./docs/project/project-tracker-guide.md) for detailed instructions
- **Quick Reference**: See [Tracker Quick Reference](./docs/project/tracker-quick-reference.md) for common commands
- **Management Scripts**: Use `scripts/project/project-tracker.py` for automated issue management

## Troubleshooting
- **'cargo' is not recognized:**
  - Rust is not installed or not in your PATH. Install from [https://rustup.rs/](https://rustup.rs/), then restart your terminal.
- **'react-scripts' is not recognized:**
  - Run `npm install react-scripts@5.0.1 --save-dev` in the `ui` directory, then `npm install` again.
- **Could not find a required file. Name: index.js:**
  - Ensure `ui/src/index.tsx` exists. If not, create it with the correct React entry point code.
- **npm error enoent Could not read package.json:**
  - Make sure you are in the correct directory (`reStrike_VTA_Cursor`), not the parent folder.
- **TypeScript/JSX errors:**
  - Run `npm install --save-dev @types/react @types/react-dom` in the `ui` directory.

## Usage
1. Start the development server:
   ```bash
   npm run start
   ```
2. Run tests:
   ```bash
   npm test
   ```

## Contributing
1. Fork the repo and create your branch.
2. Submit a pull request with a clear description.
3. Follow the issue templates for bug reports and feature requests.

## License
MIT
# reStrike_VTA