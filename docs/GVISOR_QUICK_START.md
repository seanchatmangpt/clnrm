# gVisor Quick Start Guide

## Prerequisites

Install required tools:

```bash
# 1. Install gVisor (runsc)
curl -fsSL https://gvisor.dev/archive.key | sudo apt-key add -
sudo add-apt-repository "deb [arch=amd64,arm64] https://storage.googleapis.com/gvisor/releases release main"
sudo apt-get update && sudo apt-get install -y runsc

# 2. Install OCI image tools
sudo apt-get install -y skopeo
curl -Lo umoci https://github.com/opencontainers/umoci/releases/download/v0.4.7/umoci.amd64
chmod +x umoci
sudo mv umoci /usr/local/bin/

# 3. Verify installation
runsc --version
skopeo --version
umoci --version
```

## Basic Usage

### 1. Pull and Setup an OCI Image

```bash
# Pull alpine image
skopeo copy docker://alpine:latest oci:/tmp/alpine:latest

# Unpack to bundle
umoci unpack --image /tmp/alpine:latest /tmp/alpine-bundle
```

### 2. Create OCI Config

```bash
cd /tmp/alpine-bundle

# Generate config.json
cat > config.json <<EOF
{
  "ociVersion": "1.0.0",
  "root": {
    "path": "rootfs",
    "readonly": false
  },
  "process": {
    "terminal": false,
    "user": {
      "uid": 0,
      "gid": 0
    },
    "args": [
      "sh", "-c", "echo 'Hello from gVisor!'"
    ],
    "env": [
      "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
    ],
    "cwd": "/"
  },
  "mounts": [
    {
      "destination": "/proc",
      "type": "proc",
      "source": "proc"
    },
    {
      "destination": "/dev",
      "type": "tmpfs",
      "source": "tmpfs",
      "options": ["nosuid", "strictatime", "mode=755"]
    }
  ],
  "linux": {
    "namespaces": [
      {"type": "pid"},
      {"type": "network"},
      {"type": "ipc"},
      {"type": "uts"},
      {"type": "mount"}
    ]
  }
}
EOF
```

### 3. Run Container with runsc

```bash
# Create network namespace
sudo ip netns add test-container

# Run container
sudo runsc \
  --root /var/run/runsc \
  --network sandbox \
  --platform ptrace \
  run \
  --bundle /tmp/alpine-bundle \
  test-container

# Output: Hello from gVisor!
```

### 4. Cleanup

```bash
# Delete container
sudo runsc --root /var/run/runsc delete test-container

# Delete network namespace
sudo ip netns delete test-container

# Cleanup bundle
rm -rf /tmp/alpine-bundle
```

## Network Setup Example

### Create Isolated Network

```bash
CONTAINER_ID="test-$(uuidgen)"

# 1. Create network namespace
sudo ip netns add $CONTAINER_ID

# 2. Create veth pair
sudo ip link add veth-host type veth peer name veth-container

# 3. Move container veth to namespace
sudo ip link set veth-container netns $CONTAINER_ID

# 4. Configure host veth
sudo ip addr add 10.88.0.1/24 dev veth-host
sudo ip link set veth-host up

# 5. Configure container veth (in namespace)
sudo ip netns exec $CONTAINER_ID ip addr add 10.88.0.2/24 dev veth-container
sudo ip netns exec $CONTAINER_ID ip link set veth-container up
sudo ip netns exec $CONTAINER_ID ip link set lo up

# 6. Add default route
sudo ip netns exec $CONTAINER_ID ip route add default via 10.88.0.1

# 7. Test network
sudo ip netns exec $CONTAINER_ID ping -c 1 10.88.0.1
```

### Port Mapping Example

```bash
# Map host port 8080 to container port 80
sudo iptables -t nat -A PREROUTING \
  -p tcp --dport 8080 \
  -j DNAT --to-destination 10.88.0.2:80

# Enable forwarding
sudo iptables -A FORWARD -p tcp -d 10.88.0.2 --dport 80 -j ACCEPT
```

## Filesystem Mount Example

### Setup Rootfs with Volumes

```bash
# Create volume directory
mkdir -p /tmp/myvolume
echo "Hello from volume" > /tmp/myvolume/test.txt

# Add to config.json
cat >> config.json <<EOF
  {
    "destination": "/data",
    "type": "none",
    "source": "/tmp/myvolume",
    "options": ["rbind", "ro"]
  }
EOF
```

## Resource Limits Example

### Configure Memory and CPU Limits

```json
{
  "linux": {
    "resources": {
      "memory": {
        "limit": 536870912
      },
      "cpu": {
        "shares": 512
      },
      "pids": {
        "limit": 50
      }
    }
  }
}
```

## Debugging

### Enable Debug Logging

```bash
sudo runsc \
  --root /var/run/runsc \
  --debug \
  --debug-log=/tmp/runsc-debug.log \
  --strace \
  run --bundle /tmp/alpine-bundle test-container

# View logs
cat /tmp/runsc-debug.log
```

### Common Issues

**Issue**: runsc: Permission denied
**Solution**: Run with sudo or configure user namespaces

**Issue**: Network namespace not found
**Solution**: Ensure network namespace is created before running container

**Issue**: Image pull timeout
**Solution**: Check network connectivity, increase timeout

**Issue**: /proc mount failed
**Solution**: Ensure proc is available in mounts array

## Performance Tips

1. **Cache OCI Images**: Store unpacked bundles for reuse
2. **Pre-allocate Network Namespaces**: Create namespace pool
3. **Use tmpfs**: Mount /tmp as tmpfs for fast I/O
4. **Enable KVM Platform**: Use `--platform kvm` for better performance (requires /dev/kvm)
5. **Shared Filesystem**: Use `--file-access shared` for read-heavy workloads

## Next Steps

1. Read full design document: `/home/user/clnrm/docs/GVISOR_ISOLATION_DESIGN.md`
2. Explore code examples in: `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor/`
3. Run integration tests: `cargo test --test gvisor_integration`
4. Review security guidelines
