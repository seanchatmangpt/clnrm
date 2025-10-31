#!/bin/bash
# OTLP Configuration Script
# Sets up OpenTelemetry environment variables for Weaver validation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
OTLP_PORT=${OTLP_PORT:-4317}
OTLP_PROTOCOL=${OTLP_PROTOCOL:-grpc}  # grpc or http
SERVICE_NAME=${SERVICE_NAME:-clnrm}
SERVICE_VERSION=${SERVICE_VERSION:-1.2.0}
DEPLOYMENT_ENV=${DEPLOYMENT_ENV:-testing}

# Determine endpoint based on protocol
if [[ "$OTLP_PROTOCOL" == "grpc" ]]; then
    OTLP_ENDPOINT="http://localhost:${OTLP_PORT}"
else
    OTLP_ENDPOINT="http://localhost:${OTLP_PORT}/v1/traces"
fi

# ========== FUNCTIONS ==========

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Export OTLP environment variables
export_otlp_vars() {
    # Core OTLP endpoint
    export OTEL_EXPORTER_OTLP_ENDPOINT="$OTLP_ENDPOINT"

    # Service information
    export OTEL_SERVICE_NAME="$SERVICE_NAME"
    export OTEL_SERVICE_VERSION="$SERVICE_VERSION"

    # Resource attributes
    export OTEL_RESOURCE_ATTRIBUTES="service.name=${SERVICE_NAME},service.version=${SERVICE_VERSION},deployment.environment=${DEPLOYMENT_ENV}"

    # Protocol configuration
    export OTEL_EXPORTER_OTLP_PROTOCOL="$OTLP_PROTOCOL"

    # Signal-specific configuration
    export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="$OTLP_ENDPOINT"
    export OTEL_EXPORTER_OTLP_METRICS_ENDPOINT="$OTLP_ENDPOINT"
    export OTEL_EXPORTER_OTLP_LOGS_ENDPOINT="$OTLP_ENDPOINT"

    # Batch processing (optimize for testing)
    export OTEL_BSP_SCHEDULE_DELAY=1000  # 1 second (faster than default 5s)
    export OTEL_BSP_MAX_QUEUE_SIZE=2048
    export OTEL_BSP_MAX_EXPORT_BATCH_SIZE=512

    # Sampling (100% for validation)
    export OTEL_TRACES_SAMPLER=always_on

    # Logging (useful for debugging)
    export RUST_LOG=${RUST_LOG:-info}

    log_success "OTLP environment variables exported"
}

# Validate configuration
validate_config() {
    local errors=0

    # Check if port is a number
    if ! [[ "$OTLP_PORT" =~ ^[0-9]+$ ]]; then
        log_error "Invalid port: $OTLP_PORT (must be a number)"
        errors=$((errors + 1))
    fi

    # Check protocol is valid
    if [[ "$OTLP_PROTOCOL" != "grpc" && "$OTLP_PROTOCOL" != "http" ]]; then
        log_error "Invalid protocol: $OTLP_PROTOCOL (must be 'grpc' or 'http')"
        errors=$((errors + 1))
    fi

    # Check service name is not empty
    if [[ -z "$SERVICE_NAME" ]]; then
        log_error "Service name cannot be empty"
        errors=$((errors + 1))
    fi

    if [[ $errors -gt 0 ]]; then
        log_error "Configuration validation failed with $errors errors"
        return 1
    fi

    log_success "Configuration validated"
    return 0
}

# Print configuration summary
print_config() {
    echo ""
    echo "OTLP Configuration:"
    echo "  Endpoint:     $OTEL_EXPORTER_OTLP_ENDPOINT"
    echo "  Protocol:     $OTEL_EXPORTER_OTLP_PROTOCOL"
    echo "  Service Name: $OTEL_SERVICE_NAME"
    echo "  Version:      $OTEL_SERVICE_VERSION"
    echo "  Environment:  $DEPLOYMENT_ENV"
    echo "  Batch Delay:  ${OTEL_BSP_SCHEDULE_DELAY}ms"
    echo "  Sampler:      $OTEL_TRACES_SAMPLER"
    echo "  Log Level:    $RUST_LOG"
    echo ""
}

# Test OTLP endpoint connectivity
test_endpoint() {
    log_info "Testing OTLP endpoint connectivity..."

    # Check if port is listening
    if command -v lsof >/dev/null 2>&1; then
        if lsof -i :$OTLP_PORT >/dev/null 2>&1; then
            log_success "Port $OTLP_PORT is listening"
            return 0
        else
            log_warning "Port $OTLP_PORT is not listening"
            log_info "Weaver must be started before running tests"
            return 1
        fi
    elif command -v netstat >/dev/null 2>&1; then
        if netstat -an | grep -q ":$OTLP_PORT "; then
            log_success "Port $OTLP_PORT is listening"
            return 0
        else
            log_warning "Port $OTLP_PORT is not listening"
            log_info "Weaver must be started before running tests"
            return 1
        fi
    else
        log_warning "Cannot verify port (lsof/netstat not available)"
        return 1
    fi
}

