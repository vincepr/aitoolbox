# Plan 03: Add `knowledge-cli list` Discovery Subcommand

## Issue reference
- `docs/issues/03-no-discovery-subcommand.md`

## Scope
- Add explicit discovery subcommand and backing store query API.

## Files
- Modify: `crates/knowledge-cli/src/main.rs`
- Modify: `crates/knowledge-core/src/store.rs`
- Modify: `crates/knowledge-core/src/schema.rs` (only if query/index support needs adjustment)
- Modify: `crates/knowledge-cli/tests/query_cli.rs`

## Tasks
1. Add CLI subcommand `list` with `--grep`, `--kind`, and `--limit` args.
2. Add core `KnowledgeStore::list(pattern, kind, limit)` returning display record type.
3. Implement SQL filter against canonical, namespace, package, repo, and aliases.
4. Ensure result format: `<canonical_name>\t<kind>\t<repo_name>`.
5. Keep empty result as exit 0 with no output.
6. Add CLI tests for grep-hit, grep-miss, kind-filter, and limit.
7. Run verification baseline.

## Acceptance checks
- `knowledge-cli list --grep pricestock` returns matching entities.
- `knowledge-cli list --kind library --limit 5` returns at most five rows.
