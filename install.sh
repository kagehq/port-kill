#!/bin/bash

# Installation script for port-kill
# This script builds and installs the port-kill application for your platform

set -e

echo "🚀 Port Kill Installation Script"
echo "================================"
echo ""

# Usage/help
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  echo "Usage: ./install.sh [--dashboard] [--all]"
  echo ""
  echo "Options:"
  echo "  --dashboard   Install dashboard dependencies only (Nuxt)"
  echo "  --all         Build Rust binary and install dashboard dependencies"
  echo ""
  exit 0
fi

# Flags
DASHBOARD_ONLY=false
ALL_SETUP=false

for arg in "$@"; do
  case "$arg" in
    --dashboard)
      DASHBOARD_ONLY=true
      ;;
    --all)
      ALL_SETUP=true
      ;;
  esac
done

# Detect the operating system
if [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macOS"
    BUILD_SCRIPT="./build-macos.sh"
    RUN_SCRIPT="./run.sh"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="Linux"
    BUILD_SCRIPT="./build-linux.sh"
    RUN_SCRIPT="./run-linux.sh"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    PLATFORM="Windows"
    BUILD_SCRIPT="build-windows.bat"
    RUN_SCRIPT="run-windows.bat"
else
    echo "⚠️  Unknown operating system: $OSTYPE"
    echo "   Attempting generic build..."
    PLATFORM="Unknown"
    BUILD_SCRIPT="cargo build --release"
    RUN_SCRIPT="./target/release/port-kill"
fi

echo "📋 Detected Platform: $PLATFORM"
echo "🔨 Build Script: $BUILD_SCRIPT"
echo "▶️  Run Script: $RUN_SCRIPT"
echo ""

# If dashboard-only, skip Rust checks
if [[ "$DASHBOARD_ONLY" == false ]]; then
  # Check if Rust is installed
  if ! command -v cargo &> /dev/null; then
      echo "❌ Rust is not installed or not in PATH"
      echo ""
      echo "📦 Please install Rust first:"
      echo "   Visit: https://rustup.rs/"
      echo ""
      echo "   Or run:"
      echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
      echo ""
      exit 1
  fi

  echo "✅ Rust detected: $(cargo --version)"
  echo ""
fi

# Build the application using platform-specific script
if [[ "$DASHBOARD_ONLY" == false ]]; then
  echo "🔨 Building port-kill for $PLATFORM..."

  if [[ "$PLATFORM" == "Windows" ]]; then
      # For Windows, we need to use the batch file
      if command -v cmd &> /dev/null; then
          cmd //c "$BUILD_SCRIPT"
      else
          echo "❌ Windows build script not available or cmd not found"
          echo "   Please run: $BUILD_SCRIPT"
          exit 1
      fi
  else
      # For Unix-like systems, use the shell script
      if [ -f "$BUILD_SCRIPT" ]; then
          bash "$BUILD_SCRIPT"
      else
          echo "❌ Build script not found: $BUILD_SCRIPT"
          echo "   Falling back to generic build..."
          cargo build --release
      fi
  fi

  echo ""
  echo "✅ Build completed successfully!"
  echo ""
fi

# Check if the binary was created
if [[ "$DASHBOARD_ONLY" == true ]] || [ -f "target/release/port-kill" ] || [ -f "target/release/port-kill.exe" ]; then
    echo "📦 Binary created successfully!"
    echo ""
    echo "🎯 Quick Start:"
    if [[ "$DASHBOARD_ONLY" == false ]]; then
      echo "   $RUN_SCRIPT"
    else
      echo "   cd dashboard && npm run dev"
    fi
    echo ""
    echo "🔧 Common Usage Examples:"
    echo ""
    echo "   # Default monitoring (ports 2000-6000):"
    echo "   $RUN_SCRIPT"
    echo ""
    echo "   # Monitor specific ports:"
    echo "   $RUN_SCRIPT --ports 3000,8000,8080"
    echo ""
    echo "   # Console mode with verbose logging:"
    echo "   $RUN_SCRIPT --console --verbose"
    echo ""
    echo "   # Ignore system processes:"
    echo "   $RUN_SCRIPT --ignore-ports 5353,5000,7000 --ignore-processes Chrome,ControlCe"
    echo ""
    echo "   # Docker support:"
    echo "   $RUN_SCRIPT --docker --ports 3000,8000"
    echo ""
    if [[ "$DASHBOARD_ONLY" == false ]]; then
      echo "📖 For more options:"
      echo "   $RUN_SCRIPT --help"
      echo ""
    fi

    # Dashboard setup if requested
    if [[ "$DASHBOARD_ONLY" == true || "$ALL_SETUP" == true ]]; then
      echo "🧩 Dashboard setup (Nuxt)"
      if ! command -v npm &> /dev/null; then
        echo "❌ npm not found. Please install Node.js (v18+) first: https://nodejs.org/"
        exit 1
      fi
      pushd dashboard >/dev/null
      echo "📦 Installing dashboard dependencies..."
      npm install
      echo "✅ Done."
      echo ""
      echo "▶️  Start dashboard:"
      echo "   npm run dev   # http://localhost:3002"
      echo ""
      echo "🔨 Build dashboard (Nitro):"
      echo "   npm run build  # server output in .output/"
      echo ""
      echo "📦 Static generate (optional):"
      echo "   npm run generate  # static output in dist/"
      popd >/dev/null
    fi

    echo "🎉 Installation complete! Happy coding! 🚀"
else
    echo "❌ Error: Binary not found"
    echo ""
    echo "💡 Troubleshooting:"
    echo "   1. Check if Rust is properly installed: cargo --version"
    echo "   2. Try building manually: $BUILD_SCRIPT"
    echo "   3. Check for error messages above"
    echo ""
    exit 1
fi
