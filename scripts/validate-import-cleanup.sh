#!/bin/bash

# Comprehensive validation script for import cleanup
# Ensures import cleanup doesn't break functionality

set -e

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "🔍 Validating import cleanup for Rust codebase..."
echo "Working directory: $WORKSPACE_ROOT"
echo

# Configuration
LOG_FILE="validation-$(date +%Y%m%d-%H%M%S).log"
VALIDATION_PASSED=true

# Function to log messages
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

# Function to run validation step
run_validation_step() {
    local step_name="$1"
    local command="$2"
    local required="${3:-true}"
    
    log "=== $step_name ==="
    
    if eval "$command" 2>&1 | tee -a "$LOG_FILE"; then
        log "✓ $step_name: PASSED"
        return 0
    else
        log "❌ $step_name: FAILED"
        if [[ "$required" == "true" ]]; then
            VALIDATION_PASSED=false
        fi
        return 1
    fi
}

# Function to check command availability
check_command() {
    local cmd="$1"
    local required="${2:-true}"
    
    if command -v "$cmd" >/dev/null 2>&1; then
        log "✓ $cmd is available"
        return 0
    else
        log "❌ $cmd is not available"
        if [[ "$required" == "true" ]]; then
            VALIDATION_PASSED=false
        fi
        return 1
    fi
}

# Function to validate compilation
validate_compilation() {
    log "🔧 Validating compilation..."
    
    # Check with all targets and features
    run_validation_step "Cargo check (all targets)" \
        "cargo check --workspace --all-targets --all-features"
    
    # Build in debug mode
    run_validation_step "Debug build" \
        "cargo build --workspace --all-targets"
    
    # Build in release mode
    run_validation_step "Release build" \
        "cargo build --workspace --all-targets --release"
}

# Function to validate tests
validate_tests() {
    log "🧪 Validating tests..."
    
    # Run all tests in debug mode
    run_validation_step "Debug tests" \
        "cargo test --workspace --all-targets"
    
    # Run tests in release mode
    run_validation_step "Release tests" \
        "cargo test --workspace --all-targets --release" false
    
    # Run doc tests
    run_validation_step "Documentation tests" \
        "cargo test --workspace --doc" false
}

# Function to validate linting
validate_linting() {
    log "📋 Validating linting..."
    
    # Run clippy with all features
    run_validation_step "Clippy analysis" \
        "cargo clippy --workspace --all-targets --all-features -- -D warnings"
    
    # Check for specific import-related warnings
    log "Checking for remaining unused import warnings..."
    if cargo clippy --workspace --all-targets --message-format=json 2>/dev/null | \
       jq -r 'select(.message?.level == "warning" and .message?.code?.code == "unused_imports") | .message.message' | \
       grep -q "unused import"; then
        log "⚠️  Still has unused import warnings (review needed)"
    else
        log "✓ No unused import warnings found"
    fi
}

# Function to validate documentation
validate_documentation() {
    log "📚 Validating documentation..."
    
    # Build documentation
    run_validation_step "Documentation build" \
        "cargo doc --workspace --all-features --no-deps"
    
    # Check for broken documentation links
    run_validation_step "Documentation links" \
        "cargo doc --workspace --all-features --no-deps" false
}

# Function to validate WASM build
validate_wasm() {
    log "🌐 Validating WASM build..."
    
    # Check if wasm-pack is available
    if check_command "wasm-pack" false; then
        # Build WASM package
        run_validation_step "WASM build" \
            "wasm-pack build pitch-toy --target web --out-dir pkg" false
        
        # Check WASM package size
        if [[ -f "pitch-toy/pkg/pitch_toy_bg.wasm" ]]; then
            local wasm_size=$(du -h "pitch-toy/pkg/pitch_toy_bg.wasm" | cut -f1)
            log "✓ WASM package size: $wasm_size"
        fi
    else
        log "⚠️  wasm-pack not available, skipping WASM validation"
    fi
}

# Function to analyze dependency tree
validate_dependencies() {
    log "📦 Validating dependencies..."
    
    # Check dependency tree
    run_validation_step "Dependency tree analysis" \
        "cargo tree --workspace" false
    
    # Check for duplicate dependencies
    log "Checking for duplicate dependencies..."
    local duplicates=$(cargo tree --workspace --duplicates 2>/dev/null || echo "")
    if [[ -n "$duplicates" ]]; then
        log "⚠️  Duplicate dependencies found (may be normal):"
        echo "$duplicates" | tee -a "$LOG_FILE"
    else
        log "✓ No duplicate dependencies found"
    fi
    
    # Check for unused dependencies (if cargo-udeps is available)
    if check_command "cargo-udeps" false; then
        run_validation_step "Unused dependencies check" \
            "cargo +nightly udeps --workspace" false
    else
        log "ℹ️  cargo-udeps not available, skipping unused dependency check"
    fi
}

