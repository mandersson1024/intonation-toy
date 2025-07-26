#!/bin/bash

# Dead Code Detection Script for Rust Project
# Detects unused code using multiple approaches for comprehensive analysis

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="$PROJECT_ROOT/target/dead-code-reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Ensure report directory exists
mkdir -p "$REPORT_DIR"

echo -e "${BLUE}🔍 Starting dead code detection...${NC}"
echo "Project root: $PROJECT_ROOT"
echo "Report directory: $REPORT_DIR"
echo

# Function to log with timestamp
log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

# Function to detect dead code using cargo check
detect_with_cargo_check() {
    log "Running cargo check for dead code warnings..."
    
    cd "$PROJECT_ROOT"
    
    # Run cargo check with dead_code warnings
    RUSTFLAGS="-W dead_code" cargo check --workspace --all-targets --message-format=json 2>/dev/null \
        | jq -r 'select(.message.code.code == "dead_code") | 
                 {
                   file: .message.spans[0].file_name,
                   line: .message.spans[0].line_start,
                   column: .message.spans[0].column_start,
                   message: .message.message,
                   code_type: (.message.message | 
                     if test("function") then "function"
                     elif test("struct") then "struct"
                     elif test("enum") then "enum"
                     elif test("variant") then "enum_variant"
                     elif test("field") then "field"
                     elif test("module") then "module"
                     elif test("constant") then "constant"
                     elif test("static") then "static"
                     elif test("type") then "type"
                     else "unknown"
                     end),
                   confidence: "high"
                 }' > "$REPORT_DIR/cargo-check-dead-code.json"
    
    local count=$(jq length "$REPORT_DIR/cargo-check-dead-code.json")
    echo -e "${GREEN}✓${NC} Found $count dead code items via cargo check"
}

# Function to detect dead code using clippy
detect_with_clippy() {
    log "Running clippy for additional dead code detection..."
    
    cd "$PROJECT_ROOT"
    
    # Run clippy with dead code and unused lints
    cargo clippy --workspace --all-targets --message-format=json -- \
        -W dead_code -W unused_imports -W unused_variables -W unused_mut 2>/dev/null \
        | jq -r 'select(.message.code.code and 
                         (.message.code.code | test("dead_code|unused_"))) | 
                 {
                   file: .message.spans[0].file_name,
                   line: .message.spans[0].line_start,
                   column: .message.spans[0].column_start,
                   message: .message.message,
                   lint_code: .message.code.code,
                   code_type: (.message.message | 
                     if test("function") then "function"
                     elif test("struct") then "struct"
                     elif test("enum") then "enum"
                     elif test("variant") then "enum_variant"
                     elif test("field") then "field"
                     elif test("import") then "import"
                     elif test("variable") then "variable"
                     elif test("module") then "module"
                     else "unknown"
                     end),
                   confidence: "high"
                 }' > "$REPORT_DIR/clippy-dead-code.json"
    
    local count=$(jq length "$REPORT_DIR/clippy-dead-code.json")
    echo -e "${GREEN}✓${NC} Found $count additional items via clippy"
}

# Function to analyze patterns with ripgrep
analyze_patterns() {
    log "Analyzing code patterns for potential dead code..."
    
    cd "$PROJECT_ROOT"
    
    # Create patterns analysis
    cat > "$REPORT_DIR/pattern-analysis.json" << 'EOF'
[]
EOF
    
    # Find TODO/PLACEHOLDER marked items
    log "  - Searching for TODO/PLACEHOLDER items..."
    rg --type rust --json "(?i)(todo|placeholder|fixme|unimplemented)" \
        --field-match-separator=: \
        pitch-toy/ dev-console/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 {
                   file: .data.path.text,
                   line: .data.line_number,
                   column: .data.submatches[0].start,
                   message: ("TODO/PLACEHOLDER found: " + .data.lines.text),
                   code_type: "todo_placeholder",
                   confidence: "medium"
                 }' \
        | jq -s '.' > "$REPORT_DIR/todo-placeholder-items.json"
    
    # Find functions with no references (simplified analysis)
    log "  - Searching for potentially unused functions..."
    rg --type rust --json "^(?:pub\s+)?fn\s+(\w+)" \
        pitch-toy/ dev-console/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 .data.submatches[0].match.text' \
        | sort -u > "$REPORT_DIR/all-functions.txt" || echo "No functions found"
    
    # Find debug-only items that might be unused
    log "  - Searching for debug-only items..."
    rg --type rust --json "#\[cfg\(debug_assertions\)\]" \
        pitch-toy/ dev-console/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 {
                   file: .data.path.text,
                   line: .data.line_number,
                   column: .data.submatches[0].start,
                   message: "Debug-only code found",
                   code_type: "debug_only",
                   confidence: "low"
                 }' \
        | jq -s '.' > "$REPORT_DIR/debug-only-items.json"
    
    # Find test-related code that might be orphaned
    log "  - Searching for test-related items..."
    rg --type rust --json "#\[cfg\(test\)\]|#\[test\]" \
        pitch-toy/ dev-console/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 {
                   file: .data.path.text,
                   line: .data.line_number,
                   column: .data.submatches[0].start,
                   message: "Test-related code found",
                   code_type: "test_related",
                   confidence: "low"
                 }' \
        | jq -s '.' > "$REPORT_DIR/test-related-items.json"
    
    local todo_count=$(jq length "$REPORT_DIR/todo-placeholder-items.json")
    local debug_count=$(jq length "$REPORT_DIR/debug-only-items.json")
    local test_count=$(jq length "$REPORT_DIR/test-related-items.json")
    
    echo -e "${GREEN}✓${NC} Pattern analysis complete:"
    echo "  - TODO/PLACEHOLDER items: $todo_count"
    echo "  - Debug-only items: $debug_count"
    echo "  - Test-related items: $test_count"
}

