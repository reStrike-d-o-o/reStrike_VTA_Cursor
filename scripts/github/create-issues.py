#!/usr/bin/env python3
"""
GitHub Issues Creator
Automatically creates GitHub issues from current todos for the reStrike VTA project.
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

    def create_testing_tasks(self) -> List[Dict[str, Any]]:
        """Create all testing phase tasks"""
        issues = []
        
        # Frontend Testing Tasks
        frontend_tasks = [
            {
                "title": "[TASK] Test VideoClips Component Functionality",
                "body": """## Task Description
Test all functionality of the VideoClips.tsx component (315 lines) including clip management, search, filtering, and UI interactions.

## Acceptance Criteria
- [ ] Clip creation and deletion works correctly
- [ ] Search functionality filters clips properly
- [ ] Tag-based filtering works as expected
- [ ] Grid layout displays clips correctly
- [ ] Statistics display accurate information
- [ ] All user interactions work smoothly

## Related Components
- [x] Frontend (React)
- [ ] Backend (Rust)
- [ ] Documentation
- [x] Testing
- [ ] CI/CD

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [ ] Small (< 4 hours)
- [x] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Technical Notes
- Test with various video file formats
- Verify thumbnail generation
- Test search performance with many clips
- Validate tag management functionality

## Definition of Done
- [ ] All functionality tested manually
- [ ] Edge cases identified and documented
- [ ] Performance tested with large clip collections
- [ ] No critical bugs found
- [ ] Test results documented""",
                "labels": ["task", "frontend", "testing", "phase:testing", "priority:high", "effort:medium"]
            },
            {
                "title": "[TASK] Test Settings Component Configuration",
                "body": """## Task Description
Test all functionality of the Settings.tsx component (402 lines) including overlay settings, OBS integration settings, and advanced configuration options.

## Acceptance Criteria
- [ ] Overlay position changes work correctly
- [ ] Theme switching functions properly
- [ ] Opacity and scale adjustments work
- [ ] Settings persistence works
- [ ] OBS status display is accurate
- [ ] All tabs and navigation work

## Related Components
- [x] Frontend (React)
- [ ] Backend (Rust)
- [ ] Documentation
- [x] Testing
- [ ] CI/CD

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [ ] Small (< 4 hours)
- [x] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Technical Notes
- Test settings persistence across browser sessions
- Verify live preview functionality
- Test all keyboard shortcuts
- Validate form validation

## Definition of Done
- [ ] All settings options tested
- [ ] Settings persistence verified
- [ ] UI responsiveness confirmed
- [ ] No configuration issues found
- [ ] Test results documented""",
                "labels": ["task", "frontend", "testing", "phase:testing", "priority:high", "effort:medium"]
            },
            {
                "title": "[TASK] Test Overlay Component Video Playback",
                "body": """## Task Description
Test all functionality of the Overlay.tsx component (306 lines) including video playback controls, positioning, themes, and real-time status display.

## Acceptance Criteria
- [ ] Video playback controls work correctly
- [ ] Overlay positioning functions properly
- [ ] Theme switching works as expected
- [ ] Opacity and scale adjustments work
- [ ] Status bar displays accurate information
- [ ] Fullscreen mode works correctly

## Related Components
- [x] Frontend (React)
- [ ] Backend (Rust)
- [ ] Documentation
- [x] Testing
- [ ] CI/CD

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [ ] Small (< 4 hours)
- [x] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Technical Notes
- Test with various video formats
- Verify overlay positioning on different screen sizes
- Test performance with high-resolution videos
- Validate real-time status updates

## Definition of Done
- [ ] All video controls tested
- [ ] Overlay positioning verified
- [ ] Performance acceptable
- [ ] No playback issues found
- [ ] Test results documented""",
                "labels": ["task", "frontend", "testing", "phase:testing", "priority:high", "effort:medium"]
            }
        ]
        
        # Backend Testing Tasks
        backend_tasks = [
            {
                "title": "[TASK] Test UDP PSS Protocol Parser",
                "body": """## Task Description
