//! clnrm-template - Template Engine for Cleanroom Testing Framework
//!
//! This crate provides Tera-based template rendering capabilities for test configuration files,
//! enabling dynamic test generation with custom functions and macro libraries.

pub mod error;
pub mod renderer;
pub mod context;
pub mod determinism;
pub mod functions;
pub mod discovery;
pub mod validation;
pub mod cache;
pub mod debug;
pub mod toml;
pub mod simple;
pub mod custom;
#[cfg(feature = "async")]
pub mod r#async;
pub mod builder;
pub mod integration;

pub use error::{TemplateError, Result};
pub use renderer::{TemplateRenderer, render_template, render_template_file, is_template, get_cached_template_renderer, OutputFormat};
pub use context::TemplateContext;
pub use determinism::DeterminismConfig;
pub use discovery::{TemplateDiscovery, TemplateLoader};
pub use validation::{TemplateValidator, ValidationRule, SchemaValidator};
pub use cache::{TemplateCache, CachedRenderer};
pub use debug::{TemplateDebugger, DebugInfo, TemplateAnalyzer};
pub use toml::{TomlFile, TomlLoader, TomlWriter, TomlMerger};
pub use simple::{render, render_file, render_with_context, render_with_json, render_to_format, TemplateBuilder, quick};
pub use custom::{CustomFunction, CustomFilter, FunctionRegistry, register_custom_function, register_custom_filter};
#[cfg(feature = "async")]
pub use r#async::{AsyncTemplateRenderer, async_render, async_render_file, async_render_with_json};
pub use builder::TemplateEngineBuilder;
pub use integration::{WebIntegration, CliIntegration, TemplateCli, TemplateServer};

/// Macro library content embedded at compile time
pub const MACRO_LIBRARY: &str = include_str!("_macros.toml.tera");