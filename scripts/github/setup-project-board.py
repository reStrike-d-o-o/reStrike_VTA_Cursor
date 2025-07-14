#!/usr/bin/env python3
"""
GitHub Project Board Setup
Sets up a GitHub Project Board with proper columns and automation for the reStrike VTA project.
"""

import json
import os
import sys
import requests
from typing import Dict, List, Any

class GitHubProjectSetup:
    def __init__(self, repo_owner: str, repo_name: str, token: str):
        self.repo_owner = repo_owner
        self.repo_name = repo_name
        self.token = token
        self.api_base = "https://api.github.com"
        self.headers = {
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28"
        }

    def create_project_board(self, name: str, description: str) -> Dict[str, Any]:
        """Create a GitHub Project Board (Classic)"""
        url = f"{self.api_base}/repos/{self.repo_owner}/{self.repo_name}/projects"
        
        data = {
            "name": name,
            "body": description
        }
        
        response = requests.post(url, headers=self.headers, json=data)
        
        if response.status_code == 201:
            return response.json()
        else:
            print(f"❌ Failed to create project board: {response.status_code} - {response.text}")
            return None

    def create_project_column(self, project_id: int, name: str) -> Dict[str, Any]:
        """Create a column in a project board"""
        url = f"{self.api_base}/projects/{project_id}/columns"
        
        data = {"name": name}
        
        response = requests.post(url, headers=self.headers, json=data)
        
        if response.status_code == 201:
            return response.json()
        else:
            print(f"❌ Failed to create column '{name}': {response.status_code} - {response.text}")
            return None

    def setup_restrike_project(self) -> None:
        """Set up the complete reStrike VTA project board"""
        print("🚀 Setting up reStrike VTA Project Board...")
        
        # Create main project board
        project_name = "reStrike VTA Development"
        project_description = """
## reStrike VTA Development Project Board

Track the development progress of the reStrike VTA Windows desktop application for taekwondo referees.

### Project Status: 98% Complete - Production Ready

#### Current Phases:
- **Phase 1**: Core System Testing (Immediate Priority)
- **Phase 2**: Windows Production Deployment
- **Phase 3**: Future Enhancements

#### Key Features:
- 🎯 Instant Video Replay with 10-second buffer
- 📡 Real-time Competition Monitoring via PSS protocol
- 🎥 OBS Studio Integration for recording/streaming
- 🤖 Automated Highlight Generation

#### Architecture:
- **Frontend**: React 18 + TypeScript + Zustand + Tailwind CSS
- **Backend**: Rust + Tauri framework
- **Target**: Windows 10/11 desktop application

### Column Workflow:
1. **Backlog** - Future planned tasks and enhancements
2. **Todo** - Ready to start, dependencies met
3. **In Progress** - Currently being worked on
4. **Review** - Awaiting review or testing
5. **Done** - Completed tasks
6. **Blocked** - Blocked by external dependencies

### Labels:
- **Type**: task, epic, bug, enhancement
- **Component**: frontend, backend, integration, testing
- **Priority**: high, medium, low
- **Phase**: testing, production, enhancement
- **Effort**: small (<4h), medium (4-16h), large (16h+)
        """.strip()
        
        project = self.create_project_board(project_name, project_description)
        
        if not project:
            print("❌ Failed to create project board")
            return
        
        print(f"✅ Created project board: {project['name']} (#{project['number']})")
        project_id = project['id']
        
        # Create columns
        columns = [
            "🗂️ Backlog",
            "📋 Todo", 
            "🔄 In Progress",
            "👀 Review",
            "✅ Done",
            "🚫 Blocked"
        ]
        
        created_columns = []
        for column_name in columns:
            print(f"Creating column: {column_name}")
            column = self.create_project_column(project_id, column_name)
            if column:
                created_columns.append(column)
                print(f"✅ Created column: {column['name']}")
            else:
                print(f"❌ Failed to create column: {column_name}")
        
        print(f"\n🎉 Project board setup complete!")
        print(f"📊 Created {len(created_columns)} columns")
        print(f"🔗 Project URL: {project['html_url']}")
        
        # Display setup instructions
        self.display_setup_instructions(project)

    def display_setup_instructions(self, project: Dict[str, Any]) -> None:
        """Display post-setup instructions"""
        print(f"""
📋 **Next Steps for Project Board Setup:**

1. **Configure Project Board Automation:**
   - Go to: {project['html_url']}/settings
   - Enable automation for columns:
     • "In Progress" → Auto-move when issue assigned
     • "Review" → Auto-move when PR opened
     • "Done" → Auto-move when issue closed
     • "Blocked" → Manual management

2. **Set up Personal Access Token (PAT):**
   - Go to: https://github.com/settings/tokens
   - Create a new token with these scopes:
     • repo (Full control of private repositories)
     • project (Full control of projects)
   - Add to repository secrets as PAT_TOKEN

3. **Configure Workflows:**
   - The issue-management.yml workflow will auto-assign issues
   - The project-board.yml workflow will auto-update status
   - Update project URL in workflows if needed

4. **Project Board Features:**
   - Drag and drop issues between columns
   - Filter by labels, assignees, milestones
   - Track progress with built-in metrics
   - Link issues to pull requests automatically

5. **Team Access:**
   - Invite team members to repository
   - Assign appropriate permissions
   - Set up notification preferences

🔗 **Useful Links:**
   • Project Board: {project['html_url']}
   • Issues: https://github.com/{self.repo_owner}/{self.repo_name}/issues
   • Repository: https://github.com/{self.repo_owner}/{self.repo_name}

📈 **Current Project Status:**
   • Application: 98% Complete - Production Ready
   • Frontend: 1,691 lines (5 React components)
   • Backend: 1,663 lines (3 core plugins)
   • Next Phase: Core System Testing

✨ **Pro Tips:**
   • Use issue templates for consistent formatting
   • Apply labels immediately for better organization
   • Link related issues using #issue_number
   • Update project status regularly
   • Use milestones for release planning
        """)

def main():
    """Main function to set up GitHub project board"""
    # Get configuration from environment
    repo_owner = os.getenv('GITHUB_REPO_OWNER', 'damjanZGB')
    repo_name = os.getenv('GITHUB_REPO_NAME', 'reStrike_VTA_Cursor')
    token = os.getenv('GITHUB_TOKEN')
    
    if not token:
        print("❌ Error: GITHUB_TOKEN environment variable not set")
        print("Please set GITHUB_TOKEN with a valid GitHub Personal Access Token")
        print("The token needs 'repo' and 'project' scopes for full functionality")
        sys.exit(1)
    
    # Set up project board
    setup = GitHubProjectSetup(repo_owner, repo_name, token)
    setup.setup_restrike_project()

if __name__ == "__main__":
    main() 