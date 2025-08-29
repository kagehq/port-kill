#!/bin/bash

echo "🍎 Building Port Kill for macOS..."

# Ensure we're using the macOS Cargo.toml
if [ -f "Cargo.linux.toml" ]; then
    echo "⚠️  Detected Linux Cargo.toml, switching to macOS configuration..."
    # Backup current Cargo.toml if it's the Linux version
    if grep -q "tray-item" Cargo.toml 2>/dev/null; then
        mv Cargo.toml Cargo.linux.toml.backup
        echo "📦 Restored macOS Cargo.toml"
    fi
fi

# Build the macOS version
echo "🔨 Building with cargo..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ macOS version built successfully!"
    echo "📦 Binary location: ./target/release/port-kill"
    echo "📦 Console binary: ./target/release/port-kill-console"
    echo ""
    echo "🧪 To test:"
    echo "   ./target/release/port-kill --console --ports 3000,8000 --verbose"
else
    echo "❌ Build failed!"
    exit 1
fi
