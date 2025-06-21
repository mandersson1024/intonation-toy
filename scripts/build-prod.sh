#!/bin/bash
# 🚀 Production Build Script
# Builds pitch-toy for production with maximum optimization

set -e

echo "🚀 Starting Production Build..."
echo "📁 Build Config: build-configs/production.toml"

# Clean previous build
echo "🧹 Cleaning previous production build..."
rm -rf dist/production
mkdir -p dist/production

# Build WASM with release profile
echo "🦀 Building Rust/WASM (release profile)..."
CARGO_PROFILE_RELEASE_LTO=fat \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
CARGO_PROFILE_RELEASE_PANIC=abort \
wasm-pack build \
  --target web \
  --out-dir dist/production \
  --out-name pitch_toy \
  --release \
  --features "basic-features"

# Optimize WASM further with wasm-opt
echo "⚡ Optimizing WASM with wasm-opt..."
if command -v wasm-opt &> /dev/null; then
    wasm-opt -Oz --strip-debug --strip-producers \
        dist/production/pitch_toy_bg.wasm \
        -o dist/production/pitch_toy_bg.wasm
    echo "✅ WASM optimization complete"
else
    echo "⚠️  wasm-opt not found, skipping additional optimization"
fi

# Process and minify JavaScript files
echo "📄 Processing and minifying JavaScript files..."
cp web/*.html dist/production/
cp web/*.css dist/production/ 2>/dev/null || true

# Minify JavaScript files (basic minification)
for js_file in web/*.js; do
    filename=$(basename "$js_file")
    # Simple minification - remove comments and extra whitespace
    sed -e '/^[[:space:]]*\/\//d' -e '/^[[:space:]]*\/\*/,/\*\//d' \
        -e 's/[[:space:]]*\/\/.*$//' -e '/^[[:space:]]*$/d' \
        "$js_file" > "dist/production/$filename"
done

# Generate production-specific files
echo "🔧 Generating production configuration..."
cat > dist/production/build-info.js << EOF
// Production Build Information
window.BUILD_INFO = {
  target: 'production',
  timestamp: '$(date -u +"%Y-%m-%dT%H:%M:%SZ")',
  profile: 'release',
  features: ['basic-features'],
  debug: false,
  optimization: 'maximum',
  sourceMap: false
};
EOF

# Create production deployment script
cat > dist/production/deploy.sh << 'EOF'
#!/bin/bash
echo "🚀 Production Deployment Script"
echo "📦 Compressing assets..."

# Compress with gzip
find . -type f \( -name "*.js" -o -name "*.wasm" -o -name "*.html" -o -name "*.css" \) \
    -exec gzip -9 -k {} \;

echo "✅ Assets compressed"
echo "📊 Deployment size:"
du -sh .
echo "📋 Files ready for deployment:"
ls -la *.gz
EOF
chmod +x dist/production/deploy.sh

# Generate integrity hashes for security
echo "🔒 Generating integrity hashes..."
cd dist/production
for file in *.js *.wasm; do
    if [ -f "$file" ]; then
        hash=$(openssl dgst -sha384 -binary "$file" | openssl base64 -A)
        echo "sha384-$hash" > "$file.integrity"
    fi
done
cd ../..

echo "✅ Production build complete!"
echo "📂 Output directory: dist/production"
echo "🚀 To deploy: cd dist/production && ./deploy.sh"
echo "🔍 Features enabled: basic-features only"
echo "📊 Build size:"
du -sh dist/production/
echo "📊 Individual file sizes:"
ls -lh dist/production/*.{js,wasm} 2>/dev/null || true 