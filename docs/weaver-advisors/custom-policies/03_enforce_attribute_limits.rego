# Policy: Enforce attribute value size limits
# Purpose: Prevent oversized attribute values that impact performance
# Severity: IMPROVEMENT (should be optimized)

package live_check_advice

import rego.v1

# String attribute length limits
max_string_length := 256
warn_string_length := 128

# Array limits
max_array_elements := 100
warn_array_elements := 50

# Deny strings exceeding hard limit
deny contains advice if {
    input.sample.attribute
    input.sample.attribute.type == "string"
    attr_value := input.sample.attribute.value

    string_length := count(attr_value)
    string_length > max_string_length

    advice := {
        "type": "advice",
        "advice_type": "value_too_long",
        "advice_level": "violation",
        "message": sprintf(
            "String attribute '%s' exceeds %d character limit: %d characters. Truncate or store in logs instead.",
            [input.sample.attribute.name, max_string_length, string_length]
        ),
        "context": {
            "attribute_name": input.sample.attribute.name,
            "current_length": string_length,
            "max_length": max_string_length,
            "suggestion": "Store large values in structured logs, not span attributes"
        }
    }
}

# Warn about strings approaching limit
warn contains advice if {
    input.sample.attribute
    input.sample.attribute.type == "string"
    attr_value := input.sample.attribute.value

    string_length := count(attr_value)
    string_length > warn_string_length
    string_length <= max_string_length

    advice := {
        "type": "advice",
        "advice_type": "value_approaching_limit",
        "advice_level": "improvement",
        "message": sprintf(
            "String attribute '%s' is %d characters (warn threshold: %d). Consider shortening.",
            [input.sample.attribute.name, string_length, warn_string_length]
        ),
        "context": {
            "attribute_name": input.sample.attribute.name,
            "current_length": string_length,
            "warn_threshold": warn_string_length
        }
    }
}

# Check for arrays with too many elements (if supported in future)
# This is a placeholder for when array attributes are supported
warn contains advice if {
    input.sample.attribute
    input.sample.attribute.type == "array"
    array_value := input.sample.attribute.value

    element_count := count(array_value)
    element_count > warn_array_elements

    advice := {
        "type": "advice",
        "advice_type": "array_too_large",
        "advice_level": "improvement",
        "message": sprintf(
            "Array attribute '%s' has %d elements (warn threshold: %d). Consider pagination or sampling.",
            [input.sample.attribute.name, element_count, warn_array_elements]
        ),
        "context": {
            "attribute_name": input.sample.attribute.name,
            "element_count": element_count,
            "warn_threshold": warn_array_elements
        }
    }
}

# Warn about numeric precision that might cause cardinality issues
warn contains advice if {
    input.sample.attribute
    input.sample.attribute.type == "double"
    attr_name := input.sample.attribute.name

    # High-precision duration values should use histogram metrics instead
    contains(attr_name, "duration")

    advice := {
        "type": "advice",
        "advice_type": "high_cardinality_numeric",
        "advice_level": "improvement",
        "message": sprintf(
            "Duration attribute '%s' with high precision may cause cardinality issues. Consider using histogram metrics for duration measurements.",
            [attr_name]
        ),
        "context": {
            "attribute_name": attr_name,
            "suggestion": "Use histogram metrics (clnrm.test.duration) for duration measurements"
        }
    }
}
