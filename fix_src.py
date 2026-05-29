import os
import re

# 1. Fix config_error -> configuration_error globally
for root, _, files in os.walk('crates/clnrm-core/src'):
    for file in files:
        if not file.endswith('.rs'): continue
        path = os.path.join(root, file)
        with open(path, 'r') as f: content = f.read()
        if 'CleanroomError::config_error(' in content:
            content = content.replace('CleanroomError::config_error(', 'CleanroomError::configuration_error(')
            with open(path, 'w') as f: f.write(content)

# 2. Fix GvisorBackend::verify_docker_available -> is_available
for root, _, files in os.walk('crates/clnrm-core/src'):
    for file in files:
        if not file.endswith('.rs'): continue
        path = os.path.join(root, file)
        with open(path, 'r') as f: content = f.read()
        if 'GvisorBackend::verify_docker_available' in content:
            content = content.replace('crate::backend::GvisorBackend::verify_docker_available()?', 
                                      'crate::backend::GvisorBackend::is_available().then(|| ()).ok_or_else(|| CleanroomError::runtime_error("gVisor not available"))?')
            with open(path, 'w') as f: f.write(content)

# 3. Add ServiceHandle::new
path = 'crates/clnrm-core/src/cleanroom.rs'
with open(path, 'r') as f: content = f.read()
if 'impl ServiceHandle {' not in content:
    new_impl = '''impl ServiceHandle {
    pub fn new(service_name: &str) -> Self {
        Self {
            id: format!("{}-{}", service_name, uuid::Uuid::new_v4().simple()),
            service_name: service_name.to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }
}
'''
    content = content.replace('pub struct ServiceHandle {', new_impl + 'pub struct ServiceHandle {')
    with open(path, 'w') as f: f.write(content)

# 4. Make GvisorBackend::new synchronous
path = 'crates/clnrm-core/src/backend/gvisor.rs'
with open(path, 'r') as f: content = f.read()
content = content.replace('pub async fn new(image: impl Into<String>) -> Result<Self> {', 'pub fn new(image: impl Into<String>) -> Result<Self> {')
with open(path, 'w') as f: f.write(content)

for root, _, files in os.walk('crates/clnrm-core/src'):
    for file in files:
        if not file.endswith('.rs'): continue
        path = os.path.join(root, file)
        with open(path, 'r') as f: content = f.read()
        if 'GvisorBackend::new' in content and '.await' in content:
            content = content.replace('GvisorBackend::new(&default_image).await', 'GvisorBackend::new(&default_image)')
            content = content.replace('GvisorBackend::new("alpine:latest").await', 'GvisorBackend::new("alpine:latest")')
            content = content.replace('crate::backend::GvisorBackend::new("rust:1-slim").await', 'crate::backend::GvisorBackend::new("rust:1-slim")')
            with open(path, 'w') as f: f.write(content)

# 5. Add tera::Error conversion to error.rs
path = 'crates/clnrm-core/src/error.rs'
with open(path, 'r') as f: content = f.read()
if 'impl From<tera::Error>' not in content:
    conversion = '''
impl From<tera::Error> for CleanroomError {
    fn from(err: tera::Error) -> Self {
        CleanroomError::template_error(err.to_string())
    }
}
'''
    content += conversion
    with open(path, 'w') as f: f.write(content)

# 6. Fix BackendSelector methods in engine.rs
path = 'crates/clnrm-core/src/backend/engine.rs'
with open(path, 'r') as f: content = f.read()

selector_methods = '''    pub fn register(&mut self, backend: Box<dyn ExecutionEngine>) {
        self.backends.insert(backend.backend_type(), std::sync::Arc::from(backend));
    }

    pub fn select(&self, env: &CompiledEnvironment) -> Result<std::sync::Arc<dyn ExecutionEngine>> {
        let selected = if env.graph.nodes.len() > 1 { BackendType::Container } else { self.default };
        self.backends.get(&selected).cloned().ok_or_else(|| CleanroomError::internal_error("No backend"))
    }'''

if 'pub fn register' not in content:
    content = content.replace('''        Self {
            backends: HashMap::new(),
            default,
        }
    }''', '''        Self {
            backends: HashMap::new(),
            default,
        }
    }

''' + selector_methods)
    with open(path, 'w') as f: f.write(content)
