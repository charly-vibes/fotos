# Change: Add GNOME Shell Extension Integration

## Why

Fotos runs as a Tauri app with a system tray icon, but GNOME Shell dropped the legacy system tray in GNOME 3.26+. Users on GNOME have no native panel entry point and no way to trigger captures from the desktop. Adding a thin GNOME Shell extension restores this integration while keeping Fotos fully cross-platform.

## What Changes

- **New: GNOME Shell extension** (`gnome-extension/`) — GJS panel indicator with popup menu, GNOME-native keybindings, and D-Bus calls into Fotos
- **New: D-Bus service in Fotos** (`src-tauri/src/dbus.rs`, Linux-only) — exposes `io.github.charly.Fotos` on the session bus so the extension can activate the window and trigger captures
- **New capability spec: `gnome-integration`** — requirements for the D-Bus interface, extension panel behavior, native shortcuts, and cold-launch flow
- **Flatpak manifest update** — adds `--own-name=io.github.charly.Fotos` permission
- **Justfile additions** — `gnome-schema`, `gnome-install`, `gnome-pack` recipes

## What Does NOT Change

- Tauri app architecture (still cross-platform)
- Windows behavior (D-Bus code is `#[cfg(target_os = "linux")]`, no Windows changes)
- Existing global shortcuts registered via Tauri plugin (still present; extension shortcuts supplement them on GNOME)
- MCP server, IPC socket, annotation engine — untouched
- CI workflows — no changes needed; `gnome-schema` is a local developer recipe only (`glib-compile-schemas` is not required in CI)

## Impact

- **Affected specs**: new capability `gnome-integration` (no existing spec modified)
- **Affected code**:
  - `src-tauri/src/lib.rs` — start D-Bus service in setup
  - `src-tauri/src/dbus.rs` — new file
  - `src-tauri/Cargo.toml` — ensure `zbus` has `tokio` feature
  - `flatpak/io.github.charly.fotos.yml` — add `--own-name` permission
  - `justfile` — new recipes
  - `gnome-extension/` — new top-level directory
