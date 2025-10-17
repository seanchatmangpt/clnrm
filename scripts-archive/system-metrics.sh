#!/usr/bin/env bash
# ==============================================================================
# system-metrics.sh
# System Metrics Collection and Analysis for Cleanroom Testing Framework
#
# This script collects comprehensive system metrics including CPU, memory,
# disk usage, Docker resources, and build performance metrics.
#
# Usage:
#   ./scripts/system-metrics.sh                    # Display current metrics
#   ./scripts/system-metrics.sh --json             # Output in JSON format
#   ./scripts/system-metrics.sh --monitor 60       # Monitor for 60 seconds
#   ./scripts/system-metrics.sh --report           # Generate detailed report
#   ./scripts/system-metrics.sh --cleanup          # Clean up system resources
#
# Exit Codes:
#   0 - Success
#   1 - Warning (some metrics unavailable)
#   2 - Error (critical metrics failed)
# ==============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
METRICS_FILE="$PROJECT_ROOT/system-metrics.json"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Metrics storage
declare -A METRICS
declare -A WARNINGS
declare -A ERRORS

# Initialize arrays
METRICS=()
WARNINGS=()
ERRORS=()

# Log functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
    WARNINGS["$1"]="true"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ERRORS["$1"]="true"
}

log_header() {
    echo -e "${PURPLE}[METRICS]${NC} $1"
}

# Get CPU information
get_cpu_info() {
    log_header "⚡ CPU Information"
    
    if command -v nproc &> /dev/null; then
        local cpu_cores=$(nproc)
        METRICS["cpu_cores"]="$cpu_cores"
        echo "  CPU Cores: $cpu_cores"
    elif command -v sysctl &> /dev/null; then
        local cpu_cores=$(sysctl -n hw.ncpu 2>/dev/null || echo "unknown")
        METRICS["cpu_cores"]="$cpu_cores"
        echo "  CPU Cores: $cpu_cores"
    else
        log_warning "CPU core count unavailable"
        METRICS["cpu_cores"]="unknown"
    fi
    
    # CPU usage (if available)
    if command -v top &> /dev/null; then
        local cpu_usage=$(top -l 1 -s 0 | grep "CPU usage" | awk '{print $3}' | sed 's/%//' 2>/dev/null || echo "unknown")
        METRICS["cpu_usage_percent"]="$cpu_usage"
        echo "  CPU Usage: ${cpu_usage}%"
    else
        log_warning "CPU usage monitoring unavailable"
        METRICS["cpu_usage_percent"]="unknown"
    fi
}

# Get memory information
get_memory_info() {
    log_header "💾 Memory Information"
    
    if command -v free &> /dev/null; then
        # Linux
        local mem_info=$(free -h | grep "Mem:")
        local total_mem=$(echo "$mem_info" | awk '{print $2}')
        local used_mem=$(echo "$mem_info" | awk '{print $3}')
        local available_mem=$(echo "$mem_info" | awk '{print $7}')
        
        METRICS["memory_total"]="$total_mem"
        METRICS["memory_used"]="$used_mem"
        METRICS["memory_available"]="$available_mem"
        
        echo "  Total Memory: $total_mem"
        echo "  Used Memory: $used_mem"
        echo "  Available Memory: $available_mem"
    elif command -v vm_stat &> /dev/null; then
        # macOS
        local page_size=$(vm_stat | head -1 | awk '{print $8}' | sed 's/\.//')
        local free_pages=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//')
        local active_pages=$(vm_stat | grep "Pages active" | awk '{print $3}' | sed 's/\.//')
        local inactive_pages=$(vm_stat | grep "Pages inactive" | awk '{print $3}' | sed 's/\.//')
        local wired_pages=$(vm_stat | grep "Pages wired down" | awk '{print $4}' | sed 's/\.//')
        
        local total_pages=$((free_pages + active_pages + inactive_pages + wired_pages))
        local total_mb=$((total_pages * page_size / 1024 / 1024))
        local used_mb=$(((active_pages + wired_pages) * page_size / 1024 / 1024))
        local free_mb=$((free_pages * page_size / 1024 / 1024))
        
        METRICS["memory_total"]="${total_mb}MB"
        METRICS["memory_used"]="${used_mb}MB"
        METRICS["memory_available"]="${free_mb}MB"
        
        echo "  Total Memory: ${total_mb}MB"
        echo "  Used Memory: ${used_mb}MB"
        echo "  Available Memory: ${free_mb}MB"
    else
        log_warning "Memory information unavailable"
        METRICS["memory_total"]="unknown"
        METRICS["memory_used"]="unknown"
        METRICS["memory_available"]="unknown"
    fi
}

