# gVisor Architecture Diagrams

## 1. System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        CLEANROOM TESTING FRAMEWORK                       │
└───────────────────────────┬─────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         BACKEND ABSTRACTION                              │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
│  │ TestcontainerBe  │  │  GvisorBackend   │  │   MockBackend    │      │
│  │   (OLD/LEGACY)   │  │     (NEW)        │  │   (TESTING)      │      │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘      │
└───────────────────────────┬─────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      ISOLATION COMPONENTS                                │
│                                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │   Network    │  │  Filesystem  │  │   Resource   │  │  Cleanup   │  │
│  │   Manager    │  │   Manager    │  │   Manager    │  │  Manager   │  │
│  │              │  │              │  │              │  │            │  │
│  │ • Namespaces │  │ • OCI Images │  │ • Memory     │  │ • Orphans  │  │
│  │ • veth pairs │  │ • Mounts     │  │ • CPU        │  │ • Temp     │  │
│  │ • IP alloc   │  │ • Volumes    │  │ • Disk       │  │ • Network  │  │
│  │ • Port map   │  │ • Tmpfs      │  │ • Limits     │  │ • Bundles  │  │
│  │ • DNS        │  │ • Perms      │  │              │  │            │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘  │
└───────────────────────────┬─────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        GVISOR RUNTIME (runsc)                            │
│                                                                           │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                   USER-SPACE KERNEL (Sentry)                       │  │
│  │                                                                     │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐          │  │
│  │  │ Syscall  │  │ Network  │  │   File   │  │ Process  │          │  │
│  │  │  Filter  │  │  Stack   │  │  System  │  │  Mgmt    │          │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘          │  │
│  │                                                                     │  │
│  │  All syscalls intercepted and validated before kernel              │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└───────────────────────────┬─────────────────────────────────────────────┘
                            │ (Filtered syscalls only)
                            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          HOST LINUX KERNEL                               │
│                      (Minimal Exposure - Hardened)                       │
└─────────────────────────────────────────────────────────────────────────┘
```

## 2. Network Isolation Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           HOST NETWORK                                   │
│                         (Physical/Virtual)                               │
└────────────────┬────────────────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      HOST NETWORK NAMESPACE                              │
│                                                                           │
│  ┌─────────────┐                                                         │
│  │  eth0       │ (Host physical interface)                              │
│  │  10.0.2.15  │                                                         │
│  └─────────────┘                                                         │
│        │                                                                  │
│        │ NAT/Forwarding                                                  │
│        │                                                                  │
│  ┌─────────────┐         ┌────────────────────────────────────────┐     │
│  │ veth0-host  │◄───────►│        iptables NAT                     │     │
│  │ 10.88.0.1/24│         │  • DNAT (port mapping)                 │     │
│  └─────────────┘         │  • MASQUERADE (return traffic)         │     │
│        │                 │  • FORWARD (routing)                    │     │
│        │                 └────────────────────────────────────────┘     │
│        │                                                                  │
│        │ (veth pair - virtual ethernet cable)                            │
│        │                                                                  │
└────────┼──────────────────────────────────────────────────────────────┬─┘
         │                                                               │
         │                                                               │
         ▼                                                               │
┌─────────────────────────────────────────────────────────────────────┐  │
│              CONTAINER NETWORK NAMESPACE (container-1)              │  │
│                                                                      │  │
│  ┌─────────────────┐                                                │  │
│  │ veth0-container │                                                │  │
│  │  10.88.0.2/24   │ ◄── Container's network interface             │  │
│  └─────────────────┘                                                │  │
│         │                                                            │  │
│         │                                                            │  │
│  ┌──────▼──────┐          ┌──────────────┐                         │  │
│  │    lo       │          │ Route Table  │                         │  │
│  │ 127.0.0.1   │          │  Default:    │                         │  │
│  └─────────────┘          │  10.88.0.1   │                         │  │
│                           └──────────────┘                          │  │
│                                                                      │  │
│  ┌──────────────────────────────────────────────────────────┐      │  │
│  │              /etc/resolv.conf                             │      │  │
│  │  nameserver 8.8.8.8                                      │      │  │
│  │  nameserver 8.8.4.4                                      │      │  │
│  │  search localdomain                                      │      │  │
│  └──────────────────────────────────────────────────────────┘      │  │
└──────────────────────────────────────────────────────────────────────┘  │
                                                                           │
                        (Same pattern for container-2, etc.)               │
                                                                           │
┌──────────────────────────────────────────────────────────────────────┐  │
│              CONTAINER NETWORK NAMESPACE (container-2)               │◄─┘
│  veth0-container: 10.88.0.3/24                                       │
└──────────────────────────────────────────────────────────────────────┘
```