Test the plugin_udp.rs module (640 lines) for PSS protocol message parsing, event generation, and real-time data processing.

## Acceptance Criteria
- [ ] All PSS message types parse correctly
- [ ] Event generation works as expected
- [ ] UDP server starts and stops properly
- [ ] Error handling works correctly
- [ ] Performance is acceptable
- [ ] Statistics tracking is accurate

## Related Components
- [ ] Frontend (React)
- [x] Backend (Rust)
- [ ] Documentation
- [x] Testing
- [ ] CI/CD

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [ ] Small (< 4 hours)
- [x] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Technical Notes
- Test with various PSS message formats
- Verify UDP socket handling
- Test error recovery mechanisms
- Validate event broadcasting

## Definition of Done
- [ ] All PSS events parse correctly
- [ ] UDP server stability verified
- [ ] Error handling tested
- [ ] Performance benchmarked
- [ ] Test results documented""",
                "labels": ["task", "backend", "testing", "phase:testing", "priority:high", "effort:medium"]
            },
            {
                "title": "[TASK] Test OBS WebSocket Integration",
                "body": """## Task Description
Test the plugin_obs.rs module (455 lines) for OBS WebSocket v4/v5 dual protocol support, connection management, and all OBS control features.

## Acceptance Criteria
- [ ] Both v4 and v5 protocols work correctly
- [ ] Connection management is stable
- [ ] Authentication works for both protocols
- [ ] All OBS commands execute properly
- [ ] Error handling is robust
- [ ] Multiple connections supported

## Related Components
- [ ] Frontend (React)
- [x] Backend (Rust)
- [ ] Documentation
- [x] Testing
- [x] Integration

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [ ] Small (< 4 hours)
- [x] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Technical Notes
- Test with actual OBS Studio instances
- Verify both protocol versions
- Test reconnection logic
- Validate command response handling

## Definition of Done
- [ ] Both protocols tested and working
- [ ] Connection stability verified
- [ ] All commands functional
- [ ] Error handling tested
- [ ] Test results documented""",
                "labels": ["task", "backend", "testing", "integration", "phase:testing", "priority:high", "effort:medium"]
            },
            {
                "title": "[TASK] Test Video Playback Integration",
                "body": """## Task Description
Test the plugin_playback.rs module (568 lines) for mpv integration, video playback controls, and metadata handling.

## Acceptance Criteria
- [ ] mpv integration works correctly
- [ ] All playback controls function properly
- [ ] Video metadata extraction works
- [ ] Performance is acceptable
- [ ] Error handling is robust
- [ ] Multiple video formats supported

## Related Components
- [ ] Frontend (React)
- [x] Backend (Rust)
- [ ] Documentation
- [x] Testing
- [ ] CI/CD

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [ ] Small (< 4 hours)
- [x] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Technical Notes
- Test with various video formats
- Verify mpv process management
- Test seek and control operations
- Validate metadata extraction

## Definition of Done
- [ ] mpv integration stable
- [ ] All controls functional
- [ ] Performance acceptable
- [ ] Format support verified
- [ ] Test results documented""",
                "labels": ["task", "backend", "testing", "phase:testing", "priority:high", "effort:medium"]
            }
        ]
        
        # Integration Testing Tasks
        integration_tasks = [
            {
                "title": "[TASK] Test Frontend-Backend Integration",
                "body": """## Task Description
Test all Tauri command integration between React frontend and Rust backend, ensuring proper communication and data flow.

## Acceptance Criteria
- [ ] All Tauri commands execute correctly
- [ ] Data serialization/deserialization works
- [ ] Error propagation functions properly
- [ ] Performance is acceptable
- [ ] Real-time updates work correctly

## Related Components
- [x] Frontend (React)
- [x] Backend (Rust)
- [ ] Documentation
- [x] Testing
- [ ] CI/CD

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [ ] Small (< 4 hours)
- [x] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Dependencies
- [ ] Frontend component testing
- [ ] Backend plugin testing

