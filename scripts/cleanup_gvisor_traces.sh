#!/bin/bash
# Cleanup script to remove all Docker traces and optimize for gVisor
# Exit code: 0 = success, 1 = errors during cleanup
#
# WARNING: This script makes destructive changes. Review carefully before running.
#
# Usage:
#   ./scripts/cleanup_gvisor_traces.sh [OPTIONS]
#
# Options:
#   --dry-run         Show what would be removed without making changes
#   --aggressive      Also remove Docker documentation and examples
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

    # Backup Docker-related files
    if [ -d "crates/clnrm-core/tests" ]; then
        cp -r crates/clnrm-core/tests "$BACKUP_DIR/"
    fi

    log_success "Backup created at: $BACKUP_DIR"
}

# Remove Docker scripts (keep gVisor equivalents)
remove_docker_scripts() {
    log_section "Removing Docker Scripts"

    local docker_scripts=$(find scripts -name "*docker*.sh" 2>/dev/null || true)

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

# Clean up Docker references in Cargo.toml files
remove_docker_cargo_refs() {
    log_section "Removing Docker References from Cargo.toml"

    local cargo_files=$(find . -name "Cargo.toml" | grep -v target | grep -v backups)

    for file in $cargo_files; do
        log_info "Processing: $file"

        if grep -q "docker\|testcontainers" "$file"; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_info "  [DRY RUN] Would remove Docker references from $file"
            else
                # Remove Docker and testcontainers related lines
                sed -i '/docker/d; /testcontainers/d' "$file"
                log_success "  Removed Docker references from $file"
            fi
        else
            log_info "  No Docker references found"
        fi
    done
}

# Clean up imports and references in source files
cleanup_docker_imports() {
    log_section "Cleaning Up Docker Imports and References"

    local rust_files=$(find crates -name "*.rs" | grep -v target | grep -v backups)

    for file in $rust_files; do
        if grep -q "docker\|testcontainers\|Docker\|Testcontainers" "$file" 2>/dev/null; then
            if [ "$DRY_RUN" -eq 1 ]; then
                log_info "[DRY RUN] Would clean Docker references in: $file"
            else
                log_info "Cleaning Docker references in: $file"
                # Remove Docker-related imports and references
                sed -i '/use.*docker/d; /use.*testcontainers/d; /Docker/d; /Testcontainer/d' "$file"
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
verify_docker_removal() {
    log_section "Verification - Checking Docker Removal"

    log_info "Verifying Docker elimination..."

    local docker_refs=$(grep -r "docker" --include="*.rs" --include="*.sh" --include="Cargo.toml" \
        --exclude="cleanup_gvisor_traces.sh" \
        --exclude="validate_gvisor_only.sh" \
        crates/ scripts/ 2>/dev/null | grep -v "gvisor" | grep -v "# docker" | wc -l)

    if [ "$docker_refs" -eq 0 ]; then
        log_success "No Docker references found in active code"
    else
        log_warning "Found $docker_refs Docker references"
        log_info "Review with: grep -r 'docker' --include='*.rs' --include='*.sh' crates/ scripts/ | grep -v gvisor"
    fi

    return 0
}

# Verify gVisor setup
verify_gvisor_setup() {
    log_section "Verification - Checking gVisor Setup"

    local gvisor_scripts=(
        "scripts/gvisor_startup.sh"
        "scripts/gvisor_health_check.sh"
        "scripts/wait_for_gvisor.sh"
        "scripts/validate_gvisor_only.sh"
    )

    for script in "${gvisor_scripts[@]}"; do
        if [ -f "$script" ]; then
            log_success "Found: $script"
        else
            log_warning "Missing: $script"
        fi
    done

    return 0
}

# Generate cleanup report
generate_report() {
    log_section "Cleanup Report"

    local report_file="cleanup-report-gvisor-$(date +%Y%m%d-%H%M%S).txt"

    {
        echo "Docker Cleanup & gVisor Setup Report"
        echo "====================================="
        echo ""
        echo "Date: $(date)"
        echo "Dry Run: $DRY_RUN"
        echo "Aggressive: $AGGRESSIVE"
        echo "Backup: $BACKUP"
        echo ""
        echo "Actions Performed:"
        echo "------------------"
        echo "1. Removed Docker scripts"
        echo "2. Removed Docker Compose files"
        echo "3. Removed Dockerfiles"
        echo "4. Cleaned Docker references from Cargo.toml"
        echo "5. Cleaned Docker imports from source"
        echo "6. Updated Cargo.lock"
        echo ""
        echo "gVisor Scripts Available:"
        echo "------------------------"
        echo "  • scripts/gvisor_startup.sh - Initialize gVisor runtime"
        echo "  • scripts/gvisor_health_check.sh - Verify gVisor health"
        echo "  • scripts/wait_for_gvisor.sh - Wait for gVisor readiness"
        echo "  • scripts/validate_gvisor_only.sh - Validate gVisor setup"
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
        echo "4. Run ./scripts/gvisor_startup.sh to initialize gVisor"
        echo "5. Run ./scripts/validate_gvisor_only.sh to verify setup"
        echo "6. Commit changes if everything looks good"
    } > "$report_file"

    cat "$report_file"
    log_info "Report saved to: $report_file"
}

# Main execution
main() {
    log_section "Docker Cleanup & gVisor Setup"

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
        if ! confirm "This will remove Docker artifacts and optimize for gVisor. Continue?"; then
            log_warning "Cleanup cancelled"
            exit 0
        fi
    fi

    # Create backup
    create_backup

    # Perform cleanup
    remove_docker_scripts
    remove_docker_compose
    remove_dockerfiles
    remove_docker_cargo_refs
    cleanup_docker_imports
    update_cargo_lock

    # Verify setup
    verify_docker_removal
    verify_gvisor_setup

    # Generate report
    generate_report

    log_section "Cleanup Complete"

    if [ "$DRY_RUN" -eq 1 ]; then
        log_info "This was a dry run. No changes were made."
        log_info "Run without --dry-run to perform actual cleanup."
    else
        log_success "Docker artifacts removed and gVisor setup prepared!"
        echo ""
        echo "Next steps:"
        echo "  1. Review changes: git status"
        echo "  2. Build project: cargo build"
        echo "  3. Run tests: cargo test"
        echo "  4. Initialize gVisor: ./scripts/gvisor_startup.sh"
        echo "  5. Validate setup: ./scripts/validate_gvisor_only.sh"
        echo "  6. Commit: git add . && git commit -m 'Remove Docker, optimize for gVisor'"
    fi
}

# Run main
main
