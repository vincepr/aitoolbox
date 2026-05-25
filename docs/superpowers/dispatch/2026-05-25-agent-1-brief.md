# Agent 1 Brief - Schema Migrations

Origin todos:
- `todos/reconciled/01-schema-migrations-and-version-gate.md`
- Sources: `todos/mnemosyne/02-schema-migrations-and-version-verification.md`, `todos/signetai/01-daemonized-knowledge-service.md`

Scope:
- Implement versioned migration ledger + verify gate.
- Add `knowledge-cli migrate` (`--verify`, `--dry-run`).
- No backward compatibility requirement; interfaces may change.
