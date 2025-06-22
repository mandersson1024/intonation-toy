#!/bin/bash

# Yew Production Build Script
# Optimized builds for deployment

set -e

echo "🦀 Building Yew application (Production Mode)"
echo "=============================================="

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "❌ trunk is not installed. Installing..."
    cargo install trunk
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
trunk clean

# Build for production with optimizations
echo "🔨 Building optimized Yew app for production..."
trunk build --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Yew production build completed successfully!"
    echo "📦 Output directory: dist/"
    
    # Show file sizes for optimization verification
    echo ""
    echo "📊 Build artifact sizes:"
    find dist/ -name "*.wasm" -exec ls -lh {} \;
    find dist/ -name "*.js" -exec ls -lh {} \;
    
    echo ""
    echo "🚀 Production build ready for deployment!"
    echo "📁 Deploy the contents of the 'dist/' directory"
else
    echo "❌ Yew production build failed!"
    exit 1
fi 