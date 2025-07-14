#!/bin/bash

# OBS WebSocket Dual-Protocol Setup Script
# This script helps set up OBS WebSocket for reStrike VTA

echo "🎬 OBS WebSocket Dual-Protocol Setup"
echo "===================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check if OBS is installed
check_obs_installation() {
    echo -e "${BLUE}📋 Checking OBS Studio installation...${NC}"
    
    if command -v obs >/dev/null 2>&1; then
        echo -e "${GREEN}✅ OBS Studio found${NC}"
        return 0
    elif command -v obs-studio >/dev/null 2>&1; then
        echo -e "${GREEN}✅ OBS Studio found${NC}"
        return 0
    else
        echo -e "${RED}❌ OBS Studio not found${NC}"
        echo -e "${YELLOW}Please install OBS Studio from: https://obsproject.com/${NC}"
        return 1
    fi
}

# Function to download OBS WebSocket plugin
download_obs_websocket() {
    local version=$1
    local download_url=""
    local filename=""
    
    echo -e "${BLUE}📥 Downloading OBS WebSocket ${version}...${NC}"
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [[ "$version" == "v4" ]]; then
            download_url="https://github.com/obsproject/obs-websocket/releases/download/4.9.1/obs-websocket-4.9.1-compat-Qt5-64bit.tar.gz"
            filename="obs-websocket-4.9.1-compat-Qt5-64bit.tar.gz"
        else
            download_url="https://github.com/obsproject/obs-websocket/releases/download/5.2.3/obs-websocket-5.2.3-linux-x64.tar.gz"
            filename="obs-websocket-5.2.3-linux-x64.tar.gz"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        if [[ "$version" == "v4" ]]; then
            download_url="https://github.com/obsproject/obs-websocket/releases/download/4.9.1/obs-websocket-4.9.1-macos-x64.tar.gz"
            filename="obs-websocket-4.9.1-macos-x64.tar.gz"
        else
            download_url="https://github.com/obsproject/obs-websocket/releases/download/5.2.3/obs-websocket-5.2.3-macos-x64.tar.gz"
            filename="obs-websocket-5.2.3-macos-x64.tar.gz"
        fi
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        if [[ "$version" == "v4" ]]; then
            download_url="https://github.com/obsproject/obs-websocket/releases/download/4.9.1/obs-websocket-4.9.1-windows-x64.zip"
            filename="obs-websocket-4.9.1-windows-x64.zip"
        else
            download_url="https://github.com/obsproject/obs-websocket/releases/download/5.2.3/obs-websocket-5.2.3-windows-x64.zip"
            filename="obs-websocket-5.2.3-windows-x64.zip"
        fi
    fi
    
    if [[ -n "$download_url" ]]; then
        echo -e "${YELLOW}Downloading from: $download_url${NC}"
        
        # Create downloads directory
        mkdir -p downloads/obs-websocket
        
        # Download the file
        if curl -L -o "downloads/obs-websocket/$filename" "$download_url"; then
            echo -e "${GREEN}✅ Downloaded OBS WebSocket ${version}${NC}"
            return 0
        else
            echo -e "${RED}❌ Failed to download OBS WebSocket ${version}${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ Unsupported operating system${NC}"
        return 1
    fi
}

