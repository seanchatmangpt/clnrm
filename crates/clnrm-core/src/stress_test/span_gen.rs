//! OTEL span stress generator
//!
//! Generates configurable OTEL span hierarchies for stress testing.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, span, Level};

/// Configuration for span generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanConfig {
    /// Maximum depth of span nesting
    pub max_depth: usize,

    /// Number of spans per level
    pub spans_per_level: usize,

    /// Add attributes to spans
    pub add_attributes: bool,

    /// Number of attributes per span
    pub attributes_per_span: usize,

    /// Add events to spans
    pub add_events: bool,

    /// Number of events per span
    pub events_per_span: usize,
}

impl Default for SpanConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            spans_per_level: 1,
            add_attributes: true,
            attributes_per_span: 5,
            add_events: true,
            events_per_span: 2,
        }
    }
}

/// Stress testing profile for span generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStressProfile {
    /// Light load: shallow, few spans
    Light,
    /// Medium load: moderate depth and span count
    Medium,
    /// Heavy load: deep nesting, many spans
    Heavy,
    /// Extreme load: maximum depth and span count
    Extreme,
}

impl SpanStressProfile {
    /// Get span configuration for this profile
    pub fn config(&self) -> SpanConfig {
        match self {
            Self::Light => SpanConfig {
                max_depth: 2,
                spans_per_level: 2,
                add_attributes: true,
                attributes_per_span: 3,
                add_events: false,
                events_per_span: 0,
            },
            Self::Medium => SpanConfig {
                max_depth: 5,
                spans_per_level: 3,
                add_attributes: true,
                attributes_per_span: 5,
                add_events: true,
                events_per_span: 2,
            },
            Self::Heavy => SpanConfig {
                max_depth: 10,
                spans_per_level: 5,
                add_attributes: true,
                attributes_per_span: 10,
                add_events: true,
                events_per_span: 5,
            },
            Self::Extreme => SpanConfig {
                max_depth: 20,
                spans_per_level: 10,
                add_attributes: true,
                attributes_per_span: 20,
                add_events: true,
                events_per_span: 10,
            },
        }
    }
}

/// OTEL span generator for stress testing
#[derive(Debug)]
pub struct SpanGenerator {
    config: SpanConfig,
    spans_generated: usize,
}

impl SpanGenerator {
    /// Create a new span generator
    pub fn new(config: SpanConfig) -> Self {
        Self {
            config,
            spans_generated: 0,
        }
    }

    /// Create from stress profile
    pub fn from_profile(profile: SpanStressProfile) -> Self {
        Self::new(profile.config())
    }

    /// Generate stress test spans
    ///
    /// Creates a hierarchy of spans with configurable depth and attributes
    pub fn generate(&mut self, test_name: &str, container: &str) -> Result<SpanGenerationStats> {
        let start_time = Instant::now();
        self.spans_generated = 0;

        // Generate nested spans using tracing macros instead of OpenTelemetry directly
        // This is simpler and avoids type issues with BoxedTracer vs SdkTracer
        let _root = span!(
            Level::INFO,
            "stress_test",
            test.name = %test_name,
            container.image = %container,
            stress.max_depth = self.config.max_depth
        );
        let _guard = _root.enter();

        self.spans_generated += 1;

        // Generate nested spans recursively
        self.generate_nested_tracing(test_name, 0)?;

        let duration = start_time.elapsed();

        Ok(SpanGenerationStats {
            total_spans: self.spans_generated,
            max_depth: self.config.max_depth,
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Generate nested spans recursively using tracing
    fn generate_nested_tracing(&mut self, test_name: &str, current_depth: usize) -> Result<()> {
        if current_depth >= self.config.max_depth {
            return Ok(());
        }

        for i in 0..self.config.spans_per_level {
            let _span = span!(
                Level::DEBUG,
                "stress.level",
                depth = current_depth,
                index = i,
                test.name = %test_name
            );
            let _guard = _span.enter();

            // Add debug events
            if self.config.add_events {
                for j in 0..self.config.events_per_span {
                    debug!(event.index = j, "Generated stress event");
                }
            }

            self.spans_generated += 1;

            // Recurse to next level
            self.generate_nested_tracing(test_name, current_depth + 1)?;
        }

        Ok(())
    }

    /// Get total spans that will be generated (without actually generating)
    pub fn estimate_span_count(&self) -> usize {
        let mut total = 1; // root span

        for depth in 0..self.config.max_depth {
            total += self.config.spans_per_level.pow(depth as u32 + 1);
        }

        total
    }

    /// Reset generator state
    pub fn reset(&mut self) {
        self.spans_generated = 0;
    }
}

/// Statistics from span generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanGenerationStats {
    /// Total number of spans generated
    pub total_spans: usize,

    /// Maximum depth reached
    pub max_depth: usize,

    /// Time taken to generate spans (ms)
    pub duration_ms: u64,
}

/// Generate simple stress spans using tracing macros
///
/// Alternative to SpanGenerator for simpler use cases
pub fn generate_stress_spans_simple(test_id: &str, depth: usize) {
    let _root = span!(Level::INFO, "stress_test", test.id = %test_id, depth = depth);

    for level in 0..depth {
        let _level_span = span!(
            Level::DEBUG,
            "stress_level",
            level = level,
            test.id = %test_id
        );

        debug!("Generated stress span at level {}", level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_estimation() {
        let config = SpanConfig {
            max_depth: 3,
            spans_per_level: 2,
            add_attributes: false,
            attributes_per_span: 0,
            add_events: false,
            events_per_span: 0,
        };

        let generator = SpanGenerator::new(config);
        let estimate = generator.estimate_span_count();

        // Root: 1
        // Level 0: 2^1 = 2
        // Level 1: 2^2 = 4
        // Level 2: 2^3 = 8
        // Total: 1 + 2 + 4 + 8 = 15
        assert_eq!(estimate, 15);
    }

    #[test]
    fn test_stress_profiles() {
        let light = SpanStressProfile::Light.config();
        let extreme = SpanStressProfile::Extreme.config();

        assert!(light.max_depth < extreme.max_depth);
        assert!(light.spans_per_level < extreme.spans_per_level);
    }
}
