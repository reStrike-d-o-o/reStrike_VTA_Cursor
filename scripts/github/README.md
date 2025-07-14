# GitHub Automation Scripts

This directory contains scripts for automating GitHub project management and issue tracking for the reStrike VTA project.

## Scripts Overview

### 🎯 `create-issues.py`
Automatically creates GitHub issues from current todos using the GitHub API.

**Features:**
- Creates testing phase tasks from current todos
- Generates production deployment tasks
- Creates epic issues for major feature categories
- Applies appropriate labels and categorization
- Follows GitHub issue templates

**Usage:**
```bash
export GITHUB_TOKEN="your_github_token"
export GITHUB_REPO_OWNER="damjanZGB"
export GITHUB_REPO_NAME="reStrike_VTA_Cursor"
python3 scripts/github/create-issues.py
```

### 📋 `setup-project-board.py`
Sets up a GitHub Project Board with proper columns and automation.

**Features:**
- Creates project board with professional description
- Sets up workflow columns (Backlog, Todo, In Progress, Review, Done, Blocked)
- Provides setup instructions for automation
- Configures team access and permissions guidance

**Usage:**
```bash
export GITHUB_TOKEN="your_github_token"
python3 scripts/github/setup-project-board.py
```

## Prerequisites

### 1. GitHub Personal Access Token (PAT)
Create a PAT with the following scopes:
- `repo` - Full control of private repositories
- `project` - Full control of projects
- `workflow` - Update GitHub Action workflows

**Steps:**
1. Go to [GitHub Settings > Tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Select the required scopes
4. Copy the token and set as environment variable

### 2. Python Dependencies
```bash
pip install requests
```

### 3. Environment Variables
```bash
export GITHUB_TOKEN="ghp_your_token_here"
export GITHUB_REPO_OWNER="damjanZGB"
export GITHUB_REPO_NAME="reStrike_VTA_Cursor"
```

## GitHub Integration Features

### 🏷️ **Automated Labeling**
Issues are automatically labeled based on:
- **Type**: `[TASK]`, `[EPIC]`, `[BUG]`, `[FEATURE]`
- **Component**: `frontend`, `backend`, `integration`, `testing`
- **Priority**: `priority:high`, `priority:medium`, `priority:low`
- **Phase**: `phase:testing`, `phase:production`, `phase:enhancement`
- **Effort**: `effort:small`, `effort:medium`, `effort:large`

### 📊 **Project Board Integration**
- Automatic addition of new issues to project board
- Status updates based on label changes
- Column automation for workflow management
- Progress tracking and metrics

### 🔄 **Workflow Automation**
- Auto-labeling based on issue title and content
- Project board status synchronization
- Epic validation and formatting
- Issue closure on completion

## Issue Templates Used

### Task Template
```markdown
[TASK] Component Name - Brief Description

## Task Description
Clear description of what needs to be done.

## Acceptance Criteria
- [ ] Criteria 1
- [ ] Criteria 2

## Priority & Effort
- Priority: High/Medium/Low
- Effort: Small/Medium/Large

## Dependencies
- [ ] Issue #XXX

## Definition of Done
- [ ] Implementation complete
- [ ] Tests passing
- [ ] Documentation updated
```

### Epic Template
```markdown
[EPIC] Feature Category Name

## Epic Overview
High-level description of the feature set.

## Business Value
Why this epic is important.

## User Stories
- As a [user], I want [goal] so that [benefit]

## Success Metrics
How we measure success.

## Timeline
Target dates and milestones.
```

## Project Board Structure

### Columns:
1. **🗂️ Backlog** - Future planned tasks
2. **📋 Todo** - Ready to start
3. **🔄 In Progress** - Currently being worked
4. **👀 Review** - Awaiting review/testing
5. **✅ Done** - Completed tasks
6. **🚫 Blocked** - Blocked by dependencies

### Automation Rules:
- New issues → Automatically added to "Todo"
- Assigned issues → Move to "In Progress"
- Pull request opened → Move to "Review"
- Issue closed → Move to "Done"
- Blocked label → Move to "Blocked"

## Usage Examples

### Create All Issues from Current Todos
```bash
# Set environment variables
export GITHUB_TOKEN="your_token"

# Run the issue creation script
python3 scripts/github/create-issues.py

# Expected output:
# 🚀 Creating GitHub Issues from Todos...
# 📋 Creating Testing Phase Tasks...
# ✅ Created issue #1: [TASK] Test VideoClips Component
# ✅ Created issue #2: [TASK] Test Settings Component
# ...
# 🎉 Successfully created 15 GitHub issues!
```

### Setup Project Board
```bash
# Run the project board setup
python3 scripts/github/setup-project-board.py

# Expected output:
# 🚀 Setting up reStrike VTA Project Board...
# ✅ Created project board: reStrike VTA Development (#1)
# ✅ Created column: 🗂️ Backlog
# ✅ Created column: 📋 Todo
# ...
# 🎉 Project board setup complete!
```

## Integration with CI/CD

The scripts work together with GitHub Actions workflows:

### `.github/workflows/issue-management.yml`
- Auto-labels new issues
- Validates epic format
- Updates project status

### `.github/workflows/project-board.yml`
- Adds issues to project board
- Updates column status based on labels
- Closes completed issues

### `.github/workflows/labels.yml`
- Syncs label configuration
- Maintains consistent labeling

## Best Practices

### 1. **Consistent Issue Creation**
- Use the provided templates
- Apply appropriate labels immediately
- Link related issues using #issue_number
- Set realistic effort estimates

### 2. **Project Board Management**
- Update issue status regularly
- Move issues between columns as work progresses
- Use comments to track progress
- Close issues when truly complete

### 3. **Epic Management**
- Break large epics into manageable tasks
- Update epic description with progress
- Link all related tasks to epics
- Track success metrics

### 4. **Team Collaboration**
- Assign issues to specific team members
- Use mentions (@username) for communication
- Review and comment on others' work
- Keep project board current

## Troubleshooting

### Common Issues:

**Token Permissions Error:**
```
❌ Failed to create issue: 403 - Forbidden
```
Solution: Ensure PAT has `repo` and `project` scopes.

**Rate Limiting:**
```
❌ Failed to create issue: 403 - API rate limit exceeded
```
Solution: Wait for rate limit reset or use authenticated requests.

**Project Board Not Found:**
```
❌ Failed to add to project board
```
Solution: Update project URL in workflows and scripts.

### Getting Help:
- Check GitHub API documentation
- Verify token permissions
- Review repository settings
- Test with minimal examples

---

**📝 Note**: These scripts implement GitHub best practices for project management and issue tracking, providing professional-grade project organization for the reStrike VTA development.

**🔄 Last Updated**: 2025-01-27
**👤 Maintained by**: Development Team
**✅ Status**: Production Ready 