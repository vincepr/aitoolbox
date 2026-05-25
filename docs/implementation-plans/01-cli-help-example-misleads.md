# Plan 01: Fix Misleading Canonical Name Examples

## Issue reference
- `docs/issues/01-cli-help-example-misleads.md`

## Scope
- Update CLI help text/examples to use real canonical shape (`kebab-lowercase-dash`).

## Files
- Modify: `crates/knowledge-cli/src/main.rs`
- Modify: `crates/knowledge-cli/tests/help.rs`

## Tasks
1. Update `get` argument help string example to `laika-marketplaces-jobs-pricestock`.
2. Update after-help usage examples to match canonical format.
3. Update help snapshot test expectations.
4. Run verification baseline.

## Acceptance checks
- `knowledge-cli --help` and `knowledge-cli get --help` no longer show dot-PascalCase examples.
- Help test snapshots pass.