## Technical Notes
- Test all Tauri command handlers
- Verify data type conversions
- Test error handling across boundaries
- Validate real-time event streaming

## Definition of Done
- [ ] All commands tested and working
- [ ] Data flow verified
- [ ] Error handling tested
- [ ] Performance acceptable
- [ ] Test results documented""",
                "labels": ["task", "frontend", "backend", "testing", "integration", "phase:testing", "priority:high", "effort:medium"]
            }
        ]
        
        return frontend_tasks + backend_tasks + integration_tasks

    def create_production_tasks(self) -> List[Dict[str, Any]]:
        """Create all production deployment tasks"""
        return [
            {
                "title": "[TASK] Generate Windows Executable Build",
                "body": """## Task Description
Generate production Windows .exe executable using Tauri build system and verify all dependencies are bundled correctly.

## Acceptance Criteria
- [ ] Windows .exe builds successfully
- [ ] All dependencies are bundled
- [ ] Application runs on clean Windows system
- [ ] No missing DLL errors
- [ ] Icon and metadata correct
- [ ] File size is reasonable

## Related Components
- [x] Frontend (React)
- [x] Backend (Rust)
- [ ] Documentation
- [ ] Testing
- [x] CI/CD

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [x] Small (< 4 hours)
- [ ] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Dependencies
- [ ] All integration testing complete

## Technical Notes
- Use Tauri production build configuration
- Test on multiple Windows versions
- Verify code signing if applicable
- Check antivirus compatibility

## Definition of Done
- [ ] Executable builds successfully
- [ ] Dependencies verified
- [ ] Clean system testing passed
- [ ] Performance acceptable
- [ ] Build documented""",
                "labels": ["task", "infrastructure", "phase:production", "priority:high", "effort:small"]
            },
            {
                "title": "[TASK] Create MSI Installer Package",
                "body": """## Task Description
Create MSI installer package for professional Windows deployment and test installation on clean Windows systems.

## Acceptance Criteria
- [ ] MSI installer creates successfully
- [ ] Installation process is smooth
- [ ] Uninstallation works correctly
- [ ] Start menu shortcuts created
- [ ] Registry entries proper
- [ ] Professional appearance

## Related Components
- [ ] Frontend (React)
- [ ] Backend (Rust)
- [ ] Documentation
- [ ] Testing
- [x] Infrastructure

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [x] Small (< 4 hours)
- [ ] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Dependencies
- [ ] Windows executable generation

## Technical Notes
- Use WiX Toolset or similar
- Include proper metadata
- Test on various Windows versions
- Verify admin privileges handling

## Definition of Done
- [ ] MSI installer created
- [ ] Installation tested
- [ ] Uninstallation verified
- [ ] Professional quality
- [ ] Documentation updated""",
                "labels": ["task", "infrastructure", "phase:production", "priority:high", "effort:small"]
            },
            {
                "title": "[TASK] Performance Validation and Optimization",
                "body": """## Task Description
Validate application performance including memory usage (<2GB), CPU usage, video playback performance, and real-time data processing efficiency.

## Acceptance Criteria
- [ ] Memory usage under 2GB
- [ ] CPU usage reasonable
- [ ] Video playback smooth
- [ ] Real-time data responsive
- [ ] No memory leaks detected
- [ ] Performance benchmarks documented

## Related Components
- [x] Frontend (React)
- [x] Backend (Rust)
- [ ] Documentation
- [x] Testing
- [ ] CI/CD

## Priority
- [x] High (Critical path)
- [ ] Medium (Important)
- [ ] Low (Nice to have)

## Estimated Effort
- [ ] Small (< 4 hours)
- [x] Medium (4-16 hours)
- [ ] Large (16+ hours)

## Dependencies
- [ ] Windows executable generation
- [ ] Integration testing complete

