#!/bin/bash
set -e

MODE=$1  # dev, release

if ! command -v trunk &> /dev/null; then
    echo "trunk is not installed. Installing..."
    cargo install trunk
fi

case "$MODE" in
  dev)
    echo "Building for development (profile.dev)..."
    trunk serve
    ;;
  release)
    echo "Building for production (profile.release)..."
    trunk serve --release
    ;;
  *)
    echo "Usage: $0 [dev|release]"
    exit 1
    ;;
esac
