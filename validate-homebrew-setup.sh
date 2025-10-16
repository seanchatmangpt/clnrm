#!/bin/bash
# Homebrew Setup Validation Script
# This script validates the Homebrew formula and setup

set -e

echo "🍺 Validating Homebrew Setup for Cleanroom CLI"
echo "=============================================="

# Check if we're in the right directory
if [ ! -f "homebrew/Formula/clnrm.rb" ]; then
    echo "❌ Error: homebrew/Formula/clnrm.rb not found"
    echo "   Make sure you're running this from the project root"
    exit 1
fi

echo "✅ Found formula file: homebrew/Formula/clnrm.rb"

# Validate Ruby syntax
echo -e "\n📋 Step 1: Validating Ruby syntax..."
if ruby -c homebrew/Formula/clnrm.rb; then
    echo "✅ Ruby syntax is valid"
else
    echo "❌ Ruby syntax error in formula"
    exit 1
fi

# Check formula structure
echo -e "\n📋 Step 2: Checking formula structure..."
if grep -q "class Clnrm < Formula" homebrew/Formula/clnrm.rb; then
    echo "✅ Formula class definition found"
else
    echo "❌ Missing formula class definition"
    exit 1
fi

if grep -q "desc " homebrew/Formula/clnrm.rb; then
    echo "✅ Description found"
else
    echo "❌ Missing description"
    exit 1
fi

if grep -q "homepage " homebrew/Formula/clnrm.rb; then
    echo "✅ Homepage found"
else
    echo "❌ Missing homepage"
    exit 1
fi

if grep -q "url " homebrew/Formula/clnrm.rb; then
    echo "✅ URL found"
else
    echo "❌ Missing URL"
    exit 1
fi

if grep -q "sha256 " homebrew/Formula/clnrm.rb; then
    echo "✅ SHA256 checksum found"
else
    echo "❌ Missing SHA256 checksum"
    exit 1
fi

if grep -q "license " homebrew/Formula/clnrm.rb; then
    echo "✅ License found"
else
    echo "❌ Missing license"
    exit 1
fi

# Check bottle support
echo -e "\n📋 Step 3: Checking bottle support..."
if grep -q "bottle do" homebrew/Formula/clnrm.rb; then
    echo "✅ Bottle support found"
else
    echo "❌ Missing bottle support"
    exit 1
fi

# Check dependencies
echo -e "\n📋 Step 4: Checking dependencies..."
if grep -q "depends_on \"rust\"" homebrew/Formula/clnrm.rb; then
    echo "✅ Rust dependency found"
else
    echo "❌ Missing Rust dependency"
    exit 1
fi

# Check install method
echo -e "\n📋 Step 5: Checking install method..."
if grep -q "def install" homebrew/Formula/clnrm.rb; then
    echo "✅ Install method found"
else
    echo "❌ Missing install method"
    exit 1
fi

if grep -q "cargo.*install" homebrew/Formula/clnrm.rb; then
    echo "✅ Cargo install command found"
else
    echo "❌ Missing cargo install command"
    exit 1
fi

# Check test method
echo -e "\n📋 Step 6: Checking test method..."
if grep -q "def test" homebrew/Formula/clnrm.rb; then
    echo "✅ Test method found"
else
    echo "❌ Missing test method"
    exit 1
fi

if grep -q "clnrm.*--version" homebrew/Formula/clnrm.rb; then
    echo "✅ Version test found"
else
    echo "❌ Missing version test"
    exit 1
fi

# Check homebrew-core formula
echo -e "\n📋 Step 7: Checking homebrew-core formula..."
if [ -f "homebrew/homebrew-core-formula/clnrm.rb" ]; then
    echo "✅ Homebrew-core formula found"
    if ruby -c homebrew/homebrew-core-formula/clnrm.rb; then
        echo "✅ Homebrew-core formula syntax is valid"
    else
        echo "❌ Homebrew-core formula syntax error"
        exit 1
    fi
else
    echo "❌ Homebrew-core formula not found"
    exit 1
fi

# Check GitHub Actions workflow
echo -e "\n📋 Step 8: Checking GitHub Actions workflow..."
if [ -f ".github/workflows/homebrew-release.yml" ]; then
    echo "✅ GitHub Actions workflow found"
else
    echo "❌ GitHub Actions workflow not found"
    exit 1
fi

# Check documentation
echo -e "\n📋 Step 9: Checking documentation..."
if [ -f "docs/HOMEBREW_RELEASE.md" ]; then
    echo "✅ Homebrew release documentation found"
else
    echo "❌ Homebrew release documentation not found"
    exit 1
fi

# Check updated README
echo -e "\n📋 Step 10: Checking updated README..."
if grep -q "brew tap seanchatmangpt/clnrm" README.md; then
    echo "✅ Homebrew installation instructions found in README"
else
    echo "❌ Homebrew installation instructions not found in README"
    exit 1
fi

# Check updated CLI guide
echo -e "\n📋 Step 11: Checking updated CLI guide..."
if grep -q "brew tap seanchatmangpt/clnrm" docs/CLI_GUIDE.md; then
    echo "✅ Homebrew installation instructions found in CLI guide"
else
    echo "❌ Homebrew installation instructions not found in CLI guide"
    exit 1
fi

echo -e "\n🎉 All Homebrew setup validation checks passed!"
echo ""
echo "📋 Summary:"
echo "  ✅ Formula syntax is valid"
echo "  ✅ Formula structure is complete"
echo "  ✅ Bottle support is configured"
echo "  ✅ Dependencies are specified"
echo "  ✅ Install method is implemented"
echo "  ✅ Test method is implemented"
echo "  ✅ Homebrew-core formula is ready"
echo "  ✅ GitHub Actions workflow is configured"
echo "  ✅ Documentation is complete"
echo "  ✅ README and CLI guide are updated"
echo ""
echo "🚀 Next steps:"
echo "  1. Create a GitHub release to trigger bottle building"
echo "  2. Update formula with actual SHA256 hashes from release"
echo "  3. Create homebrew-clnrm tap repository"
echo "  4. Copy formula to tap repository"
echo "  5. Test installation: brew tap seanchatmangpt/clnrm && brew install clnrm"
echo ""
echo "📚 See docs/HOMEBREW_RELEASE.md for detailed release process"
