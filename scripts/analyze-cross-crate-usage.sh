#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/target/cross-crate-analysis"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

mkdir -p "$OUTPUT_DIR"

echo -e "${BLUE}🔍 Analyzing cross-crate usage patterns...${NC}"
echo "Project root: $PROJECT_ROOT"
echo "Output directory: $OUTPUT_DIR"
echo

cd "$PROJECT_ROOT"

# Function to log with timestamp
log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

# Extract public API from pitch-toy crate
extract_pitch_toy_public_api() {
    log "Extracting public API from pitch-toy crate..."
    
    # Find all pub items in pitch-toy
    rg --type rust --json "pub\s+(fn|struct|enum|trait|type|const|static|mod|use)" \
        pitch-toy/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 {
                   file: .data.path.text,
                   line: .data.line_number,
                   column: .data.submatches[0].start,
                   item_text: .data.lines.text,
                   item_type: (.data.lines.text | 
                     if test("pub fn") then "function"
                     elif test("pub struct") then "struct"
                     elif test("pub enum") then "enum"
                     elif test("pub trait") then "trait"
                     elif test("pub type") then "type"
                     elif test("pub const") then "const"
                     elif test("pub static") then "static"
                     elif test("pub mod") then "module"
                     elif test("pub use") then "re_export"
                     else "unknown"
                     end),
                   item_name: (.data.lines.text | 
                     if test("pub fn") then (. | match("pub fn\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub struct") then (. | match("pub struct\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub enum") then (. | match("pub enum\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub trait") then (. | match("pub trait\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub type") then (. | match("pub type\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub const") then (. | match("pub const\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub static") then (. | match("pub static\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub mod") then (. | match("pub mod\\s+(\\w+)"; "g") | .captures[0].string)
                     else "unknown"
                     end)
                 }' \
        | jq -s '.' > "$OUTPUT_DIR/pitch-toy-public-api.json"
    
    local count=$(jq length "$OUTPUT_DIR/pitch-toy-public-api.json")
    echo -e "${GREEN}✓${NC} Found $count public API items in pitch-toy"
}

# Extract public API from dev-console crate
extract_dev_console_public_api() {
    log "Extracting public API from dev-console crate..."
    
    # Find all pub items in dev-console
    rg --type rust --json "pub\s+(fn|struct|enum|trait|type|const|static|mod|use)" \
        dev-console/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 {
                   file: .data.path.text,
                   line: .data.line_number,
                   column: .data.submatches[0].start,
                   item_text: .data.lines.text,
                   item_type: (.data.lines.text | 
                     if test("pub fn") then "function"
                     elif test("pub struct") then "struct"
                     elif test("pub enum") then "enum"
                     elif test("pub trait") then "trait"
                     elif test("pub type") then "type"
                     elif test("pub const") then "const"
                     elif test("pub static") then "static"
                     elif test("pub mod") then "module"
                     elif test("pub use") then "re_export"
                     else "unknown"
                     end),
                   item_name: (.data.lines.text | 
                     if test("pub fn") then (. | match("pub fn\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub struct") then (. | match("pub struct\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub enum") then (. | match("pub enum\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub trait") then (. | match("pub trait\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub type") then (. | match("pub type\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub const") then (. | match("pub const\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub static") then (. | match("pub static\\s+(\\w+)"; "g") | .captures[0].string)
                     elif test("pub mod") then (. | match("pub mod\\s+(\\w+)"; "g") | .captures[0].string)
                     else "unknown"
                     end)
                 }' \
        | jq -s '.' > "$OUTPUT_DIR/dev-console-public-api.json"
    
    local count=$(jq length "$OUTPUT_DIR/dev-console-public-api.json")
    echo -e "${GREEN}✓${NC} Found $count public API items in dev-console"
}

# Analyze WASM exports
analyze_wasm_exports() {
    log "Analyzing WASM exports..."
    
    # Look for wasm-bindgen exports
    rg --type rust --json "#\[wasm_bindgen\]" \
        pitch-toy/ dev-console/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 {
                   file: .data.path.text,
                   line: .data.line_number,
                   context: .data.lines.text,
                   export_type: "wasm_bindgen"
                 }' \
        | jq -s '.' > "$OUTPUT_DIR/wasm-exports.json"
    
    # Look for extern "C" exports
    rg --type rust --json "extern\\s+\"C\"" \
        pitch-toy/ dev-console/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 {
                   file: .data.path.text,
                   line: .data.line_number,
                   context: .data.lines.text,
                   export_type: "extern_c"
                 }' \
        | jq -s '.' >> "$OUTPUT_DIR/wasm-exports.json"
    
    local count=$(jq length "$OUTPUT_DIR/wasm-exports.json")
    echo -e "${GREEN}✓${NC} Found $count WASM export declarations"
}

# Analyze console command usage
analyze_console_commands() {
    log "Analyzing console command usage..."
    
    # Find console command definitions and their usage of pitch-toy APIs
    rg --type rust --json "Command|command" \
        dev-console/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 {
                   file: .data.path.text,
                   line: .data.line_number,
                   context: .data.lines.text,
                   usage_type: "console_command"
                 }' \
        | jq -s '.' > "$OUTPUT_DIR/console-commands.json"
    
    # Find imports from pitch-toy in dev-console
    rg --type rust --json "use\\s+pitch_toy::" \
        dev-console/ 2>/dev/null \
        | jq -r 'select(.type == "match") | 
                 {
                   file: .data.path.text,
                   line: .data.line_number,
                   import_statement: .data.lines.text,
                   usage_type: "pitch_toy_import"
                 }' \
        | jq -s '.' > "$OUTPUT_DIR/pitch-toy-imports.json"
    
    local cmd_count=$(jq length "$OUTPUT_DIR/console-commands.json")
    local import_count=$(jq length "$OUTPUT_DIR/pitch-toy-imports.json")
    echo -e "${GREEN}✓${NC} Found $cmd_count console commands and $import_count pitch-toy imports"
}

# Analyze cross-crate usage patterns
analyze_cross_crate_usage() {
    log "Analyzing cross-crate usage patterns..."
    
    # Create comprehensive analysis
    cat > "$OUTPUT_DIR/analyze_usage.py" << 'EOF'
#!/usr/bin/env python3
import json
import re
from collections import defaultdict

def load_json_file(filepath):
    try:
        with open(filepath, 'r') as f:
            return json.load(f)
    except (json.JSONDecodeError, FileNotFoundError):
        return []

def extract_item_name(text):
    """Extract item name from pub declaration"""
    patterns = [
        r'pub fn\s+(\w+)',
        r'pub struct\s+(\w+)',
        r'pub enum\s+(\w+)', 
        r'pub trait\s+(\w+)',
        r'pub type\s+(\w+)',
        r'pub const\s+(\w+)',
        r'pub static\s+(\w+)',
        r'pub mod\s+(\w+)'
    ]
    
    for pattern in patterns:
        match = re.search(pattern, text)
        if match:
            return match.group(1)
    return "unknown"

def analyze_usage_patterns():
    # Load data
    pitch_toy_api = load_json_file('pitch-toy-public-api.json')
    dev_console_api = load_json_file('dev-console-public-api.json')
    wasm_exports = load_json_file('wasm-exports.json')
    console_commands = load_json_file('console-commands.json')
    pitch_toy_imports = load_json_file('pitch-toy-imports.json')
    
    # Analyze pitch-toy public API usage
    pitch_toy_usage = {}
    for item in pitch_toy_api:
        item_name = item.get('item_name', extract_item_name(item.get('item_text', '')))
        if item_name and item_name != "unknown":
            pitch_toy_usage[item_name] = {
                'file': item['file'],
                'line': item['line'],
                'type': item['item_type'],
                'text': item['item_text'],
                'used_externally': False,
                'used_in_wasm': False,
                'used_in_console': False,
                'usage_references': []
            }
    
    # Check WASM export usage
    wasm_context = '\n'.join([item.get('context', '') for item in wasm_exports])
    for api_name in pitch_toy_usage:
        if api_name in wasm_context:
            pitch_toy_usage[api_name]['used_in_wasm'] = True
            pitch_toy_usage[api_name]['used_externally'] = True
    
    # Check console command usage
    console_context = '\n'.join([item.get('context', '') for item in console_commands])
    import_context = '\n'.join([item.get('import_statement', '') for item in pitch_toy_imports])
    
    for api_name in pitch_toy_usage:
        if api_name in console_context or api_name in import_context:
            pitch_toy_usage[api_name]['used_in_console'] = True
            pitch_toy_usage[api_name]['used_externally'] = True
    
    # Identify unused public APIs
    unused_public_apis = []
    internal_only_apis = []
    critical_apis = []
    
    for api_name, usage_info in pitch_toy_usage.items():
        if not usage_info['used_externally']:
            if usage_info['type'] in ['function', 'struct', 'enum', 'trait']:
                unused_public_apis.append({
                    'name': api_name,
                    'file': usage_info['file'],
                    'line': usage_info['line'],
                    'type': usage_info['type'],
                    'recommendation': 'Consider making private or removing if truly unused'
                })
            else:
                internal_only_apis.append({
                    'name': api_name,
                    'file': usage_info['file'],
                    'line': usage_info['line'],
                    'type': usage_info['type'],
                    'recommendation': 'Review if public visibility is needed'
                })
        elif usage_info['used_in_wasm'] or usage_info['used_in_console']:
            critical_apis.append({
                'name': api_name,
                'file': usage_info['file'],
                'line': usage_info['line'],
                'type': usage_info['type'],
                'wasm_usage': usage_info['used_in_wasm'],
                'console_usage': usage_info['used_in_console'],
                'recommendation': 'PRESERVE - Essential for external interfaces'
            })
    
    # Generate analysis report
    analysis = {
        'summary': {
            'total_public_apis': len(pitch_toy_usage),
            'unused_public_apis': len(unused_public_apis),
            'internal_only_apis': len(internal_only_apis),
            'critical_apis': len(critical_apis),
            'wasm_exports_count': len(wasm_exports),
            'console_commands_count': len(console_commands),
            'pitch_toy_imports_count': len(pitch_toy_imports)
        },
        'unused_public_apis': unused_public_apis,
        'internal_only_apis': internal_only_apis,
        'critical_apis': critical_apis,
        'all_api_usage': pitch_toy_usage
    }
    
    return analysis

# Run analysis
analysis = analyze_usage_patterns()

# Save results
with open('cross-crate-analysis.json', 'w') as f:
    json.dump(analysis, f, indent=2)

# Generate summary report
with open('cross-crate-summary.txt', 'w') as f:
    f.write("Cross-Crate Usage Analysis Report\n")
    f.write("=" * 50 + "\n\n")
    
    summary = analysis['summary']
    f.write(f"Total public APIs in pitch-toy: {summary['total_public_apis']}\n")
    f.write(f"Unused public APIs: {summary['unused_public_apis']}\n")
    f.write(f"Internal-only APIs: {summary['internal_only_apis']}\n")
    f.write(f"Critical APIs (WASM/Console): {summary['critical_apis']}\n")
    f.write(f"WASM exports found: {summary['wasm_exports_count']}\n")
    f.write(f"Console commands found: {summary['console_commands_count']}\n")
    f.write(f"pitch-toy imports in dev-console: {summary['pitch_toy_imports_count']}\n\n")
    
    if analysis['unused_public_apis']:
        f.write("UNUSED PUBLIC APIs (candidates for removal/privatization):\n")
        f.write("-" * 60 + "\n")
        for api in analysis['unused_public_apis']:
            f.write(f"- {api['name']} ({api['type']}) in {api['file']}:{api['line']}\n")
            f.write(f"  Recommendation: {api['recommendation']}\n\n")
    
    if analysis['critical_apis']:
        f.write("CRITICAL APIs (MUST PRESERVE):\n")
        f.write("-" * 30 + "\n")
        for api in analysis['critical_apis']:
            usage_types = []
            if api['wasm_usage']:
                usage_types.append("WASM")
            if api['console_usage']:
                usage_types.append("Console")
            f.write(f"- {api['name']} ({api['type']}) - Used by: {', '.join(usage_types)}\n")
            f.write(f"  File: {api['file']}:{api['line']}\n\n")
    
    f.write("RECOMMENDATIONS:\n")
    f.write("-" * 20 + "\n")
    f.write("1. Review unused public APIs for potential privatization\n")
    f.write("2. Preserve all critical APIs used by WASM exports or console commands\n")
    f.write("3. Consider consolidating similar internal-only APIs\n")
    f.write("4. Use #[doc(hidden)] for APIs that must be public but are internal\n")
    f.write("5. Regular cross-crate analysis to prevent API bloat\n")

print(f"Cross-crate analysis complete!")
print(f"Found {summary['total_public_apis']} public APIs")
print(f"  - {summary['unused_public_apis']} unused public APIs")
print(f"  - {summary['critical_apis']} critical APIs")
print(f"  - {summary['internal_only_apis']} internal-only APIs")
EOF

    python3 "$OUTPUT_DIR/analyze_usage.py"
    
    echo -e "${GREEN}✓${NC} Cross-crate analysis complete"
}

# Generate final report
generate_final_report() {
    log "Generating final cross-crate analysis report..."
    
    # Move reports to project root
    cp "$OUTPUT_DIR/cross-crate-analysis.json" "$PROJECT_ROOT/"
    cp "$OUTPUT_DIR/cross-crate-summary.txt" "$PROJECT_ROOT/"
    
    echo -e "${GREEN}✓${NC} Reports saved to project root"
}

# Display summary
display_summary() {
    echo
    echo -e "${YELLOW}📊 CROSS-CRATE ANALYSIS SUMMARY${NC}"
    echo "===================================="
    
    if [[ -f "$PROJECT_ROOT/cross-crate-analysis.json" ]]; then
        local total_apis=$(jq '.summary.total_public_apis' "$PROJECT_ROOT/cross-crate-analysis.json")
        local unused_apis=$(jq '.summary.unused_public_apis' "$PROJECT_ROOT/cross-crate-analysis.json")
        local critical_apis=$(jq '.summary.critical_apis' "$PROJECT_ROOT/cross-crate-analysis.json")
        
        echo "Total public APIs analyzed: $total_apis"
        echo "  - Unused public APIs: $unused_apis"
        echo "  - Critical APIs (WASM/Console): $critical_apis"
        echo
        
        if [ "$unused_apis" -gt 0 ]; then
            echo -e "${YELLOW}⚠️  $unused_apis unused public APIs found - consider privatization${NC}"
        fi
        
        if [ "$critical_apis" -gt 0 ]; then
            echo -e "${RED}🔒 $critical_apis critical APIs identified - MUST PRESERVE${NC}"
        fi
        
        echo
        echo "Reports generated:"
        echo "  - cross-crate-analysis.json (detailed analysis)"
        echo "  - cross-crate-summary.txt (human-readable summary)"
        echo
        echo "Next steps:"
        echo "1. Review unused public APIs for privatization opportunities"
        echo "2. Ensure critical APIs are preserved during dead code cleanup"
        echo "3. Use findings to guide safe dead code removal"
    else
        echo -e "${RED}Error: Analysis report not generated${NC}"
    fi
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
    
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}Error: python3 is required but not installed${NC}"
        exit 1
    fi
    
    # Run analysis steps
    extract_pitch_toy_public_api
    extract_dev_console_public_api
    analyze_wasm_exports
    analyze_console_commands
    analyze_cross_crate_usage
    generate_final_report
    display_summary
    
    echo -e "${GREEN}🎉 Cross-crate analysis complete!${NC}"
}

# Run main function
main "$@"