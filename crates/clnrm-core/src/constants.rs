//! Authoritative constants and defaults for the clnrm framework.
//!
//! This module centralizes "magic numbers" and strings used across the codebase,
//! ensuring they are documented and can be made configurable in the future.

use std::time::Duration;

/// The default loopback address used for local service binding and communication.
pub const DEFAULT_LOOPBACK_ADDRESS: &str = "127.0.0.1";

/// The default OTLP gRPC port.
pub const DEFAULT_OTLP_GRPC_PORT: u16 = 4317;

/// The default OTLP HTTP port.
pub const DEFAULT_OTLP_HTTP_PORT: u16 = 4318;

/// The default port range start for service port allocation.
pub const DEFAULT_PORT_RANGE_START: u16 = 10000;

/// The default port range end for service port allocation.
pub const DEFAULT_PORT_RANGE_END: u16 = 20000;

/// Default timeout for flushing telemetry data on shutdown.
pub const DEFAULT_TELEMETRY_FLUSH_TIMEOUT: Duration = Duration::from_millis(500);

/// Default sleep duration during shutdown to allow async tasks to complete.
pub const DEFAULT_SHUTDOWN_SLEEP: Duration = Duration::from_millis(100);

/// Default interval for periodic metrics export.
pub const DEFAULT_METRICS_EXPORT_INTERVAL: Duration = Duration::from_secs(1);

/// Default timeout for waiting for Weaver to become ready.
pub const DEFAULT_WEAVER_READY_TIMEOUT: Duration = Duration::from_secs(10);

/// Default timeout for Weaver graceful shutdown.
pub const DEFAULT_WEAVER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// Default health check timeout for Weaver.
pub const DEFAULT_WEAVER_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(30);
