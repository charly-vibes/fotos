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
just package          # build Flatpak
just install          # build + install Flatpak locally
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
# Wai — Workflow Context

This project uses **wai** to track the *why* behind decisions — research,
reasoning, and design choices that shaped the code. Run `wai status` first
to orient yourself.

## When Starting a Session

1. Run `wai status` to see active projects, current phase, and suggestions.
2. Check the phase — it tells you what kind of work is expected right now:
   - **research** → gather information, explore options, document findings
   - **design** → make architectural decisions, write design docs
   - **plan** → break work into tasks, define implementation order
   - **implement** → write code, guided by existing research/plans/designs
   - **review** → validate work against plans and designs
   - **archive** → wrap up, move to archives
3. Read existing artifacts with `wai search "<topic>"` before starting new work.

## Capturing Work

Record the reasoning behind your work, not just the output:

```bash
wai add research "findings"         # What you learned, options explored, trade-offs
wai add plan "approach"             # How you'll implement, in what order, why
wai add design "decisions"          # Architecture choices and rationale
wai add research --file notes.md    # Import longer content from a file
```

**What goes where:**
- **Research** = facts, explorations, comparisons, prior art, constraints discovered
- **Plans** = sequenced steps, task breakdowns, implementation strategies
- **Designs** = architectural decisions, component relationships, API shapes, trade-offs chosen

Use `--project <name>` if multiple projects exist. Otherwise wai picks the first one.

## Advancing Phases

Move the project forward when the current phase's work is done:

```bash
wai phase show          # Where are we now?
wai phase next          # Advance to next phase
wai phase set <phase>   # Jump to a specific phase (flexible, not enforced)
```

Phases are a guide, not a gate. Skip or go back as needed.

## When Ending a Session

Create a handoff so the next session (yours or someone else's) has context:

```bash
wai handoff create <project>
```

This generates a template with sections for: what was done, key decisions,
open questions, and next steps. Fill it in before stopping.

## Quick Reference

```bash
wai status                    # Project status and next steps
wai phase show                # Current project phase
wai new project "name"        # Create a new project
wai add research "notes"      # Add research notes
wai add plan "plan"           # Add a plan document
wai add design "design"       # Add a design document
wai search "query"            # Search across all artifacts
wai handoff create <project>  # Generate handoff document
wai sync                      # Sync agent configs
wai show                      # Overview of all items
wai timeline <project>        # Chronological view of artifacts
wai doctor                    # Check workspace health
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

## Landing the Plane (Session Completion)

**When ending a work session**, complete ALL steps below.

**Note:** This project uses ephemeral branches — code is merged to `main` locally, not pushed to a remote. The beads issue tracker lives in the repo and is synced via `bd sync --from-main`.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** — create beads issues for anything that needs follow-up
2. **Run quality gates** (if code changed) — `just check`, `just lint`, `just test`
3. **Update issue status** — close finished work (`bd close <id>`), update in-progress items
4. **Commit and sync:**
   ```bash
   git status                  # review what changed
   git add <files>             # stage code changes
   bd sync --from-main         # pull beads updates from main
   git commit -m "..."         # commit code + beads state together
   ```
5. **Create a handoff** — `wai handoff create tracer-bullet` so the next session has context
6. **Verify** — `git status` shows a clean working tree

**CRITICAL RULES:**
- NEVER say "done" before committing your changes
- Do NOT use `git push` — this repo has no upstream remote; merges happen locally
- Run `bd sync --from-main` before committing to avoid beads conflicts
