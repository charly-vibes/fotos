<!-- WAI:START -->
# Workflow Tools

This project uses **wai** to track the *why* behind decisions — research,
reasoning, and design choices that shaped the code. Run `wai status` first
to orient yourself.

Detected workflow tools:
- **wai** — research, reasoning, and design decisions
- **beads (bd)** — issue tracking (tasks, bugs, dependencies)
- **openspec** — specifications and change proposals (see `openspec/AGENTS.md`)

## When to Use What

| Need | Tool | Example |
|------|------|---------|
| Record reasoning/research | wai | `wai add research "findings"` |
| Capture design decisions | wai | `wai add design "architecture choice"` |
| Session context transfer | wai | `wai handoff create <project>` |
| Track work items/bugs | beads | `bd create --title="..." --type=task` |
| Find available work | beads | `bd ready` |
| Manage dependencies | beads | `bd dep add <blocked> <blocker>` |
| Propose system changes | openspec | Read `openspec/AGENTS.md` |
| Define requirements | openspec | `openspec validate --strict` |

Key distinction:
- **wai** = *why* decisions were made (reasoning, context, handoffs)
- **beads** = *what* needs to be done (concrete tasks, status tracking)
- **openspec** = *what the system should look like* (specs, requirements, proposals)

## Starting a Session

1. Run `wai status` to see active projects, current phase, and suggestions.
2. Run `bd ready` to find available work items.
3. Check `openspec list` for active change proposals.
4. Check the phase — it tells you what kind of work is expected:
   - **research** → gather information, explore options
   - **design** → make architectural decisions
   - **plan** → break work into tasks
   - **implement** → write code, guided by research/plans
   - **review** → validate against plans
   - **archive** → wrap up
5. Read existing artifacts with `wai search "<topic>"` before starting new work.

## Capturing Work

Record the reasoning behind your work, not just the output:

```bash
wai add research "findings"         # What you learned, trade-offs
wai add plan "approach"             # How you'll implement, why
wai add design "decisions"          # Architecture choices, rationale
wai add research --file notes.md    # Import longer content
```

Use `--project <name>` if multiple projects exist. Otherwise wai picks the first one.

Phases are a guide, not a gate. Use `wai phase show` / `wai phase next`.

## Ending a Session

1. Create a handoff: `wai handoff create <project>`
2. Update issue status: `bd close <id>` for completed work
3. File new issues for remaining work: `bd create --title="..."`
4. Commit your changes (handoff + code)

## Quick Reference

### wai
```bash
wai status                    # Project status and next steps
wai add research "notes"      # Add research artifact
wai add plan "plan"           # Add plan artifact
wai add design "design"       # Add design artifact
wai search "query"            # Search across artifacts
wai handoff create <project>  # Session handoff
wai phase show                # Current phase
wai doctor                    # Workspace health
```

### beads
```bash
bd ready                     # Available work
bd show <id>                 # Issue details
bd create --title="..."      # New issue
bd update <id> --status=in_progress
bd close <id>                # Complete work
```

### openspec
Read `openspec/AGENTS.md` for full instructions.
```bash
openspec list              # Active changes
openspec list --specs      # Capabilities
```

## Structure

The `.wai/` directory organizes artifacts using the PARA method:
- **projects/** — active work with phase tracking and dated artifacts
- **areas/** — ongoing responsibilities (no end date)
- **resources/** — reference material, agent configs, templates
- **archives/** — completed or inactive items

Do not edit `.wai/config.toml` directly. Use `wai` commands instead.

Keep this managed block so `wai init` can refresh the instructions.

<!-- WAI:END -->

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