//! Integration helpers for web frameworks and CLI tools
//!
//! Provides integration utilities for common use cases:
//! - Web framework integration (Axum, Actix, Warp)
//! - CLI tool integration with command-line arguments
//! - Template server for development
//! - Configuration management for applications

use crate::error::{TemplateError, Result};
use crate::context::{TemplateContext, TemplateContextBuilder};
use crate::renderer::{TemplateRenderer, OutputFormat};
use crate::simple::{render, render_to_format, TemplateBuilder};
use crate::validation::{TemplateValidator, ValidationRule};
use crate::builder::TemplateEngineBuilder;
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;

/// Web framework integration helpers
pub mod web {
    use super::*;

    /// Template context for web applications
    ///
    /// Provides common variables for web application templates
    pub fn web_context() -> TemplateContextBuilder {
        TemplateContextBuilder::with_defaults()
            .var("timestamp", chrono::Utc::now().to_rfc3339())
            .var("environment", std::env::var("ENV").unwrap_or_else(|_| "development".to_string()))
    }

    /// Render template for HTTP response
    ///
    /// # Arguments
    /// * `template` - Template content
    /// * `vars` - Template variables
    /// * `format` - Output format for response
    pub fn render_response(template: &str, vars: HashMap<&str, &str>, format: OutputFormat) -> Result<(String, String)> {
        let mut json_vars = HashMap::new();
        for (key, value) in vars {
            json_vars.insert(key.to_string(), Value::String(value.to_string()));
        }

        let content = render_to_format(template, json_vars, format)?;

        let content_type = match format {
            OutputFormat::Json => "application/json",
            OutputFormat::Yaml => "application/x-yaml",
            OutputFormat::Toml => "application/toml",
            OutputFormat::Plain => "text/plain",
        };

        Ok((content, content_type.to_string()))
    }

    /// Template middleware for web frameworks
    ///
    /// Provides template rendering capability for web handlers
    pub struct TemplateMiddleware {
        /// Template engine for rendering
        engine: TemplateRenderer,
        /// Default output format
        format: OutputFormat,
    }

    impl TemplateMiddleware {
        /// Create new template middleware
        pub fn new() -> Result<Self> {
            Ok(Self {
                engine: TemplateRenderer::new()?,
                format: OutputFormat::Toml,
            })
        }

        /// Set default output format
        pub fn with_format(mut self, format: OutputFormat) -> Self {
            self.format = format;
            self
        }

        /// Render template with variables
        pub fn render(&mut self, template: &str, vars: HashMap<&str, &str>) -> Result<String> {
            let mut json_vars = HashMap::new();
            for (key, value) in vars {
                json_vars.insert(key.to_string(), Value::String(value.to_string()));
            }

            self.engine.merge_user_vars(json_vars);
            self.engine.render_str(template, "web_template")
        }

        /// Render template to specific format
        pub fn render_to_format(&mut self, template: &str, vars: HashMap<&str, &str>, format: OutputFormat) -> Result<String> {
            let rendered = self.render(template, vars)?;
            match format {
                OutputFormat::Toml => Ok(rendered),
                OutputFormat::Json => crate::simple::convert_to_json(&rendered),
                OutputFormat::Yaml => crate::simple::convert_to_yaml(&rendered),
                OutputFormat::Plain => crate::simple::strip_template_syntax(&rendered),
            }
        }
    }

    /// Axum handler for template rendering
    ///
    /// Example usage:
    /// ```rust,ignore
    /// use axum::{routing::get, Router};
    /// use clnrm_template::integration::web::axum_template_handler;
    ///
    /// let app = Router::new()
    ///     .route("/config", get(axum_template_handler));
    /// ```
    pub async fn axum_template_handler() -> Result<axum::response::Response<String>, axum::response::Response<String>> {
        let mut vars = HashMap::new();
        vars.insert("service", "web-api");
        vars.insert("port", "3000");

        let (content, content_type) = render_response(
            "service = \"{{ service }}\"\nport = {{ port }}",
            vars,
            OutputFormat::Toml
        ).map_err(|e| axum::response::Response::builder()
            .status(500)
            .body(format!("Template error: {}", e))
            .unwrap())?;

        Ok(axum::response::Response::builder()
            .header("content-type", content_type)
            .body(content)
            .unwrap())
    }

