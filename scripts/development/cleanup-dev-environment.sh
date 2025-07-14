#!/bin/bash

# reStrike VTA Development Environment Cleanup Script
# This script cleans up all development processes, ports, and temporary files

set -e

echo "🧹 Starting reStrike VTA Development Environment Cleanup..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Function to check if a process is running
is_process_running() {
    pgrep -f "$1" > /dev/null 2>&1
}

# Function to kill processes by pattern
kill_processes() {
    local pattern="$1"
    local description="$2"
    
    if is_process_running "$pattern"; then
        print_status "Stopping $description..."
        pkill -f "$pattern" || true
        sleep 2
        
        # Force kill if still running
        if is_process_running "$pattern"; then
            print_warning "Force killing $description..."
            pkill -9 -f "$pattern" || true
        fi
        
        print_success "$description stopped"
    else
        print_status "$description not running"
    fi
}

# Function to check if port is in use
check_port() {
    local port="$1"
    local service="$2"
    
    if netstat -tulpn 2>/dev/null | grep -q ":$port "; then
        print_warning "Port $port ($service) is still in use"
        netstat -tulpn 2>/dev/null | grep ":$port " || true
    else
        print_success "Port $port ($service) is free"
    fi
}

# Function to clean up temporary files
cleanup_temp_files() {
    print_status "Cleaning up temporary files..."
    
    # Clean npm cache
    if command -v npm >/dev/null 2>&1; then
        print_status "Clearing npm cache..."
        npm cache clean --force 2>/dev/null || true
    fi
    
    # Clean cargo cache (optional)
    if command -v cargo >/dev/null 2>&1; then
        print_status "Clearing cargo cache..."
        cargo clean 2>/dev/null || true
    fi
    
    # Clean node_modules cache
    if [ -d "ui/node_modules/.cache" ]; then
        print_status "Clearing React cache..."
        rm -rf ui/node_modules/.cache
    fi
    
    # Clean build artifacts
    if [ -d "target" ]; then
        print_status "Clearing Rust build artifacts..."
        rm -rf target
    fi
    
    # Clean temporary files
    find . -name "*.tmp" -delete 2>/dev/null || true
    find . -name "*.log" -delete 2>/dev/null || true
    
    print_success "Temporary files cleaned"
}

# Function to reset ports
reset_ports() {
    print_status "Checking port status..."
    
    # Check all development ports
    check_port 3000 "React Frontend"
    check_port 1420 "Tauri Backend"
    check_port 6000 "UDP PSS Protocol"
    check_port 4455 "OBS WebSocket"
    check_port 8080 "Development Server"
}

# Function to show current status
show_status() {
    print_status "Current development environment status:"
    echo
    
    # Check running processes
    echo "📋 Running Processes:"
    if is_process_running "npm start"; then
        echo "  ✅ React Frontend (npm start)"
    else
        echo "  ❌ React Frontend (npm start)"
    fi
    
    if is_process_running "cargo"; then
        echo "  ✅ Rust Backend (cargo)"
    else
        echo "  ❌ Rust Backend (cargo)"
    fi
    
    if is_process_running "mpv"; then
        echo "  ✅ mpv Player"
    else
        echo "  ❌ mpv Player"
    fi
    
    echo
    
    # Check port usage
    echo "🔌 Port Usage:"
    for port in 3000 1420 6000 4455 8080; do
        if netstat -tulpn 2>/dev/null | grep -q ":$port "; then
            echo "  ⚠️  Port $port: IN USE"
        else
            echo "  ✅ Port $port: FREE"
        fi
    done
    
    echo
}

# Main cleanup function
main_cleanup() {
    print_status "Starting comprehensive cleanup..."
    echo
    
    # Stop all development servers
    print_status "Stopping development servers..."
    kill_processes "npm start" "React Frontend"
    kill_processes "cargo" "Rust Backend"
    kill_processes "tauri" "Tauri CLI"
    kill_processes "mpv" "mpv Player"
    kill_processes "node.*3000" "Node.js on port 3000"
    kill_processes "node.*1420" "Node.js on port 1420"
    
    echo
    
    # Clean up temporary files
    cleanup_temp_files
    
    echo
    
    # Reset and check ports
    reset_ports
    
    echo
    
    # Show final status
    show_status
    
    print_success "Cleanup completed successfully!"
    echo
    print_status "To restart development:"
    echo "  Frontend: cd ui && npm start"
    echo "  Backend:  cargo run"
    echo "  Both:     npm run dev"
}

# Quick cleanup function (just stop processes)
quick_cleanup() {
    print_status "Performing quick cleanup (stop processes only)..."
    echo
    
    kill_processes "npm start" "React Frontend"
    kill_processes "cargo" "Rust Backend"
    kill_processes "tauri" "Tauri CLI"
    kill_processes "mpv" "mpv Player"
    
    echo
    print_success "Quick cleanup completed!"
}

# Status check function
status_check() {
    show_status
}

# Help function
show_help() {
    echo "reStrike VTA Development Environment Cleanup Script"
    echo
    echo "Usage: $0 [OPTION]"
    echo
    echo "Options:"
    echo "  --cleanup, -c     Full cleanup (stop processes, clear cache, check ports)"
    echo "  --quick, -q       Quick cleanup (stop processes only)"
    echo "  --status, -s      Show current status"
    echo "  --help, -h        Show this help message"
    echo
    echo "Examples:"
    echo "  $0 --cleanup      # Full cleanup"
    echo "  $0 --quick        # Quick cleanup"
    echo "  $0 --status       # Check status"
    echo
}

# Parse command line arguments
case "${1:-}" in
    --cleanup|-c)
        main_cleanup
        ;;
    --quick|-q)
        quick_cleanup
        ;;
    --status|-s)
        status_check
        ;;
    --help|-h|"")
        show_help
        ;;
    *)
        print_error "Unknown option: $1"
        show_help
        exit 1
        ;;
esac 