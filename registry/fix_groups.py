import yaml
from pathlib import Path

# Mapping of attribute IDs in events to their brief and examples
attr_fixes = {
    "sandbox.id": {
        "brief": "gvisor sandbox identifier",
        "examples": ["abc123def456"]
    },
    "bundle.path": {
        "brief": "Path to OCI bundle directory",
        "examples": ["/var/run/runsc/container"]
    },
    "gvisor.sandbox.pid": {
        "brief": "Host PID of runsc sandbox process"
    },
    "gvisor.network.mode": {
        "brief": "gvisor network mode",
        "examples": ["none", "sandbox"]
    },
    "exit_code": {
        "brief": "Command exit code"
    },
    "duration_ms": {
        "brief": "Execution duration in milliseconds"
    },
    "verified": {
        "brief": "Whether isolation check passed"
    },
    "isolation.type": {
        "brief": "Type of isolation being verified",
        "examples": ["network", "filesystem"]
    },
    "isolation.method": {
        "brief": "Method used for isolation verification",
        "examples": ["gvisor_netstack", "namespace_check"]
    },
    "memory_bytes": {
        "brief": "Memory usage in bytes"
    },
    "cpu_time_ns": {
        "brief": "CPU time in nanoseconds"
    },
    "pid_count": {
        "brief": "Number of processes in container"
    },
    "syscall.name": {
        "brief": "Name of the syscall",
        "examples": ["ptrace", "mount"]
    }
}

file_path = Path("/Users/sac/clnrm/registry/core/gvisor_container.yaml")
with open(file_path, 'r') as f:
    data = yaml.safe_load(f)

for group in data['groups']:
    if group.get('type') == 'event':
        for attr in group.get('attributes', []):
            attr_id = attr.get('id')
            if attr_id in attr_fixes:
                fix = attr_fixes[attr_id]
                if 'brief' not in attr:
                    attr['brief'] = fix['brief']
                if 'examples' in fix and 'examples' not in attr:
                    attr['examples'] = fix['examples']

with open(file_path, 'w') as f:
    yaml.dump(data, f, default_flow_style=False, sort_keys=False, width=100)
print("Updated event attributes in gvisor_container.yaml")
