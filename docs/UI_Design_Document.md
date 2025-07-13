# User Interface Design Document

## Layout Structure

- **Docked Sidebar Mode**
  - App docks to **left or right edge** of the screen
  - Occupies **100% of screen height**
  - Width dynamically changes based on mode:
    - **Collapsed:** 150x150 px floating panel
    - **Expanded:** ~30–40% of screen width for full recording list view

- **Expanded View**
  - Top: Live Control Buttons (grid layout)
  - Middle: Status Bar (match info, recording indicator)
  - Bottom: **Recording List Panel**
    - Scrollable
    - Most recent recording is always at the top
    - Auto-selected and visually highlighted

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
  - **Newest entry always at the top, selected, and visually distinct**
  - Clicking an item launches the corresponding video via `mpv`

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
- **Color-Blind Friendly Mode**
  - High-contrast toggle available in Settings
