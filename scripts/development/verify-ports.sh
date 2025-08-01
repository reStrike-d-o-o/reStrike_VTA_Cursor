#!/bin/bash

# Port Forwarding Verification Script for reStrike VTA
# This script checks if all required ports are accessible

echo "🔍 Verifying Port Forwarding for reStrike VTA"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if a port is accessible
check_port() {
    local port=$1
    local service=$2
    local protocol=${3:-tcp}
    
    echo -n "Checking $service (port $port)... "
    
    if [ "$protocol" = "udp" ]; then
        # For UDP, we can only check if the port is in use, not if it's accessible
        if netstat -tulpn 2>/dev/null | grep -q ":$port.*udp"; then
            echo -e "${GREEN}✅ UDP port $port is in use${NC}"
        else
            echo -e "${YELLOW}⚠️  UDP port $port not in use (may be normal)${NC}"
        fi
    else
        # For TCP, check if we can connect
        if timeout 2 bash -c "</dev/tcp/localhost/$port" 2>/dev/null; then
            echo -e "${GREEN}✅ Port $port is accessible${NC}"
        else
            echo -e "${RED}❌ Port $port is not accessible${NC}"
        fi
    fi
}

# Function to check if a service is running
check_service() {
    local service=$1
    local command=$2
    
    echo -n "Checking $service... "
    if command -v $command >/dev/null 2>&1; then
        echo -e "${GREEN}✅ $service is available${NC}"
    else
        echo -e "${RED}❌ $service is not available${NC}"
    fi
}

echo ""
echo "📋 Checking Required Services:"
check_service "Node.js" "node"
check_service "Rust" "rustc"
check_service "Cargo" "cargo"
check_service "Tauri CLI" "cargo tauri"
check_service "mpv" "mpv"

echo ""
echo "🌐 Checking Port Accessibility:"
check_port 3000 "React Frontend"
check_port 1420 "Tauri Backend"
check_port 8888 "UDP PSS Protocol" "udp"
check_port 4455 "OBS WebSocket"
check_port 8080 "Development Server"

echo ""
echo "🔧 Additional Checks:"

# Check network interfaces
echo ""
echo "🌍 Network Interfaces:"
ip addr show | grep -E "inet.*scope global" | awk '{print "  " $2}' 2>/dev/null || echo "Network interface check not available"

echo ""
echo "📝 Summary:"
echo "==========="
echo "• React Frontend (3000): Should be accessible when running 'npm start' in ui/"
echo "• Tauri Backend (1420): Should be accessible when running 'npm start' in root"
echo "• UDP PSS Protocol (8888): Will be used by the UDP plugin for competition data"
echo "• OBS WebSocket (4455): Requires OBS Studio with WebSocket plugin enabled"
echo "• Development Server (8080): Available for additional development services"

echo ""
echo "🚀 Next Steps:"
echo "1. Start React frontend: cd ui && npm start"
echo "2. Start Tauri backend: npm start"
echo "3. Configure OBS Studio WebSocket plugin on port 4455"
echo "4. Test UDP reception on port 8888"

echo ""
echo "✅ Port forwarding verification complete!" 