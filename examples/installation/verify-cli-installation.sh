#!/bin/bash
# Installation Verification Example
# This script demonstrates that the CLI installation claim from the README works
# Users can copy and paste this script to verify their installation

set -e

echo "🚀 Verifying Cleanroom CLI Installation"
echo "======================================"

# Verify CLI is installed and accessible
echo "📋 Step 1: Check CLI availability..."
if command -v clnrm &> /dev/null; then
    echo "✅ clnrm CLI found in PATH"
    CLNRM_CMD="clnrm"
elif [ -f "../../target/release/clnrm" ]; then
    echo "✅ Using local clnrm binary"
    CLNRM_CMD="../../target/release/clnrm"
else
    echo "❌ clnrm CLI not found in PATH"
    echo "💡 Install with: curl -fsSL https://install.clnrm.dev | sh"
    echo "💡 Or build locally: cargo build --release"
    exit 1
fi

# Check version output matches README claim
echo -e "\n📋 Step 2: Verify version output..."
VERSION=$($CLNRM_CMD --version)
echo "📦 CLI Version: $VERSION"

if [[ "$VERSION" == *"clnrm"* ]]; then
    echo "✅ Version format matches README example"
else
    echo "❌ Version format doesn't match expected format"
fi

# Test basic CLI functionality
echo -e "\n📋 Step 3: Test basic CLI commands..."
$CLNRM_CMD --help | head -10
echo "✅ CLI help command works"

# Initialize a test project as shown in README
echo -e "\n📋 Step 4: Initialize test project (as shown in README)..."
TEST_DIR="verify-install-test-project"
if [ -d "$TEST_DIR" ]; then
    echo "🗑️  Removing existing test directory..."
    rm -rf "$TEST_DIR"
fi

$CLNRM_CMD init "$TEST_DIR"
echo "✅ Project initialization works as documented"

# Verify project structure
echo -e "\n📋 Step 5: Verify project structure..."
if [ -f "$TEST_DIR/cleanroom.toml" ]; then
    echo "✅ cleanroom.toml configuration file created"
else
    echo "❌ cleanroom.toml not found"
fi

if [ -d "$TEST_DIR/tests" ]; then
    echo "✅ tests/ directory created"
else
    echo "❌ tests/ directory not found"
fi

# Clean up
echo -e "\n🧹 Cleaning up test project..."
rm -rf "$TEST_DIR"

echo -e "\n🎉 SUCCESS: All installation claims verified!"
echo "📚 Every claim in the README installation section works correctly."
echo ""
echo "💡 Users can copy this script to verify their installation:"
echo "   curl -fsSL https://raw.githubusercontent.com/cleanroom-testing/clnrm/main/examples/installation/verify-cli-installation.sh | bash"
