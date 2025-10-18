# Test Timeout Enforcement for CI/CD
# 
# This script ensures all tests complete within 1 second
# Exit codes:
# 0 = All tests passed within timeout
# 124 = Tests timed out (build fails)
# Other = Test failures

set -euo pipefail

echo "🧪 Running tests with 1-second timeout enforcement..."

# Run tests with timeout
if timeout 1s cargo test --lib --quiet; then
    echo "✅ All tests passed within 1-second timeout"
    exit 0
else
    exit_code=$?
    if [ $exit_code -eq 124 ]; then
        echo "❌ Tests exceeded 1-second timeout - build failed"
        echo "💡 Remove or optimize slow tests to pass CI"
        echo "💡 Focus on unit tests - integration testing via clnrm.toml files"
    else
        echo "❌ Tests failed with exit code $exit_code"
    fi
    exit $exit_code
fi
