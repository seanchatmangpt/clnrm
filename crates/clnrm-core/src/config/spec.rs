//! clnrm Configuration Format
//!
//! Single canonical TOML format with:
//! - `[test]` section for test metadata
//! - `[containers.X]` for container definitions
//! - `container = "name"` in steps (required)
//! - Parse-time validation of all references
//!
//! # Quick Start
//!
//! ```
//! use clnrm_core::config::spec::Config;
//!
//! let toml = r#"
//! [test]
//! name = "my_test"
//! timeout = "60s"
//!
//! [containers.alpine]
//! image = "alpine:latest"
//!
//! [[steps]]
//! name = "hello"
//! container = "alpine"
//! exec = ["echo", "hello"]
//! "#;
//!
//! let config = Config::from_toml(toml).unwrap(); // OK: doc example
//! assert_eq!(config.test.name, "my_test");
//! assert!(config.containers.contains_key("alpine"));
//! ```
//!
//! # Example Config (TOML Format)
//!
//! ```toml
//! [test]
//! name = "my_test"
//! timeout = "60s"
//!
//! [containers.postgres]
//! image = "postgres:15"
//! env = { POSTGRES_PASSWORD = "test" }
//! ports = ["5432:5432"]
//! healthcheck = "pg_isready -U postgres"
//!
//! [[steps]]
//! name = "verify_db"
//! container = "postgres"
//! exec = ["pg_isready", "-U", "postgres"]
//! assert.exit_code = 0
//! ```

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test Configuration
///
/// The main configuration structure for clnrm tests.
///
/// # Examples
///
/// ## Basic Configuration
///
/// ```
/// use clnrm_core::config::spec::Config;
///
/// let config = Config::from_toml(r#"
/// [test]
/// name = "basic_test"
///
/// [containers.alpine]
/// image = "alpine:latest"
///
/// [[steps]]
/// name = "run"
/// container = "alpine"
/// exec = ["echo", "hello"]
/// "#).unwrap(); // OK: doc example
///
/// assert_eq!(config.test.name, "basic_test");
/// ```
///
/// ## With Environment Variables
///
/// ```
/// use clnrm_core::config::spec::Config;
///
/// let config = Config::from_toml(r#"
/// [test]
/// name = "env_test"
///
/// [containers.app]
/// image = "alpine:latest"
/// env = { MY_VAR = "hello", DEBUG = "true" }
///
/// [[steps]]
/// name = "check_env"
/// container = "app"
/// exec = ["sh", "-c", "echo $MY_VAR"]
/// "#).unwrap(); // OK: doc example
///
/// let app = config.containers.get("app").unwrap(); // OK: doc example
/// assert_eq!(app.env.get("MY_VAR"), Some(&"hello".to_string()));
/// ```
///
/// ## With Step Dependencies
///
/// ```
/// use clnrm_core::config::spec::Config;
///
/// let config = Config::from_toml(r#"
/// [test]
/// name = "ordered_test"
///
/// [containers.alpine]
/// image = "alpine:latest"
///
/// [[steps]]
/// name = "first"
/// container = "alpine"
/// exec = ["echo", "1"]
///
/// [[steps]]
/// name = "second"
/// container = "alpine"
/// exec = ["echo", "2"]
/// depends_on = ["first"]
/// "#).unwrap(); // OK: doc example
///
/// let order = config.step_execution_order().unwrap(); // OK: doc example
/// let first_pos = order.iter().position(|&x| x == "first").unwrap(); // OK: doc example
/// let second_pos = order.iter().position(|&x| x == "second").unwrap(); // OK: doc example
/// assert!(first_pos < second_pos);
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Test metadata
    pub test: TestSection,

    /// Container definitions
    #[serde(default)]
    pub containers: HashMap<String, ContainerSpec>,

    /// Test steps
    #[serde(default)]
    pub steps: Vec<Step>,

    /// Optional parallel execution limit
    #[serde(default)]
    pub parallel: Option<u32>,
}

/// Test metadata section
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestSection {
    /// Test name (required)
    pub name: String,

    /// Test description
    #[serde(default)]
    pub description: Option<String>,

    /// Test timeout (e.g., "60s", "5m")
    #[serde(default)]
    pub timeout: Option<String>,

    /// Parallel execution limit (overrides top-level)
    #[serde(default)]
    pub parallel: Option<u32>,
}

