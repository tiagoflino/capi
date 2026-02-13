#!/usr/bin/env bash
set -e

REPO_OWNER="tiagoflino"
REPO_NAME="capi"
INSTALL_DIR="$HOME/.capi"
BIN_DIR="$INSTALL_DIR/bin"
LIB_DIR="$BIN_DIR/lib"
VERSION="latest"
TARBALL_NAME="capi-linux-x64.tar.gz"

echo "Installing Capi..."

ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    echo "Error: This installer only supports x86_64 Linux."
    exit 1
fi

mkdir -p "$INSTALL_DIR"
rm -rf "$INSTALL_DIR/tmp_extract"
mkdir -p "$INSTALL_DIR/tmp_extract"

if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/latest/download/$TARBALL_NAME"
else
    DOWNLOAD_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$VERSION/$TARBALL_NAME"
fi

if [ -n "$LOCAL_PATH" ]; then
    echo "Installing from local file: $LOCAL_PATH"
    tar -xzf "$LOCAL_PATH" -C "$INSTALL_DIR/tmp_extract"
else
    echo "Downloading from $DOWNLOAD_URL..."
    if command -v curl >/dev/null 2>&1; then
        curl -L "$DOWNLOAD_URL" -o "$INSTALL_DIR/$TARBALL_NAME"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$INSTALL_DIR/$TARBALL_NAME" "$DOWNLOAD_URL"
    else
        echo "Error: curl or wget required."
        exit 1
    fi
    tar -xzf "$INSTALL_DIR/$TARBALL_NAME" -C "$INSTALL_DIR/tmp_extract"
    rm "$INSTALL_DIR/$TARBALL_NAME"
fi

SOURCE_DIR="$INSTALL_DIR/tmp_extract/capi-linux-x64"

mkdir -p "$BIN_DIR"
cp "$SOURCE_DIR/bin/capi" "$BIN_DIR/"
cp "$SOURCE_DIR/bin/capi-engine" "$BIN_DIR/"
cp "$SOURCE_DIR/bin/capi-server" "$BIN_DIR/"
cp "$SOURCE_DIR/bin/capi-ui" "$BIN_DIR/" 2>/dev/null || true
mkdir -p "$LIB_DIR"
cp "$SOURCE_DIR/bin/lib/"* "$LIB_DIR/"

rm -rf "$INSTALL_DIR/tmp_extract"

echo "Installed to $INSTALL_DIR"

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
        echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$RC_FILE"
        echo "Run 'source $RC_FILE' or restart your terminal."
    else
        echo "$BIN_DIR is already in PATH."
    fi
fi

echo "--------------------------------------------------"
echo "Installation complete!"
echo "Run 'capi serve' to start the server."
echo "Run 'capi run <model>' for interactive chat."
echo "--------------------------------------------------"
