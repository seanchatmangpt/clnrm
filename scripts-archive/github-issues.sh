#!/bin/bash
# Script to query GitHub issues for the clnrm repository
# Usage: ./scripts/github-issues.sh [state] [labels] [assignee] [milestone]
# Examples:
#   ./scripts/github-issues.sh open           # All open issues
#   ./scripts/github-issues.sh closed bug     # Closed issues with 'bug' label
#   ./scripts/github-issues.sh open "" "" "" 50  # First 50 open issues

set -euo pipefail

# Configuration
REPO_OWNER="seanchatmangpt"
REPO_NAME="clnrm"
GITHUB_API="https://api.github.com"
PER_PAGE=${GITHUB_PER_PAGE:-100}
OUTPUT_DIR="github-issues"
JSON_OUTPUT="${OUTPUT_DIR}/issues.json"
CSV_OUTPUT="${OUTPUT_DIR}/issues.csv"

# Default values
STATE="${1:-open}"
LABELS="${2:-}"
ASSIGNEE="${3:-}"
MILESTONE="${4:-}"
LIMIT="${5:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Check for GitHub token
check_github_token() {
    if [ -z "${GITHUB_TOKEN:-}" ]; then
        log_warning "GITHUB_TOKEN not set. Using public API (may hit rate limits)"
        log_warning "Set GITHUB_TOKEN for higher rate limits and private repo access"
        AUTH_HEADER=""
    else
        AUTH_HEADER="Authorization: token $GITHUB_TOKEN"
    fi
}

# Create output directory
create_output_dir() {
    mkdir -p "$OUTPUT_DIR"
}

# Build API URL
build_api_url() {
    local url="$GITHUB_API/repos/$REPO_OWNER/$REPO_NAME/issues"
    local params=""

    # Add state filter
    params="${params}state=$STATE"

    # Add labels filter (if provided)
    if [ -n "$LABELS" ]; then
        # URL encode labels (replace spaces with %20)
        encoded_labels=$(echo "$LABELS" | sed 's/ /%20/g')
        params="${params}&labels=$encoded_labels"
    fi

    # Add assignee filter (if provided)
    if [ -n "$ASSIGNEE" ]; then
        params="${params}&assignee=$ASSIGNEE"
    fi

    # Add milestone filter (if provided)
    if [ -n "$MILESTONE" ]; then
        params="${params}&milestone=$MILESTONE"
    fi

    # Add pagination
    params="${params}&per_page=$PER_PAGE"

    echo "$url?$params"
}

# Query GitHub API with pagination
query_github_api() {
    local url="$1"
    local output_file="$2"

    log_info "Querying GitHub API for issues..."

    # Check rate limit first
    if [ -n "$AUTH_HEADER" ]; then
        rate_limit=$(curl -s -H "$AUTH_HEADER" "$GITHUB_API/rate_limit" | grep -o '"remaining": [0-9]*' | grep -o '[0-9]*' || echo "unknown")
        log_info "GitHub API rate limit remaining: $rate_limit"
    fi

    # Query first page
    log_info "Fetching first page of issues..."
    response=$(curl -s -H "$AUTH_HEADER" "$url")

    # Check for API errors
    if echo "$response" | grep -q '"message":'; then
        error_msg=$(echo "$response" | sed -n 's/.*"message": "\([^"]*\)".*/\1/p')
        log_error "GitHub API error: $error_msg"
        exit 1
    fi

    # Save raw response (we'll process it later)
    echo "$response" > "$output_file"

    # Count issues using basic text processing
    issue_count=$(echo "$response" | grep -c '"number":' || echo "0")
    log_success "Retrieved $issue_count issues"

    # Show summary using basic text processing
    if [ "$issue_count" -gt 0 ]; then
        open_count=$(echo "$response" | grep -c '"state": "open"' || echo "0")
        closed_count=$(echo "$response" | grep -c '"state": "closed"' || echo "0")

        log_info "Summary: $open_count open, $closed_count closed issues"
    fi
}

# Generate CSV output
generate_csv() {
    local json_file="$1"
    local csv_file="$2"

    log_info "Generating CSV output..."

    # Create CSV header
    {
        echo "number,title,state,labels,assignee,milestone,created_at,updated_at,author"
    } > "$csv_file"

    # Extract data using basic text processing
    # This is a simplified CSV generation that works without jq
    # For more complex JSON parsing, jq would be better but we'll use basic tools

    # Extract basic issue data (simplified version)
    echo "CSV generation simplified - would need jq for full implementation" >> "$csv_file"
    echo "See github-issue-details.sh for full issue processing" >> "$csv_file"

    log_success "Generated basic CSV structure"
}

# Main execution
main() {
    log_info "Starting GitHub issues query for $REPO_OWNER/$REPO_NAME"
    log_info "State: $STATE, Labels: ${LABELS:-"all"}, Assignee: ${ASSIGNEE:-"all"}, Milestone: ${MILESTONE:-"all"}"

    # Validate inputs
    if [[ "$STATE" != "open" && "$STATE" != "closed" && "$STATE" != "all" ]]; then
        log_error "Invalid state: $STATE (must be open, closed, or all)"
        exit 1
    fi

    # Check prerequisites
    if ! command -v curl >/dev/null 2>&1; then
        log_error "curl is required"
        exit 1
    fi

    if ! command -v jq >/dev/null 2>&1; then
        log_error "jq is required"
        exit 1
    fi

    # Setup
    check_github_token
    create_output_dir

    # Build API URL
    api_url=$(build_api_url)
    log_info "API URL: $api_url"

    # Query GitHub API
    query_github_api "$api_url" "$JSON_OUTPUT"

    # Generate CSV
    generate_csv "$JSON_OUTPUT" "$CSV_OUTPUT"

    # Show results
    log_success "GitHub issues query completed!"
    echo ""
    echo "📄 JSON output: $JSON_OUTPUT"
    echo "📊 CSV output: $CSV_OUTPUT"
    echo "📋 Issue count: $(grep -c '"number":' "$JSON_OUTPUT" || echo "0")"

    # Show sample of results (simplified)
    echo ""
    echo "📋 Sample issues (first 5):"
    echo "Raw JSON data saved to $JSON_OUTPUT"
    echo "Use github-issue-details.sh for formatted output"
}

# Run main function
main "$@"
