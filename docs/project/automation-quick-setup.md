# Quick Automation Setup

## 🚀 **Immediate Configuration Steps**

### **1. Add PAT Token to Repository**
```bash
# Go to: https://github.com/reStrike-d-o-o/reStrike_VTA_Cursor/settings/secrets/actions
# Click: "New repository secret"
# Name: PAT_TOKEN
# Value: 
```

### **2. Configure Project Board Workflows**
**URL**: https://github.com/users/damjanZGB/projects/3/settings/workflows

**Add these workflows:**

**Auto-add Workflow**:
- Repository: `reStrike-d-o-o/reStrike_VTA_Cursor`
- When: Issues are opened
- Then: Add to project

**Status Workflows**:
- Issues opened → Set Status to "📋 Todo"
- Issues assigned → Set Status to "🔄 In Progress"  
- Issues closed → Set Status to "✅ Done"

### **3. Test Automation**
Create a test issue to verify automation works:
```bash
# This will trigger the workflows and add to project board automatically
```

## ✅ **Verification Checklist**
- [ ] PAT token added to repository secrets
- [ ] Auto-add workflow configured
- [ ] Status workflows configured
- [ ] Test issue created and automatically added to board 