# Get disk information
get_disk_info() {
    log_header "💽 Disk Information"
    
    if command -v df &> /dev/null; then
        local disk_info=$(df -h . | tail -1)
        local total_disk=$(echo "$disk_info" | awk '{print $2}')
        local used_disk=$(echo "$disk_info" | awk '{print $3}')
        local available_disk=$(echo "$disk_info" | awk '{print $4}')
        local usage_percent=$(echo "$disk_info" | awk '{print $5}')
        
        METRICS["disk_total"]="$total_disk"
        METRICS["disk_used"]="$used_disk"
        METRICS["disk_available"]="$available_disk"
        METRICS["disk_usage_percent"]="$usage_percent"
        
        echo "  Total Disk: $total_disk"
        echo "  Used Disk: $used_disk"
        echo "  Available Disk: $available_disk"
        echo "  Usage: $usage_percent"
        
        # Check if disk usage is high
        local usage_num=$(echo "$usage_percent" | sed 's/%//')
        if [[ $usage_num -gt 90 ]]; then
            log_warning "High disk usage: $usage_percent"
        fi
    else
        log_error "Disk information unavailable"
        METRICS["disk_total"]="unknown"
        METRICS["disk_used"]="unknown"
        METRICS["disk_available"]="unknown"
        METRICS["disk_usage_percent"]="unknown"
    fi
}

# Get Docker information
get_docker_info() {
    log_header "🐳 Docker Information"
    
    if command -v docker &> /dev/null; then
        # Docker version
        local docker_version=$(docker --version 2>/dev/null | awk '{print $3}' | sed 's/,//' || echo "unknown")
        METRICS["docker_version"]="$docker_version"
        echo "  Docker Version: $docker_version"
        
        # Docker daemon status
        if docker ps &> /dev/null; then
            METRICS["docker_daemon_status"]="running"
            echo "  Docker Daemon: Running"
            
            # Docker system info
            local docker_info=$(docker system df 2>/dev/null || echo "")
            if [[ -n "$docker_info" ]]; then
                echo "  Docker Resources:"
                echo "$docker_info" | tail -n +2 | while read -r line; do
                    echo "    $line"
                done
                
                # Extract specific metrics
                local images_size=$(echo "$docker_info" | grep "Images" | awk '{print $3}')
                local containers_size=$(echo "$docker_info" | grep "Containers" | awk '{print $3}')
                local volumes_size=$(echo "$docker_info" | grep "Local Volumes" | awk '{print $4}')
                
                METRICS["docker_images_size"]="$images_size"
                METRICS["docker_containers_size"]="$containers_size"
                METRICS["docker_volumes_size"]="$volumes_size"
            fi
            
            # Running containers
            local running_containers=$(docker ps --format "table {{.Names}}\t{{.Status}}" 2>/dev/null | wc -l)
            METRICS["docker_running_containers"]=$((running_containers - 1))
            echo "  Running Containers: $((running_containers - 1))"
        else
            log_warning "Docker daemon not running"
            METRICS["docker_daemon_status"]="stopped"
        fi
    else
        log_warning "Docker not installed"
        METRICS["docker_version"]="not_installed"
        METRICS["docker_daemon_status"]="not_available"
    fi
}

# Get Rust/Cargo information
get_rust_info() {
    log_header "🦀 Rust Information"
    
    if command -v rustc &> /dev/null; then
        local rust_version=$(rustc --version | awk '{print $2}')
        METRICS["rust_version"]="$rust_version"
        echo "  Rust Version: $rust_version"
    else
        log_error "Rust not found"
        METRICS["rust_version"]="not_found"
    fi
    
    if command -v cargo &> /dev/null; then
        local cargo_version=$(cargo --version | awk '{print $2}')
        METRICS["cargo_version"]="$cargo_version"
        echo "  Cargo Version: $cargo_version"
    else
        log_error "Cargo not found"
        METRICS["cargo_version"]="not_found"
    fi
}

# Get build performance metrics
get_build_metrics() {
    log_header "🔨 Build Performance Metrics"
    
    if command -v cargo &> /dev/null; then
        # Check if target directory exists
        if [[ -d "target" ]]; then
            local target_size=$(du -sh target 2>/dev/null | awk '{print $1}' || echo "unknown")
            METRICS["target_directory_size"]="$target_size"
            echo "  Target Directory Size: $target_size"
        else
            METRICS["target_directory_size"]="not_present"
            echo "  Target Directory: Not present"
        fi
        
        # Cargo cache size
        local cargo_home="${CARGO_HOME:-$HOME/.cargo}"
        if [[ -d "$cargo_home" ]]; then
            local cargo_cache_size=$(du -sh "$cargo_home" 2>/dev/null | awk '{print $1}' || echo "unknown")
            METRICS["cargo_cache_size"]="$cargo_cache_size"
            echo "  Cargo Cache Size: $cargo_cache_size"
        else
            METRICS["cargo_cache_size"]="not_found"
            echo "  Cargo Cache: Not found"
        fi
        
        # Build time estimation (if possible)
        if [[ -f "Cargo.toml" ]]; then
            log_info "Estimating build time..."
            local start_time=$(date +%s)
            if timeout 60 cargo check --quiet 2>/dev/null; then
                local end_time=$(date +%s)
                local build_time=$((end_time - start_time))
                METRICS["estimated_build_time_seconds"]="$build_time"
                echo "  Estimated Build Time: ${build_time}s"
            else
                log_warning "Build time estimation timed out"
                METRICS["estimated_build_time_seconds"]="timeout"
            fi
        fi
    else
        log_error "Cargo not available for build metrics"
    fi
}

