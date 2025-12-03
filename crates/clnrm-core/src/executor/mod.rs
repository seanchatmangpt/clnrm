//! Test Executor
//!
//! Executes tests against containers using the canonical Config format.
//! Uses `docker exec` to run commands in RUNNING containers (not new containers).

pub mod container_manager;
pub mod runner;

pub use container_manager::{ContainerHandle, ContainerManager, DockerContainerManager};
pub use runner::{ExecutionResult, StepResult, TestRunner};
