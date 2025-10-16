//! Fuzz target for error handling and propagation
//!
//! Tests error creation, chaining, and display for:
//! - Format string vulnerabilities
//! - Stack overflow in error chains
//! - Serialization issues
//! - Display/Debug trait panics

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use clnrm_core::error::{CleanroomError, ErrorKind};

#[derive(Debug, Arbitrary)]
struct FuzzError {
    kind: FuzzErrorKind,
    message: String,
    context: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Arbitrary)]
enum FuzzErrorKind {
    ContainerError,
    NetworkError,
    ResourceLimitExceeded,
    Timeout,
    ConfigurationError,
    PolicyViolation,
    ValidationError,
    ServiceError,
    InternalError,
}

fuzz_target!(|data: &[u8]| {
    let mut unstructured = Unstructured::new(data);

    if let Ok(fuzz_error) = FuzzError::arbitrary(&mut unstructured) {
        // Convert fuzz error kind to actual error kind
        let kind = match fuzz_error.kind {
            FuzzErrorKind::ContainerError => ErrorKind::ContainerError,
            FuzzErrorKind::NetworkError => ErrorKind::NetworkError,
            FuzzErrorKind::ResourceLimitExceeded => ErrorKind::ResourceLimitExceeded,
            FuzzErrorKind::Timeout => ErrorKind::Timeout,
            FuzzErrorKind::ConfigurationError => ErrorKind::ConfigurationError,
            FuzzErrorKind::PolicyViolation => ErrorKind::PolicyViolation,
            FuzzErrorKind::ValidationError => ErrorKind::ValidationError,
            FuzzErrorKind::ServiceError => ErrorKind::ServiceError,
            FuzzErrorKind::InternalError => ErrorKind::InternalError,
        };

        // Create error with various message formats
        let mut error = CleanroomError::new(kind, &fuzz_error.message);

        // Test error chaining
        if let Some(context) = &fuzz_error.context {
            error = error.with_context(context);
        }

        if let Some(source) = &fuzz_error.source {
            error = error.with_source(source);
        }

        // Test Display trait (format string vulnerabilities)
        let _ = format!("{}", error);
        let _ = error.to_string();

        // Test Debug trait
        let _ = format!("{:?}", error);

        // Test serialization
        let _ = serde_json::to_string(&error);

        // Test helper methods
        let _ = CleanroomError::container_error(&fuzz_error.message);
        let _ = CleanroomError::network_error(&fuzz_error.message);
        let _ = CleanroomError::timeout_error(&fuzz_error.message);
        let _ = CleanroomError::validation_error(&fuzz_error.message);

        // Test error with context chaining
        if let Some(ctx) = &fuzz_error.context {
            if let Some(src) = &fuzz_error.source {
                let chained = CleanroomError::new(kind, &fuzz_error.message)
                    .with_context(ctx)
                    .with_source(src);

                let _ = format!("{}", chained);
                let _ = serde_json::to_string(&chained);
            }
        }
    }
});
