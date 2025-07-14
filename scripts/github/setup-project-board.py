#!/usr/bin/env python3
"""
GitHub Project Board Setup for reStrike VTA
Creates and configures GitHub project board with automation for comprehensive project management.
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
            "Accept": "application/vnd.github.v3+json",
            "X-GitHub-Api-Version": "2022-11-28"
        }

    def create_project_board(self, name: str, description: str) -> Dict[str, Any]:
        """Create a GitHub project board"""
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
        """Create a column in the project board"""
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
        print("🚀 Setting up reStrike VTA GitHub Project Board...")
        print("=" * 80)
        
        # Create main project board
        project_description = """
# reStrike VTA - Complete Project Management Board

## 🎯 Project Overview
reStrike VTA is a **98% complete** Windows desktop application for taekwondo referees providing instant video replay, real-time competition monitoring, and professional OBS integration.

## 📊 Current Status: Production Ready
- **Frontend**: ✅ COMPLETE - React 18 with 1,691 lines (5 components)
- **Backend**: ✅ COMPLETE - Rust with 1,663 lines (3 core plugins) 
- **Integration**: ✅ COMPLETE - Tauri Windows desktop app
- **Architecture**: ✅ COMPLETE - Scalable, maintainable structure

## 🎯 Immediate Priorities (2% Remaining)
1. **Core System Testing** - Verify all implemented features
2. **Windows Build** - Generate production .exe and MSI installer
3. **Real-World Testing** - Test with competition equipment
4. **Documentation** - User manual and deployment guide

## 🗺️ Development Phases
- **Phase 1**: Core System Testing (Immediate - 1-2 weeks)
- **Phase 2**: Windows Production Deployment (Next - 2-4 weeks)
- **Phase 3**: Future Enhancements (Long-term - 3+ months)

## 📋 Epic Categories
1. **Video System Enhancements** - Advanced playback & management (20+ features)
2. **OBS Studio Integration** - Professional broadcasting (18+ features)
3. **PSS Protocol & Competition** - Real-time analytics (18+ features)
4. **UI/UX Enhancements** - Modern platform features (18+ features)
5. **System & Performance** - Enterprise capabilities (18+ features)
6. **Modern Platform Features** - Cloud & AI integration (12+ features)

## 🏷️ Label System
- **Priority**: critical, high, medium, low, future
- **Component**: frontend, backend, integration, testing, ci/cd, infrastructure
- **Type**: task, bug, enhancement, epic, documentation
- **Phase**: testing, production, enhancement
- **Effort**: small (<4h), medium (4-16h), large (16h+)

## 🔄 Workflow Automation
This board uses GitHub Actions for automated issue management, project board updates, and status synchronization.

