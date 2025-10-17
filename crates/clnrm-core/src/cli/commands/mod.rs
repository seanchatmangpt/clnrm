//! CLI commands module
//!
//! Exports all CLI command implementations with their associated functionality.

pub mod health;
pub mod init;
pub mod plugins;
pub mod report;
pub mod run;
pub mod self_test;
pub mod services;
pub mod template;
pub mod validate;

// Re-export all public functions for easy access
pub use run::{
    run_single_test, run_tests, run_tests_parallel, run_tests_parallel_with_results,
    run_tests_sequential, run_tests_sequential_with_results, validate_test_assertions,
    watch_and_run,
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
