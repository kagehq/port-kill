#!/bin/bash

# Port Kill Release Script
# This script creates a new release by creating a tag and pushing it to GitHub

set -e

if [ $# -eq 0 ]; then
    echo "🚀 Port Kill Release Script"
    echo "=========================="
    echo ""
    echo "Usage: ./release.sh <version>"
    echo ""
    echo "Examples:"
    echo "  ./release.sh 0.1.0    # Creates v0.1.0 release"
    echo "  ./release.sh 0.2.0    # Creates v0.2.0 release"
    echo "  ./release.sh 1.0.0    # Creates v1.0.0 release"
    echo ""
    echo "This will:"
    echo "  1. Create a git tag v<version>"
    echo "  2. Push the tag to GitHub"
    echo "  3. Trigger automatic release creation"
    echo "  4. Build and upload binaries for all platforms"
    echo ""
    exit 1
fi

VERSION=$1

# Validate version format (simple check)
if [[ ! $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "❌ Invalid version format: $VERSION"
    echo "   Please use semantic versioning (e.g., 0.1.0, 1.0.0, 2.1.3)"
    exit 1
fi

TAG="v$VERSION"

echo "🚀 Creating release for Port Kill $VERSION"
echo "=========================================="
echo ""

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "⚠️  Warning: You're not on the main branch (current: $CURRENT_BRANCH)"
    echo "   It's recommended to create releases from the main branch"
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Release cancelled"
        exit 1
    fi
fi

# Check if tag already exists
if git tag -l | grep -q "^$TAG$"; then
    echo "❌ Tag $TAG already exists!"
    echo "   Please use a different version or delete the existing tag:"
    echo "   git tag -d $TAG && git push origin :refs/tags/$TAG"
    exit 1
fi

# Check if there are uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  Warning: You have uncommitted changes"
    echo "   It's recommended to commit all changes before creating a release"
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Release cancelled"
        exit 1
    fi
fi

echo "📋 Release Summary:"
echo "   Version: $VERSION"
echo "   Tag: $TAG"
echo "   Branch: $CURRENT_BRANCH"
echo ""

# Create the tag
echo "🏷️  Creating tag $TAG..."
git tag $TAG

if [ $? -eq 0 ]; then
    echo "✅ Tag created successfully"
else
    echo "❌ Failed to create tag"
    exit 1
fi

# Push the tag
echo "📤 Pushing tag to GitHub..."
git push origin $TAG

if [ $? -eq 0 ]; then
    echo "✅ Tag pushed successfully"
else
    echo "❌ Failed to push tag"
    echo "   You may need to:"
    echo "   1. Check your GitHub credentials"
    echo "   2. Ensure you have push access to the repository"
    exit 1
fi

echo ""
echo "🎉 Release process started!"
echo ""
echo "📋 What happens next:"
echo "   1. GitHub Actions will automatically create a release"
echo "   2. Binaries will be built for all platforms (macOS, Linux, Windows)"
echo "   3. Release assets will be uploaded"
echo "   4. Install scripts will become available"
echo ""
echo "🔗 Monitor progress:"
echo "   - GitHub Actions: https://github.com/treadiehq/port-kill/actions"
echo "   - Releases: https://github.com/treadiehq/port-kill/releases"
echo ""
echo "⏱️  This process typically takes 5-10 minutes to complete."
echo ""
echo "�� Happy releasing!"