---
**Last Updated**: Latest session - Complete application stack operational
**Project Status**: 98% Complete - Production Testing Phase
**Next Milestone**: Windows executable generation and real-world testing
        """
        
        project = self.create_project_board(
            name="reStrike VTA - Complete Project Management",
            description=project_description.strip()
        )
        
        if not project:
            print("❌ Failed to create project board")
            return
        
        project_id = project['id']
        project_url = project['html_url']
        
        print(f"✅ Created project board: {project['name']}")
        print(f"   📋 URL: {project_url}")
        print(f"   🆔 ID: {project_id}")
        
        # Create columns with comprehensive workflow
        columns = [
            ("🗂️ Backlog", "Future planned tasks and long-term enhancements"),
            ("📋 Todo", "Ready to start - dependencies met, requirements clear"),
            ("🔄 In Progress", "Currently being worked on - limit work in progress"),
            ("👀 Review", "Code review, testing, or stakeholder approval needed"),
            ("✅ Done", "Completed and deployed - ready for closure"),
            ("🚫 Blocked", "Blocked by external dependencies or issues")
        ]
        
        print(f"\n📋 Creating project columns...")
        created_columns = []
        
        for column_name, column_description in columns:
            print(f"   📄 Creating column: {column_name}")
            column = self.create_project_column(project_id, column_name)
            if column:
                created_columns.append(column)
                print(f"      ✅ Created: {column['name']} (ID: {column['id']})")
            else:
                print(f"      ❌ Failed to create column: {column_name}")
        
        print(f"\n📊 Project Board Setup Summary:")
        print(f"   📋 Project: {project['name']}")
        print(f"   📄 Columns: {len(created_columns)}/{len(columns)} created")
        print(f"   🔗 URL: {project_url}")
        
        # Display column workflow information
        print(f"\n🔄 Column Workflow:")
        workflow_descriptions = {
            "🗂️ Backlog": "Long-term planning and future enhancements",
            "📋 Todo": "Immediate tasks ready for development",
            "🔄 In Progress": "Active development (limit WIP to 3-5 items)",
            "👀 Review": "Awaiting code review, testing, or approval", 
            "✅ Done": "Completed, tested, and ready for deployment",
            "🚫 Blocked": "Waiting for external dependencies or decisions"
        }
        
        for column_name, description in workflow_descriptions.items():
            print(f"   {column_name}: {description}")
        
        # Setup instructions
        self.display_setup_instructions(project)

    def display_setup_instructions(self, project: Dict[str, Any]) -> None:
        """Display setup instructions for the project board"""
        print(f"\n" + "=" * 80)
        print(f"🎉 Project Board Setup Complete!")
        print(f"=" * 80)
        
        print(f"\n📋 **Project Board Information:**")
        print(f"   📍 Name: {project['name']}")
        print(f"   🔗 URL: {project['html_url']}")
        print(f"   📝 Description: Comprehensive project management for reStrike VTA")
        
        print(f"\n🚀 **Next Steps:**")
        print(f"   1. **Add Issues to Board**: Use automated GitHub Actions or manually add issues")
        print(f"   2. **Configure Automation**: Set up GitHub Actions workflows for auto-management")
        print(f"   3. **Set Milestones**: Create milestones for major releases and phases")
        print(f"   4. **Assign Team Members**: Add collaborators and assign ownership")
        print(f"   5. **Start Development**: Begin with high-priority testing tasks")
        
        print(f"\n🔧 **Automation Setup:**")
        print(f"   📁 Workflow Files: .github/workflows/issue-management.yml")
        print(f"   📁 Project Board: .github/workflows/project-board.yml") 
        print(f"   🏷️ Labels: .github/labels.yml")
        print(f"   📋 Templates: .github/ISSUE_TEMPLATE/")
        
        print(f"\n📊 **Current Project Status:**")
        print(f"   ✅ Application: 98% Complete - Production Ready")
        print(f"   🎯 Immediate: Core system testing and Windows build")
        print(f"   📈 Future: 6 major enhancement categories with 100+ features")
        print(f"   🏆 Goal: Professional taekwondo referee toolkit")
        
        print(f"\n🎯 **Priority Task Categories:**")
        priorities = [
            ("🚨 Critical", "Immediate testing and production deployment"),
            ("🔥 High", "Core system validation and Windows build"),
            ("📈 Medium", "Future enhancement epics and planning"),
            ("📋 Low", "Documentation and optimization tasks"),
            ("🔮 Future", "Long-term platform and AI features")
        ]
        
        for priority, description in priorities:
            print(f"   {priority}: {description}")
        
        print(f"\n📚 **Documentation Resources:**")
        print(f"   📖 PROJECT_CONTEXT.md - Master project overview")
        print(f"   📋 docs/README.md - Complete documentation index")
        print(f"   🔧 docs/project/github-integration-guide.md - GitHub setup guide")
        print(f"   ⚡ .cursor/rules/context.mdc - Latest development status")
        
        print(f"\n🔗 **Useful Commands:**")
        print(f"   # Create issues from documentation")
        print(f"   python3 scripts/github/create-issues.py")
        print(f"   ")
        print(f"   # Set up GitHub CLI")
        print(f"   gh auth login")
        print(f"   gh repo view {self.repo_owner}/{self.repo_name}")
        print(f"   ")
        print(f"   # View project board")
        print(f"   gh project list --owner {self.repo_owner}")
        
        print(f"\n⚠️ **Important Notes:**")
        print(f"   • Project is 98% complete - focus on testing and deployment")
        print(f"   • All major features implemented - 1,691 frontend + 1,663 backend lines")
        print(f"   • Windows desktop application - production ready architecture")
        print(f"   • Comprehensive enhancement roadmap - 6 major categories planned")
        print(f"   • Professional project management - following industry best practices")
        
        print(f"\n🎉 **Success Metrics:**")
        print(f"   📊 Technical: <50ms UI response, 99.9% uptime, 4K@60fps video")
        print(f"   👥 User: <5min setup, >80% feature adoption, >4.5/5 rating")
        print(f"   🏆 Business: >50% competition coverage, >1000 active users")
        
        print(f"\n" + "=" * 80)
        print(f"Ready for professional taekwondo competition management! 🥋✨")
        print(f"=" * 80)

def main():
    """Main function to set up GitHub project board"""
    # Configuration
    repo_owner = "damjanZGB"  # Replace with actual repo owner
    repo_name = "reStrike_VTA_Cursor"  # Replace with actual repo name
    
    # Get GitHub token from environment
    token = os.getenv("GITHUB_TOKEN")
    if not token:
        print("❌ Error: GITHUB_TOKEN environment variable not set")
        print("   Please set your GitHub Personal Access Token:")
        print("   export GITHUB_TOKEN=your_token_here")
        print("   ")
        print("   Token requirements:")
        print("   • repo scope (full repository access)")
        print("   • project scope (project management)")
        print("   • workflow scope (GitHub Actions)")
        sys.exit(1)
    
    # Set up project board
    setup = GitHubProjectSetup(repo_owner, repo_name, token)
    setup.setup_restrike_project()

if __name__ == "__main__":
    main() 