#!/bin/bash

# Development script for pitch-toy
# Now defaults to Yew development mode

set -e

# Standard development port
DEV_PORT=8080

echo "🦀 Starting Yew development mode..."
echo "=================================="

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "❌ trunk is not installed. Installing..."
    cargo install trunk
fi

echo "🚀 Starting Yew development server on port ${DEV_PORT}..."
echo "📝 Yew app will be available at: http://localhost:${DEV_PORT}/"
echo "🔄 Hot reload is enabled - changes will auto-refresh"
echo ""
trunk serve --port ${DEV_PORT} 