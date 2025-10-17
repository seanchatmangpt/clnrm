#!/bin/bash
# Fake wrapper script that attempts to spoof test success
# This demonstrates the failure mode that clnrm's analyzer must catch

set -e

echo "🎭 FAKE WRAPPER: Attempting to spoof test results..."
echo ""
echo "✅ Tests passed: 100%"
echo "✅ Coverage: 95%"
echo "✅ All assertions passed"
echo ""
echo "PASS"

# Exit 0 to simulate success
exit 0

# What this script does NOT do (and clnrm will detect):
# - Does not create OTEL spans
# - Does not launch containers (no container.start events)
# - Does not create parent-child span relationships
# - Does not emit lifecycle events (container.exec, container.stop)
# - Does not produce hermetic resource attributes
# - Does not respect temporal ordering constraints
# - Does not satisfy span count requirements
# - Does not produce a verifiable trace digest
