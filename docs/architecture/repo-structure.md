# Repository Structure

This repository is organized to support multiple kinds of tooling without forcing a single implementation path too early.

## Principles

- The repo is a framework host, not one monolithic app.
- Core behavior should stay domain-agnostic.
- Human-authored definitions should remain easy to inspect and review.
- Implementation detail should not leak into architectural boundaries.

## Top-Level Areas

- `crates/`
  Rust code for core logic, CLIs, libraries, and services.
- `packages/`
  TypeScript packages when a web UI or supporting JS tooling becomes necessary.
- `apps/`
  End-user applications such as a session explorer or local dashboard.
- `config/`
  Human-authored configuration templates, schemas, and defaults.
- `skills/`
  Reusable skills, prompts, workflow definitions, and helper metadata.
- `scripts/`
  Shell or PowerShell helpers where a compiled tool would be excessive.
- `docs/`
  Architecture notes, future ideas, and formal decisions.

## Directory Materialization

- Empty placeholder directories are not kept in git just to reserve names.
- Areas like `apps/`, `packages/`, and `skills/templates/` are created when first used.
- `.gitkeep` files are only used when an otherwise-empty tracked directory is currently required.

## Expected Boundary Style

- Rust-first for typed core logic.
- TypeScript second for frontend or UI-adjacent tooling.
- Shell scripts only for thin orchestration or local convenience.
- Storage choices remain open per subsystem; no single global database assumption is required at this stage.
