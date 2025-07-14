# GitHub Project Board Automation Setup Guide

## 🤖 **AUTOMATION CONFIGURATION**

This guide provides step-by-step instructions for configuring automated status synchronization between GitHub issues and the **new GitHub Projects** (Projects v2).

---

## 📋 **OVERVIEW**

### **Project Board**: https://github.com/users/damjanZGB/projects/3
### **Repository**: https://github.com/damjanZGB/reStrike_VTA_Cursor

The automation system uses:
- **GitHub Labels** for status tracking
- **New GitHub Projects built-in automation** for column management
- **GitHub Actions** for advanced workflow automation
- **Automatic Status Sync** between labels and project fields

---

## ⚙️ **STEP 1: CONFIGURE NEW GITHUB PROJECTS AUTOMATION**

### **Access Project Settings**
1. **Navigate to Project Board**: https://github.com/users/damjanZGB/projects/3
2. **Click "⚙️" (Settings)** in the top-right corner of the project board
3. **Select "Workflows" tab** from the left sidebar

### **Configure Built-in Workflows**

Unlike classic projects, the new GitHub Projects use **built-in workflows** that are much more powerful:

#### **🔧 Auto-add to Project Workflow**
1. **Click "Add workflow"**
2. **Select "Auto-add to project"**
3. **Configure filters**:
   - **Repository**: `damjanZGB/reStrike_VTA_Cursor`
   - **Issue or PR state**: `open`
   - **Labels**: (optional) specific labels to auto-add

#### **🔄 Auto-archive Workflow**
1. **Click "Add workflow"**  
2. **Select "Auto-archive"**
3. **Configure trigger**: "When items are closed"

#### **📝 Set Status Workflow**
1. **Click "Add workflow"**
2. **Select "Set status"**  
3. **Configure multiple rules**:
   - **When**: Issue opened → **Set Status**: Todo
   - **When**: PR opened → **Set Status**: In Progress  
   - **When**: Issue/PR closed → **Set Status**: Done

---

## 🏷️ **STEP 2: UPDATE GITHUB ACTIONS WORKFLOWS**

The current workflows need to be updated for new GitHub Projects. Here's the corrected configuration:

### **Update `.github/workflows/project-board.yml`**

```yaml
name: Project Board Management

on:
  issues:
    types: [opened, closed, reopened, labeled, unlabeled]
  pull_request:
    types: [opened, closed, merged, labeled, unlabeled]

jobs:
  update-project:
    runs-on: ubuntu-latest
    steps:
      - name: Add to project
        uses: actions/add-to-project@v0.5.0
        with:
          project-url: https://github.com/users/damjanZGB/projects/3
          github-token: ${{ secrets.PAT_TOKEN }}

      - name: Update project fields
        uses: titoportas/update-project-fields@v0.1.0
        with:
          project-url: https://github.com/users/damjanZGB/projects/3
          github-token: ${{ secrets.PAT_TOKEN }}
          item-id: ${{ steps.add-to-project.outputs.itemId }}
          field-keys: Status
          field-values: |
            ${{ 
              (github.event.action == 'opened' && github.event_name == 'issues') && 'Todo' ||
              (github.event.action == 'opened' && github.event_name == 'pull_request') && 'In Progress' ||
              (github.event.action == 'closed') && 'Done' ||
              'Todo'
            }}
```

### **Alternative: Use Specialized Action for New Projects**

For more robust automation, use the `project-beta-automations` action:

```yaml
name: Project Board Automation

on:
  issues:
    types: [opened, reopened, closed]
  pull_request:
    types: [opened, reopened, closed]

env:
  PROJECT_ID: 3  # Your project number
  ORG: damjanZGB
  TODO: "📋 Todo"
  IN_PROGRESS: "🔄 In Progress" 
  DONE: "✅ Done"

jobs:
  issue-opened:
    if: github.event_name == 'issues' && (github.event.action == 'opened' || github.event.action == 'reopened')
    runs-on: ubuntu-latest
    steps:
      - name: Move issue to Todo
        uses: leonsteinhaeuser/project-beta-automations@v2.2.1
        with:
          gh_token: ${{ secrets.PAT_TOKEN }}
          user: ${{ env.ORG }}
          project_id: ${{ env.PROJECT_ID }}
          resource_node_id: ${{ github.event.issue.node_id }}
          status_value: ${{ env.TODO }}

  issue-closed:
    if: github.event_name == 'issues' && github.event.action == 'closed'
    runs-on: ubuntu-latest
    steps:
      - name: Move issue to Done
        uses: leonsteinhaeuser/project-beta-automations@v2.2.1
        with:
          gh_token: ${{ secrets.PAT_TOKEN }}
          user: ${{ env.ORG }}
          project_id: ${{ env.PROJECT_ID }}
          resource_node_id: ${{ github.event.issue.node_id }}
          status_value: ${{ env.DONE }}

  pr-opened:
    if: github.event_name == 'pull_request' && (github.event.action == 'opened' || github.event.action == 'reopened')
    runs-on: ubuntu-latest
    steps:
      - name: Move PR to In Progress
        uses: leonsteinhaeuser/project-beta-automations@v2.2.1
        with:
          gh_token: ${{ secrets.PAT_TOKEN }}
          user: ${{ env.ORG }}
          project_id: ${{ env.PROJECT_ID }}
          resource_node_id: ${{ github.event.pull_request.node_id }}
          status_value: ${{ env.IN_PROGRESS }}

  pr-closed:
    if: github.event_name == 'pull_request' && github.event.action == 'closed'
    runs-on: ubuntu-latest
    steps:
      - name: Move PR to Done
        uses: leonsteinhaeuser/project-beta-automations@v2.2.1
        with:
          gh_token: ${{ secrets.PAT_TOKEN }}
          user: ${{ env.ORG }}
          project_id: ${{ env.PROJECT_ID }}
          resource_node_id: ${{ github.event.pull_request.node_id }}
          status_value: ${{ env.DONE }}
```

