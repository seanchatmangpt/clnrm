import os
import re

file_path = "crates/clnrm-core/tests/v1_2_1_regression.rs"

if os.path.exists(file_path):
    with open(file_path, "r") as f:
        content = f.read()
    
    # Fix the missing name attribute in [[expect.span]] and duplication
    content = re.sub(r'\[expect\.span\]', '[[expect.span]]', content)
    content = re.sub(r'\[\[expect\.span\]\]\n\s*names = \["test\.span"\]', '[[expect.span]]\n        name = "test.span"', content)
    
    # Fix duplicate [otel_validation] blocks
    content = re.sub(r'\[otel_validation\]\n\s*enabled = true\n\s*\[otel_validation\]\n\s*enabled = true\n\s*\[\[otel_validation\.expected_spans\]\]', '[otel_validation]\n        enabled = true\n        [[otel_validation.expected_spans]]', content)

    # For v1_0_0 config, we can just replace [[expect.span]] names = [] with name = ""
    content = re.sub(r'names = \["([^"]+)", "([^"]+)"\]', r'name = "\1"\n        [[expect.span]]\n        name = "\2"', content)
    
    # For missing required fields still fails, we need to ensure the test fails. It was failing because maybe it passed?
    # Wait, missing_required_fields tests if step has no command/exec. Let's look at it if it fails.

    with open(file_path, "w") as f:
        f.write(content)
