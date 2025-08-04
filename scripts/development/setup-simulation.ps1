# Setup script for reStrikeVTA simulation environment (Windows PowerShell)
# This script ensures all simulation files are in place and Python dependencies are installed

param(
    [switch]$Force
)

Write-Host "🔧 Setting up reStrikeVTA simulation environment..." -ForegroundColor Cyan

# Get the project root directory
$PROJECT_ROOT = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))
$SIMULATION_DIR = Join-Path $PROJECT_ROOT "simulation"

Write-Host "📁 Project root: $PROJECT_ROOT" -ForegroundColor Gray
Write-Host "📁 Simulation directory: $SIMULATION_DIR" -ForegroundColor Gray

# Check if simulation directory exists
if (-not (Test-Path $SIMULATION_DIR)) {
    Write-Host "❌ Error: Simulation directory not found at $SIMULATION_DIR" -ForegroundColor Red
    Write-Host "Please ensure the simulation files are present in the project." -ForegroundColor Yellow
    exit 1
}

# Check for critical simulation files
$CRITICAL_FILES = @(
    "main.py",
    "requirements.txt",
    "config/config.json",
    "core/tkstrike_hardware_simulator.py",
    "core/automated_simulator.py",
    "core/self_test_system.py"
)

Write-Host "🔍 Checking critical simulation files..." -ForegroundColor Cyan
foreach ($file in $CRITICAL_FILES) {
    $filePath = Join-Path $SIMULATION_DIR $file
    if (-not (Test-Path $filePath)) {
        Write-Host "❌ Error: Critical simulation file not found: $file" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Found: $file" -ForegroundColor Green
}

# Check Python installation
Write-Host "🐍 Checking Python installation..." -ForegroundColor Cyan
$PYTHON_CMD = $null

# Try different Python commands
$pythonCommands = @("python", "py", "python3")
foreach ($cmd in $pythonCommands) {
    try {
        $version = & $cmd --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            $PYTHON_CMD = $cmd
            break
        }
    }
    catch {
        # Command not found, try next
    }
}

if (-not $PYTHON_CMD) {
    Write-Host "❌ Error: Python not found. Please install Python 3.8 or higher." -ForegroundColor Red
    Write-Host "Download from: https://www.python.org/downloads/" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Python found: $PYTHON_CMD" -ForegroundColor Green

# Check Python version
Write-Host "🔍 Checking Python version..." -ForegroundColor Cyan
try {
    $PYTHON_VERSION = & $PYTHON_CMD --version 2>&1
    Write-Host "📋 Python version: $PYTHON_VERSION" -ForegroundColor Gray
    
    # Extract version numbers
    if ($PYTHON_VERSION -match "Python\s+(\d+)\.(\d+)") {
        $MAJOR_VERSION = [int]$matches[1]
        $MINOR_VERSION = [int]$matches[2]
        
        if ($MAJOR_VERSION -lt 3 -or ($MAJOR_VERSION -eq 3 -and $MINOR_VERSION -lt 8)) {
            Write-Host "❌ Error: Python version too low. Required: 3.8+, Found: $MAJOR_VERSION.$MINOR_VERSION" -ForegroundColor Red
            exit 1
        }
        Write-Host "✅ Python version check passed" -ForegroundColor Green
    }
    else {
        Write-Host "⚠️  Warning: Could not parse Python version, continuing anyway..." -ForegroundColor Yellow
    }
}
catch {
    Write-Host "⚠️  Warning: Could not check Python version, continuing anyway..." -ForegroundColor Yellow
}

# Install Python dependencies
Write-Host "📦 Installing Python dependencies..." -ForegroundColor Cyan
Push-Location $SIMULATION_DIR
try {
    $pipResult = & $PYTHON_CMD -m pip install -r requirements.txt 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Python dependencies installed successfully" -ForegroundColor Green
    }
    else {
        Write-Host "❌ Error: Failed to install Python dependencies" -ForegroundColor Red
        Write-Host $pipResult -ForegroundColor Red
        exit 1
    }
}
finally {
    Pop-Location
}

# Test simulation environment
Write-Host "🧪 Testing simulation environment..." -ForegroundColor Cyan
Push-Location $SIMULATION_DIR
try {
    $testResult = & $PYTHON_CMD main.py --list-scenarios 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Simulation environment test passed" -ForegroundColor Green
    }
    else {
        Write-Host "❌ Error: Simulation environment test failed" -ForegroundColor Red
        Write-Host "Trying to run simulation test with more verbose output..." -ForegroundColor Yellow
        & $PYTHON_CMD main.py --list-scenarios
        exit 1
    }
}
finally {
    Pop-Location
}

Write-Host ""
Write-Host "🎉 Simulation environment setup completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Next steps:" -ForegroundColor Cyan
Write-Host "1. Start the reStrikeVTA application: cd src-tauri && cargo tauri dev" -ForegroundColor White
Write-Host "2. Open the PSS Drawer → Simulation Tab" -ForegroundColor White
Write-Host "3. Test the simulation functionality" -ForegroundColor White
Write-Host ""
Write-Host "🔧 If you encounter issues:" -ForegroundColor Cyan
Write-Host "- Check the application logs for detailed error messages" -ForegroundColor White
Write-Host "- Ensure Python 3.8+ is installed and in PATH" -ForegroundColor White
Write-Host "- Verify all simulation files are present in the simulation/ directory" -ForegroundColor White 