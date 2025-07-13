#!/bin/bash

# Install Latest mpv Script
# This script installs the latest mpv version using official builds

echo "🎬 Installing Latest mpv Version"
echo "================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get >/dev/null 2>&1; then
            echo "ubuntu"
        elif command -v yum >/dev/null 2>&1; then
            echo "centos"
        elif command -v pacman >/dev/null 2>&1; then
            echo "arch"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Function to install mpv on Ubuntu/Debian
install_ubuntu() {
    echo -e "${BLUE}Installing mpv on Ubuntu/Debian...${NC}"
    
    # Try PPA first
    echo "Attempting to install from PPA..."
    sudo apt-get update
    sudo apt-get install -y software-properties-common
    
    if sudo add-apt-repository ppa:mpv-player/mpv-stable; then
        sudo apt-get update
        sudo apt-get install -y mpv
        echo -e "${GREEN}✅ mpv installed from PPA${NC}"
    else
        echo -e "${YELLOW}⚠️  PPA not available, trying alternative methods...${NC}"
        
        # Try downloading from official builds
        echo "Downloading latest mpv build..."
        MPV_VERSION=$(curl -s https://api.github.com/repos/mpv-player/mpv/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
        
        if [ -n "$MPV_VERSION" ]; then
            echo "Latest mpv version: $MPV_VERSION"
            
            # Download and install
            cd /tmp
            wget -O mpv.tar.gz "https://github.com/mpv-player/mpv/archive/refs/tags/${MPV_VERSION}.tar.gz"
            
            if [ $? -eq 0 ]; then
                echo -e "${GREEN}✅ Downloaded mpv ${MPV_VERSION}${NC}"
                echo "Note: This is source code. For binary installation, visit:"
                echo "https://mpv.io/installation/"
            else
                echo -e "${RED}❌ Failed to download mpv${NC}"
            fi
        else
            echo -e "${RED}❌ Could not determine latest mpv version${NC}"
        fi
    fi
}

# Function to install mpv on macOS
install_macos() {
    echo -e "${BLUE}Installing mpv on macOS...${NC}"
    
    if command -v brew >/dev/null 2>&1; then
        echo "Installing via Homebrew..."
        brew install mpv
        echo -e "${GREEN}✅ mpv installed via Homebrew${NC}"
    else
        echo -e "${YELLOW}⚠️  Homebrew not found. Please install Homebrew first:${NC}"
        echo "https://brew.sh/"
    fi
}

# Function to install mpv on Windows
install_windows() {
    echo -e "${BLUE}Installing mpv on Windows...${NC}"
    echo "Please download mpv from: https://mpv.io/installation/"
    echo "Or use Chocolatey: choco install mpv"
    echo "Or use Scoop: scoop install mpv"
}

# Function to install mpv on Arch Linux
install_arch() {
    echo -e "${BLUE}Installing mpv on Arch Linux...${NC}"
    sudo pacman -S mpv
    echo -e "${GREEN}✅ mpv installed via pacman${NC}"
}

# Main installation logic
OS=$(detect_os)
echo "Detected OS: $OS"

case $OS in
    "ubuntu"|"debian")
        install_ubuntu
        ;;
    "macos")
        install_macos
        ;;
    "windows")
        install_windows
        ;;
    "arch")
        install_arch
        ;;
    *)
        echo -e "${RED}❌ Unsupported OS: $OS${NC}"
        echo "Please visit https://mpv.io/installation/ for manual installation"
        ;;
esac

# Verify installation
echo ""
echo -e "${BLUE}Verifying installation...${NC}"
if command -v mpv >/dev/null 2>&1; then
    echo -e "${GREEN}✅ mpv is installed${NC}"
    mpv --version | head -1
else
    echo -e "${RED}❌ mpv installation failed${NC}"
    echo "Please install manually from: https://mpv.io/installation/"
fi

echo ""
echo -e "${GREEN}🎬 mpv installation script completed!${NC}" 