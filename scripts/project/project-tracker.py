#!/usr/bin/env python3
"""
reStrike VTA Project Tracker Management Script

This script provides utilities for managing the project tracker using GitHub Issues
and the comprehensive feature request template.
"""

import json
import subprocess
import sys
from datetime import datetime
from typing import Dict, List, Optional, Any
import argparse
import os

class Colors:
    """ANSI color codes for terminal output"""
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    PURPLE = '\033[0;35m'
    CYAN = '\033[0;36m'
    WHITE = '\033[1;37m'
    NC = '\033[0m'  # No Color

class ProjectTracker:
    """Main project tracker management class"""
    
    def __init__(self):
        self.categories = {
            'core': 'Core Application',
            'obs': 'OBS Integration', 
            'udp': 'UDP Protocol',
            'video': 'Video Playback',
            'ui': 'UI/UX',
            'dev-tools': 'Development Tools',
            'docs': 'Documentation',
            'infrastructure': 'Infrastructure',
            'security': 'Security',
            'performance': 'Performance',
            'testing': 'Testing',
            'bug-fix': 'Bug Fix'
        }
        
        self.priorities = {
            'critical': 'Critical',
            'high': 'High',
            'medium': 'Medium',
            'low': 'Low',
            'future': 'Future'
        }
        
        self.statuses = {
            'backlog': 'Backlog',
            'planning': 'Planning',
            'in-progress': 'In Progress',
            'review': 'Review',
            'testing': 'Testing',
            'complete': 'Complete',
            'deployed': 'Deployed'
        }

    def run_gh_command(self, command: str) -> Dict[str, Any]:
        """Run a GitHub CLI command and return JSON output"""
        try:
            result = subprocess.run(
                f"gh {command} --json number,title,labels,assignees,state,createdAt,updatedAt",
                shell=True,
                capture_output=True,
                text=True,
                check=True
            )
            return json.loads(result.stdout)
        except subprocess.CalledProcessError as e:
            print(f"{Colors.RED}Error running GitHub CLI command: {e}{Colors.NC}")
            return []
        except json.JSONDecodeError as e:
            print(f"{Colors.RED}Error parsing JSON output: {e}{Colors.NC}")
            return []

    def list_issues(self, filters: Optional[Dict[str, str]] = None) -> List[Dict[str, Any]]:
        """List issues with optional filters"""
        command = "issue list"
        
        if filters:
            for key, value in filters.items():
                if key == 'label':
                    command += f" --label '{value}'"
                elif key == 'assignee':
                    command += f" --assignee '{value}'"
                elif key == 'state':
                    command += f" --state {value}"
                elif key == 'search':
                    command += f" --search '{value}'"
        
        return self.run_gh_command(command)

    def create_issue(self, title: str, body: str, labels: List[str] = None, assignees: List[str] = None) -> bool:
        """Create a new issue"""
        command = f"issue create --title '{title}' --body '{body}'"
        
        if labels:
            command += f" --label '{','.join(labels)}'"
        
        if assignees:
            command += f" --assignee '{','.join(assignees)}'"
        
        try:
            subprocess.run(command, shell=True, check=True)
            print(f"{Colors.GREEN}✅ Issue created successfully{Colors.NC}")
            return True
        except subprocess.CalledProcessError as e:
            print(f"{Colors.RED}❌ Error creating issue: {e}{Colors.NC}")
            return False

    def update_issue(self, issue_number: int, **kwargs) -> bool:
        """Update an existing issue"""
        command = f"issue edit {issue_number}"
        
        if 'title' in kwargs:
            command += f" --title '{kwargs['title']}'"
        
        if 'body' in kwargs:
            command += f" --body '{kwargs['body']}'"
        
        if 'add-label' in kwargs:
            command += f" --add-label '{kwargs['add-label']}'"
        
        if 'remove-label' in kwargs:
            command += f" --remove-label '{kwargs['remove-label']}'"
        
        if 'add-assignee' in kwargs:
            command += f" --add-assignee '{kwargs['add-assignee']}'"
        
        if 'remove-assignee' in kwargs:
            command += f" --remove-assignee '{kwargs['remove-assignee']}'"
        
        try:
            subprocess.run(command, shell=True, check=True)
            print(f"{Colors.GREEN}✅ Issue {issue_number} updated successfully{Colors.NC}")
            return True
        except subprocess.CalledProcessError as e:
            print(f"{Colors.RED}❌ Error updating issue {issue_number}: {e}{Colors.NC}")
            return False

    def show_summary(self) -> None:
        """Show a summary of all issues"""
        print(f"{Colors.CYAN}📊 Project Tracker Summary{Colors.NC}")
        print("=" * 50)
        
        # Get all issues
        issues = self.list_issues()
        
        if not issues:
            print(f"{Colors.YELLOW}No issues found{Colors.NC}")
            return
        
        # Count by priority
        priority_counts = {}
        for issue in issues:
            labels = [label['name'] for label in issue.get('labels', [])]
            for priority in self.priorities.keys():
                if f'priority-{priority}' in labels:
                    priority_counts[priority] = priority_counts.get(priority, 0) + 1
                    break
        
        # Count by status
        status_counts = {}
        for issue in issues:
            labels = [label['name'] for label in issue.get('labels', [])]
            for status in self.statuses.keys():
                if f'status-{status}' in labels:
                    status_counts[status] = status_counts.get(status, 0) + 1
                    break
        
        # Count by category
        category_counts = {}
        for issue in issues:
            labels = [label['name'] for label in issue.get('labels', [])]
            for category in self.categories.keys():
                if f'category-{category}' in labels:
                    category_counts[category] = category_counts.get(category, 0) + 1
                    break
        
        print(f"\n{Colors.BLUE}📈 Priority Distribution:{Colors.NC}")
        for priority, count in priority_counts.items():
            print(f"  {self.priorities[priority]}: {count}")
        
        print(f"\n{Colors.BLUE}📊 Status Distribution:{Colors.NC}")
        for status, count in status_counts.items():
            print(f"  {self.statuses[status]}: {count}")
        
        print(f"\n{Colors.BLUE}🏷️ Category Distribution:{Colors.NC}")
        for category, count in category_counts.items():
            print(f"  {self.categories[category]}: {count}")
        
        print(f"\n{Colors.BLUE}📋 Total Issues: {len(issues)}{Colors.NC}")

    def show_issues_by_priority(self, priority: str) -> None:
        """Show issues by priority level"""
        if priority not in self.priorities:
            print(f"{Colors.RED}Invalid priority: {priority}{Colors.NC}")
            return
        
        issues = self.list_issues({'label': f'priority-{priority}'})
        
        print(f"{Colors.CYAN}📋 Issues with Priority: {self.priorities[priority]}{Colors.NC}")
        print("=" * 60)
        
        for issue in issues:
            labels = [label['name'] for label in issue.get('labels', [])]
            status = "Unknown"
            category = "Unknown"
            
            for status_key in self.statuses.keys():
                if f'status-{status_key}' in labels:
                    status = self.statuses[status_key]
                    break
            
            for category_key in self.categories.keys():
                if f'category-{category_key}' in labels:
                    category = self.categories[category_key]
                    break
            
            print(f"\n{Colors.YELLOW}#{issue['number']}{Colors.NC} {issue['title']}")
            print(f"  Status: {status}")
            print(f"  Category: {category}")
            print(f"  Updated: {issue['updatedAt'][:10]}")

    def show_issues_by_status(self, status: str) -> None:
        """Show issues by status"""
        if status not in self.statuses:
            print(f"{Colors.RED}Invalid status: {status}{Colors.NC}")
            return
        
        issues = self.list_issues({'label': f'status-{status}'})
        
        print(f"{Colors.CYAN}📋 Issues with Status: {self.statuses[status]}{Colors.NC}")
        print("=" * 60)
        
        for issue in issues:
            labels = [label['name'] for label in issue.get('labels', [])]
            priority = "Unknown"
            category = "Unknown"
            
            for priority_key in self.priorities.keys():
                if f'priority-{priority_key}' in labels:
                    priority = self.priorities[priority_key]
                    break
            
            for category_key in self.categories.keys():
                if f'category-{category_key}' in labels:
                    category = self.categories[category_key]
                    break
            
            print(f"\n{Colors.YELLOW}#{issue['number']}{Colors.NC} {issue['title']}")
            print(f"  Priority: {priority}")
            print(f"  Category: {category}")
            print(f"  Updated: {issue['updatedAt'][:10]}")

    def generate_report(self, output_file: str = "project_report.md") -> None:
        """Generate a comprehensive project report"""
        issues = self.list_issues()
        
        report = f"""# reStrike VTA Project Report

Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

## Summary

Total Issues: {len(issues)}

## Issues by Priority

"""
        
        # Priority breakdown
        priority_counts = {}
        for issue in issues:
            labels = [label['name'] for label in issue.get('labels', [])]
            for priority in self.priorities.keys():
                if f'priority-{priority}' in labels:
                    priority_counts[priority] = priority_counts.get(priority, 0) + 1
                    break
        
        for priority, count in priority_counts.items():
            report += f"- {self.priorities[priority]}: {count}\n"
        
        report += "\n## Issues by Status\n\n"
        
        # Status breakdown
        status_counts = {}
        for issue in issues:
            labels = [label['name'] for label in issue.get('labels', [])]
            for status in self.statuses.keys():
                if f'status-{status}' in labels:
                    status_counts[status] = status_counts.get(status, 0) + 1
                    break
        
        for status, count in status_counts.items():
            report += f"- {self.statuses[status]}: {count}\n"
        
        report += "\n## Recent Issues\n\n"
        
        # Recent issues (last 10)
        recent_issues = sorted(issues, key=lambda x: x['updatedAt'], reverse=True)[:10]
        for issue in recent_issues:
            report += f"- #{issue['number']}: {issue['title']}\n"
        
        # Write report
        with open(output_file, 'w') as f:
            f.write(report)
        
        print(f"{Colors.GREEN}✅ Report generated: {output_file}{Colors.NC}")

    def create_feature_template(self, title: str, category: str, priority: str, description: str) -> str:
        """Create a feature request template"""
        if category not in self.categories:
            raise ValueError(f"Invalid category: {category}")
        
        if priority not in self.priorities:
            raise ValueError(f"Invalid priority: {priority}")
        
        template = f"""## 📋 Feature Request & Project Tracker

### 🎯 Feature Category
- [x] **{self.categories[category]}** - {category.replace('-', ' ').title()}

### 🚀 Priority Level
- [x] **{self.priorities[priority]}** - {priority.title()}

### 📊 Development Status
- [x] **Backlog** - Not yet started

---

## 🎯 Feature Description

### Problem Statement
{description}

### Proposed Solution
[Describe the proposed solution]

### User Story
**As a** [type of user]
**I want** [goal/desire]
**So that** [benefit/value]

### Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3
- [ ] Criterion 4

---

## 🔧 Technical Requirements

### Dependencies
- [ ] Dependency 1
- [ ] Dependency 2

### Implementation Details
- **Frontend**: [React component changes needed]
- **Backend**: [Rust/Tauri modifications required]
- **Database**: [Schema changes or new tables]
- **API**: [New endpoints or modifications]

### Files to Modify
- [ ] `ui/src/components/`
- [ ] `src/`
- [ ] `config/`
- [ ] `docs/`

---

## 📈 Success Metrics

### Key Performance Indicators
- [ ] Metric 1: Target value
- [ ] Metric 2: Target value

### Testing Requirements
- [ ] Unit tests
- [ ] Integration tests
- [ ] End-to-end tests

---

## 🗓️ Timeline

### Estimated Effort
- **Development**: X days/weeks
- **Testing**: X days/weeks
- **Documentation**: X days/weeks
- **Total**: X days/weeks

### Milestones
- [ ] **Design Complete**: [Date]
- [ ] **Development Start**: [Date]
- [ ] **MVP Ready**: [Date]
- [ ] **Testing Complete**: [Date]
- [ ] **Release Ready**: [Date]

---

## 🏷️ Labels & Metadata

### Labels
- `enhancement`
- `tracker`
- `priority-{priority}`
- `category-{category}`
- `status-backlog`

### Assignees
- @damjanZGB

---

**📋 Template Version**: 2.0
**🔄 Created**: {datetime.now().strftime('%Y-%m-%d')}
**👤 Created by**: Project Tracker Script
"""
        
        return template

