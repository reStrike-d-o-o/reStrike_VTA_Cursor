#!/bin/bash

# reStrike VTA Frontend Startup Script
# Automatically finds the next free port and starts React development server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_header() {
    echo -e "${PURPLE}🚀 reStrike VTA Frontend Startup${NC}"
    echo "=========================================="
}

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

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
    
    print_status "Looking for free port starting from $start_port..."
    
    while [ $port -le $max_port ]; do
        if is_port_available $port; then
            print_success "Found free port: $port"
            echo $port
            return 0
        fi
        print_status "Port $port is occupied, trying next..."
        port=$((port + 1))
    done
    
    print_error "No free ports found between $start_port and $max_port"
    return 1
}

# Function to kill existing processes on a port
kill_port_processes() {
    local port=$1
    local pids=$(lsof -ti:$port 2>/dev/null || true)
    
    if [ -n "$pids" ]; then
        print_warning "Killing processes on port $port: $pids"
        echo $pids | xargs kill -9 2>/dev/null || true
        sleep 2
    fi
}

# Function to start React development server
start_react_server() {
    local port=$1
    local env=${2:-""}
    
    print_status "Starting React development server on port $port..."
    
    cd ui
    
    # Set environment variables
    export PORT=$port
    export CHOKIDAR_USEPOLLING=true
    export FAST_REFRESH=true
    export WATCHPACK_POLLING=true
    
    # Add environment-specific variables
    if [ -n "$env" ]; then
        export REACT_APP_ENVIRONMENT=$env
    fi
    
    # Start the server in background
    npm start &
    local server_pid=$!
    
    cd ..
    
    # Wait for server to start
    print_status "Waiting for server to start..."
    local attempts=0
    local max_attempts=30
    
    while [ $attempts -lt $max_attempts ]; do
        if curl -s http://localhost:$port >/dev/null 2>&1; then
            print_success "React development server started successfully!"
            print_success "Server URL: http://localhost:$port"
            echo $server_pid
            return 0
        fi
        
        sleep 1
        attempts=$((attempts + 1))
        
        if [ $((attempts % 5)) -eq 0 ]; then
            print_status "Still waiting for server... (attempt $attempts/$max_attempts)"
        fi
    done
    
    print_error "Failed to start React development server after $max_attempts attempts"
    kill $server_pid 2>/dev/null || true
    return 1
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -p, --port PORT     Start on specific port (default: auto-detect)"
    echo "  -e, --env ENV       Set environment (web, windows, default: auto)"
    echo "  -f, --force         Force kill processes on target port"
    echo "  -h, --help          Show this help message"
    echo
    echo "Examples:"
    echo "  $0                    # Auto-detect port and environment"
    echo "  $0 -p 3001           # Start on port 3001"
    echo "  $0 -e web            # Start in web environment"
    echo "  $0 -f                # Force kill processes and start"
}

# Main function
main() {
    local target_port=""
    local environment=""
    local force_kill=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -p|--port)
                target_port="$2"
                shift 2
                ;;
            -e|--env)
                environment="$2"
                shift 2
                ;;
            -f|--force)
                force_kill=true
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    print_header
    
    # Determine target port
    if [ -n "$target_port" ]; then
        if [ "$force_kill" = true ]; then
            kill_port_processes $target_port
        fi
        
        if ! is_port_available $target_port; then
            print_error "Port $target_port is not available"
            exit 1
        fi
        
        port=$target_port
    else
        # Auto-detect free port
        port=$(find_free_port 3000 3010)
        if [ $? -ne 0 ]; then
            exit 1
        fi
    fi
    
    # Determine environment
    if [ -z "$environment" ]; then
        # Auto-detect environment
        if [ -n "$__TAURI__" ] || command -v tauri >/dev/null 2>&1; then
            environment="windows"
        else
            environment="web"
        fi
    fi
    
    print_status "Environment: $environment"
    print_status "Port: $port"
    
    # Start the server
    server_pid=$(start_react_server $port $environment)
    if [ $? -eq 0 ]; then
        print_success "Frontend startup completed successfully!"
        print_success "PID: $server_pid"
        print_success "URL: http://localhost:$port"
        print_success "Environment: $environment"
        
        # Save PID to file for later cleanup
        echo $server_pid > .frontend.pid
        echo $port > .frontend.port
        
        exit 0
    else
        print_error "Frontend startup failed"
        exit 1
    fi
}

# Run main function
main "$@" 