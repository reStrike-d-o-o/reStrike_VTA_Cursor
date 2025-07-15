#!/bin/bash

# reStrike VTA Development Management Wrapper
# This script provides easy access to all development management commands

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
    echo -e "${PURPLE}🚀 reStrike VTA Development Management${NC}"
    echo "=================================================="
}

print_command() {
    echo -e "${CYAN}$1${NC}"
}

print_help() {
    print_header
    echo
    echo "Usage: ./scripts/dev.sh [COMMAND]"
    echo
    echo "Development Commands:"
    print_command "  start-frontend    Start React frontend"
    print_command "  start-backend     Start Rust backend"
    print_command "  start-all         Start both frontend and backend"
    print_command "  stop-all          Stop all development servers"
    echo
    echo "Management Commands:"
    print_command "  status            Show development environment status"
    print_command "  ports             List all ports and their status"
    print_command "  services          List all services and their status"
    print_command "  cleanup           Full cleanup (stop processes, clear cache)"
    print_command "  quick-cleanup     Quick cleanup (stop processes only)"
    print_command "  health            Run health checks"
    echo
    echo "Utility Commands:"
    print_command "  install-deps      Install all dependencies"
    print_command "  build             Build the project"
    print_command "  test              Run tests"
    print_command "  update-config     Update configuration status"
    echo
    echo "Examples:"
    echo "  ./scripts/dev.sh start-frontend"
    echo "  ./scripts/dev.sh status"
    echo "  ./scripts/dev.sh cleanup"
    echo
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to start frontend
start_frontend() {
    echo -e "${BLUE}Starting React frontend...${NC}"
    ./scripts/development/start-frontend-final.sh
}

# Function to start backend
start_backend() {
    echo -e "${BLUE}Starting Rust backend...${NC}"
    cargo run &
    echo -e "${GREEN}Rust backend started${NC}"
}

# Function to start all services
start_all() {
    echo -e "${BLUE}Starting all development services...${NC}"
    start_backend
    sleep 2
    start_frontend
    echo -e "${GREEN}All services started!${NC}"
}

# Function to stop all services
stop_all() {
    echo -e "${BLUE}Stopping all development services...${NC}"
    ./scripts/development/cleanup-dev-environment.sh --quick
}

# Function to install dependencies
install_deps() {
    echo -e "${BLUE}Installing dependencies...${NC}"
    
    # Install root dependencies
    echo "Installing root dependencies..."
    npm install
    
    # Install UI dependencies
    echo "Installing UI dependencies..."
    cd ui
    npm install
    cd ..
    
    echo -e "${GREEN}All dependencies installed!${NC}"
}

# Function to build project
build_project() {
    echo -e "${BLUE}Building project...${NC}"
    
    # Build Rust backend
    echo "Building Rust backend..."
    cargo build
    
    # Build React frontend
    echo "Building React frontend..."
    cd ui
    npm run build
    cd ..
    
    echo -e "${GREEN}Project built successfully!${NC}"
}

# Function to run tests
run_tests() {
    echo -e "${BLUE}Running tests...${NC}"
    
    # Run Rust tests
    echo "Running Rust tests..."
    cargo test
    
    # Run React tests
    echo "Running React tests..."
    cd ui
    npm test
    cd ..
    
    echo -e "${GREEN}All tests completed!${NC}"
}

# Main function
main() {
    case "${1:-}" in
        start-frontend)
            start_frontend
            ;;
        start-backend)
            start_backend
            ;;
        start-all)
            start_all
            ;;
        stop-all)
            stop_all
            ;;
        status)
            python3 scripts/development/manage-dev-resources.py status
            ;;
        ports)
            python3 scripts/development/manage-dev-resources.py ports
            ;;
        services)
            python3 scripts/development/manage-dev-resources.py services
            ;;
        cleanup)
            ./scripts/development/cleanup-dev-environment.sh --cleanup
            ;;
        quick-cleanup)
            ./scripts/development/cleanup-dev-environment.sh --quick
            ;;
        health)
            python3 scripts/development/manage-dev-resources.py health
            ;;
        install-deps)
            install_deps
            ;;
        build)
            build_project
            ;;
        test)
            run_tests
            ;;
        update-config)
            python3 scripts/development/manage-dev-resources.py update
            ;;
        help|--help|-h|"")
            print_help
            ;;
        *)
            echo -e "${RED}Unknown command: $1${NC}"
            echo
            print_help
            exit 1
            ;;
    esac
}

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: This script must be run from the project root directory.${NC}"
    exit 1
fi

# Run main function
main "$@" 