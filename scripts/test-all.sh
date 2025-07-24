#!/bin/bash
set -e

echo "🧪 Running all tests with wasm-pack test --node..."

# Test all packages in the workspace
packages=("pitch-toy" "dev-console")

for package in "${packages[@]}"; do
    echo "🔍 Testing $package..."
    wasm-pack test --node "$package"
    echo "✅ $package tests passed"
    echo ""
done

echo "🎉 All tests completed successfully!"