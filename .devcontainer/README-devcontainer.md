# Dev Container Usage Guide for reStrike VTA

## Getting Started

1. **Open in VS Code** (or compatible editor):
   - Use the "Reopen in Container" command if prompted, or open the Command Palette and select "Dev Containers: Reopen in Container".

2. **Dependencies**
   - All required tools (Node 18, Rust stable, Tauri CLI) are installed automatically.
   - Node and Rust dependencies for both backend and frontend are installed on first container build.

3. **Running the Frontend (React UI)**
   - Open a terminal in `/ui`:
     ```bash
     cd ui
     npm run start
     ```
   - The React app will be available on [http://localhost:3000](http://localhost:3000) (port is forwarded).

4. **Running the Backend (Tauri/Rust)**
   - From the project root:
     ```bash
     npm run start
     ```
   - This will start the Tauri backend (port 1420 is forwarded by default).

5. **Hot Reloading**
   - Both frontend and backend support hot reloading. Edit files and see changes live.

6. **Extra Tools**
   - If you need additional tools (e.g., ffmpeg, mpv), add them to a `.devcontainer/Dockerfile` and rebuild the container.

## Troubleshooting
- If you see missing dependencies, rerun `npm install` in both root and `ui/`.
- For Rust issues, run `cargo check` or `cargo build` in the root.

## Notes
- All ports and dependencies are managed by the container for a consistent dev experience.
- For more, see the main `README.md`. 