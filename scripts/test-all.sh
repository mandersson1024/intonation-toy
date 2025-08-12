#!/bin/bash
set -e

# Parse command line arguments
MODE=${1:-"both"}

echo "🧪 Running tests in mode: $MODE"

# Test all packages in the workspace
packages=("intonation-toy" "dev-console")

if [[ "$MODE" == "native" || "$MODE" == "both" ]]; then
    echo "🔍 Running native tests..."
    for package in "${packages[@]}"; do
        echo "🔍 Testing $package (native)..."
        cargo test --manifest-path "$package/Cargo.toml" --features test-native
        echo "✅ $package native tests passed"
    done
    echo ""
fi

if [[ "$MODE" == "web" || "$MODE" == "both" ]]; then
    echo "🔍 Running web tests..."
    for package in "${packages[@]}"; do
        echo "🔍 Testing $package (web)..."
        wasm-pack test --node "$package"
        echo "✅ $package web tests passed"
    done
    echo ""
fi

echo "🎉 All tests completed successfully!"

# Usage information
if [[ "$MODE" != "native" && "$MODE" != "web" && "$MODE" != "both" ]]; then
    echo ""
    echo "Usage: $0 [native|web|both]"
    echo "  native - Run only native tests with cargo test"
    echo "  web    - Run only web tests with wasm-pack test"  
    echo "  both   - Run both native and web tests (default)"
fi