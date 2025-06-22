#!/bin/bash

# Yew Development Build Script
# Integrates with existing pitch-toy development infrastructure

set -e

echo "🦀 Building Yew application (Development Mode)"
echo "================================================"

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "❌ trunk is not installed. Installing..."
    cargo install trunk
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
trunk clean

# Build for development with debug symbols and source maps
echo "🔨 Building Yew app with debug symbols..."
trunk build --dev

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Yew development build completed successfully!"
    echo "📦 Output directory: dist/"
    echo "🌐 To serve: trunk serve --port 8080"
    
    # List build artifacts
    echo ""
    echo "📋 Build artifacts:"
    ls -la dist/
else
    echo "❌ Yew build failed!"
    exit 1
fi 