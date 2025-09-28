#!/bin/bash

# Simple Nix test script
echo "🧪 Testing Nix Configuration"
echo "============================"
echo ""

# Check if Nix is available
if ! command -v nix &> /dev/null; then
    echo "❌ Nix is not installed"
    echo "   Install Nix from: https://nixos.org/download.html"
    echo "   Then run: echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf"
    exit 1
fi

echo "✅ Nix is available: $(nix --version)"
echo ""

# Test flake check
echo "🔍 Checking flake configuration..."
if nix flake check . 2>/dev/null; then
    echo "✅ Flake configuration is valid"
else
    echo "❌ Flake configuration has issues:"
    nix flake check . 2>&1 | head -10
    exit 1
fi
echo ""

# Test development shell
echo "🧪 Testing development shell..."
if nix develop --dry-run . 2>/dev/null; then
    echo "✅ Development shell configuration is valid"
else
    echo "❌ Development shell has issues"
    exit 1
fi
echo ""

# Test build (dry run)
echo "🔨 Testing build (dry run)..."
if nix build --dry-run .#default 2>/dev/null; then
    echo "✅ Build configuration is valid"
else
    echo "❌ Build configuration has issues:"
    nix build --dry-run .#default 2>&1 | head -10
    exit 1
fi
echo ""

echo "🎉 All Nix tests passed!"
echo ""
echo "📋 Next steps:"
echo "   1. Enter development shell: nix develop"
echo "   2. Build the project: nix build"
echo "   3. Run the binary: ./result/bin/port-kill --help"
echo ""
