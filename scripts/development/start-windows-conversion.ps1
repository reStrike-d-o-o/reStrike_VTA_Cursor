# reStrike VTA - Windows Conversion Starter Script
# This script helps start the Windows-only conversion process

param(
    [switch]$DryRun,
    [switch]$Force
)

$commitId = "4d222ceed0cd698b7e3ba0d7037f51388d553803"

Write-Host "🚀 reStrike VTA - Windows-Only Conversion Starter" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host "Starting Point: Commit $commitId" -ForegroundColor Yellow
Write-Host ""

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "❌ Error: package.json not found. Please run this script from the project root." -ForegroundColor Red
    exit 1
}

# Check current git status
Write-Host "📋 Checking Git Status..." -ForegroundColor Green
$currentBranch = git branch --show-current
$hasChanges = git status --porcelain

if ($hasChanges) {
    Write-Host "⚠️  Warning: You have uncommitted changes." -ForegroundColor Yellow
    Write-Host "   Current changes:" -ForegroundColor Yellow
    git status --short | ForEach-Object { Write-Host "   $_" -ForegroundColor Yellow }
    Write-Host ""
    
    if (-not $Force) {
        $response = Read-Host "Do you want to commit these changes before conversion? (y/n)"
        if ($response -eq "y" -or $response -eq "Y") {
            $commitMessage = Read-Host "Enter commit message"
            git add .
            git commit -m $commitMessage
            Write-Host "✅ Changes committed." -ForegroundColor Green
        }
    }
}

# Create conversion branch
$conversionBranch = "windows-only-conversion"
Write-Host "🌿 Creating conversion branch: $conversionBranch" -ForegroundColor Green

if ($DryRun) {
    Write-Host "  Would create branch: $conversionBranch" -ForegroundColor Yellow
} else {
    git checkout -b $conversionBranch
    Write-Host "✅ Created and switched to branch: $conversionBranch" -ForegroundColor Green
}

# Show conversion options
Write-Host ""
Write-Host "🛠️  Conversion Options:" -ForegroundColor Green
Write-Host "1. Automated Conversion (Recommended)" -ForegroundColor White
Write-Host "   - Run the full conversion script" -ForegroundColor Gray
Write-Host "   - Automatically removes environment system" -ForegroundColor Gray
Write-Host "   - Updates all components and configuration" -ForegroundColor Gray
Write-Host ""
Write-Host "2. Manual Conversion" -ForegroundColor White
Write-Host "   - Follow the step-by-step guide" -ForegroundColor Gray
Write-Host "   - More control over the process" -ForegroundColor Gray
Write-Host "   - Better for learning and understanding" -ForegroundColor Gray
Write-Host ""

if (-not $DryRun) {
    $choice = Read-Host "Choose conversion method (1 or 2)"
    
    switch ($choice) {
        "1" {
            Write-Host "🚀 Starting automated conversion..." -ForegroundColor Green
            if (Test-Path "scripts/development/convert-to-windows-only.ps1") {
                & "scripts/development/convert-to-windows-only.ps1" -DryRun:$DryRun -Force:$Force
            } else {
                Write-Host "❌ Error: Conversion script not found at scripts/development/convert-to-windows-only.ps1" -ForegroundColor Red
            }
        }
        "2" {
            Write-Host "📖 Manual conversion selected." -ForegroundColor Green
            Write-Host "Please follow the guide at: docs/development/WINDOWS_ONLY_CONVERSION_GUIDE.md" -ForegroundColor Yellow
            Write-Host "Starting point commit: $commitId" -ForegroundColor Yellow
        }
        default {
            Write-Host "❌ Invalid choice. Please run the script again and choose 1 or 2." -ForegroundColor Red
        }
    }
} else {
    Write-Host "🔍 DRY RUN MODE - No actual changes will be made" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "📚 Documentation:" -ForegroundColor Green
Write-Host "- Conversion Guide: docs/development/WINDOWS_ONLY_CONVERSION_GUIDE.md" -ForegroundColor White
Write-Host "- Conversion Tracking: docs/development/WINDOWS_CONVERSION_TRACKING.md" -ForegroundColor White
Write-Host "- VSCode Setup: docs/development/VSCODE_WINDOWS_SETUP.md" -ForegroundColor White
Write-Host ""
Write-Host "🎯 Starting Point: Commit $commitId" -ForegroundColor Yellow
Write-Host "✅ Conversion process ready to begin!" -ForegroundColor Green 