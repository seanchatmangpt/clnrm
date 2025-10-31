#!/bin/bash
# Wait for Docker daemon to be ready

echo "Waiting for Docker daemon to start..."
echo ""
echo "If Docker Desktop is not running:"
echo "  1. Open Docker Desktop application"
echo "  2. Wait for the whale icon in menu bar to stop animating"
echo ""

MAX_WAIT=60
ELAPSED=0

while [ $ELAPSED -lt $MAX_WAIT ]; do
    if docker ps >/dev/null 2>&1; then
        echo "✅ Docker daemon is ready!"
        exit 0
    fi

    echo -n "."
    sleep 2
    ELAPSED=$((ELAPSED + 2))
done

echo ""
echo "❌ Docker daemon did not start within $MAX_WAIT seconds"
echo ""
echo "Troubleshooting:"
echo "  1. Check Docker Desktop is running"
echo "  2. Check for errors in Docker Desktop logs"
echo "  3. Try restarting Docker Desktop"
echo ""
exit 1