# Function to install OBS WebSocket plugin
install_obs_websocket() {
    local version=$1
    
    echo -e "${BLUE}🔧 Installing OBS WebSocket ${version}...${NC}"
    
    # Find OBS installation directory
    local obs_path=""
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        obs_path="/usr/lib/obs-plugins"
        if [[ ! -d "$obs_path" ]]; then
            obs_path="/usr/local/lib/obs-plugins"
        fi
        if [[ ! -d "$obs_path" ]]; then
            obs_path="$HOME/.config/obs-studio/plugins"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        obs_path="/Applications/OBS.app/Contents/PlugIns"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        obs_path="C:/Program Files/obs-studio/bin/64bit"
    fi
    
    if [[ -d "$obs_path" ]]; then
        echo -e "${GREEN}✅ Found OBS plugins directory: $obs_path${NC}"
        
        # Extract and install
        local filename=""
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            if [[ "$version" == "v4" ]]; then
                filename="obs-websocket-4.9.1-compat-Qt5-64bit.tar.gz"
            else
                filename="obs-websocket-5.2.3-linux-x64.tar.gz"
            fi
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            if [[ "$version" == "v4" ]]; then
                filename="obs-websocket-4.9.1-macos-x64.tar.gz"
            else
                filename="obs-websocket-5.2.3-macos-x64.tar.gz"
            fi
        elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
            if [[ "$version" == "v4" ]]; then
                filename="obs-websocket-4.9.1-windows-x64.zip"
            else
                filename="obs-websocket-5.2.3-windows-x64.zip"
            fi
        fi
        
        if [[ -f "downloads/obs-websocket/$filename" ]]; then
            echo -e "${YELLOW}Extracting and installing...${NC}"
            
            # Create backup
            if [[ -f "$obs_path/obs-websocket.so" ]] || [[ -f "$obs_path/obs-websocket.dll" ]]; then
                echo -e "${YELLOW}Creating backup of existing installation...${NC}"
                cp "$obs_path"/obs-websocket.* "$obs_path"/obs-websocket.backup.$(date +%Y%m%d_%H%M%S) 2>/dev/null || true
            fi
            
            # Extract and copy files
            if [[ "$OSTYPE" == "linux-gnu"* ]]; then
                tar -xzf "downloads/obs-websocket/$filename" -C "downloads/obs-websocket/"
                sudo cp downloads/obs-websocket/obs-websocket-*/bin/64bit/obs-websocket.so "$obs_path/"
                sudo cp -r downloads/obs-websocket/obs-websocket-*/data/obs-plugins/obs-websocket "$obs_path/../data/obs-plugins/"
            elif [[ "$OSTYPE" == "darwin"* ]]; then
                tar -xzf "downloads/obs-websocket/$filename" -C "downloads/obs-websocket/"
                sudo cp downloads/obs-websocket/obs-websocket-*/bin/obs-websocket.so "$obs_path/"
                sudo cp -r downloads/obs-websocket/obs-websocket-*/data/obs-plugins/obs-websocket "$obs_path/../data/obs-plugins/"
            elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
                unzip -q "downloads/obs-websocket/$filename" -d "downloads/obs-websocket/"
                cp downloads/obs-websocket/obs-websocket-*/bin/64bit/obs-websocket.dll "$obs_path/"
                cp -r downloads/obs-websocket/obs-websocket-*/data/obs-plugins/obs-websocket "$obs_path/../data/obs-plugins/"
            fi
            
            echo -e "${GREEN}✅ OBS WebSocket ${version} installed successfully${NC}"
            return 0
        else
            echo -e "${RED}❌ Installation file not found${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ OBS plugins directory not found${NC}"
        echo -e "${YELLOW}Please install OBS Studio first${NC}"
        return 1
    fi
}

# Function to create configuration template
create_config_template() {
    echo -e "${BLUE}📝 Creating configuration template...${NC}"
    
    cat > obs_websocket_config.json << 'EOF'
{
  "obs_connections": [
    {
      "name": "Main OBS",
      "host": "localhost",
      "port": 4455,
      "password": "your_password_here",
      "protocol_version": "v5",
      "enabled": true
    },
    {
      "name": "Backup OBS",
      "host": "192.168.1.100",
      "port": 4444,
      "password": "backup_password",
      "protocol_version": "v4",
      "enabled": true
    }
  ]
}
EOF
    
    echo -e "${GREEN}✅ Configuration template created: obs_websocket_config.json${NC}"
    echo -e "${YELLOW}Please edit this file with your actual OBS connection details${NC}"
}

# Function to show setup instructions
show_setup_instructions() {
    echo -e "${BLUE}📋 OBS WebSocket Setup Instructions${NC}"
    echo "=========================================="
    echo ""
    echo "1. Start OBS Studio"
    echo "2. Go to Tools → WebSocket Server Settings"
    echo "3. Configure the following settings:"
    echo ""
    echo "   For OBS WebSocket v5:"
    echo "   - Server Port: 4455"
    echo "   - Enable Authentication: Yes"
    echo "   - Password: (set a strong password)"
    echo ""
    echo "   For OBS WebSocket v4:"
    echo "   - Server Port: 4444"
    echo "   - Enable Authentication: Yes"
    echo "   - Password: (set a strong password)"
    echo ""
    echo "4. Click 'OK' to save settings"
    echo "5. Restart OBS Studio"
    echo ""
    echo "6. Update the configuration file with your settings:"
    echo "   - Edit obs_websocket_config.json"
    echo "   - Replace passwords with your actual passwords"
    echo "   - Update host/IP addresses if needed"
    echo ""
    echo "7. Test the connection in reStrike VTA"
    echo ""
}

