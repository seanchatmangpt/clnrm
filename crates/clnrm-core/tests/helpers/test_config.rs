//! Test Configuration Helpers
//!
//! Utilities for generating test TOML configurations with Weaver live-check enabled.

use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};

/// Builder for creating test TOML configurations
pub struct TestConfigBuilder {
    test_name: String,
    weaver_enabled: bool,
    validation_mode: String,
    otlp_port: Option<u16>,
    admin_port: Option<u16>,
    fail_fast: bool,
    stream: bool,
    critical_spans: Vec<String>,
    required_attributes: Vec<String>,
    services: Vec<(String, String)>, // (name, image)
    steps: Vec<TestStep>,
}

/// Test step configuration
pub struct TestStep {
    pub name: String,
    pub command: Vec<String>,
    pub service: Option<String>,
    pub expected_output_regex: Option<String>,
}

impl TestConfigBuilder {
    /// Create new builder with test name
    pub fn new(test_name: impl Into<String>) -> Self {
        Self {
            test_name: test_name.into(),
            weaver_enabled: true,
            validation_mode: "strict".to_string(),
            otlp_port: None,
            admin_port: None,
            fail_fast: false,
            stream: false,
            critical_spans: Vec::new(),
            required_attributes: Vec::new(),
            services: Vec::new(),
            steps: Vec::new(),
        }
    }

    /// Enable or disable Weaver
    pub fn weaver_enabled(mut self, enabled: bool) -> Self {
        self.weaver_enabled = enabled;
        self
    }

    /// Set validation mode
    pub fn validation_mode(mut self, mode: impl Into<String>) -> Self {
        self.validation_mode = mode.into();
        self
    }

    /// Set OTLP port
    pub fn otlp_port(mut self, port: u16) -> Self {
        self.otlp_port = Some(port);
        self
    }

    /// Set admin port
    pub fn admin_port(mut self, port: u16) -> Self {
        self.admin_port = Some(port);
        self
    }

    /// Enable fail-fast mode
    pub fn fail_fast(mut self, enabled: bool) -> Self {
        self.fail_fast = enabled;
        self
    }

    /// Enable streaming output
    pub fn stream(mut self, enabled: bool) -> Self {
        self.stream = enabled;
        self
    }

    /// Add critical span for 80/20 mode
    pub fn add_critical_span(mut self, span: impl Into<String>) -> Self {
        self.critical_spans.push(span.into());
        self
    }

    /// Add required attribute for 80/20 mode
    pub fn add_required_attribute(mut self, attr: impl Into<String>) -> Self {
        self.required_attributes.push(attr.into());
        self
    }

    /// Add a service
    pub fn add_service(mut self, name: impl Into<String>, image: impl Into<String>) -> Self {
        self.services.push((name.into(), image.into()));
        self
    }

