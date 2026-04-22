# Fotos — Agent Instructions

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
just setup-distrobox  # one-time: create distrobox + install deps
just setup-flatpak    # one-time: install Flatpak SDK runtimes
just gen-cargo-sources # regenerate flatpak/cargo-sources.json after dependency changes
```

If the fedora distrobox doesn't exist yet, run `just setup-distrobox` first.

**Flatpak setup (one-time):**
1. `just setup-flatpak` — installs GNOME SDK 48 and Rust extension runtimes from Flathub
2. `just gen-cargo-sources` — generates `flatpak/cargo-sources.json` from `Cargo.lock` so the Flatpak build can fetch crates offline

Run `just gen-cargo-sources` again whenever `Cargo.lock` changes (i.e. after adding/updating dependencies).

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

<!-- WAI:START -->
# Workflow Tools

This project uses **wai** to track the *why* behind decisions — research,
reasoning, and design choices that shaped the code. Run `wai status` first
to orient yourself.

Detected workflow tools:
- **wai** — research, reasoning, and design decisions
- **beads** — issue tracking (tasks, bugs, dependencies). CLI command: **`bd`** (not `beads`)
- **openspec** — specifications and change proposals (see `openspec/AGENTS.md`)

> **CRITICAL**: Apply TDD and Tidy First throughout — not just when writing code:
> - **Planning/task creation**: each ticket should map to a red→green→refactor cycle; refactoring tasks must be separate tickets from feature tasks.
> - **Design**: define the test shape (inputs/outputs) before designing the implementation.
> - **Implementation**: write the failing test first, then make it pass, then tidy in a separate commit.

> **When beginning research or creating a ticket**: run `wai search "<topic>"` to check for existing patterns before writing new content.

## Quick Start

1. `wai sync` — ensure agent tools are projected
2. `wai status` — see active projects, phase, and suggestions
3. `bd ready` — find available work items

When context reaches ~40%: stop and tell the user — responses degrade past
this point. Recommend `wai close` then `/clear` to resume cleanly.
Do NOT skip `wai close` — it enables resume detection.

## Detailed Instructions

Full workflow reference — session lifecycle, capturing work, command cheat
sheets, cross-tool sync, and PARA structure — lives in **`.wai/AGENTS.md`**.
Read it at the start of your first session or when you need detailed guidance.

Keep this managed block so `wai init` can refresh the instructions.

<!-- WAI:END -->

<!-- WAI:REFLECT:REF:START -->
## Accumulated Project Patterns

Project-specific conventions, gotchas, and architecture notes live in
`.wai/resources/reflections/`. Run `wai search "<topic>"` to retrieve relevant
context before starting research or creating tickets.

> **Before research or ticket creation**: always run `wai search "<topic>"` to
> check for known patterns. Do not rediscover what is already documented.
<!-- WAI:REFLECT:REF:END -->


## Landing the Plane (Session Completion)

**When ending a work session**, complete ALL steps below.

**Note:** This project uses ephemeral branches — code is merged to `main` locally, not pushed to a remote. For any beads follow-up beyond closing issues, run `bd` and use the commands your installed version offers.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** — create beads issues for anything that needs follow-up
2. **Run quality gates** (if code changed) — `just check`, `just lint`, `just test`, `just spec-validate`
3. **Update issue status** — close finished work (`bd close <id>`), update in-progress items
4. **Commit and review tracker state:**
   ```bash
   git status                  # review what changed
   git add <files>             # stage code changes
   git commit -m "..."         # commit code + beads state together
   ```
   If beads needs any extra follow-up beyond `bd close`, run `bd` and use the
   commands your installed version offers.
5. **Create a handoff** — `wai handoff create tracer-bullet` so the next session has context
6. **Verify** — `git status` shows a clean working tree

**CRITICAL RULES:**
- NEVER say "done" before committing your changes
- Do NOT use `git push` — this repo has no upstream remote; merges happen locally
- If you need beads follow-up beyond `bd close`, inspect the installed CLI with `bd`

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
