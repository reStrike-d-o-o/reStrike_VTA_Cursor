# User Interface Design Document

## Frontend Setup
- **Navigate to the UI directory:**
  ```bash
  cd reStrike_VTA_Cursor/ui
  npm install
  npm run start
  ```
- **First Launch:**
  - The app will show a blank page with a heading (`reStrike VTA Overlay`) until you add more UI components.
  - Ensure `react-scripts` is installed at version 5.0.1 for React 18 compatibility.
  - If you see errors about missing types, install `@types/react` and `@types/react-dom` as dev dependencies.

## Layout Structure

- **Docked Sidebar Mode**
  - App docks to **left or right edge** of the screen
  - Occupies **100% of screen height**
  - **Dynamic Width:**
    - **Collapsed:** 150x150px floating panel
    - **Expanded:** ~30-40% of screen width - not limited, if needed can go full screen

- **Expanded View**
  - Top: Live Control Buttons (grid layout)
  - Middle: Status Bar (match info, recording indicator)
  - Bottom: **Recording List Panel**
    - Scrollable
    - Most recent recording is always at the top
    - Auto-selected and visually highlighted
    - **Flag Display**: Country flags shown next to athlete names

- **Collapsed View**
  - Small square (150x150 px) at docked edge
  - Shows only a **blinking red record button** and minimal status icons
  - Clicking expands the full UI

- **Video Player Launch**
  - External video player (`mpv`) launched via CMD/PowerShell
  - Main UI auto-minimizes when playback is triggered

## Core Components

- **Red Blinking Record Button**
  - Visible in both collapsed and expanded states
  - Pulsates based on OBS recording status (via WebSocket)
  - Large and central in collapsed view

- **Big Action Buttons**
  - "Play Replay", "Manual Mode", "Mark Event", etc.
  - Clearly labeled and touch-friendly

- **Status Panel**
  - Match metadata: Match #, athletes, decision timer
  - Connection and license status indicators

- **Recording List**
  - Scrollable list of previously saved recordings
  - Each item includes:
    - Timestamp
    - Match #
    - Players or event summary
    - **Country flags** for athletes (IOC codes)
  - **Newest entry always at the top, selected, and visually distinct**
  - Clicking an item launches the corresponding video via `mpv`

- **Flag Management Components**
  - **Flag Display**: Country flags shown as small icons next to athlete names
  - **Flag Fallback**: Emoji flags displayed when image files are unavailable
  - **Flag Utility**: Centralized flag management with IOC code support
  - **Flag Download**: Automated IOC flag download and management system

## Interaction Patterns

- **Docking & Visibility**
  - Stays locked to left or right side of screen
  - Always-on-top window layer
  - Automatically hides to collapsed view when inactive

- **Click to Expand**
  - Click the 150x150 px panel to expand full UI
  - Auto-collapse after playback if no interaction for 20s (configurable)

- **Auto-Scroll**
  - Recording list auto-scrolls to the top on new save
  - Newest recording is automatically focused and highlighted

- **External Playback**
  - CMD or PowerShell executes `mpv` with `--start=10` and video path
  - UI hides during video playback

- **Global Shortcut Listener**
  - App registers system-wide hotkeys
  - Works even when minimized or collapsed
  - Designed for use with StreamDeck or similar devices
  - Mappable keys via Settings UI (e.g. Ctrl+Alt+1 = Start Replay)

- **Flag Display**
  - Flags automatically load and display based on IOC codes
  - Fallback to emoji flags if image files are missing
  - Optimized loading for performance with 253+ flag images

## Visual Design Elements & Color Scheme

- **Dark Theme**
  - Optimized for arenas and low-light environments
- **Color Palette**
  - Red – Recording, alerts
  - Blue – Navigation, playback
  - Green – Connected/Ready
  - Grey – Inactive items

- **Animations**
  - Blinking record button
  - Smooth transitions for expansion/collapse
  - Highlight fade on new recording entry
  - Flag loading animations

## Mobile, Web App, Desktop Considerations

- **Platform:** Desktop only (Windows, Mac, Linux)
- **Display Mode:**
  - Designed for laptops used at ringside
  - App never obscures full-screen OBS feed
  - Functions like a control dock rather than a main window

## Typography

- **Font:** Segoe UI, Roboto, or system sans-serif
- **Sizes:**
  - Large Buttons: 20pt
  - Status Info: 14–16pt
  - Recording List: 14pt, bold highlight for selected

- **Visual Cues:**
  - Consistent icon + text usage
  - High-contrast backgrounds for readability
  - Flag icons sized appropriately for UI context

## Accessibility

- **Collapsed/Expanded Toggle Keyboard Shortcut**
  - Optional hotkey (e.g. Ctrl+Space) for fast toggle
- **StreamDeck Integration**
  - Fully compatible with macro keyboards
  - Mappable actions via Settings
- **Large Click Zones**
  - All elements minimum 44x44 px
- **Screen Reader-Friendly Labels**
  - Accessible labeling for all buttons and list items
  - Flag alt-text includes country names
- **Color-Blind Friendly Mode**
  - High-contrast toggle available in Settings
  - Flag fallbacks ensure visibility for all users

### Atomic Atoms (2024)
- Button
- Input
- Checkbox
- Label
- StatusDot (Badge)
- Icon

All status indicators and icons are now atomic. Accessibility linter issues have been addressed as of this update.

## Atomic Component Structure & Workflow (2024)

- All UI components are now organized by atomic design (atoms, molecules, organisms, layouts).
- All moves/refactors require copying the original file before deletion.
- The workflow for atomic reorganization: audit, copy, move, update imports, test, delete originals.
- See PROJECT_STRUCTURE.md and FRONTEND_DEVELOPMENT_SUMMARY.md for more.

## Diagnostics & Logs Manager

- Logging and Download Logs are displayed side by side (horizontal layout) above the Live Data section.
- Download Logs uses a table for log files; double-clicking a row downloads the file. The dropdown filters the table.
- All logs are now saved in a dedicated 'log' folder in the installation directory. The folder is created automatically if missing. Log file is 'log/backend.log'.
- Live Data is below, with a scrollable, auto-scrolling rich text area for live stream data.
- All sections follow atomic design and are ready for backend integration.

**Atomic Structure:**
- Atoms: Toggle, Dropdown, Button, ListItem, RichTextArea, Label, Icon.
- Molecules: LogToggleGroup, LogDownloadList, LiveDataPanel.
- Organism: DiagnosticsAndLogsManager (contains all three groups).

**Backend Wiring Plan:**
- Logging toggles will dispatch actions to enable/disable logging in the backend via Tauri commands.
- Download group will fetch log file lists and download files from the backend.
- LIVE DATA group will subscribe to live data streams from the backend, filtered by type.

For now, the UI uses dummy data and stub handlers, ready for backend integration.
