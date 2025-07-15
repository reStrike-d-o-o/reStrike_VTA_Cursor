#!/bin/bash

# reStrike VTA Frontend Startup Script (Simplified)
# Automatically finds the next free port and starts React development server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🚀 reStrike VTA Frontend Startup${NC}"
echo "=========================================="

# Function to check if a port is available
is_port_available() {
    local port=$1
    ! netstat -tulpn 2>/dev/null | grep -q ":$port "
}

# Function to find the next available port
find_free_port() {
    local start_port=${1:-3000}
    local max_port=${2:-3010}
    local port=$start_port
    
    echo -e "${BLUE}[INFO]${NC} Looking for free port starting from $start_port..."
    
    while [ $port -le $max_port ]; do
        if is_port_available $port; then
            echo -e "${GREEN}[SUCCESS]${NC} Found free port: $port"
            echo $port
            return 0
        fi
        echo -e "${BLUE}[INFO]${NC} Port $port is occupied, trying next..."
        port=$((port + 1))
    done
    
    echo -e "${RED}[ERROR]${NC} No free ports found between $start_port and $max_port"
    return 1
}

# Function to kill existing processes on a port
kill_port_processes() {
    local port=$1
    local pids=$(lsof -ti:$port 2>/dev/null || true)
    
    if [ -n "$pids" ]; then
        echo -e "${YELLOW}[WARNING]${NC} Killing processes on port $port: $pids"
        echo $pids | xargs kill -9 2>/dev/null || true
        sleep 2
    fi
}

# Parse command line arguments
PORT=""
FORCE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  -p, --port PORT     Start on specific port (default: auto-detect)"
            echo "  -f, --force         Force kill processes on target port"
            echo "  -h, --help          Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}[ERROR]${NC} Unknown option: $1"
            exit 1
            ;;
    esac
done

# Determine target port
if [ -n "$PORT" ]; then
    if [ "$FORCE" = true ]; then
        kill_port_processes $PORT
    fi
    
    if ! is_port_available $PORT; then
        echo -e "${RED}[ERROR]${NC} Port $PORT is not available"
        exit 1
    fi
    
    port=$PORT
else
    # Auto-detect free port
    port=$(find_free_port 3000 3010)
    if [ $? -ne 0 ]; then
        exit 1
    fi
fi

echo -e "${BLUE}[INFO]${NC} Starting React development server on port $port..."

# Change to UI directory
cd ui

# Set environment variables
export PORT=$port
export CHOKIDAR_USEPOLLING=true
export FAST_REFRESH=true
export WATCHPACK_POLLING=true

# Start the server
echo -e "${BLUE}[INFO]${NC} Starting server with: PORT=$port"
npm start &

# Save the PID
server_pid=$!
echo $server_pid > ../.frontend.pid
echo $port > ../.frontend.port

# Go back to root directory
cd ..

echo -e "${GREEN}[SUCCESS]${NC} React development server started!"
echo -e "${GREEN}[SUCCESS]${NC} PID: $server_pid"
echo -e "${GREEN}[SUCCESS]${NC} URL: http://localhost:$port"
echo -e "${BLUE}[INFO]${NC} Server is starting up... Please wait a moment for it to be ready."

# Wait a moment and check if server is responding
sleep 5
if curl -s http://localhost:$port >/dev/null 2>&1; then
    echo -e "${GREEN}[SUCCESS]${NC} Server is responding on http://localhost:$port"
else
    echo -e "${YELLOW}[WARNING]${NC} Server may still be starting up. Please check http://localhost:$port in a few moments."
fi

echo -e "${BLUE}[INFO]${NC} To stop the server, run: kill $server_pid"
echo -e "${BLUE}[INFO]${NC} Or use: ./scripts/development/dev.sh stop-all" 