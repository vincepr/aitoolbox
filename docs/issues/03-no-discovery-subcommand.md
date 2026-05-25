# 03 — No discovery subcommand: exact-lookup is unusable when canonical names are unknown

Status: `[planned]`
Effort: medium
Area: `crates/knowledge-cli`, `crates/knowledge-core`

## Symptom

`knowledge-cli` has only `get` (exact lookup). An agent that knows a partial name, a class name, or a `.NET` namespace has no supported way to find the canonical entity ID:

```console
$ knowledge-cli search PriceStock
error: unrecognized subcommand 'search'

$ knowledge-cli list --grep pricestock
error: unrecognized subcommand 'list'
```

The skill `SKILL.md` says "exact-first lookup, bounded fallback" — but the CLI offers no mechanism for the fallback. In practice agents either give up and decompile, or shell out to `sqlite3` directly (which the skill should not require).

## Root cause

`crates/knowledge-cli/src/main.rs:75-180` enumerates the available subcommands. There is no `search` / `list` / `grep` variant. The underlying `knowledge-core::store::KnowledgeStore` also exposes no public listing/filtering API beyond `find_primary_entity` (exact match) and `load_related_entities` (graph traversal from a known id).

## Proposed fix

Add a new subcommand `knowledge-cli list` with:

- `--grep <pattern>`: substring (or `LIKE '%pattern%'`) match against `canonical_name`, `namespace`, `package_name`, `repo_name`, and `aliases.alias`.
- `--kind <domain|system|library|project|lesson>`: optional filter.
- `--limit <n>`: default 20.
- Output: one entity per line, `<canonical_name>\t<kind>\t<repo_name>`. Suitable for piping to `head` / `fzf`.

Backed by a new `KnowledgeStore::list(pattern: Option<&str>, kind: Option<&str>, limit: u32) -> Result<Vec<EntityRecord>>` in `knowledge-core/src/store.rs` using the same indexes already declared in `crates/knowledge-core/src/schema.rs` (`idx_entities_canonical_name`, `idx_entities_namespace`, `idx_entities_package_name`, `idx_entities_repo_name`).

## Alternative considered

Add fuzzy/semantic matching to `get` directly (auto-fall-back when exact misses). Rejected for now: the skill contract is "exact-first with explicit fallback"; conflating the two would erase the agent's signal that it dropped from exact to fuzzy. A separate `list` subcommand keeps the boundary explicit.

## Acceptance criteria

- `knowledge-cli list --grep pricestock` returns the matching canonical names.
- `knowledge-cli list --kind library --limit 5` returns 5 library entities.
- Empty result set exits 0 with no output (not an error).
- Performance: `list --grep ...` against an index with 10k entities returns in <50ms on a warm cache.
- `crates/knowledge-cli/tests/query_cli.rs` covers grep-hit, grep-miss, kind-filter, limit.

## Related

- [05-skill-doesnt-teach-naming.md](05-skill-doesnt-teach-naming.md) — once `list` exists, the skill should describe the exact→list→get flow.
