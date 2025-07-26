#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Validating dead code removal...${NC}"
echo "Project root: $PROJECT_ROOT"
echo

cd "$PROJECT_ROOT"

# Track validation results
VALIDATION_ERRORS=0
VALIDATION_WARNINGS=0

# Function to log with timestamp and level
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case "$level" in
        "INFO")
            echo -e "${BLUE}[${timestamp}] INFO:${NC} $message"
            ;;
        "WARN")
            echo -e "${YELLOW}[${timestamp}] WARN:${NC} $message"
            ((VALIDATION_WARNINGS++))
            ;;
        "ERROR")
            echo -e "${RED}[${timestamp}] ERROR:${NC} $message"
            ((VALIDATION_ERRORS++))
            ;;
        "SUCCESS")
            echo -e "${GREEN}[${timestamp}] SUCCESS:${NC} $message"
            ;;
    esac
}

# Function to run a command and capture its output and exit code
run_validation_command() {
    local description="$1"
    local command="$2"
    local allow_warnings="${3:-false}"
    
    log "INFO" "Running: $description"
    
    if eval "$command" >/dev/null 2>&1; then
        log "SUCCESS" "$description - PASSED"
        return 0
    else
        local exit_code=$?
        if [[ "$allow_warnings" == "true" && $exit_code -eq 0 ]]; then
            log "WARN" "$description - PASSED with warnings"
            return 0
        else
            log "ERROR" "$description - FAILED (exit code: $exit_code)"
            return 1
        fi
    fi
}

# Function to validate compilation
validate_compilation() {
    log "INFO" "=== Compilation Validation ==="
    
    # Check basic compilation
    run_validation_command "Basic compilation check" "cargo check --workspace --all-targets"
    
    # Check release compilation
    run_validation_command "Release compilation check" "cargo check --workspace --all-targets --release"
    
    # Check with all features
    if cargo metadata --format-version 1 | jq -e '.packages[] | select(.name == "pitch-toy") | .features | length > 0' >/dev/null 2>&1; then
        run_validation_command "All features compilation check" "cargo check --workspace --all-targets --all-features"
    else
        log "INFO" "No features defined, skipping all-features check"
    fi
    
    # Check documentation compilation
    run_validation_command "Documentation compilation" "cargo doc --workspace --no-deps" true
}

# Function to validate tests
validate_tests() {
    log "INFO" "=== Test Validation ==="
    
    # Run unit tests
    run_validation_command "Unit tests" "cargo test --workspace"
    
    # Run tests in release mode
    run_validation_command "Release mode tests" "cargo test --workspace --release"
    
    # Run WASM tests if available
    if command -v wasm-pack &> /dev/null; then
        run_validation_command "WASM tests" "wasm-pack test --node --all-features"
    else
        log "WARN" "wasm-pack not found, skipping WASM tests"
    fi
}

# Function to validate linting
validate_linting() {
    log "INFO" "=== Linting Validation ==="
    
    # Run clippy
    run_validation_command "Clippy analysis" "cargo clippy --workspace --all-targets -- -D warnings" true
    
    # Check formatting
    run_validation_command "Code formatting check" "cargo fmt --all -- --check"
    
    # Check for new dead code warnings
    local dead_code_output
    dead_code_output=$(RUSTFLAGS="-W dead_code" cargo check --workspace --all-targets --message-format=json 2>/dev/null | jq -r 'select(.message.code.code == "dead_code") | .message.message' | head -5)
    
    if [[ -n "$dead_code_output" ]]; then
        log "WARN" "New dead code warnings detected:"
        echo "$dead_code_output" | while read -r line; do
            log "WARN" "  $line"
        done
    else
        log "SUCCESS" "No dead code warnings detected"
    fi
}

# Function to validate WASM build
validate_wasm_build() {
    log "INFO" "=== WASM Build Validation ==="
    
    if command -v wasm-pack &> /dev/null; then
        # Build for different targets
        run_validation_command "WASM build for web" "wasm-pack build --target web"
        run_validation_command "WASM build for bundler" "wasm-pack build --target bundler"
        run_validation_command "WASM build for nodejs" "wasm-pack build --target nodejs"
        
        # Check generated files
        if [[ -d "pkg" ]]; then
            local pkg_files
            pkg_files=$(find pkg -name "*.wasm" -o -name "*.js" -o -name "*.ts" | wc -l)
            if [[ $pkg_files -gt 0 ]]; then
                log "SUCCESS" "WASM package generated with $pkg_files files"
            else
                log "ERROR" "WASM package directory exists but no WASM files found"
            fi
        else
            log "ERROR" "WASM package directory not created"
        fi
    else
        log "WARN" "wasm-pack not available, skipping WASM build validation"
    fi
}

# Function to validate specific functionality
validate_functionality() {
    log "INFO" "=== Functionality Validation ==="
    
    # Test that removed code doesn't break imports
    local broken_imports
    broken_imports=$(cargo check --workspace --all-targets --message-format=json 2>/dev/null | jq -r 'select(.message.code.code == "E0432" or .message.code.code == "E0433") | .message.message' | head -5)
    
    if [[ -n "$broken_imports" ]]; then
        log "ERROR" "Broken imports detected after dead code removal:"
        echo "$broken_imports" | while read -r line; do
            log "ERROR" "  $line"
        done
    else
        log "SUCCESS" "No broken imports detected"
    fi
    
    # Verify that essential functionality still works
    log "INFO" "Testing essential module functionality..."
    
    # Check that main modules can be imported
    local test_import_result
    test_import_result=$(cargo check --workspace --all-targets 2>&1 | grep -c "error" || echo "0")
    
    if [[ "$test_import_result" == "0" ]]; then
        log "SUCCESS" "All module imports successful"
    else
        log "ERROR" "Module import errors detected: $test_import_result errors"
    fi
}

