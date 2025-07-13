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
- **Node.js:** v18+
- **Rust:** Stable (install via [rustup.rs](https://rustup.rs/))
- **Tauri CLI:** Install with `cargo install tauri-cli`
- **Frontend:** React 18 + TypeScript + Zustand + Tailwind CSS + framer-motion
- **Bundler:** Tauri (for native desktop app)
- **Linting:** ESLint (with TypeScript and React plugin)

## Quick Start
1. **Clone the repository:**
   ```bash
   git clone https://github.com/reStrike-d-o-o/reStrike_VTA
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