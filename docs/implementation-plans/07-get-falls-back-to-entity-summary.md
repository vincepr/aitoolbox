# Plan 07: Use `entities.summary` When No Note Is Linked

## Issue reference
- `docs/issues/get-falls-back-to-entity-summary.md`

## Scope
- Ensure `knowledge-cli get` renders useful summary text from `entities.summary` when `note_refs` has no row.

## Files
- Modify: `crates/knowledge-core/src/store.rs`
- Modify: `crates/knowledge-cli/src/main.rs`
- Modify: `crates/knowledge-core/tests/exact_lookup.rs`
- Modify: `crates/knowledge-cli/tests/query_cli.rs`
- Modify: `docs/features/knowledge-system/README.md` (if output contract is documented there)

## Tasks
1. Extend the query/read path to load `entities.summary` alongside note-backed summary data.
2. Implement summary precedence for `get`:
   - note summary (from `note_refs`) first
   - fallback to non-empty `entities.summary`
   - else keep `No note summary stored`
3. Add explicit rendering marker for DB summary fallback (for example, `Summary:`) so source is clear.
4. Keep output deterministic and backward-compatible except where fallback now provides non-empty text.
5. Add unit/integration tests for:
   - exact match with no note but non-empty `entities.summary`
   - exact match with both note and `entities.summary` (note wins)
   - exact match with neither summary source (existing message remains)
6. Run verification baseline.

## Acceptance checks
- `knowledge-cli get <entity>` no longer prints `No note summary stored` when `entities.summary` is populated.
- Note-linked summaries still take precedence over column fallback.
- Tests cover all summary precedence branches.