# Generate shell export script
generate_export_script() {
    local output_file="${1:-/tmp/otlp_env.sh}"

    cat > "$output_file" << EOF
#!/bin/bash
# OTLP Environment Variables
# Generated: $(date)
# Source this file: source $output_file

export OTEL_EXPORTER_OTLP_ENDPOINT="$OTLP_ENDPOINT"
export OTEL_SERVICE_NAME="$SERVICE_NAME"
export OTEL_SERVICE_VERSION="$SERVICE_VERSION"
export OTEL_RESOURCE_ATTRIBUTES="service.name=${SERVICE_NAME},service.version=${SERVICE_VERSION},deployment.environment=${DEPLOYMENT_ENV}"
export OTEL_EXPORTER_OTLP_PROTOCOL="$OTLP_PROTOCOL"
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="$OTLP_ENDPOINT"
export OTEL_EXPORTER_OTLP_METRICS_ENDPOINT="$OTLP_ENDPOINT"
export OTEL_EXPORTER_OTLP_LOGS_ENDPOINT="$OTLP_ENDPOINT"
export OTEL_BSP_SCHEDULE_DELAY=1000
export OTEL_BSP_MAX_QUEUE_SIZE=2048
export OTEL_BSP_MAX_EXPORT_BATCH_SIZE=512
export OTEL_TRACES_SAMPLER=always_on
export RUST_LOG=${RUST_LOG:-info}

echo "OTLP environment configured:"
echo "  Endpoint: \$OTEL_EXPORTER_OTLP_ENDPOINT"
echo "  Service:  \$OTEL_SERVICE_NAME"
EOF

    chmod +x "$output_file"
    log_success "Export script generated: $output_file"
    log_info "Source with: source $output_file"
}

# ========== MAIN LOGIC ==========

main() {
    local mode="${1:-export}"  # export, validate, test, generate

    echo "================================================================================"
    echo "OTLP Configuration - clnrm v1.2.0"
    echo "================================================================================"
    echo ""

    # Parse command line arguments
    case "$mode" in
        export)
            # Export variables and print config
            if ! validate_config; then
                exit 1
            fi
            export_otlp_vars
            print_config
            log_success "Variables exported to current shell"
            echo ""
            log_info "To use in a new shell, run:"
            echo "  source <($0 generate)"
            echo ""
            ;;

        validate)
            # Only validate without exporting
            log_info "Validating configuration..."
            if ! validate_config; then
                exit 1
            fi
            print_config
            ;;

        test)
            # Export and test connectivity
            if ! validate_config; then
                exit 1
            fi
            export_otlp_vars
            print_config
            test_endpoint
            ;;

        generate)
            # Generate export script
            if ! validate_config; then
                exit 1
            fi
            export_otlp_vars
            generate_export_script "/tmp/otlp_env.sh"
            ;;

        help|--help|-h)
            echo "Usage: $0 [MODE]"
            echo ""
            echo "Modes:"
            echo "  export    Export OTLP variables to current shell (default)"
            echo "  validate  Validate configuration without exporting"
            echo "  test      Export and test endpoint connectivity"
            echo "  generate  Generate a shell export script"
            echo ""
            echo "Environment Variables:"
            echo "  OTLP_PORT          OTLP port (default: 4317)"
            echo "  OTLP_PROTOCOL      Protocol: grpc or http (default: grpc)"
            echo "  SERVICE_NAME       Service name (default: clnrm)"
            echo "  SERVICE_VERSION    Service version (default: 1.2.0)"
            echo "  DEPLOYMENT_ENV     Environment (default: testing)"
            echo "  RUST_LOG           Rust log level (default: info)"
            echo ""
            echo "Examples:"
            echo "  # Export with defaults"
            echo "  source $0"
            echo ""
            echo "  # Export with custom port"
            echo "  OTLP_PORT=5317 source $0"
            echo ""
            echo "  # Generate script for later use"
            echo "  $0 generate"
            echo "  source /tmp/otlp_env.sh"
            echo ""
            echo "  # Test connectivity"
            echo "  $0 test"
            echo ""
            exit 0
            ;;

        *)
            log_error "Unknown mode: $mode"
            echo ""
            echo "Run '$0 help' for usage information"
            exit 1
            ;;
    esac

    echo "================================================================================"
}

# ========== ENTRY POINT ==========

main "$@"
