#!/bin/bash
set -e

# This script is a wrapper for the Makefile to ensure CI/CD consistency
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Building Capi via Makefile..."
make bundle

echo "Build complete!"
echo "Artifacts can be found in target/release/ and capi-ui/src-tauri/target/release/bundle/"

