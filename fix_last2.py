import os

# 1. Fix backend/mod.rs (format string)
path = 'crates/clnrm-core/src/backend/mod.rs'
with open(path, 'r') as f: content = f.read()
content = content.replace('format!(name)', 'format!("{name}")')
with open(path, 'w') as f: f.write(content)

# 2. Fix cleanroom.rs (duplicate derive)
path = 'crates/clnrm-core/src/cleanroom.rs'
with open(path, 'r') as f: content = f.read()
content = content.replace('#[derive(Debug, Clone)]\n#[derive(Debug, Clone)]\npub struct ServiceHandle', '#[derive(Debug, Clone)]\npub struct ServiceHandle')
with open(path, 'w') as f: f.write(content)

# 3. Fix pool.rs and stress_test/pool.rs (with_startup_timeout -> with_timeout)
def fix_timeout(path):
    if not os.path.exists(path): return
    with open(path, 'r') as f: content = f.read()
    content = content.replace('with_startup_timeout(', 'with_timeout(')
    with open(path, 'w') as f: f.write(content)

fix_timeout('crates/clnrm-core/src/backend/pool.rs')
fix_timeout('crates/clnrm-core/src/stress_test/pool.rs')
