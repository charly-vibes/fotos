#!/usr/bin/env bash
# Build Fotos as a Flatpak package.
# Requires: flatpak-builder

set -euo pipefail

# Flatpak reads from the committed git state of the source branch.
# Abort if source files have uncommitted changes that would be silently excluded.
# Only checks directories that end up in the Flatpak build; ignores project
# tracking tools (.beads, .wai, .claude, etc.) and untracked files.
SOURCE_DIRS="src-tauri src-ui src-mcp flatpak"
DIRTY=$(git diff HEAD -- $SOURCE_DIRS 2>/dev/null; git diff --cached -- $SOURCE_DIRS 2>/dev/null)
if [ -n "$DIRTY" ]; then
    echo ""
    echo "ERROR: Uncommitted changes in source files."
    echo "The Flatpak manifest sources from the committed git state (branch: main),"
    echo "so these changes will NOT be included in the build."
    echo ""
    echo "Commit your changes first, then re-run this script."
    echo ""
    git status --short -- $SOURCE_DIRS
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
