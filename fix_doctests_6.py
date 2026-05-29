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
        content = re.sub(r'run_tests_with_shard_and_report\&', 'run_tests_with_shard_and_report_doc&', content)
        content = re.sub(r'run_tests_with_shard_and_report\(', 'run_tests_with_shard_and_report_doc(', content)

    if "scheduler/mod.rs" in file_path:
        content = re.sub(r'clnrm_core::capabilities::LimitsConfig::default\(\)', 'clnrm_core::policy::ResourceLimits::default()', content)
        

    with open(file_path, "w") as f:
        f.write(content)
