#!/bin/bash

# Setup script for reStrikeVTA simulation environment
# This script ensures all simulation files are in place and Python dependencies are installed

set -e

echo "🔧 Setting up reStrikeVTA simulation environment..."

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SIMULATION_DIR="$PROJECT_ROOT/simulation"

echo "📁 Project root: $PROJECT_ROOT"
echo "📁 Simulation directory: $SIMULATION_DIR"

# Check if simulation directory exists
if [ ! -d "$SIMULATION_DIR" ]; then
    echo "❌ Error: Simulation directory not found at $SIMULATION_DIR"
    echo "Please ensure the simulation files are present in the project."
    exit 1
fi

# Check for critical simulation files
CRITICAL_FILES=(
    "main.py"
    "requirements.txt"
    "config/config.json"
    "core/tkstrike_hardware_simulator.py"
    "core/automated_simulator.py"
    "core/self_test_system.py"
)

echo "🔍 Checking critical simulation files..."
for file in "${CRITICAL_FILES[@]}"; do
    if [ ! -f "$SIMULATION_DIR/$file" ]; then
        echo "❌ Error: Critical simulation file not found: $file"
        exit 1
    fi
    echo "✅ Found: $file"
done

# Check Python installation
echo "🐍 Checking Python installation..."
if command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
elif command -v python &> /dev/null; then
    PYTHON_CMD="python"
elif command -v py &> /dev/null; then
    PYTHON_CMD="py"
else
    echo "❌ Error: Python not found. Please install Python 3.8 or higher."
    exit 1
fi

echo "✅ Python found: $PYTHON_CMD"

# Check Python version
echo "🔍 Checking Python version..."
PYTHON_VERSION=$($PYTHON_CMD --version 2>&1)
echo "📋 Python version: $PYTHON_VERSION"

# Extract version numbers
if [[ $PYTHON_VERSION =~ Python[[:space:]]([0-9]+)\.([0-9]+) ]]; then
    MAJOR_VERSION=${BASH_REMATCH[1]}
    MINOR_VERSION=${BASH_REMATCH[2]}
    
    if [ "$MAJOR_VERSION" -lt 3 ] || ([ "$MAJOR_VERSION" -eq 3 ] && [ "$MINOR_VERSION" -lt 8 ]); then
        echo "❌ Error: Python version too low. Required: 3.8+, Found: $MAJOR_VERSION.$MINOR_VERSION"
        exit 1
    fi
    echo "✅ Python version check passed"
else
    echo "⚠️  Warning: Could not parse Python version, continuing anyway..."
fi

# Install Python dependencies
echo "📦 Installing Python dependencies..."
cd "$SIMULATION_DIR"
if $PYTHON_CMD -m pip install -r requirements.txt; then
    echo "✅ Python dependencies installed successfully"
else
    echo "❌ Error: Failed to install Python dependencies"
    exit 1
fi

# Test simulation environment
echo "🧪 Testing simulation environment..."
if $PYTHON_CMD main.py --list-scenarios > /dev/null 2>&1; then
    echo "✅ Simulation environment test passed"
else
    echo "❌ Error: Simulation environment test failed"
    echo "Trying to run simulation test with more verbose output..."
    $PYTHON_CMD main.py --list-scenarios
    exit 1
fi

echo ""
echo "🎉 Simulation environment setup completed successfully!"
echo ""
echo "📋 Next steps:"
echo "1. Start the reStrikeVTA application: cd src-tauri && cargo tauri dev"
echo "2. Open the PSS Drawer → Simulation Tab"
echo "3. Test the simulation functionality"
echo ""
echo "🔧 If you encounter issues:"
echo "- Check the application logs for detailed error messages"
echo "- Ensure Python 3.8+ is installed and in PATH"
echo "- Verify all simulation files are present in the simulation/ directory" 