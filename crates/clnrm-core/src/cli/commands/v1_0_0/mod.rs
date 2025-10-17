//! PRD v1.0 Commands Module
//!
//! Contains implementations for PRD v1.0 additional commands:
//! - pull: Pre-pull Docker images
//! - graph: Visualize OTEL traces
//! - repro: Reproduce baseline runs
//! - redgreen: TDD workflow validation
//! - render: Template rendering with variables
//! - spans: OTEL span filtering
//! - collector: Local OTEL collector management

pub mod collector;
pub mod graph;
pub mod pull;
pub mod render;
pub mod repro;
pub mod spans;
pub mod tdd;

// Re-export all public functions
pub use collector::{show_collector_logs, show_collector_status, start_collector, stop_collector};
pub use graph::visualize_graph;
pub use pull::pull_images;
pub use render::render_template_with_vars;
pub use repro::reproduce_baseline;
pub use spans::filter_spans;
pub use tdd::run_red_green_validation;
