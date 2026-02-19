# Fotos

AI-powered screenshot capture, annotation, and analysis tool built with Tauri 2.

## Features

- Screenshot capture (fullscreen, region, window) across Linux Wayland/X11 and Windows
- Annotation tools: arrow, rectangle, ellipse, text, blur, step numbers, freehand, highlight, crop
- AI-powered OCR (Tesseract), PII auto-detection and blur, LLM vision analysis
- MCP server for AI agent integration (Claude Desktop, Cursor, etc.)
- Vanilla JS/HTML/CSS frontend with HTML5 Canvas — no web frameworks

## Development

```bash
# Prerequisites: Rust, Tauri CLI, system dependencies
cargo install tauri-cli

# Run in development mode
cargo tauri dev

# Build for release
cargo tauri build
```

## Architecture

- `src-tauri/` — Rust backend (capture, AI, file I/O, IPC)
- `src-mcp/` — MCP server binary (JSON-RPC 2.0 over stdio)
- `src-ui/` — Frontend (vanilla JS, HTML5 Canvas)

## License

Apache-2.0