# Function to generate comprehensive report
generate_report() {
    log "Generating comprehensive dead code report..."
    
    # Combine all findings
    jq -s 'add' \
        "$REPORT_DIR/cargo-check-dead-code.json" \
        "$REPORT_DIR/clippy-dead-code.json" \
        "$REPORT_DIR/todo-placeholder-items.json" \
        "$REPORT_DIR/debug-only-items.json" \
        "$REPORT_DIR/test-related-items.json" \
        > "$REPORT_DIR/dead-code-report.json"
    
    # Generate summary
    cat > "$REPORT_DIR/dead-code-summary.txt" << EOF
Dead Code Detection Report
Generated: $(date)
Project: pitch-toy

SUMMARY:
$(jq -r '
group_by(.code_type) | 
map({
  type: .[0].code_type,
  count: length,
  confidence_breakdown: (group_by(.confidence) | map({confidence: .[0].confidence, count: length}))
}) | 
.[] | "- \(.type): \(.count) items (\(.confidence_breakdown | map("\(.count) \(.confidence)") | join(", ")))"
' "$REPORT_DIR/dead-code-report.json")

TOTAL ITEMS: $(jq length "$REPORT_DIR/dead-code-report.json")

HIGH CONFIDENCE ITEMS (Compiler/Clippy warnings):
$(jq -r '.[] | select(.confidence == "high") | "- \(.file):\(.line) - \(.code_type): \(.message)"' "$REPORT_DIR/dead-code-report.json")

DETAILED BREAKDOWN BY FILE:
$(jq -r '
group_by(.file) | 
map({
  file: .[0].file,
  items: length,
  types: (map(.code_type) | unique)
}) | 
.[] | "- \(.file): \(.items) items (\(.types | join(", ")))"
' "$REPORT_DIR/dead-code-report.json")

RECOMMENDATIONS:
1. Start with high-confidence items (compiler/clippy warnings)
2. Review TODO/PLACEHOLDER items for removal or implementation
3. Analyze debug-only code for actual usage
4. Consider test-related code for cleanup opportunities

FILES TO REVIEW:
$(jq -r 'map(.file) | unique | .[]' "$REPORT_DIR/dead-code-report.json")
EOF
    
    echo -e "${GREEN}✓${NC} Report generated successfully"
    echo "  - Detailed report: $REPORT_DIR/dead-code-report.json"
    echo "  - Summary report: $REPORT_DIR/dead-code-summary.txt"
}

# Function to display summary
display_summary() {
    echo
    echo -e "${YELLOW}📊 DEAD CODE DETECTION SUMMARY${NC}"
    echo "=================================="
    
    local total_items=$(jq length "$REPORT_DIR/dead-code-report.json")
    local high_confidence=$(jq '[.[] | select(.confidence == "high")] | length' "$REPORT_DIR/dead-code-report.json")
    local medium_confidence=$(jq '[.[] | select(.confidence == "medium")] | length' "$REPORT_DIR/dead-code-report.json")
    local low_confidence=$(jq '[.[] | select(.confidence == "low")] | length' "$REPORT_DIR/dead-code-report.json")
    
    echo "Total dead code items found: $total_items"
    echo "  - High confidence (compiler/clippy): $high_confidence"
    echo "  - Medium confidence (patterns): $medium_confidence"
    echo "  - Low confidence (analysis): $low_confidence"
    echo
    
    if [ "$high_confidence" -gt 0 ]; then
        echo -e "${RED}⚠️  High confidence items require immediate attention${NC}"
    fi
    
    if [ "$medium_confidence" -gt 0 ]; then
        echo -e "${YELLOW}⚠️  Medium confidence items should be reviewed${NC}"
    fi
    
    echo
    echo "Next steps:"
    echo "1. Review the summary: cat $REPORT_DIR/dead-code-summary.txt"
    echo "2. Analyze detailed report: $REPORT_DIR/dead-code-report.json"
    echo "3. Use cross-crate analysis script for public API cleanup"
    echo "4. Remove dead code systematically with validation"
}

# Main execution
main() {
    # Check dependencies
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}Error: jq is required but not installed${NC}"
        exit 1
    fi
    
    if ! command -v rg &> /dev/null; then
        echo -e "${RED}Error: ripgrep (rg) is required but not installed${NC}"
        exit 1
    fi
    
    # Run detection steps
    detect_with_cargo_check
    detect_with_clippy
    analyze_patterns
    generate_report
    display_summary
    
    echo -e "${GREEN}🎉 Dead code detection complete!${NC}"
}

# Run main function
main "$@"