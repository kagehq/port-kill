#!/bin/bash

# Port Kill Release Installer
# Downloads and installs the latest release for your platform

set -e

REPO="kagehq/port-kill"
LATEST_RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"

echo "🚀 Port Kill Release Installer"
echo "=============================="
echo ""

# Detect platform
PLATFORM=""
BINARY_NAME=""
CONSOLE_BINARY_NAME=""

if [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macos"
    BINARY_NAME="port-kill-macos"
    CONSOLE_BINARY_NAME="port-kill-console-macos"
    echo "✅ Detected platform: macOS"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="linux"
    BINARY_NAME="port-kill-linux"
    CONSOLE_BINARY_NAME="port-kill-console-linux"
    echo "✅ Detected platform: Linux"
else
    echo "❌ Unsupported platform: $OSTYPE"
    echo "   Please download manually from: https://github.com/$REPO/releases"
    exit 1
fi

# Get latest release info
echo "📡 Fetching latest release information..."
LATEST_TAG=$(curl -s "$LATEST_RELEASE_URL" | grep '"tag_name"' | cut -d'"' -f4)

if [[ -z "$LATEST_TAG" ]]; then
    echo "❌ No releases found or failed to get latest release information"
    echo ""
    echo "📋 No releases are currently available. You have two options:"
    echo ""
    echo "   1. 🏗️  Build from source (recommended):"
    echo "      ./install.sh"
    echo ""
    echo "   2. 📦 Wait for a release to be published:"
    echo "      Visit: https://github.com/$REPO/releases"
    echo ""
    echo "   To create a release, the repository owner needs to:"
    echo "   - Go to GitHub repository"
    echo "   - Click 'Releases' → 'Create a new release'"
    echo "   - Set tag (e.g., v0.1.0) and publish"
    echo ""
    exit 1
fi

echo "📦 Latest release: $LATEST_TAG"

# Create installation directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

echo "📁 Installing to: $INSTALL_DIR"

# Download and install binary
echo "⬇️  Downloading $BINARY_NAME..."
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$BINARY_NAME"
if ! curl -L -o "$INSTALL_DIR/port-kill" "$DOWNLOAD_URL"; then
    echo "❌ Failed to download $BINARY_NAME"
    echo "   URL: $DOWNLOAD_URL"
    echo "   Please check if the release assets are available"
    exit 1
fi
chmod +x "$INSTALL_DIR/port-kill"

# Download and install console binary
echo "⬇️  Downloading $CONSOLE_BINARY_NAME..."
CONSOLE_DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$CONSOLE_BINARY_NAME"
if ! curl -L -o "$INSTALL_DIR/port-kill-console" "$CONSOLE_DOWNLOAD_URL"; then
    echo "❌ Failed to download $CONSOLE_BINARY_NAME"
    echo "   URL: $CONSOLE_DOWNLOAD_URL"
    echo "   Please check if the release assets are available"
    exit 1
fi
chmod +x "$INSTALL_DIR/port-kill-console"

echo ""
echo "✅ Installation complete!"
echo ""
echo "📋 Usage:"
echo "   System tray mode: port-kill --ports 3000,8000"
echo "   Console mode:     port-kill-console --console --ports 3000,8000"
echo ""
echo "🔧 Make sure $INSTALL_DIR is in your PATH:"
echo "   export PATH=\"\$PATH:$INSTALL_DIR\""
echo ""
echo "📖 For more options: port-kill --help"
