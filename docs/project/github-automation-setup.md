# GitHub Project Board Automation Setup Guide

## 🤖 **AUTOMATION CONFIGURATION**

This guide provides step-by-step instructions for configuring automated status synchronization between GitHub issues and the project board.

---

## 📋 **OVERVIEW**

### **Project Board**: https://github.com/users/damjanZGB/projects/3
### **Repository**: https://github.com/damjanZGB/reStrike_VTA_Cursor

The automation system uses:
- **GitHub Labels** for status tracking
- **Built-in Project Workflows** for column automation
- **GitHub Actions** for advanced automation
- **Automatic Status Sync** between labels and board columns

---

## ⚙️ **STEP 1: CONFIGURE PROJECT BOARD AUTOMATION**

### **Access Project Settings**
1. **Navigate to Project Board**: https://github.com/users/damjanZGB/projects/3
2. **Click "⚙️" (Settings)** in the top-right corner of the project board
3. **Select "Workflows" tab** from the left sidebar

### **Configure Column Automation**

#### **🗂️ Backlog Column**
- **Auto-add items**: No automation needed (manual management)
- **Auto-archive**: Never

#### **📋 Todo Column**  
- **Auto-add items**: ✅ Enable
  - **Trigger**: "When issues are labeled with `status:todo`"
- **Auto-archive**: Never

#### **🔄 In Progress Column**
- **Auto-add items**: ✅ Enable
  - **Trigger**: "When issues are labeled with `status:in-progress`"
- **Auto-archive**: Never

#### **👀 Review Column**
- **Auto-add items**: ✅ Enable
  - **Trigger**: "When issues are labeled with `status:review`"
- **Auto-archive**: Never

#### **✅ Done Column**
- **Auto-add items**: ✅ Enable
  - **Trigger**: "When issues are closed"
- **Auto-archive**: After 1 week (optional)

#### **🚫 Blocked Column**
- **Auto-add items**: ✅ Enable
  - **Trigger**: "When issues are labeled with `status:blocked`"
- **Auto-archive**: Never

---

## 🏷️ **STEP 2: UNDERSTAND LABEL SYSTEM**

### **Status Labels** (Primary Automation)
- `status:todo` → **📋 Todo** column
- `status:in-progress` → **🔄 In Progress** column
- `status:review` → **👀 Review** column
- `status:done` → **✅ Done** column (also triggered by closing issue)
- `status:blocked` → **🚫 Blocked** column

### **Component Labels**
- `frontend` - React/TypeScript components
- `backend` - Rust plugins and core logic
- `integration` - OBS WebSocket, external systems
- `testing` - Testing and validation tasks
- `ci/cd` - Continuous integration/deployment
- `infrastructure` - System setup and configuration

### **Priority Labels**
- `priority:high` - Critical path items (red)
- `priority:medium` - Important items (yellow)
- `priority:low` - Nice to have items (green)

### **Effort Labels**
- `effort:small` - Less than 4 hours (light green)
- `effort:medium` - 4-16 hours (light yellow)
- `effort:large` - More than 16 hours (light red)

### **Phase Labels**
- `phase:testing` - Core system testing phase
- `phase:production` - Windows production deployment
- `phase:enhancement` - Future enhancement features

---

## 🔧 **STEP 3: TEST AUTOMATION**

### **Test Status Movement**
1. **Select Any Issue**: Go to https://github.com/damjanZGB/reStrike_VTA_Cursor/issues
2. **Add Status Label**: 
   - Click "Labels" in the right sidebar
   - Add `status:in-progress` label
3. **Verify Movement**: Check project board - issue should move to "In Progress" column
4. **Test Completion**:
   - Close the issue (or add `status:done` label)
   - Verify it moves to "Done" column

### **Test Issue Creation**
1. **Create New Issue**: Use any issue template
2. **Add Labels**: Add appropriate component and priority labels
3. **Verify**: Check that issue appears in "Backlog" initially
4. **Move to Todo**: Add `status:todo` label and verify movement

---

## 🤖 **STEP 4: ADVANCED AUTOMATION (GitHub Actions)**

The repository includes advanced automation workflows:

### **Auto-Labeling** (`.github/workflows/issue-management.yml`)
- **Triggers**: When issues are opened
- **Function**: Automatically adds labels based on title keywords
- **Examples**:
  - Title contains "frontend" → adds `frontend` label
  - Title contains "critical" → adds `priority:high` label
  - Title contains "obs" → adds `integration` label

### **Project Board Sync** (`.github/workflows/project-board.yml`)
- **Triggers**: When issues are labeled or unlabeled
- **Function**: Ensures issues move between columns based on labels
- **Features**: Auto-close issues marked as done

### **Label Management** (`.github/workflows/labels.yml`)
- **Triggers**: When `.github/labels.yml` is updated
- **Function**: Synchronizes repository labels with configuration
- **Benefit**: Maintains consistent labeling across team

---

## 📊 **STEP 5: DAILY WORKFLOW**

