#!/usr/bin/env bash
# Build Fotos as a Flatpak package.
# Requires: flatpak-builder

set -euo pipefail

MANIFEST="flatpak/io.github.charly.fotos.yml"
BUILD_DIR=".flatpak-build"
REPO_DIR=".flatpak-repo"

echo "Building Fotos Flatpak..."
flatpak-builder --repo="$REPO_DIR" --force-clean "$BUILD_DIR" "$MANIFEST"

echo "Updating repository summary..."
flatpak build-update-repo "$REPO_DIR"

echo "Done. Install with:"
echo "  flatpak install --user file://\$PWD/$REPO_DIR io.github.charly.fotos"
