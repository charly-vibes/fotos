#!/usr/bin/env bash
# Build Fotos as a Flatpak package.
# Requires: flatpak-builder

set -euo pipefail

# Flatpak reads from the committed git state of the source branch.
# Warn loudly if there are uncommitted changes so they don't get silently excluded.
if ! git diff --quiet HEAD 2>/dev/null || ! git diff --cached --quiet 2>/dev/null; then
    echo ""
    echo "ERROR: You have uncommitted changes."
    echo "The Flatpak manifest sources from the committed git state (branch: main),"
    echo "so any unstaged or staged-but-not-committed changes will NOT be included."
    echo ""
    echo "Commit your changes first, then re-run this script."
    echo ""
    git status --short
    echo ""
    exit 1
fi

MANIFEST="flatpak/io.github.charly.fotos.yml"
BUILD_DIR=".flatpak-build"
REPO_DIR=".flatpak-repo"

echo "Building Fotos Flatpak..."
flatpak-builder --repo="$REPO_DIR" --force-clean "$BUILD_DIR" "$MANIFEST"

echo "Updating repository summary..."
flatpak build-update-repo "$REPO_DIR"

echo "Done. Install with:"
echo "  flatpak install --user file://\$PWD/$REPO_DIR io.github.charly.fotos"
