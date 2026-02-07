#!/bin/bash
set -e

# Configuration
OV_VERSION="2024.5.0.0"
OV_SHORT_VER="2024.5"
OV_URL="https://storage.openvinotoolkit.org/repositories/openvino_genai/packages/${OV_SHORT_VER}/linux/openvino_genai_ubuntu22_${OV_VERSION}_x86_64.tar.gz"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LIBS_DIR="$PROJECT_ROOT/libs"
OV_DIR="$LIBS_DIR/openvino"

# Create libs directory
mkdir -p "$LIBS_DIR"

# Download and extract OpenVINO if not present
if [ ! -d "$OV_DIR" ]; then
    echo "Downloading OpenVINO GenAI ${OV_VERSION}..."
    curl -L "$OV_URL" -o "$LIBS_DIR/openvino_genai.tar.gz"
    
    echo "Extracting..."
    tar -xzf "$LIBS_DIR/openvino_genai.tar.gz" -C "$LIBS_DIR"
    
    # Rename extracted folder
    EXTRACTED_DIR=$(find "$LIBS_DIR" -maxdepth 1 -type d -name "openvino_genai_ubuntu22*" | head -n 1)
    if [ -n "$EXTRACTED_DIR" ]; then
        mv "$EXTRACTED_DIR" "$OV_DIR"
    fi
    
    rm "$LIBS_DIR/openvino_genai.tar.gz"
    echo "OpenVINO installed to $OV_DIR"
else
    echo "OpenVINO already found at $OV_DIR"
fi

# Set environment variables
export OPENVINO_ROOT="$OV_DIR"
export LD_LIBRARY_PATH="$OPENVINO_ROOT/runtime/lib/intel64:$OPENVINO_ROOT/runtime/3rdparty/tbb/lib:$LD_LIBRARY_PATH"

echo "Building Capi (Linux)..."
cd "$PROJECT_ROOT"

# Install deps if needed (assume user has them or run npm install)
if [ ! -d "node_modules" ]; then
    echo "Installing frontend dependencies..."
    cd capi-ui && npm install && cd ..
fi

# Build Tauri App (deb, rpm)
# This will use the bundle.targets from tauri.conf.json
echo "Running Tauri Build..."
npm run tauri build

# Create Portable Tarball
echo "Creating portable tarball..."
RELEASE_DIR="$PROJECT_ROOT/target/release"
TARBALL_NAME="capi-linux-x64.tar.gz"
STAGING_DIR="$PROJECT_ROOT/temp_bundle_linux"

rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR/bin"
mkdir -p "$STAGING_DIR/lib"

# Copy Binary
cp "$RELEASE_DIR/capi-ui" "$STAGING_DIR/bin/capi"

# Copy OpenVINO Libs
cp "$OV_DIR"/runtime/lib/intel64/*.so* "$STAGING_DIR/lib/"
cp "$OV_DIR"/runtime/3rdparty/tbb/lib/*.so* "$STAGING_DIR/lib/"
cp "$OV_DIR"/runtime/bin/intel64/plugins.xml "$STAGING_DIR/bin/" 2>/dev/null || true

# Helper wrapper script
cat > "$STAGING_DIR/bin/capi-wrapper" << 'EOF'
#!/bin/sh
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
LIB_DIR="$(dirname "$SCRIPT_DIR")/lib"
export LD_LIBRARY_PATH="$LIB_DIR:$LD_LIBRARY_PATH"
exec "$SCRIPT_DIR/capi" "$@"
EOF
chmod +x "$STAGING_DIR/bin/capi-wrapper"

# Archive
cd "$STAGING_DIR"
tar -czf "$PROJECT_ROOT/$TARBALL_NAME" *
cd "$PROJECT_ROOT"
rm -rf "$STAGING_DIR"

echo "Build complete!"
echo "Artifacts:"
echo " - Portable: $TARBALL_NAME"
echo " - Deb/Rpm: target/release/bundle/"