def show_help():
    """Show help information"""
    help_text = f"""
{Colors.CYAN}reStrike VTA Project Tracker Management{Colors.NC}

{Colors.BLUE}Usage:{Colors.NC}
  python3 scripts/project_tracker.py <command> [options]

{Colors.BLUE}Commands:{Colors.NC}
  summary                    Show project summary
  priority <level>           Show issues by priority (critical, high, medium, low, future)
  status <status>            Show issues by status (backlog, planning, in-progress, review, testing, complete, deployed)
  report [output_file]       Generate project report (default: project_report.md)
  create <title> <category> <priority> <description>  Create new feature issue
  update <issue_number> <field> <value>  Update issue field

{Colors.BLUE}Categories:{Colors.NC}
  core, obs, udp, video, ui, dev-tools, docs, infrastructure, security, performance, testing, bug-fix

{Colors.BLUE}Priorities:{Colors.NC}
  critical, high, medium, low, future

{Colors.BLUE}Statuses:{Colors.NC}
  backlog, planning, in-progress, review, testing, complete, deployed

{Colors.BLUE}Examples:{Colors.NC}
  python3 scripts/project_tracker.py summary
  python3 scripts/project_tracker.py priority high
  python3 scripts/project_tracker.py status in-progress
  python3 scripts/project_tracker.py report my_report.md
  python3 scripts/project_tracker.py create "Instant Replay" video high "Implement instant replay system"
  python3 scripts/project_tracker.py update 123 add-label "status-in-progress"

{Colors.YELLOW}Note:{Colors.NC} Requires GitHub CLI (gh) to be installed and authenticated.
"""
    print(help_text)

