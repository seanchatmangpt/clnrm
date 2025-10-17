#!/usr/bin/env bash
# ==============================================================================
# ci-health-check.sh
# GitHub Actions Workflow Health Check and Failure Analysis
#
# This script monitors CI/CD pipeline health and provides detailed failure
# analysis for the Cleanroom Testing Framework.
#
# Usage:
#   ./scripts/ci-health-check.sh                    # Check current status
#   ./scripts/ci-health-check.sh --detailed         # Detailed analysis
#   ./scripts/ci-health-check.sh --fix-suggestions  # Suggest fixes
#   ./scripts/ci-health-check.sh --history 7        # Check last 7 days
#
# Exit Codes:
#   0 - All workflows healthy
#   1 - Some workflows failing
#   2 - Critical failures detected
# ==============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GITHUB_REPO="seanchatmangpt/clnrm"
GITHUB_API="https://api.github.com"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters
TOTAL_WORKFLOWS=0
HEALTHY_WORKFLOWS=0
FAILED_WORKFLOWS=0
WARNING_WORKFLOWS=0

# Log functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((HEALTHY_WORKFLOWS++))
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_WORKFLOWS++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    ((WARNING_WORKFLOWS++))
}

log_header() {
    echo -e "${PURPLE}[HEADER]${NC} $1"
}

# Check if GitHub CLI is available
check_gh_cli() {
    if ! command -v gh &> /dev/null; then
        log_error "GitHub CLI (gh) not found. Please install it first:"
        echo "  brew install gh"
        echo "  gh auth login"
        exit 1
    fi

    if ! gh auth status &> /dev/null; then
        log_error "GitHub CLI not authenticated. Please run:"
        echo "  gh auth login"
        exit 1
    fi
}

# Get workflow runs
get_workflow_runs() {
    local days=${1:-1}
    local limit=${2:-10}
    
    log_info "Fetching workflow runs from last $days days (limit: $limit)..."
    
    gh api "repos/$GITHUB_REPO/actions/runs" \
        --paginate \
        --jq ".workflow_runs[] | select(.created_at > \"$(date -u -d "$days days ago" +%Y-%m-%dT%H:%M:%SZ)\") | {id, name, status, conclusion, created_at, head_branch, html_url}" \
        | head -n "$limit"
}

# Analyze workflow failure
analyze_workflow_failure() {
    local run_id=$1
    local workflow_name=$2
    
    log_info "Analyzing failure for workflow: $workflow_name (run: $run_id)"
    
    # Get job details
    local jobs=$(gh api "repos/$GITHUB_REPO/actions/runs/$run_id/jobs" \
        --jq '.jobs[] | {name, conclusion, steps: [.steps[] | {name, conclusion, number}]}')
    
    echo "$jobs" | jq -r '
        .name as $job_name |
        .conclusion as $job_conclusion |
        if $job_conclusion == "failure" then
            "❌ Job: " + $job_name + " - FAILED",
            (.steps[] | select(.conclusion == "failure") | "   Step " + (.number|tostring) + ": " + .name + " - FAILED")
        elif $job_conclusion == "cancelled" then
            "⚠️  Job: " + $job_name + " - CANCELLED"
        else
            empty
        end
    '
}

