import os
import re

file_path = "crates/clnrm-core/tests/span_enforcement.rs"

if os.path.exists(file_path):
    with open(file_path, "r") as f:
        content = f.read()
    
    content = re.sub(r'\[otel_validation\]\n\s*enabled = true\n\n\s*\[otel_validation\]\n\s*enabled = true\n\s*\[\[otel_validation\.expected_spans\]\]', '[otel_validation]\n        enabled = true\n        [[otel_validation.expected_spans]]', content)

    with open(file_path, "w") as f:
        f.write(content)
