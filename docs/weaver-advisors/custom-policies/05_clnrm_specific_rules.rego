# Policy: CLNRM-specific validation rules
# Purpose: Enforce clnrm framework requirements and detect false positives
# Severity: VIOLATION (breaks clnrm guarantees)

package live_check_advice

import rego.v1

# Rule: test.isolated must always be true for clnrm
deny contains advice if {
    input.sample.span
    input.sample.span.name == "clnrm.test_execution"

    # Check if test.isolated attribute exists and is false
    attr := input.sample.attributes[_]
    attr.name == "test.isolated"
    attr.value == false

    advice := {
        "type": "advice",
        "advice_type": "isolation_violation",
        "advice_level": "violation",
        "message": "test.isolated must be true. Cleanroom framework requires hermetic isolation.",
        "context": {
            "span_name": input.sample.span.name,
            "attribute_name": "test.isolated",
            "expected_value": true,
            "actual_value": false,
            "impact": "Breaks core cleanroom guarantee of test isolation"
        }
    }
}

# Rule: container.id must exist for test_execution spans
deny contains advice if {
    input.sample.span
    input.sample.span.name == "clnrm.test_execution"

    # Check if container.id is missing
    not has_container_id

    advice := {
        "type": "advice",
        "advice_type": "missing_container_id",
        "advice_level": "violation",
        "message": "container.id is required for test_execution spans. This proves a container actually ran.",
        "context": {
            "span_name": input.sample.span.name,
            "missing_attribute": "container.id",
            "impact": "Cannot prove test ran in a container - possible stub implementation"
        }
    }
}

has_container_id if {
    attr := input.sample.attributes[_]
    attr.name == "container.id"
}

# Rule: test.duration_ms must be positive
deny contains advice if {
    input.sample.span
    input.sample.span.name == "clnrm.test_execution"

    attr := input.sample.attributes[_]
    attr.name == "test.duration_ms"

    # Duration is zero or negative
    attr.value <= 0

    advice := {
        "type": "advice",
        "advice_type": "invalid_duration",
        "advice_level": "violation",
        "message": sprintf(
            "test.duration_ms must be positive, got: %v. Zero duration suggests stub implementation.",
            [attr.value]
        ),
        "context": {
            "attribute_name": "test.duration_ms",
            "current_value": attr.value,
            "impact": "Zero/negative duration proves no actual execution occurred"
        }
    }
}

# Rule: test.cleanup_performed must be true
deny contains advice if {
    input.sample.span
    input.sample.span.name == "clnrm.test_execution"

    attr := input.sample.attributes[_]
    attr.name == "test.cleanup_performed"
    attr.value == false

    advice := {
        "type": "advice",
        "advice_type": "cleanup_not_performed",
        "advice_level": "violation",
        "message": "test.cleanup_performed must be true. Resource cleanup is required.",
        "context": {
            "attribute_name": "test.cleanup_performed",
            "expected_value": true,
            "actual_value": false,
            "impact": "Resource leak detected - containers not cleaned up"
        }
    }
}

# Rule: Warn about very short durations (might indicate stub)
warn contains advice if {
    input.sample.span
    input.sample.span.name == "clnrm.test_execution"

    attr := input.sample.attributes[_]
    attr.name == "test.duration_ms"

    # Duration is suspiciously short (< 1ms)
    attr.value < 1.0
    attr.value > 0

    advice := {
        "type": "advice",
        "advice_type": "suspiciously_short_duration",
        "advice_level": "improvement",
        "message": sprintf(
            "test.duration_ms is suspiciously short: %v ms. Verify this is not a stub implementation.",
            [attr.value]
        ),
        "context": {
            "attribute_name": "test.duration_ms",
            "current_value": attr.value,
            "warning": "Container creation typically takes > 1ms"
        }
    }
}

# Rule: container.image.name should not be 'fake' or 'test'
warn contains advice if {
    input.sample.span
    input.sample.span.name == "clnrm.test_execution"

    attr := input.sample.attributes[_]
    attr.name == "container.image.name"

    image_lower := lower(attr.value)
    test_patterns := ["fake", "test", "mock", "stub"]
    pattern := test_patterns[_]
    contains(image_lower, pattern)

    advice := {
        "type": "advice",
        "advice_type": "test_image_name",
        "advice_level": "improvement",
        "message": sprintf(
            "container.image.name '%s' contains test pattern '%s'. Use real container images.",
            [attr.value, pattern]
        ),
        "context": {
            "attribute_name": "container.image.name",
            "detected_pattern": pattern,
            "suggestion": "Use official images like alpine:latest, postgres:15, etc."
        }
    }
}

# Rule: Detect incomplete plugin lifecycle (state must transition properly)
warn contains advice if {
    input.sample.span
    input.sample.span.name == "clnrm.plugin_execution"

    attr := input.sample.attributes[_]
    attr.name == "plugin.state"

    # Final state must be running or stopped, not initializing
    final_states := ["running", "stopped", "failed"]
    not attr.value in final_states

    advice := {
        "type": "advice",
        "advice_type": "incomplete_plugin_lifecycle",
        "advice_level": "improvement",
        "message": sprintf(
            "plugin.state is '%s' which is not a terminal state. Plugin lifecycle may be incomplete.",
            [attr.value]
        ),
        "context": {
            "attribute_name": "plugin.state",
            "current_state": attr.value,
            "expected_terminal_states": final_states
        }
    }
}