# Check specific workflow health
check_workflow_health() {
    local workflow_name=$1
    local days=${2:-1}
    
    log_header "Checking workflow: $workflow_name"
    ((TOTAL_WORKFLOWS++))
    
    # Get recent runs for this workflow
    local runs=$(gh api "repos/$GITHUB_REPO/actions/workflows" \
        --jq ".workflows[] | select(.name == \"$workflow_name\") | .id" | head -1)
    
    if [[ -z "$runs" ]]; then
        log_warning "Workflow '$workflow_name' not found"
        return
    fi
    
    local workflow_id=$runs
    local recent_runs=$(gh api "repos/$GITHUB_REPO/actions/workflows/$workflow_id/runs" \
        --jq ".workflow_runs[0:3] | .[] | {id, status, conclusion, created_at}")
    
    local success_count=0
    local total_count=0
    
    echo "$recent_runs" | jq -r '
        .id as $id |
        .status as $status |
        .conclusion as $conclusion |
        .created_at as $created |
        if $status == "completed" then
            if $conclusion == "success" then
                "✅ " + ($created | split("T")[0]) + " - SUCCESS"
            elif $conclusion == "failure" then
                "❌ " + ($created | split("T")[0]) + " - FAILED"
            elif $conclusion == "cancelled" then
                "⚠️  " + ($created | split("T")[0]) + " - CANCELLED"
            else
                "❓ " + ($created | split("T")[0]) + " - " + $conclusion
            end
        else
            "🔄 " + ($created | split("T")[0]) + " - " + $status
        end
    ' | while read -r line; do
        echo "  $line"
        ((total_count++))
        if [[ "$line" == *"SUCCESS"* ]]; then
            ((success_count++))
        fi
    done
    
    # Calculate health score
    if [[ $total_count -gt 0 ]]; then
        local health_score=$((success_count * 100 / total_count))
        
        if [[ $health_score -ge 80 ]]; then
            log_success "Workflow health: $health_score% (recent runs)"
        elif [[ $health_score -ge 50 ]]; then
            log_warning "Workflow health: $health_score% (needs attention)"
        else
            log_error "Workflow health: $health_score% (critical issues)"
        fi
    fi
}

# Generate fix suggestions
generate_fix_suggestions() {
    log_header "🔧 Common Fix Suggestions"
    
    echo "Based on common CI failures in Rust projects:"
    echo ""
    echo "1. Compilation Errors:"
    echo "   - Run: cargo make clippy"
    echo "   - Check: cargo make fmt-check"
    echo "   - Fix: cargo make fix"
    echo ""
    echo "2. Test Failures:"
    echo "   - Run: cargo make test-verbose"
    echo "   - Check: cargo make test-integration"
    echo "   - Debug: RUST_BACKTRACE=1 cargo test"
    echo ""
    echo "3. Dependency Issues:"
    echo "   - Update: cargo make update-deps"
    echo "   - Audit: cargo make audit"
    echo "   - Check: cargo make check-deps"
    echo ""
    echo "4. Documentation Issues:"
    echo "   - Build: cargo make docs-build"
    echo "   - Validate: cargo make docs-validate"
    echo ""
    echo "5. Performance Issues:"
    echo "   - Benchmark: cargo make benchmark-performance"
    echo "   - Profile: cargo make system-metrics"
    echo ""
    echo "6. Best Practices:"
    echo "   - Validate: cargo make validate-best-practices"
    echo "   - Check: cargo make validate-production-readiness"
}

# Main function
main() {
    local detailed=false
    local fix_suggestions=false
    local history_days=1
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --detailed)
                detailed=true
                shift
                ;;
            --fix-suggestions)
                fix_suggestions=true
                shift
                ;;
            --history)
                history_days="$2"
                shift 2
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --detailed         Show detailed analysis"
                echo "  --fix-suggestions  Show common fix suggestions"
                echo "  --history DAYS     Check last N days (default: 1)"
                echo "  --help, -h         Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    log_header "🏥 Clnrm CI Health Check"
    echo "Repository: $GITHUB_REPO"
    echo "Checking last $history_days days"
    echo ""
    
    # Check prerequisites
    check_gh_cli
    
    # Check key workflows
    check_workflow_health "CI" "$history_days"
    check_workflow_health "Test" "$history_days"
    check_workflow_health "Build" "$history_days"
    check_workflow_health "Release" "$history_days"
    check_workflow_health "Documentation" "$history_days"
    
    echo ""
    log_header "📊 Health Summary"
    echo "Total Workflows Checked: $TOTAL_WORKFLOWS"
    echo "Healthy: $HEALTHY_WORKFLOWS"
    echo "Warnings: $WARNING_WORKFLOWS"
    echo "Failed: $FAILED_WORKFLOWS"
    
    if [[ $detailed == true ]]; then
        echo ""
        log_header "📋 Recent Workflow Runs"
        get_workflow_runs "$history_days" 10
    fi
    
    if [[ $fix_suggestions == true ]]; then
        echo ""
        generate_fix_suggestions
    fi
    
    # Determine exit code
    if [[ $FAILED_WORKFLOWS -gt 0 ]]; then
        exit 2
    elif [[ $WARNING_WORKFLOWS -gt 0 ]]; then
        exit 1
    else
        exit 0
    fi
}

# Run main function
main "$@"