# Function to test connection
test_connection() {
    local host=$1
    local port=$2
    local version=$3
    
    echo -e "${BLUE}🔍 Testing connection to ${host}:${port} (${version})...${NC}"
    
    if command -v nc >/dev/null 2>&1; then
        if nc -z "$host" "$port" 2>/dev/null; then
            echo -e "${GREEN}✅ Connection successful${NC}"
            return 0
        else
            echo -e "${RED}❌ Connection failed${NC}"
            return 1
        fi
    elif command -v telnet >/dev/null 2>&1; then
        if timeout 5 bash -c "</dev/tcp/$host/$port" 2>/dev/null; then
            echo -e "${GREEN}✅ Connection successful${NC}"
            return 0
        else
            echo -e "${RED}❌ Connection failed${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  Cannot test connection (netcat/telnet not available)${NC}"
        return 0
    fi
}

# Main setup function
main() {
    echo -e "${BLUE}🚀 Starting OBS WebSocket Dual-Protocol Setup${NC}"
    echo ""
    
    # Check OBS installation
    if ! check_obs_installation; then
        exit 1
    fi
    
    # Ask user which versions to install
    echo -e "${BLUE}📦 Which OBS WebSocket versions do you want to install?${NC}"
    echo "1. OBS WebSocket v5 only (recommended)"
    echo "2. OBS WebSocket v4 only"
    echo "3. Both v4 and v5 (for dual-protocol support)"
    echo "4. Skip installation (manual setup)"
    echo ""
    read -p "Enter your choice (1-4): " choice
    
    case $choice in
        1)
            echo -e "${BLUE}Installing OBS WebSocket v5...${NC}"
            if download_obs_websocket "v5" && install_obs_websocket "v5"; then
                echo -e "${GREEN}✅ OBS WebSocket v5 installation completed${NC}"
            else
                echo -e "${RED}❌ OBS WebSocket v5 installation failed${NC}"
                exit 1
            fi
            ;;
        2)
            echo -e "${BLUE}Installing OBS WebSocket v4...${NC}"
            if download_obs_websocket "v4" && install_obs_websocket "v4"; then
                echo -e "${GREEN}✅ OBS WebSocket v4 installation completed${NC}"
            else
                echo -e "${RED}❌ OBS WebSocket v4 installation failed${NC}"
                exit 1
            fi
            ;;
        3)
            echo -e "${BLUE}Installing both OBS WebSocket v4 and v5...${NC}"
            if download_obs_websocket "v5" && install_obs_websocket "v5"; then
                echo -e "${GREEN}✅ OBS WebSocket v5 installation completed${NC}"
            else
                echo -e "${RED}❌ OBS WebSocket v5 installation failed${NC}"
                exit 1
            fi
            
            if download_obs_websocket "v4" && install_obs_websocket "v4"; then
                echo -e "${GREEN}✅ OBS WebSocket v4 installation completed${NC}"
            else
                echo -e "${RED}❌ OBS WebSocket v4 installation failed${NC}"
                exit 1
            fi
            ;;
        4)
            echo -e "${YELLOW}Skipping installation...${NC}"
            ;;
        *)
            echo -e "${RED}Invalid choice${NC}"
            exit 1
            ;;
    esac
    
    # Create configuration template
    create_config_template
    
    # Show setup instructions
    show_setup_instructions
    
    # Test connections if OBS is running
    echo -e "${BLUE}🧪 Testing connections...${NC}"
    echo "Make sure OBS Studio is running with WebSocket enabled"
    echo ""
    
    read -p "Test connection to localhost:4455 (v5)? (y/n): " test_v5
    if [[ "$test_v5" == "y" || "$test_v5" == "Y" ]]; then
        test_connection "localhost" "4455" "v5"
    fi
    
    read -p "Test connection to localhost:4444 (v4)? (y/n): " test_v4
    if [[ "$test_v4" == "y" || "$test_v4" == "Y" ]]; then
        test_connection "localhost" "4444" "v4"
    fi
    
    echo ""
    echo -e "${GREEN}🎉 OBS WebSocket setup completed!${NC}"
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. Configure OBS WebSocket settings in OBS Studio"
    echo "2. Update the configuration file with your settings"
    echo "3. Start reStrike VTA and test the connections"
    echo "4. Check the documentation for advanced configuration"
    echo ""
    echo -e "${YELLOW}Documentation: docs/OBS_WEBSOCKET_CONFIG.md${NC}"
}

# Run main function
main "$@" 