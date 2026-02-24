# Changelog

All notable changes to Fotos are documented here.

## [0.2.0] - 2026-02-24

### Added
- Clipboard support on Wayland via XDG portal
- Save As with native file-chooser dialog (XDG portal)
- Zoom controls and fit-to-page button in the viewer
- Toast notifications for copy, save, and save-as operations
- Tracing / structured logging (`RUST_LOG` env var)
- System tray via `libayatana-appindicator3` (bundled in Flatpak)
- Global capture shortcuts now work inside the Flatpak sandbox via XDG portal

### Fixed
- Canvas stale pixels after crop (`clearRect` was not accounting for the canvas transform)
- `crop_image` Rust command rejected float coordinates — now rounded before dispatch
- Clipboard write required a user-activation gesture — now triggered correctly
- Save As rejected portal-returned paths — path handling corrected
- Zoom rendering artifacts at non-integer scale factors
- Tesseract `tessdata` path resolution inside Flatpak
- `fetch()` on `data:` URLs caused a "TypeError: Load failed" in WebKitGTK — replaced with inline decode
- XDG portal screenshot required `interactive=true` to avoid silent failure in Flatpak
- App window was visible behind the screenshot selection overlay — now hidden during capture
- Flatpak IPC: added `connect-src` to CSP; guarded build script against uncommitted sources

### Infrastructure
- Migrated to `tesseract` crate; Leptonica and Tesseract now built as shared libs in Flatpak
- `libayatana-appindicator3` and its full dependency chain bundled in the Flatpak manifest
- `flatpak-cargo-generator` output regenerated for new `env-filter` feature flag
- Local Flatpak install uses a named remote and explicit repo summary

## [0.1.0] - 2025-01-01

Initial release.

- Screenshot capture (region and full-screen) via XDG portal
- Canvas-based annotation with crop tool
- OCR via Tesseract (text extraction from screenshots)
- Copy-to-clipboard and save operations
- Custom HTML title bar
- Flatpak packaging (`io.github.charly.fotos`)
- MCP server (`fotos-mcp`) for AI tool integration