    /// Actix-web handler for template rendering
    pub async fn actix_template_handler() -> Result<actix_web::HttpResponse, actix_web::Error> {
        let mut vars = HashMap::new();
        vars.insert("service", "web-api");
        vars.insert("version", "1.0.0");

        let content = render_to_format(
            "Service: {{ service }} v{{ version }}",
            vars,
            OutputFormat::Plain
        ).map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

        Ok(actix_web::HttpResponse::Ok()
            .content_type("text/plain")
            .body(content))
    }
}

/// CLI tool integration helpers
pub mod cli {
    use super::*;
    use clap::{Arg, ArgMatches, Command};

    /// Create CLI command for template rendering
    pub fn template_command() -> Command {
        Command::new("template")
            .about("Render templates with variables")
            .arg(Arg::new("template")
                .help("Template file to render")
                .required(true)
                .index(1))
            .arg(Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file (default: stdout)")
                .value_name("FILE"))
            .arg(Arg::new("format")
                .short('f')
                .long("format")
                .help("Output format: toml, json, yaml, plain")
                .default_value("toml"))
            .arg(Arg::new("variable")
                .short('v')
                .long("var")
                .help("Template variables (key=value)")
                .action(clap::ArgAction::Append)
                .value_name("KEY=VALUE"))
            .arg(Arg::new("context")
                .short('c')
                .long("context")
                .help("Context file with variables (JSON/TOML)"))
    }

    /// Execute template command
    ///
    /// # Arguments
    /// * `matches` - Command line arguments
    pub fn execute_template_command(matches: &ArgMatches) -> Result<()> {
        let template_path = matches.get_one::<String>("template")
            .ok_or_else(|| TemplateError::ValidationError("Template file required".to_string()))?;

        let format_str = matches.get_one::<String>("format")
            .map(|s| s.as_str())
            .unwrap_or("toml");

        let format = match format_str {
            "json" => OutputFormat::Json,
            "yaml" => OutputFormat::Yaml,
            "plain" => OutputFormat::Plain,
            "toml" | _ => OutputFormat::Toml,
        };

        // Parse variables
        let mut vars = HashMap::new();
        if let Some(var_strings) = matches.get_many::<String>("variable") {
            for var_str in var_strings {
                if let Some((key, value)) = var_str.split_once('=') {
                    vars.insert(key, value);
                }
            }
        }

        // Load context from file if provided
        if let Some(context_path) = matches.get_one::<String>("context") {
            let context_content = std::fs::read_to_string(context_path)
                .map_err(|e| TemplateError::IoError(format!("Failed to read context file: {}", e)))?;

            // Try to parse as JSON first, then TOML
            let context_vars: HashMap<String, Value> = if let Ok(json) = serde_json::from_str(&context_content) {
                json
            } else if let Ok(toml) = toml::from_str(&context_content) {
                toml
            } else {
                return Err(TemplateError::ValidationError(
                    "Context file must be valid JSON or TOML".to_string()
                ));
            };

            // Merge context variables
            for (key, value) in context_vars {
                vars.insert(&key, match value {
                    Value::String(s) => s.as_str(),
                    _ => value.to_string().as_str(),
                });
            }
        }

        // Render template
        let result = render_to_format(template_path, vars, format)?;

        // Output result
        if let Some(output_path) = matches.get_one::<String>("output") {
            std::fs::write(output_path, &result)
                .map_err(|e| TemplateError::IoError(format!("Failed to write output: {}", e)))?;
            println!("Template rendered to: {}", output_path);
        } else {
            println!("{}", result);
        }

        Ok(())
    }

    /// Template CLI application
    ///
    /// Provides a complete CLI tool for template rendering
    pub struct TemplateCli {
        /// Base command
        command: Command,
    }

    impl TemplateCli {
        /// Create new template CLI
        pub fn new() -> Self {
            Self {
                command: template_command(),
            }
        }

        /// Run CLI with arguments
        pub fn run(self, args: Vec<String>) -> Result<()> {
            let matches = self.command.get_matches_from(args);
            execute_template_command(&matches)
        }

        /// Get the command for integration with other CLI tools
        pub fn command(self) -> Command {
            self.command
        }
    }
}

