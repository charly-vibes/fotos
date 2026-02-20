Smoke Test: Setup Dependencies Identified

During smoke test preparation, identified missing system dependencies in the fedora distrobox:

## Missing Dependencies (now installed):
1. mesa-libgbm-devel - Required for GBM (Generic Buffer Management) linking
2. tauri-cli - Cargo plugin for running Tauri dev server

## Installation:
```bash
distrobox enter fedora -- sudo dnf install -y mesa-libgbm-devel
distrobox enter fedora -- cargo install tauri-cli
```

## Recommendation:
Update justfile setup-distrobox recipe to include mesa-libgbm-devel in the dnf install list.

The setup-distrobox recipe already includes tauri-cli installation, but mesa-libgbm-devel is missing.
