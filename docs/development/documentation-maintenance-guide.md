# Documentation Maintenance Guide

> **Note:** All architecture, onboarding, and coding conventions are defined in .cursor/rules/context.mdc (single source of truth). Project is Windows-only; Docker/devcontainer is fully removed. All onboarding, build, and documentation reference Windows-native setup only.

## 🎯 Purpose

This guide explains how to effectively use and maintain the reStrike VTA documentation system for consistent project development and AI assistance.

## 📋 For New AI Assistants

### **🚨 CRITICAL: Always Include These Files in Context**

When starting a new chat, always include:

1. **`PROJECT_CONTEXT.md`** - Master project document with complete state
2. **`README.md`** - Project overview and quick start guide  
3. **`.cursor/rules/context.mdc`** - Latest development status and context rules
4. **`docs/README.md`** - Documentation navigation index
5. **Previous chat history** - For continuity

### **📖 Reading Order for Understanding**

1. **First**: Read `PROJECT_CONTEXT.md` sections:
   - Mission Statement (understand the Windows desktop app purpose)
   - Current Development Status (know what's done/in-progress)
   - Architecture Overview (understand the Tauri + React + Rust stack)
   - Known Issues (avoid duplicating work)

2. **Second**: Check `.cursor/rules/context.mdc` for:
   - Latest development status
   - Recently completed work
   - Immediate next steps

3. **Third**: Use `docs/README.md` to navigate to specific documentation

## 🔄 Documentation Maintenance Rules

### **⚡ Immediate Updates Required After:**

1. **✅ Completing Major Features**
   ```markdown
   Update locations:
   - PROJECT_CONTEXT.md → "Recently Completed" section
   - .cursor/rules/context.mdc → "Latest Development Status"
   - README.md → Version numbers if applicable
   ```

2. **🐛 Resolving Critical Issues**
   ```markdown
   Move issue from:
   - PROJECT_CONTEXT.md: "Current Issues" → "Recently Resolved" 
   - .cursor/rules/context.mdc: Update status to "✅ RESOLVED"
   ```

3. **🏗️ Architecture Changes**
   ```markdown
   Update:
   - PROJECT_CONTEXT.md → "Architecture Overview" section
   - docs/PROJECT_STRUCTURE.md → File organization
   - README.md → Technology stack if changed
   ```

4. **📦 Framework/Dependency Updates**
   ```markdown
   Update:
   - PROJECT_CONTEXT.md → "Framework Updates & Dependencies"
   - .cursor/rules/context.mdc → "Recent Framework Updates"
   - README.md → Version requirements
   ```

### **📅 Scheduled Maintenance**

#### **Daily** (After each development session)
- [ ] Update `.cursor/rules/context.mdc` with progress
- [ ] Note any new issues discovered
- [ ] Update "Immediate Next Steps" priorities

#### **Weekly** (Every development cycle)
- [ ] Review `PROJECT_CONTEXT.md` for accuracy
- [ ] Update completion percentages
- [ ] Move old issues to resolved if fixed
- [ ] Review and update "Development Priorities"

#### **Monthly** (Major review)
- [ ] Comprehensive review of all documentation
- [ ] Remove outdated information
- [ ] Update architecture if evolved
- [ ] Review and update development workflows

## 🛠️ Practical Usage Examples

### **Scenario 1: Starting New Feature Development**

```bash
# 1. Read current state
cat PROJECT_CONTEXT.md | grep -A 10 "Current Development Status"

# 2. Check immediate priorities  
cat .cursor/rules/context.mdc | grep -A 5 "Immediate Next Steps"

# 3. Use project tools
./scripts/development/dev.sh status
./scripts/project/project-tracker.py summary
```

### **Scenario 2: After Completing a Feature**

```markdown
# Update PROJECT_CONTEXT.md
## ✅ Recently Completed (Add new section)
- **UDP Plugin Implementation**: Complete PSS protocol handling
- **Video Playback**: mpv integration with Windows optimization
- **Testing Framework**: Unit and integration tests added

# Update .cursor/rules/context.mdc  
### Latest Development Status (January 28, 2025):
- **✅ NEW**: UDP protocol implementation completed
- **✅ WORKING**: Video playback with mpv integration
```

### **Scenario 3: Resolving Issues**

```markdown
# Move from "Current Issues" to "Recently Resolved" in PROJECT_CONTEXT.md

## Recently Resolved Issues ✅
- **Tailwind CSS PostCSS Error**: Fixed by using v3.4.17 and correct config
- **Port Conflicts**: Resolved with comprehensive port configuration  
- **Video Playback Performance**: Optimized with Windows-specific mpv build
```

## 📝 Update Templates

### **Template 1: Feature Completion Update**

```markdown
# PROJECT_CONTEXT.md
## ✅ Recently Completed
- **[Feature Name]**: [Brief description]
- **Implementation**: [Technical details]
- **Status**: [Complete/Testing/Documented]
- **Next**: [Related follow-up work]

# .cursor/rules/context.mdc
### Latest Development Status ([Date]):
- **✅ NEW**: [Feature] completed and working
```

### **Template 2: Issue Resolution Update**

```markdown
# PROJECT_CONTEXT.md - Move issue
## Recently Resolved Issues ✅
- **[Issue Name]**: [Resolution summary]
- **Root Cause**: [What caused it]
- **Solution**: [How it was fixed]
- **Prevention**: [How to avoid in future]

# .cursor/rules/context.mdc
- **✅ RESOLVED**: [Issue] - [Brief solution]
```

### **Template 3: Architecture Change Update**

```markdown
# PROJECT_CONTEXT.md
## 🏗️ Architecture Overview
### Updated Technology Stack
- **Backend**: [New stack information]
- **Frontend**: [Updated framework versions]
- **Build**: [New build pipeline details]

## Recent Changes
- **[Date]**: [What changed and why]
```

## 🎯 Quality Assurance Checklist

### **Before Committing Documentation Updates**

- [ ] **Accuracy**: All version numbers and status correct
- [ ] **Completeness**: No missing information for new features
- [ ] **Consistency**: Same information across all documents  
- [ ] **Clarity**: Someone new can understand the current state
- [ ] **Links**: All cross-references work correctly

### **Monthly Documentation Review**

- [ ] **Outdated Sections**: Remove obsolete information
- [ ] **Missing Features**: Add any undocumented functionality
- [ ] **Broken Links**: Fix any broken cross-references
- [ ] **Structure**: Reorganize if documentation has grown unwieldy
- [ ] **Accessibility**: Ensure new contributors can follow guides

## 🚀 Benefits of This System

### **For AI Assistants**
- **Immediate Context**: Understand project state without exploration
- **Avoid Duplication**: See what's already implemented/documented
- **Consistent Quality**: Follow established patterns and practices
- **Error Prevention**: Know about resolved issues and their solutions

### **For Human Developers**
- **Onboarding**: New developers get complete project understanding
- **Continuity**: Work can continue seamlessly across sessions
- **Knowledge Retention**: Nothing is lost when switching between tasks
- **Quality Control**: Consistent documentation standards maintained

### **For Project Management**
- **Status Tracking**: Always know current project state
- **Decision Making**: Historical context for technical decisions
- **Risk Management**: Known issues and their solutions documented
- **Planning**: Clear understanding of what's done and what's next

## 📞 Getting Help

### **If Documentation is Inconsistent**
1. Check the latest timestamps in each file
2. Prioritize information in this order:
   - `.cursor/rules/context.mdc` (most recent)
   - `PROJECT_CONTEXT.md` (comprehensive)
   - `README.md` (overview)

### **If Information is Missing**
1. Check if it's in a specific category under `docs/`
2. Use `grep -r "search term" docs/` to find information
3. Update the relevant documents when you find the answer

### **If Structure Needs Changes**
1. Follow the current organization in `docs/PROJECT_STRUCTURE.md`
2. Update navigation indexes when adding new documents
3. Maintain the established file naming conventions

---

**📝 Note**: This guide should be updated whenever the documentation system evolves or new maintenance procedures are established.

**🔄 Last Updated**: January 27, 2025
**👤 Maintained by**: Development Team
**📋 Status**: Active and Required for All Development Work 