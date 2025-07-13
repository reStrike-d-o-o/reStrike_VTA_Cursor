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