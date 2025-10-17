#!/bin/bash
# Script to fetch full details of GitHub issues from a JSON file
# Usage: ./scripts/github-issue-details.sh [issues.json] [output_dir]
# Examples:
#   ./scripts/github-issue-details.sh github-issues/issues.json github-issue-details/
#   ./scripts/github-issue-details.sh github-issues/issues.json

set -euo pipefail

# Configuration
REPO_OWNER="seanchatmangpt"
REPO_NAME="clnrm"
GITHUB_API="https://api.github.com"
OUTPUT_DIR="${2:-github-issue-details}"
ISSUES_FILE="${1:-github-issues/issues.json}"
DETAILS_DIR="${OUTPUT_DIR}/details"
MARKDOWN_DIR="${OUTPUT_DIR}/markdown"

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

# Create output directories
create_output_dirs() {
    mkdir -p "$DETAILS_DIR" "$MARKDOWN_DIR"
}

# Fetch individual issue details
fetch_issue_details() {
    local issue_number="$1"
    local output_file="$DETAILS_DIR/issue-${issue_number}.json"

    log_info "Fetching details for issue #$issue_number..."

    # Check if we already have the full details
    if [ -f "$output_file" ]; then
        log_info "Details already cached for issue #$issue_number"
        return 0
    fi

    local url="$GITHUB_API/repos/$REPO_OWNER/$REPO_NAME/issues/$issue_number"

    # Query GitHub API for full issue details
    response=$(curl -s -H "$AUTH_HEADER" "$url")

    # Check for API errors
    if echo "$response" | jq -e '.message' >/dev/null 2>&1; then
        error_msg=$(echo "$response" | jq -r '.message')
        log_error "GitHub API error for issue #$issue_number: $error_msg"
        return 1
    fi

    # Save full issue details
    echo "$response" > "$output_file"

    # Wait a bit to avoid rate limiting
    sleep 0.5
}

