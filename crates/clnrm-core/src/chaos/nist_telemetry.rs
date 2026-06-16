use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::cleanroom::CleanroomEnvironment;
use async_trait::async_trait;
use std::time::{Duration, SystemTime};

// ─── Span Dropper ─────────────────────────────────────────────────────────

/// Probabilistically drops OTEL spans to simulate telemetry loss.
pub struct SpanDropper;

impl SpanDropper {
    /// Return `true` with probability `drop_rate` (0.0 = never, 1.0 = always).
    pub fn should_drop(drop_rate: f64) -> bool {
        let r: f64 = rand::random();
        r < drop_rate
    }
}

// ─── Attribute Corruptor ──────────────────────────────────────────────────

/// Corrupts string attributes in telemetry records.
pub struct AttributeCorruptor;

impl AttributeCorruptor {
    /// Return a modified copy of `value` with a few characters randomly altered.
    ///
    /// If `value` is empty the empty string is returned unchanged.
    pub fn corrupt_string(value: &str) -> String {
        if value.is_empty() {
            return String::new();
        }
        let mut bytes: Vec<u8> = value.as_bytes().to_vec();
        // Corrupt ~20 % of characters (at least 1)
        let corrupt_count = ((bytes.len() as f64 * 0.2).ceil() as usize).max(1);
        for _ in 0..corrupt_count {
            let idx = (rand::random::<u64>() as usize) % bytes.len();
            // Replace with a random printable ASCII character.
            bytes[idx] = 33 + (rand::random::<u8>() % 94); // '!' .. '~'
        }
        // Lossy conversion is fine for chaos testing.
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

// ─── Timestamp Manipulator ────────────────────────────────────────────────

/// Shifts OTEL timestamps to simulate clock-skew or replay attacks.
pub struct TimestampManipulator;

impl TimestampManipulator {
    /// Add `shift_ms` milliseconds to `ts` (negative values subtract time).
    ///
    /// Saturates at [`SystemTime::UNIX_EPOCH`] when the shift would go below zero.
    pub fn shift_timestamp(ts: SystemTime, shift_ms: i64) -> SystemTime {
        if shift_ms >= 0 {
            ts + Duration::from_millis(shift_ms as u64)
        } else {
            let delta = Duration::from_millis((-shift_ms) as u64);
            ts.checked_sub(delta).unwrap_or(SystemTime::UNIX_EPOCH)
        }
    }
}

/// Simulates a telemetry spoofing attack attempting to bypass container boundaries
/// and inject forged OTLP traces directly into the host's validation port
/// to masquerade as a successful test run.
pub struct TelemetrySpoofingAttack {
    container_name: String,
    validation_port: u16,
}

impl TelemetrySpoofingAttack {
    /// Creates a new TelemetrySpoofingAttack targeting the specified validation port.
    pub fn new(container_name: &str, validation_port: u16) -> Self {
        Self {
            container_name: container_name.to_string(),
            validation_port,
        }
    }
}

#[async_trait]
impl NistAttackVector for TelemetrySpoofingAttack {
    async fn execute(
        &self,
        env: &CleanroomEnvironment,
    ) -> Result<AttackResult, crate::error::CleanroomError> {
        let payload = r#"{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"forged-service"}}]},"scopeSpans":[{"spans":[{"name":"forged-test-pass","spanId":"0000000000000001","traceId":"00000000000000000000000000000001"}]}]}]}"#;

        let command = vec![
            "curl".to_string(),
            "-s".to_string(),
            "-X".to_string(),
            "POST".to_string(),
            format!(
                "http://host.docker.internal:{}/v1/traces",
                self.validation_port
            ),
            "-H".to_string(),
            "Content-Type: application/json".to_string(),
            "-d".to_string(),
            payload.to_string(),
        ];

        match env
            .execute_in_container(&self.container_name, &command, None, None)
            .await
        {
            Ok(result) => {
                // If the command fails, the network boundary blocked the injection.
                if result.exit_code != 0
                    || result.stderr.contains("Connection refused")
                    || result.stderr.contains("Could not resolve host")
                    || result.stderr.contains("Failed to connect")
                {
                    Ok(AttackResult::Blocked)
                } else {
                    // The injection succeeded, meaning the telemetry spoofing was successful.
                    Ok(AttackResult::Success)
                }
            }
            Err(_) => {
                // Execution failed at the environment level, which acts as a block.
                Ok(AttackResult::Blocked)
            }
        }
    }
}