/// Template server for development workflows
pub mod server {
    use super::*;
    use std::net::SocketAddr;

    /// Development template server
    ///
    /// Provides HTTP endpoints for template rendering during development
    pub struct TemplateServer {
        /// Template engine
        engine: TemplateEngineBuilder,
        /// Server address
        address: SocketAddr,
    }

    impl TemplateServer {
        /// Create new template server
        ///
        /// # Arguments
        /// * `address` - Server address to bind to
        pub fn new(address: SocketAddr) -> Self {
            Self {
                engine: TemplateEngineBuilder::new(),
                address,
            }
        }

        /// Configure template engine
        pub fn with_engine<F>(mut self, f: F) -> Self
        where
            F: FnOnce(TemplateEngineBuilder) -> TemplateEngineBuilder,
        {
            self.engine = f(self.engine);
            self
        }

        /// Start development server (simplified implementation)
        pub async fn start(self) -> Result<()> {
            // In a real implementation, this would start an HTTP server
            // with endpoints for template rendering, validation, etc.
            println!("Template development server starting on {}", self.address);
            println!("Features: template rendering, validation, linting");

            // For now, just return success
            Ok(())
        }
    }
}

/// Application configuration helpers
pub mod config {
    use super::*;

    /// Load application configuration from templates
    ///
    /// # Arguments
    /// * `config_paths` - Paths to configuration templates
    /// * `env` - Environment name
    pub fn load_app_config(config_paths: Vec<&Path>, env: &str) -> Result<HashMap<String, Value>> {
        let mut config = HashMap::new();

        for path in config_paths {
            let template_content = std::fs::read_to_string(path)
                .map_err(|e| TemplateError::IoError(format!("Failed to read config template: {}", e)))?;

            let mut vars = HashMap::new();
            vars.insert("environment", env);

            let rendered = render(&template_content, vars)?;

            // Parse rendered config
            let parsed: Value = if path.extension().and_then(|s| s.to_str()) == Some("json") {
                serde_json::from_str(&rendered)
                    .map_err(|e| TemplateError::ValidationError(format!("Invalid JSON config: {}", e)))?
            } else {
                toml::from_str(&rendered)
                    .map_err(|e| TemplateError::ValidationError(format!("Invalid TOML config: {}", e)))?
            };

            // Merge into config
            if let Value::Object(obj) = parsed {
                config.extend(obj);
            }
        }

        Ok(config)
    }

    /// Generate configuration files for deployment
    ///
    /// # Arguments
    /// * `template_dir` - Directory containing configuration templates
    /// * `output_dir` - Directory to write generated configs
    /// * `environment` - Target environment
    pub fn generate_deployment_configs(template_dir: &Path, output_dir: &Path, environment: &str) -> Result<()> {
        use walkdir::WalkDir;

        std::fs::create_dir_all(output_dir)
            .map_err(|e| TemplateError::IoError(format!("Failed to create output directory: {}", e)))?;

        for entry in WalkDir::new(template_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let template_path = entry.path();
                let relative_path = template_path.strip_prefix(template_dir)
                    .map_err(|_| TemplateError::ValidationError("Invalid template path".to_string()))?;

                let output_path = output_dir.join(relative_path);

                // Create output directory structure
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| TemplateError::IoError(format!("Failed to create output directory: {}", e)))?;
                }

                let template_content = std::fs::read_to_string(template_path)
                    .map_err(|e| TemplateError::IoError(format!("Failed to read template: {}", e)))?;

                let mut vars = HashMap::new();
                vars.insert("environment", environment);
                vars.insert("timestamp", chrono::Utc::now().to_rfc3339().as_str());

                let rendered = render(&template_content, vars)?;

                std::fs::write(&output_path, rendered)
                    .map_err(|e| TemplateError::IoError(format!("Failed to write config: {}", e)))?;

                println!("Generated: {}", output_path.display());
            }
        }

        Ok(())
    }
}

/// Database integration helpers
pub mod database {
    use super::*;

