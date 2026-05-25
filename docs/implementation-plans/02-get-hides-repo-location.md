# Plan 02: Expose Local and Git Location in `get`

## Issue reference
- `docs/issues/02-get-hides-repo-location.md`

## Scope
- Extend exact lookup response with optional location fields and print them in CLI/plain + structured output.

## Files
- Modify: `crates/knowledge-core/src/store.rs`
- Modify: `crates/knowledge-cli/src/main.rs`
- Modify: `crates/knowledge-core/tests/exact_lookup.rs`
- Modify: `crates/knowledge-cli/tests/query_cli.rs`

## Tasks
1. Add `EntityLocation` to core query model and attach `Option<EntityLocation>` to `QueryAnswer`.
2. Extend `query_exact` to fetch location row for matched entity.
3. Update CLI formatter to print `local:` and `git:` lines when present.
4. Extend structured output JSON to include nested location object.
5. Add/adjust tests for location present, partial, and absent cases.
6. Run verification baseline.

## Acceptance checks
- `knowledge-cli get <canonical>` shows location lines when stored.
- No extra blank/empty output when location is absent.
- Structured output includes location fields.