# Function to check for unused dependencies
validate_dependencies() {
    log "INFO" "=== Dependency Validation ==="
    
    # Check for unused dependencies if cargo-udeps is available
    if command -v cargo-udeps &> /dev/null; then
        run_validation_command "Unused dependencies check" "cargo udeps --workspace" true
    else
        log "INFO" "cargo-udeps not available, skipping unused dependency check"
        log "INFO" "Install with: cargo install cargo-udeps"
    fi
    
    # Check for outdated dependencies if cargo-outdated is available
    if command -v cargo-outdated &> /dev/null; then
        run_validation_command "Outdated dependencies check" "cargo outdated --workspace" true
    else
        log "INFO" "cargo-outdated not available, skipping outdated dependency check"
    fi
}

# Function to validate binary size impact
validate_binary_size() {
    log "INFO" "=== Binary Size Validation ==="
    
    # Build in release mode and check size
    if run_validation_command "Release build for size check" "cargo build --workspace --release"; then
        local target_dir="target/release"
        if [[ -d "$target_dir" ]]; then
            local binary_files
            binary_files=$(find "$target_dir" -type f -name "pitch-toy*" -executable | head -5)
            
            if [[ -n "$binary_files" ]]; then
                log "SUCCESS" "Release binaries built successfully:"
                echo "$binary_files" | while read -r binary; do
                    local size
                    size=$(du -h "$binary" | cut -f1)
                    log "INFO" "  $(basename "$binary"): $size"
                done
            else
                log "INFO" "No executable binaries found (library-only project)"
            fi
        fi
    fi
}

# Function to run security audit
validate_security() {
    log "INFO" "=== Security Validation ==="
    
    # Run cargo audit if available
    if command -v cargo-audit &> /dev/null; then
        run_validation_command "Security audit" "cargo audit" true
    else
        log "INFO" "cargo-audit not available, skipping security audit"
        log "INFO" "Install with: cargo install cargo-audit"
    fi
}

# Function to validate performance impact
validate_performance() {
    log "INFO" "=== Performance Validation ==="
    
    # Build benchmarks if available
    if [[ -d "benches" ]] || grep -q "\[\[bench\]\]" Cargo.toml 2>/dev/null; then
        run_validation_command "Benchmark compilation" "cargo bench --no-run" true
    else
        log "INFO" "No benchmarks found, skipping benchmark validation"
    fi
    
    # Check for debug assertions in release builds
    local debug_assertions
    debug_assertions=$(cargo rustc --release -- --print cfg | grep -c debug_assertions || echo "0")
    
    if [[ "$debug_assertions" == "0" ]]; then
        log "SUCCESS" "Debug assertions properly disabled in release builds"
    else
        log "WARN" "Debug assertions may be enabled in release builds"
    fi
}

# Function to display final results
display_results() {
    echo
    echo -e "${BLUE}=== Validation Results ===${NC}"
    echo "=================================="
    
    if [[ $VALIDATION_ERRORS -eq 0 ]]; then
        log "SUCCESS" "All critical validations passed!"
    else
        log "ERROR" "$VALIDATION_ERRORS critical validation(s) failed!"
    fi
    
    if [[ $VALIDATION_WARNINGS -gt 0 ]]; then
        log "WARN" "$VALIDATION_WARNINGS warning(s) detected - review recommended"
    fi
    
    echo
    echo "Summary:"
    echo "  - Errors: $VALIDATION_ERRORS"
    echo "  - Warnings: $VALIDATION_WARNINGS"
    
    if [[ $VALIDATION_ERRORS -eq 0 ]]; then
        echo -e "${GREEN}✅ Dead code removal validation completed successfully!${NC}"
        echo
        echo "Next steps:"
        echo "1. Review any warnings above"
        echo "2. Consider running dead code detection again to verify cleanup"
        echo "3. Update documentation if significant changes were made"
        return 0
    else
        echo -e "${RED}❌ Validation failed with $VALIDATION_ERRORS error(s)${NC}"
        echo
        echo "Action required:"
        echo "1. Fix the errors listed above"
        echo "2. Re-run this validation script"
        echo "3. Consider reverting problematic changes if needed"
        return 1
    fi
}

# Main execution
main() {
    # Check dependencies
    local missing_deps=()
    
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo")
    fi
    
    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log "ERROR" "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    # Run all validation steps
    validate_compilation
    validate_tests
    validate_linting
    validate_wasm_build
    validate_functionality
    validate_dependencies
    validate_binary_size
    validate_security
    validate_performance
    
    # Display final results and exit with appropriate code
    display_results
}

# Handle script arguments
case "${1:-all}" in
    "compile"|"compilation")
        validate_compilation
        display_results
        ;;
    "test"|"tests")
        validate_tests
        display_results
        ;;
    "lint"|"linting")
        validate_linting
        display_results
        ;;
    "wasm")
        validate_wasm_build
        display_results
        ;;
    "deps"|"dependencies")
        validate_dependencies
        display_results
        ;;
    "quick")
        log "INFO" "Running quick validation (compile + test + lint only)"
        validate_compilation
        validate_tests
        validate_linting
        display_results
        ;;
    "all"|"")
        main
        ;;
    *)
        echo "Usage: $0 [compile|test|lint|wasm|deps|quick|all]"
        echo
        echo "Validation modes:"
        echo "  compile     - Test compilation only"
        echo "  test        - Run tests only"
        echo "  lint        - Run linting only"
        echo "  wasm        - Test WASM build only"
        echo "  deps        - Check dependencies only"
        echo "  quick       - Run core validations (compile + test + lint)"
        echo "  all         - Run all validations (default)"
        exit 1
        ;;
esac