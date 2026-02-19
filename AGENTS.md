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

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
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
