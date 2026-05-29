import os
import re

# 1. Fix ServiceHandle in cleanroom.rs
path = 'crates/clnrm-core/src/cleanroom.rs'
with open(path, 'r') as f: content = f.read()

# I added `impl ServiceHandle` BEFORE `pub struct ServiceHandle`, and I also messed up the derives if they were there.
# Let's cleanly define ServiceHandle.
# The error output showed:
# 35 |   #[derive(Debug, Clone)]
# 36 |
# 37 | / impl ServiceHandle {
# 38 | |     pub fn new(service_name: &str) -> Self {
# ...
# 45 | | }

# So let's find that block and replace it correctly
pattern = r'#\[derive\(Debug,\s*Clone\)\]\s*impl ServiceHandle \{.*?\}\s*pub struct ServiceHandle \{.*?\}'
new_block = '''
#[derive(Debug, Clone)]
pub struct ServiceHandle {
    pub id: String,
    pub service_name: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ServiceHandle {
    pub fn new(service_name: &str) -> Self {
        Self {
            id: format!("{}-{}", service_name, uuid::Uuid::new_v4().simple()),
            service_name: service_name.to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }
}
'''
if 'impl ServiceHandle' in content:
    # Manual replacement strategy since regex might fail across multiple newlines
    content = re.sub(pattern, new_block, content, flags=re.DOTALL)
    # If the above failed, try replacing just the impl
    content = content.replace('#[derive(Debug, Clone)]\nimpl ServiceHandle {', '#[derive(Debug, Clone)]\npub struct ServiceHandle {\n    pub id: String,\n    pub service_name: String,\n    pub metadata: std::collections::HashMap<String, String>,\n}\n\nimpl ServiceHandle {')
    # and then remove the original struct if it exists below
    content = re.sub(r'impl ServiceHandle \{.*?\}\s*pub struct ServiceHandle \{.*?\}', new_block, content, flags=re.DOTALL)

with open(path, 'w') as f: f.write(content)


# 2. Fix error.rs (HashMap, tera::Error)
path = 'crates/clnrm-core/src/error.rs'
with open(path, 'r') as f: content = f.read()
if 'use std::collections::HashMap;' not in content:
    content = content.replace('use std::fmt;', 'use std::fmt;\nuse std::collections::HashMap;')

content = re.sub(r'impl From<tera::Error> for CleanroomError \{.*?\}', '', content, flags=re.DOTALL)
with open(path, 'w') as f: f.write(content)

# 3. Fix backend/oci/cache.rs (sha2::Digest)
path = 'crates/clnrm-core/src/backend/oci/cache.rs'
with open(path, 'r') as f: content = f.read()
if 'use sha2::Digest;' not in content:
    content = content.replace('use serde::{Deserialize, Serialize};', 'use serde::{Deserialize, Serialize};\nuse sha2::Digest;')
    with open(path, 'w') as f: f.write(content)
