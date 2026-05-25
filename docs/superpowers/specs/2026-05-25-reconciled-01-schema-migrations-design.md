# Reconciled 01: Schema Migrations and Version Gate Design
**DONE:** `2026-05-25-reconciled-01-schema-migrations-design.md`

**Goal:** Introduce a versioned, transactional migration system and strict startup compatibility checks.

## Scope
- Add `schema_migrations` ledger.
- Move schema setup from one bootstrap batch to ordered migrations.
- Add `knowledge-cli migrate` with `--verify` and `--dry-run`.
- Enforce startup fail-fast when schema is incompatible.

## Non-Goals
- Daemon/service behavior.
- Semantic retrieval changes.

## Requirements
1. Migrations must be idempotent and run in a single transaction per step.
2. Version checks must execute before mutation commands.
3. Existing database users can upgrade deterministically.

## Testing
- Unit tests for migration plan building and version comparison.
- Integration tests with historical schema snapshots.

## Done Criteria
- Fresh DB and upgraded DB have identical schema + migration rows.
- `knowledge-cli migrate --verify` exits non-zero on mismatch.