## Technical Notes
- Use profiling tools
- Test with realistic data loads
- Monitor resource usage over time
- Identify and fix bottlenecks

## Definition of Done
- [ ] Performance targets met
- [ ] Bottlenecks identified and fixed
- [ ] Long-running stability verified
- [ ] Benchmarks documented
- [ ] Optimization complete""",
                "labels": ["task", "backend", "frontend", "testing", "phase:production", "priority:high", "effort:medium"]
            }
        ]

    def create_epics(self) -> List[Dict[str, Any]]:
        """Create Epic issues for major feature categories"""
        return [
            {
                "title": "[EPIC] Core System Testing Phase",
                "body": """## Epic Overview
Complete comprehensive testing of all implemented features including frontend components, backend plugins, and integration between systems.

## Business Value
Ensures the application is production-ready with all features working correctly before Windows deployment. Critical for successful launch and user adoption.

## User Stories
- As a developer, I want all frontend components tested so that users have a smooth UI experience
- As a developer, I want all backend plugins tested so that core functionality is reliable
- As a developer, I want integration testing complete so that frontend-backend communication is stable
- As a referee, I want a bug-free application so that I can rely on it during competitions

## Acceptance Criteria
- [ ] All 5 React components (1,691 lines) tested and functional
- [ ] All 3 Rust plugins (1,663 lines) tested and operational
- [ ] Frontend-backend integration via Tauri verified
- [ ] Keyboard shortcuts and user interactions tested
- [ ] Performance benchmarks meet requirements
- [ ] No critical bugs remaining

## Related Tasks
- [ ] Frontend component testing (VideoClips, Settings, Overlay, ObsWebSocketManager, App)
- [ ] Backend plugin testing (UDP, OBS, Playback)
- [ ] Integration testing (Tauri commands, real-time data flow)
- [ ] Performance validation
- [ ] User acceptance testing preparation

## Technical Architecture
- **Frontend**: React 18 + TypeScript + Zustand + Tailwind CSS
- **Backend**: Rust + Tauri + Plugin Architecture
- **Integration**: Tauri commands for frontend-backend communication
- **Testing**: Manual testing with automated validation where possible

## Dependencies
- [ ] Complete application stack implementation (✅ Done)
- [ ] Development environment setup (✅ Done)
- [ ] Documentation and testing procedures

## Success Metrics
- Zero critical bugs found
- All functionality working as specified
- Performance within acceptable limits
- Ready for Windows production build

## Timeline
- **Start Date**: 2025-01-27
- **Target Completion**: 2025-02-03
- **Milestones**:
  - [ ] Frontend testing complete (2025-01-29)
  - [ ] Backend testing complete (2025-01-31)
  - [ ] Integration testing complete (2025-02-02)
  - [ ] Performance validation complete (2025-02-03)

## Risk Assessment
- **Risk 1**: Complex integration issues discovered - Mitigation: Comprehensive integration testing early
- **Risk 2**: Performance bottlenecks found - Mitigation: Performance testing with realistic loads
- **Risk 3**: Time constraints - Mitigation: Parallel testing where possible

## Notes
This epic represents the final validation phase before production deployment. Success here directly leads to Windows executable generation and MSI installer creation.""",
                "labels": ["epic", "testing", "phase:testing", "priority:high"]
            },
            {
                "title": "[EPIC] Windows Production Deployment",
                "body": """## Epic Overview
Generate production-ready Windows desktop application including executable build, MSI installer, and deployment verification on target Windows systems.

## Business Value
Delivers the final Windows desktop application ready for distribution to taekwondo referees and competition officials. Enables real-world usage and adoption.

## User Stories
- As a referee, I want a professional Windows installer so that I can easily install the application
- As a competition organizer, I want a reliable executable so that I can deploy it across multiple systems
- As an IT administrator, I want proper installation/uninstallation so that I can manage the software
- As a user, I want the application to run smoothly on Windows 10/11 so that I can use it for competitions

