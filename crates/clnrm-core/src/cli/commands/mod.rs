//! CLI commands module
//!
//! Exports all CLI command implementations with their associated functionality.

pub mod analyze;
pub mod collector;
pub mod collector_noun_verb;
pub mod dev;
pub mod diff;
pub mod dry_run;
pub mod fmt;
pub mod graph;
pub mod health;
pub mod init;
pub mod lint;
pub mod live_check;
pub mod plugins;
pub mod prd_commands;
pub mod pull;
pub mod record;
pub mod redgreen;
pub mod redgreen_impl;
pub mod render;
pub mod report;
pub mod repro;
pub mod run;
pub mod self_test;
pub mod services;
pub mod services_noun_verb;
pub mod spans;
pub mod template;
pub mod validate;

// Re-export all public functions for easy access
pub use run::{
    run_tests, run_tests_parallel, run_tests_parallel_with_results, run_tests_sequential,
    run_tests_sequential_with_results, run_tests_with_shard,
};

pub use init::init_project;
pub use template::{
    generate_deterministic_template, generate_from_template, generate_full_validation_template,
    generate_lifecycle_matcher, generate_macro_library, generate_matrix_template,
    generate_otel_template,
};

pub use validate::{validate_config, validate_single_config};

pub use plugins::list_plugins;

pub use services::{ai_manage, restart_service, show_service_logs, show_service_status};

pub use report::{display_test_results, generate_framework_report, generate_report};

pub use self_test::run_self_tests;

pub use health::system_health_check;

// Re-export live-check commands
pub use live_check::{show_modes, show_status, show_version, test_weaver, validate_registry};

// Re-export commands
pub use analyze::analyze_traces;
pub use collector::{show_collector_logs, show_collector_status, start_collector, stop_collector};
pub use dev::{run_dev_mode, run_dev_mode_with_filters};
pub use diff::diff_traces;
pub use dry_run::{dry_run_validate, ValidationResult as DryRunValidationResult};
pub use fmt::format_files;
pub use graph::visualize_graph;
pub use lint::lint_files;
pub use pull::pull_images;
pub use record::run_record;
pub use redgreen::run_red_green_validation;
pub use render::render_template_with_vars;
pub use repro::reproduce_baseline;
pub use spans::filter_spans;
