# reStrike VTA – Project Guide

Welcome! This guide gives new contributors a quick tour of the codebase areas you are likely to touch most often, plus notes on the workflows that were recently added.

## Top-Level Layout

| Path | Purpose |
| ---- | ------- |
| `src-tauri/` | Rust backend (Tokio + Tauri 2). Commands are registered in `src-tauri/src/tauri_commands.rs`. |
| `ui/` | React + TypeScript frontend (strict mode). Components live in `ui/src/components/**`. |
| `scripts/` | Migration helpers, tournament tooling, and various utilities. |
| `docs/` | Architecture notes, migration plans, and this guide. |

SQLite is used for storage (see `src-tauri/src/database/**`); networking is primarily UDP plus Tauri events/IPC.

## Building, Running, Testing

```bash
# Frontend
cd ui
pnpm install
pnpm build      # build React bundle
pnpm lint       # ESLint/Prettier check

# Backend
cd ../src-tauri
cargo build --release
cargo test
```

Documentation bundles can be regenerated with `pnpm docs:all` from the repository root (uses the scripts defined in `ui/package.json`).

## Tournament Import Workflow

The legacy Python import scripts were ported to Rust and exposed as a Tauri command:

| Item | Location |
| ---- | -------- |
| Command | `tournament_import_from_directory` registered in `src-tauri/src/tauri_commands.rs` |
| Rust entry point | `src-tauri/src/importers/daedo.rs` |
| UI trigger | `ui/src/components/molecules/TournamentManagementPanel.tsx` (`openImportDialog` → `invoke('tournament_import_from_directory')`) |

Usage from the app:
1. Open **Tournament Manager → Import Tournament**.
2. Pick a Daedo-style folder (same shape as `C:\Users\Damjan\Documents\Daedo log files GO2025`).  
3. Enter a tournament name when prompted. The importer rebuilds tournament days, matches, athletes, and events directly into SQLite.

Athlete matching rules favour `wtid`, with fallbacks on name + gender. History entries are appended when metadata differs.

## Medal Ceremony UI Refresh

The medal ceremony tooling now has a clearer layout and animation preview workflow.

| Feature | UI Component |
| ------- | ------------- |
| Ceremony playlist / settings panels | `ui/src/components/ovr/MedalCeremonyPanel.tsx` |
| Animation asset manager & preview modal | `ui/src/components/ovr/MedalCeremonyAnimationManager.tsx` |

Notable behaviour:

* **Ceremony playlist column** spans the full height of the page so operators can reorder divisions without scrolling.
* **Settings panel** contains the background/break placeholders, duration controls, and a real month-view calendar for scheduling.
* **Ceremony playlist items panel** sits directly to the right and stretches to the viewport edge.
* **Animation preview modal** keeps flag animations at a fixed 450 px width while shrinking surrounding chrome so the close button and header never overlap the animation.

## Anthem Asset Downloader

Use `scripts/download_national_anthems.py` when refreshing anthem audio:

* The downloader requests each country’s `nationalanthems.info` page, parses the list of MP3 variants in the order shown on the site, and downloads every file.
* Each variant is saved as `<ioc>-vN.mp3` (e.g. `cro-v1.mp3`, `cro-v2.mp3`), preserving historical anthems.
* The newest entry on the site is copied to the canonical uppercase filename (`CRO.mp3`) so the app always plays the current anthem.
* Failed downloads and missing IOC mappings are recorded in `failed_links.md`.

## Logging & Conventions

* Prefer `log::info!`, `log::warn!`, `log::error!` in Rust; they are routed through the custom `env_logger` formatter.
* Keep React components functional with memoization hooks (`useMemo`, `useCallback`) for expensive computations.
* Stick to prepared statements / transactions in SQLite (see `src-tauri/src/database/operations.rs` examples).

## Where to Look Next

* `docs/DOCUMENTATION_INDEX.md` – catalogue of deeper design documents.
* `docs/architecture/**` – diagrams and flow charts generated from real schemas (`docs/api/**`, `docs/diagrams/**`).

Happy hacking! If you add features, drop a short note here so the next contributor can follow the breadcrumbs. :)
