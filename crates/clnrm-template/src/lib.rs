//! clnrm-template - Template Engine for Cleanroom Testing Framework
//!
//! This crate provides Tera-based template rendering capabilities for test configuration files,
//! enabling dynamic test generation with custom functions and macro libraries.

#[cfg(feature = "async")]
pub mod r#async;
pub mod builder;
pub mod cache;
pub mod context;
pub mod custom;
pub mod debug;
pub mod determinism;
pub mod discovery;
pub mod error;
pub mod functions;
pub mod renderer;
pub mod simple;
pub mod toml;
pub mod validation;
// TODO: Re-enable integration module after adding actix_web dependency
// pub mod integration;

pub use builder::TemplateEngineBuilder;
pub use cache::{CachedRenderer, TemplateCache};
pub use context::TemplateContext;
pub use custom::{
    register_custom_filter, register_custom_function, CustomFilter, CustomFunction,
    FunctionRegistry,
};
pub use debug::{DebugInfo, TemplateAnalyzer, TemplateDebugger};
pub use determinism::DeterminismConfig;
pub use discovery::{TemplateDiscovery, TemplateLoader};
pub use error::{Result, TemplateError};
#[cfg(feature = "async")]
pub use r#async::{async_render, async_render_file, async_render_with_json, AsyncTemplateRenderer};
pub use renderer::{
    get_cached_template_renderer, is_template, render_template, render_template_file, OutputFormat,
    TemplateRenderer,
};
pub use simple::{
    quick, render, render_file, render_to_format, render_with_context, render_with_json,
    TemplateBuilder,
};
pub use toml::{TomlFile, TomlLoader, TomlMerger, TomlWriter};
pub use validation::{SchemaValidator, TemplateValidator, ValidationRule};
// TODO: Re-enable integration exports after adding actix_web dependency
// pub use integration::{WebIntegration, CliIntegration, TemplateCli, TemplateServer};

/// Macro library content embedded at compile time
pub const MACRO_LIBRARY: &str = include_str!("_macros.toml.tera");
