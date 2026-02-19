# Fotos — Project Instructions

## Build Commands

This project builds inside a **fedora distrobox** (required on Bluefin/immutable Fedora).
All build recipes are in the `justfile`. Key commands:

```bash
just check          # cargo check (both crates)
just build          # cargo build --release
just dev            # cargo tauri dev
just lint           # clippy lints
just fmt            # rustfmt
just test           # cargo test
just spec-validate  # validate all OpenSpec specs
just setup-distrobox # one-time: create distrobox + install deps
```

If the fedora distrobox doesn't exist yet, run `just setup-distrobox` first.

## Architecture

- `src-tauri/` — Rust backend (Tauri 2 app: capture, AI, file I/O, IPC, credentials)
- `src-mcp/` — MCP server binary (`fotos-mcp`, JSON-RPC 2.0 over stdio)
- `src-ui/` — Frontend (vanilla JS, HTML5 Canvas, ES modules, no frameworks)
- `openspec/specs/` — 9 capability specs defining all requirements

## Conventions

- No web frameworks — vanilla JS/HTML/CSS only
- All annotation geometry stored in image coordinates (not screen coords)
- API keys stored in OS keychain (`keyring` crate, service `fotos`) — never in config/localStorage
- Command pattern for undo/redo (deltas, not snapshots)
- Tauri commands return `Result<T, String>` for IPC

<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->