/// Container specification
///
/// Unified container abstraction with:
/// - Image definition
/// - Environment variables
/// - Port mappings
/// - Volume mounts
/// - Health checks
/// - Dependencies
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContainerSpec {
    /// Container image (required)
    pub image: String,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Port mappings (format: "container:host" or just "port")
    #[serde(default)]
    pub ports: Vec<String>,

    /// Volume mounts
    #[serde(default)]
    pub volumes: Vec<VolumeSpec>,

    /// Health check command (string format for simplicity)
    #[serde(default)]
    pub healthcheck: Option<String>,

    /// Container dependencies (start these first)
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Command to run on container start
    #[serde(default)]
    pub command: Option<Vec<String>>,

    /// Working directory inside container
    #[serde(default)]
    pub workdir: Option<String>,
}

impl ContainerSpec {
    /// TRIZ Principle 15: Dynamics - Dynamic construction adapter
    /// Allows the struct to adapt to different construction patterns during evolution
    pub fn from_legacy_fields(
        _name: String, // This is actually the image name in legacy format
        image: String,
        tag: String,
        ports: Vec<String>,                // Legacy string format
        env_vars: HashMap<String, String>, // Legacy field name
        volumes: Vec<String>,              // Legacy string format
        depends_on: Vec<String>,
        command: Option<Vec<String>>,
        args: Option<Vec<String>>,   // Legacy field (merge into command)
        _user: Option<String>,       // Legacy field (not used in current struct)
        working_dir: Option<String>, // Legacy field name
        healthcheck: Option<String>, // Legacy field name
        _labels: HashMap<String, String>, // Legacy field (not used)
    ) -> Self {
        // Dynamically adapt legacy construction to current struct
        let full_image = if tag.is_empty() {
            image
        } else {
            format!("{}:{}", image, tag)
        };

        let mut final_command = command;
        if final_command.is_none() && args.is_some() {
            final_command = args; // Merge args into command
        }

        let healthcheck = healthcheck; // Keep field name
        let workdir = working_dir; // Rename field

        // Convert legacy volume strings to VolumeSpec
        let volumes: Vec<VolumeSpec> = volumes
            .into_iter()
            .filter_map(|vol| {
                let parts: Vec<&str> = vol.split(':').collect();
                if parts.len() >= 2 {
                    Some(VolumeSpec {
                        host: parts[0].to_string(),
                        container: parts[1].to_string(),
                        readonly: parts.get(2).map(|s| s.contains("ro")).unwrap_or(false),
                    })
                } else {
                    None
                }
            })
            .collect();

        Self {
            image: full_image,
            env: env_vars, // Rename field
            ports,
            volumes,
            depends_on,
            command: final_command,
            workdir,
            healthcheck,
        }
    }
}

/// Volume mount specification
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeSpec {
    /// Host path
    pub host: String,

    /// Container path
    pub container: String,

    /// Read-only flag
    #[serde(default)]
    pub readonly: bool,
}

/// Step configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Step {
    /// Step name (required)
    pub name: String,

    /// Target container (required)
    pub container: String,

    /// Command to execute (uses docker exec)
    pub exec: Vec<String>,

    /// Step dependencies
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Assertions
    #[serde(default)]
    pub assert: Option<StepAssertions>,

    /// Retry configuration
    #[serde(default)]
    pub retry: Option<RetryConfig>,

    /// Environment variables for this step only
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Step assertions
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct StepAssertions {
    /// Expected exit code
    #[serde(default)]
    pub exit_code: Option<i32>,

    /// Stdout should contain this string
    #[serde(default)]
    pub stdout_contains: Option<String>,

    /// Stdout should match this regex
    #[serde(default)]
    pub stdout_regex: Option<String>,

    /// Stdout should NOT contain this string
    #[serde(default)]
    pub stdout_not_contains: Option<String>,

    /// Stderr should contain this string
    #[serde(default)]
    pub stderr_contains: Option<String>,
}

/// Retry configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RetryConfig {
    /// Number of attempts
    pub attempts: u32,

    /// Delay between attempts (e.g., "1s", "500ms")
    pub delay: String,
}

