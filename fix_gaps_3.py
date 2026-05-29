import os
import re

files_to_fix = [
    "crates/clnrm-core/src/cleanroom.rs",
    "crates/clnrm-core/src/telemetry/generated/mod.rs"
]

for file_path in files_to_fix:
    if not os.path.exists(file_path):
        continue
    
    with open(file_path, "r") as f:
        content = f.read()
    
    content = re.sub(r'ExampleOnlyMockDatabasePlugin', 'ExampleOnlyDatabasePlugin', content)
    content = re.sub(r'example_only_mocks', 'example_only_doubles', content)
    content = re.sub(r'mock_database', 'example_database', content)
    
    with open(file_path, "w") as f:
        f.write(content)
