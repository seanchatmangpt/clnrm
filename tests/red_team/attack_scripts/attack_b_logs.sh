#!/bin/bash
# Attack B: Log Mimicry
#
# This script demonstrates a more sophisticated attack that
# mimics realistic log output with timestamps, progress indicators,
# and structured formatting that might fool log-based validation.
#
# Expected clnrm verdict: FAIL
# First failing rule: expect.counts.spans_total
# Detection time: ~0.02s

echo "========================================"
echo "Attack B: Log Mimicry (Sophisticated)"
echo "========================================"
echo ""
echo "🎭 Generating realistic log output..."
echo ""

# Mimic clnrm log format with timestamps
echo "[2025-10-16T10:00:00Z] INFO: Starting test suite"
echo "[2025-10-16T10:00:01Z] INFO: Initializing cleanroom environment"
echo "[2025-10-16T10:00:02Z] INFO: Loading service plugins..."
echo "[2025-10-16T10:00:03Z] INFO: - Loaded: generic_container"
echo "[2025-10-16T10:00:04Z] INFO: - Loaded: surrealdb"
echo "[2025-10-16T10:00:05Z] INFO: Starting container: alpine:latest"
echo "[2025-10-16T10:00:06Z] INFO: Container started (ID: abc123def456)"
echo "[2025-10-16T10:00:07Z] INFO: Executing: hello_world test"
echo "[2025-10-16T10:00:08Z] INFO: Command output: Hello, World!"
echo "[2025-10-16T10:00:09Z] INFO: Container stopped successfully"
echo "[2025-10-16T10:00:10Z] INFO: Cleanup complete"
echo ""
echo "Test Results:"
echo "  Passed: 5"
echo "  Failed: 0"
echo "  Skipped: 0"
echo "  Duration: 10.2s"
echo ""
echo "✅ All tests passed"
echo "PASS"
exit 0
