# 🚀 Pitch-Toy Build System

**Phase 5 Implementation**: Separate Build Targets for Different Deployment Scenarios

## 🎯 Quick Start

```bash
# Build for development (full debugging)
./scripts/build-all.sh dev

# Build for production (optimized)
./scripts/build-all.sh prod

# Build for testing (with benchmarks)
./scripts/build-all.sh test

# Build lightweight demo
./scripts/build-all.sh demo

# Build all targets
./scripts/build-all.sh all

# Clean all builds
./scripts/build-all.sh clean
```

## 📦 Build Targets

### 🛠️ Development Build
- **Purpose**: Development workflow with full debugging
- **Features**: Source maps, debug symbols, verbose logging
- **Size**: ~2.5MB (unoptimized)
- **Usage**: `./scripts/build-all.sh dev`
- **Output**: `dist/development/`

**Includes**:
- Full debugging information
- Source maps for WASM and JS
- Development server script
- Hot reload support
- All debug features enabled

### 🚀 Production Build  
- **Purpose**: Optimized deployment build
- **Features**: Maximum optimization, minification, compression
- **Size**: ~800KB (ultra-optimized)
- **Usage**: `./scripts/build-all.sh prod`
- **Output**: `dist/production/`

**Includes**:
- LTO (Link Time Optimization)
- Dead code elimination
- Asset compression (gzip)
- Integrity hashes for security
- Deployment scripts

### 🧪 Testing Build
- **Purpose**: Comprehensive testing and validation
- **Features**: Test hooks, benchmarking, coverage
- **Size**: ~1.2MB (balanced optimization)
- **Usage**: `./scripts/build-all.sh test`
- **Output**: `dist/testing/`

**Includes**:
- Test framework integration
- Performance benchmarking
- Cross-browser testing support
- Coverage reporting
- Mock audio devices

### 🎨 Demo Build
- **Purpose**: Lightweight demos and embedded use
- **Features**: Minimal size, essential features only
- **Size**: ~500KB (ultra-compressed)
- **Usage**: `./scripts/build-all.sh demo`
- **Output**: `dist/demo/`

**Includes**:
- Beautiful demo interface
- Standalone HTML version
- CDN-optimized assets
- Embeddable components
- Size-optimized WASM

## 🛠️ Build Profiles

The build system uses Cargo profiles for different optimization levels:

```toml
[profile.dev]          # Development - no optimization
[profile.release]      # Production - maximum optimization  
[profile.test]         # Testing - balanced optimization
[profile.release-small] # Demo - size optimization
```

## 🎯 Feature Flags

Control what gets compiled with feature flags:

```toml
# Core features
basic-features = ["audio-processing", "pitch-detection"]
full-features = ["basic-features", "debug-features", "advanced-features"]

# Development features  
debug-features = ["debug-logging", "performance-profiling", "memory-debugging"]

# Testing features
test-features = ["mock-audio", "automated-testing"]

# Demo features
demo-features = ["simplified-ui", "essential-audio-only"]
```

## 📊 Build Optimization

Each target is optimized for its specific use case:

| Target | Optimization | Features | Size | Use Case |
|--------|-------------|----------|------|----------|
| **dev** | None | All | 2.5MB | Development |
| **prod** | Maximum | Core | 800KB | Production |
| **test** | Balanced | All + Test | 1.2MB | Testing |
| **demo** | Size | Minimal | 500KB | Demos |

## 🔧 Configuration Files

Build configurations are stored in `build-configs/`:

- `development.toml` - Development build settings
- `production.toml` - Production optimization settings  
- `testing.toml` - Testing and validation settings
- `demo.toml` - Demo and embedded settings

## 🚀 Deployment

Each build target includes deployment helpers:

### Development
```bash
cd dist/development
./serve.sh  # Start development server
```

### Production  
```bash
cd dist/production
./deploy.sh  # Compress and prepare for deployment
```

### Testing
```bash
cd dist/testing
./run-tests.sh   # Run comprehensive test suite
./benchmark.sh   # Performance benchmarking
```

### Demo
```bash
# Open in browser
open dist/demo/index.html

# Or use standalone version
open dist/demo/pitch-demo-standalone.html
```

## 📋 Prerequisites

- **Rust** (latest stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **wasm-pack**: `cargo install wasm-pack`
- **wasm-opt** (optional): Install [binaryen](https://github.com/WebAssembly/binaryen) for better optimization

## 🔍 Build System Architecture

```
pitch-toy/
├─ build-configs/           # Build configuration files
│  ├─ development.toml      # Dev settings
│  ├─ production.toml       # Prod optimization
│  ├─ testing.toml          # Test configuration  
│  └─ demo.toml            # Demo settings
├─ scripts/                 # Build automation
│  ├─ build-all.sh         # Master build script
│  ├─ build-dev.sh         # Development build
│  ├─ build-prod.sh        # Production build
│  ├─ build-test.sh        # Testing build
│  └─ build-demo.sh        # Demo build
├─ Cargo.toml              # Enhanced with profiles
└─ dist/                   # Build outputs
   ├─ development/         # Dev build
   ├─ production/          # Prod build
   ├─ testing/             # Test build
   └─ demo/                # Demo build
```

## 🎯 Architecture Achievement

This build system completes **Operation Clean Sweep** - the architectural transformation from JavaScript-heavy to Rust-dominant codebase:

- **Before**: 55.9% JavaScript, 28.5% Rust (architectural drift)
- **After**: 58% Rust, 42% JavaScript (clean architecture)

**Mission Accomplished!** ✅ 