### **🚀 Starting Work**
1. **Choose Task**: Select issue from "Todo" column
2. **Assign Yourself**: Add yourself as assignee
3. **Start Work**: Add `status:in-progress` label or drag to "In Progress"
4. **Update**: Regular progress updates in issue comments

### **📝 Requesting Review**
1. **Complete Work**: Finish implementation
2. **Request Review**: Add `status:review` label or drag to "Review"
3. **Documentation**: Update issue with completion details
4. **Link PR**: If applicable, link pull request to issue

### **✅ Completing Tasks**
1. **Final Review**: Ensure all acceptance criteria met
2. **Close Issue**: Close issue (automatically moves to "Done")
3. **Follow-up**: Create new issues for related work if needed

### **🚫 Handling Blockers**
1. **Identify Blocker**: Determine what's blocking progress
2. **Mark Blocked**: Add `status:blocked` label or drag to "Blocked"
3. **Document**: Explain blocker in issue comments
4. **Escalate**: Mention team members if help needed

---

## 🎯 **STEP 6: PROJECT MANAGEMENT BEST PRACTICES**

### **🏷️ Consistent Labeling**
- **Always Use Status Labels**: Keep status current
- **Component Labeling**: Help with filtering and organization
- **Priority Setting**: Use priority labels for planning
- **Effort Estimation**: Add effort labels for capacity planning

### **📊 Progress Tracking**
- **Daily Updates**: Check project board daily
- **Weekly Reviews**: Review "Done" column progress
- **Blocked Items**: Address blocked items promptly
- **Backlog Grooming**: Regularly update backlog priorities

### **👥 Team Collaboration**
- **Assign Issues**: Assign team members to issues
- **Comment Updates**: Regular progress comments
- **Mention Team**: Use @mentions for collaboration
- **Link PRs**: Connect pull requests to issues

---

## 🔍 **STEP 7: FILTERING AND VIEWS**

### **Useful Filters**
1. **By Priority**: Filter issues by `priority:high`, `priority:medium`, `priority:low`
2. **By Component**: Filter by `frontend`, `backend`, `integration`
3. **By Phase**: Filter by `phase:testing`, `phase:production`
4. **By Assignee**: Filter by team member
5. **By Status**: Use board columns for visual status

### **Creating Custom Views**
1. **Click "View" dropdown** on project board
2. **Select "New view"**
3. **Choose filters** and sorting preferences
4. **Save view** with descriptive name

---

## 🛠️ **TROUBLESHOOTING**

### **Issues Not Moving Between Columns**
- **Check Labels**: Ensure correct status labels are applied
- **Verify Workflows**: Check that project workflows are enabled
- **Manual Move**: Drag issues manually if automation fails
- **Refresh Board**: Reload project board page

### **Labels Not Syncing**
- **Check Permissions**: Ensure PAT has proper permissions
- **Workflow Status**: Check GitHub Actions tab for errors
- **Manual Labels**: Add labels manually if automation fails
- **Repository Settings**: Verify workflow permissions

### **Automation Not Working**
- **GitHub Actions**: Check workflow runs in Actions tab
- **Permissions**: Verify PAT token has required scopes
- **Rate Limits**: Check for API rate limiting
- **Workflow Files**: Ensure `.github/workflows/` files are present

---

## 📚 **ADDITIONAL RESOURCES**

### **Documentation Links**
- **Project Board**: https://github.com/users/damjanZGB/projects/3
- **Issues**: https://github.com/damjanZGB/reStrike_VTA_Cursor/issues
- **Actions**: https://github.com/damjanZGB/reStrike_VTA_Cursor/actions
- **Integration Status**: [GitHub Integration Status](./github-integration-status.md)

### **Configuration Files**
- **Issue Templates**: [.github/ISSUE_TEMPLATE/](../../.github/ISSUE_TEMPLATE/)
- **Workflow Files**: [.github/workflows/](../../.github/workflows/)
- **Labels Config**: [.github/labels.yml](../../.github/labels.yml)
- **Automation Scripts**: [scripts/github/](../../scripts/github/)

### **Support**
- **GitHub Docs**: [GitHub Projects Documentation](https://docs.github.com/en/issues/planning-and-tracking-with-projects)
- **Automation Help**: [GitHub Actions Documentation](https://docs.github.com/en/actions)
- **Project Issues**: Create issue in repository for automation problems

---

## 🎉 **SUCCESS CRITERIA**

After completing this setup, you should have:

✅ **Automated Column Movement**: Issues automatically move between columns based on labels  
✅ **Status Synchronization**: Labels sync with project board status  
✅ **Auto-Labeling**: New issues get appropriate labels automatically  
✅ **Workflow Integration**: GitHub Actions handle automation tasks  
✅ **Team Productivity**: Reduced manual project management overhead  

---

**📝 Last Updated**: January 27, 2025  
**👤 Created by**: Development Team  
**🔧 Automation Status**: Ready for Implementation

---

*This automation setup ensures professional project management with minimal manual overhead, allowing the team to focus on development while maintaining clear visibility into project progress.* 