    /// Template context for database operations
    pub fn database_context() -> TemplateContextBuilder {
        TemplateContextBuilder::with_defaults()
            .var("database_url", std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://db.sqlite".to_string()))
            .var("database_type", std::env::var("DATABASE_TYPE").unwrap_or_else(|_| "sqlite".to_string()))
    }

    /// Render database migration template
    ///
    /// # Arguments
    /// * `template` - Migration template
    /// * `migration_name` - Name of migration
    pub fn render_migration(template: &str, migration_name: &str) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("migration_name", migration_name);
        vars.insert("timestamp", chrono::Utc::now().timestamp().to_string().as_str());

        render(template, vars)
    }

    /// Generate database schema from templates
    ///
    /// # Arguments
    /// * `schema_templates` - Schema template files
    /// * `output_file` - Output schema file
    pub fn generate_schema(schema_templates: Vec<&Path>, output_file: &Path) -> Result<()> {
        let mut schema_parts = Vec::new();

        for template_path in schema_templates {
            let template_content = std::fs::read_to_string(template_path)
                .map_err(|e| TemplateError::IoError(format!("Failed to read schema template: {}", e)))?;

            let rendered = render(&template_content, HashMap::new())?;
            schema_parts.push(rendered);
        }

        let full_schema = schema_parts.join("\n\n");
        std::fs::write(output_file, full_schema)
            .map_err(|e| TemplateError::IoError(format!("Failed to write schema: {}", e)))?;

        Ok(())
    }
}

/// Docker integration helpers
pub mod docker {
    use super::*;

    /// Template context for Docker operations
    pub fn docker_context() -> TemplateContextBuilder {
        TemplateContextBuilder::with_defaults()
            .var("docker_registry", std::env::var("DOCKER_REGISTRY").unwrap_or_else(|_| "localhost:5000".to_string()))
            .var("image_tag", std::env::var("IMAGE_TAG").unwrap_or_else(|_| "latest".to_string()))
    }

    /// Render Dockerfile from template
    ///
    /// # Arguments
    /// * `template` - Dockerfile template
    /// * `base_image` - Base Docker image
    /// * `app_name` - Application name
    pub fn render_dockerfile(template: &str, base_image: &str, app_name: &str) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("base_image", base_image);
        vars.insert("app_name", app_name);

        render(template, vars)
    }

    /// Render docker-compose.yml from template
    ///
    /// # Arguments
    /// * `template` - docker-compose template
    /// * `service_name` - Service name
    /// * `image_name` - Docker image name
    pub fn render_docker_compose(template: &str, service_name: &str, image_name: &str) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("service_name", service_name);
        vars.insert("image_name", image_name);

        render_to_format(template, vars, OutputFormat::Yaml)
    }
}

/// Kubernetes integration helpers
pub mod kubernetes {
    use super::*;

    /// Template context for Kubernetes operations
    pub fn k8s_context() -> TemplateContextBuilder {
        TemplateContextBuilder::with_defaults()
            .var("namespace", std::env::var("K8S_NAMESPACE").unwrap_or_else(|_| "default".to_string()))
            .var("replicas", std::env::var("REPLICAS").unwrap_or_else(|_| "1".to_string()))
    }

    /// Render Kubernetes deployment from template
    ///
    /// # Arguments
    /// * `template` - Deployment template
    /// * `app_name` - Application name
    /// * `image` - Container image
    pub fn render_deployment(template: &str, app_name: &str, image: &str) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("app_name", app_name);
        vars.insert("image", image);

        render_to_format(template, vars, OutputFormat::Yaml)
    }

    /// Render Kubernetes service from template
    ///
    /// # Arguments
    /// * `template` - Service template
    /// * `service_name` - Service name
    /// * `port` - Service port
    pub fn render_service(template: &str, service_name: &str, port: u16) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("service_name", service_name);
        vars.insert("port", &port.to_string());

        render_to_format(template, vars, OutputFormat::Yaml)
    }
}

/// Testing integration helpers
pub mod testing {
    use super::*;

    /// Template context for testing scenarios
    pub fn testing_context() -> TemplateContextBuilder {
        TemplateContextBuilder::with_defaults()
            .var("test_timestamp", chrono::Utc::now().to_rfc3339())
            .var("test_id", uuid::Uuid::new_v4().to_string())
    }

    /// Render test configuration from template
    ///
    /// # Arguments
    /// * `template` - Test configuration template
    /// * `test_name` - Name of test
    pub fn render_test_config(template: &str, test_name: &str) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("test_name", test_name);
        vars.insert("test_id", &uuid::Uuid::new_v4().to_string());

