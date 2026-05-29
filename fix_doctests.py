import os
import re

files_to_fix = [
    "crates/clnrm-core/src/policy.rs",
    "crates/clnrm-core/src/scenario.rs",
    "crates/clnrm-core/src/determinism/mod.rs",
    "crates/clnrm-core/src/services/factory.rs"
]

for file_path in files_to_fix:
    if not os.path.exists(file_path):
        continue
    
    with open(file_path, "r") as f:
        content = f.read()
    
    content = re.sub(r'use clnrm::', 'use clnrm_core::', content)
    
    if "determinism/mod.rs" in file_path:
        content = re.sub(r'let config = DeterminismConfig \{', 'let config = DeterminismConfig { deterministic_ports: true, deterministic_volumes: true,', content)
        
    if "services/factory.rs" in file_path:
        content = re.sub(r'let mut config = ServiceConfig \{', 'let mut config = ServiceConfig { args: None, password: None, strict: None, depends_on: None, volumes: None, env: None,', content)
        
    with open(file_path, "w") as f:
        f.write(content)
