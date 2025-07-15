# Windows Fast Development Setup Script
# Optimized for maximum compilation speed

Write-Host "🚀 Setting up fastest Windows development environment..." -ForegroundColor Green

# 1. Install Rust with MSVC toolchain (fastest for Windows)
Write-Host "📦 Installing Rust with MSVC toolchain..." -ForegroundColor Yellow
if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Rust..."
    Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/init.exe -OutFile rustup-init.exe
    .\rustup-init.exe -y
    Remove-Item rustup-init.exe
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH","User")
}

# Set MSVC as default (fastest on Windows)
rustup default stable
rustup target add x86_64-pc-windows-msvc

# 2. Install Tauri CLI
Write-Host "📦 Installing Tauri CLI..." -ForegroundColor Yellow
cargo install tauri-cli

# 3. Install Node.js dependencies
Write-Host "📦 Installing Node.js dependencies..." -ForegroundColor Yellow
npm install
cd ui
npm install
cd ..

# 4. Configure for maximum speed
Write-Host "⚡ Configuring for maximum speed..." -ForegroundColor Yellow

# Set environment variables for speed
$env:GENERATE_SOURCEMAP = "false"
$env:FAST_REFRESH = "true"
$env:CHOKIDAR_USEPOLLING = "false"
$env:REACT_APP_FAST_DEV = "true"
$env:SKIP_PREFLIGHT_CHECK = "true"
$env:ESLINT_NO_DEV_ERRORS = "true"

# 5. Clean any existing builds
Write-Host "🧹 Cleaning existing builds..." -ForegroundColor Yellow
Remove-Item -Recurse -Force ui/build -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force target -ErrorAction SilentlyContinue
cargo clean

# 6. Test fast build
Write-Host "🧪 Testing fast build..." -ForegroundColor Yellow
cd ui
npm run build:fast
cd ..

Write-Host "✅ Fast Windows development environment ready!" -ForegroundColor Green
Write-Host ""
Write-Host "🚀 Quick Start Commands:" -ForegroundColor Cyan
Write-Host "  npm run dev:fast          # Fast development"
Write-Host "  npm run build:fast        # Fast production build"
Write-Host "  cargo tauri dev           # Tauri development"
Write-Host ""
Write-Host "📊 Performance Tips:" -ForegroundColor Cyan
Write-Host "  - Use MSVC toolchain (already configured)"
Write-Host "  - Keep Node.js 20+ LTS for fastest JS compilation"
Write-Host "  - Use fast scripts for development"
Write-Host "  - Clean caches regularly with: npm run clean:all" 