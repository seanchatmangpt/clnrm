#!/bin/bash

# Cleanroom CLI Usage Examples
# This script demonstrates how to use the cleanroom CLI tool

echo "🚀 Cleanroom CLI Usage Examples"
echo "================================="

# Basic usage
echo ""
echo "📋 Basic Commands:"
echo ""

echo "# Check version"
echo "clnrm --version"
echo "# Output: clnrm 1.0.0"
echo ""

echo "# Get help"
echo "clnrm --help"
echo ""

echo "# Initialize a new test project"
echo "clnrm init my-cleanroom-tests"
echo ""

echo "# Validate TOML configuration"
echo "clnrm validate tests/framework/container_lifecycle.toml"
echo "# Output: ✅ Configuration is valid"
echo ""

echo "# Run a single test"
echo "clnrm run tests/framework/container_lifecycle.toml"
echo ""

echo "# Run all tests in directory"
echo "clnrm run tests/"
echo ""

echo "# Run with parallel execution"
echo "clnrm run tests/ --parallel --jobs 4"
echo ""

echo "# Watch mode for development"
echo "clnrm run tests/ --watch"
echo ""

echo "# Interactive debugging"
echo "clnrm run tests/ --interactive"
echo ""

echo "# Generate reports"
echo "clnrm report tests/ --format html > report.html"
echo "clnrm run tests/ --format junit > test-results.xml"
echo ""

echo "# Service management"
echo "clnrm services status"
echo "clnrm services logs test_container --lines 50"
echo "clnrm services restart database"
echo ""

echo "# List available plugins"
echo "clnrm plugins"
echo "# Output: 📦 Available Service Plugins:"
echo "# ✅ generic_container (alpine, ubuntu, debian)"
echo "# ✅ network_tools (curl, wget)"
echo ""

echo "🎯 CI/CD Integration Examples:"
echo ""

echo "# GitHub Actions"
echo "echo '"
echo "- name: Run Cleanroom Tests"
echo "  run: clnrm run tests/ --format junit > test-results.xml"
echo ""
echo "- name: Upload Test Results"
echo "  uses: actions/upload-artifact@v3"
echo "  with:"
echo "    name: test-results"
echo "    path: test-results.xml"
echo "'"
echo ""

echo "# GitLab CI"
echo "echo '"
echo "stages:"
echo "  - test"
echo ""
echo "cleanroom_tests:"
echo "  stage: test"
echo "  script:"
echo "    - clnrm run tests/ --parallel --jobs 8"
echo "  artifacts:"
echo "    reports:"
echo "      junit: test-results.xml"
echo "'"
echo ""

echo "🔍 Advanced Features:"
echo ""

echo "# Debug mode with step-by-step execution"
echo "clnrm run tests/framework/ --debug"
echo ""

echo "# Fail fast mode"
echo "clnrm run tests/ --fail-fast"
echo ""

echo "# Verbose output"
echo "clnrm run tests/ -vvv"
echo ""

echo "📊 Output Formats:"
echo ""

echo "# Human-readable (default)"
echo "clnrm run tests/"
echo ""

echo "# JSON for programmatic processing"
echo "clnrm run tests/ --format json | jq '.'"
echo ""

echo "# JUnit XML for CI systems"
echo "clnrm run tests/ --format junit > test-results.xml"
echo ""

echo "# HTML report for stakeholders"
echo "clnrm report tests/ --format html > integration-report.html"
echo ""

echo "✅ Ready to use cleanroom CLI for framework self-testing!"
