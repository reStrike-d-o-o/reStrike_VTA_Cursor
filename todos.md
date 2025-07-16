# reStrike VTA - Master TODO List

## Phase 4: UI Organization (Atomic Design & Layout)

### 4.1 Atomic Design Implementation
- [ ] Create `atoms/`, `molecules/`, `organisms/`, and `layouts/` folders in `ui/src/components/`
- [ ] Scaffold `Button`, `Icon`, `StatusIndicator` components in `atoms/`
- [ ] Scaffold `SidebarFilter`, `NavigationItem` in `molecules/`
- [ ] Scaffold `Sidebar`, `Overlay`, `Settings`, `ObsWebSocketManager`, `VideoClips` in `organisms/`
- [ ] Scaffold layout components: `DockBar`, `TaskBar`, `AdvancedPanel`, `StatusbarDock`, `StatusbarAdvanced` in `layouts/`
- [ ] Refactor existing components into new structure
- [ ] Implement component composition patterns

### 4.2 Component Library
- [ ] Implement and document each atomic/molecular/organism component
- [ ] Create reusable atomic components (Button, Icon, etc.)
- [ ] Create SidebarFilter, NavigationItem, Sidebar, etc.

### 4.3 Layout System
- [ ] Implement DockBar layout (SidebarSmall + SidebarBig)
- [ ] Implement TaskBar layout (header with controls)
- [ ] Implement AdvancedPanel layout (main content area)
- [ ] Implement StatusbarDock (bottom of DockBar)
- [ ] Implement StatusbarAdvanced (bottom of AdvancedPanel)
- [ ] Wire up navigation logic for ADVANCED PANEL content switching
- [ ] Ensure DOCK BAR and status bars are always visible
- [ ] Embed Overlay within AdvancedPanel (not floating)

## Phase 5: Deprecation Warning Fixes & Modernization

### 5.1 React Scripts Migration
- [ ] Migrate to Vite (remove fs.F_OK deprecation warning)
- [ ] Update build configuration for Vite
- [ ] Test all functionality with Vite
- [ ] Update documentation for Vite usage

### 5.2 Dependency Updates
- [ ] Update all dependencies to latest versions
- [ ] Remove deprecated packages
- [ ] Fix any new deprecation warnings
- [ ] Test compatibility with updated dependencies

## General Maintenance & Performance
- [ ] Review all new and existing files monthly for compliance with structure, naming, maintenance, and performance conventions (see .cursor/rules/context.mdc)
- [ ] Update navigation indexes (docs/README.md, scripts/README.md) and cross-references after any file or directory changes
- [ ] Monitor and optimize frontend/backend performance using best practices. Clean caches and build artifacts weekly
- [ ] Ensure all onboarding, build, and documentation references point to .cursor/rules/context.mdc after any major change 