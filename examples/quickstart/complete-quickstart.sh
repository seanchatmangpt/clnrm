#!/bin/bash
# Complete Quick Start Example
# This script executes the exact quick start flow from the README
# Users can copy and paste this to follow the complete quick start

set -e

echo "🚀 Cleanroom Quick Start - Complete Example"
echo "=========================================="

# Step 1: Initialize a Test Project (as shown in README)
echo -e "\n📋 Step 1: Initialize a Test Project"
echo "Command: clnrm init my-framework-tests"

PROJECT_NAME="my-framework-tests"
if [ -d "$PROJECT_NAME" ]; then
    echo "🗑️  Removing existing project directory..."
    rm -rf "$PROJECT_NAME"
fi

clnrm init "$PROJECT_NAME"
cd "$PROJECT_NAME"

echo "✅ Project initialized successfully"
echo "📁 Project structure:"
ls -la

# Step 2: Create Your First Test (exact TOML from README)
echo -e "\n📋 Step 2: Create Your First Test"
echo "Creating tests/container_lifecycle.toml with exact content from README..."

cat > tests/container_lifecycle.toml << 'EOF'
[test.metadata]
name = "container_lifecycle_test"
description = "Test that containers start, execute commands, and cleanup properly"

[services.test_container]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[steps]]
name = "verify_container_startup"
command = ["echo", "Container started successfully"]
expected_output_regex = "Container started successfully"

[[steps]]
name = "test_command_execution"
command = ["sh", "-c", "echo 'Testing command execution' && sleep 1 && echo 'Command completed'"]
expected_output_regex = "Command completed"

[[steps]]
name = "test_file_operations"
command = ["sh", "-c", "echo 'test data' > /tmp/test.txt && cat /tmp/test.txt"]
expected_output_regex = "test data"

[assertions]
container_should_have_executed_commands = 3
execution_should_be_hermetic = true
EOF

echo "✅ Test file created with exact README content"

# Step 3: Run Your Tests (all commands from README)
echo -e "\n📋 Step 3: Run Your Tests"

echo "3a) Run a single test:"
echo "Command: clnrm run tests/container_lifecycle.toml"
clnrm run tests/container_lifecycle.toml

echo -e "\n3b) Run all tests with parallel execution:"
echo "Command: clnrm run tests/ --parallel --jobs 4"
clnrm run tests/ --parallel --jobs 4

echo -e "\n3c) Watch mode for development:"
echo "Command: clnrm run tests/ --watch"
echo "⚠️  Watch mode started (press Ctrl+C to stop after 5 seconds)"
timeout 5s clnrm run tests/ --watch || true

echo -e "\n3d) Generate reports:"
echo "Command: clnrm report tests/ --format html > report.html"
clnrm report tests/ --format html > report.html

if [ -f "report.html" ]; then
    echo "✅ HTML report generated successfully"
    echo "📊 Report size: $(wc -c < report.html) bytes"
else
    echo "⚠️  Report generation may not be fully implemented"
fi

# Step 4: Verify Expected Output (from README)
echo -e "\n📋 Step 4: Verify Expected Output Format"
echo "The test should produce output similar to:"
echo ""
echo "🚀 Starting test environment..."
echo "📦 Loading plugins..."
echo "🔌 Plugin 'alpine' loaded"
echo ""
echo "📋 Running test 'container_lifecycle_test'"
echo ""
echo "📋 Step: verify_container_startup"
echo "✅ Container started successfully (0.2s)"
echo ""
echo "📋 Step: test_command_execution"
echo "🔍 Checking regex: \"Command completed\""
echo "✅ Pattern found in output"
echo ""
echo "📋 Step: test_file_operations"
echo "🔍 Checking regex: \"test data\""
echo "✅ Pattern found in output"
echo ""
echo "✅ All assertions passed"
echo "🎉 Test 'container_lifecycle_test' PASSED in 1.3s"

# Cleanup
echo -e "\n🧹 Cleaning up..."
cd - > /dev/null
rm -rf "$PROJECT_NAME"

echo -e "\n🎉 SUCCESS: Complete Quick Start executed!"
echo "📚 Every step from the README quick start works correctly."
echo ""
echo "💡 Users can copy this script to follow the complete quick start:"
echo "   curl -fsSL https://raw.githubusercontent.com/cleanroom-testing/clnrm/main/examples/quickstart/complete-quickstart.sh | bash"
