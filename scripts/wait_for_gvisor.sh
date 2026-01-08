#!/bin/bash
# Wait for gVisor runtime to be ready
# Replaces wait_for_docker.sh for gVisor-based environments

echo "Waiting for gVisor runtime to start..."
echo ""
echo "gVisor Setup Information:"
echo "  • gVisor is a userspace container runtime"
echo "  • Requires runsc binary to be installed and in PATH"
echo "  • For installation: https://gvisor.dev/docs/user_guide/install/"
echo ""

MAX_WAIT=60
ELAPSED=0

while [ $ELAPSED -lt $MAX_WAIT ]; do
    # Check if runsc is available and responsive
    if runsc --version >/dev/null 2>&1; then
        echo "✅ gVisor (runsc) is ready!"
        exit 0
    fi

    echo -n "."
    sleep 2
    ELAPSED=$((ELAPSED + 2))
done

echo ""
echo "❌ gVisor runtime did not start within $MAX_WAIT seconds"
echo ""
echo "Troubleshooting:"
echo "  1. Verify runsc is installed: which runsc"
echo "  2. Check runsc is executable: runsc --version"
echo "  3. Verify kernel capabilities: uname -r"
echo "  4. Check system logs for errors"
echo ""
echo "For more information:"
echo "  • gVisor Docs: https://gvisor.dev/"
echo "  • Installation Guide: https://gvisor.dev/docs/user_guide/install/"
echo ""
exit 1