    /// Add a test step
    pub fn add_step(mut self, step: TestStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Build the TOML configuration
    pub fn build(self) -> String {
        let mut toml = String::new();

        // Test metadata
        toml.push_str("[test.metadata]\n");
        toml.push_str(&format!("name = \"{}\"\n", self.test_name));
        toml.push_str(&format!(
            "description = \"Test configuration for {}\"\n\n",
            self.test_name
        ));

        // Weaver configuration
        if self.weaver_enabled {
            toml.push_str("[weaver]\n");
            toml.push_str("enabled = true\n");
            toml.push_str("registry_path = \"registry\"\n");

            if let Some(port) = self.otlp_port {
                toml.push_str(&format!("otlp_port = {}\n", port));
            }

            if let Some(port) = self.admin_port {
                toml.push_str(&format!("admin_port = {}\n", port));
            }

            toml.push_str(&format!("stream = {}\n", self.stream));
            toml.push_str(&format!("fail_fast = {}\n\n", self.fail_fast));

            // Validation configuration
            toml.push_str("[weaver.validation]\n");
            toml.push_str(&format!("mode = \"{}\"\n", self.validation_mode));
            toml.push_str("fail_on_violation = true\n");
            toml.push_str("coverage_threshold = 80.0\n\n");

            // 80/20 configuration
            if !self.critical_spans.is_empty() || !self.required_attributes.is_empty() {
                toml.push_str("[weaver.80_20]\n");

                if !self.critical_spans.is_empty() {
                    toml.push_str("critical_spans = [\n");
                    for span in &self.critical_spans {
                        toml.push_str(&format!("  \"{}\",\n", span));
                    }
                    toml.push_str("]\n");
                }

                if !self.required_attributes.is_empty() {
                    toml.push_str("required_attributes = [\n");
                    for attr in &self.required_attributes {
                        toml.push_str(&format!("  \"{}\",\n", attr));
                    }
                    toml.push_str("]\n");
                }

                toml.push_str("\n");
            }
        }

        // Services
        for (name, image) in &self.services {
            toml.push_str(&format!("[services.{}]\n", name));
            toml.push_str("type = \"generic_container\"\n");
            toml.push_str(&format!("image = \"{}\"\n\n", image));
        }

        // Steps
        for step in &self.steps {
            toml.push_str("[[steps]]\n");
            toml.push_str(&format!("name = \"{}\"\n", step.name));

            // Command as array
            toml.push_str("command = [");
            for (i, cmd) in step.command.iter().enumerate() {
                if i > 0 {
                    toml.push_str(", ");
                }
                toml.push_str(&format!("\"{}\"", cmd));
            }
            toml.push_str("]\n");

            if let Some(ref service) = step.service {
                toml.push_str(&format!("service = \"{}\"\n", service));
            }

            if let Some(ref regex) = step.expected_output_regex {
                toml.push_str(&format!("expected_output_regex = \"{}\"\n", regex));
            }

            toml.push_str("\n");
        }

        toml
    }

    /// Write configuration to file
    pub fn write_to_file(self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let toml = self.build();
        let mut file = std::fs::File::create(path)?;
        file.write_all(toml.as_bytes())?;
        Ok(())
    }

    /// Create a temporary file with the configuration
    pub fn create_temp_file(self) -> std::io::Result<NamedTempFile> {
        let toml = self.build();
        let mut file = NamedTempFile::new()?;
        file.write_all(toml.as_bytes())?;
        Ok(file)
    }
}

// ============================================================================
// Preset Configurations
// ============================================================================

/// Create a minimal test configuration with live-check
pub fn minimal_live_check_config(test_name: &str) -> String {
    TestConfigBuilder::new(test_name)
        .weaver_enabled(true)
        .validation_mode("minimal")
        .add_service("alpine", "alpine:latest")
        .add_step(TestStep {
            name: "echo_test".to_string(),
            command: vec!["echo".to_string(), "hello".to_string()],
            service: Some("alpine".to_string()),
            expected_output_regex: Some("hello".to_string()),
        })
        .build()
}

/// Create a strict validation configuration
pub fn strict_validation_config(test_name: &str) -> String {
    TestConfigBuilder::new(test_name)
        .weaver_enabled(true)
        .validation_mode("strict")
        .fail_fast(false)
        .add_service("alpine", "alpine:latest")
        .add_step(TestStep {
            name: "test_step".to_string(),
            command: vec!["echo".to_string(), "test".to_string()],
            service: Some("alpine".to_string()),
            expected_output_regex: Some("test".to_string()),
        })
        .build()
}

/// Create 80/20 validation configuration
pub fn eighty_twenty_config(test_name: &str) -> String {
    TestConfigBuilder::new(test_name)
        .weaver_enabled(true)
        .validation_mode("80_20")
        .add_critical_span("clnrm.test.execute")
        .add_critical_span("clnrm.container.start")
        .add_critical_span("clnrm.container.stop")
        .add_required_attribute("container.id")
        .add_required_attribute("test.hermetic")
        .add_service("alpine", "alpine:latest")
        .add_step(TestStep {
            name: "quick_test".to_string(),
            command: vec!["echo".to_string(), "fast".to_string()],
            service: Some("alpine".to_string()),
            expected_output_regex: Some("fast".to_string()),
        })
        .build()
}

/// Create a configuration with live-check disabled
pub fn disabled_live_check_config(test_name: &str) -> String {
    TestConfigBuilder::new(test_name)
        .weaver_enabled(false)
        .add_service("alpine", "alpine:latest")
        .add_step(TestStep {
            name: "traditional_test".to_string(),
            command: vec!["echo".to_string(), "traditional".to_string()],
            service: Some("alpine".to_string()),
            expected_output_regex: Some("traditional".to_string()),
        })
        .build()
}

/// Create a multi-service configuration
pub fn multi_service_config(test_name: &str) -> String {
    TestConfigBuilder::new(test_name)
        .weaver_enabled(true)
        .validation_mode("strict")
        .add_service("alpine", "alpine:latest")
        .add_service("busybox", "busybox:latest")
        .add_step(TestStep {
            name: "test_alpine".to_string(),
            command: vec!["echo".to_string(), "alpine".to_string()],
            service: Some("alpine".to_string()),
            expected_output_regex: Some("alpine".to_string()),
        })
        .add_step(TestStep {
            name: "test_busybox".to_string(),
            command: vec!["echo".to_string(), "busybox".to_string()],
            service: Some("busybox".to_string()),
            expected_output_regex: Some("busybox".to_string()),
        })
        .build()
}

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a temporary directory with test configuration
pub fn create_test_dir_with_config(config_toml: &str) -> std::io::Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join(".clnrm.toml");
    std::fs::write(config_path, config_toml)?;
    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config_generation() {
        let config = minimal_live_check_config("test_minimal");
        assert!(config.contains("[weaver]"));
        assert!(config.contains("enabled = true"));
        assert!(config.contains("mode = \"minimal\""));
        assert!(config.contains("[services.alpine]"));
    }

