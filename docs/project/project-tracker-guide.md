# Project Tracker Guide

## Overview

The reStrike VTA project uses GitHub Issues with a comprehensive feature request template as a project tracker. This guide explains how to use the tracker effectively for project management and development coordination.

## 🎯 Tracker Structure

### Feature Categories

The tracker organizes features into logical categories:

| Category | Description | Examples |
|----------|-------------|----------|
| **Core Application** | Main VTA functionality | UDP server, data parsing, match management |
| **OBS Integration** | OBS WebSocket and video control | Scene switching, recording, replay buffer |
| **UDP Protocol** | PSS data collection and parsing | Protocol implementation, data validation |
| **Video Playback** | mpv integration and replay system | Video player, clip management, timeline |
| **UI/UX** | User interface and experience | React components, styling, animations |
| **Development Tools** | Build, test, and development utilities | Scripts, automation, debugging tools |
| **Documentation** | Guides, API docs, and tutorials | User guides, developer docs, API reference |
| **Infrastructure** | Dev container, CI/CD, deployment | Container setup, GitHub Actions, deployment |
| **Security** | Authentication, licensing, data protection | License system, data encryption, access control |
| **Performance** | Optimization and efficiency improvements | Caching, optimization, resource management |
| **Testing** | Unit tests, integration tests, automation | Test coverage, CI/CD testing, automation |
| **Bug Fix** | Issue resolution and stability improvements | Bug fixes, stability improvements |

### Priority Levels

| Priority | Description | Timeframe | Examples |
|----------|-------------|-----------|----------|
| **Critical** | Blocking development or core functionality | Immediate | Build failures, security vulnerabilities |
| **High** | Important for next release | 1-2 weeks | Core features, major improvements |
| **Medium** | Nice to have, planned for future | 1-2 months | Enhancements, optimizations |
| **Low** | Enhancement or optimization | 3+ months | UI polish, minor features |
| **Future** | Long-term roadmap item | 6+ months | Major architectural changes |

### Development Status

| Status | Description | Actions Required |
|--------|-------------|------------------|
| **Backlog** | Not yet started | Planning and prioritization |
| **Planning** | Requirements and design phase | Requirements gathering, design review |
| **In Progress** | Currently being developed | Development, regular updates |
| **Review** | Code review and testing | Code review, testing |
| **Testing** | QA and integration testing | QA testing, integration testing |
| **Complete** | Ready for release | Final testing, documentation |
| **Deployed** | Released and in production | Monitoring, feedback collection |

## 📋 Using the Tracker

### Creating a New Feature Request

1. **Go to GitHub Issues**
   - Navigate to the project's Issues tab
   - Click "New Issue"

2. **Select Template**
   - Choose "Feature Request & Project Tracker"
   - The template will be pre-filled

3. **Fill Out the Template**
   - Select appropriate categories and priorities
   - Provide detailed description and requirements
   - Add acceptance criteria and timeline

4. **Add Labels and Assignees**
   - Use appropriate labels for filtering
   - Assign to team members

### Example Feature Request

```markdown
## 🎯 Feature Description

### Problem Statement
Referees need to quickly access the most recent recording when a challenge is raised during a match.

### Proposed Solution
Implement an instant replay system that automatically plays the most recent 20-second clip when triggered.

### User Story
**As a** taekwondo referee
**I want** the most recent recording to play instantly when a challenge is raised
**So that** I can review the incident within the 20-second time limit

### Acceptance Criteria
- [ ] System automatically detects challenge events from UDP
- [ ] Most recent 20-second clip plays immediately
- [ ] Video player hides main UI during playback
- [ ] UI restores automatically after playback ends
```

## 🔍 Tracking and Management

### Issue Labels

Use these labels for effective filtering and organization:

#### Priority Labels
- `priority-critical`
- `priority-high`
- `priority-medium`
- `priority-low`
- `priority-future`

#### Category Labels
- `category-core`
- `category-obs`
- `category-udp`
- `category-video`
- `category-ui`
- `category-dev-tools`
- `category-docs`
- `category-infrastructure`
- `category-security`
- `category-performance`
- `category-testing`
- `category-bug-fix`

#### Status Labels
- `status-backlog`
- `status-planning`
- `status-in-progress`
- `status-review`
- `status-testing`
- `status-complete`
- `status-deployed`

### GitHub Projects Integration

Create GitHub Projects boards for different views:

#### 1. Roadmap Board
- **Columns**: Backlog, Planning, In Progress, Review, Testing, Complete
- **Purpose**: High-level project roadmap view

#### 2. Sprint Board
- **Columns**: To Do, In Progress, Review, Done
- **Purpose**: Current sprint management

#### 3. Release Board
- **Columns**: Planned, In Development, Testing, Ready for Release
- **Purpose**: Release planning and tracking

### Automated Workflows

Set up GitHub Actions for automated tracking:

```yaml
# .github/workflows/issue-tracker.yml
name: Issue Tracker Automation
on:
  issues:
    types: [opened, edited, labeled, unlabeled]

jobs:
  update-tracker:
    runs-on: ubuntu-latest
    steps:
      - name: Update Project Board
        uses: actions/add-to-project@v0.5.0
        with:
          project-url: ${{ secrets.PROJECT_URL }}
          github-token: ${{ secrets.GITHUB_TOKEN }}
```

