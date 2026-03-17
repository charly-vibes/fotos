# Documentation Update Implementation Plan

**Date**: 2026-02-25

## Overview
Update project documentation to accurately reflect the current implementation status, provide clear instructions for the MCP server, and ensure consistency across developer-facing guides (`README.md`, `AGENTS.md`, `CLAUDE.md`).

## Current State
- `README.md` provides a good overview but doesn't explicitly state what is currently implemented vs. what is planned.
- `openspec/specs/` are comprehensive but describe the "target" state, which can be confusing for new developers seeing "Not yet implemented" errors in the code.
- MCP server functionality is partially implemented (Prompts only), but documentation (in `spec.md` archived) implies full functionality.
- `AGENTS.md` and `CLAUDE.md` have redundant blocks for "Workflow Tools" and "Build Commands".

## Desired End State
- `README.md` includes a "Current Implementation Status" and a "Roadmap" section.
- A new `docs/MCP.md` (or similar) provides detailed setup and usage instructions for the MCP server, clearly noting current limitations.
- `AGENTS.md` and `CLAUDE.md` are streamlined and consistent.
- `just spec-validate` is mentioned in the quality gate documentation.

## Phase 1: README.md Enhancement
- ✅ Add "Current Implementation Status" section.
- ✅ Add "Project Roadmap" section based on open beads issues.
- ✅ Update `just` command descriptions.
- ✅ Reference `docs/MCP.md` for AI agent integration.

## Phase 2: Detailed MCP Documentation
- ✅ Create/Update `docs/MCP.md`.
- ✅ Document how to launch `fotos-mcp`.
- ✅ Provide Claude Desktop configuration examples.
- ✅ List available prompts and explain their usage.
- ✅ Explicitly state that Tools and Resources are currently stubs.
- ✅ Add "Use Case Guides" with practical examples.

## Phase 3: Developer Guide Refinement
- ✅ Review `AGENTS.md` and `CLAUDE.md`.
- ✅ Consolidate common information or ensure they are properly cross-referenced.
- ✅ Ensure all mentioned tools (`wai`, `beads`, `openspec`) are correctly described.

## Verification
- ✅ Review updated files for clarity and accuracy.
- ✅ Verify all links and references are correct.
- ✅ Ensure `just spec-validate` passes.
