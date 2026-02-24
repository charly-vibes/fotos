# Fotos

AI-powered screenshot capture, annotation, and analysis tool built with Tauri 2.

## Features

- Screenshot capture (fullscreen, region) on Linux Wayland via XDG portal and X11
- Annotation tools: arrow, rectangle, ellipse, text, blur, step numbers, freehand, highlight, crop
- Zoom controls and fit-to-page for precise annotation work
- Copy to clipboard and Save / Save As with native file-chooser dialog (Wayland portal)
- Toast notifications for copy, save, and save-as outcomes
- AI-powered OCR (Tesseract), PII auto-detection and blur, LLM vision analysis
- MCP server (`fotos-mcp`) for AI agent integration (Claude Desktop, Cursor, etc.)
- System tray icon for quick capture access
- Structured logging via `RUST_LOG`
- Vanilla JS/HTML/CSS frontend with HTML5 Canvas — no web frameworks
- Flatpak packaging (`io.github.charly.fotos`)

## Installation

### Flatpak (recommended on Linux)

```bash
just setup-flatpak   # one-time: install GNOME SDK runtimes
just install         # build and install locally
```

## Development

This project builds inside a **fedora distrobox** (required on Bluefin/immutable Fedora).

```bash
# One-time setup
just setup-distrobox    # create distrobox + install build deps
just setup-flatpak      # install Flatpak SDK runtimes (for packaging)

# Daily workflow
just dev                # cargo tauri dev (hot-reload)
just check              # cargo check (both crates)
just build              # cargo build --release
just test               # cargo test
just lint               # clippy -D warnings
just fmt                # rustfmt

# Packaging
just package            # build Flatpak
just install            # build + install Flatpak locally
just gen-cargo-sources  # regenerate flatpak/cargo-sources.json (run after Cargo.lock changes)
```

See the `justfile` for all available recipes.

## Architecture

- `src-tauri/` — Rust backend (Tauri 2: capture, AI, file I/O, IPC, credentials, system tray)
- `src-mcp/` — MCP server binary (`fotos-mcp`, JSON-RPC 2.0 over stdio)
- `src-ui/` — Frontend (vanilla JS, HTML5 Canvas, ES modules)
- `openspec/specs/` — 9 capability specs defining all requirements
- `flatpak/` — Flatpak manifest (`io.github.charly.fotos`)

## License

Apache-2.0
