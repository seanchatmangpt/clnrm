#!/bin/bash
# Bottleneck Validation Script
# Validates system limits and identifies current bottlenecks

set -e

echo "=== clnrm Bottleneck Validation ==="
echo "Date: $(date)"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check limit
check_limit() {
    local name="$1"
    local current="$2"
    local recommended="$3"
    
    if [ "$current" -ge "$recommended" ]; then
        echo -e "${GREEN}✓${NC} $name: $current (recommended: $recommended)"
    else
        echo -e "${RED}✗${NC} $name: $current (recommended: $recommended)"
    fi
}

echo "--- System Limits (1x-10x Scale) ---"

# File descriptor limit
FD_LIMIT=$(ulimit -n)
check_limit "File descriptors" "$FD_LIMIT" "15000"

# Check if Docker is running
if command -v docker &> /dev/null; then
    if docker info &> /dev/null; then
        echo -e "${GREEN}✓${NC} Docker: Running"
        
        # Docker stats
        DOCKER_CONTAINERS=$(docker ps -q | wc -l | tr -d ' ')
        echo "  Active containers: $DOCKER_CONTAINERS"
        
        # Docker memory (if on Linux)
        if [ -f /sys/fs/cgroup/memory/docker/memory.limit_in_bytes ]; then
            DOCKER_MEM=$(cat /sys/fs/cgroup/memory/docker/memory.limit_in_bytes)
            DOCKER_MEM_GB=$((DOCKER_MEM / 1024 / 1024 / 1024))
            check_limit "Docker memory limit (GB)" "$DOCKER_MEM_GB" "8"
        fi
    else
        echo -e "${RED}✗${NC} Docker: Not running"
    fi
else
    echo -e "${YELLOW}⚠${NC} Docker: Not installed"
fi

echo ""
echo "--- Network Limits (10x Scale) ---"

# Check nf_conntrack (Linux only)
if [ -f /proc/sys/net/netfilter/nf_conntrack_max ]; then
    NF_CONNTRACK=$(cat /proc/sys/net/netfilter/nf_conntrack_max)
    check_limit "nf_conntrack_max" "$NF_CONNTRACK" "1048576"
else
    echo -e "${YELLOW}⚠${NC} nf_conntrack: Not available (Linux only)"
fi

# Network interfaces
if command -v ip &> /dev/null; then
    VETH_COUNT=$(ip link show | grep -c veth || echo 0)
    echo "  Active veth pairs: $VETH_COUNT"
    
    if [ "$VETH_COUNT" -gt 500 ]; then
        echo -e "${YELLOW}⚠${NC} High veth count detected (>500)"
    fi
fi

echo ""
echo "--- Current Bottleneck Assessment ---"

# Determine current scale
CURRENT_SCALE="1x"
if [ "$DOCKER_CONTAINERS" -gt 700 ]; then
    CURRENT_SCALE="10x"
elif [ "$DOCKER_CONTAINERS" -gt 70 ]; then
    CURRENT_SCALE="~10x"
fi

echo "Current scale: $CURRENT_SCALE (based on $DOCKER_CONTAINERS containers)"

# Identify bottlenecks
echo ""
echo "Likely bottlenecks at current scale:"

if [ "$FD_LIMIT" -lt 15000 ]; then
    echo -e "${RED}🔴${NC} File descriptor limit too low"
    echo "   Fix: ulimit -n 15000 (or add to /etc/security/limits.conf)"
fi

if [ "$DOCKER_CONTAINERS" -gt 100 ]; then
    echo -e "${YELLOW}⚠️${NC} High container count - consider parallel startup"
    echo "   See: docs/EMERGENT_BOTTLENECKS_ANALYSIS.md (Section: 10x Scale)"
fi

if [ "$DOCKER_CONTAINERS" -gt 500 ]; then
    echo -e "${RED}🔴${NC} Approaching single-host limits"
    echo "   See: docs/EMERGENT_BOTTLENECKS_ANALYSIS.md (Section: 100x Scale)"
fi

echo ""
echo "--- Recommendations ---"

if [ "$CURRENT_SCALE" = "1x" ]; then
    echo "Priority 1: Implement parallel container startup (3-5x speedup)"
    echo "Priority 2: Convert metrics to AtomicU64 (eliminate RwLock)"
    echo "Priority 3: Add system limits pre-flight check"
elif [ "$CURRENT_SCALE" = "~10x" ]; then
    echo "Priority 1: Increase file descriptor limits"
    echo "Priority 2: Configure host networking mode"
    echo "Priority 3: Implement telemetry sampling (10%)"
else
    echo "Priority 1: Multi-host orchestration (Kubernetes)"
    echo "Priority 2: Distributed metrics collection"
    echo "Priority 3: Geographic partitioning"
fi

echo ""
echo "=== Validation Complete ==="
echo "Full analysis: docs/EMERGENT_BOTTLENECKS_ANALYSIS.md"
