#!/bin/bash

# Framework Update Script for reStrike VTA
# This script helps update Node.js, mpv, and other dependencies to the latest versions

echo "🔄 Framework Update Script for reStrike VTA"
echo "==========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check current versions
check_versions() {
    echo -e "${BLUE}📋 Current Versions:${NC}"
    echo "Node.js: $(node --version 2>/dev/null || echo 'Not installed')"
    echo "npm: $(npm --version 2>/dev/null || echo 'Not installed')"
    echo "mpv: $(mpv --version 2>/dev/null | head -1 || echo 'Not installed')"
    echo "Rust: $(rustc --version 2>/dev/null || echo 'Not installed')"
    echo ""
}

# Function to update Node.js (if possible)
update_nodejs() {
    echo -e "${BLUE}🔄 Updating Node.js...${NC}"
    
    echo -e "${YELLOW}⚠️  Manual Node.js update required.${NC}"
    echo "Please install Node.js v24+ from https://nodejs.org/"
    echo ""
}

# Function to update mpv
update_mpv() {
    echo -e "${BLUE}🔄 Updating mpv...${NC}"
    
    if command -v mpv >/dev/null 2>&1; then
        current_version=$(mpv --version | head -1)
        echo "Current mpv version: $current_version"
        
        # Try to update mpv
        if command -v apt-get >/dev/null 2>&1; then
            echo "Updating mpv via apt..."
            sudo apt-get update
            sudo apt-get install -y mpv
        else
            echo -e "${YELLOW}⚠️  Package manager not found. Manual mpv update required.${NC}"
            echo "Visit https://mpv.io/installation/ for installation instructions"
        fi
    else
        echo -e "${RED}❌ mpv not found. Installing...${NC}"
        if command -v apt-get >/dev/null 2>&1; then
            sudo apt-get update
            sudo apt-get install -y mpv
        else
            echo "Please install mpv manually from https://mpv.io/installation/"
        fi
    fi
    echo ""
}

# Function to update npm packages
update_npm_packages() {
    echo -e "${BLUE}🔄 Updating npm packages...${NC}"
    
    # Update root packages
    if [ -f "package.json" ]; then
        echo "Updating root package.json dependencies..."
        npm update
    fi
    
    # Update UI packages
    if [ -f "ui/package.json" ]; then
        echo "Updating UI package.json dependencies..."
        cd ui
        npm update
        cd ..
    fi
    
    echo ""
}

# Function to check for outdated packages
check_outdated() {
    echo -e "${BLUE}🔍 Checking for outdated packages...${NC}"
    
    if [ -f "package.json" ]; then
        echo "Root package.json outdated packages:"
        npm outdated || echo "All packages up to date"
    fi
    
    if [ -f "ui/package.json" ]; then
        echo ""
        echo "UI package.json outdated packages:"
        cd ui
        npm outdated || echo "All packages up to date"
        cd ..
    fi
    
    echo ""
}

# Function to provide rebuild instructions
rebuild_instructions() {
    echo -e "${BLUE}📋 Rebuild Instructions:${NC}"
    echo ""
    echo "To apply all framework updates:"
    echo ""
    echo "1. ${GREEN}Manual Update:${NC}"
    echo "   - Install Node.js v24+ from https://nodejs.org/"
    echo "   - Install latest mpv from https://mpv.io/installation/"
    echo "   - Run: npm install && cd ui && npm install"
    echo ""
    echo "2. ${GREEN}Verify updates:${NC}"
    echo "   - node --version (should show v24+)"
    echo "   - mpv --version (should show latest version)"
    echo "   - npm outdated (should show minimal outdated packages)"
    echo ""
}

# Main execution
echo "Starting framework update process..."
echo ""

check_versions
update_nodejs
update_mpv
update_npm_packages
check_outdated
rebuild_instructions

echo -e "${GREEN}✅ Framework update script completed!${NC}"
echo ""
echo "Next steps:"
echo "1. Install Node.js v24+ manually if needed"
echo "2. Test that everything still works after the updates"
echo "3. Run 'npm audit' to check for security issues" 