## 📊 Reporting and Analytics

### Issue Queries

Use GitHub's search functionality for reporting:

#### Priority Distribution
```
is:issue label:priority-high
is:issue label:priority-critical
```

#### Category Breakdown
```
is:issue label:category-core
is:issue label:category-obs
```

#### Status Overview
```
is:issue label:status-in-progress
is:issue label:status-complete
```

### Weekly Reports

Generate weekly status reports:

```bash
# scripts/generate_weekly_report.sh
#!/bin/bash

echo "# Weekly Project Status Report"
echo "Generated: $(date)"
echo ""

echo "## Issues by Priority"
echo "- Critical: $(gh issue list --label "priority-critical" --json number | jq length)"
echo "- High: $(gh issue list --label "priority-high" --json number | jq length)"
echo "- Medium: $(gh issue list --label "priority-medium" --json number | jq length)"

echo "## Issues by Status"
echo "- In Progress: $(gh issue list --label "status-in-progress" --json number | jq length)"
echo "- Review: $(gh issue list --label "status-review" --json number | jq length)"
echo "- Testing: $(gh issue list --label "status-testing" --json number | jq length)"
```

## 🎯 Best Practices

### Issue Management

1. **Keep Issues Focused**
   - One feature per issue
   - Break large features into smaller issues
   - Use dependencies to link related issues

2. **Regular Updates**
   - Update status weekly
   - Add progress comments
   - Update timeline estimates

3. **Clear Communication**
   - Use consistent terminology
   - Provide context and background
   - Include relevant links and references

### Workflow Integration

1. **Branch Naming**
   ```bash
   # Use issue numbers in branch names
   git checkout -b feature/123-instant-replay
   git checkout -b bugfix/456-udp-parsing
   ```

2. **Commit Messages**
   ```bash
   # Reference issues in commit messages
   git commit -m "feat: implement instant replay system (#123)"
   git commit -m "fix: resolve UDP parsing issue (#456)"
   ```

3. **Pull Request Integration**
   ```markdown
   ## Related Issues
   Closes #123
   Related to #456
   ```

### Team Coordination

1. **Daily Standups**
   - Review in-progress issues
   - Update status and blockers
   - Plan next steps

2. **Weekly Reviews**
   - Review completed issues
   - Plan upcoming work
   - Adjust priorities

3. **Sprint Planning**
   - Select issues for next sprint
   - Estimate effort and timeline
   - Assign team members

## 🔧 Tools and Integrations

### GitHub CLI

Use GitHub CLI for command-line issue management:

```bash
# List issues by priority
gh issue list --label "priority-high"

# Create new issue
gh issue create --title "Feature: Instant Replay" --body "$(cat template.md)"

# Update issue status
gh issue edit 123 --add-label "status-in-progress"
```

### External Integrations

#### Slack Integration
- Notify team of issue updates
- Daily status summaries
- Sprint planning reminders

#### Jira Integration
- Sync issues with Jira
- Import/export functionality
- Cross-platform tracking

#### Notion Integration
- Export issue data to Notion
- Create project documentation
- Track meeting notes

## 📈 Metrics and KPIs

### Key Metrics

1. **Velocity**
   - Issues completed per sprint
   - Story points delivered
   - Cycle time per issue

2. **Quality**
   - Issues reopened
   - Bug vs feature ratio
   - Test coverage

3. **Timeline**
   - On-time delivery rate
   - Estimation accuracy
   - Blocked time

### Dashboard Creation

Create a project dashboard:

```html
<!-- docs/dashboard.html -->
<!DOCTYPE html>
<html>
<head>
    <title>reStrike VTA Project Dashboard</title>
</head>
<body>
    <h1>Project Status Dashboard</h1>
    <div id="metrics">
        <!-- Auto-populated with GitHub API data -->
    </div>
</body>
</html>
```

## 🚀 Advanced Features

### Custom Fields

Add custom fields to issues:

```yaml
# .github/ISSUE_TEMPLATE/custom-fields.yml
custom_fields:
  - name: "Story Points"
    type: "number"
    required: false
  - name: "Sprint"
    type: "text"
    required: false
  - name: "Epic"
    type: "text"
    required: false
```

### Automation Scripts

Create automation scripts for common tasks:

```python
# scripts/issue_automation.py
import github
import json

def update_issue_status(issue_number, new_status):
    """Update issue status and add appropriate labels"""
    pass

def generate_sprint_report():
    """Generate sprint completion report"""
    pass

def assign_issues_to_sprint():
    """Assign issues to current sprint"""
    pass
```

## 📚 Resources

### Documentation
- [GitHub Issues Documentation](https://docs.github.com/en/issues)
- [GitHub Projects Guide](https://docs.github.com/en/issues/organizing-your-work-with-project-boards)
- [GitHub CLI Reference](https://cli.github.com/manual/)

### Templates
- [Issue Templates](https://github.com/actions/starter-workflows)
- [Project Templates](https://github.com/github/roadmap)
- [Workflow Templates](https://github.com/actions/starter-workflows)

### Community
- [GitHub Community](https://github.community/)
- [Stack Overflow](https://stackoverflow.com/questions/tagged/github)
- [Reddit r/github](https://www.reddit.com/r/github/)

---

**📋 Template Version**: 2.0
**🔄 Last Updated**: 2025-01-27
**👤 Maintained by**: Development Team 