### Network Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                      OUTBOUND TRAFFIC                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Container Process                                                   │
│      │ (bind to 0.0.0.0:80)                                         │
│      ▼                                                               │
│  gVisor Netstack (Sentry)                                           │
│      │ (Intercepts syscall)                                         │
│      ▼                                                               │
│  veth0-container (10.88.0.2:80)                                     │
│      │                                                               │
│      ▼ (through veth pair)                                          │
│  veth0-host (10.88.0.1)                                             │
│      │                                                               │
│      ▼ (iptables NAT)                                               │
│  eth0 (10.0.2.15) → External Network                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                      INBOUND TRAFFIC (Port 8080→80)                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  External Request (Host:8080)                                        │
│      │                                                               │
│      ▼ (iptables DNAT rule)                                         │
│  Translated to: 10.88.0.2:80                                        │
│      │                                                               │
│      ▼ (routing)                                                    │
│  veth0-host → veth0-container                                       │
│      │                                                               │
│      ▼                                                               │
│  gVisor Netstack (10.88.0.2:80)                                     │
│      │                                                               │
│      ▼                                                               │
│  Container Process receives request                                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3. Filesystem Isolation Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          HOST FILESYSTEM                                 │
│                                                                           │
│  /var/lib/cleanroom/gvisor/                                              │
│  ├── bundles/                                                            │
│  │   ├── container-1/                                                    │
│  │   │   ├── rootfs/          ◄── Container's isolated root             │
│  │   │   │   ├── bin/                                                    │
│  │   │   │   ├── etc/                                                    │
│  │   │   │   ├── lib/                                                    │
│  │   │   │   ├── proc/        (mount point)                             │
│  │   │   │   ├── dev/         (mount point)                             │
│  │   │   │   ├── sys/         (mount point)                             │
│  │   │   │   ├── tmp/         (mount point - tmpfs)                     │
│  │   │   │   └── data/        (mount point - bind mount)                │
│  │   │   └── config.json      ◄── OCI runtime spec                      │
│  │   └── container-2/                                                    │
│  ├── cache/                                                              │
│  │   └── oci/                                                            │
│  │       └── alpine-abc123/   ◄── Cached OCI images                     │
│  └── tmp/                                                                │
└───────────────────────────────────────────────────────────────────────┬─┘
                                                                          │
                                                                          │
┌─────────────────────────────────────────────────────────────────────┐ │
│              CONTAINER FILESYSTEM VIEW (container-1)                 │ │
│                    (Mount Namespace Isolated)                        │ │
│                                                                      │ │
│  /                        ◄── rootfs (read-write)                   │ │
│  ├── bin/                 ◄── From OCI image                        │ │
│  ├── etc/                 ◄── From OCI image + DNS config           │ │
│  │   └── resolv.conf      ◄── Generated by DnsResolver              │ │
│  ├── lib/                 ◄── From OCI image                        │ │
│  ├── proc/                ◄── procfs (type: proc)                   │ │
│  │   ├── 1/                                                          │ │
│  │   ├── self/                                                       │ │
│  │   └── ...                                                         │ │
│  ├── dev/                 ◄── tmpfs + device nodes                  │ │
│  │   ├── null                                                        │ │
│  │   ├── zero                                                        │ │
│  │   ├── random                                                      │ │
│  │   ├── pts/              ◄── devpts (pseudo-terminals)            │ │
│  │   └── shm/              ◄── tmpfs (shared memory)                │ │
│  ├── sys/                 ◄── sysfs (type: sysfs, read-only)        │ │
│  ├── tmp/                 ◄── tmpfs (ephemeral, cleared on stop)    │ │
│  └── data/                ◄── Bind mount from host (optional)       │ │
│      └── ...               ◄── /host/path → /data (read-only/rw)    │ │
│                                                                      │ │
└──────────────────────────────────────────────────────────────────────┘ │
                                                                          │
                        ISOLATION GUARANTEES:                             │
                        • Cannot see host /                               │
                        • Cannot see other containers                     │
                        • Mounts are private (MS_PRIVATE)                 │
                        • Changes isolated to this namespace              │
                                                                          │
                                                                          │
