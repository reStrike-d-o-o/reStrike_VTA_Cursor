# reStrike VTA Documentation

Welcome to the reStrike VTA project documentation. This directory contains comprehensive documentation organized by category for easy navigation and maintenance.

## 📚 Documentation Categories

### 🚀 [Development](./development/)
Development guides, checklists, and environment management documentation.

- **[Container Restart Guide](./development/container-restart.md)** - How to restart the dev container for framework updates
- **[Development Management](./development/development-management.md)** - Tools and scripts for managing the development environment
- **[Framework Updates](./development/framework-updates.md)** - Procedures for updating Node.js, mpv, and other frameworks
- **[Port Forwarding](./development/port-forwarding.md)** - Port configuration and management
- **[Framework Update Summary](./development/framework-update-summary.md)** - Summary of recent framework updates
- **[Checklists](./development/checklists/)** - Development verification checklists

### 📋 [Project Management](./project/)
Project tracking, management, and coordination documentation.

- **[Project Tracker Guide](./project/project-tracker-guide.md)** - Comprehensive guide to using the project tracker
- **[Project Management Summary](./project/project-management-summary.md)** - Overview of the project management system
- **[Tracker Quick Reference](./project/tracker-quick-reference.md)** - Quick reference for common tracker commands

### 📖 [Requirements](./requirements/)
Product requirements, specifications, and design documents.

- **[Instant Video Replay PRD](./requirements/instant-video-replay-prd.md)** - Product Requirements Document
- **[Software Requirements](./requirements/software-requirements.md)** - Technical requirements specification
- **[UI Design Document](./requirements/ui-design-document.md)** - User interface design specifications

### 🔌 [Integration](./integration/)
Integration guides for external systems and APIs.

- **[OBS Dual Protocol](./integration/obs-dual-protocol.md)** - OBS WebSocket v4 and v5 implementation guide
- **[OBS WebSocket Config](./integration/obs-websocket-config.md)** - OBS WebSocket configuration guide

### 🔧 [API Documentation](./api/)
API references and protocol specifications.

- **[OBS WebSocket API](./api/obs-websocket.md)** - Complete OBS WebSocket protocol reference
- **[UDP Protocol](./api/udp-protocol.md)** - UDP protocol specification (coming soon)

### 🏗️ [Project Structure](./PROJECT_STRUCTURE.md)
Comprehensive guide to the project's organized file structure and organization principles.

## 🚀 Quick Start

### For New Developers
1. Start with **[Project Structure](./PROJECT_STRUCTURE.md)** to understand the codebase organization
2. Review **[Development Management](./development/development-management.md)** for environment setup
3. Check **[Development Checklists](./development/checklists/)** for verification steps

### For Project Management
1. Read **[Project Tracker Guide](./project/project-tracker-guide.md)** for issue management
2. Use **[Tracker Quick Reference](./project/tracker-quick-reference.md)** for common commands
3. Review **[Project Management Summary](./project/project-management-summary.md)** for system overview

### For Integration Work
1. Start with **[OBS Dual Protocol](./integration/obs-dual-protocol.md)** for OBS integration
2. Reference **[OBS WebSocket API](./api/obs-websocket.md)** for protocol details
3. Follow **[OBS WebSocket Config](./integration/obs-websocket-config.md)** for setup

## 📝 Documentation Standards

### File Naming
- **Guides**: `kebab-case.md` (e.g., `container-restart.md`)
- **References**: `kebab-case.md` (e.g., `obs-websocket.md`)
- **Specifications**: `kebab-case.md` (e.g., `software-requirements.md`)

### Cross-References
Use relative paths for cross-references:
```markdown
See [Container Restart Guide](./development/container-restart.md)
See [OBS Configuration](./integration/obs-websocket-config.md)
```

### Maintenance
- Update this index when adding new documentation
- Keep cross-references current when moving files
- Review and update monthly

## 🔄 Recent Updates

- **2025-01-27**: Reorganized documentation structure for better navigation
- **2025-01-27**: Created comprehensive project structure guide
- **2025-01-27**: Moved all documentation to categorized directories
- **2025-01-27**: Updated all cross-references and navigation links

## 📞 Getting Help

- **Documentation Issues**: Create an issue with the `documentation` label
- **Missing Information**: Check if it's covered in the requirements or integration guides
- **Structure Questions**: Refer to the [Project Structure Guide](./PROJECT_STRUCTURE.md)

---

**📝 Note**: This documentation is maintained by the development team and should be updated whenever the project structure or processes change.

**🔄 Last Updated**: $(date)
**👤 Maintained by**: Development Team 