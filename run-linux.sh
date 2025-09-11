#!/bin/bash

# Port Kill - Linux Run Script
# This script runs the port-kill application on Linux with logging enabled
# Usage: ./run-linux.sh [options]
# Examples:
#   ./run-linux.sh                           # Default: ports 2000-6000
#   ./run-linux.sh --start-port 3000         # Ports 3000-6000
#   ./run-linux.sh --end-port 8080           # Ports 2000-8080
#   ./run-linux.sh --ports 3000,8000,8080    # Specific ports only
#   ./run-linux.sh --console                 # Run in console mode
#   ./run-linux.sh --verbose                 # Enable verbose logging
#   ./run-linux.sh --docker                  # Enable Docker container monitoring
#   ./run-linux.sh --docker --ports 3000,3001 # Monitor specific ports with Docker
#   ./run-linux.sh --show-pid                # Show process IDs in output
#   ./run-linux.sh --console --show-pid      # Console mode with PIDs shown

echo "🐧 Starting Port Kill on Linux..."
echo "📊 Status bar icon should appear shortly"
echo ""

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "⚠️  Warning: This script is designed for Linux systems"
    echo "   Current OS: $OSTYPE"
    echo "   For macOS, use: ./run.sh"
    echo ""
fi

# Check if the Linux application is built
if [ ! -f "./target/release/port-kill" ]; then
    echo "❌ Linux application not built. Running Linux build first..."
    ./build-linux.sh
    if [ $? -ne 0 ]; then
        echo "❌ Linux build failed!"
        exit 1
    fi
fi

# Run the Linux application with logging and pass through all arguments
echo "🚀 Starting Port Kill..."
echo "📊 If system tray is not available, it will automatically switch to console mode"
echo ""

RUST_LOG=info ./target/release/port-kill "$@"
