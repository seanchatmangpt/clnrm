# Policy: Require 'clnrm.' prefix for custom attributes
# Purpose: Enforce namespace conventions for custom attributes
# Severity: VIOLATION (must be fixed)

package live_check_advice

import rego.v1

# List of allowed standard prefixes (OTel semantic conventions)
allowed_prefixes := [
    "service.",
    "deployment.",
    "http.",
    "db.",
    "messaging.",
    "rpc.",
    "container.",
    "k8s.",
    "cloud.",
    "test.",      # OTel test conventions
    "error.",     # OTel error conventions
    "exception."  # OTel exception conventions
]

# Check if attribute has an allowed prefix
has_allowed_prefix(attr_name) if {
    prefix := allowed_prefixes[_]
    startswith(attr_name, prefix)
}

# Require clnrm prefix for custom attributes not in registry
deny contains advice if {
    input.sample.attribute
    attr_name := input.sample.attribute.name

    # Not in registry (custom attribute)
    not input.registry_attribute

    # Doesn't have standard OTel prefix
    not has_allowed_prefix(attr_name)

    # Doesn't have clnrm prefix
    not startswith(attr_name, "clnrm.")

    advice := {
        "type": "advice",
        "advice_type": "missing_namespace_prefix",
        "advice_level": "violation",
        "message": sprintf(
            "Custom attribute '%s' must start with 'clnrm.' prefix. Use 'clnrm.%s' instead.",
            [attr_name, attr_name]
        ),
        "context": {
            "attribute_name": attr_name,
            "suggested_name": sprintf("clnrm.%s", [attr_name]),
            "reason": "Custom attributes must be namespaced to avoid conflicts"
        }
    }
}

# Warn about underscore naming (should use dot notation)
warn contains advice if {
    input.sample.attribute
    attr_name := input.sample.attribute.name

    # Has underscore in name
    contains(attr_name, "_")

    # Not a standard OTel attribute pattern
    not has_allowed_prefix(attr_name)

    advice := {
        "type": "advice",
        "advice_type": "underscore_naming",
        "advice_level": "improvement",
        "message": sprintf(
            "Attribute '%s' uses underscore naming. Consider dot notation (e.g., 'clnrm.%s') for consistency with OTel conventions.",
            [attr_name, replace(attr_name, "_", ".")]
        ),
        "context": {
            "attribute_name": attr_name,
            "suggested_name": sprintf("clnrm.%s", [replace(attr_name, "_", ".")])
        }
    }
}
