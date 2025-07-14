# GitHub Integration Guide

This guide provides complete instructions for setting up GitHub-based project management and tracking for the reStrike VTA project.

## 🎯 Overview

The reStrike VTA project uses GitHub as the central hub for:
- **Issue Tracking** - All tasks, bugs, and features
- **Project Management** - Kanban-style project boards
- **Workflow Automation** - Automated labeling and status updates
- **Team Collaboration** - Assignments, reviews, and communication
- **Progress Monitoring** - Real-time project status and metrics

## 🚀 Quick Start

### 1. **Set Up Personal Access Token**
```bash
# 1. Go to GitHub Settings > Developer settings > Personal access tokens
# 2. Click "Generate new token (classic)"
# 3. Select these scopes:
#    - repo (Full control of private repositories)
#    - project (Full control of projects)
#    - workflow (Update GitHub Action workflows)
# 4. Copy the token and set as environment variable

export GITHUB_TOKEN="ghp_your_token_here"
export GITHUB_REPO_OWNER="damjanZGB"
export GITHUB_REPO_NAME="reStrike_VTA_Cursor"
```

### 2. **Create GitHub Issues from Todos**
```bash
# Install dependencies
pip install requests

# Run the issue creation script
python3 scripts/github/create-issues.py

# Expected output:
# 🚀 Creating GitHub Issues from Todos...
# ✅ Created 15 GitHub issues!
```

### 3. **Set Up Project Board**
```bash
# Run the project board setup
python3 scripts/github/setup-project-board.py

# Expected output:
# 🚀 Setting up reStrike VTA Project Board...
# ✅ Created project board with 6 columns
```

### 4. **Configure Repository Secrets**
```bash
# Add PAT_TOKEN to repository secrets:
# 1. Go to repository Settings > Secrets and variables > Actions
# 2. Click "New repository secret"
# 3. Name: PAT_TOKEN
# 4. Value: your GitHub token
```

## 📋 Project Structure

### **Issue Types**
- **🎯 Tasks** - Specific development work items
- **🐛 Bugs** - Issues that need fixing
- **✨ Features** - New functionality requests
- **📚 Epics** - Large feature sets or initiatives
- **📖 Documentation** - Documentation improvements

### **Labels System**

