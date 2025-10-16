#!/usr/bin/env bash

#############################################################################
# SurrealDB Test Environment Setup Script
#############################################################################
# This script prepares the environment for SurrealDB integration testing
# by verifying prerequisites, pulling images, and setting up test data.
#
# Usage: ./setup.sh [--skip-pull] [--verbose]
#############################################################################

set -euo pipefail

# Configuration
SURREALDB_IMAGE="surrealdb/surrealdb:latest"
SURREALDB_VERSION="v1.5.4"
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${TEST_DIR}/../.." && pwd)"
SKIP_PULL=false
VERBOSE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

#############################################################################
# Utility Functions
#############################################################################

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "\n${BLUE}==>${NC} $1"
}

verbose_log() {
    if [ "$VERBOSE" = true ]; then
        echo -e "${BLUE}[VERBOSE]${NC} $1"
    fi
}

#############################################################################
# Parse Arguments
#############################################################################

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-pull)
                SKIP_PULL=true
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                cat << EOF
SurrealDB Test Environment Setup

Usage: ./setup.sh [OPTIONS]

Options:
    --skip-pull     Skip Docker image pull (use cached image)
    --verbose, -v   Enable verbose output
    --help, -h      Show this help message

Examples:
    ./setup.sh                  # Full setup with image pull
    ./setup.sh --skip-pull      # Use cached image
    ./setup.sh --verbose        # Detailed output

EOF
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
}

#############################################################################
# Prerequisite Checks
#############################################################################

check_docker() {
    log_step "Checking Docker availability"

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        echo "Please install Docker from: https://docs.docker.com/get-docker/"
        exit 1
    fi

    verbose_log "Docker command found: $(command -v docker)"

    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        echo ""
        echo "Please start Docker:"
        echo "  - macOS/Windows: Start Docker Desktop"
        echo "  - Linux: sudo systemctl start docker"
        exit 1
    fi

    local docker_version
    docker_version=$(docker --version | awk '{print $3}' | sed 's/,//')
    log_success "Docker is running (version: ${docker_version})"
}

check_rust() {
    log_step "Checking Rust toolchain"

    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo is not installed"
        echo "Please install Rust from: https://rustup.rs/"
        exit 1
    fi

    local rust_version
    rust_version=$(rustc --version | awk '{print $2}')
    log_success "Rust toolchain available (version: ${rust_version})"
}

check_network() {
    log_step "Checking network connectivity"

    if ! curl -s --max-time 5 https://hub.docker.com > /dev/null; then
        log_warning "Cannot reach Docker Hub (network issue or firewall)"
        log_warning "Image pull may fail if image not cached locally"
    else
        verbose_log "Network connectivity to Docker Hub verified"
        log_success "Network connectivity OK"
    fi
}

#############################################################################
# Docker Image Management
#############################################################################

pull_surrealdb_image() {
    if [ "$SKIP_PULL" = true ]; then
        log_info "Skipping image pull (--skip-pull flag set)"
        return 0
    fi

    log_step "Pulling SurrealDB Docker image"
    log_info "Image: ${SURREALDB_IMAGE}"

    # Check if image already exists
    if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${SURREALDB_IMAGE}$"; then
        log_info "Image already exists locally"
        verbose_log "$(docker images ${SURREALDB_IMAGE} --format 'Size: {{.Size}}, Created: {{.CreatedSince}}')"
    fi

    # Pull image (updates if newer version available)
    if docker pull "${SURREALDB_IMAGE}"; then
        local image_size
        image_size=$(docker images "${SURREALDB_IMAGE}" --format "{{.Size}}")
        log_success "Successfully pulled SurrealDB image (${image_size})"
    else
        log_error "Failed to pull SurrealDB image"
        echo "Attempting to continue with cached image..."

        if ! docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${SURREALDB_IMAGE}$"; then
            log_error "No cached image available"
            exit 1
        fi
    fi
}

verify_image() {
    log_step "Verifying SurrealDB image"

    if ! docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${SURREALDB_IMAGE}$"; then
        log_error "SurrealDB image not found"
        exit 1
    fi

    # Test run container
    verbose_log "Running test container to verify image..."
    local container_id
    container_id=$(docker run -d --rm \
        -p 8000 \
        "${SURREALDB_IMAGE}" \
        start --user root --pass root 2>&1)

    if [ $? -eq 0 ]; then
        verbose_log "Test container started: ${container_id:0:12}"
        sleep 2
        docker stop "${container_id}" &> /dev/null || true
        log_success "Image verification successful"
    else
        log_warning "Could not verify image with test container"
    fi
}

