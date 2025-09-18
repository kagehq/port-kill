#!/bin/bash

# Port Kill MCP Server Installer
# Quick setup for MCP server

set -e

echo "🚀 Installing Port Kill MCP Server..."

# Check if we're in the right directory
if [ ! -f "mcp/package.json" ]; then
    echo "❌ Error: Please run this script from the port-kill root directory"
    echo "   Expected to find: mcp/package.json"
    exit 1
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Error: Node.js is required but not installed"
    echo "   Please install Node.js 18+ from https://nodejs.org/"
    exit 1
fi

# Check Node.js version
NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "❌ Error: Node.js 18+ is required, found version $NODE_VERSION"
    echo "   Please upgrade Node.js from https://nodejs.org/"
    exit 1
fi

# Install MCP dependencies
echo "📦 Installing MCP dependencies..."
cd mcp
npm install

# Build the server
echo "🔨 Building MCP server..."
npm run build

echo "✅ MCP server installed successfully!"
echo ""
echo "🚀 To start the MCP server:"
echo "   cd mcp && npm run dev"
echo ""
echo "🌐 For HTTP wrapper (optional):"
echo "   cd mcp && HTTP_PORT=8787 npm run dev"
echo ""
echo "📖 For Cursor integration:"
echo "   The .cursor/mcp.json config is already included"
echo "   Restart Cursor to detect the server"
