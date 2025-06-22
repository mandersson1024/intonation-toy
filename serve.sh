#!/bin/bash

set -e

PORT=8080

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "trunk is not installed. Installing..."
    cargo install trunk
fi

echo "Starting server on port ${PORT}..."
echo "App will be available at: http://localhost:${PORT}/"
echo "Hot reload is enabled - changes will auto-refresh"
echo ""
trunk serve --port ${PORT}
