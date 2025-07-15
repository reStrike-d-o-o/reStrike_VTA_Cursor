#!/bin/bash

# Fast Development Script for reStrike VTA
# Optimized for maximum development speed

set -e

echo "🚀 Starting Fast Development Mode..."

# Clean up any previous processes
echo "🧹 Cleaning up previous processes..."
pkill -f "tauri" || true
pkill -f "react-scripts" || true

# Clear caches
echo "🗑️ Clearing caches..."
rm -rf ui/build
rm -rf target
rm -rf node_modules/.cache
rm -rf ui/node_modules/.cache

# Set performance environment variables
export GENERATE_SOURCEMAP=false
export FAST_REFRESH=true
export CHOKIDAR_USEPOLLING=false
export REACT_APP_FAST_DEV=true
export SKIP_PREFLIGHT_CHECK=true
export ESLINT_NO_DEV_ERRORS=true

# Start UI in fast mode
echo "⚡ Starting React dev server in fast mode..."
cd ui
npm run start:fast &
UI_PID=$!
cd ..

# Start Tauri in fast mode
echo "⚡ Starting Tauri in fast mode..."
cargo tauri dev --no-watch &
TAURI_PID=$!

echo "✅ Fast development environment started!"
echo "📊 UI PID: $UI_PID"
echo "📊 Tauri PID: $TAURI_PID"
echo ""
echo "🛑 To stop: Ctrl+C or run 'pkill -f tauri && pkill -f react-scripts'"

# Wait for both processes
wait 