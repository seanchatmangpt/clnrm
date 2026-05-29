import os
import re

file_path = "crates/clnrm-core/tests/span_enforcement.rs"

if os.path.exists(file_path):
    with open(file_path, "r") as f:
        content = f.read()
    
    content = re.sub(r'\[expect\.span\]', '[[expect.span]]', content)
    content = re.sub(r'\[\[otel_validation\.expected_spans\]\]', '[otel_validation]\nenabled = true\n[[otel_validation.expected_spans]]', content)

    with open(file_path, "w") as f:
        f.write(content)