---

## 🔧 **STEP 3: UPDATE PAT TOKEN PERMISSIONS**

For new GitHub Projects, your PAT token needs different scopes:

### **Required Scopes for New Projects:**
- ✅ **`project`** - Read and write access to projects
- ✅ **`repo`** - Full control of repositories
- ✅ **`org:read`** - Read organization membership (for org projects)

### **Update Your PAT Token:**
1. Go to https://github.com/settings/tokens
2. Edit your existing token or create a new one
3. Ensure the **`project`** scope is selected (not just classic project scopes)
4. Update the `PAT_TOKEN` secret in your repository

---

## 🎯 **STEP 4: CONFIGURE STATUS FIELD VALUES**

Make sure your project's Status field has the correct options:

1. **Go to Project Settings**: https://github.com/users/damjanZGB/projects/3/settings
2. **Click on "Status" field**
3. **Ensure these options exist**:
   - 📋 Todo (or Backlog)
   - 🔄 In Progress  
   - 👀 Review (optional)
   - ✅ Done
   - 🚫 Blocked (optional)

---

## 🧪 **STEP 5: TEST THE AUTOMATION**

### **Test Issue Workflow**
1. **Create a test issue** in your repository
2. **Verify** it automatically appears in the project
3. **Check** that it's assigned to the "Todo" status
4. **Close the issue** and verify it moves to "Done"

### **Test PR Workflow**  
1. **Create a test pull request**
2. **Verify** it appears in "In Progress" 
3. **Close/merge the PR** and verify it moves to "Done"

---

## 🔍 **STEP 6: TROUBLESHOOTING**

### **Common Issues with New Projects:**

#### **❌ "Project not found" Error**
- **Solution**: Use the project **number** (3) not the full URL
- **Check**: Ensure PAT has `project` scope, not just classic project scopes

#### **❌ Items not moving between columns**
- **Solution**: Check that Status field values match exactly (case-sensitive)
- **Verify**: Built-in workflows are properly configured

#### **❌ "Insufficient permissions" Error**
- **Solution**: Regenerate PAT with `project` scope
- **Note**: Classic project permissions don't work for new projects

### **Verification Commands**
```bash
# Test with GitHub CLI (requires project scope)
gh project list --owner damjanZGB

# View project items
gh project item-list 3 --owner damjanZGB
```

---

## 📚 **KEY DIFFERENCES: Classic vs New Projects**

| Feature | Classic Projects | New Projects (v2) |
|---------|------------------|-------------------|
| **Scope** | Repository-level | User/Org-level |
| **API** | REST API | GraphQL API |
| **Automation** | Basic workflows | Built-in workflows + Actions |
| **Fields** | Status only | Custom fields (Status, Priority, etc.) |
| **PAT Scopes** | `repo`, `admin:org` | `project`, `repo` |

---

## 🎉 **RECOMMENDED QUICK FIX**

Since you already have the project set up, the **fastest solution** is:

### **1. Update PAT Token Scopes**
- Add `project` scope to your existing PAT
- Update `PAT_TOKEN` secret in repository

### **2. Use Built-in Workflows (Recommended)**
- Go to project settings → Workflows
- Add "Auto-add to project" workflow
- Add "Set status" workflows for different triggers
- This requires **no code changes**

### **3. Alternative: Update GitHub Actions**
- Replace current workflow files with the new project automation above
- Use the `leonsteinhaeuser/project-beta-automations@v2.2.1` action

---

## 📝 **IMMEDIATE ACTION ITEMS**

1. ✅ **Update PAT Token**: Add `project` scope
2. ✅ **Configure Built-in Workflows**: Use project settings
3. ✅ **Test Automation**: Create test issue/PR
4. ✅ **Update Documentation**: Reflect new setup

---

**📝 Last Updated**: January 27, 2025  
**👤 Created by**: Development Team  
**🔧 Automation Status**: Updated for New GitHub Projects (v2)

---

*This automation setup is specifically designed for the new GitHub Projects and will work correctly with your current project board structure.* 