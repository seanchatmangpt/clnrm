# Policy: Reject test-related attributes in production environments
# Purpose: Prevent test/debug attributes from leaking into production telemetry
# Severity: VIOLATION (blocks deployment)

package live_check_advice

import rego.v1

# Deny attributes containing "test" prefix when in production
deny contains advice if {
    input.sample.attribute
    deployment_env := input.sample.resource_attributes["deployment.environment"]
    deployment_env == "production"

    # Check if attribute name contains "test"
    contains(input.sample.attribute.name, "test")

    # Exclude legitimate test.* attributes defined in registry
    not input.registry_attribute

    advice := {
        "type": "advice",
        "advice_type": "test_in_production",
        "advice_level": "violation",
        "message": sprintf(
            "Test attribute '%s' not allowed in production environment. Remove or rename this attribute.",
            [input.sample.attribute.name]
        ),
        "context": {
            "attribute_name": input.sample.attribute.name,
            "environment": deployment_env,
            "span_name": input.sample.span.name
        }
    }
}

# Warn about test-related values even in valid attributes
warn contains advice if {
    input.sample.attribute
    deployment_env := input.sample.resource_attributes["deployment.environment"]
    deployment_env == "production"

    # Check string values
    input.sample.attribute.type == "string"
    attr_value := input.sample.attribute.value

    # Look for test-related patterns in values
    test_patterns := ["test", "debug", "mock", "fake", "stub"]
    pattern := test_patterns[_]
    contains(lower(attr_value), pattern)

    advice := {
        "type": "advice",
        "advice_type": "test_value_in_production",
        "advice_level": "improvement",
        "message": sprintf(
            "Attribute '%s' contains test-related value '%s' in production. Consider using production-appropriate values.",
            [input.sample.attribute.name, pattern]
        ),
        "context": {
            "attribute_name": input.sample.attribute.name,
            "detected_pattern": pattern,
            "environment": deployment_env
        }
    }
}
