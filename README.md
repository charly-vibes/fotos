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
just spec-validate      # validate all OpenSpec specs
just spec-list          # list all OpenSpec capabilities

# Packaging
just package            # build Flatpak
just install            # build + install Flatpak locally
just gen-cargo-sources  # regenerate flatpak/cargo-sources.json (run after Cargo.lock changes)
```

See the `justfile` for all available recipes.

## Current Implementation Status

Fotos is currently in active development (**tracer-bullet** project, **implement** phase).

| Component | Status | Implementation Details |
|---|---|---|
| **Capture** | Partial | Fullscreen capture on Linux (Wayland portal & X11) and Windows (xcap). |
| **Annotation** | Partial | Canvas engine with zoom/pan and history (undo/redo). Selection tool supports move/resize. |
| **AI Features** | Partial | OCR (Tesseract) and PII detection/auto-blur implemented. LLM vision is in progress. |
| **MCP Server** | Partial | Prompts are fully implemented. Tools and Resources are currently stubs. |
| **UI Shell** | Stable | Toolbar, canvas layers, and basic panels are functional. |

See [docs/MCP.md](docs/MCP.md) for details on AI agent integration.

## Project Roadmap

Key features currently in the works (tracked via `beads`):

- **LLM Vision Integration**: Support for Claude, OpenAI, Gemini, and local Ollama.
- **Enhanced Capture**: Monitor and window enumeration and capture modes.
- **Full MCP Support**: Implement all 6 MCP tools and resources with an IPC bridge to the main app.
- **Annotation Tools**: Complete the implementation of all 9 planned drawing tools.
- **Theme System**: Full support for light, dark, and system themes via CSS custom properties.

## Architecture

- `src-tauri/` — Rust backend (Tauri 2: capture, AI, file I/O, IPC, credentials, system tray)
- `src-mcp/` — MCP server binary (`fotos-mcp`, JSON-RPC 2.0 over stdio)
- `src-ui/` — Frontend (vanilla JS, HTML5 Canvas, ES modules)
- `openspec/specs/` — 9 capability specs defining all requirements
- `flatpak/` — Flatpak manifest (`io.github.charly.fotos`)

## License

Apache-2.0