┌─────────────────────────────────────────────────────────────────────┐  │
│                        MOUNT PROPAGATION                             │  │
├─────────────────────────────────────────────────────────────────────┤  │
│                                                                      │  │
│  MS_PRIVATE (Default)                                               │  │
│    Host mount changes → NOT visible in container                    │  │
│    Container mounts   → NOT visible on host                         │  │
│    Use for: /proc, /dev, /sys, /tmp                                 │  │
│                                                                      │  │
│  MS_SLAVE (One-way from host)                                       │  │
│    Host mount changes → visible in container                        │  │
│    Container mounts   → NOT visible on host                         │  │
│    Use for: Bind mounts of host directories                         │  │
│                                                                      │  │
│  MS_SHARED (Bidirectional)                                          │  │
│    Host mount changes → visible in container                        │  │
│    Container mounts   → visible on host                             │  │
│    Use for: Special cases only (not recommended)                    │  │
│                                                                      │  │
└──────────────────────────────────────────────────────────────────────┘  │
                                                                           │
                                                                           │
┌──────────────────────────────────────────────────────────────────────┐  │
│                    OCI IMAGE LAYER EXTRACTION                         │  │
├──────────────────────────────────────────────────────────────────────┤  │
│                                                                       │  │
│  Registry (docker.io)                                                │  │
│       │                                                               │  │
│       ▼ (skopeo copy)                                                │  │
│  Local OCI Layout                                                    │  │
│  /cache/oci/alpine-abc123/                                           │  │
│  ├── blobs/                                                          │  │
│  │   ├── sha256:aaa... (config)                                     │  │
│  │   ├── sha256:bbb... (layer 1)                                    │  │
│  │   └── sha256:ccc... (layer 2)                                    │  │
│  └── index.json                                                      │  │
│       │                                                               │  │
│       ▼ (umoci unpack)                                               │  │
│  Bundle Directory                                                    │  │
│  /bundles/container-1/                                               │  │
│  ├── rootfs/          ◄── Layers extracted and merged               │  │
│  │   ├── bin/                                                        │  │
│  │   ├── etc/                                                        │  │
│  │   └── ...                                                         │  │
│  └── config.json      ◄── OCI runtime spec                          │  │
│                                                                       │  │
│  Layer Merging:                                                      │  │
│    Layer 1 (base)     → rootfs/                                     │  │
│    Layer 2 (changes)  → rootfs/ (overlay, handle .wh. whiteouts)    │  │
│                                                                       │  │
└───────────────────────────────────────────────────────────────────────┘  │
                                                                           │
