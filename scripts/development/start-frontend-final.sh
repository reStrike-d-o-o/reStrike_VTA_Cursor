#!/bin/bash

# reStrike VTA Frontend Startup Script (Final)
# Automatically finds the next free port and starts React development server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🚀 reStrike VTA Frontend Startup (Final)${NC}"
echo "=============================================="

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

# Function to wait for server to start and find actual port
wait_for_server() {
    local max_wait=60
    local wait_time=0
    
    echo -e "${BLUE}[INFO]${NC} Waiting for server to start..."
    
    while [ $wait_time -lt $max_wait ]; do
        # Check common React ports
        for port in 3000 3001 3002 3003 3004 3005; do
            if curl -s http://localhost:$port >/dev/null 2>&1; then
                echo -e "${GREEN}[SUCCESS]${NC} Server is responding on http://localhost:$port"
                echo $port
                return 0
            fi
        done
        
        sleep 2
        wait_time=$((wait_time + 2))
        
        if [ $((wait_time % 10)) -eq 0 ]; then
            echo -e "${BLUE}[INFO]${NC} Still waiting for server... ($wait_time/$max_wait seconds)"
        fi
    done
    
    echo -e "${RED}[ERROR]${NC} Server failed to start within $max_wait seconds"
    return 1
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

# Kill any existing React processes
echo -e "${BLUE}[INFO]${NC} Stopping any existing React processes..."
pkill -f "react-scripts" 2>/dev/null || true
pkill -f "npm start" 2>/dev/null || true
sleep 2

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

echo -e "${BLUE}[INFO]${NC} Starting React development server..."

# Change to UI directory
cd ui

# Set environment variables
export PORT=$port
export CHOKIDAR_USEPOLLING=true
export FAST_REFRESH=true
export WATCHPACK_POLLING=true

# Start the server
echo -e "${BLUE}[INFO]${NC} Starting server with: PORT=$port"
npm start > ../.frontend.log 2>&1 &
server_pid=$!

# Save the PID
echo $server_pid > ../.frontend.pid

# Go back to root directory
cd ..

echo -e "${GREEN}[SUCCESS]${NC} React development server started!"
echo -e "${GREEN}[SUCCESS]${NC} PID: $server_pid"

# Wait for server to start and find actual port
actual_port=$(wait_for_server)
if [ $? -eq 0 ]; then
    echo -e "${GREEN}[SUCCESS]${NC} Server is ready on http://localhost:$actual_port"
    
    # Update port file with actual port
    echo $actual_port > .frontend.port
    
    echo -e "${BLUE}[INFO]${NC} To stop the server, run: kill $server_pid"
    echo -e "${BLUE}[INFO]${NC} Or use: ./scripts/development/dev.sh stop-all"
    echo -e "${BLUE}[INFO]${NC} Logs available in: .frontend.log"
else
    echo -e "${RED}[ERROR]${NC} Server failed to start properly"
    echo -e "${BLUE}[INFO]${NC} Check logs in: .frontend.log"
    exit 1
fi 