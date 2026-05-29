import os
import re

files_to_fix = [
    "crates/clnrm-core/src/cli/commands/run/mod.rs",
    "crates/clnrm-core/src/determinism/mod.rs",
    "crates/clnrm-core/src/otel/mod.rs",
    "crates/clnrm-core/src/macros.rs",
    "crates/clnrm-core/src/services/factory.rs",
    "crates/clnrm-core/src/scheduler/mod.rs",
    "crates/clnrm-core/src/telemetry/semantic_conventions.rs",
    "crates/clnrm-core/src/telemetry/weaver_stats.rs"
]

for file_path in files_to_fix:
    if not os.path.exists(file_path):
        continue
    
    with open(file_path, "r") as f:
        content = f.read()
    
    if "cli/commands/run/mod.rs" in file_path:
        content = re.sub(r'run_tests_with_shard_and_report\(&paths, &config, None, Some\(Path::new\("junit.xml"\)\), "none", None\)\.await', 'run_tests_with_shard_and_report(&paths, &config, None, Some(Path::new("junit.xml")), "none", None, None).await', content)

    if "determinism/mod.rs" in file_path:
        content = re.sub(r'deterministic_ports: true, deterministic_volumes: true,', 'deterministic_ports: Some(true), deterministic_volumes: Some(true),', content)

    if "otel/mod.rs" in file_path:
        content = re.sub(r'let spans = StdoutSpanParser::parse\(stdout\)\?;', 'let spans = StdoutSpanParser::parse(stdout).unwrap();', content)

    if "macros.rs" in file_path:
        content = re.sub(r'use clnrm_core::\{cleanroom_test, with_database, with_cache\};', 'use clnrm_core::{with_database, with_cache};\n/// use clnrm_core_macros::cleanroom_test;', content)

    if "services/factory.rs" in file_path:
        content = re.sub(r'let mut config = ServiceConfig \{ args: None, password: None, strict: None, depends_on: None, volumes: None, env: None,', 'let mut config = ServiceConfig { args: None, password: None, strict: None, username: None, wait_for_span: None, wait_for_span_timeout_secs: None,', content)

    if "scheduler/mod.rs" in file_path:
        content = re.sub(r'capability_budget: CapabilityBudget::default\(\),', 'capability_budget: HashMap::new(),', content)
        
    if "semantic_conventions.rs" in file_path:
        content = re.sub(r'SpanBuilder::', 'clnrm_core::telemetry::semantic_conventions::SpanBuilder::', content)

    if "weaver_stats.rs" in file_path:
        content = re.sub(r'let config = WeaverConfig', 'let config = WeaverConfig { enabled: false, registry: PathBuf::from("."), ..WeaverConfig::default() };\n/// let _ = config', content)

    with open(file_path, "w") as f:
        f.write(content)
