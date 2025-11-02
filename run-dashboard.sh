#!/bin/bash

# Port Kill Dashboard Launcher
# This script builds the Rust application and starts the dashboard

set -e

echo "🚀 Starting Port Kill Dashboard..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the port-kill root directory"
    exit 1
fi

# Build the Rust application
echo "🔨 Building Port Kill Rust application..."
cargo build --release

# Check if build was successful
if [ ! -f "target/release/port-kill-console" ]; then
    echo "❌ Error: Failed to build port-kill-console binary"
    exit 1
fi

echo "✅ Rust application built successfully"

# Check if dashboard directory exists
if [ ! -d "dashboard" ]; then
    echo "❌ Error: Dashboard directory not found. Please ensure dashboard exists."
    exit 1
fi

# Install dashboard dependencies if needed
if [ ! -d "dashboard/node_modules" ]; then
    echo "📦 Installing dashboard dependencies..."
    cd dashboard
    npm install
    cd ..
fi

# Start the dashboard
echo "🌐 Starting dashboard on http://localhost:3002..."
cd dashboard
npm run dev