└───────────────────────────────────────────────────────────────────────┘
```

## 4. Resource Isolation Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      RESOURCE CONTROL HIERARCHY                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Host System Resources                                                   │
│  ├── Total Memory: 16GB                                                  │
│  ├── Total CPU: 8 cores                                                  │
│  └── Total Disk: 500GB                                                   │
│        │                                                                  │
│        ▼ (cgroups v2 hierarchy)                                          │
│  /sys/fs/cgroup/                                                         │
│  ├── system.slice/                                                       │
│  └── cleanroom.slice/          ◄── Cleanroom cgroup root                │
│      ├── memory.max = 10GB     ◄── Total limit for all containers       │
│      ├── cpu.max = 600%        ◄── 6 cores total                        │
│      │                                                                    │
│      ├── container-1/                                                    │
│      │   ├── memory.max = 512MB                                         │
│      │   ├── cpu.weight = 1024  (1 CPU)                                 │
│      │   ├── pids.max = 100                                             │
│      │   └── io.max = 10MB/s                                            │
│      │                                                                    │
│      ├── container-2/                                                    │
│      │   ├── memory.max = 1GB                                           │
│      │   ├── cpu.weight = 2048  (2 CPUs)                                │
│      │   ├── pids.max = 200                                             │
│      │   └── io.max = 20MB/s                                            │
│      │                                                                    │
│      └── container-N/                                                    │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                    NAMESPACE ISOLATION                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Container Process Tree:                                                 │
│                                                                           │
│  PID Namespace (container-1)         PID Namespace (container-2)         │
│  ┌─────────────────────────┐        ┌─────────────────────────┐         │
│  │ PID 1: /init            │        │ PID 1: /init            │         │
│  │ PID 2: /bin/sh          │        │ PID 2: /app             │         │
│  │ PID 3: test process     │        │ PID 3: worker           │         │
│  └─────────────────────────┘        └─────────────────────────┘         │
│                                                                           │
│  (Isolated - cannot see each other's processes)                          │
│                                                                           │
│  ───────────────────────────────────────────────────────────────         │
│                                                                           │
│  Network Namespace Isolation:                                            │
│                                                                           │
│  container-1: netns "c1"             container-2: netns "c2"             │
│  ┌─────────────────────────┐        ┌─────────────────────────┐         │
│  │ veth0: 10.88.0.2/24     │        │ veth0: 10.88.0.3/24     │         │
│  │ lo: 127.0.0.1           │        │ lo: 127.0.0.1           │         │
│  └─────────────────────────┘        └─────────────────────────┘         │
│                                                                           │
│  (Separate IP addresses, routing tables, iptables)                       │
│                                                                           │
│  ───────────────────────────────────────────────────────────────         │
│                                                                           │
│  Mount Namespace Isolation:                                              │
│                                                                           │
│  container-1: mntns "m1"             container-2: mntns "m2"             │
│  ┌─────────────────────────┐        ┌─────────────────────────┐         │
│  │ /: rootfs-1             │        │ /: rootfs-2             │         │
│  │ /proc: proc             │        │ /proc: proc             │         │
│  │ /tmp: tmpfs-1           │        │ /tmp: tmpfs-2           │         │
│  └─────────────────────────┘        └─────────────────────────┘         │
│                                                                           │
│  (Separate filesystems, cannot access each other's files)                │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘
```