def main():
    """Main function"""
    parser = argparse.ArgumentParser(description='reStrike VTA Project Tracker Management')
    parser.add_argument('command', help='Command to execute')
    parser.add_argument('args', nargs='*', help='Command arguments')
    
    args = parser.parse_args()
    
    tracker = ProjectTracker()
    
    if args.command == 'summary':
        tracker.show_summary()
    
    elif args.command == 'priority':
        if len(args.args) < 1:
            print(f"{Colors.RED}Error: Priority level required{Colors.NC}")
            sys.exit(1)
        tracker.show_issues_by_priority(args.args[0])
    
    elif args.command == 'status':
        if len(args.args) < 1:
            print(f"{Colors.RED}Error: Status required{Colors.NC}")
            sys.exit(1)
        tracker.show_issues_by_status(args.args[0])
    
    elif args.command == 'report':
        output_file = args.args[0] if args.args else 'project_report.md'
        tracker.generate_report(output_file)
    
    elif args.command == 'create':
        if len(args.args) < 4:
            print(f"{Colors.RED}Error: Title, category, priority, and description required{Colors.NC}")
            sys.exit(1)
        
        title = args.args[0]
        category = args.args[1]
        priority = args.args[2]
        description = ' '.join(args.args[3:])
        
        try:
            template = tracker.create_feature_template(title, category, priority, description)
            labels = [f'enhancement', 'tracker', f'priority-{priority}', f'category-{category}', 'status-backlog']
            tracker.create_issue(title, template, labels, ['damjanZGB'])
        except ValueError as e:
            print(f"{Colors.RED}Error: {e}{Colors.NC}")
            sys.exit(1)
    
    elif args.command == 'update':
        if len(args.args) < 3:
            print(f"{Colors.RED}Error: Issue number, field, and value required{Colors.NC}")
            sys.exit(1)
        
        issue_number = int(args.args[0])
        field = args.args[1]
        value = args.args[2]
        
        kwargs = {field: value}
        tracker.update_issue(issue_number, **kwargs)
    
    elif args.command == 'help':
        show_help()
    
    else:
        print(f"{Colors.RED}Unknown command: {args.command}{Colors.NC}")
        show_help()
        sys.exit(1)

if __name__ == '__main__':
    main() 