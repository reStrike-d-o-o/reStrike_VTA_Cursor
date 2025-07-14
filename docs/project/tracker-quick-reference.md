# Project Tracker Quick Reference

## 🚀 Quick Start

### Create a New Feature Issue
```bash
# Using the script
python3 scripts/project_tracker.py create "Feature Name" category priority "Description"

# Example
python3 scripts/project_tracker.py create "Instant Replay" video high "Implement instant replay system"
```

### View Project Summary
```bash
python3 scripts/project_tracker.py summary
```

### Generate Report
```bash
python3 scripts/project_tracker.py report my_report.md
```

## 📋 Categories

| Code | Category | Description |
|------|----------|-------------|
| `core` | Core Application | Main VTA functionality |
| `obs` | OBS Integration | OBS WebSocket and video control |
| `udp` | UDP Protocol | PSS data collection and parsing |
| `video` | Video Playback | mpv integration and replay system |
| `ui` | UI/UX | User interface and experience |
| `dev-tools` | Development Tools | Build, test, and development utilities |
| `docs` | Documentation | Guides, API docs, and tutorials |
| `infrastructure` | Infrastructure | Dev container, CI/CD, deployment |
| `security` | Security | Authentication, licensing, data protection |
| `performance` | Performance | Optimization and efficiency improvements |
| `testing` | Testing | Unit tests, integration tests, automation |
| `bug-fix` | Bug Fix | Issue resolution and stability improvements |

## 🎯 Priorities

| Priority | Timeframe | Description |
|----------|-----------|-------------|
| `critical` | Immediate | Blocking development or core functionality |
| `high` | 1-2 weeks | Important for next release |
| `medium` | 1-2 months | Nice to have, planned for future |
| `low` | 3+ months | Enhancement or optimization |
| `future` | 6+ months | Long-term roadmap item |

## 📊 Statuses

| Status | Description | Actions Required |
|--------|-------------|------------------|
| `backlog` | Not yet started | Planning and prioritization |
| `planning` | Requirements and design phase | Requirements gathering, design review |
| `in-progress` | Currently being developed | Development, regular updates |
| `review` | Code review and testing | Code review, testing |
| `testing` | QA and integration testing | QA testing, integration testing |
| `complete` | Ready for release | Final testing, documentation |
| `deployed` | Released and in production | Monitoring, feedback collection |

## 🔍 Common Commands

### View Issues by Priority
```bash
python3 scripts/project_tracker.py priority high
python3 scripts/project_tracker.py priority critical
```

### View Issues by Status
```bash
python3 scripts/project_tracker.py status in-progress
python3 scripts/project_tracker.py status backlog
```

### Update Issue Status
```bash
python3 scripts/project_tracker.py update 123 add-label "status-in-progress"
python3 scripts/project_tracker.py update 123 add-label "priority-high"
```

## 🏷️ Labels Reference

### Priority Labels
- `priority-critical`
- `priority-high`
- `priority-medium`
- `priority-low`
- `priority-future`

### Category Labels
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

### Status Labels
- `status-backlog`
- `status-planning`
- `status-in-progress`
- `status-review`
- `status-testing`
- `status-complete`
- `status-deployed`

### Other Labels
- `enhancement`
- `tracker`
- `bug`
- `documentation`

## 📈 GitHub Search Queries

### Priority Distribution
```
is:issue label:priority-high
is:issue label:priority-critical
```

### Category Breakdown
```
is:issue label:category-core
is:issue label:category-obs
```

### Status Overview
```
is:issue label:status-in-progress
is:issue label:status-complete
```

### Combined Filters
```
is:issue label:priority-high label:status-in-progress
is:issue label:category-video label:priority-critical
```

## 🔧 Workflow Integration

### Branch Naming
```bash
git checkout -b feature/123-instant-replay
git checkout -b bugfix/456-udp-parsing
```

### Commit Messages
```bash
git commit -m "feat: implement instant replay system (#123)"
git commit -m "fix: resolve UDP parsing issue (#456)"
```

### Pull Request Description
```markdown
## Related Issues
Closes #123
Related to #456

## Changes Made
- Implemented instant replay system
- Added video playback controls
- Updated documentation
```

## 📊 Weekly Workflow

### Monday - Planning
```bash
# Generate weekly report
python3 scripts/project_tracker.py report weekly_report.md

# Review high priority issues
python3 scripts/project_tracker.py priority high
```

### Daily - Updates
```bash
# Check in-progress issues
python3 scripts/project_tracker.py status in-progress

# Update issue status
python3 scripts/project_tracker.py update 123 add-label "status-review"
```

### Friday - Review
```bash
# Generate completion report
python3 scripts/project_tracker.py report completion_report.md

# Plan next week
python3 scripts/project_tracker.py status backlog
```

## 🚨 Troubleshooting

### GitHub CLI Not Found
```bash
# Install GitHub CLI
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update
sudo apt install gh

# Authenticate
gh auth login
```

### Permission Issues
```bash
# Check authentication
gh auth status

# Re-authenticate if needed
gh auth login
```

### Script Errors
```bash
# Check Python version
python3 --version

# Install dependencies
pip3 install argparse

# Check script permissions
ls -la scripts/project_tracker.py
chmod +x scripts/project_tracker.py
```

## 📞 Support

### Documentation
- [Project Tracker Guide](./PROJECT_TRACKER_GUIDE.md)
- [Feature Request Template](../.github/ISSUE_TEMPLATE/feature_request.md)
- [GitHub Issues Documentation](https://docs.github.com/en/issues)

### Help Commands
```bash
# Script help
python3 scripts/project_tracker.py help

# GitHub CLI help
gh issue --help
gh issue create --help
```

---

**📋 Quick Reference Version**: 1.0
**🔄 Last Updated**: 2025-01-27
**👤 Maintained by**: Development Team 