# Generate markdown from issue details
generate_markdown() {
    local issue_number="$1"
    local details_file="$DETAILS_DIR/issue-${issue_number}.json"
    local markdown_file="$MARKDOWN_DIR/issue-${issue_number}.md"

    if [ ! -f "$details_file" ]; then
        log_error "Issue details not found for #$issue_number"
        return 1
    fi

    log_info "Generating markdown for issue #$issue_number..."

    # Extract data using basic text processing and format as markdown
    # This is a simplified version - full implementation would use jq
    {
        echo "# Issue #$issue_number"
        echo ""
        echo "## Status"
        echo ""
        echo "- **State:** $(grep -o '"state": "[^"]*"' "$details_file" | head -1 | cut -d'"' -f4 || echo "Unknown")"
        echo "- **Title:** $(grep -o '"title": "[^"]*"' "$details_file" | head -1 | sed 's/"title": "\(.*\)"[^"]*$/\1/' || echo "Unknown")"
        echo "- **Author:** @$(grep -o '"login": "[^"]*"' "$details_file" | head -1 | cut -d'"' -f4 || echo "Unknown")"
        echo "- **Created:** $(grep -o '"created_at": "[^"]*"' "$details_file" | head -1 | cut -d'"' -f4 | cut -d'T' -f1 || echo "Unknown")"
        echo ""
        echo "## Description"
        echo ""
        echo "Full issue details saved to JSON. Use GitHub web interface for complete content."
        echo ""
        echo "## Links"
        echo ""
        echo "- [View on GitHub](https://github.com/$REPO_OWNER/$REPO_NAME/issues/$issue_number)"
        echo "- [Edit on GitHub](https://github.com/$REPO_OWNER/$REPO_NAME/issues/$issue_number/edit)"
    } > "$markdown_file"

    log_success "Generated markdown for issue #$issue_number"
}

# Main execution
main() {
    # Validate input file exists
    if [ ! -f "$ISSUES_FILE" ]; then
        log_error "Issues file not found: $ISSUES_FILE"
        log_error "Run ./scripts/github-issues.sh first to generate the issues list"
        exit 1
    fi

    log_info "Starting GitHub issue details fetch..."
    log_info "Issues file: $ISSUES_FILE"
    log_info "Output directory: $OUTPUT_DIR"

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
    create_output_dirs

    # Read issues from JSON file (simplified without jq)
    issue_numbers=$(grep -o '"number": [0-9]*' "$ISSUES_FILE" | grep -o '[0-9]*')

    if [ -z "$issue_numbers" ]; then
        log_warning "No issue numbers found in $ISSUES_FILE"
        exit 1
    fi

    total_issues=$(echo "$issue_numbers" | wc -l)
    log_info "Found $total_issues issues to process"

    # Process each issue
    processed=0
    failed=0

    while IFS= read -r issue_number; do
        if [ -n "$issue_number" ] && [ "$issue_number" != "null" ]; then
            if fetch_issue_details "$issue_number"; then
                generate_markdown "$issue_number"
                ((processed++))
            else
                ((failed++))
            fi
        fi
    done <<< "$issue_numbers"

    # Generate summary
    log_success "Issue details processing completed!"
    echo ""
    echo "📊 Summary:"
    echo "  ✅ Successfully processed: $processed issues"
    echo "  ❌ Failed: $failed issues"
    echo "  📁 Details saved to: $DETAILS_DIR/"
    echo "  📝 Markdown saved to: $MARKDOWN_DIR/"

    # Generate index files
    generate_index_files

    log_success "GitHub issue details fetch completed!"
}

# Generate index files for easy navigation
generate_index_files() {
    log_info "Generating index files..."

    # Generate JSON index
    {
        echo "{"
        echo "  \"generated_at\": \"$(date -Iseconds)\","
        echo "  \"total_issues\": $total_issues,"
        echo "  \"processed_issues\": $processed,"
        echo "  \"failed_issues\": $failed,"
        echo "  \"issues\": ["
        first=true
        for file in "$DETAILS_DIR"/issue-*.json; do
            if [ -f "$file" ]; then
                issue_number=$(basename "$file" | sed 's/issue-\([0-9]*\)\.json/\1/')
                if [ "$first" = true ]; then
                    first=false
                else
                    echo ","
                fi
                # Simplified JSON processing without jq
                echo "{\"number\": $issue_number, \"details_available\": true}"
            fi
        done
        echo "  ]"
        echo "}"
    } > "${OUTPUT_DIR}/issues-index.json"

    # Generate markdown index
    {
        echo "# GitHub Issues Summary"
        echo ""
        echo "Generated on $(date)"
        echo ""
        echo "## Overview"
        echo ""
        echo "- **Total Issues:** $total_issues"
        echo "- **Successfully Processed:** $processed"
        echo "- **Failed:** $failed"
        echo ""
        echo "## Issues"
        echo ""
        echo "| Issue # | Title | State | Author | Created |"
        echo "|---------|-------|-------|--------|---------|"
        for file in "$DETAILS_DIR"/issue-*.json; do
            if [ -f "$file" ]; then
                issue_number=$(basename "$file" | sed 's/issue-\([0-9]*\)\.json/\1/')
                # Extract basic info using grep and sed (simplified)
                title=$(grep -o '"title": "[^"]*"' "$file" | head -1 | sed 's/"title": "\(.*\)"[^"]*$/\1/' | sed 's/|/\\|/g' || echo "Unknown Title")
                state=$(grep -o '"state": "[^"]*"' "$file" | head -1 | cut -d'"' -f4 || echo "unknown")
                author=$(grep -o '"login": "[^"]*"' "$file" | head -1 | cut -d'"' -f4 || echo "unknown")
                created=$(grep -o '"created_at": "[^"]*"' "$file" | head -1 | cut -d'"' -f4 | cut -d'T' -f1 || echo "unknown")
                echo "| #$issue_number | $title | $state | @$author | $created |"
            fi
        done
        echo ""
        echo "## Files"
        echo ""
        echo "- **JSON Details:** \`$DETAILS_DIR/\`"
        echo "- **Markdown Files:** \`$MARKDOWN_DIR/\`"
        echo "- **Issues Index:** \`issues-index.json\`"
        echo "- **CSV Export:** \`../github-issues/issues.csv\`"
    } > "${OUTPUT_DIR}/README.md"

    log_success "Generated index files"
}

# Run main function with all arguments
main "$@"
