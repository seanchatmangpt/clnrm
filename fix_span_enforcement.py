import os
import re

file_path = "crates/clnrm-core/tests/span_enforcement.rs"

if os.path.exists(file_path):
    with open(file_path, "r") as f:
        content = f.read()
    
    content = re.sub(r'names = \["([^"]+)"\]', r'name = "\1"', content)
    
    # For multiple names, we just test one for now or split them.
    content = re.sub(r'names = \["test\.execution", "step\.run"\]\n\s*count\.min = 2', r'name = "test.execution"\n        count.min = 1\n        [[expect.span]]\n        name = "step.run"\n        count.min = 1', content)
    content = re.sub(r'names = \["span1", "span2", "span3"\]\n\s*count\.min = 3\n\s*count\.max = 10', r'name = "span1"\n        count.min = 1\n        count.max = 10\n        [[expect.span]]\n        name = "span2"\n        count.min = 1\n        count.max = 10\n        [[expect.span]]\n        name = "span3"\n        count.min = 1\n        count.max = 10', content)
    content = re.sub(r'names = \["test\.run", "step\.execute"\]\n\s*count\.min = 2', r'name = "test.run"\n        count.min = 1\n        [[expect.span]]\n        name = "step.execute"\n        count.min = 1', content)

    # For those missing name entirely
    content = re.sub(r'\[\[expect\.span\]\]\n\s*count\.min', r'[[expect.span]]\n        name = "default"\n        count.min', content)
    content = re.sub(r'\[\[expect\.span\]\]\n\s*count\.max', r'[[expect.span]]\n        name = "default"\n        count.max', content)

    with open(file_path, "w") as f:
        f.write(content)