## Acceptance Criteria
- [ ] Windows .exe executable builds successfully with all dependencies
- [ ] MSI installer package created with professional appearance
- [ ] Installation/uninstallation tested on clean Windows systems
- [ ] Performance validated on target hardware
- [ ] Application runs without errors on Windows 10/11
- [ ] Documentation updated for deployment procedures

## Related Tasks
- [ ] Windows executable generation using Tauri build
- [ ] MSI installer creation with proper metadata
- [ ] Clean system installation testing
- [ ] Performance validation and optimization
- [ ] Deployment documentation
- [ ] Distribution preparation

## Technical Architecture
- **Build System**: Tauri with Windows-specific configuration
- **Installer**: MSI package with WiX Toolset
- **Dependencies**: All libraries bundled or properly referenced
- **Deployment**: Standalone executable with minimal external dependencies

## Dependencies
- [ ] Core system testing phase completion
- [ ] All critical bugs resolved
- [ ] Performance requirements met
- [ ] Windows development environment ready

## Success Metrics
- Successful installation on 100% of tested Windows systems
- Application launches within 5 seconds
- Memory usage under 2GB during normal operation
- No crashes during extended usage testing

## Timeline
- **Start Date**: 2025-02-03
- **Target Completion**: 2025-02-10
- **Milestones**:
  - [ ] Executable build complete (2025-02-05)
  - [ ] MSI installer created (2025-02-06)
  - [ ] Clean system testing complete (2025-02-08)
  - [ ] Performance validation complete (2025-02-10)

## Risk Assessment
- **Risk 1**: Build dependencies missing - Mitigation: Comprehensive dependency testing
- **Risk 2**: Windows compatibility issues - Mitigation: Testing on multiple Windows versions
- **Risk 3**: Performance degradation in production build - Mitigation: Early performance testing

## Notes
This epic delivers the final production-ready application for Windows deployment. Success enables real-world usage by taekwondo referees and competition officials.""",
                "labels": ["epic", "infrastructure", "phase:production", "priority:high"]
            },
            {
                "title": "[EPIC] Video System Enhancements",
                "body": """## Epic Overview
Enhance the video playback and management system with advanced features including slow-motion controls, multi-angle support, auto-thumbnails, and professional-grade video processing capabilities.

## Business Value
Provides advanced video analysis capabilities that differentiate the application from basic replay tools, offering professional-grade features for detailed match analysis and improved referee decision-making.

## User Stories
- As a referee, I want slow-motion playback so that I can analyze close calls in detail
- As a referee, I want frame-by-frame navigation so that I can examine specific moments precisely
- As a competition organizer, I want multiple camera angles so that I can provide comprehensive coverage
- As a user, I want automatic thumbnails so that I can quickly identify clips visually
- As an operator, I want video effects controls so that I can enhance visibility for analysis

## Acceptance Criteria
- [ ] Variable speed playback (0.25x, 0.5x, 2x) implemented
- [ ] Frame-by-frame navigation with arrow keys
- [ ] Multiple camera feed synchronization
- [ ] Automatic thumbnail generation for all clips
- [ ] Video effects (contrast, brightness, color adjustment)
- [ ] Advanced export options with multiple formats
- [ ] Cloud storage integration for clip backup
- [ ] Batch operations for clip management

## Related Tasks
- [ ] Implement slow-motion playback controls
- [ ] Add frame-by-frame navigation
- [ ] Develop multi-angle synchronization
- [ ] Create thumbnail generation system
- [ ] Add video effects processing
- [ ] Implement advanced export features
- [ ] Integrate cloud storage options
- [ ] Develop batch operation tools

## Technical Architecture
- **Video Engine**: Enhanced mpv integration with custom controls
- **Processing**: Hardware-accelerated video effects
- **Storage**: Local + cloud hybrid storage system
- **Synchronization**: Timeline-based multi-stream synchronization

