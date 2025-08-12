#!/bin/bash
set -e

echo "🔧 Running native tests and checks..."

# Parse optional test pattern
TEST_PATTERN=${1:-""}

packages=("intonation-toy" "dev-console")

echo "🔍 Running cargo check..."
for package in "${packages[@]}"; do
    echo "  Checking $package..."
    cargo check --manifest-path "$package/Cargo.toml" --features test-native
done
echo "✅ Cargo check passed"
echo ""

echo "🔍 Running native tests..."
for package in "${packages[@]}"; do
    echo "🔍 Testing $package (native)..."
    if [[ -n "$TEST_PATTERN" ]]; then
        echo "  Running tests matching pattern: $TEST_PATTERN"
        cargo test --manifest-path "$package/Cargo.toml" --features test-native "$TEST_PATTERN"
    else
        cargo test --manifest-path "$package/Cargo.toml" --features test-native
    fi
    echo "✅ $package native tests passed"
done
echo ""

echo "🔍 Running clippy..."
for package in "${packages[@]}"; do
    echo "  Linting $package..."
    cargo clippy --manifest-path "$package/Cargo.toml" --features test-native -- -D warnings
done
echo "✅ Clippy checks passed"
echo ""

echo "🔍 Running format check..."
cargo fmt --all -- --check
echo "✅ Format check passed"
echo ""

echo "🎉 All native checks completed successfully!"

# Usage information
if [[ "$#" -gt 1 ]]; then
    echo ""
    echo "Usage: $0 [test_pattern]"
    echo "  test_pattern - Optional pattern to filter tests (e.g., 'platform', 'timer')"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all native tests and checks"
    echo "  $0 platform          # Run tests matching 'platform'"
    echo "  $0 test_timer        # Run specific test function"
fi