use std::collections::HashMap;

use crate::error::CleanroomError;
use crate::cleanroom::{CleanroomEnvironment, ExecutionResult};

/// A motion originating from an untrusted chamber.
///
/// This explicitly strips away authority from the underlying execution result.
/// An `UntrustedMotion` cannot be used directly; it MUST pass through the `AdmissionKernel`
/// to be elevated to an authoritative `ExecutionResult`.
#[derive(Debug, Clone)]
pub struct UntrustedMotion {
    raw_result: ExecutionResult,
}

impl UntrustedMotion {
    /// Create a new `UntrustedMotion`. Only accessible within this module hierarchy
    /// to ensure it can only be created by the chamber.
    pub(crate) fn new(raw_result: ExecutionResult) -> Self {
        Self { raw_result }
    }
}

/// The kernel responsible for elevating `UntrustedMotion` to authority.
///
/// The `AdmissionKernel` inspects motions from the untrusted chamber and applies
/// policy (e.g. signature verification, structural integrity checks) before
/// returning a trusted result.
#[derive(Debug, Default)]
pub struct AdmissionKernel;

impl AdmissionKernel {
    pub fn new() -> Self {
        Self
    }

    /// Admit an untrusted motion, elevating it to an authoritative `ExecutionResult`.
    ///
    /// In this implementation, the kernel acts as a boundary type. Future iterations
    /// can add cryptographic auditing or structural integrity constraints here.
    pub fn admit(&self, motion: UntrustedMotion) -> Result<ExecutionResult, CleanroomError> {
        // Here we could perform structural or cryptographic validation on the motion.
        // For now, it passes through the raw_result, formally elevating its status.
        Ok(motion.raw_result)
    }
}

/// The chamber wrapper that completely strips away authority.
///
/// It wraps a `CleanroomEnvironment` and ensures all outputs are `UntrustedMotion`.
#[derive(Debug)]
pub struct UntrustedChamber<'a> {
    env: &'a CleanroomEnvironment,
}

impl<'a> UntrustedChamber<'a> {
    /// Create a new untrusted chamber wrapping an existing cleanroom environment.
    pub fn new(env: &'a CleanroomEnvironment) -> Self {
        Self { env }
    }

    /// Execute a command in the untrusted container.
    ///
    /// The output is explicitly stripped of authority and typed as `UntrustedMotion`.
    pub async fn execute_in_container(
        &self,
        container_name: &str,
        command: &[String],
        workdir: Option<&str>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> Result<UntrustedMotion, CleanroomError> {
        let result = self
            .env
            .execute_in_container(container_name, command, workdir, env_vars)
            .await?;
        
        Ok(UntrustedMotion::new(result))
    }

    /// Execute a command in an untrusted service.
    ///
    /// The output is explicitly stripped of authority and typed as `UntrustedMotion`.
    pub async fn execute_in_service(
        &self,
        service_handle: &crate::cleanroom::ServiceHandle,
        command: &[String],
    ) -> Result<UntrustedMotion, CleanroomError> {
        let result = self
            .env
            .execute_in_service(service_handle, command)
            .await?;

        Ok(UntrustedMotion::new(result))
    }

    /// Execute a test logic closure in the untrusted environment.
    ///
    /// The output must be carefully handled, but since this closure might execute
    /// logic that returns anything, we map it into an `UntrustedMotion` if it returns
    /// an `ExecutionResult`.
    /// 
    /// Note: Providing an `execute_in_container` method is the most direct application
    /// of the wrapper.
}