#############################################################################
# Test Environment Setup
#############################################################################

create_test_directories() {
    log_step "Creating test directories"

    local dirs=(
        "${TEST_DIR}/data"
        "${TEST_DIR}/configs"
        "${TEST_DIR}/outputs"
    )

    for dir in "${dirs[@]}"; do
        if [ ! -d "$dir" ]; then
            mkdir -p "$dir"
            verbose_log "Created directory: $dir"
        else
            verbose_log "Directory exists: $dir"
        fi
    done

    log_success "Test directories ready"
}

create_sample_data_file() {
    log_step "Creating sample data file"

    local data_file="${TEST_DIR}/test-data.sql"

    if [ -f "$data_file" ]; then
        log_info "Sample data file already exists: ${data_file}"
        return 0
    fi

    cat > "$data_file" << 'EOF'
-- SurrealDB Test Data
-- This file contains sample data for integration testing

-- Define namespace and database
USE NS test;
USE DB test;

-- Create tables with schema definitions
DEFINE TABLE users SCHEMAFULL;
DEFINE FIELD username ON users TYPE string;
DEFINE FIELD email ON users TYPE string;
DEFINE FIELD age ON users TYPE int;
DEFINE FIELD created_at ON users TYPE datetime DEFAULT time::now();

DEFINE TABLE products SCHEMAFULL;
DEFINE FIELD name ON products TYPE string;
DEFINE FIELD price ON products TYPE decimal;
DEFINE FIELD category ON products TYPE string;
DEFINE FIELD in_stock ON products TYPE bool DEFAULT true;

DEFINE TABLE orders SCHEMAFULL;
DEFINE FIELD user_id ON orders TYPE record(users);
DEFINE FIELD product_id ON orders TYPE record(products);
DEFINE FIELD quantity ON orders TYPE int;
DEFINE FIELD order_date ON orders TYPE datetime DEFAULT time::now();

-- Insert sample users
INSERT INTO users (username, email, age) VALUES
    ('alice', 'alice@example.com', 30),
    ('bob', 'bob@example.com', 25),
    ('charlie', 'charlie@example.com', 35);

-- Insert sample products
INSERT INTO products (name, price, category, in_stock) VALUES
    ('Laptop', 999.99, 'Electronics', true),
    ('Mouse', 29.99, 'Electronics', true),
    ('Desk', 199.99, 'Furniture', true),
    ('Chair', 149.99, 'Furniture', false);

-- Insert sample orders (references to users and products)
INSERT INTO orders (user_id, product_id, quantity) VALUES
    (users:alice, products:laptop, 1),
    (users:bob, products:mouse, 2),
    (users:alice, products:desk, 1);

-- Sample queries to verify functionality

-- Query 1: Get all users
SELECT * FROM users;

-- Query 2: Get users older than 28
SELECT * FROM users WHERE age > 28;

-- Query 3: Get all in-stock products
SELECT * FROM products WHERE in_stock = true;

-- Query 4: Get orders with user and product details (JOIN)
SELECT
    order_date,
    user_id.username AS user_name,
    product_id.name AS product_name,
    quantity
FROM orders;

-- Query 5: Count products by category
SELECT
    category,
    count() AS total_products
FROM products
GROUP BY category;
EOF

    log_success "Sample data file created: ${data_file}"
}

install_surrealdb_cli() {
    log_step "Checking SurrealDB CLI"

    if command -v surreal &> /dev/null; then
        local cli_version
        cli_version=$(surreal version | head -n1 | awk '{print $2}')
        log_success "SurrealDB CLI already installed (version: ${cli_version})"
        return 0
    fi

    log_info "SurrealDB CLI not found, installing..."

    # Determine OS and architecture
    local os arch install_cmd
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$arch" in
        x86_64)
            arch="amd64"
            ;;
        aarch64|arm64)
            arch="arm64"
            ;;
        *)
            log_warning "Unsupported architecture: $arch"
            log_info "Skipping CLI installation (not required for tests)"
            return 0
            ;;
    esac

    case "$os" in
        darwin)
            if command -v brew &> /dev/null; then
                log_info "Installing via Homebrew..."
                brew install surrealdb/tap/surreal
            else
                log_info "Installing via curl..."
                curl -sSf https://install.surrealdb.com | sh
            fi
            ;;
        linux)
            log_info "Installing via curl..."
            curl -sSf https://install.surrealdb.com | sh
            ;;
        *)
            log_warning "Unsupported OS: $os"
            log_info "Skipping CLI installation (not required for tests)"
            return 0
            ;;
    esac

    if command -v surreal &> /dev/null; then
        log_success "SurrealDB CLI installed successfully"
    else
        log_warning "CLI installation may have failed (check PATH)"
        log_info "CLI is optional for running tests"
    fi
}

