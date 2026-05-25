# Plan 08: Add Related Children Output for Parent Matches

## Issue reference
- `docs/issues/get-returns-children-on-parent-match.md`

## Scope
- When `knowledge-cli get` exact-matches a parent-shaped entity (`domain`/`system`), return bounded related child candidates with stable ordering.

## Files
- Modify: `crates/knowledge-core/src/store.rs`
- Modify: `crates/knowledge-core/src/migrations.rs` (only if additional indexes are needed)
- Modify: `crates/knowledge-cli/src/main.rs`
- Modify: `crates/knowledge-core/tests/exact_lookup.rs`
- Modify: `crates/knowledge-cli/tests/query_cli.rs`
- Modify: `skills/core/knowledge-get/SKILL.md`
- Modify: `plugins/aitoolbox-knowledge-skills/skills/knowledge-get/SKILL.md`

## Tasks
1. Add a store-level related-children query API with limit support (default 10).
2. Implement child candidate strategy in this order:
   - explicit `relationships` edges when present
   - canonical prefix-extension heuristic (`parent-*`, relevant system prefix variants)
   - optional namespace/package-root heuristic if required for recall
3. Implement ordering priority:
   - entities with known notes first
   - then by `kind` rank (`lesson` > `library` > `system` > others)
   - then canonical name for stability
4. Add CLI rendering block after primary `get` output:
   - `Related (<shown> of <total>):`
   - include `id`, `canonical_name`, `kind`, and note availability marker
5. Add `--related-limit` argument to `get` (default 10, bounded parser).
6. Ensure `Top matches` remains separate from `Related` so similarity ranking and hierarchy are not conflated.
7. Add tests for:
   - parent match with related children returned and ordered
   - no related children path (block omitted or explicit empty message)
   - `--related-limit` override behavior
8. Update skill docs to teach `Related` usage as the default next-step navigation path after parent hits.
9. Run verification baseline.

## Acceptance checks
- Parent `get` responses include a bounded, ordered related-children block.
- `Related` output is deterministic and distinct from `Top matches`.
- Callers can choose next query directly from printed child canonical names.