#### Type Labels
| Label | Description | Color |
|-------|-------------|-------|
| `task` | Development task | ![#0075ca](https://via.placeholder.com/15/0075ca/000000?text=+) `#0075ca` |
| `bug` | Something isn't working | ![#d73a4a](https://via.placeholder.com/15/d73a4a/000000?text=+) `#d73a4a` |
| `enhancement` | New feature request | ![#a2eeef](https://via.placeholder.com/15/a2eeef/000000?text=+) `#a2eeef` |
| `epic` | Large feature set | ![#8b5dba](https://via.placeholder.com/15/8b5dba/000000?text=+) `#8b5dba` |
| `documentation` | Documentation related | ![#0075ca](https://via.placeholder.com/15/0075ca/000000?text=+) `#0075ca` |

#### Component Labels
| Label | Description | Color |
|-------|-------------|-------|
| `frontend` | React frontend | ![#61dafb](https://via.placeholder.com/15/61dafb/000000?text=+) `#61dafb` |
| `backend` | Rust backend | ![#ce422b](https://via.placeholder.com/15/ce422b/000000?text=+) `#ce422b` |
| `integration` | External integration | ![#fbca04](https://via.placeholder.com/15/fbca04/000000?text=+) `#fbca04` |
| `testing` | Testing related | ![#1d76db](https://via.placeholder.com/15/1d76db/000000?text=+) `#1d76db` |
| `ci/cd` | CI/CD pipeline | ![#2ea44f](https://via.placeholder.com/15/2ea44f/000000?text=+) `#2ea44f` |

#### Priority Labels
| Label | Description | Color |
|-------|-------------|-------|
| `priority:high` | Critical path | ![#b60205](https://via.placeholder.com/15/b60205/000000?text=+) `#b60205` |
| `priority:medium` | Important | ![#fbca04](https://via.placeholder.com/15/fbca04/000000?text=+) `#fbca04` |
| `priority:low` | Nice to have | ![#0e8a16](https://via.placeholder.com/15/0e8a16/000000?text=+) `#0e8a16` |

#### Status Labels
| Label | Description | Color |
|-------|-------------|-------|
| `status:todo` | Not started | ![#ededed](https://via.placeholder.com/15/ededed/000000?text=+) `#ededed` |
| `status:in-progress` | Being worked on | ![#fbca04](https://via.placeholder.com/15/fbca04/000000?text=+) `#fbca04` |
| `status:review` | Ready for review | ![#0052cc](https://via.placeholder.com/15/0052cc/000000?text=+) `#0052cc` |
| `status:done` | Completed | ![#0e8a16](https://via.placeholder.com/15/0e8a16/000000?text=+) `#0e8a16` |
| `status:blocked` | Blocked | ![#d93f0b](https://via.placeholder.com/15/d93f0b/000000?text=+) `#d93f0b` |

#### Effort Labels
| Label | Description | Color |
|-------|-------------|-------|
| `effort:small` | < 4 hours | ![#c2e0c6](https://via.placeholder.com/15/c2e0c6/000000?text=+) `#c2e0c6` |
| `effort:medium` | 4-16 hours | ![#fef2c0](https://via.placeholder.com/15/fef2c0/000000?text=+) `#fef2c0` |
| `effort:large` | 16+ hours | ![#f9d0c4](https://via.placeholder.com/15/f9d0c4/000000?text=+) `#f9d0c4` |

#### Phase Labels
| Label | Description | Color |
|-------|-------------|-------|
| `phase:testing` | Core testing phase | ![#1f77b4](https://via.placeholder.com/15/1f77b4/000000?text=+) `#1f77b4` |
| `phase:production` | Production deployment | ![#ff7f0e](https://via.placeholder.com/15/ff7f0e/000000?text=+) `#ff7f0e` |
| `phase:enhancement` | Future enhancements | ![#2ca02c](https://via.placeholder.com/15/2ca02c/000000?text=+) `#2ca02c` |

## 📊 Project Board

### **Column Structure**
1. **🗂️ Backlog** - Future planned tasks and enhancements
2. **📋 Todo** - Ready to start, dependencies met
3. **🔄 In Progress** - Currently being worked on
4. **👀 Review** - Awaiting review or testing
5. **✅ Done** - Completed tasks
6. **🚫 Blocked** - Blocked by external dependencies

### **Automation Rules**
- **New Issues** → Automatically added to "Todo" column
- **Assigned Issues** → Automatically moved to "In Progress"
- **Pull Request Opened** → Move to "Review" column
- **Issue Closed** → Move to "Done" column
- **Blocked Label Added** → Move to "Blocked" column

## 🔧 Workflow Automation

### **GitHub Actions Workflows**

#### `.github/workflows/issue-management.yml`
- **Auto-labeling** based on issue title and content
- **Epic validation** ensures proper formatting
- **Project board integration** adds issues automatically
- **Status synchronization** between labels and board

#### `.github/workflows/project-board.yml`
- **Column management** based on issue status
- **Automatic closure** when marked as done
- **Progress tracking** and status updates

#### `.github/workflows/labels.yml`
- **Label synchronization** from configuration file
- **Consistent labeling** across the repository

### **Automated Label Application**

Issues are automatically labeled based on title keywords:

```yaml
# Example auto-labeling rules:
- Title contains "frontend" → Add "frontend" label
- Title contains "backend" → Add "backend" label
- Title contains "test" → Add "testing" label
- Title contains "critical" → Add "priority:high" label
- Title prefix "[TASK]" → Add "task" label
- Title prefix "[EPIC]" → Add "epic" label
```

## 📝 Issue Templates

### **Task Template**
```markdown
---
name: Task
about: Create a task for project development
title: "[TASK] "
labels: task
---

## Task Description
Clear description of what needs to be done.

## Acceptance Criteria
- [ ] Criteria 1
- [ ] Criteria 2
- [ ] Criteria 3

## Related Components
- [ ] Frontend (React)
- [ ] Backend (Rust)
- [ ] Documentation
- [ ] Testing

## Priority & Effort
- Priority: High/Medium/Low
- Effort: Small/Medium/Large

## Dependencies
- [ ] Issue #XXX

## Definition of Done
- [ ] Implementation complete
- [ ] Tests written and passing
- [ ] Documentation updated
- [ ] Code reviewed
```

### **Epic Template**
```markdown
---
name: Epic
about: Track large feature sets
title: "[EPIC] "
labels: epic
---

## Epic Overview
High-level description of the feature set.

## Business Value
Why this epic is important.

## User Stories
- As a [user type], I want [goal] so that [benefit]

## Acceptance Criteria
- [ ] High-level criteria

## Related Tasks
- [ ] #XXX - Task name

## Success Metrics
- Metric 1: Target value
- Metric 2: Target value

## Timeline
- Start Date: YYYY-MM-DD
- Target Completion: YYYY-MM-DD

## Dependencies
- [ ] External dependency 1
```

## 🎯 Current Project Status

### **Phase 1: Core System Testing** (Current)
**Status**: In Progress  
**Issues**: 10+ testing tasks created  
**Timeline**: 2025-01-27 to 2025-02-03

#### Testing Tasks:
- [ ] Frontend component testing (VideoClips, Settings, Overlay)
- [ ] Backend plugin testing (UDP, OBS, Playback)
- [ ] Integration testing (Tauri commands)
- [ ] Performance validation
- [ ] User acceptance testing preparation

### **Phase 2: Windows Production Deployment**
**Status**: Planned  
**Issues**: 3+ deployment tasks created  
**Timeline**: 2025-02-03 to 2025-02-10

#### Deployment Tasks:
- [ ] Windows executable generation
- [ ] MSI installer creation
- [ ] Clean system testing
- [ ] Performance optimization

### **Phase 3: Future Enhancements**
**Status**: Backlog  
**Issues**: 6+ epic categories planned  
**Timeline**: 2025-02-10 onwards

#### Enhancement Epics:
- [ ] Video System Enhancements (20+ features)
- [ ] OBS Studio Integration (18+ features)
- [ ] PSS Protocol & Competition (18+ features)
- [ ] User Interface & Experience (18+ features)
- [ ] System & Performance (18+ features)
- [ ] Modern Platform Features (12+ features)

## 👥 Team Collaboration

### **Issue Assignment**
```bash
# Assign issue to team member
@username please take a look at this issue

# Multiple assignees
/assign @user1 @user2

# Self-assign
/assign @me
```

### **Linking Issues**
```markdown
# Reference related issues
Related to #123
Fixes #456
Closes #789

# Epic relationship
Part of epic #100
Blocked by #50
```

### **Status Updates**
```markdown
# Update progress
Currently working on the frontend implementation.
Expected completion: 2025-01-30

# Request review
@reviewer ready for review, please check the implementation

# Report blocking issue
Blocked by missing API endpoint #123
```

## 📈 Metrics and Reporting

### **Project Health Metrics**
- **Velocity** - Issues closed per week
- **Lead Time** - Time from creation to completion
- **Cycle Time** - Time from start to completion
- **Burndown** - Remaining work vs. time

### **Quality Metrics**
- **Bug Rate** - Bugs per feature
- **Rework Rate** - Issues reopened
- **Review Time** - Time in review status
- **Test Coverage** - Percentage of code tested

### **Team Metrics**
- **Workload Distribution** - Issues per team member
- **Collaboration Index** - Cross-team interactions
- **Knowledge Sharing** - Documentation contributions

## 🔍 Monitoring and Alerts

### **Automated Monitoring**
- **Stale Issues** - Issues inactive for >7 days
- **Blocked Items** - Issues blocked for >3 days
- **Review Backlog** - Items waiting for review >2 days
- **Epic Progress** - Epic completion percentage

### **Notification Settings**
```yaml
# Team notification preferences:
- Issue assignments → Immediate notification
- Status changes → Daily digest
- Epic updates → Weekly summary
- Critical bugs → Immediate alert
```

## 🚀 Advanced Features

### **Custom Views and Filters**
```bash
# Filter by labels
label:frontend label:priority:high

# Filter by status
is:open label:status:in-progress

# Filter by assignee
assignee:username is:open

# Filter by milestone
milestone:"Phase 1" is:open
```

### **Saved Searches**
- **My Issues**: `assignee:@me is:open`
- **High Priority**: `label:priority:high is:open`
- **Frontend Tasks**: `label:frontend label:task is:open`
- **Blocked Items**: `label:status:blocked is:open`

### **Integration with External Tools**
- **Slack Notifications** - Issue updates in team channels
- **Calendar Integration** - Milestones and deadlines
- **Time Tracking** - Integration with time tracking tools
- **Documentation** - Links to related documentation

## 🛠️ Maintenance and Best Practices

### **Weekly Maintenance**
- [ ] Review and update project board
- [ ] Close completed issues
- [ ] Update epic progress
- [ ] Triage new issues
- [ ] Update labels and priorities

### **Monthly Reviews**
- [ ] Analyze velocity and metrics
- [ ] Review workflow effectiveness
- [ ] Update automation rules
- [ ] Team retrospective
- [ ] Process improvements

### **Best Practices**
1. **Clear Titles** - Use descriptive, searchable titles
2. **Complete Descriptions** - Include all necessary context
3. **Proper Labels** - Apply appropriate labels immediately
4. **Link Related Work** - Connect issues, PRs, and documentation
5. **Regular Updates** - Keep status current
6. **Team Communication** - Use comments for collaboration

## 📚 Resources

### **GitHub Documentation**
- [Issues Guide](https://docs.github.com/en/issues)
- [Project Boards](https://docs.github.com/en/issues/organizing-your-work-with-project-boards)
- [GitHub Actions](https://docs.github.com/en/actions)
- [Labels Guide](https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/managing-labels)

### **Project-Specific Resources**
- [Issue Templates](../.github/ISSUE_TEMPLATE/)
- [Automation Scripts](../scripts/github/)
- [Workflow Configurations](../.github/workflows/)
- [Project Tracker Guide](./project-tracker-guide.md)

### **Tools and Extensions**
- **GitHub CLI** - Command-line interface for GitHub
- **GitHub Mobile** - Mobile app for notifications
- **Browser Extensions** - Enhanced GitHub interface
- **IDE Integrations** - GitHub integration in VS Code/Cursor

---

**📝 Note**: This integration guide provides comprehensive GitHub project management setup following industry best practices. The system is designed to scale with team growth and project complexity.

**🔄 Last Updated**: 2025-01-27  
**👤 Maintained by**: Development Team  
**✅ Status**: Production Ready 