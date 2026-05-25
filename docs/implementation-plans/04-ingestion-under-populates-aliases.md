# Plan 04: Populate Namespace, Package, and Alias Data During Ingestion

## Issue reference
- `docs/issues/04-ingestion-under-populates-aliases.md`

## Scope
- Fill `namespace`, `package_name`, and deterministic alias set during ingestion.
- Add configurable prefix mapping (e.g. `laika -> Relaxdays.Laika`) from source config.

## Files
- Modify: `crates/knowledge-core/src/ingest.rs`
- Modify: `crates/knowledge-core/src/store.rs` (case-insensitive alias lookup normalization if needed)
- Modify: `crates/knowledge-core/tests/import_sources.rs`
- Modify: `config/knowledge/sources.example.json`

## Tasks
1. Add per-source prefix mapping field to ingestion config model.
2. Implement namespace derivation from canonical name + mapping.
3. Set `package_name = namespace` for mapped entities.
4. Generate required aliases per entity (full namespace, path form, no-org prefix, bare repo).
5. Normalize alias matching case-insensitively (store/query normalization strategy).
6. Make ingest idempotent so re-runs do not duplicate aliases.
7. Add tests for derivation, mapping config, deduplication, and case-insensitive matching.
8. Run verification baseline.

## Acceptance checks
- Re-ingestion populates namespace/package and alias rows for target entities.
- Natural query variants resolve via `get`.
- Re-running `init` does not duplicate alias entries.
