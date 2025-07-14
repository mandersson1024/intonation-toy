#!/bin/bash
set -e

echo "🧪 Running all tests with wasm-pack test --headless --chrome..."

# Test all packages in the workspace
packages=("pitch-toy" "observable-data" "event-dispatcher" "egui-dev-console")

for package in "${packages[@]}"; do
    echo "🔍 Testing $package..."
    wasm-pack test --headless --chrome "$package"
    echo "✅ $package tests passed"
    echo ""
done

echo "🎉 All tests completed successfully!"