#############################################################################
# Connectivity Tests
#############################################################################

run_connectivity_test() {
    log_step "Running basic connectivity test"

    log_info "Starting test SurrealDB container..."

    local container_id
    container_id=$(docker run -d --rm \
        -p 8000:8000 \
        --name surrealdb_test_$$ \
        "${SURREALDB_IMAGE}" \
        start --user root --pass root 2>&1)

    if [ $? -ne 0 ]; then
        log_error "Failed to start test container"
        return 1
    fi

    verbose_log "Container ID: ${container_id:0:12}"
    log_info "Waiting for SurrealDB to be ready..."

    # Wait for service to be ready (max 30 seconds)
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if curl -s -f http://localhost:8000/health &> /dev/null; then
            log_success "SurrealDB is responding"
            break
        fi

        attempt=$((attempt + 1))
        if [ $attempt -eq $max_attempts ]; then
            log_error "Timeout waiting for SurrealDB to respond"
            docker logs "surrealdb_test_$$" 2>&1 | tail -n 20
            docker stop "surrealdb_test_$$" &> /dev/null || true
            return 1
        fi

        sleep 1
    done

    # Test query execution
    log_info "Testing query execution..."

    local query_result
    query_result=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -H "Accept: application/json" \
        -u root:root \
        -d '{"sql": "SELECT * FROM users"}' \
        http://localhost:8000/sql 2>&1)

    if [ $? -eq 0 ]; then
        verbose_log "Query result: $query_result"
        log_success "Query execution successful"
    else
        log_warning "Query execution test failed (container may need more time)"
    fi

    # Cleanup
    log_info "Cleaning up test container..."
    docker stop "surrealdb_test_$$" &> /dev/null || true

    log_success "Connectivity test completed"
}

#############################################################################
# Cleanup Functions
#############################################################################

cleanup_old_containers() {
    log_step "Cleaning up old test containers"

    local old_containers
    old_containers=$(docker ps -a --filter "name=surrealdb_test" --format "{{.ID}}" 2>/dev/null)

    if [ -n "$old_containers" ]; then
        verbose_log "Found old containers: $old_containers"
        echo "$old_containers" | xargs -r docker rm -f &> /dev/null || true
        log_success "Cleaned up old containers"
    else
        verbose_log "No old containers to clean up"
        log_info "No old test containers found"
    fi
}

#############################################################################
# Main Setup Flow
#############################################################################

print_banner() {
    cat << 'EOF'
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   SurrealDB Test Environment Setup                       ║
║   Cleanroom Testing Framework                            ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
    echo ""
}

print_summary() {
    echo ""
    log_step "Setup Summary"
    echo ""
    echo "  Test Directory:   ${TEST_DIR}"
    echo "  Project Root:     ${PROJECT_ROOT}"
    echo "  Docker Image:     ${SURREALDB_IMAGE}"
    echo "  Sample Data:      ${TEST_DIR}/test-data.sql"
    echo ""
    log_success "Environment setup completed successfully!"
    echo ""
    echo "Next steps:"
    echo "  1. Review test documentation:  cat README.md"
    echo "  2. Run all tests:              cargo test --test surrealdb_integration"
    echo "  3. Run specific test:          cargo test --test surrealdb_integration test_name"
    echo "  4. Use TOML configs:           cargo run -- run tests/surrealdb/"
    echo ""
}

main() {
    print_banner

    parse_args "$@"

    # Prerequisites
    check_docker
    check_rust
    check_network

    # Docker setup
    cleanup_old_containers
    pull_surrealdb_image
    verify_image

    # Test environment
    create_test_directories
    create_sample_data_file
    install_surrealdb_cli

    # Validation
    run_connectivity_test

    # Summary
    print_summary
}

#############################################################################
# Script Entry Point
#############################################################################

main "$@"