        render_to_format(template, vars, OutputFormat::Toml)
    }

    /// Generate test data from template
    ///
    /// # Arguments
    /// * `template` - Test data template
    /// * `count` - Number of test data items to generate
    pub fn generate_test_data(template: &str, count: usize) -> Result<Vec<String>> {
        let mut results = Vec::new();

        for i in 0..count {
            let mut vars = HashMap::new();
            vars.insert("index", &i.to_string());
            vars.insert("id", &format!("test_{}", i));

            let result = render(template, vars)?;
            results.push(result);
        }

        Ok(results)
    }
}

/// Build system integration helpers
pub mod build {
    use super::*;

    /// Template context for build operations
    pub fn build_context() -> TemplateContextBuilder {
        TemplateContextBuilder::with_defaults()
            .var("build_time", chrono::Utc::now().to_rfc3339())
            .var("git_commit", std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string()))
            .var("build_number", std::env::var("BUILD_NUMBER").unwrap_or_else(|_| "1".to_string()))
    }

    /// Render build configuration from template
    ///
    /// # Arguments
    /// * `template` - Build configuration template
    /// * `project_name` - Project name
    pub fn render_build_config(template: &str, project_name: &str) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("project_name", project_name);
        vars.insert("build_time", chrono::Utc::now().to_rfc3339().as_str());

        render_to_format(template, vars, OutputFormat::Toml)
    }

    /// Generate CI/CD configuration from templates
    ///
    /// # Arguments
    /// * `template_dir` - Directory with CI/CD templates
    /// * `output_dir` - Directory to write generated configs
    /// * `project_name` - Project name
    pub fn generate_ci_configs(template_dir: &Path, output_dir: &Path, project_name: &str) -> Result<()> {
        use walkdir::WalkDir;

        std::fs::create_dir_all(output_dir)
            .map_err(|e| TemplateError::IoError(format!("Failed to create output directory: {}", e)))?;

        for entry in WalkDir::new(template_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let template_path = entry.path();
                let file_name = template_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                let output_path = output_dir.join(file_name);

                let template_content = std::fs::read_to_string(template_path)
                    .map_err(|e| TemplateError::IoError(format!("Failed to read template: {}", e)))?;

                let mut vars = HashMap::new();
                vars.insert("project_name", project_name);
                vars.insert("build_time", chrono::Utc::now().to_rfc3339().as_str());

                let rendered = render(&template_content, vars)?;

                std::fs::write(&output_path, rendered)
                    .map_err(|e| TemplateError::IoError(format!("Failed to write config: {}", e)))?;

                println!("Generated CI config: {}", output_path.display());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_context() {
        let context = web::web_context().build();
        assert!(context.vars.contains_key("timestamp"));
        assert!(context.vars.contains_key("environment"));
    }

    #[test]
    fn test_cli_command_creation() {
        let command = cli::template_command();
        let matches = command.try_get_matches_from(vec![
            "template", "test.toml", "-v", "name=value", "-f", "json"
        ]).unwrap();

        assert_eq!(matches.get_one::<String>("template").unwrap(), "test.toml");
        assert_eq!(matches.get_one::<String>("format").unwrap(), "json");
    }

    #[test]
    fn test_docker_context() {
        let context = docker::docker_context().build();
        assert!(context.vars.contains_key("docker_registry"));
        assert!(context.vars.contains_key("image_tag"));
    }

    #[test]
    fn test_database_context() {
        let context = database::database_context().build();
        assert!(context.vars.contains_key("database_url"));
        assert!(context.vars.contains_key("database_type"));
    }

    #[test]
    fn test_kubernetes_context() {
        let context = kubernetes::k8s_context().build();
        assert!(context.vars.contains_key("namespace"));
        assert!(context.vars.contains_key("replicas"));
    }

    #[test]
    fn test_testing_context() {
        let context = testing::testing_context().build();
        assert!(context.vars.contains_key("test_timestamp"));
        assert!(context.vars.contains_key("test_id"));
    }

    #[test]
    fn test_build_context() {
        let context = build::build_context().build();
        assert!(context.vars.contains_key("build_time"));
        assert!(context.vars.contains_key("git_commit"));
    }
}
