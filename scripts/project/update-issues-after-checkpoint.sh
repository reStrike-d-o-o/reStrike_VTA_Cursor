#!/bin/bash

# reStrike VTA - Update GitHub Issues After Environment System Checkpoint
# This script updates GitHub issues to reflect the completed environment system implementation

set -e

echo "🔄 Updating GitHub Issues after Environment System Checkpoint..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to update issue status
update_issue_status() {
    local issue_number=$1
    local new_status=$2
    local comment=$3
    
    echo -e "${BLUE}Updating Issue #${issue_number} to status: ${new_status}${NC}"
    
    # Update issue labels
    gh issue edit $issue_number --remove-label "status-backlog" --add-label "status-${new_status}"
    
    # Add comment if provided
    if [ ! -z "$comment" ]; then
        gh issue comment $issue_number --body "$comment"
    fi
    
    echo -e "${GREEN}✅ Issue #${issue_number} updated successfully${NC}"
}

# Function to create new issue
create_issue() {
    local title="$1"
    local body="$2"
    local labels="$3"
    
    echo -e "${BLUE}Creating new issue: ${title}${NC}"
    
    gh issue create --title "$title" --body "$body" --label "$labels"
    
    echo -e "${GREEN}✅ Issue created successfully${NC}"
}

# Check if GitHub CLI is authenticated
if ! gh auth status >/dev/null 2>&1; then
    echo -e "${RED}❌ GitHub CLI not authenticated. Please run 'gh auth login' first.${NC}"
    exit 1
fi

echo -e "${YELLOW}📋 Starting GitHub Issues Update...${NC}"

# Update OBS Integration Issues to "In Progress"

echo -e "${BLUE}🔄 Moving OBS Integration Issues to 'In Progress'...${NC}"

# Issue #29: OBS WebSocket Connection Management
update_issue_status 29 "in-progress" "✅ **Environment System Integration Complete**

The OBS WebSocket connection management has been successfully integrated with the new environment system:

**✅ Completed:**
- Environment-aware OBS connections (Web vs Windows)
- Dual protocol support (v4/v5) with environment detection
- Tauri commands for Windows environment
- Direct WebSocket for web environment
- Authentication handling for both protocols
- Connection status monitoring

**🔄 Next Steps:**
- Complete scene and source control implementation
- Add recording and streaming controls
- Implement status monitoring dashboard

**🌐 Environment System Integration:**
- Uses `useEnvironmentObs()` hook for environment-aware operations
- Automatic protocol detection and connection method selection
- Error handling for both environments

This issue is now **In Progress** and ready for the next phase of OBS integration."

# Issue #30: OBS Scene and Source Control
update_issue_status 30 "in-progress" "✅ **Ready for Environment-Aware Implementation**

This issue is now ready to begin implementation with the environment system:

**🎯 Implementation Plan:**
- Use environment-aware OBS operations
- Implement scene switching for both environments
- Add source control and management
- Create environment-specific UI components

**🌐 Environment System Integration:**
- Will use `useEnvironmentObs()` hook
- Environment-specific scene control methods
- Conditional rendering for different environments

**📋 Acceptance Criteria:**
- [ ] Scene switching works in both web and Windows environments
- [ ] Source control available in both environments
- [ ] Environment-specific error handling
- [ ] UI adapts to current environment

This issue is now **In Progress** and ready for development."

# Issue #31: OBS Recording and Streaming
update_issue_status 31 "in-progress" "✅ **Ready for Environment-Aware Implementation**

This issue is now ready to begin implementation with the environment system:

**🎯 Implementation Plan:**
- Environment-aware recording controls
- Streaming management for both environments
- Status monitoring and feedback
- Integration with existing OBS connection system

**🌐 Environment System Integration:**
- Will use `useEnvironmentObs()` hook
- Environment-specific recording methods
- Status display in environment-aware UI

**📋 Acceptance Criteria:**
- [ ] Recording start/stop works in both environments
- [ ] Streaming controls available in both environments
- [ ] Real-time status monitoring
- [ ] Environment-specific error handling

This issue is now **In Progress** and ready for development."

# Issue #32: OBS Status Monitoring
update_issue_status 32 "in-progress" "✅ **Environment System Integration Complete**

The OBS status monitoring has been successfully integrated with the environment system:

**✅ Completed:**
- Environment-aware status polling
- Real-time status display in sidebar
- Color-coded status indicators (REC, STR, CPU)
- Environment-specific status methods
- Status bar integration with environment system

