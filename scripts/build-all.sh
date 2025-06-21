#!/bin/bash
# 🚀 Master Build Script for Pitch-Toy
# Builds all targets or specific ones based on Phase 5 configuration

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🏗️  PITCH-TOY MASTER BUILD SYSTEM"
echo "📁 Project Root: $PROJECT_ROOT"
echo "🚀 Phase 5: Separate Build Targets"
echo ""

# Available build targets
TARGETS=("dev" "prod")

# Function to show usage
show_usage() {
    echo "Usage: $0 [target1] [target2] ... | all | clean | help"
    echo ""
    echo "Available targets:"
    echo "  dev     🛠️  Development build (full debugging)"
    echo "  prod    🚀 Production build (maximum optimization)"
    echo ""
    echo "Special commands:"
    echo "  all     Build all targets"
    echo "  clean   Clean all build directories"
    echo "  help    Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 dev          # Build development only"
    echo "  $0 prod demo    # Build production and demo"
    echo "  $0 all          # Build all targets"
    echo "  $0 clean        # Clean all builds"
}

# Function to clean all builds
clean_all() {
    echo "🧹 Cleaning all build directories..."
    rm -rf dist/
    echo "✅ All build directories cleaned"
}

# Function to build specific target
build_target() {
    local target=$1
    case $target in
        dev)
            echo "🛠️  Building Development Target..."
            "$SCRIPT_DIR/build-dev.sh"
            ;;
        prod)
            echo "🚀 Building Production Target..."
            "$SCRIPT_DIR/build-prod.sh"
            ;;

        demo)
            echo "🎨 Building Demo Target..."
            "$SCRIPT_DIR/build-demo.sh"
            ;;
        *)
            echo "❌ Unknown target: $target"
            echo "Available targets: ${TARGETS[*]}"
            return 1
            ;;
    esac
}

# Function to build all targets
build_all() {
    echo "🚀 Building ALL targets..."
    local start_time=$(date +%s)
    
    for target in "${TARGETS[@]}"; do
        echo ""
        echo "=================================================="
        echo "Building target: $target"
        echo "=================================================="
        build_target "$target"
    done
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo ""
    echo "🎉 ALL BUILDS COMPLETE!"
    echo "⏱️  Total build time: ${duration}s"
    echo "📊 Build summary:"
    
    for target in "${TARGETS[@]}"; do
        if [ -d "dist/$target" ]; then
            size=$(du -sh "dist/$target" | cut -f1)
            echo "  $target: $size"
        fi
    done
}

# Function to validate prerequisites
check_prerequisites() {
    echo "🔍 Checking prerequisites..."
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        echo "❌ Rust/Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Check for wasm-pack
    if ! command -v wasm-pack &> /dev/null; then
        echo "❌ wasm-pack not found. Please install: cargo install wasm-pack"
        exit 1
    fi
    
    # Check for optional tools
    if command -v wasm-opt &> /dev/null; then
        echo "✅ wasm-opt found (optimization available)"
    else
        echo "⚠️  wasm-opt not found (install binaryen for better optimization)"
    fi
    
    echo "✅ Prerequisites check complete"
}

# Main execution
cd "$PROJECT_ROOT"

# Handle arguments
if [ $# -eq 0 ]; then
    show_usage
    exit 0
fi

case $1 in
    help|--help|-h)
        show_usage
        exit 0
        ;;
    clean)
        clean_all
        exit 0
        ;;
    all)
        check_prerequisites
        build_all
        exit 0
        ;;
    *)
        check_prerequisites
        
        # Build specified targets
        for target in "$@"; do
            if [[ " ${TARGETS[*]} " == *" $target "* ]]; then
                echo ""
                echo "=================================================="
                echo "Building target: $target"
                echo "=================================================="
                build_target "$target"
            else
                echo "❌ Unknown target: $target"
                echo "Available targets: ${TARGETS[*]}"
                exit 1
            fi
        done
        ;;
esac

echo ""
echo "🎉 Build process complete!"
echo "📂 Output directories:"
for target in "$@"; do
    if [[ " ${TARGETS[*]} " == *" $target "* ]] && [ -d "dist/$target" ]; then
        echo "  dist/$target/"
    fi
done 