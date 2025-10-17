#!/bin/bash
# Attack A: Echo Pass
#
# This script demonstrates the simplest fake-green attack:
# printing success messages and exiting with code 0 without
# actually executing any tests.
#
# Expected clnrm verdict: FAIL
# First failing rule: expect.counts.spans_total
# Detection time: ~0.02s

echo "========================================"
echo "Attack A: Echo Pass (Trivial Forgery)"
echo "========================================"
echo ""
echo "🎭 Simulating malicious test wrapper..."
echo ""
echo "✅ Tests passed: 100%"
echo "✅ Coverage: 95%"
echo "✅ All assertions passed"
echo "✅ No errors detected"
echo ""
echo "PASS"
echo ""
echo "Exit code: 0 (CI would accept this)"
exit 0