## 5. Cleanup and Lifecycle Management

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     CONTAINER LIFECYCLE                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  1. CREATE                                                               │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ • Generate container ID                              │            │
│     │ • Allocate IP address (10.88.0.x)                   │            │
│     │ • Create network namespace                           │            │
│     │ • Pull/unpack OCI image                             │            │
│     │ • Setup rootfs                                       │            │
│     │ • Generate config.json                               │            │
│     └──────────────────────────────────────────────────────┘            │
│                              │                                           │
│                              ▼                                           │
│  2. START                                                                │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ • Setup veth pair                                    │            │
│     │ • Configure networking                               │            │
│     │ • Setup iptables rules                               │            │
│     │ • Mount filesystems (/proc, /dev, /sys, /tmp)       │            │
│     │ • Apply resource limits (cgroups)                    │            │
│     │ • Start runsc container                              │            │
│     └──────────────────────────────────────────────────────┘            │
│                              │                                           │
│                              ▼                                           │
│  3. RUNNING                                                              │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ • Container process executing                        │            │
│     │ • Network active                                     │            │
│     │ • Filesystem mounted                                 │            │
│     │ • Resources monitored                                │            │
│     └──────────────────────────────────────────────────────┘            │
│                              │                                           │
│                              ▼                                           │
│  4. STOP                                                                 │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ • Send SIGTERM to container                          │            │
│     │ • Wait for graceful shutdown (timeout: 10s)         │            │
│     │ • Send SIGKILL if needed                             │            │
│     │ • Collect exit code                                  │            │
│     └──────────────────────────────────────────────────────┘            │
│                              │                                           │
│                              ▼                                           │
│  5. CLEANUP                                                              │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ • Delete runsc container                             │            │
│     │ • Remove iptables rules                              │            │
│     │ • Delete veth interfaces                             │            │
│     │ • Delete network namespace                           │            │
│     │ • Release IP address                                 │            │
│     │ • Unmount filesystems                                │            │
│     │ • Remove bundle directory                            │            │
│     │ • Release cgroup resources                           │            │
│     └──────────────────────────────────────────────────────┘            │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                     ORPHAN CLEANUP PROCESS                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Periodic Cleanup Task (every 5 minutes):                                │
│                                                                           │
│  1. List Active Containers                                               │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ runsc --root /var/run/runsc list                     │            │
│     │   → [container-1, container-2, ...]                  │            │
│     └──────────────────────────────────────────────────────┘            │
│                              │                                           │
│                              ▼                                           │
│  2. Find Orphaned Network Namespaces                                     │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ ip netns list                                        │            │
│     │   → [c1, c2, c3-orphaned, c4-orphaned]              │            │
│     │ Compare with active containers                       │            │
│     │   → Orphans: [c3-orphaned, c4-orphaned]             │            │
│     └──────────────────────────────────────────────────────┘            │
│                              │                                           │
│                              ▼                                           │
│  3. Cleanup Orphaned Resources                                           │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ For each orphan:                                     │            │
│     │   • ip netns delete <orphan>                         │            │
│     │   • Remove bundle directory                          │            │
│     │   • Release IP address                               │            │
│     │   • Clean iptables rules                             │            │
│     └──────────────────────────────────────────────────────┘            │
│                              │                                           │
│                              ▼                                           │
│  4. Cleanup Old Temporary Files                                          │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ Find files older than 24 hours:                      │            │
│     │   /var/lib/cleanroom/gvisor/tmp/*                   │            │
│     │   /var/lib/cleanroom/gvisor/cache/tmp/*             │            │
│     │ Delete old files                                     │            │
│     └──────────────────────────────────────────────────────┘            │
│                              │                                           │
│                              ▼                                           │
│  5. Enforce Disk Limits                                                  │
│     ┌──────────────────────────────────────────────────────┐            │
│     │ Check total disk usage                               │            │
│     │ If > 10GB:                                           │            │
│     │   • Remove old cached images                         │            │
│     │   • Remove old bundles                               │            │
│     └──────────────────────────────────────────────────────┘            │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘
```

## 6. Error Handling Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     ERROR HANDLING & RECOVERY                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ERROR: Network Namespace Already Exists                                 │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ Detection: ip netns add fails with "File exists"      │              │
│  │ Recovery:                                              │              │
│  │   1. Delete existing namespace                         │              │
│  │   2. Cleanup associated veth interfaces                │              │
│  │   3. Retry namespace creation                          │              │
│  │ Prevention: Use UUID-based names                       │              │
│  └───────────────────────────────────────────────────────┘              │
│                                                                           │
│  ERROR: Image Pull Timeout                                               │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ Detection: skopeo timeout                              │              │
│  │ Recovery:                                              │              │
│  │   1. Check network connectivity                        │              │
│  │   2. Retry with exponential backoff (3 attempts)      │              │
│  │   3. Fall back to cached image if available           │              │
│  │ Prevention: Pre-pull commonly used images             │              │
│  └───────────────────────────────────────────────────────┘              │
│                                                                           │
│  ERROR: IP Address Exhaustion                                            │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ Detection: No available IPs in subnet                  │              │
│  │ Recovery:                                              │              │
│  │   1. Trigger orphan cleanup                            │              │
│  │   2. Release IPs from stopped containers               │              │
│  │   3. Retry allocation                                  │              │
│  │ Prevention: Use larger subnet (/16 instead of /24)    │              │
│  └───────────────────────────────────────────────────────┘              │
│                                                                           │
│  ERROR: Container Start Failure                                          │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ Detection: runsc run returns non-zero exit             │              │
│  │ Recovery:                                              │              │
│  │   1. Force delete container (runsc delete --force)    │              │
│  │   2. Cleanup bundle directory                          │              │
│  │   3. Cleanup network namespace                         │              │
│  │   4. Log error for debugging                           │              │
│  │ Prevention: Validate config.json before runsc         │              │
│  └───────────────────────────────────────────────────────┘              │
│                                                                           │
│  ERROR: Disk Space Exhaustion                                            │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ Detection: Write fails with ENOSPC                     │              │
│  │ Recovery:                                              │              │
│  │   1. Trigger aggressive cleanup                        │              │
│  │   2. Remove old cached images                          │              │
│  │   3. Remove old bundles                                │              │
│  │   4. Retry operation                                   │              │
│  │ Prevention: Monitor disk usage, set quotas            │              │
│  └───────────────────────────────────────────────────────┘              │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘
```

## 7. Security Layers

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        SECURITY DEFENSE IN DEPTH                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Layer 1: Application (Test Code)                                        │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ • User-provided test code                             │              │
│  │ • Untrusted input                                     │              │
│  │ • May be malicious                                    │              │
│  └───────────────────────────────────────────────────────┘              │
│                              │                                           │
│                              ▼ (Isolated)                                │
│  Layer 2: gVisor Sentry (User-Space Kernel)                             │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ • All syscalls intercepted                            │              │
│  │ • Syscall filtering and validation                    │              │
│  │ • Emulated kernel subsystems                          │              │
│  │ • Memory isolation                                    │              │
│  └───────────────────────────────────────────────────────┘              │
│                              │                                           │
│                              ▼ (Filtered)                                │
│  Layer 3: Linux Namespaces                                               │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ • PID namespace (process isolation)                   │              │
│  │ • Network namespace (network isolation)               │              │
│  │ • Mount namespace (filesystem isolation)              │              │
│  │ • IPC namespace (IPC isolation)                       │              │
│  │ • UTS namespace (hostname isolation)                  │              │
│  │ • User namespace (UID/GID mapping)                    │              │
│  └───────────────────────────────────────────────────────┘              │
│                              │                                           │
│                              ▼ (Controlled)                              │
│  Layer 4: cgroups (Resource Limits)                                     │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ • Memory limits (prevent OOM)                         │              │
│  │ • CPU limits (prevent CPU exhaustion)                 │              │
│  │ • PID limits (prevent fork bombs)                     │              │
│  │ • I/O limits (prevent disk DoS)                       │              │
│  └───────────────────────────────────────────────────────┘              │
│                              │                                           │
│                              ▼ (Limited)                                 │
│  Layer 5: Seccomp (Syscall Filtering)                                   │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ • Whitelist approved syscalls                         │              │
│  │ • Block dangerous syscalls (ptrace, reboot, etc.)    │              │
│  │ • Reduce kernel attack surface                        │              │
│  └───────────────────────────────────────────────────────┘              │
│                              │                                           │
│                              ▼ (Hardened)                                │
│  Layer 6: Linux Kernel                                                   │
│  ┌───────────────────────────────────────────────────────┐              │
│  │ • Minimal syscall exposure                            │              │
│  │ • SELinux/AppArmor policies                           │              │
│  │ • Kernel hardening                                    │              │
│  └───────────────────────────────────────────────────────┘              │
│                                                                           │
│  Attack Surface Reduction:                                               │
│    Docker: 2 layers (container → kernel)                                │
│    gVisor: 6 layers (app → sentry → ns → cgroups → seccomp → kernel)   │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘
```

These diagrams provide a complete visual reference for understanding the gVisor isolation architecture, covering all aspects from network and filesystem isolation to resource management, cleanup, error handling, and security layers.
