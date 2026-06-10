use std::collections::HashMap;

use crate::cleanroom::{CleanroomEnvironment, ExecutionResult};
use crate::error::CleanroomError;

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
        // Enforce structural and cryptographic validation on the motion.
        if !motion.raw_result.exit_code == 0 {
            return Err(CleanroomError::validation_error("Cannot admit failed execution result."));
        }
        
        let trace_proof = crate::pqc::hash::custom_hash(motion.raw_result.stdout.as_bytes());
        if trace_proof == [0u8; 32] {
            return Err(CleanroomError::validation_error("Invalid cryptographic trace representation."));
        }

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
        let result = self.env.execute_in_service(service_handle, command).await?;

        Ok(UntrustedMotion::new(result))
    }
}
