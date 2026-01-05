#!/bin/bash
# Cleanup script to remove all Docker and testcontainers traces
# Exit code: 0 = success, 1 = errors during cleanup
#
# WARNING: This script makes destructive changes. Review carefully before running.
#
# Usage:
#   ./scripts/cleanup_docker_traces.sh [OPTIONS]
#
# Options:
#   --dry-run         Show what would be removed without making changes
#   --aggressive      Also remove Docker-related documentation and examples
#   --backup          Create backup before removal
#   --force           Skip confirmation prompts
#
# Environment:
#   BACKUP_DIR        Backup directory (default: ./backups)

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
DRY_RUN=0
AGGRESSIVE=0
BACKUP=0
FORCE=0
BACKUP_DIR=${BACKUP_DIR:-./backups/$(date +%Y%m%d-%H%M%S)}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --aggressive)
            AGGRESSIVE=1
            shift
            ;;
        --backup)
            BACKUP=1
            shift
            ;;
        --force)
            FORCE=1
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

log_section() {
    echo ""
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}================================================${NC}"
    echo ""
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_info() {
    echo "$1"
}

# Confirm action
confirm() {
    if [ "$FORCE" -eq 1 ]; then
        return 0
    fi

    local message=$1
    echo -e "${YELLOW}$message (y/n)${NC}"
    read -r response
    [[ "$response" =~ ^[Yy]$ ]]
}

# Create backup
create_backup() {
    if [ "$BACKUP" -eq 0 ]; then
        return
    fi

    log_section "Creating Backup"

    mkdir -p "$BACKUP_DIR"

    log_info "Backing up files to: $BACKUP_DIR"

    # Backup Cargo.toml files
    find . -name "Cargo.toml" -exec cp --parents {} "$BACKUP_DIR" \;

    # Backup backend files
    if [ -d "crates/clnrm-core/src/backend" ]; then
        cp -r crates/clnrm-core/src/backend "$BACKUP_DIR/"
    fi

    # Backup test files
    if [ -d "crates/clnrm-core/tests" ]; then
        cp -r crates/clnrm-core/tests "$BACKUP_DIR/"
    fi

    log_success "Backup created at: $BACKUP_DIR"
}

# Remove testcontainers dependencies from Cargo.toml
remove_cargo_dependencies() {
    log_section "Removing Testcontainers Dependencies"

    local cargo_files=$(find . -name "Cargo.toml" | grep -v target | grep -v backups)

    for file in $cargo_files; do
        log_info "Processing: $file"

        if grep -q "testcontainers" "$file"; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_info "  [DRY RUN] Would remove testcontainers from $file"
            else
                # Remove testcontainers lines
                sed -i '/testcontainers/d' "$file"
                log_success "  Removed testcontainers from $file"
            fi
        else
            log_info "  No testcontainers dependencies found"
        fi
    done
}

