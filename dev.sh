#!/bin/bash

# Development script for pitch-toy
# Builds WASM and starts development server

set -e

# Standard development port
DEV_PORT=8080

echo "🦀 Building WASM package..."
wasm-pack build --target web --out-dir pkg

if [ $? -eq 0 ]; then
    echo "✅ WASM build successful!"
    echo ""
    echo "🚀 Starting development server on port ${DEV_PORT}..."
    echo "📝 Demo will be available at: http://localhost:${DEV_PORT}/web/"
    echo ""
    ruby serve.rb ${DEV_PORT}
else
    echo "❌ WASM build failed!"
    exit 1
fi 