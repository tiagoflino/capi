#!/usr/bin/env bash
set -e

REPO_OWNER="yourusername"
REPO_NAME="capi"
INSTALL_DIR="$HOME/.capi"
BIN_DIR="$INSTALL_DIR/bin"
LIB_DIR="$INSTALL_DIR/lib"
VERSION="latest"
TARBALL_NAME="capi-linux-x64.tar.gz"

echo "Installing Capi..."

# Check Architecture
ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    echo "Error: This installer only supports x86_64 Linux."
    exit 1
fi

# Prepare Directory
mkdir -p "$INSTALL_DIR"
rm -rf "$INSTALL_DIR/tmp_extract"
mkdir -p "$INSTALL_DIR/tmp_extract"

# Determine Download URL
if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_url="https://github.com/$REPO_OWNER/$REPO_NAME/releases/latest/download/$TARBALL_NAME"
else
    DOWNLOAD_url="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$VERSION/$TARBALL_NAME"
fi

if [ -n "$LOCAL_PATH" ]; then
    echo "Installing from local file: $LOCAL_PATH"
    tar -xzf "$LOCAL_PATH" -C "$INSTALL_DIR/tmp_extract"
else
    echo "Downloading from $DOWNLOAD_url..."
    if command -v curl >/dev/null 2>&1; then
        curl -L "$DOWNLOAD_url" -o "$INSTALL_DIR/$TARBALL_NAME"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$INSTALL_DIR/$TARBALL_NAME" "$DOWNLOAD_url"
    else
        echo "Error: curl or wget required."
        exit 1
    fi
    tar -xzf "$INSTALL_DIR/$TARBALL_NAME" -C "$INSTALL_DIR/tmp_extract"
    rm "$INSTALL_DIR/$TARBALL_NAME"
fi

# Move files into place
# The tarball structure from build_linux.sh is:
# bin/capi
# bin/capi-wrapper
# lib/*.so
cp -r "$INSTALL_DIR/tmp_extract/bin" "$INSTALL_DIR/"
cp -r "$INSTALL_DIR/tmp_extract/lib" "$INSTALL_DIR/"
rm -rf "$INSTALL_DIR/tmp_extract"

echo "Installed to $INSTALL_DIR"

# Shell Configuration
SHELL_NAME=$(basename "$SHELL")
RC_FILE=""

case "$SHELL_NAME" in
    bash) RC_FILE="$HOME/.bashrc" ;;
    zsh)  RC_FILE="$HOME/.zshrc" ;;
    *)    echo "Warning: Unknown shell. Please add $BIN_DIR to your PATH manually." ;;
esac

if [ -n "$RC_FILE" ]; then
    if ! grep -q "$BIN_DIR" "$RC_FILE"; then
        echo "Adding $BIN_DIR to PATH in $RC_FILE..."
        echo "" >> "$RC_FILE"
        echo "# Capi" >> "$RC_FILE"
        echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$RC_FILE"
        echo "Run 'source $RC_FILE' or restart your terminal to use 'capi-wrapper' (or just 'capi' if you prefer)."
    else
        echo "$BIN_DIR is already in PATH."
    fi
fi

# Suggestion
echo "--------------------------------------------------"
echo "Installation complete!"
echo "To run the app:"
echo "  $BIN_DIR/capi-wrapper"
echo "--------------------------------------------------"
