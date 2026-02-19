# Project Context

## Purpose
Fotos is an AI-first screenshot capture, annotation, and analysis tool built with Tauri 2. It enables users to capture screenshots, annotate them with drawing tools on an HTML5 Canvas, and run AI analysis (OCR, PII detection, LLM vision). It exposes an MCP server so AI agents can programmatically capture, annotate, and analyze screenshots.

App identifier: `io.github.charly.fotos`

## Tech Stack
- Tauri 2 (Rust backend + native webview)
- Rust (capture, OCR, PII detection, LLM calls, file I/O, MCP server, IPC)
- Vanilla JS / HTML / CSS with ES modules (no web frameworks)
- HTML5 Canvas (triple-layer annotation engine)
- MCP protocol (separate `fotos-mcp` binary, JSON-RPC 2.0 over stdio)

## Project Conventions

### Code Style
- No web frameworks — vanilla JS/HTML/CSS with ES modules
- Rust does the heavy lifting — capture, AI, file operations, IPC
- Frontend state via a simple event-emitter store (no Redux/Zustand)
- Plain serializable objects for annotation data (no class instances)
- CSS custom properties for theming, CSS Grid for layout, no CSS frameworks

### Architecture Patterns
- Two-process model: Tauri app (GUI + backend) + MCP server (stdio, stateless)
- IPC bridge between processes (Unix socket on Linux, named pipe on Windows)
- Command pattern for undo/redo (store deltas, not snapshots)
- Triple-layer canvas: base (screenshot), annotations (committed), active (preview)
- Platform abstraction for capture (xcap for X11/Windows, xdg-desktop-portal for Wayland)

### Testing Strategy
- Rust integration tests under `tests/rust/`
- JS unit tests under `tests/js/` (plain assertions, no framework)
- `cargo check` must pass for workspace

### Git Workflow
- Ephemeral branches, local merge to main (no push)
- Conventional-style commit messages
- Use beads for multi-session work tracking

## Domain Context
- Screenshots are captured via platform-specific backends and assigned UUIDs
- Annotations are plain objects with image-space coordinates (not screen coordinates)
- AI processing pipeline: OCR (Tesseract) → PII regex matching → LLM vision analysis
- MCP server delegates all work to the main app via IPC; can also operate standalone

## Important Constraints
- Cross-platform: Linux (Wayland GNOME/KDE, X11) and Windows
- Flatpak is the primary Linux distribution format
- API keys stored in OS keychain only (never in config files or localStorage)
- CSP restricts frontend to self-origin; no eval(), no inline scripts
- No web framework dependencies

## External Dependencies
- Tesseract OCR (bundled tessdata for English)
- LLM APIs: Anthropic Claude, OpenAI, Google Gemini, Ollama (local)
- xdg-desktop-portal (Linux Wayland screenshot capture)
- OS keychain (GNOME Keyring, KWallet, Windows Credential Manager)