# Monitor system for specified duration
monitor_system() {
    local duration=$1
    local interval=5
    local iterations=$((duration / interval))
    
    log_header "📊 Monitoring System for ${duration}s (${iterations} samples)"
    
    for ((i=1; i<=iterations; i++)); do
        echo "Sample $i/$iterations:"
        get_cpu_info
        get_memory_info
        echo ""
        sleep "$interval"
    done
}

# Generate JSON report
generate_json_report() {
    local output_file="$1"
    
    log_info "Generating JSON metrics report..."
    
    # Create JSON structure
    cat > "$output_file" << EOF
{
  "timestamp": "$TIMESTAMP",
  "project": "clnrm",
  "system_metrics": {
$(for key in "${!METRICS[@]}"; do
    echo "    \"$key\": \"${METRICS[$key]}\","
done | sed '$s/,$//')
  },
  "warnings": [
$(for warning in "${!WARNINGS[@]}"; do
    echo "    \"$warning\","
done | sed '$s/,$//')
  ],
  "errors": [
$(for error in "${!ERRORS[@]}"; do
    echo "    \"$error\","
done | sed '$s/,$//')
  ],
  "summary": {
    "total_metrics": ${#METRICS[@]},
    "warnings_count": ${#WARNINGS[@]},
    "errors_count": ${#ERRORS[@]},
    "system_health": "$([ ${#ERRORS[@]} -eq 0 ] && echo "healthy" || echo "degraded")"
  }
}
EOF
    
    log_success "JSON report generated: $output_file"
}

# Cleanup system resources
cleanup_system() {
    log_header "🧹 System Cleanup"
    
    # Clean Docker resources
    if command -v docker &> /dev/null && docker ps &> /dev/null; then
        log_info "Cleaning Docker resources..."
        docker system prune -f 2>/dev/null || log_warning "Docker cleanup failed"
        docker volume prune -f 2>/dev/null || log_warning "Docker volume cleanup failed"
    fi
    
    # Clean Cargo cache
    if command -v cargo &> /dev/null; then
        log_info "Cleaning Cargo cache..."
        cargo clean 2>/dev/null || log_warning "Cargo cleanup failed"
    fi
    
    # Clean temporary files
    log_info "Cleaning temporary files..."
    find . -name "*.orig" -delete 2>/dev/null || true
    find . -name "*.rej" -delete 2>/dev/null || true
    find . -name ".DS_Store" -delete 2>/dev/null || true
    
    log_success "System cleanup completed"
}

# Main function
main() {
    local json_output=false
    local monitor_duration=0
    local generate_report=false
    local cleanup=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --json)
                json_output=true
                shift
                ;;
            --monitor)
                monitor_duration="$2"
                shift 2
                ;;
            --report)
                generate_report=true
                shift
                ;;
            --cleanup)
                cleanup=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --json           Output metrics in JSON format"
                echo "  --monitor SECS   Monitor system for specified seconds"
                echo "  --report         Generate detailed JSON report"
                echo "  --cleanup        Clean up system resources"
                echo "  --help, -h       Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Handle cleanup mode
    if [[ "$cleanup" == true ]]; then
        cleanup_system
        exit 0
    fi
    
    # Handle monitoring mode
    if [[ $monitor_duration -gt 0 ]]; then
        monitor_system "$monitor_duration"
        exit 0
    fi
    
    # Collect metrics
    log_header "📊 Clnrm System Metrics Collection"
    echo "Timestamp: $TIMESTAMP"
    echo ""
    
    get_cpu_info
    echo ""
    get_memory_info
    echo ""
    get_disk_info
    echo ""
    get_docker_info
    echo ""
    get_rust_info
    echo ""
    get_build_metrics
    echo ""
    
    # Generate report if requested
    if [[ "$generate_report" == true ]]; then
        generate_json_report "$METRICS_FILE"
    fi
    
    # JSON output mode
    if [[ "$json_output" == true ]]; then
        generate_json_report "/dev/stdout"
    fi
    
    # Summary
    log_header "📋 Metrics Summary"
    echo "Total Metrics Collected: ${#METRICS[@]}"
    echo "Warnings: ${#WARNINGS[@]}"
    echo "Errors: ${#ERRORS[@]}"
    
    local error_count=${#ERRORS[@]}
    local warning_count=${#WARNINGS[@]}
    
    if [[ $error_count -gt 0 ]]; then
        log_error "System has critical issues"
        exit 2
    elif [[ $warning_count -gt 0 ]]; then
        log_warning "System has warnings"
        exit 1
    else
        log_success "System metrics collection completed successfully"
        exit 0
    fi
}

# Run main function
main "$@"
