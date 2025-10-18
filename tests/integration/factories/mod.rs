//! Test data factories for integration tests
//!
//! Factories provide a fluent API for building test data with reasonable
//! defaults and the ability to customize specific fields.

use rand::Rng;
use std::collections::HashMap;

/// Builder for creating test backend configurations
#[derive(Debug, Clone)]
pub struct BackendConfigBuilder {
    name: String,
    image: String,
    tag: String,
    env_vars: HashMap<String, String>,
    timeout: u64,
    hermetic: bool,
    deterministic: bool,
}

impl Default for BackendConfigBuilder {
    fn default() -> Self {
        Self {
            name: "test-backend".to_string(),
            image: "alpine".to_string(),
            tag: "latest".to_string(),
            env_vars: HashMap::new(),
            timeout: 30,
            hermetic: true,
            deterministic: true,
        }
    }
}

impl BackendConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.image = image.into();
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn hermetic(mut self, hermetic: bool) -> Self {
        self.hermetic = hermetic;
        self
    }

    pub fn deterministic(mut self, deterministic: bool) -> Self {
        self.deterministic = deterministic;
        self
    }

    pub fn full_image(&self) -> String {
        format!("{}:{}", self.image, self.tag)
    }

    pub fn build(self) -> BackendConfig {
        BackendConfig {
            name: self.name,
            image: self.image,
            tag: self.tag,
            env_vars: self.env_vars,
            timeout: self.timeout,
            hermetic: self.hermetic,
            deterministic: self.deterministic,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub name: String,
    pub image: String,
    pub tag: String,
    pub env_vars: HashMap<String, String>,
    pub timeout: u64,
    pub hermetic: bool,
    pub deterministic: bool,
}

/// Builder for creating test commands
#[derive(Debug, Clone)]
pub struct CommandBuilder {
    command: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
    working_dir: Option<String>,
}

impl Default for CommandBuilder {
    fn default() -> Self {
        Self {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            env_vars: HashMap::new(),
            working_dir: None,
        }
    }
}

impl CommandBuilder {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(|a| a.into()));
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    pub fn working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn build(self) -> TestCommand {
        TestCommand {
            command: self.command,
            args: self.args,
            env_vars: self.env_vars,
            working_dir: self.working_dir,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestCommand {
    pub command: String,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<String>,
}

/// Builder for creating test results
#[derive(Debug, Clone)]
pub struct ResultBuilder {
    exit_code: i32,
    stdout: String,
    stderr: String,
    duration_ms: u64,
    backend: String,
    concurrent: bool,
}

impl Default for ResultBuilder {
    fn default() -> Self {
        Self {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 100,
            backend: "testcontainers".to_string(),
            concurrent: false,
        }
    }
}

impl ResultBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }

    pub fn stdout(mut self, stdout: impl Into<String>) -> Self {
        self.stdout = stdout.into();
        self
    }

    pub fn stderr(mut self, stderr: impl Into<String>) -> Self {
        self.stderr = stderr.into();
        self
    }

    pub fn duration_ms(mut self, duration: u64) -> Self {
        self.duration_ms = duration;
        self
    }

    pub fn backend(mut self, backend: impl Into<String>) -> Self {
        self.backend = backend.into();
        self
    }

    pub fn concurrent(mut self, concurrent: bool) -> Self {
        self.concurrent = concurrent;
        self
    }

    pub fn build(self) -> TestResult {
        TestResult {
            exit_code: self.exit_code,
            stdout: self.stdout,
            stderr: self.stderr,
            duration_ms: self.duration_ms,
            backend: self.backend,
            concurrent: self.concurrent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub backend: String,
    pub concurrent: bool,
}

/// Generate random test data
pub struct RandomDataGenerator {
    rng: rand::rngs::ThreadRng,
}

impl Default for RandomDataGenerator {
    fn default() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
}

impl RandomDataGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn random_string(&mut self, length: usize) -> String {
        use rand::distributions::Alphanumeric;
        use rand::Rng;

        self.rng
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }

    pub fn random_port(&mut self) -> u16 {
        self.rng.gen_range(10000..60000)
    }

    pub fn random_duration_ms(&mut self) -> u64 {
        self.rng.gen_range(10..1000)
    }

    pub fn random_exit_code(&mut self) -> i32 {
        self.rng.gen_range(0..3)
    }

    pub fn random_bool(&mut self) -> bool {
        self.rng.gen_bool(0.5)
    }
}