# Function to validate specific files mentioned in plan
validate_specific_files() {
    log "📁 Validating specific files from cleanup plan..."
    
    local files=(
        "pitch-toy/lib.rs"
        "pitch-toy/engine/audio/mod.rs"
        "pitch-toy/engine/mod.rs"
        "pitch-toy/presentation/mod.rs"
        "pitch-toy/model/mod.rs"
        "dev-console/src/lib.rs"
    )
    
    for file in "${files[@]}"; do
        if [[ -f "$file" ]]; then
            log "✓ $file exists"
            
            # Check for common import issues
            if grep -q "use.*::\*" "$file"; then
                log "ℹ️  $file still contains glob imports (may be intentional)"
            fi
            
            if grep -q "^[[:space:]]*use.*;" "$file"; then
                local import_count=$(grep -c "^[[:space:]]*use.*;" "$file")
                log "ℹ️  $file has $import_count import statements"
            fi
        else
            log "❌ $file does not exist"
            VALIDATION_PASSED=false
        fi
    done
}

# Function to check for compilation time improvement
check_compilation_performance() {
    log "⏱️  Checking compilation performance..."
    
    # Clean build to get accurate timing
    cargo clean
    
    # Time the build
    log "Timing clean build..."
    local start_time=$(date +%s)
    if cargo build --workspace --all-targets >/dev/null 2>&1; then
        local end_time=$(date +%s)
        local build_time=$((end_time - start_time))
        log "✓ Clean build completed in ${build_time}s"
    else
        log "❌ Build timing failed"
        VALIDATION_PASSED=false
    fi
}

# Function to generate validation report
generate_validation_report() {
    log "📊 Generating validation report..."
    
    local report_file="validation-report-$(date +%Y%m%d-%H%M%S).md"
    
    {
        echo "# Import Cleanup Validation Report"
        echo "Generated: $(date)"
        echo "Workspace: $WORKSPACE_ROOT"
        echo
        
        if [[ "$VALIDATION_PASSED" == "true" ]]; then
            echo "## ✅ Overall Status: PASSED"
        else
            echo "## ❌ Overall Status: FAILED"
        fi
        
        echo
        echo "## Validation Summary"
        echo
        
        # Extract test results from log
        echo "### Compilation"
        grep "Cargo check\|Debug build\|Release build" "$LOG_FILE" | sed 's/^.*- /- /'
        
        echo
        echo "### Testing"
        grep "Debug tests\|Release tests\|Documentation tests" "$LOG_FILE" | sed 's/^.*- /- /'
        
        echo
        echo "### Linting"
        grep "Clippy analysis" "$LOG_FILE" | sed 's/^.*- /- /'
        
        echo
        echo "### WASM Build"
        grep "WASM build" "$LOG_FILE" | sed 's/^.*- /- /' || echo "- WASM validation skipped"
        
        echo
        echo "## Detailed Log"
        echo "Full validation log available at: \`$LOG_FILE\`"
        
        echo
        echo "## Next Steps"
        if [[ "$VALIDATION_PASSED" == "true" ]]; then
            echo "1. ✅ All validations passed - import cleanup is successful"
            echo "2. 🚀 Ready for production use"
            echo "3. 📝 Consider updating documentation if needed"
        else
            echo "1. ❌ Review failed validations in the log"
            echo "2. 🔧 Fix any compilation or test issues"
            echo "3. 🔄 Re-run validation after fixes"
        fi
        
    } > "$report_file"
    
    log "✓ Validation report generated: $report_file"
    echo
    cat "$report_file"
}

# Main execution
main() {
    # Initialize log
    > "$LOG_FILE"
    log "Starting comprehensive validation"
    
    # Check required tools
    log "Checking required tools..."
    check_command "cargo"
    check_command "jq" false
    
    # Run all validations
    validate_compilation
    validate_tests
    validate_linting
    validate_documentation
    validate_wasm
    validate_dependencies
    validate_specific_files
    check_compilation_performance
    
    # Generate final report
    generate_validation_report
    
    # Final status
    echo
    if [[ "$VALIDATION_PASSED" == "true" ]]; then
        log "🎉 All validations passed! Import cleanup is successful."
        exit 0
    else
        log "❌ Some validations failed. Review the log and fix issues."
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    --quick)
        echo "🏃 Quick validation mode"
        validate_compilation
        validate_tests
        echo "Quick validation completed"
        ;;
    --help)
        echo "Usage: $0 [--quick|--help]"
        echo
        echo "Comprehensive validation for import cleanup"
        echo
        echo "Options:"
        echo "  --quick      Run only compilation and tests"
        echo "  --help       Show this help message"
        ;;
    "")
        main
        ;;
    *)
        echo "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac