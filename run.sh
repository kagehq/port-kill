#!/bin/bash

# Port Kill - Easy Run Script
# This script runs the port-kill application with logging enabled
# Usage: ./run.sh [options]
# Examples:
#   ./run.sh                           # Default: ports 2000-6000
#   ./run.sh --start-port 3000         # Ports 3000-6000
#   ./run.sh --end-port 8080           # Ports 2000-8080
#   ./run.sh --ports 3000,8000,8080    # Specific ports only
#   ./run.sh --console                 # Run in console mode
#   ./run.sh --verbose                 # Enable verbose logging
#   ./run.sh --docker                  # Enable Docker container monitoring
#   ./run.sh --docker --ports 3000,3001 # Monitor specific ports with Docker
#   ./run.sh --show-pid                # Show process IDs in output
#   ./run.sh --console --show-pid      # Console mode with PIDs shown

# Check if console mode is requested
CONSOLE_MODE=false
for arg in "$@"; do
    if [ "$arg" = "--console" ]; then
        CONSOLE_MODE=true
        break
    fi
done

if [ "$CONSOLE_MODE" = true ]; then
    echo "🚀 Starting Port Kill (Console Mode)..."
    echo "📡 Console monitoring started"
    echo ""
    
    # Check if the console application is built
    if [ ! -f "./target/release/port-kill-console" ]; then
        echo "❌ Console application not built. Running build first..."
        cargo build --release
        if [ $? -ne 0 ]; then
            echo "❌ Build failed!"
            exit 1
        fi
    fi
    
    # Run the console application
    RUST_LOG=info ./target/release/port-kill-console "$@"
else
    echo "🚀 Starting Port Kill..."
    echo "📊 Status bar icon should appear shortly"
    echo ""
    
    # Check if the application is built
    if [ ! -f "./target/release/port-kill" ]; then
        echo "❌ Application not built. Running build first..."
        cargo build --release
        if [ $? -ne 0 ]; then
            echo "❌ Build failed!"
            exit 1
        fi
    fi
    
    # Run the GUI application
    RUST_LOG=info ./target/release/port-kill "$@"
fi