**🔄 Next Steps:**
- Add more detailed status information
- Implement status history
- Add status alerts and notifications

**🌐 Environment System Integration:**
- Uses environment-aware status polling
- Status display adapts to current environment
- Error handling for both environments

This issue is now **In Progress** and ready for enhancement."

# Create new issues for environment system

echo -e "${BLUE}📝 Creating new issues for environment system...${NC}"

# Issue: Environment System Documentation
create_issue "Environment System Documentation Maintenance" "## 🎯 Environment System Documentation

### Problem Statement
The environment system documentation needs regular maintenance and updates as the system evolves.

### Proposed Solution
Maintain comprehensive documentation for the environment system including:
- Architecture guides
- Usage examples
- Best practices
- Troubleshooting guides
- Migration guides

### User Story
**As a** developer
**I want** up-to-date environment system documentation
**So that** I can effectively use and maintain the environment system

### Acceptance Criteria
- [ ] Architecture documentation is current
- [ ] Usage examples are comprehensive
- [ ] Best practices are documented
- [ ] Troubleshooting guide is complete
- [ ] Migration guide is available

### Implementation Details
- **Documentation**: Update `docs/development/environment-system.md`
- **Examples**: Add more usage examples
- **Troubleshooting**: Expand troubleshooting section
- **Migration**: Update migration guide

### Timeline
- **Documentation**: 1 week
- **Examples**: 1 week
- **Testing**: 1 week
- **Total**: 3 weeks" "enhancement,documentation,priority-medium,category-docs,status-backlog"

# Issue: Environment-Specific Testing
create_issue "Environment-Specific Testing Automation" "## 🎯 Environment-Specific Testing

### Problem Statement
The environment system needs comprehensive automated testing for both web and Windows environments.

### Proposed Solution
Implement automated testing that covers:
- Environment detection
- Environment-specific features
- Component rendering
- API calls
- Error handling

### User Story
**As a** developer
**I want** automated tests for the environment system
**So that** I can ensure reliability across environments

### Acceptance Criteria
- [ ] Environment detection tests
- [ ] Component rendering tests
- [ ] API call tests
- [ ] Error handling tests
- [ ] Integration tests

### Implementation Details
- **Frontend**: React Testing Library for component tests
- **Backend**: Unit tests for environment detection
- **Integration**: End-to-end tests for both environments
- **CI/CD**: Automated testing in GitHub Actions

### Timeline
- **Unit Tests**: 2 weeks
- **Integration Tests**: 2 weeks
- **CI/CD Setup**: 1 week
- **Total**: 5 weeks" "enhancement,testing,priority-medium,category-testing,status-backlog"

# Issue: Environment Performance Optimization
create_issue "Environment System Performance Optimization" "## 🎯 Environment System Performance

### Problem Statement
The environment system needs performance optimization for faster environment switching and reduced overhead.

### Proposed Solution
Optimize the environment system for:
- Faster environment detection
- Reduced memory usage
- Optimized component rendering
- Efficient API calls

### User Story
**As a** user
**I want** fast environment switching
**So that** I can work efficiently in different environments

### Acceptance Criteria
- [ ] Environment detection < 100ms
- [ ] Component rendering optimized
- [ ] Memory usage reduced
- [ ] API call performance improved

### Implementation Details
- **Detection**: Optimize environment detection logic
- **Rendering**: Implement lazy loading for environment-specific components
- **Memory**: Reduce memory footprint
- **API**: Optimize environment-specific API calls

### Timeline
- **Detection Optimization**: 1 week
- **Rendering Optimization**: 2 weeks
- **Memory Optimization**: 1 week
- **Total**: 4 weeks" "enhancement,performance,priority-low,category-performance,status-backlog"

echo -e "${GREEN}✅ GitHub Issues Update Complete!${NC}"

echo -e "${YELLOW}📊 Summary of Changes:${NC}"
echo "  - Moved 4 OBS issues to 'In Progress'"
echo "  - Created 3 new environment system issues"
echo "  - Updated issue descriptions with environment system integration"

echo -e "${BLUE}🎯 Next Steps:${NC}"
echo "  1. Review updated issues in GitHub"
echo "  2. Update project board columns"
echo "  3. Begin development on OBS integration issues"
echo "  4. Monitor environment system performance"

echo -e "${GREEN}🎉 Checkpoint Complete! Environment System is ready for production.${NC}" 