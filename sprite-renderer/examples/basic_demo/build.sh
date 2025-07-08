#!/bin/bash

# Build and serve script for basic demo

set -e

echo "Basic Sprite Renderer Demo"
echo "========================="
echo ""

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "❌ Trunk is not installed!"
    echo "Install with: cargo install trunk"
    echo "Or visit: https://trunkrs.dev/#install"
    exit 1
fi

echo "Available commands:"
echo "1. Build only:     ./build.sh build"
echo "2. Serve with hot reload: ./build.sh serve"
echo "3. Build for release: ./build.sh release"
echo ""

case "${1:-serve}" in
    "build")
        echo "🔨 Building demo..."
        trunk build
        echo "✅ Demo built in ./dist/"
        echo "💡 Serve with: trunk serve or python3 -m http.server -d dist 8080"
        ;;
    "serve")
        echo "🚀 Starting development server with hot reload..."
        echo "📂 Demo will open at http://localhost:8080"
        trunk serve
        ;;
    "release")
        echo "🔨 Building optimized release..."
        trunk build --release
        echo "✅ Release build completed in ./dist/"
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo "Use: build, serve, or release"
        exit 1
        ;;
esac