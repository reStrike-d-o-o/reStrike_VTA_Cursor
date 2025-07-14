#!/usr/bin/env python3
"""
GitHub Issues Creator for reStrike VTA Project
Automatically creates GitHub issues from project documentation tasks and roadmap items.
"""

import json
import os
import sys
import requests
from typing import Dict, List, Any
from datetime import datetime

class GitHubIssuesCreator:
    def __init__(self, repo_owner: str, repo_name: str, token: str):
        self.repo_owner = repo_owner
        self.repo_name = repo_name
        self.token = token
        self.api_base = "https://api.github.com"
        self.headers = {
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github.v3+json",
            "X-GitHub-Api-Version": "2022-11-28"
        }

    def create_issue(self, title: str, body: str, labels: List[str] = None, assignees: List[str] = None) -> Dict[str, Any]:
        """Create a GitHub issue"""
        url = f"{self.api_base}/repos/{self.repo_owner}/{self.repo_name}/issues"
        
        data = {
            "title": title,
            "body": body,
            "labels": labels or [],
            "assignees": assignees or []
        }
        
        response = requests.post(url, headers=self.headers, json=data)
        
        if response.status_code == 201:
            return response.json()
        else:
            print(f"❌ Failed to create issue '{title}': {response.status_code} - {response.text}")
            return None

    def create_immediate_priority_tasks(self) -> List[Dict[str, Any]]:
        """Create immediate priority tasks (2% remaining for production)"""
        issues = []
        
        # Core System Testing Phase (Immediate Priority)
        testing_tasks = [
            {
                "title": "[TASK] Complete Frontend Testing - React Components Verification",
                "body": """## 🎯 Task Description
Verify functionality of all 5 React components (1,691 lines total) in the frontend.

## ✅ Acceptance Criteria
- [ ] VideoClips.tsx (315 lines) - Clip management functionality tested
- [ ] Settings.tsx (402 lines) - Configuration interface tested  
- [ ] Overlay.tsx (306 lines) - Video overlay system tested
- [ ] ObsWebSocketManager.tsx - OBS connection management tested
- [ ] App.tsx (268 lines) - Main application navigation tested
- [ ] Keyboard shortcuts (Ctrl+1-5, Space, F11) tested
- [ ] State management (Zustand) operations tested
- [ ] Error handling and loading states validated

## 🔗 Related Components
- [x] Frontend (React)
- [ ] Testing

## 📊 Priority & Effort
- **Priority**: High (Critical path)
- **Effort**: Medium (4-16 hours)

## 📝 Technical Notes
Frontend is currently running successfully on port 3000. All components are implemented and need comprehensive functionality testing.

## ✅ Definition of Done
- [ ] All React components tested and functional
- [ ] Keyboard shortcuts working correctly
- [ ] State management operations verified
- [ ] Error handling tested
- [ ] Documentation updated with test results
""",
                "labels": ["task", "frontend", "testing", "priority:high", "effort:medium", "phase:testing"]
            },
            {
                "title": "[TASK] Complete Backend Testing - Rust Plugins Validation",
                "body": """## 🎯 Task Description
Validate functionality of all 3 core Rust plugins (1,663 lines total) in the backend.

## ✅ Acceptance Criteria
- [ ] plugin_udp.rs (640 lines) - PSS protocol message parsing verified
- [ ] plugin_obs.rs (455 lines) - OBS WebSocket v4/v5 connections tested
- [ ] plugin_playback.rs (568 lines) - mpv video integration tested
- [ ] Tauri command system validation
- [ ] Event system real-time processing tested
- [ ] Error handling from Rust to frontend verified

## 🔗 Related Components
- [x] Backend (Rust)
- [ ] Testing

## 📊 Priority & Effort
- **Priority**: High (Critical path)
- **Effort**: Medium (4-16 hours)

## 📝 Technical Notes
Backend compiles successfully with zero errors. All core plugins implemented and need comprehensive validation testing.

## ✅ Definition of Done
- [ ] All Rust plugins tested and operational
- [ ] Protocol parsing verified
- [ ] OBS connections tested
- [ ] Video integration confirmed
- [ ] Error handling validated
""",
                "labels": ["task", "backend", "testing", "priority:high", "effort:medium", "phase:testing"]
            },
            {
                "title": "[TASK] Integration Testing - Frontend ↔ Backend Communication",
                "body": """## 🎯 Task Description
Test integration between React frontend and Rust backend via Tauri framework.

## ✅ Acceptance Criteria
- [ ] Frontend-backend communication via Tauri tested
- [ ] Video clip playback through React → Tauri → mpv chain verified
- [ ] OBS connection management through React interface tested
- [ ] Real-time PSS data processing and display validated
- [ ] State synchronization between frontend and backend confirmed

## 🔗 Related Components
- [x] Frontend (React)
- [x] Backend (Rust)
- [ ] Testing

## 📊 Priority & Effort
- **Priority**: High (Critical path)
- **Effort**: Medium (4-16 hours)

## 📝 Technical Notes
Tauri framework configured for Windows desktop application. Integration points need comprehensive testing.

## ✅ Definition of Done
- [ ] Full integration chain tested
- [ ] Real-time data flow verified
- [ ] Error scenarios tested
- [ ] Performance validated
""",
                "labels": ["task", "integration", "testing", "priority:high", "effort:medium", "phase:testing"]
            }
        ]
        
        # Production Deployment Tasks
        production_tasks = [
            {
                "title": "[TASK] Generate Windows Executable - Production Build",
                "body": """## 🎯 Task Description
Generate production Windows .exe executable using Tauri build system.

## ✅ Acceptance Criteria
- [ ] Windows .exe executable generated successfully
- [ ] All dependencies bundled correctly
- [ ] Build process documented
- [ ] Executable tested on clean Windows system

## 🔗 Related Components
- [x] Backend (Rust)
- [ ] CI/CD

## 📊 Priority & Effort
- **Priority**: High (Critical path)
- **Effort**: Small (< 4 hours)

## 📝 Technical Notes
Tauri configured for Windows 10/11 target. Build system ready for production packaging.

## ✅ Definition of Done
- [ ] Successful .exe generation
- [ ] All dependencies included
- [ ] Build documented
- [ ] Clean system testing
""",
                "labels": ["task", "ci/cd", "infrastructure", "priority:high", "effort:small", "phase:production"]
            },
            {
                "title": "[TASK] Create MSI Installer Package",
                "body": """## 🎯 Task Description
Create professional MSI installer package for Windows deployment.

## ✅ Acceptance Criteria
- [ ] MSI installer package created
- [ ] Installation process tested on clean Windows systems
- [ ] Uninstall process verified
- [ ] Installation documentation created

## 🔗 Related Components
- [ ] Infrastructure
- [ ] Documentation

## 📊 Priority & Effort
- **Priority**: High (Critical path)
- **Effort**: Small (< 4 hours)

## 📝 Technical Notes
MSI installer needed for professional Windows deployment. Should include all required dependencies.

## ✅ Definition of Done
- [ ] Working MSI installer created
- [ ] Installation tested
- [ ] Uninstall tested
- [ ] Documentation complete
""",
                "labels": ["task", "infrastructure", "priority:high", "effort:small", "phase:production"]
            }
        ]
        
        # Minor Implementation Gaps (2% remaining)
        implementation_gaps = [
            {
                "title": "[TASK] Complete SQLite Data Storage Plugin",
                "body": """## 🎯 Task Description
Complete implementation of plugin_store.rs for SQLite data storage.

## ✅ Acceptance Criteria
- [ ] SQLite plugin fully implemented (currently stub)
- [ ] Data persistence for video clips
- [ ] Settings storage and retrieval
- [ ] Database migration system
- [ ] Error handling for database operations

## 🔗 Related Components
- [x] Backend (Rust)
- [ ] Testing

## 📊 Priority & Effort
- **Priority**: Medium (Important)
- **Effort**: Medium (4-16 hours)

## 📝 Technical Notes
Currently plugin_store.rs is a basic stub. Need full SQLite implementation for data persistence.

## ✅ Definition of Done
- [ ] Full SQLite implementation
- [ ] Data persistence working
- [ ] Migration system in place
- [ ] Tests written and passing
""",
                "labels": ["task", "backend", "priority:medium", "effort:medium"]
            },
            {
                "title": "[TASK] Implement License Validation System",
                "body": """## 🎯 Task Description
Implement license validation system in plugin_license.rs if required.

## ✅ Acceptance Criteria
- [ ] License validation logic implemented
- [ ] Hardware-based licensing if needed
- [ ] License verification on startup
- [ ] Graceful handling of license issues

## 🔗 Related Components
- [x] Backend (Rust)
- [ ] Security

## 📊 Priority & Effort
- **Priority**: Low (Nice to have)
- **Effort**: Small (< 4 hours)

## 📝 Technical Notes
Currently plugin_license.rs is a basic stub. Determine if licensing is required for deployment.

## ✅ Definition of Done
- [ ] License system designed
- [ ] Implementation complete
- [ ] Testing performed
- [ ] Documentation updated
""",
                "labels": ["task", "backend", "security", "priority:low", "effort:small"]
            }
        ]
        
        return testing_tasks + production_tasks + implementation_gaps

    def create_future_enhancement_epics(self) -> List[Dict[str, Any]]:
        """Create Epic issues for future enhancement categories"""
        epics = []
        
        # Video System Enhancements Epic
        video_epic = {
            "title": "[EPIC] Video System Enhancements - Advanced Playback & Management",
            "body": """## Epic Overview
Comprehensive enhancement of the video system with advanced playback features, clip management, and multi-format support.

## Business Value
Provides professional-grade video analysis capabilities for taekwondo referees, enabling frame-by-frame analysis, slow motion, and advanced clip management.

## User Stories
- As a referee, I want slow motion controls so that I can analyze impacts in detail
- As a referee, I want frame-by-frame navigation so that I can see exact contact moments
- As a referee, I want multiple camera angles so that I can view incidents from different perspectives
- As a tournament organizer, I want automatic thumbnails so that clips are easily identifiable
- As a referee, I want video effects controls so that I can enhance visibility of contacts

## Acceptance Criteria
High-level criteria that define when this epic is complete:
- [ ] Advanced playback controls (slow motion, frame-by-frame) implemented
- [ ] Multi-format video support added
- [ ] Automatic thumbnail generation working
- [ ] Video effects and enhancement controls available
- [ ] Batch clip operations functional
- [ ] Cloud storage integration operational

## Related Tasks
Advanced Playback Features (20+ features):
- [ ] Slow Motion Controls: Variable speed playback (0.25x, 0.5x, 2x)
- [ ] Frame-by-Frame Navigation: Precise analysis capabilities  
- [ ] Multiple Angle Support: Synchronize multiple camera feeds
- [ ] Video Effects: Contrast, brightness, color adjustment
- [ ] Zoom and Pan: Digital zoom for detailed analysis
- [ ] Audio Controls: Volume mixing, audio tracks selection

Clip Management System (20+ features):
- [ ] Auto-Thumbnails: Generate video preview thumbnails
- [ ] Metadata Extraction: Automatic duration, resolution detection
- [ ] Batch Operations: Multi-select clip operations
- [ ] Export System: Export clips in various formats
- [ ] Cloud Storage: Integration with cloud storage services
- [ ] Backup System: Automatic clip backup and recovery

Advanced Video Features (20+ features):
- [ ] Multi-Format Support: Support for more video codecs
- [ ] Live Streaming: Direct streaming integration
- [ ] Video Compression: On-the-fly compression options
- [ ] Watermarking: Add competition watermarks to clips
- [ ] Video Filters: Apply real-time video filters
- [ ] Picture-in-Picture: Multiple video overlay support

## Technical Architecture
Enhance existing mpv integration with additional video processing capabilities. Implement multi-format support and advanced playback controls.

## Dependencies
- [ ] Current video playback system (plugin_playback.rs)
- [ ] React frontend video components
- [ ] Tauri framework for native integration

## Success Metrics
- Video analysis time reduced by 50%
- Support for 10+ video formats
- Frame-accurate navigation capability
- Multi-camera synchronization accuracy <10ms

## Timeline
- **Start Date**: After core testing completion
- **Target Completion**: 3-6 months
- **Milestones**:
  - [ ] Advanced playback controls (Month 1)
  - [ ] Multi-format support (Month 2)
  - [ ] Clip management enhancements (Month 3)

## Risk Assessment
- **Technical Complexity**: High video processing requirements
- **Performance Impact**: Need to maintain real-time performance
- **Hardware Dependencies**: Requires adequate GPU acceleration
""",
            "labels": ["epic", "video", "enhancement", "priority:medium"]
        }
        
        # OBS Studio Integration Epic  
        obs_epic = {
            "title": "[EPIC] OBS Studio Integration Enhancements - Professional Broadcasting",
            "body": """## Epic Overview
Advanced OBS Studio integration features for professional broadcasting, streaming, and recording capabilities.

## Business Value
Enables professional-grade broadcasting capabilities for taekwondo competitions with automated recording, scene management, and streaming features.

## User Stories
- As a broadcaster, I want automated recording on match events so that no important moments are missed
- As a technical director, I want scene templates so that setup is quick and consistent
- As a referee, I want instant replay graphics so that reviews are professional
- As a tournament organizer, I want multi-platform streaming so that audiences can watch anywhere
- As a commentator, I want audio mixing controls so that commentary is properly balanced

## Acceptance Criteria
High-level criteria that define when this epic is complete:
- [ ] Advanced OBS controls for scene and source management
- [ ] Automated recording and streaming features
- [ ] Professional graphics package integration
- [ ] Multi-platform streaming capabilities
- [ ] Audio mixing and commentary integration

## Related Tasks
Advanced OBS Controls (18+ features):
- [ ] Scene Templates: Predefined scene configurations
- [ ] Source Management: Add/remove/configure sources remotely
- [ ] Filter Controls: Real-time filter adjustment
- [ ] Transition Effects: Custom transition management
- [ ] Audio Mixing: Remote audio level control
- [ ] Hotkey Integration: Trigger OBS hotkeys remotely

Recording & Streaming Features (18+ features):
- [ ] Automatic Recording: Auto-record on match start
- [ ] Stream Health Monitoring: Bandwidth and quality monitoring
- [ ] Multi-Platform Streaming: Stream to multiple platforms
- [ ] Recording Presets: Quality presets for different uses
- [ ] Clip Auto-Export: Export replay buffer clips automatically
- [ ] Live Annotations: Add live text overlays during recording

Professional Broadcasting (18+ features):
- [ ] Multi-Camera Support: Switch between multiple cameras
- [ ] Graphics Package: Lower thirds, scoreboards, timers
- [ ] Sponsor Integration: Dynamic sponsor logo insertion
- [ ] Instant Replay Graphics: Professional replay overlays
- [ ] Commentary Integration: Audio commentary mixing
- [ ] Social Media Integration: Auto-post highlights to social media

## Technical Architecture
Extend existing OBS WebSocket integration (plugin_obs.rs) with advanced control capabilities and automated workflows.

## Dependencies
- [ ] Current OBS WebSocket implementation (v4/v5 support)
- [ ] ObsWebSocketManager React component
- [ ] OBS Studio with WebSocket plugin

## Success Metrics
- Automated recording coverage: 100% of matches
- Stream reliability: 99.9% uptime
- Scene switching time: <200ms
- Multi-platform streaming capability

## Timeline
- **Start Date**: After video system enhancements
- **Target Completion**: 6-9 months
- **Milestones**:
  - [ ] Advanced controls (Month 1)
  - [ ] Automated workflows (Month 2)
  - [ ] Professional graphics (Month 3)

## Risk Assessment
- **OBS Compatibility**: Need to maintain compatibility across OBS versions
- **Performance**: Real-time streaming requirements
- **Network Dependencies**: Reliable internet connection required
""",
            "labels": ["epic", "obs", "integration", "priority:medium"]
        }
        
        # PSS Protocol & Competition Integration Epic
        pss_epic = {
            "title": "[EPIC] PSS Protocol & Competition Integration - Real-time Analytics",
            "body": """## Epic Overview
Enhanced PSS protocol support with advanced competition management, real-time analytics, and tournament integration.

## Business Value
Provides comprehensive competition management with real-time analytics, automated reporting, and tournament administration capabilities.

## User Stories
- As a referee, I want real-time protocol validation so that data integrity is guaranteed
- As a tournament director, I want automated match reports so that administration is streamlined
- As a data analyst, I want performance metrics so that athlete development can be tracked
- As a competition organizer, I want tournament bracket integration so that event management is unified
- As a referee supervisor, I want audit logs so that decisions can be reviewed

## Acceptance Criteria
High-level criteria that define when this epic is complete:
- [ ] Enhanced protocol validation and debugging capabilities
- [ ] Comprehensive competition management features
- [ ] Real-time analytics and reporting dashboard
- [ ] Tournament bracket and scheduling integration
- [ ] Advanced data export and analysis tools

## Related Tasks
Enhanced Protocol Support (18+ features):
- [ ] Protocol Validation: Real-time protocol compliance checking
- [ ] Custom Events: Support for competition-specific events
- [ ] Data Logging: Comprehensive competition data logging
- [ ] Event Filtering: Configurable event filtering and processing
- [ ] Multi-Protocol Support: Support additional competition protocols
- [ ] Protocol Debugging: Real-time protocol message debugging

Competition Management (18+ features):
- [ ] Match Templates: Predefined match configurations
- [ ] Athlete Database: Comprehensive athlete information system
- [ ] Tournament Brackets: Tournament management integration
- [ ] Official Reports: Generate official match reports
- [ ] Statistics Tracking: Advanced match statistics
- [ ] Competition Scheduling: Integration with scheduling systems

Real-Time Analytics (18+ features):
- [ ] Live Statistics: Real-time match analytics dashboard
- [ ] Performance Metrics: Athlete performance tracking
- [ ] Predictive Analytics: Match outcome predictions
- [ ] Heat Maps: Visual representation of scoring patterns
- [ ] Trend Analysis: Long-term performance trends
- [ ] Data Export: Export analytics data for further analysis

## Technical Architecture
Extend existing PSS protocol implementation (plugin_udp.rs) with advanced parsing, validation, and analytics capabilities.

## Dependencies
- [ ] Current PSS protocol implementation (640 lines)
- [ ] UDP server functionality
- [ ] React analytics dashboard components

## Success Metrics
- Protocol parsing accuracy: 99.99%
- Real-time analytics latency: <100ms
- Data export completeness: 100%
- Competition administration time reduction: 75%

## Timeline
- **Start Date**: Parallel with OBS enhancements
- **Target Completion**: 6-12 months
- **Milestones**:
  - [ ] Enhanced protocol support (Month 2)
  - [ ] Competition management (Month 4)
  - [ ] Analytics dashboard (Month 6)

## Risk Assessment
- **Protocol Evolution**: PSS protocol may change over time
- **Data Volume**: Large competitions generate significant data
- **Real-time Requirements**: Sub-second response times required
""",
            "labels": ["epic", "udp", "protocol", "analytics", "priority:medium"]
        }
        
        # UI/UX Enhancement Epic
        ui_epic = {
            "title": "[EPIC] User Interface & Experience Enhancements - Modern Platform",
            "body": """## Epic Overview
Comprehensive UI/UX enhancements including modern themes, accessibility features, mobile companion, and advanced navigation.

## Business Value
Provides world-class user experience with accessibility compliance, multi-platform support, and advanced customization capabilities.

## User Stories
- As a referee, I want customizable themes so that the interface works in different lighting conditions
- As a visually impaired user, I want accessibility features so that the application is usable
- As a mobile user, I want a companion app so that I can control the system remotely
- As a referee, I want workspace management so that I can customize my setup
- As an international user, I want multi-language support so that I can use the app in my language

## Acceptance Criteria
High-level criteria that define when this epic is complete:
- [ ] Modern UI enhancements with customization options
- [ ] Comprehensive accessibility features
- [ ] Mobile companion application
- [ ] Advanced navigation and workspace management
- [ ] Multi-language support and localization

## Related Tasks
Modern UI Enhancements (18+ features):
- [ ] Custom Themes: Multiple color themes and customization
- [ ] Layout Customization: Drag-and-drop interface customization
- [ ] Accessibility Features: Screen reader support, high contrast
- [ ] Multi-Language Support: Internationalization (i18n)
- [ ] Mobile Companion: Mobile app for remote control
- [ ] Touch Interface: Touch-friendly controls for tablets

Advanced Navigation (18+ features):
- [ ] Workspace Management: Multiple workspace configurations
- [ ] Quick Actions: Customizable quick action buttons
- [ ] Search System: Global search across all features
- [ ] Recent Items: Quick access to recent clips and settings
- [ ] Favorites System: Bookmark frequently used features
- [ ] Context Menus: Right-click context menus throughout

Visualization & Analytics Dashboard (18+ features):
- [ ] Real-Time Dashboards: Live competition status dashboard
- [ ] Data Visualization: Charts and graphs for match data
- [ ] Performance Indicators: Key performance indicators (KPIs)
- [ ] Alert System: Visual and audio alerts for important events
- [ ] Status Monitoring: System health and performance monitoring
- [ ] Notification Center: Centralized notification management

## Technical Architecture
Enhance existing React frontend with advanced theming, accessibility libraries, and responsive design frameworks.

## Dependencies
- [ ] Current React frontend implementation
- [ ] Tailwind CSS styling system
- [ ] Framer Motion animation library

## Success Metrics
- Accessibility compliance: WCAG 2.1 AA standard
- Mobile companion app downloads: 1000+ users
- Theme customization usage: 80% of users
- Multi-language support: 5+ languages

## Timeline
- **Start Date**: After core system stability
- **Target Completion**: 9-12 months
- **Milestones**:
  - [ ] Theme and accessibility (Month 3)
  - [ ] Mobile companion (Month 6)
  - [ ] Advanced navigation (Month 9)

## Risk Assessment
- **Accessibility Compliance**: Complex requirements for full compliance
- **Mobile Platform Differences**: iOS and Android development complexity
- **Internationalization**: Translation and cultural considerations
""",
            "labels": ["epic", "ui", "ux", "accessibility", "priority:medium"]
        }
        
        # System & Performance Epic
        system_epic = {
            "title": "[EPIC] System & Performance Enhancements - Enterprise Features",
            "body": """## Epic Overview
Advanced system features including enterprise capabilities, performance optimization, and automation frameworks.

## Business Value
Enables enterprise deployment with multi-user support, advanced configuration, and automated workflows for large-scale tournament operations.

## User Stories
- As an enterprise admin, I want user authentication so that access can be controlled
- As a system administrator, I want performance monitoring so that issues can be prevented
- As a developer, I want API endpoints so that third-party integrations are possible
- As a tournament organizer, I want automation scripts so that repetitive tasks are eliminated
- As an IT manager, I want centralized management so that multiple installations can be controlled

## Acceptance Criteria
High-level criteria that define when this epic is complete:
- [ ] Advanced configuration and profile management
- [ ] Enterprise features with authentication and audit logging
- [ ] Performance optimization and monitoring
- [ ] Integration and automation frameworks
- [ ] API endpoints for third-party integration

## Related Tasks
Advanced Configuration (18+ features):
- [ ] Profile Management: Multiple user profiles and preferences
- [ ] Hardware Acceleration: GPU acceleration for video processing
- [ ] Network Optimization: Optimize for various network conditions
- [ ] Resource Management: Advanced memory and CPU management
- [ ] Plugin System: Third-party plugin support
- [ ] API Endpoints: REST API for external integrations

Enterprise Features (18+ features):
- [ ] User Authentication: Multi-user support with permissions
- [ ] Audit Logging: Comprehensive action logging
- [ ] Database Integration: Enterprise database connectivity
- [ ] Centralized Management: Central configuration management
- [ ] Backup & Recovery: Enterprise-grade backup solutions
- [ ] Monitoring & Alerts: System monitoring and alerting

Integration & Automation (18+ features):
- [ ] Third-Party APIs: Integration with competition management systems
- [ ] Automation Scripts: Scriptable automation for repetitive tasks
- [ ] Webhook Support: Real-time event notifications via webhooks
- [ ] Command Line Interface: CLI for advanced users and automation
- [ ] Scheduled Tasks: Automated maintenance and cleanup tasks
- [ ] Update System: Automatic update checking and installation

## Technical Architecture
Implement enterprise features using modern authentication, monitoring, and automation frameworks integrated with existing Rust/React architecture.

## Dependencies
- [ ] Current application architecture
- [ ] Database system (SQLite/PostgreSQL)
- [ ] Authentication frameworks

## Success Metrics
- Multi-user support: 100+ concurrent users
- System uptime: 99.99%
- API response time: <50ms
- Automation coverage: 90% of repetitive tasks

## Timeline
- **Start Date**: After UI/UX enhancements
- **Target Completion**: 12-18 months
- **Milestones**:
  - [ ] Advanced configuration (Month 3)
  - [ ] Enterprise features (Month 6)
  - [ ] Integration framework (Month 9)

## Risk Assessment
- **Security Complexity**: Enterprise security requirements
- **Performance Impact**: Additional features may impact performance
- **Integration Challenges**: Third-party system compatibility
""",
            "labels": ["epic", "system", "performance", "enterprise", "priority:low"]
        }
        
        # Modern Platform Features Epic
        platform_epic = {
            "title": "[EPIC] Modern Platform Features - Cloud & AI Integration",
            "body": """## Epic Overview
Next-generation platform features including cloud integration, AI/ML capabilities, and modern deployment options.

## Business Value
Positions the platform for future growth with cloud capabilities, artificial intelligence features, and modern deployment architectures.

## User Stories
- As a tournament organizer, I want cloud sync so that data is available across multiple devices
- As a referee, I want AI highlight detection so that important moments are automatically identified
- As a data analyst, I want ML-powered analytics so that insights are automatically generated
- As a mobile user, I want cloud access so that I can review matches remotely
- As a competition league, I want automated quality assessment so that video standards are maintained

## Acceptance Criteria
High-level criteria that define when this epic is complete:
- [ ] Cloud integration with sync and backup capabilities
- [ ] AI/ML features for automated analysis and insights
- [ ] Modern deployment and scaling options
- [ ] Mobile and web access platforms
- [ ] Advanced collaboration features

## Related Tasks
Cloud Integration (12+ features):
- [ ] Cloud Sync: Synchronize settings and clips across devices
- [ ] Remote Access: Access application remotely via web interface
- [ ] Collaboration: Multi-user collaboration features
- [ ] Cloud Analytics: Cloud-based analytics and reporting
- [ ] Backup Services: Cloud backup and restore capabilities
- [ ] License Management: Cloud-based license management

AI & Machine Learning (12+ features):
- [ ] Automatic Highlight Detection: AI-powered highlight identification
- [ ] Smart Clip Creation: Intelligent clip creation based on match events
- [ ] Predictive Analysis: AI-driven match analysis and predictions
- [ ] Performance Analytics: ML-based performance analysis
- [ ] Automated Tagging: Automatic clip tagging and categorization
- [ ] Quality Assessment: AI-based video quality assessment

## Technical Architecture
Implement cloud services integration and AI/ML frameworks alongside existing desktop application architecture.

## Dependencies
- [ ] Cloud infrastructure (AWS/Azure/GCP)
- [ ] AI/ML frameworks (TensorFlow/PyTorch)
- [ ] Web application framework

## Success Metrics
- Cloud sync reliability: 99.9%
- AI accuracy: >95% for highlight detection
- Remote access performance: <200ms latency
- Mobile app adoption: 50% of desktop users

## Timeline
- **Start Date**: After system enhancements
- **Target Completion**: 18-24 months
- **Milestones**:
  - [ ] Cloud integration (Month 6)
  - [ ] AI/ML features (Month 12)
  - [ ] Mobile platform (Month 18)

## Risk Assessment
- **Cloud Dependencies**: Reliance on third-party cloud services
- **AI Complexity**: Machine learning model development and training
- **Platform Fragmentation**: Multiple platform maintenance overhead
""",
            "labels": ["epic", "cloud", "ai", "platform", "priority:future"]
        }
        
        epics.extend([video_epic, obs_epic, pss_epic, ui_epic, system_epic, platform_epic])
        return epics

    def create_development_infrastructure_tasks(self) -> List[Dict[str, Any]]:
        """Create development infrastructure and tooling tasks"""
        infrastructure_tasks = [
            {
                "title": "[TASK] Install Tauri CLI - Development Environment Setup",
                "body": """## 🎯 Task Description
Install Tauri CLI in the development environment to enable Tauri backend development and testing.

## ✅ Acceptance Criteria
- [ ] Tauri CLI installed successfully (`cargo install tauri-cli`)
- [ ] `tauri --version` command working
- [ ] Tauri development server can start
- [ ] Build commands functional

## 🔗 Related Components
- [x] Backend (Rust)
- [ ] Infrastructure

## 📊 Priority & Effort
- **Priority**: High (Critical path)
- **Effort**: Small (< 4 hours)

## 📝 Technical Notes
Currently missing from development environment. Required for Tauri backend development and testing.

## ✅ Definition of Done
- [ ] Tauri CLI installed and working
- [ ] Development server tested
- [ ] Build process verified
- [ ] Documentation updated
""",
                "labels": ["task", "infrastructure", "dev-tools", "priority:high", "effort:small"]
            },
            {
                "title": "[TASK] Address Security Vulnerabilities - NPM Audit",
                "body": """## 🎯 Task Description
Address security vulnerabilities found in NPM dependencies (9 vulnerabilities: 3 moderate, 6 high).

## ✅ Acceptance Criteria
- [ ] Security audit performed (`npm audit`)
- [ ] High and moderate vulnerabilities addressed
- [ ] Dependencies updated safely
- [ ] Application functionality verified after updates

## 🔗 Related Components
- [x] Frontend (React)
- [ ] Security

## 📊 Priority & Effort
- **Priority**: High (Critical path)
- **Effort**: Small (< 4 hours)

## 📝 Technical Notes
Current audit shows 9 vulnerabilities (3 moderate, 6 high). Need to resolve without breaking functionality.

## ✅ Definition of Done
- [ ] Vulnerabilities resolved
- [ ] Application tested after updates
- [ ] Clean security audit
- [ ] Dependencies documented
""",
                "labels": ["task", "security", "frontend", "priority:high", "effort:small"]
            },
            {
                "title": "[TASK] Enable Tauri Commands - Frontend-Backend Bridge",
                "body": """## 🎯 Task Description
Re-enable commented Tauri commands in tauri_commands.rs for frontend-backend communication.

## ✅ Acceptance Criteria
- [ ] All TODO comments in tauri_commands.rs resolved
- [ ] OBS connection commands enabled
- [ ] Frontend can communicate with backend via Tauri
- [ ] Commands tested and functional

## 🔗 Related Components
- [x] Backend (Rust)
- [x] Frontend (React)

## 📊 Priority & Effort
- **Priority**: High (Critical path)
- **Effort**: Medium (4-16 hours)

## 📝 Technical Notes
Many Tauri commands are commented out with TODO notes. Need to re-enable when Tauri is properly integrated.

## ✅ Definition of Done
- [ ] Tauri commands enabled
- [ ] Frontend integration working
- [ ] Commands tested
- [ ] Error handling verified
""",
                "labels": ["task", "backend", "integration", "priority:high", "effort:medium"]
            }
        ]
        
        return infrastructure_tasks

    def create_documentation_tasks(self) -> List[Dict[str, Any]]:
        """Create documentation and maintenance tasks"""
        doc_tasks = [
            {
                "title": "[TASK] User Manual Creation - End User Documentation",
                "body": """## 🎯 Task Description
Create comprehensive user manual for end users (taekwondo referees and tournament organizers).

## ✅ Acceptance Criteria
- [ ] Installation guide for Windows systems
- [ ] Quick start guide for new users
- [ ] Feature documentation with screenshots
- [ ] Troubleshooting section
- [ ] FAQ based on common questions

## 🔗 Related Components
- [ ] Documentation

## 📊 Priority & Effort
- **Priority**: Medium (Important)
- **Effort**: Large (16+ hours)

## 📝 Technical Notes
Need user-friendly documentation for non-technical users in tournament environments.

## ✅ Definition of Done
- [ ] Complete user manual created
- [ ] Installation guide tested
- [ ] Screenshots and examples included
- [ ] Reviewed by actual users
""",
                "labels": ["task", "documentation", "priority:medium", "effort:large"]
            },
            {
                "title": "[TASK] Deployment Guide Creation - Technical Documentation",
                "body": """## 🎯 Task Description
Create technical deployment guide for system administrators and IT personnel.

## ✅ Acceptance Criteria
- [ ] System requirements documented
- [ ] Network configuration guide
- [ ] OBS Studio setup instructions
- [ ] Troubleshooting for IT support
- [ ] Security considerations

## 🔗 Related Components
- [ ] Documentation
- [ ] Infrastructure

## 📊 Priority & Effort
- **Priority**: Medium (Important)
- **Effort**: Medium (4-16 hours)

## 📝 Technical Notes
Technical documentation for professional tournament environments and IT departments.

## ✅ Definition of Done
- [ ] Deployment guide complete
- [ ] Network setup documented
- [ ] Security guide included
- [ ] IT troubleshooting section
""",
                "labels": ["task", "documentation", "infrastructure", "priority:medium", "effort:medium"]
            }
        ]
        
        return doc_tasks

    def create_all_issues(self) -> None:
        """Create all issues from project documentation"""
        all_issues = []
        
        print("🚀 Creating GitHub issues from reStrike VTA project documentation...")
        print("=" * 80)
        
        # Immediate priority tasks (2% remaining for production)
        print("\n📋 Creating Immediate Priority Tasks...")
        immediate_tasks = self.create_immediate_priority_tasks()
        all_issues.extend(immediate_tasks)
        
        # Development infrastructure tasks
        print("\n🔧 Creating Development Infrastructure Tasks...")
        infrastructure_tasks = self.create_development_infrastructure_tasks()
        all_issues.extend(infrastructure_tasks)
        
        # Documentation tasks
        print("\n📚 Creating Documentation Tasks...")
        doc_tasks = self.create_documentation_tasks()
        all_issues.extend(doc_tasks)
        
        # Future enhancement epics
        print("\n🎯 Creating Future Enhancement Epics...")
        epics = self.create_future_enhancement_epics()
        all_issues.extend(epics)
        
        # Create all issues
        created_count = 0
        failed_count = 0
        
        for issue in all_issues:
            print(f"\n📝 Creating: {issue['title']}")
            result = self.create_issue(
                title=issue['title'],
                body=issue['body'],
                labels=issue['labels']
            )
            
            if result:
                print(f"   ✅ Created issue #{result['number']}: {result['html_url']}")
                created_count += 1
            else:
                print(f"   ❌ Failed to create issue")
                failed_count += 1
        
        print("\n" + "=" * 80)
        print(f"📊 Summary:")
        print(f"   ✅ Successfully created: {created_count} issues")
        print(f"   ❌ Failed to create: {failed_count} issues")
        print(f"   📋 Total processed: {len(all_issues)} issues")
        
        if created_count > 0:
            print(f"\n🎉 GitHub project management integration complete!")
            print(f"   📁 View all issues: https://github.com/{self.repo_owner}/{self.repo_name}/issues")
            print(f"   📋 Project board: https://github.com/{self.repo_owner}/{self.repo_name}/projects")
            
        print("\n🔗 Next Steps:")
        print("   1. Review created issues and epics")
        print("   2. Set up project board with automation")
        print("   3. Assign issues to team members")
        print("   4. Configure milestone tracking")
        print("   5. Set up automated workflows")

def main():
    """Main function to create GitHub issues"""
    # Configuration
    repo_owner = "damjanZGB"  # Replace with actual repo owner
    repo_name = "reStrike_VTA_Cursor"  # Replace with actual repo name
    
    # Get GitHub token from environment
    token = os.getenv("GITHUB_TOKEN")
    if not token:
        print("❌ Error: GITHUB_TOKEN environment variable not set")
        print("   Please set your GitHub Personal Access Token:")
        print("   export GITHUB_TOKEN=your_token_here")
        sys.exit(1)
    
    # Create issues
    creator = GitHubIssuesCreator(repo_owner, repo_name, token)
    creator.create_all_issues()

if __name__ == "__main__":
    main() 