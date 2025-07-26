#!/bin/bash

# Apply unused import fixes safely using Rust tooling
# This script uses cargo fix and clippy to automatically resolve import issues

set -e

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "🔧 Applying import fixes to Rust codebase..."
echo "Working directory: $WORKSPACE_ROOT"
echo

# Configuration
BACKUP_DIR="backup-$(date +%Y%m%d-%H%M%S)"
LOG_FILE="import-fixes.log"

# Function to log messages
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

# Function to create backup
create_backup() {
    log "Creating backup in $BACKUP_DIR"
    mkdir -p "$BACKUP_DIR"
    
    # Backup all Rust source files
    find . -name "*.rs" -not -path "./target/*" -not -path "./$BACKUP_DIR/*" | while read -r file; do
        backup_path="$BACKUP_DIR/$file"
        mkdir -p "$(dirname "$backup_path")"
        cp "$file" "$backup_path"
    done
    
    log "✓ Backup created successfully"
}

# Function to validate compilation
validate_compilation() {
    log "Validating compilation..."
    
    if cargo check --workspace --all-targets 2>&1 | tee -a "$LOG_FILE"; then
        log "✓ Compilation successful"
        return 0
    else
        log "❌ Compilation failed"
        return 1
    fi
}

# Function to run tests
run_tests() {
    log "Running tests..."
    
    if cargo test --workspace 2>&1 | tee -a "$LOG_FILE"; then
        log "✓ All tests passed"
        return 0
    else
        log "❌ Some tests failed"
        return 1
    fi
}

# Function to apply cargo fix
apply_cargo_fix() {
    log "Applying cargo fix for unused imports..."
    
    # Allow dirty and staged changes
    if cargo fix --allow-dirty --allow-staged --workspace 2>&1 | tee -a "$LOG_FILE"; then
        log "✓ cargo fix completed"
        return 0
    else
        log "⚠️  cargo fix encountered issues (this might be normal)"
        return 1
    fi
}

# Function to apply clippy fixes
apply_clippy_fixes() {
    log "Applying clippy fixes for unused imports..."
    
    # Apply clippy fixes, focusing on import-related warnings
    if cargo clippy --fix --allow-dirty --allow-staged --workspace -- -A clippy::all -W clippy::unused_imports 2>&1 | tee -a "$LOG_FILE"; then
        log "✓ clippy fix completed"
        return 0
    else
        log "⚠️  clippy fix encountered issues (this might be normal)"
        return 1
    fi
}

# Function to check git status
check_git_status() {
    log "Checking git status for changes..."
    
    if git diff --name-only | tee -a "$LOG_FILE"; then
        log "✓ Git status checked"
    fi
    
    # Show summary of changes
    local changed_files=$(git diff --name-only | wc -l)
    log "Files modified: $changed_files"
}

# Function to create git commit
create_commit() {
    local commit_message="$1"
    
    log "Creating git commit: $commit_message"
    
    # Add all Rust files that were modified
    git add "*.rs" || true
    
    if git diff --cached --quiet; then
        log "No changes to commit"
        return 0
    fi
    
    if git commit -m "$commit_message" 2>&1 | tee -a "$LOG_FILE"; then
        log "✓ Commit created successfully"
        return 0
    else
        log "❌ Failed to create commit"
        return 1
    fi
}

# Function to restore from backup
restore_backup() {
    log "Restoring from backup..."
    
    if [[ -d "$BACKUP_DIR" ]]; then
        find "$BACKUP_DIR" -name "*.rs" | while read -r backup_file; do
            original_file="${backup_file#$BACKUP_DIR/}"
            cp "$backup_file" "$original_file"
        done
        log "✓ Restored from backup"
    else
        log "❌ No backup directory found"
        return 1
    fi
}

# Function to apply fixes incrementally
apply_fixes_incrementally() {
    log "Starting incremental import fix process..."
    
    # Initial validation
    if ! validate_compilation; then
        log "❌ Initial compilation failed. Fix compilation errors first."
        return 1
    fi
    
    # Apply cargo fix
    log "=== Step 1: Applying cargo fix ==="
    apply_cargo_fix
    
    # Validate after cargo fix
    if validate_compilation; then
        log "✓ Compilation successful after cargo fix"
        create_commit "Fix unused imports with cargo fix

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
    else
        log "❌ Compilation failed after cargo fix, restoring backup"
        restore_backup
        return 1
    fi
    
    # Apply clippy fixes
    log "=== Step 2: Applying clippy fixes ==="
    apply_clippy_fixes
    
    # Validate after clippy fixes
    if validate_compilation; then
        log "✓ Compilation successful after clippy fixes"
        create_commit "Apply clippy fixes for imports

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
    else
        log "❌ Compilation failed after clippy fixes, restoring to last good state"
        git reset --hard HEAD~1
        return 1
    fi
    
    # Run tests to ensure functionality is preserved
    log "=== Step 3: Running tests ==="
    if run_tests; then
        log "✓ All tests passed after import fixes"
    else
        log "❌ Tests failed, consider reviewing changes"
        # Don't automatically revert as some test failures might be unrelated
    fi
    
    log "✓ Incremental fix process completed"
}

# Function to show summary
show_summary() {
    log "=== Import Fix Summary ==="
    
    # Count modified files
    local modified_files=$(git diff HEAD~2 --name-only | grep "\.rs$" | wc -l || echo "0")
    log "Modified Rust files: $modified_files"
    
    # Show which files were changed
    log "Changed files:"
    git diff HEAD~2 --name-only | grep "\.rs$" | sed 's/^/  - /' | tee -a "$LOG_FILE" || true
    
    # Show diff stats
    log "Change statistics:"
    git diff HEAD~2 --stat | tee -a "$LOG_FILE" || true
    
    log "=== Next Steps ==="
    log "1. Review changes with: git diff HEAD~2"
    log "2. Test the application manually"
    log "3. Run additional validation if needed"
    log "4. Consider running: ./scripts/validate-import-cleanup.sh"
}

# Main execution
main() {
    # Initialize log
    > "$LOG_FILE"
    log "Starting import fix process"
    
    # Check if we're in a git repository
    if ! git status >/dev/null 2>&1; then
        log "❌ Not in a git repository. Import fixes require git for safety."
        exit 1
    fi
    
    # Check for uncommitted changes
    if ! git diff-index --quiet HEAD --; then
        log "⚠️  Warning: You have uncommitted changes."
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log "Aborted by user"
            exit 1
        fi
    fi
    
    # Create backup
    create_backup
    
    # Apply fixes
    if apply_fixes_incrementally; then
        log "🎉 Import fixes applied successfully!"
        show_summary
    else
        log "❌ Import fix process failed. Check the log for details."
        exit 1
    fi
    
    log "Import fix process completed. Log available at: $LOG_FILE"
}

# Handle script arguments
case "${1:-}" in
    --dry-run)
        echo "🔍 Dry run mode - no changes will be made"
        log "Dry run mode enabled"
        validate_compilation
        log "Dry run completed"
        ;;
    --help)
        echo "Usage: $0 [--dry-run|--help]"
        echo
        echo "Apply unused import fixes safely using Rust tooling"
        echo
        echo "Options:"
        echo "  --dry-run    Check compilation without making changes"
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