# Remove TestcontainerBackend files
remove_testcontainer_backend() {
    log_section "Removing TestcontainerBackend Implementation"

    local backend_file="crates/clnrm-core/src/backend/testcontainer.rs"

    if [ -f "$backend_file" ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            log_info "[DRY RUN] Would remove: $backend_file"
        else
            if confirm "Remove TestcontainerBackend file ($backend_file)?"; then
                rm "$backend_file"
                log_success "Removed: $backend_file"
            else
                log_info "Skipped: $backend_file"
            fi
        fi
    else
        log_info "TestcontainerBackend file not found (already removed)"
    fi

    # Update mod.rs to remove testcontainer module
    local mod_file="crates/clnrm-core/src/backend/mod.rs"

    if [ -f "$mod_file" ]; then
        if grep -q "pub mod testcontainer" "$mod_file"; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_info "[DRY RUN] Would remove testcontainer module from $mod_file"
            else
                sed -i '/pub mod testcontainer/d' "$mod_file"
                sed -i '/pub use testcontainer/d' "$mod_file"
                sed -i '/use testcontainer/d' "$mod_file"
                sed -i '/TestcontainerBackend/d' "$mod_file"
                log_success "Removed testcontainer references from $mod_file"
            fi
        fi
    fi
}

# Remove Docker-related test files
remove_docker_tests() {
    log_section "Removing Docker-Related Tests"

    # Find Docker test files
    local docker_tests=$(find . -name "*docker*.rs" -o -name "*testcontainer*.rs" | grep -v target | grep -v backups)

    for test_file in $docker_tests; do
        # Skip if in phase4_e2e_docker (legacy tests)
        if [[ "$test_file" == *"phase4_e2e_docker"* ]]; then
            if [ "$AGGRESSIVE" -eq 1 ]; then
                if [ "$DRY_RUN" -eq 1 ]; then
                    log_info "[DRY RUN] Would remove: $test_file"
                else
                    if confirm "Remove Docker test file ($test_file)?"; then
                        rm "$test_file"
                        log_success "Removed: $test_file"
                    fi
                fi
            else
                log_info "Skipping (use --aggressive to remove): $test_file"
            fi
        fi
    done

    # Remove phase4_e2e_docker directory if aggressive
    if [ "$AGGRESSIVE" -eq 1 ]; then
        local docker_test_dir="crates/clnrm-core/tests/weaver/phase4_e2e_docker"
        if [ -d "$docker_test_dir" ]; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_info "[DRY RUN] Would remove directory: $docker_test_dir"
            else
                if confirm "Remove Docker test directory ($docker_test_dir)?"; then
                    rm -rf "$docker_test_dir"
                    log_success "Removed directory: $docker_test_dir"
                fi
            fi
        fi
    fi
}

# Remove Docker scripts
remove_docker_scripts() {
    log_section "Removing Docker Scripts"

    local docker_scripts=$(find scripts -name "*docker*.sh" 2>/dev/null | grep -v validate_docker_elimination.sh || true)

    for script in $docker_scripts; do
        if [ "$DRY_RUN" -eq 1 ]; then
            log_info "[DRY RUN] Would remove: $script"
        else
            if confirm "Remove Docker script ($script)?"; then
                rm "$script"
                log_success "Removed: $script"
            else
                log_info "Skipped: $script"
            fi
        fi
    done
}

# Remove Docker Compose files
remove_docker_compose() {
    log_section "Removing Docker Compose Files"

    local compose_files=$(find . -name "docker-compose*.yml" -o -name "docker-compose*.yaml" | grep -v target | grep -v backups)

    for compose in $compose_files; do
        if [ "$AGGRESSIVE" -eq 1 ]; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_info "[DRY RUN] Would remove: $compose"
            else
                if confirm "Remove Docker Compose file ($compose)?"; then
                    rm "$compose"
                    log_success "Removed: $compose"
                fi
            fi
        else
            log_info "Skipping (use --aggressive to remove): $compose"
        fi
    done
}

# Remove Dockerfiles
remove_dockerfiles() {
    log_section "Removing Dockerfiles"

    local dockerfiles=$(find . -name "Dockerfile*" | grep -v target | grep -v backups)

    for dockerfile in $dockerfiles; do
        if [ "$AGGRESSIVE" -eq 1 ]; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_info "[DRY RUN] Would remove: $dockerfile"
            else
                if confirm "Remove Dockerfile ($dockerfile)?"; then
                    rm "$dockerfile"
                    log_success "Removed: $dockerfile"
                fi
            fi
        else
            log_info "Skipping (use --aggressive to remove): $dockerfile"
        fi
    done
}

# Clean up imports and references
cleanup_imports() {
    log_section "Cleaning Up Imports and References"

    local rust_files=$(find crates -name "*.rs" | grep -v target | grep -v backups)

    for file in $rust_files; do
        if grep -q "use testcontainers" "$file" || grep -q "testcontainers::" "$file"; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_info "[DRY RUN] Would clean imports in: $file"
            else
                log_info "Cleaning imports in: $file"
                # Remove testcontainers imports
                sed -i '/use testcontainers/d' "$file"
                # This is a simple cleanup - manual review may be needed
                log_warning "Manual review recommended for: $file"
            fi
        fi
    done
}

# Update Cargo.lock
update_cargo_lock() {
    log_section "Updating Cargo.lock"

    if [ "$DRY_RUN" -eq 0 ]; then
        log_info "Running cargo update to refresh dependencies..."
        cargo update
        log_success "Cargo.lock updated"
    else
        log_info "[DRY RUN] Would run: cargo update"
    fi
}

# Verification
verify_cleanup() {
    log_section "Verification"

    log_info "Running Docker elimination validation..."

    if [ "$DRY_RUN" -eq 0 ]; then
        if ./scripts/validate_docker_elimination.sh; then
            log_success "Verification passed - Docker completely eliminated!"
        else
            log_warning "Verification found some remaining Docker references"
            log_info "Manual cleanup may be required"
        fi
    else
        log_info "[DRY RUN] Would run: ./scripts/validate_docker_elimination.sh"
    fi
}

# Generate cleanup report
generate_report() {
    log_section "Cleanup Report"

    local report_file="cleanup-report-$(date +%Y%m%d-%H%M%S).txt"

    {
        echo "Docker/Testcontainers Cleanup Report"
        echo "======================================"
        echo ""
        echo "Date: $(date)"
        echo "Dry Run: $DRY_RUN"
        echo "Aggressive: $AGGRESSIVE"
        echo "Backup: $BACKUP"
        echo ""
        echo "Actions Performed:"
        echo "------------------"
        echo "1. Removed testcontainers dependencies from Cargo.toml"
        echo "2. Removed TestcontainerBackend implementation"
        echo "3. Removed Docker-related test files"
        echo "4. Removed Docker scripts"
        if [ "$AGGRESSIVE" -eq 1 ]; then
            echo "5. Removed Docker Compose files"
            echo "6. Removed Dockerfiles"
        fi
        echo ""
        echo "Backup Location:"
        echo "----------------"
        if [ "$BACKUP" -eq 1 ]; then
            echo "$BACKUP_DIR"
        else
            echo "No backup created"
        fi
        echo ""
        echo "Next Steps:"
        echo "-----------"
        echo "1. Review and test changes"
        echo "2. Run cargo build to verify compilation"
        echo "3. Run cargo test to verify tests pass"
        echo "4. Run ./scripts/validate_docker_elimination.sh"
        echo "5. Commit changes if everything looks good"
    } > "$report_file"

    cat "$report_file"
    log_info "Report saved to: $report_file"
}

# Main execution
main() {
    log_section "Docker/Testcontainers Cleanup"

    if [ "$DRY_RUN" -eq 1 ]; then
        log_warning "DRY RUN MODE - No changes will be made"
    fi

    echo "Configuration:"
    echo "  Dry Run: $DRY_RUN"
    echo "  Aggressive: $AGGRESSIVE"
    echo "  Backup: $BACKUP"
    echo "  Force: $FORCE"
    echo ""

    if [ "$DRY_RUN" -eq 0 ] && [ "$FORCE" -eq 0 ]; then
        if ! confirm "This will make destructive changes. Continue?"; then
            log_warning "Cleanup cancelled"
            exit 0
        fi
    fi

    # Create backup
    create_backup

    # Perform cleanup
    remove_cargo_dependencies
    remove_testcontainer_backend
    remove_docker_tests
    remove_docker_scripts
    remove_docker_compose
    remove_dockerfiles
    cleanup_imports
    update_cargo_lock

    # Verify cleanup
    verify_cleanup

    # Generate report
    generate_report

    log_section "Cleanup Complete"

    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "This was a dry run. No changes were made."
        log_info "Run without --dry-run to perform actual cleanup."
    else
        log_success "Docker and testcontainers traces removed!"
        echo ""
        echo "Next steps:"
        echo "  1. Review changes: git status"
        echo "  2. Build project: cargo build"
        echo "  3. Run tests: cargo test"
        echo "  4. Validate: ./scripts/validate_docker_elimination.sh"
        echo "  5. Commit: git add . && git commit -m 'Remove Docker/testcontainers dependencies'"
    fi
}

# Run main
main