impl Config {
    /// Parse config from TOML string
    ///
    /// Parses and validates the configuration in a single step.
    /// All container and step references are validated at parse time.
    ///
    /// # Examples
    ///
    /// ```
    /// use clnrm_core::config::spec::Config;
    ///
    /// let config = Config::from_toml(r#"
    /// [test]
    /// name = "example"
    ///
    /// [containers.alpine]
    /// image = "alpine:latest"
    ///
    /// [[steps]]
    /// name = "hello"
    /// container = "alpine"
    /// exec = ["echo", "hello"]
    /// "#).unwrap(); // OK: doc example
    ///
    /// assert_eq!(config.test.name, "example");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - TOML syntax is invalid
    /// - Required fields are missing
    /// - Container references don't exist
    /// - Circular dependencies are detected
    ///
    /// ```
    /// use clnrm_core::config::spec::Config;
    ///
    /// // Invalid container reference fails
    /// let result = Config::from_toml(r#"
    /// [test]
    /// name = "bad_test"
    ///
    /// [containers.alpine]
    /// image = "alpine:latest"
    ///
    /// [[steps]]
    /// name = "bad_step"
    /// container = "nonexistent"
    /// exec = ["echo", "hello"]
    /// "#);
    ///
    /// assert!(result.is_err());
    /// assert!(result.unwrap_err().to_string().contains("nonexistent"));
    /// ```
    pub fn from_toml(content: &str) -> Result<Self> {
        let config: Config = toml::from_str(content).map_err(|e| {
            CleanroomError::validation_error(format!("Failed to parse config: {}", e))
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration at parse time
    pub fn validate(&self) -> Result<()> {
        // 1. Validate test name
        if self.test.name.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Test name cannot be empty",
            ));
        }

        // 3. Validate at least one step exists
        if self.steps.is_empty() {
            return Err(CleanroomError::validation_error(
                "At least one step is required",
            ));
        }

        // 4. Validate all container references in steps exist
        for step in &self.steps {
            if !self.containers.contains_key(&step.container) {
                return Err(CleanroomError::validation_error(format!(
                    "Step '{}' references container '{}' which is not defined in [containers]",
                    step.name, step.container
                )));
            }
        }

        // 5. Validate all step.depends_on references exist
        let step_names: std::collections::HashSet<_> =
            self.steps.iter().map(|s| s.name.as_str()).collect();
        for step in &self.steps {
            for dep in &step.depends_on {
                if !step_names.contains(dep.as_str()) {
                    return Err(CleanroomError::validation_error(format!(
                        "Step '{}' depends on '{}' which is not defined",
                        step.name, dep
                    )));
                }
            }
        }

        // 6. Validate container.depends_on references exist
        let container_names: std::collections::HashSet<_> = self.containers.keys().collect();
        for (name, spec) in &self.containers {
            for dep in &spec.depends_on {
                if !container_names.contains(&dep) {
                    return Err(CleanroomError::validation_error(format!(
                        "Container '{}' depends on '{}' which is not defined",
                        name, dep
                    )));
                }
            }
        }

        // 7. Detect circular dependencies in containers
        self.detect_circular_container_deps()?;

        // 8. Detect circular dependencies in steps
        self.detect_circular_step_deps()?;

        // 9. Validate each container spec
        for (name, spec) in &self.containers {
            spec.validate().map_err(|e| {
                CleanroomError::validation_error(format!("Container '{}': {}", name, e))
            })?;
        }

        // 10. Validate each step
        for step in &self.steps {
            step.validate().map_err(|e| {
                CleanroomError::validation_error(format!("Step '{}': {}", step.name, e))
            })?;
        }

        Ok(())
    }

    /// Detect circular dependencies in containers
    fn detect_circular_container_deps(&self) -> Result<()> {
        for start in self.containers.keys() {
            let mut visited = std::collections::HashSet::new();
            let mut stack = vec![start.as_str()];

            while let Some(current) = stack.pop() {
                if visited.contains(current) {
                    return Err(CleanroomError::validation_error(format!(
                        "Circular dependency detected in containers: '{}'",
                        current
                    )));
                }
                visited.insert(current);

                if let Some(spec) = self.containers.get(current) {
                    for dep in &spec.depends_on {
                        stack.push(dep.as_str());
                    }
                }
            }
        }
        Ok(())
    }

