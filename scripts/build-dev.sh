#!/bin/bash
# 🛠️ Development Build Script
# Builds pitch-toy for development with full debugging capabilities

set -e

echo "🛠️  Starting Development Build..."
echo "📁 Build Config: build-configs/development.toml"

# Clean previous build
echo "🧹 Cleaning previous development build..."
rm -rf dist/development
mkdir -p dist/development

# Build WASM with development profile
echo "🦀 Building Rust/WASM (development profile)..."
CARGO_PROFILE_DEV_DEBUG=true \
CARGO_PROFILE_DEV_OPT_LEVEL=0 \
wasm-pack build \
  --target web \
  --out-dir dist/development \
  --out-name pitch_toy_dev \
  --dev \
  --features "full-features"

# Copy and process JavaScript files
echo "📄 Processing JavaScript files..."
cp web/*.js dist/development/
cp web/*.html dist/development/
cp web/*.css dist/development/ 2>/dev/null || true

# Generate development-specific files
echo "🔧 Generating development configuration..."
cat > dist/development/build-info.js << EOF
// Development Build Information
window.BUILD_INFO = {
  target: 'development',
  timestamp: '$(date -u +"%Y-%m-%dT%H:%M:%SZ")',
  profile: 'dev',
  features: ['full-features', 'debug-features'],
  debug: true,
  optimization: 'none',
  sourceMap: true
};
console.log('🛠️ Development build loaded:', window.BUILD_INFO);
EOF

# Create development server script
cat > dist/development/serve.sh << 'EOF'
#!/bin/bash
echo "🚀 Starting development server..."
echo "📍 Serving from: $(pwd)"
echo "🌐 Access at: http://localhost:8080"
python3 -m http.server 8080 || python -m SimpleHTTPServer 8080
EOF
chmod +x dist/development/serve.sh

# Generate source maps for debugging
echo "🗺️  Generating source maps..."
# Note: wasm-pack automatically generates source maps in dev mode

echo "✅ Development build complete!"
echo "📂 Output directory: dist/development"
echo "🚀 To serve: cd dist/development && ./serve.sh"
echo "🔍 Features enabled: full-features, debug-features"
echo "📊 Build size:"
du -sh dist/development/ 