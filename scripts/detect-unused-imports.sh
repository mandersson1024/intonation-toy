#!/bin/bash

# Detect unused imports in Rust codebase
# This script uses multiple approaches to identify unused imports

set -e

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "🔍 Detecting unused imports in Rust codebase..."
echo "Working directory: $WORKSPACE_ROOT"
echo

# Create output directory
mkdir -p reports

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to extract unused import warnings from JSON output
extract_unused_imports() {
    local json_file="$1"
    local output_file="$2"
    
    if [[ -f "$json_file" ]]; then
        # Extract warnings about unused imports
        jq -r '
            select(.message?.level == "warning" and 
                   (.message?.code?.code == "unused_imports" or 
                    .message?.message | test("unused import"; "i"))) |
            {
                file: .message.spans[0].file_name,
                line: .message.spans[0].line_start,
                column: .message.spans[0].column_start,
                message: .message.message,
                code: (.message.code?.code // "unknown"),
                confidence: "high"
            }
        ' "$json_file" >> "$output_file" 2>/dev/null || true
    fi
}

# Function to analyze imports with ripgrep
analyze_imports_with_ripgrep() {
    echo "📊 Analyzing import patterns with ripgrep..."
    
    # Find all use statements
    rg --type rust --line-number "^[[:space:]]*use\s+" \
        --json > "reports/all-imports.json" 2>/dev/null || true
    
    # Find glob imports (use ... ::*)
    rg --type rust --line-number "use.*::\*" \
        > "reports/glob-imports.txt" 2>/dev/null || true
    
    # Find re-exports (pub use)
    rg --type rust --line-number "pub\s+use\s+" \
        > "reports/reexports.txt" 2>/dev/null || true
    
    # Find conditional imports (#[cfg(...)])
    rg --type rust --line-number -A1 "#\[cfg\(" \
        > "reports/conditional-imports.txt" 2>/dev/null || true
    
    echo "  ✓ Import pattern analysis complete"
}

# Function to run cargo-based detection
run_cargo_detection() {
    echo "🔧 Running cargo-based unused import detection..."
    
    # Clean previous builds to ensure fresh warnings
    cargo clean -q
    
    # Run cargo check with JSON output
    echo "  Running cargo check..."
    cargo check --workspace --all-targets --message-format=json \
        > "reports/cargo-check.json" 2>&1 || true
    
    # Run cargo clippy with JSON output
    echo "  Running cargo clippy..."
    cargo clippy --workspace --all-targets --message-format=json \
        > "reports/cargo-clippy.json" 2>&1 || true
    
    echo "  ✓ Cargo analysis complete"
}

# Function to generate reports
generate_reports() {
    echo "📝 Generating unused imports report..."
    
    # Initialize report files
    echo "[]" > "reports/unused-imports.json"
    > "reports/unused-imports-summary.txt"
    
    # Extract from cargo check
    extract_unused_imports "reports/cargo-check.json" "reports/unused-imports.json"
    
    # Extract from cargo clippy
    extract_unused_imports "reports/cargo-clippy.json" "reports/unused-imports.json"
    
    # Generate summary
    {
        echo "# Unused Imports Detection Report"
        echo "Generated: $(date)"
        echo "Workspace: $WORKSPACE_ROOT"
        echo
        
        # Count findings by type
        echo "## Summary"
        echo "- Total unused import warnings: $(jq length reports/unused-imports.json 2>/dev/null || echo "0")"
        echo "- Glob imports found: $(wc -l < reports/glob-imports.txt 2>/dev/null || echo "0")"
        echo "- Re-exports found: $(wc -l < reports/reexports.txt 2>/dev/null || echo "0")"
        echo
        
        # List findings by file
        echo "## Findings by File"
        if [[ -f "reports/unused-imports.json" ]] && [[ $(jq length reports/unused-imports.json) -gt 0 ]]; then
            jq -r 'group_by(.file) | .[] | "### \(.[0].file)", (.[] | "- Line \(.line): \(.message)")' \
                reports/unused-imports.json 2>/dev/null || echo "No detailed findings available"
        else
            echo "No unused imports detected by cargo tools."
        fi
        
        echo
        echo "## Glob Imports (Manual Review Needed)"
        if [[ -f "reports/glob-imports.txt" ]] && [[ -s "reports/glob-imports.txt" ]]; then
            cat "reports/glob-imports.txt"
        else
            echo "No glob imports found."
        fi
        
        echo
        echo "## Re-exports (Review for Necessity)"
        if [[ -f "reports/reexports.txt" ]] && [[ -s "reports/reexports.txt" ]]; then
            head -20 "reports/reexports.txt"
            if [[ $(wc -l < "reports/reexports.txt") -gt 20 ]]; then
                echo "... ($(( $(wc -l < "reports/reexports.txt") - 20 )) more entries)"
            fi
        else
            echo "No re-exports found."
        fi
        
    } > "reports/unused-imports-summary.txt"
    
    echo "  ✓ Report generated: reports/unused-imports-summary.txt"
    echo "  ✓ Detailed data: reports/unused-imports.json"
}

# Function to check for special cases
check_special_cases() {
    echo "⚠️  Checking for special cases that need manual review..."
    
    # Check for macro usage that might use imports
    echo "  Checking for macro usage..."
    rg --type rust "macro_rules!|#\[derive\(|#\[proc_macro" \
        > "reports/macro-usage.txt" 2>/dev/null || true
    
    # Check for WASM-specific imports
    echo "  Checking for WASM-specific imports..."
    rg --type rust "wasm_bindgen|#\[cfg\(target_arch.*wasm" \
        > "reports/wasm-imports.txt" 2>/dev/null || true
    
    # Check for test-only imports
    echo "  Checking for test-only imports..."
    rg --type rust "#\[cfg\(test\)\]|#\[test\]" \
        > "reports/test-imports.txt" 2>/dev/null || true
    
    echo "  ✓ Special cases analysis complete"
}

# Main execution
main() {
    # Check dependencies
    if ! command_exists jq; then
        echo "❌ jq is required but not installed. Please install jq."
        exit 1
    fi
    
    if ! command_exists rg; then
        echo "❌ ripgrep (rg) is required but not installed. Please install ripgrep."
        exit 1
    fi
    
    # Run detection
    run_cargo_detection
    analyze_imports_with_ripgrep
    check_special_cases
    generate_reports
    
    echo
    echo "🎉 Unused imports detection complete!"
    echo "📄 View summary: cat reports/unused-imports-summary.txt"
    echo "📊 View detailed data: jq . reports/unused-imports.json"
    echo
    echo "⚠️  Important Notes:"
    echo "   - Review glob imports manually for actual usage"
    echo "   - Check re-exports for public API requirements"
    echo "   - Verify conditional compilation imports are needed"
    echo "   - Test thoroughly after any import removals"
}

main "$@"