    /// Detect circular dependencies in steps
    fn detect_circular_step_deps(&self) -> Result<()> {
        let step_map: HashMap<_, _> = self.steps.iter().map(|s| (s.name.as_str(), s)).collect();

        for step in &self.steps {
            let mut visited = std::collections::HashSet::new();
            let mut stack = vec![step.name.as_str()];

            while let Some(current) = stack.pop() {
                if visited.contains(current) {
                    return Err(CleanroomError::validation_error(format!(
                        "Circular dependency detected in steps: '{}'",
                        current
                    )));
                }
                visited.insert(current);

                if let Some(s) = step_map.get(current) {
                    for dep in &s.depends_on {
                        stack.push(dep.as_str());
                    }
                }
            }
        }
        Ok(())
    }

    /// Get execution order for containers (respecting depends_on)
    ///
    /// Returns containers in topological order based on their dependencies.
    ///
    /// # Examples
    ///
    /// ```
    /// use clnrm_core::config::spec::Config;
    ///
    /// let config = Config::from_toml(r#"
    /// [test]
    /// name = "ordered"
    ///
    /// [containers.db]
    /// image = "postgres:15"
    ///
    /// [containers.cache]
    /// image = "redis:7"
    ///
    /// [containers.app]
    /// image = "myapp:latest"
    /// depends_on = ["db", "cache"]
    ///
    /// [[steps]]
    /// name = "check"
    /// container = "app"
    /// exec = ["echo", "ok"]
    /// "#).unwrap(); // OK: doc example
    ///
    /// let order = config.container_execution_order().unwrap(); // OK: doc example
    ///
    /// // db and cache come before app
    /// let app_pos = order.iter().position(|&x| x == "app").unwrap(); // OK: doc example
    /// let db_pos = order.iter().position(|&x| x == "db").unwrap(); // OK: doc example
    /// let cache_pos = order.iter().position(|&x| x == "cache").unwrap(); // OK: doc example
    ///
    /// assert!(db_pos < app_pos);
    /// assert!(cache_pos < app_pos);
    /// ```
    pub fn container_execution_order(&self) -> Result<Vec<&str>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();

        fn visit<'a>(
            name: &'a str,
            containers: &'a HashMap<String, ContainerSpec>,
            visited: &mut std::collections::HashSet<&'a str>,
            result: &mut Vec<&'a str>,
        ) {
            if visited.contains(name) {
                return;
            }
            visited.insert(name);

            if let Some(spec) = containers.get(name) {
                for dep in &spec.depends_on {
                    visit(dep.as_str(), containers, visited, result);
                }
            }
            result.push(name);
        }

        for name in self.containers.keys() {
            visit(name.as_str(), &self.containers, &mut visited, &mut result);
        }

        Ok(result)
    }

    /// Get execution order for steps (respecting depends_on)
    pub fn step_execution_order(&self) -> Result<Vec<&str>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let step_map: HashMap<_, _> = self.steps.iter().map(|s| (s.name.as_str(), s)).collect();

        fn visit<'a>(
            name: &'a str,
            steps: &HashMap<&'a str, &'a Step>,
            visited: &mut std::collections::HashSet<&'a str>,
            result: &mut Vec<&'a str>,
        ) {
            if visited.contains(name) {
                return;
            }
            visited.insert(name);

            if let Some(step) = steps.get(name) {
                for dep in &step.depends_on {
                    visit(dep.as_str(), steps, visited, result);
                }
            }
            result.push(name);
        }

        for step in &self.steps {
            visit(step.name.as_str(), &step_map, &mut visited, &mut result);
        }

        Ok(result)
    }
}

