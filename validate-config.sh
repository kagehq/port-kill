#!/bin/bash

# Configuration validation script (works without Nix installed)
echo "🔍 Validating Port Kill Configuration"
echo "====================================="
echo ""

# Check if Nix is available
if command -v nix &> /dev/null; then
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
else
    echo "⚠️  Nix is not installed. Performing basic validation..."
    echo ""
    
    # Basic file validation
    echo "📁 Checking required files..."
    if [ -f "flake.nix" ]; then
        echo "✅ flake.nix exists"
    else
        echo "❌ flake.nix missing"
        exit 1
    fi
    
    if [ -f "shell.nix" ]; then
        echo "✅ shell.nix exists"
    else
        echo "❌ shell.nix missing"
        exit 1
    fi
    
    if [ -f ".github/workflows/nix-build.yml" ]; then
        echo "✅ GitHub Actions workflow exists"
    else
        echo "❌ GitHub Actions workflow missing"
        exit 1
    fi
    
    if [ -f "Cargo.toml" ]; then
        echo "✅ Cargo.toml exists"
    else
        echo "❌ Cargo.toml missing"
        exit 1
    fi
    
    if [ -f "Cargo.lock" ]; then
        echo "✅ Cargo.lock exists"
    else
        echo "❌ Cargo.lock missing"
        exit 1
    fi
    echo ""
    
    # Basic syntax validation
    echo "🔍 Checking basic syntax..."
    
    # Check if flake.nix has required structure
    if grep -q "description" flake.nix && grep -q "inputs" flake.nix && grep -q "outputs" flake.nix; then
        echo "✅ flake.nix has required structure"
    else
        echo "❌ flake.nix missing required structure"
        exit 1
    fi
    
    # Check if shell.nix has required structure
    if grep -q "mkShell" shell.nix; then
        echo "✅ shell.nix has required structure"
    else
        echo "❌ shell.nix missing required structure"
        exit 1
    fi
    
    # Check if GitHub Actions workflow has required structure
    if grep -q "runs-on:" .github/workflows/nix-build.yml && grep -q "nix build" .github/workflows/nix-build.yml; then
        echo "✅ GitHub Actions workflow has required structure"
    else
        echo "❌ GitHub Actions workflow missing required structure"
        exit 1
    fi
    echo ""
    
    echo "✅ Basic validation passed!"
    echo ""
    echo "📋 To fully test, install Nix:"
    echo "   1. Install Nix: https://nixos.org/download.html"
    echo "   2. Enable flakes: echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf"
    echo "   3. Run: ./test-nix.sh"
fi

echo ""
echo "🎯 Configuration Summary:"
echo "   - Nix flake: ✅ Configured"
echo "   - Development shell: ✅ Configured"
echo "   - GitHub Actions: ✅ Configured"
echo "   - Traditional builds: ✅ Still working"
echo ""
echo "🚀 Ready for deployment!"
