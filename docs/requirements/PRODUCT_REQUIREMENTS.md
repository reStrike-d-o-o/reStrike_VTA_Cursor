# Instant Video Replay Manager & AI Data Analyzer – Product Requirements Document

> **Note:**
> The finalized stack is: **Tauri (Rust backend), React 18 (TypeScript), Zustand (state), Tailwind CSS (styling), framer-motion (animation)**. Development is primarily on Windows for best compatibility with OBS and mpv. Some features may not work on Mac/Linux.

## 1. Elevator Pitch

A cross-platform Instant Video Replay Manager designed for taekwondo referees, enabling rapid video review and AI-assisted data analysis. The app integrates with taekwondo PSS systems via UDP, controls OBS Studio over WebSocket, and manages local video playback using mpv. With built-in automation, an intuitive manual mode, AI-ready architecture, and a licensing system tied to local hardware, it empowers referees to resolve match challenges within seconds.

## 2. Who is this app for

- **Primary Users**: Taekwondo referees during live competitions.
- **Secondary Users**: Tournament organizers and technical assistants.
- **Usage Context**: Fast-paced match environments where decisions must be made within 20 seconds or less.

## 3. Functional Requirements

### Core Features

- **UDP Server**
  - Listens to and parses datagrams from PSS systems.
  - Configurable network interface (LAN/WiFi/Any).
  - IPv4-only binding with custom bind address support.
  - Human-readable protocol definition file (editable via UI).

- **OBS Studio Control**
  - WebSocket control for multiple OBS instances.
  - Commands for scene switching and buffer-based recording (clip extraction).
  - Automatically generate 20-second video from buffer on event trigger, jump to 10th second on playback.

- **Video Playback**
  - Launch clips in `mpv` player.
  - UI hides during playback and auto-restores after player exit.

- **Recording Archive**
  - Recordings auto-named using match metadata (match number, athlete names, weight class, etc.).
  - Table of all stored recordings shown on main window.
  - Filter/search by match, player, or event.
  - Visual timeline per recording showing major events parsed from UDP data.
  - Clicking an event segment plays the related clip.

- **Manual Mode**
  - Bypass UDP to directly control OBS and playback via UI.

- **Local Storage**
  - SQLite DB stores parsed UDP events, metadata, and settings.
  - Recording metadata and file paths stored for quick access and filtering.

- **AI Data Analyzer**
  - Parses and interprets structured event data from UDP (e.g. strikes, penalties).
  - Visual annotations on timeline to help referees understand event context.
  - Future upgrade path to support video-based AI review.

- **Licensing System**
  - One-time online activation tied to hardware.
  - Optional online license check at app startup.
  - Prevents unauthorized use and enables subscription enforcement.

- **Logging System**
  - All logs are now saved in a dedicated 'log' folder in the installation directory. The folder is created automatically if missing. Log file is 'log/backend.log'.

## 4. User Stories

- **As a referee**, I want the most recent recording to play instantly when a challenge is raised so I can review the incident.
- **As a referee**, I want a searchable list of previous clips organized by match so I can manually verify past challenges.
- **As a referee**, I want visual timelines on each recording showing significant moments so I can quickly jump to the relevant part.
- **As a referee**, I want a simple interface with minimal input to avoid distractions during matches.
- **As a technical assistant**, I want to configure protocol formats and logging from a centralized settings screen.
- **As an organizer**, I want the license to be secure and tied to the machine, while being verified online.

## 5. User Interface

### Framework Recommendation
- **Tauri + React** for native performance, low overhead, and modern cross-platform GUI.
- Alternatively, **Electron + Svelte** for faster UI prototyping and third-party integration flexibility.

### Main Window

- **Top Bar**: Status indicators (UDP connection, OBS sync, license, etc.)
- **Live Control Panel**: Buttons for manual scene switch, replay, mark event, etc.
- **Recent Recording Display**: Auto-open most recent challenge video with overlay.
- **Recording Archive Table**:
  - Columns: Timestamp, Match #, Players, Category, Event Tags
  - Click to open video at the relevant moment.
- **Event Timeline Visualizer**:
  - Horizontal timeline with colored markers for major UDP-tagged events (e.g. hit, fall, timeout).
  - Hover for tooltip, click to play.

### Other Windows

- **Settings**
  - Network Interface
  - Protocol file upload
  - OBS credentials
  - Log directory
  - mpv path
  - License status

- **License Activation**
  - Input key, online verification, status display.
  - Periodic check (optional).

### Visual Style

- **Dark Theme**: Default, high contrast for indoor arenas.
- **Touch Friendly**: Large UI elements for fast input.
- **Animation**: Pulsating record button, smooth tab transitions.
- **Hide-on-playback**: Main window hides when `mpv` launches.

## AdvancedPanel (Frontend)
The AdvancedPanel is now a drawer-based UI:
- Left sidebar with icons for: PSS, OBS, Video, AI Analyzer, Settings
- Right content area for selected drawer
- Each drawer is a modular section for advanced features
- Diagnostics & Logs Manager: Logging and Download Logs are side by side above Live Data, which is scrollable and auto-scrolls as data arrives. Download Logs uses a table for log files; double-clicking a row downloads the file. Dropdown filters the table.
- Backend wiring for Diagnostics & Logs Manager is in progress: Tauri commands for logging toggles, log file listing, log file download, and live data streaming. See todo list for progress.

See [FRONTEND_DEVELOPMENT_SUMMARY.md] and [ui-design-document.md] for implementation details.
