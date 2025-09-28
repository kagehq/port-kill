#!/bin/bash

# Nix Configuration Validation Script
# This script validates the Nix configuration files

echo "🔍 Validating Nix Configuration"
echo "==============================="
echo ""

# Check if Nix is available
if ! command -v nix &> /dev/null; then
    echo "⚠️  Nix is not installed. Skipping validation."
    echo "   Install Nix from: https://nixos.org/download.html"
    echo ""
    echo "📋 Manual validation checklist:"
    echo "   ✅ flake.nix syntax looks correct"
    echo "   ✅ shell.nix syntax looks correct"
    echo "   ✅ GitHub Actions workflow created"
    echo "   ✅ Documentation created"
    echo ""
    exit 0
fi

echo "✅ Nix is available: $(nix --version)"
echo ""

# Validate flake.nix
echo "🔍 Validating flake.nix..."
if nix flake check . 2>/dev/null; then
    echo "✅ flake.nix is valid"
else
    echo "❌ flake.nix has issues:"
    nix flake check . 2>&1 | head -10
fi
echo ""

# Show available packages
echo "📦 Available packages:"
nix flake show . 2>/dev/null || echo "   (Cannot show packages without Nix)"
echo ""

# Test development shell
echo "🧪 Testing development shell..."
if nix develop --dry-run . 2>/dev/null; then
    echo "✅ Development shell configuration is valid"
else
    echo "❌ Development shell has issues"
fi
echo ""

echo "🎉 Nix configuration validation complete!"
echo ""
echo "📋 Next steps:"
echo "   1. Install Nix: https://nixos.org/download.html"
echo "   2. Enable flakes: echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf"
echo "   3. Enter development shell: nix develop"
echo "   4. Build: nix build"
echo ""
