import os
import re

files_to_fix = [
    "crates/clnrm-core/src/cli/commands/run/mod.rs",
    "crates/clnrm-core/src/scheduler/mod.rs",
]

for file_path in files_to_fix:
    if not os.path.exists(file_path):
        continue
    
    with open(file_path, "r") as f:
        content = f.read()
    
    if "cli/commands/run/mod.rs" in file_path:
        content = re.sub(r'None\)\.await\?;', 'clnrm_core::telemetry::live_check::ValidationConfig { enabled: false, mode: clnrm_core::telemetry::live_check::ValidationMode::Strict, admin_port: 0, otlp_port: 0 }).await?;', content)

    if "scheduler/mod.rs" in file_path:
        content = re.sub(r'clnrm_core::policy::ResourceLimits::default\(\)', 'clnrm_core::backend::ResourceLimits::default()', content)
        
    with open(file_path, "w") as f:
        f.write(content)