impl ContainerSpec {
    /// Validate container specification
    pub fn validate(&self) -> Result<()> {
        if self.image.trim().is_empty() {
            return Err(CleanroomError::validation_error("Image cannot be empty"));
        }

        // Validate port format
        for port in &self.ports {
            if !Self::is_valid_port_spec(port) {
                return Err(CleanroomError::validation_error(format!(
                    "Invalid port specification: '{}'. Use 'container:host' or 'port' format.",
                    port
                )));
            }
        }

        // Validate volume paths
        for vol in &self.volumes {
            if vol.host.trim().is_empty() {
                return Err(CleanroomError::validation_error(
                    "Volume host path cannot be empty",
                ));
            }
            if vol.container.trim().is_empty() {
                return Err(CleanroomError::validation_error(
                    "Volume container path cannot be empty",
                ));
            }
        }

        Ok(())
    }

    fn is_valid_port_spec(port: &str) -> bool {
        let parts: Vec<_> = port.split(':').collect();
        match parts.len() {
            1 => parts[0].parse::<u16>().is_ok(),
            2 => parts[0].parse::<u16>().is_ok() && parts[1].parse::<u16>().is_ok(),
            _ => false,
        }
    }
}

impl Step {
    /// Validate step configuration
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Step name cannot be empty",
            ));
        }

        if self.container.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Step container cannot be empty",
            ));
        }

        if self.exec.is_empty() {
            return Err(CleanroomError::validation_error(
                "Step exec command cannot be empty",
            ));
        }

        // Validate regex patterns if present
        if let Some(assertions) = &self.assert {
            if let Some(regex) = &assertions.stdout_regex {
                regex::Regex::new(regex).map_err(|e| {
                    CleanroomError::validation_error(format!("Invalid stdout_regex: {}", e))
                })?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let config = r#"
[test]
name = "my_test"
timeout = "60s"

[containers.alpine]
image = "alpine:latest"
env = { MY_VAR = "hello" }

[[steps]]
name = "verify"
container = "alpine"
exec = ["echo", "hello"]
"#;
        let result = Config::from_toml(config);
        assert!(result.is_ok());
        let cfg = result.unwrap();
        assert_eq!(cfg.test.name, "my_test");
        assert_eq!(cfg.containers.len(), 1);
        assert_eq!(cfg.steps.len(), 1);
    }

    #[test]
    fn test_parse_invalid_container_ref() {
        let config = r#"
[test]
name = "my_test"

[containers.alpine]
image = "alpine:latest"

[[steps]]
name = "verify"
container = "nonexistent"
exec = ["echo", "hello"]
"#;
        let result = Config::from_toml(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nonexistent"));
    }

    #[test]
    fn test_parse_invalid_step_dep() {
        let config = r#"
[test]
name = "my_test"

[containers.alpine]
image = "alpine:latest"

[[steps]]
name = "step1"
container = "alpine"
exec = ["echo", "hello"]
depends_on = ["nonexistent_step"]
"#;
        let result = Config::from_toml(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nonexistent_step"));
    }

    #[test]
    fn test_container_execution_order() {
        let config = r#"
[test]
name = "ordered_test"

[containers.db]
image = "postgres:15"

[containers.app]
image = "myapp:latest"
depends_on = ["db"]

[[steps]]
name = "check"
container = "app"
exec = ["echo", "ok"]
"#;
        let cfg = Config::from_toml(config).unwrap();
        let order = cfg.container_execution_order().unwrap();

        // db should come before app
        let db_pos = order.iter().position(|&x| x == "db").unwrap();
        let app_pos = order.iter().position(|&x| x == "app").unwrap();
        assert!(db_pos < app_pos);
    }

    #[test]
    fn test_circular_dep_detection() {
        let config = r#"
[test]
name = "circular_test"

[containers.a]
image = "alpine:latest"
depends_on = ["b"]

[containers.b]
image = "alpine:latest"
depends_on = ["a"]

[[steps]]
name = "check"
container = "a"
exec = ["echo", "ok"]
"#;
        let result = Config::from_toml(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular"));
    }

    #[test]
    fn test_env_vars_on_container() {
        let config = r#"
[test]
name = "env_test"

[containers.alpine]
image = "alpine:latest"
env = { MY_VAR = "hello", ANOTHER = "world" }

[[steps]]
name = "check_env"
container = "alpine"
exec = ["sh", "-c", "echo $MY_VAR"]
"#;
        let cfg = Config::from_toml(config).unwrap();
        let container = cfg.containers.get("alpine").unwrap();
        assert_eq!(container.env.get("MY_VAR"), Some(&"hello".to_string()));
        assert_eq!(container.env.get("ANOTHER"), Some(&"world".to_string()));
    }

    #[test]
    fn test_step_assertions() {
        let config = r#"
[test]
name = "assertion_test"

[containers.alpine]
image = "alpine:latest"
env = { MY_VAR = "hello_world" }

[[steps]]
name = "verify_env"
container = "alpine"
exec = ["sh", "-c", "echo $MY_VAR"]

[steps.assert]
exit_code = 0
stdout_contains = "hello_world"
"#;
        let cfg = Config::from_toml(config).unwrap();
        assert_eq!(cfg.steps.len(), 1);
        let step = &cfg.steps[0];
        assert!(step.assert.is_some());
        let assertions = step.assert.as_ref().unwrap();
        assert_eq!(assertions.exit_code, Some(0));
        assert_eq!(assertions.stdout_contains, Some("hello_world".to_string()));
    }

    #[test]
    fn test_step_dependencies() {
        let config = r#"
[test]
name = "dep_test"

[containers.alpine]
image = "alpine:latest"

[[steps]]
name = "step1"
container = "alpine"
exec = ["echo", "first"]

[[steps]]
name = "step2"
container = "alpine"
exec = ["echo", "second"]
depends_on = ["step1"]

[[steps]]
name = "step3"
container = "alpine"
exec = ["echo", "third"]
depends_on = ["step2"]
"#;
        let cfg = Config::from_toml(config).unwrap();
        let order = cfg.step_execution_order().unwrap();

        let step1_pos = order.iter().position(|&x| x == "step1").unwrap();
        let step2_pos = order.iter().position(|&x| x == "step2").unwrap();
        let step3_pos = order.iter().position(|&x| x == "step3").unwrap();

        assert!(step1_pos < step2_pos);
        assert!(step2_pos < step3_pos);
    }

    #[test]
    fn test_multiple_env_vars() {
        // Test the exact format from env-vars-test.clnrm.toml
        let config = r#"
[test]
name = "env_vars_validation"
description = "Validates environment variables are available in docker exec"
timeout = "30s"

[containers.alpine]
image = "alpine:latest"
env = { MY_VAR = "hello_world", ANOTHER_VAR = "test_value", DB_HOST = "localhost" }

[[steps]]
name = "verify_my_var"
container = "alpine"
exec = ["sh", "-c", "echo $MY_VAR"]

[steps.assert]
exit_code = 0
stdout_contains = "hello_world"

[[steps]]
name = "verify_another_var"
container = "alpine"
exec = ["sh", "-c", "echo $ANOTHER_VAR"]
depends_on = ["verify_my_var"]

[steps.assert]
exit_code = 0
stdout_contains = "test_value"

[[steps]]
name = "verify_db_host"
container = "alpine"
exec = ["sh", "-c", "echo $DB_HOST"]
depends_on = ["verify_another_var"]

[steps.assert]
exit_code = 0
stdout_contains = "localhost"
"#;
        let cfg = Config::from_toml(config).unwrap();

        // Verify config parsed correctly
        assert_eq!(cfg.test.name, "env_vars_validation");
        assert_eq!(cfg.test.timeout, Some("30s".to_string()));

        // Verify container env vars
        let container = cfg.containers.get("alpine").unwrap();
        assert_eq!(container.env.len(), 3);
        assert_eq!(
            container.env.get("MY_VAR"),
            Some(&"hello_world".to_string())
        );
        assert_eq!(
            container.env.get("ANOTHER_VAR"),
            Some(&"test_value".to_string())
        );
        assert_eq!(container.env.get("DB_HOST"), Some(&"localhost".to_string()));

        // Verify steps with assertions
        assert_eq!(cfg.steps.len(), 3);

        // Step 1: no dependencies
        assert!(cfg.steps[0].depends_on.is_empty());
        assert!(cfg.steps[0].assert.is_some());

        // Step 2: depends on step 1
        assert_eq!(cfg.steps[1].depends_on, vec!["verify_my_var"]);

        // Step 3: depends on step 2
        assert_eq!(cfg.steps[2].depends_on, vec!["verify_another_var"]);
    }
}