    #[test]
    fn test_strict_config_generation() {
        let config = strict_validation_config("test_strict");
        assert!(config.contains("mode = \"strict\""));
        assert!(config.contains("fail_fast = false"));
    }

    #[test]
    fn test_eighty_twenty_config_generation() {
        let config = eighty_twenty_config("test_80_20");
        assert!(config.contains("mode = \"80_20\""));
        assert!(config.contains("[weaver.80_20]"));
        assert!(config.contains("critical_spans"));
        assert!(config.contains("clnrm.test.execute"));
    }

    #[test]
    fn test_disabled_config_generation() {
        let config = disabled_live_check_config("test_disabled");
        // Should not have weaver section when disabled
        assert!(!config.contains("[weaver]") || config.contains("enabled = false"));
    }

    #[test]
    fn test_multi_service_config() {
        let config = multi_service_config("test_multi");
        assert!(config.contains("[services.alpine]"));
        assert!(config.contains("[services.busybox]"));
        assert!(config.contains("test_alpine"));
        assert!(config.contains("test_busybox"));
    }

    #[test]
    fn test_config_builder_ports() {
        let config = TestConfigBuilder::new("test_ports")
            .otlp_port(4317)
            .admin_port(8080)
            .build();

        assert!(config.contains("otlp_port = 4317"));
        assert!(config.contains("admin_port = 8080"));
    }

    #[test]
    fn test_config_builder_critical_spans() {
        let config = TestConfigBuilder::new("test_spans")
            .validation_mode("80_20")
            .add_critical_span("span1")
            .add_critical_span("span2")
            .add_required_attribute("attr1")
            .build();

        assert!(config.contains("critical_spans"));
        assert!(config.contains("\"span1\""));
        assert!(config.contains("\"span2\""));
        assert!(config.contains("required_attributes"));
        assert!(config.contains("\"attr1\""));
    }
}
