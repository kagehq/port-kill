#!/bin/bash

echo "🔍 Linux Debug Script for Port Kill"
echo "=================================="
echo ""

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "⚠️  This script is designed for Linux systems"
    echo "   Current OS: $OSTYPE"
    echo "   Please run this on a Linux system"
    exit 1
fi

echo "✅ Running on Linux system"
echo ""

# Check for required packages
echo "📦 Checking for required packages..."
echo ""

# Check pkg-config
if command -v pkg-config &> /dev/null; then
    echo "✅ pkg-config is installed"
else
    echo "❌ pkg-config is NOT installed"
    echo "   Install with: sudo apt-get install pkg-config"
fi

# Check GTK packages
echo ""
echo "🔍 Checking GTK packages..."

# Check if we can find GTK libraries
if pkg-config --exists gtk+-3.0 2>/dev/null; then
    echo "✅ GTK+ 3.0 is available"
else
    echo "❌ GTK+ 3.0 is NOT available"
    echo "   Install with: sudo apt-get install libgtk-3-dev"
fi

if pkg-config --exists gdk-pixbuf-2.0 2>/dev/null; then
    echo "✅ GDK Pixbuf is available"
else
    echo "❌ GDK Pixbuf is NOT available"
    echo "   Install with: sudo apt-get install libgdk-pixbuf2.0-dev"
fi

if pkg-config --exists atk 2>/dev/null; then
    echo "✅ ATK is available"
else
    echo "❌ ATK is NOT available"
    echo "   Install with: sudo apt-get install libatk1.0-dev"
fi

# Check display environment
echo ""
echo "🖥️  Checking display environment..."

if [[ -n "$DISPLAY" ]]; then
    echo "✅ DISPLAY is set: $DISPLAY"
else
    echo "❌ DISPLAY is NOT set"
    echo "   This might cause tray icon issues"
fi

if [[ -n "$WAYLAND_DISPLAY" ]]; then
    echo "✅ WAYLAND_DISPLAY is set: $WAYLAND_DISPLAY"
else
    echo "ℹ️  WAYLAND_DISPLAY is NOT set (using X11)"
fi

# Check if we're in a desktop environment
if [[ -n "$XDG_CURRENT_DESKTOP" ]]; then
    echo "✅ Desktop environment: $XDG_CURRENT_DESKTOP"
else
    echo "⚠️  No desktop environment detected"
fi

# Check for lsof
echo ""
echo "🔧 Checking system tools..."

if command -v lsof &> /dev/null; then
    echo "✅ lsof is available"
else
    echo "❌ lsof is NOT available"
    echo "   Install with: sudo apt-get install lsof"
fi

# Check for docker
if command -v docker &> /dev/null; then
    echo "✅ Docker is available"
else
    echo "ℹ️  Docker is NOT available (optional)"
fi

# Test lsof functionality
echo ""
echo "🧪 Testing lsof functionality..."

# Test if we can list processes on common dev ports
echo "   Testing port 3000..."
if lsof -i :3000 -sTCP:LISTEN 2>/dev/null | head -5; then
    echo "   ✅ lsof can access port 3000"
else
    echo "   ℹ️  No processes on port 3000 (this is normal)"
fi

echo "   Testing port 8000..."
if lsof -i :8000 -sTCP:LISTEN 2>/dev/null | head -5; then
    echo "✅ lsof can access port 8000"
else
    echo "ℹ️  No processes on port 8000 (this is normal)"
fi

echo ""
echo "🎯 Recommendations:"
echo "=================="

if ! command -v pkg-config &> /dev/null; then
    echo "1. Install pkg-config: sudo apt-get install pkg-config"
fi

if ! pkg-config --exists gtk+-3.0 2>/dev/null; then
    echo "2. Install GTK packages: sudo apt-get install libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev libxdo-dev"
fi

if [[ -z "$DISPLAY" ]]; then
    echo "3. Set DISPLAY environment: export DISPLAY=:0"
fi

echo "4. Try running Port Kill: ./run-linux.sh --console --ports 3000,8000 --verbose"
echo "5. If tray doesn't work, use console mode: ./run-linux.sh --console --ports 3000,8000"

echo ""
echo "🔍 For more detailed debugging, run:"
echo "   RUST_LOG=debug ./run-linux.sh --console --ports 3000,8000"
