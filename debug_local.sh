#!/bin/bash

# Local debugging script to test GitHub Actions conditions
# Run this locally to debug issues before pushing to GitHub

set -e

echo "🔍 Local Debug Script for Port Kill"
echo "==================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Not in port-kill directory (Cargo.toml not found)"
    exit 1
fi

echo "✅ In port-kill directory"
echo ""

# Check Rust installation
echo "🔧 Checking Rust installation..."
if command -v rustc &> /dev/null; then
    echo "✅ Rust installed: $(rustc --version)"
else
    echo "❌ Rust not installed"
    exit 1
fi

if command -v cargo &> /dev/null; then
    echo "✅ Cargo installed: $(cargo --version)"
else
    echo "❌ Cargo not installed"
    exit 1
fi

echo ""

# Check source files
echo "📁 Checking source files..."
if [ -f "src/main_console.rs" ]; then
    echo "✅ src/main_console.rs exists"
else
    echo "❌ src/main_console.rs not found"
    exit 1
fi

if [ -f "src/main_windows.rs" ]; then
    echo "✅ src/main_windows.rs exists"
else
    echo "❌ src/main_windows.rs not found"
    exit 1
fi

echo ""

# Test console binary build
echo "🔨 Testing console binary build..."
echo "Running: cargo build --release --bin port-kill-console"
if cargo build --release --bin port-kill-console; then
    echo "✅ Console binary build successful"
else
    echo "❌ Console binary build failed"
    exit 1
fi

# Check if binary was created
if [ -f "./target/release/port-kill-console" ]; then
    echo "✅ Console binary created: ./target/release/port-kill-console"
    ls -la ./target/release/port-kill-console
    file ./target/release/port-kill-console
else
    echo "❌ Console binary not found"
    exit 1
fi

echo ""

# Test console binary execution
echo "🧪 Testing console binary execution..."

echo "Testing --version..."
if ./target/release/port-kill-console --version; then
    echo "✅ --version command successful"
else
    echo "❌ --version command failed"
    exit 1
fi

echo "Testing --help..."
if ./target/release/port-kill-console --help > /dev/null 2>&1; then
    echo "✅ --help command successful"
else
    echo "❌ --help command failed"
    exit 1
fi

echo ""

# Test Windows-specific build (if on macOS/Linux)
if [[ "$OSTYPE" == "darwin"* ]] || [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "🪟 Testing Windows-specific build (cross-compilation)..."
    
    # Check if Windows target is available
    if rustup target list | grep -q "x86_64-pc-windows-msvc"; then
        echo "✅ Windows target available"
        
        # Try to build Windows binary
        echo "Running: cargo build --release --bin port-kill --target x86_64-pc-windows-msvc"
        if cargo build --release --bin port-kill --target x86_64-pc-windows-msvc 2>/dev/null; then
            echo "✅ Windows binary build successful"
        else
            echo "⚠️  Windows binary build failed (expected on non-Windows)"
        fi
    else
        echo "⚠️  Windows target not available (run: rustup target add x86_64-pc-windows-msvc)"
    fi
fi

echo ""
echo "🎉 Local debug completed successfully!"
echo ""
echo "If all tests passed locally but GitHub Actions still fails,"
echo "the issue might be:"
echo "1. Environment differences between local and GitHub Actions"
echo "2. Different Rust/Cargo versions"
echo "3. Missing dependencies in GitHub Actions environment"
echo "4. File permission or path issues"
