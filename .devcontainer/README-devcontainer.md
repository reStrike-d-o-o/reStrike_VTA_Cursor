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

---

# ✅ Dev Container Project Verification Checklist

**Run these commands in the Dev Container terminal (not your host OS).**

---

## 1. Check Core Tools

```bash
# Node.js
node --version

# Rust
rustc --version

# Cargo
cargo --version

# Tauri CLI
tauri --version

# mpv (media player)
mpv --version
```
_All should print a version number. If any are missing, see troubleshooting below._

---

## 2. Check Dependency Installation

```bash
# At project root
npm install

# In the ui/ directory
cd ui
npm install
cd ..
```

---

## 3. Check for Vulnerabilities

```bash
# At project root
npm audit

# In the ui/ directory
cd ui
npm audit
cd ..
```

---

## 4. Start the React Frontend

```bash
cd ui
npm run start
# Visit http://localhost:3000 in your browser (should show your React app)
cd ..
```

---

## 5. Start the Tauri Backend

```bash
npm run start
# (If you have a Tauri backend entry point)
```

---

## 6. Test mpv Playback

```bash
mpv --version
# (Or try playing a sample file if you have one)
```

---

## 7. Optional: Check Rust Build

```bash
cargo build
```

---

## 8. Review and Update Documentation

- Open `.devcontainer/README-devcontainer.md` and `README.md`
- Ensure instructions match your working setup.

---

## Troubleshooting

- If any tool is missing, try rebuilding the container:  
  **VS Code Command Palette → “Dev Containers: Rebuild and Reopen in Container”**
- If `tauri` is missing, run:  
  `cargo install tauri-cli`
- If `mpv` is missing, check your `.devcontainer/Dockerfile` and rebuild the container.

---

**You can copy-paste this checklist into a Markdown file in your repo (e.g., `DEV-CONTAINER-CHECKLIST.md`) for future reference.**

Let me know if you want this checklist added to your repo, or if you want a script to automate any of these steps! 