# Changelog

All notable changes to Fotos are documented here.

## [Unreleased]

## [0.3.0] - 2026-02-25

### Added

- **LLM vision analysis** — analyze screenshots with Claude, OpenAI, Gemini, or a local Ollama model via the `analyze_image` command
- **Image compression** — compress captures before sending to LLM backends to reduce latency and cost
- **Settings modal** (⚙ button in toolbar): central place for configuration; open with the gear icon or `Escape` to close
- **API key management**: enter and store API keys for Anthropic (Claude), OpenAI (GPT-4o), and Google (Gemini) directly in the Settings modal; keys are stored in the OS keychain (GNOME Keyring, KWallet, Windows Credential Manager) via the `keyring` crate — never in config files, `localStorage`, or the Tauri store
  - **Show/Hide toggle** on each password field
  - **Test** button: validates the key with a lightweight authenticated request to the provider's models endpoint
  - **Delete** button: removes the key from the keychain immediately
  - Key status (masked last-4 characters, "No key set", "Connected", or error) shown inline
- **Monitor capture** (`mode: "monitor"`): capture a specific monitor by index via `xcap::Monitor::all()`
- **Window capture** (`mode: "window"`): capture a specific window by ID via `xcap::Window::all()`; minimized windows are rejected with a clear error
- `list_monitors` and `list_windows` Tauri commands returning id, name, position, size, and primary flag (monitors) or id, title, app name, and geometry (windows)
- **Full selection tool**: move and resize handles for placed annotations, with full undo/redo support
- **Color picker UI**: inline picker for annotation stroke and fill colors
- **JPEG and WebP export**: save images in JPEG or WebP format in addition to PNG
- **JSON annotation export/import**: round-trip annotation data as structured JSON
- **OCR bounding boxes**: Tesseract results rendered as overlay bounding boxes on the canvas
- **PII auto-blur**: automatically detect and blur personally identifiable information in screenshots
- **Embedded font for PNG export**: text annotations render correctly in exported PNGs without a system font
- **MCP server** (`fotos-mcp`): JSON-RPC 2.0 protocol core, prompt templates, and a Unix-socket IPC bridge to the main Fotos app for AI agent integration
- **GitHub Pages**: cargo doc published automatically on every push to `main`
- **Homebrew and Scoop install instructions** added to README

### Fixed

- Windows build: gate `ashpd` portal module and all call sites behind `#[cfg(target_os = "linux")]`; the crate is Linux-only and was causing the Windows CI job to fail with an unresolved module error
- CI: apply `cargo fmt --all` across the workspace to pass the formatting gate

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
