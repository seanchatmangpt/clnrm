#!/bin/bash
# Quick Test Suite Runner for Optimus Prime Platform
# Runs the 3 core tests: API endpoints, Child mode, Executive mode

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🚀 Optimus Prime Platform - Quick Test Suite"
echo "=============================================="
echo ""
echo "Running 3 core tests:"
echo "  1. API Endpoints"
echo "  2. Child Mode"
echo "  3. Executive Mode"
echo ""

# Check if clnrm is installed
if ! command -v clnrm &> /dev/null; then
    echo "❌ Error: clnrm not found. Please install it first:"
    echo "   cargo install clnrm"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo "❌ Error: Docker is not running. Please start Docker first."
    exit 1
fi

echo "✅ Prerequisites checked"
echo ""

# Test 1: API Endpoints
echo "📝 Test 1/3: API Endpoints"
echo "-------------------------"
if clnrm run api-endpoints.clnrm.toml; then
    echo "✅ API Endpoints test passed"
else
    echo "❌ API Endpoints test failed"
    exit 1
fi
echo ""

# Test 2: Child Mode
echo "📝 Test 2/3: Child Mode"
echo "-------------------------"
if clnrm run child-mode.clnrm.toml; then
    echo "✅ Child Mode test passed"
else
    echo "❌ Child Mode test failed"
    exit 1
fi
echo ""

# Test 3: Executive Mode
echo "📝 Test 3/3: Executive Mode"
echo "-------------------------"
if clnrm run executive-mode.clnrm.toml; then
    echo "✅ Executive Mode test passed"
else
    echo "❌ Executive Mode test failed"
    exit 1
fi
echo ""

echo "=============================================="
echo "✅ All 3 quick suite tests passed!"
echo "=============================================="
echo ""
echo "To run additional tests:"
echo "  clnrm run admin-dashboard.clnrm.toml"
echo "  clnrm run integration-full.clnrm.toml"
echo "  clnrm run performance.clnrm.toml"
echo "  clnrm run security.clnrm.toml"
