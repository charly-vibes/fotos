#!/usr/bin/env bash
# Development environment setup for Fotos.
# Run: ./scripts/dev.sh

set -euo pipefail

echo "Checking development dependencies..."

# Check Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo not found. Install via https://rustup.rs"
    exit 1
fi

# Check Tauri CLI
if ! cargo tauri --version &> /dev/null 2>&1; then
    echo "Installing Tauri CLI..."
    cargo install tauri-cli
fi

# Check system dependencies (Linux)
if [[ "$(uname)" == "Linux" ]]; then
    echo "Checking Linux system dependencies..."
    # Tauri 2 requires: webkit2gtk-4.1, libappindicator3, librsvg2
    # Tesseract OCR: tesseract, leptonica
fi

echo "Starting Tauri development server..."
cargo tauri dev
