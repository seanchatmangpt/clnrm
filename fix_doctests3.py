import os
import re

files_to_fix = [
    "crates/clnrm-core/src/cli/commands/run/mod.rs",
    "crates/clnrm-core/src/macros.rs",
    "crates/clnrm-core/src/scheduler/mod.rs",
    "crates/clnrm-core/src/telemetry/weaver_stats.rs"
]

for file_path in files_to_fix:
    if not os.path.exists(file_path):
        continue
    
    with open(file_path, "r") as f:
        content = f.read()
    
    if "cli/commands/run/mod.rs" in file_path:
        content = re.sub(r'None\)\.await\?;', 'ValidationConfig { enabled: false, mode: ValidationMode::Strict, admin_port: 0, otlp_port: 0 }).await?;', content)

    if "macros.rs" in file_path:
        content = re.sub(r'/// use clnrm_core_macros::cleanroom_test;', '', content)
        content = re.sub(r'/// let user = register_user\("jane@example\.com"\)\?;', '', content)

    if "scheduler/mod.rs" in file_path:
        content = re.sub(r'capability_budget: HashMap::new\(\)', 'capability_budget: std::collections::HashMap::new()', content)
        
    if "weaver_stats.rs" in file_path:
        content = re.sub(r'let mut report = WeaverReport::generate\(\&config, None\)\?;', '/// let mut report = WeaverReport::generate(&config, None)?;', content)

    with open(file_path, "w") as f:
        f.write(content)