## Dependencies
- [ ] Core video playback system working
- [ ] Performance optimization complete
- [ ] Cloud storage API integration

## Success Metrics
- Smooth playback at all speed levels
- Frame-accurate navigation
- <100ms synchronization between camera feeds
- Thumbnail generation <2 seconds per clip

## Timeline
- **Start Date**: 2025-02-10
- **Target Completion**: 2025-04-10
- **Milestones**:
  - [ ] Advanced playback controls (2025-02-24)
  - [ ] Multi-angle support (2025-03-10)
  - [ ] Thumbnail system (2025-03-24)
  - [ ] Cloud integration (2025-04-10)

## Risk Assessment
- **Risk 1**: Performance impact from advanced features - Mitigation: Hardware acceleration and optimization
- **Risk 2**: Synchronization complexity - Mitigation: Robust timeline architecture
- **Risk 3**: Cloud integration complexity - Mitigation: Standard APIs and incremental implementation

## Notes
This epic represents the evolution from basic replay to professional video analysis tool, significantly enhancing the application's value proposition for professional taekwondo competitions.""",
                "labels": ["epic", "enhancement", "frontend", "backend", "phase:enhancement", "priority:medium"]
            }
        ]

    def create_all_issues(self) -> None:
        """Create all GitHub issues from todos"""
        print("🚀 Creating GitHub Issues from Todos...")
        
        # Create testing tasks
        print("\n📋 Creating Testing Phase Tasks...")
        testing_tasks = self.create_testing_tasks()
        created_issues = []
        
        for task in testing_tasks:
            print(f"Creating: {task['title']}")
            issue = self.create_issue(
                title=task['title'],
                body=task['body'],
                labels=task['labels']
            )
            if issue:
                created_issues.append(issue)
                print(f"✅ Created issue #{issue['number']}: {issue['title']}")
            else:
                print(f"❌ Failed to create: {task['title']}")
        
        # Create production tasks
        print("\n🏭 Creating Production Phase Tasks...")
        production_tasks = self.create_production_tasks()
        
        for task in production_tasks:
            print(f"Creating: {task['title']}")
            issue = self.create_issue(
                title=task['title'],
                body=task['body'],
                labels=task['labels']
            )
            if issue:
                created_issues.append(issue)
                print(f"✅ Created issue #{issue['number']}: {issue['title']}")
            else:
                print(f"❌ Failed to create: {task['title']}")
        
        # Create epics
        print("\n🎯 Creating Epic Issues...")
        epics = self.create_epics()
        
        for epic in epics:
            print(f"Creating: {epic['title']}")
            issue = self.create_issue(
                title=epic['title'],
                body=epic['body'],
                labels=epic['labels']
            )
            if issue:
                created_issues.append(issue)
                print(f"✅ Created epic #{issue['number']}: {issue['title']}")
            else:
                print(f"❌ Failed to create: {epic['title']}")
        
        print(f"\n🎉 Successfully created {len(created_issues)} GitHub issues!")
        print("\n📊 Summary:")
        print(f"  • Testing Tasks: {len(testing_tasks)} issues")
        print(f"  • Production Tasks: {len(production_tasks)} issues")
        print(f"  • Epics: {len(epics)} issues")
        print(f"  • Total: {len(created_issues)} issues")

def main():
    """Main function to create GitHub issues"""
    # Get configuration from environment or command line
    repo_owner = os.getenv('GITHUB_REPO_OWNER', 'damjanZGB')
    repo_name = os.getenv('GITHUB_REPO_NAME', 'reStrike_VTA_Cursor')
    token = os.getenv('GITHUB_TOKEN')
    
    if not token:
        print("❌ Error: GITHUB_TOKEN environment variable not set")
        print("Please set GITHUB_TOKEN with a valid GitHub Personal Access Token")
        sys.exit(1)
    
    # Create issues
    creator = GitHubIssuesCreator(repo_owner, repo_name, token)
    creator.create_all_issues()

if __name__ == "__main__":
    main() 